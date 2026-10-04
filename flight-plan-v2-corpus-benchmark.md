# Flight plan v2 corpus benchmark

This records the initial cycle 2609 run, which permitted unknown-fix recovery.
The runner now rejects unknown fixes and unknown connections by default. See the
[latest cycle 2610 audit](flight-plan-v2-unresolved-audit.md) for strict results
using the newer downloaded route CSV and navdata.

Measured on 2026-10-04 using `data/Route-Server.csv` and Navigraph DFD v2 cycle
2609.1.0, locally on arm64 in a Cargo debug build. These are development-build
measurements, not release-build or server throughput measurements.

## Coverage and correctness

The runner calls the public `v2::parse_route` pipeline used by the CLI and HTTP
endpoint: lexer/parser, candidate resolver, solver, constructor, and expander.
Each CSV row becomes `Dep Route Arr`, trimming fields and uppercasing the airport
fields. All 20,920 rows are exercised, including duplicates; there are 20,785
unique complete route strings. Inputs contain 3–49 tokens including endpoints
(mean 15.33).

The final run passed **20,920 / 20,920** rows, with no parsing errors, panics,
timeouts, disconnected segments, or incorrect departure/arrival identifiers.
Every result was nonempty, preserved the requested endpoints, and had equal
fixes on both sides of each segment boundary.

Successful parsing permits the existing `AnyFix::Unknown` recovery behavior.
**455 routes (2.17%) contain unresolved fixes**, representing 38 distinct
identifiers. The most common are ZHLY (146 rows), VDSR (64), ZBUH (47), UTTT
(40), and AVKES (24). These routes are not completely resolved against navdata.
The per-row JSON records their identifiers; the counts are routes containing
each identifier, not occurrences within a route.

The initial run exposed a lexer panic on short identifiers beginning with
K, N, or M, including KBOS and NLG. The speed/altitude handler now uses checked
string slices and lets these tokens fall through to identifier parsing. Its
regression test also covers an invalid UTF-8 slicing boundary in `M中`.

## Full pipeline timing

The full run used a temporary **indexed copy** of the navigation database. The
original data file was unchanged. Nine indexes cover identifier lookup, airway
lookup/order, and SID/STAR lookup/order; their definitions are in
`examples/route_v2_indexes.sql`.

| Metric | Indexed copy, full corpus |
| --- | ---: |
| Wall time | 177.55 s |
| Sequential throughput | 117.83 routes/s |
| Mean | 8.47 ms/route |
| Median | 7.78 ms/route |
| p95 | 16.51 ms/route |
| p99 | 20.84 ms/route |
| Maximum | 47.12 ms/route |

CSV loading, database/service initialization, route-string formatting, and
JSON serialization are excluded from per-route timings. Wall time includes
result checking and progress reporting, but excludes the later stage profiling.
Routes execute sequentially, with one timed attempt per row and a 30-second
timeout. The OS/SQLite cache is not reset between routes or runs. These figures
describe this local corpus run rather than cold-cache latency or concurrent
endpoint capacity.

After the full run, 100 evenly spaced successful routes were profiled again,
timing each stage separately:

| Stage | Mean per sampled route |
| --- | ---: |
| Lexer + parser | 0.0224 ms |
| Candidate resolver | 5.6018 ms |
| Solver | 0.1126 ms |
| Constructor | 0.0220 ms |
| Expander | 2.9120 ms |

Lexer/parser, solver, and constructor together average **0.157 ms**. Candidate
lookup and expansion account for about 98% of sampled time. These independently
profiled sample means should not be added to reproduce the full-corpus mean.

## Original database baseline

The original database has no indexes on the relevant navigation tables. Before
the lexer fix, the first 250 rows took 74.27 seconds: 150 succeeded and 100
panicked. Restricting latency statistics to the 150 successful rows gives a
mean of **495.07 ms**, median **554.16 ms**, and p95 **741.03 ms**. The overall
250-row mean includes fast panics and is not a successful-parsing latency metric.

A separate 12-route stage sample on that database averaged 419.81 ms for the
candidate resolver and 58.72 ms for expansion. This identifies database lookup
as the main bottleneck. The indexed full-corpus timings above must not be treated
as performance of the original unindexed file. The baseline and full run use
different samples and cache conditions, so they are not a controlled speedup
comparison. No release-build benchmark was performed.

## Reproduce

Run from the repository root. To validate against the original database:

```sh
cargo run --offline --example route_v2_corpus -- \
  --navdata 'data/NavigraphDFDv2-2609.1.0.db?mode=ro' \
  --allow-unresolved \
  --output /tmp/route-v2-original.json
```

To reproduce the indexed-copy run without modifying the original:

```sh
cp data/NavigraphDFDv2-2609.1.0.db /tmp/route-v2-indexed-2609.db
sqlite3 /tmp/route-v2-indexed-2609.db < examples/route_v2_indexes.sql
cargo run --offline --example route_v2_corpus -- \
  --navdata '/tmp/route-v2-indexed-2609.db?mode=ro' \
  --allow-unresolved \
  --stage-samples 100 \
  --output /tmp/route-v2-corpus.json
```

Use `--limit 250` for a smaller run, `--csv <path>` for another corpus, or
`--release` on Cargo for a separate optimized-build measurement. The example
prints the summary, saves all per-row measurements and stage samples, and exits
with an error if any row fails. No HTTP server or PostgreSQL connection is needed.

The actual final report from this session is `/private/tmp/route-v2-corpus.json`;
the original baseline is `/private/tmp/route-v2-baseline-sample.json`. Generated
per-row reports are kept outside the repository.

V2 regression checks passed 50 tests, including the lexer regression. The two
existing solver debug tests that assert an empty result were excluded:

```sh
cargo test --offline modules::flight::flight_plan::v2:: -- \
  --skip modules::flight::flight_plan::v2::solver::test
```
