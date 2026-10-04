//! Reproducible sequential corpus run, excluding startup and report serialization.
use clap::Parser;
use futures::{FutureExt, StreamExt, stream};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    panic::AssertUnwindSafe,
    path::PathBuf,
    time::{Duration, Instant},
};
use vatprc_uniapi::modules::{
    flight::flight_plan::v2,
    navdata::{models::AnyFix, service::NavdataService},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "data/routes.csv")]
    csv: PathBuf,
    #[arg(long, default_value = "data/navdata.db?mode=ro")]
    navdata: String,
    #[arg(long, default_value = "/private/tmp/route-v2-corpus.json")]
    output: PathBuf,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long, default_value_t = 100)]
    stage_samples: usize,
    /// Count unknown fixes or connecting legs as successful recovery results.
    #[arg(long, alias = "allow-unknown-fixes")]
    allow_unresolved: bool,
}

#[derive(Deserialize, Serialize)]
struct Row {
    #[serde(rename = "Dep")]
    departure: String,
    #[serde(rename = "Arr")]
    arrival: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Route")]
    route: String,
}

impl Row {
    fn text(&self) -> String {
        format!(
            "{} {} {}",
            self.departure.trim().to_ascii_uppercase(),
            self.route.trim(),
            self.arrival.trim().to_ascii_uppercase()
        )
    }
}

#[derive(Serialize)]
struct Measurement {
    csv_line: usize,
    input: Row,
    milliseconds: f64,
    segments: usize,
    unknown_fixes: usize,
    unknown_identifiers: Vec<String>,
    unknown_connections: Vec<String>,
    error: Option<String>,
}

async fn measure(
    navdata: &NavdataService,
    row: Row,
    index: usize,
    known_connections: &HashSet<String>,
    allow_unresolved: bool,
) -> Measurement {
    let text = row.text();
    let start = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        AssertUnwindSafe(v2::parse_route(navdata, &text)).catch_unwind(),
    )
    .await;
    let milliseconds = start.elapsed().as_secs_f64() * 1000.;
    let (segments, unknown_identifiers, unknown_connections, error) = match result {
        Ok(Ok(Ok(legs))) => {
            let unknown: HashSet<_> = legs
                .iter()
                .flat_map(|leg| [&leg.from, &leg.to])
                .filter_map(|fix| match fix {
                    AnyFix::Unknown(ident) => Some(ident),
                    _ => None,
                })
                .collect();
            let continuous = legs.windows(2).all(|pair| pair[0].to == pair[1].from);
            let endpoints = legs.first().and_then(|leg| leg.from.identifier())
                == Some(row.departure.trim().to_ascii_uppercase().as_str())
                && legs.last().and_then(|leg| leg.to.identifier())
                    == Some(row.arrival.trim().to_ascii_uppercase().as_str());
            let unknown_identifiers: Vec<_> = unknown.into_iter().cloned().sorted().collect();
            let unknown_connections: Vec<_> = legs
                .iter()
                .filter_map(|leg| leg.identifier.as_ref())
                .filter(|identifier| !known_connections.contains(*identifier))
                .cloned()
                .sorted()
                .dedup()
                .collect();
            let unresolved: Vec<_> = unknown_identifiers
                .iter()
                .chain(&unknown_connections)
                .sorted()
                .dedup()
                .collect();
            let error = if !continuous {
                Some("disconnected output".to_owned())
            } else if !endpoints {
                Some("incorrect departure/arrival endpoints".to_owned())
            } else if !allow_unresolved && !unresolved.is_empty() {
                Some(format!(
                    "unresolved identifiers: {}",
                    unresolved.iter().join(", ")
                ))
            } else {
                None
            };
            (legs.len(), unknown_identifiers, unknown_connections, error)
        }
        Ok(Ok(Err(error))) => (0, Vec::new(), Vec::new(), Some(error.to_string())),
        Ok(Err(panic)) => (
            0,
            Vec::new(),
            Vec::new(),
            Some(format!(
                "panic: {}",
                panic
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown panic")
            )),
        ),
        Err(_) => (
            0,
            Vec::new(),
            Vec::new(),
            Some("timeout after 30s".to_owned()),
        ),
    };
    Measurement {
        csv_line: index + 2,
        input: row,
        milliseconds,
        segments,
        unknown_fixes: unknown_identifiers.len(),
        unknown_identifiers,
        unknown_connections,
        error,
    }
}

#[derive(Serialize)]
struct Stages {
    lexer_parser_ms: f64,
    resolver_ms: f64,
    solver_ms: f64,
    constructor_ms: f64,
    expander_ms: f64,
}

