use std::{cell::Cell, collections::BTreeSet, panic::AssertUnwindSafe, time::Instant};
use futures::{FutureExt, StreamExt, stream};
use vatprc_uniapi::modules::{
    flight::flight_plan::v2::{self, RouteParseStep},
    navdata::{models::Fix, service::NavdataService},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let csv_path = &args[1];
    let db_path = &args[2];
    let report = &args[3];
    let limit = args.get(4).map(|s| s.parse::<usize>()).transpose()?.unwrap_or(usize::MAX);
    let rows: Vec<_> = csv::Reader::from_path(csv_path)?.into_records().take(limit).collect::<Result<_, _>>()?;
    let init_start = Instant::now();
    let navdata = NavdataService::with_preferred_routes_path(format!("{db_path}?mode=ro"), csv_path).await?;
    eprintln!("Initialized {} rows in {:.3}s; DB={db_path}", rows.len(), init_start.elapsed().as_secs_f64());
    let started = Instant::now();
    let results: Vec<_> = stream::iter(rows.into_iter().enumerate()).then(|(index, row)| {
        let navdata = &navdata;
        async move {
            let route = format!("{} {} {}", row.get(0).unwrap_or("").trim().to_ascii_uppercase(), row.get(6).unwrap_or("").trim(), row.get(1).unwrap_or("").trim().to_ascii_uppercase());
            let begin = Instant::now();
            let stages = Cell::new((begin, 0usize, [0f64; 6]));
            let result = AssertUnwindSafe(v2::parse_route_with_observer(navdata, &route, |step| {
                let stage = match step {
                    RouteParseStep::Lexed(_) => 0,
                    RouteParseStep::Parsed(_) => 1,
                    RouteParseStep::Candidates(_) => 2,
                    RouteParseStep::Solved(_) => 3,
                    RouteParseStep::Constructed(_) => 4,
                    RouteParseStep::Expanded(_) => 5,
                };
                let now = Instant::now();
                let (previous, _, durations) = stages.get();
                stages.set((now, stage + 1, std::array::from_fn(|i| if i == stage { now.duration_since(previous).as_secs_f64() * 1e6 } else { durations[i] })));
            })).catch_unwind().await;
            let elapsed = begin.elapsed().as_secs_f64() * 1e6;
            let (previous, next, durations) = stages.get();
            let durations: [f64; 6] = std::array::from_fn(|i| if i == next { previous.elapsed().as_secs_f64() * 1e6 } else { durations[i] });
            let (status, error, segments, unknown_fixes, unknown_legs, invalid_coordinates) = match result {
                Ok(Ok(legs)) => (
                    "ok", String::new(), legs.len(),
                    legs.iter().flat_map(|l| [&l.from, &l.to]).filter(|f| f.is_unknown()).map(|f| f.identifier().unwrap_or("<geo>").to_owned()).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>().join(","),
                    legs.iter().filter(|l| l.is_unknown).map(|l| l.identifier.as_deref().unwrap_or("DCT").to_owned()).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>().join(","),
                    legs.iter().flat_map(|l| [&l.from, &l.to]).filter(|f| !f.is_unknown() && (f.position().is_none() || !(-90. ..=90.).contains(&f.latitude()) || !(-180. ..=180.).contains(&f.longitude()))).count(),
                ),
                Ok(Err(error)) => ("error", error.to_string(), 0, String::new(), String::new(), 0),
                Err(error) => ("panic", error.downcast_ref::<String>().cloned().or_else(|| error.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "non-string panic".to_owned()), 0, String::new(), String::new(), 0),
            };
            if (index+1)%500 == 0 || index < 5 {
                eprintln!("Progress {} rows {:.3}s (last {:.3}ms; {status})",index+1,started.elapsed().as_secs_f64(),elapsed/1000.);
            }
            let clean = |s: &str| s.replace(['\t','\n','\r'], " ");
            format!("{}\t{}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",index+2,status,clean(&error),elapsed,durations.iter().map(|v|format!("{v:.3}")).collect::<Vec<_>>().join("\t"),next,segments,unknown_fixes,unknown_legs,invalid_coordinates,clean(row.get(2).unwrap_or("")),clean(&route))
        }
    }).collect().await;
    let wall = started.elapsed().as_secs_f64();
    let header = "row\tstatus\terror\ttotal_us\tlexer_us\tparser_us\tresolver_us\tsolver_us\tconstructor_us\texpander_us\tcompleted_stages\tsegments\tunknown_fixes\tunknown_legs\tinvalid_coordinates\tname\troute\n";
    std::fs::write(report, format!("{header}{}\n", results.join("\n")))?;
    eprintln!("Complete {} rows in {wall:.3}s; report={report}",results.len());
    Ok(())
}
