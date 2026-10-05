use std::{cell::Cell, collections::BTreeSet, panic::AssertUnwindSafe, time::Instant};
use futures::{FutureExt, StreamExt, stream};
use vatprc_uniapi::modules::{
    flight::flight_plan::v2::{self, Expander, RouteParseStep},
    navdata::{models::{AnyFix, Fix, NavProc}, service::NavdataService},
};

fn same_fix(left: &AnyFix, right: &AnyFix) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
        && left.identifier() == right.identifier()
        && !left.icao_code().zip(right.icao_code()).is_some_and(|(a, b)| !a.is_empty() && !b.is_empty() && a != b)
        && (left.latitude() - right.latitude()).abs() <= 1. / 3600.
        && (left.longitude() - right.longitude()).abs() <= 1. / 3600.
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let csv_path = &args[1];
    let db_path = &args[2];
    let report = &args[3];
    let limit = args.get(4).map(|s| s.parse::<usize>()).transpose()?.unwrap_or(usize::MAX);
    let check_connections = args.get(5).is_some_and(|s| s == "check-connections");
    let rows: Vec<_> = csv::Reader::from_path(csv_path)?.into_records().take(limit).collect::<Result<_, _>>()?;
    let init_start = Instant::now();
    let navdata = NavdataService::with_preferred_routes_path(format!("{db_path}?mode=ro"), csv_path).await?;
    eprintln!("Initialized {} rows in {:.3}s; DB={db_path}", rows.len(), init_start.elapsed().as_secs_f64());
    let check = v2::parse_route(&navdata, "ZBAA ELKUR W40 YQG ZSPD").await?;
    let w40: Vec<_> = check.iter().filter(|leg| leg.identifier.as_deref() == Some("W40")).collect();
    anyhow::ensure!(w40.len() == 6, "ELKUR W40 YQG did not expand to six segments");
    anyhow::ensure!(w40.first().and_then(|leg| leg.from.identifier()) == Some("ELKUR"), "incorrect W40 entry");
    anyhow::ensure!(w40.last().and_then(|leg| leg.to.identifier()) == Some("YQG"), "incorrect W40 exit");
    eprintln!("Real navdata regression: ELKUR W40 YQG expanded into six connected segments.");
    let started = Instant::now();
    let results: Vec<_> = stream::iter(rows.into_iter().enumerate()).then(|(index, row)| {
        let navdata = &navdata;
        async move {
            let route = format!("{} {} {}", row.get(0).unwrap_or("").trim().to_ascii_uppercase(), row.get(6).unwrap_or("").trim(), row.get(1).unwrap_or("").trim().to_ascii_uppercase());
            let begin = Instant::now();
            let stages = Cell::new((begin, 0usize, [0f64; 6]));
            let unexpanded = Cell::new(String::new());
            let result = AssertUnwindSafe(v2::parse_route_with_observer(navdata, &route, |step| {
                if check_connections && let RouteParseStep::Constructed(legs) = &step {
                    let failures = legs.iter().filter(|entry| {
                        let Some(NavProc::Airway(procedure)) = &entry.procedure else { return false; };
                        if entry.leg.from.is_unknown() || entry.leg.to.is_unknown() || same_fix(&entry.leg.from, &entry.leg.to) { return false; }
                        let has_direct_segment = procedure.legs.iter().any(|edge| (same_fix(&edge.from, &entry.leg.from) && same_fix(&edge.to, &entry.leg.to)) || (same_fix(&edge.to, &entry.leg.from) && same_fix(&edge.from, &entry.leg.to)));
                        !has_direct_segment && Expander::new(vec![(*entry).clone()]).expand().is_ok_and(|expanded| expanded == [entry.leg.clone()])
                    }).map(|entry| format!("{}:{}->{}",entry.leg.identifier.as_deref().unwrap_or("?"),entry.leg.from.identifier().unwrap_or("<geo>"),entry.leg.to.identifier().unwrap_or("<geo>"))).collect::<Vec<_>>().join(",");
                    unexpanded.set(failures);
                }
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
            let output_error = match &result {
                Ok(Ok(legs)) if legs.is_empty() => "empty output".to_owned(),
                Ok(Ok(legs)) if !legs.windows(2).all(|pair| pair[0].to == pair[1].from) => "disconnected output".to_owned(),
                Ok(Ok(legs)) if legs.first().and_then(|leg| leg.from.identifier()) != Some(row.get(0).unwrap_or("").trim().to_ascii_uppercase().as_str()) || legs.last().and_then(|leg| leg.to.identifier()) != Some(row.get(1).unwrap_or("").trim().to_ascii_uppercase().as_str()) => "incorrect departure/arrival endpoints".to_owned(),
                _ => String::new(),
            };
            let (status, error, segments, unknown_fixes, unknown_legs, invalid_coordinates) = match result {
                Ok(Ok(legs)) => (
                    if output_error.is_empty() {"ok"} else {"output_error"}, output_error, legs.len(),
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
            format!("{}\t{}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",index+2,status,clean(&error),elapsed,durations.iter().map(|v|format!("{v:.3}")).collect::<Vec<_>>().join("\t"),next,segments,unknown_fixes,unknown_legs,invalid_coordinates,clean(row.get(2).unwrap_or("")),clean(&route),unexpanded.take())
        }
    }).collect().await;
    let wall = started.elapsed().as_secs_f64();
    let header = "row\tstatus\terror\ttotal_us\tlexer_us\tparser_us\tresolver_us\tsolver_us\tconstructor_us\texpander_us\tcompleted_stages\tsegments\tunknown_fixes\tunknown_legs\tinvalid_coordinates\tname\troute\tunexpanded_airways\n";
    std::fs::write(report, format!("{header}{}\n", results.join("\n")))?;
    eprintln!("Complete {} rows in {wall:.3}s; report={report}",results.len());
    Ok(())
}