async fn stages(navdata: &NavdataService, row: &Row) -> Option<Stages> {
    let text = row.text();
    let start = Instant::now();
    let parsed: Vec<_> = v2::Parser::new(v2::Lexer::new(&text).parse_all().collect())
        .parse()
        .collect();
    let lexer_parser_ms = start.elapsed().as_secs_f64() * 1000.;
    if parsed.is_empty() || parsed.iter().any(|ident| !ident.errors.is_empty()) {
        return None;
    }
    let start = Instant::now();
    let candidates = v2::CandidateResolver::new(parsed)
        .resolve_candidates(navdata)
        .await
        .ok()?
        .collect();
    let resolver_ms = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let solved = v2::Solver::new(candidates, navdata)
        .solve()
        .into_iter()
        .collect();
    let solver_ms = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let constructed = v2::Constructor::new(solved).construct();
    let constructor_ms = start.elapsed().as_secs_f64() * 1000.;
    if constructed.is_empty() {
        return None;
    }
    let start = Instant::now();
    v2::Expander::new(constructed).expand(navdata).await.ok()?;
    Some(Stages {
        lexer_parser_ms,
        resolver_ms,
        solver_ms,
        constructor_ms,
        expander_ms: start.elapsed().as_secs_f64() * 1000.,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Panics are caught per row and recorded, so one failing route cannot abort the corpus.
    std::panic::set_hook(Box::new(|_| {}));
    let rows: Vec<Row> = csv::Reader::from_path(&args.csv)?
        .into_deserialize()
        .take(args.limit.unwrap_or(usize::MAX))
        .collect::<Result<_, _>>()?;
    let count = rows.len();
    let navdata = NavdataService::with_preferred_routes_path(&args.navdata, &args.csv).await?;
    // Load this before timing; checking output names adds no database queries to
    // the per-route pipeline being measured.
    let known_connections: HashSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT route_identifier FROM tbl_er_enroute_airways
         UNION SELECT procedure_identifier FROM tbl_pd_sids
         UNION SELECT procedure_identifier FROM tbl_pe_stars",
    )
    .fetch_all(&navdata.db)
    .await?
    .into_iter()
    .collect();
    let start = Instant::now();
    let measurements: Vec<_> = stream::iter(rows.into_iter().enumerate())
        .then(|(index, row)| {
            let navdata = &navdata;
            let known_connections = &known_connections;
            async move {
                let measured = measure(
                    navdata,
                    row,
                    index,
                    known_connections,
                    args.allow_unresolved,
                )
                .await;
                if (index + 1) % 250 == 0 {
                    eprintln!(
                        "{}/{} rows, {:.1}s elapsed",
                        index + 1,
                        count,
                        start.elapsed().as_secs_f64()
                    );
                }
                measured
            }
        })
        .collect()
        .await;
    let wall_seconds = start.elapsed().as_secs_f64();
    let sample_every = count.div_ceil(args.stage_samples.max(1)).max(1);
    let samples: Vec<_> = stream::iter(
        measurements
            .iter()
            .step_by(sample_every)
            .take(args.stage_samples)
            .filter(|row| row.error.is_none()),
    )
    .then(|row| stages(&navdata, &row.input))
    .filter_map(|stage| async { stage })
    .collect()
    .await;
    let successes = measurements
        .iter()
        .filter(|row| row.error.is_none())
        .count();
    let sorted: Vec<_> = {
        use itertools::Itertools;
        measurements
            .iter()
            .map(|row| row.milliseconds)
            .sorted_by(f64::total_cmp)
            .collect()
    };
    let percentile = |p: f64| {
        sorted
            .get(((sorted.len().saturating_sub(1)) as f64 * p).round() as usize)
            .copied()
            .unwrap_or_default()
    };
    let mean = sorted.iter().sum::<f64>() / sorted.len().max(1) as f64;
    let stage_mean = |field: fn(&Stages) -> f64| {
        samples.iter().map(field).sum::<f64>() / samples.len().max(1) as f64
    };
    let summary = serde_json::json!({
        "csv": args.csv, "navdata": args.navdata, "debug_assertions": cfg!(debug_assertions),
        "allow_unresolved": args.allow_unresolved,
        "routes": count, "successes": successes, "failures": count - successes,
        "routes_with_unknown_fixes": measurements.iter().filter(|row| row.unknown_fixes > 0).count(),
        "routes_with_unknown_connections": measurements.iter().filter(|row| !row.unknown_connections.is_empty()).count(),
        "wall_seconds": wall_seconds, "routes_per_second": count as f64 / wall_seconds,
        "mean_ms": mean, "median_ms": percentile(0.5), "p95_ms": percentile(0.95), "p99_ms": percentile(0.99), "max_ms": percentile(1.),
        "stage_samples": samples.len(),
        "stage_mean_ms": { "lexer_parser": stage_mean(|s| s.lexer_parser_ms), "resolver": stage_mean(|s| s.resolver_ms), "solver": stage_mean(|s| s.solver_ms), "constructor": stage_mean(|s| s.constructor_ms), "expander": stage_mean(|s| s.expander_ms) },
    });
    std::fs::write(
        &args.output,
        serde_json::to_vec_pretty(
            &serde_json::json!({ "summary": summary, "measurements": measurements, "stage_measurements": samples }),
        )?,
    )?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    if successes != count {
        anyhow::bail!(
            "{} routes failed; see {}",
            count - successes,
            args.output.display()
        );
    }
    Ok(())
}
