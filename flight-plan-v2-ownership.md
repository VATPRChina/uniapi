# Flight plan v2 ownership and clone audit

The candidate resolver owns the navigation data for a parse. The solver and
constructor borrow that data, and the expander creates the owned API result.
This avoids copying entire procedures into search states and intermediate legs.

```text
Lexer -> Parser -> CandidateResolver
                         |
                         +-- owns candidates, fixes and procedure leg lists
                         |
                       Solver -- borrows candidates and parsed identifiers
                         |
                     Constructor -- owns logical legs, borrows chosen procedures
                         |
                      Expander -- consumes logical legs, searches borrowed edges
                         |
                  Vec<ResolvedLeg> -- owns final output, independent of candidates
```

## Changes by stage

| Stage | Previous copies | Current ownership |
| --- | --- | --- |
| Lexer/parser | Parser clones speed and cruising-level enums | Kept: these contain only small numeric values; there is no heap data to duplicate |
| Candidate resolver | Moves identifiers and loaded navigation objects into candidates | Already moves data; no deep candidate clone needed |
| Solver | Clones each candidate and its fix/procedure into every successor state; a procedure clone duplicates its full leg vector | States hold references to candidates, fixes and procedures; solved identifiers borrow the parsed identifier |
| Constructor | Clones the selected procedure, including its published legs | Borrows the selected procedure; keeps simple owned logical endpoints and identifier |
| Expander graph | Clones airway segments to build an owned forward/reverse graph | Uses a borrowed segment plus an orientation flag; reverse traversal copies no segment data |
| Expander search | Clones complete segment prefixes and visited fixes when advancing search | Copies only edge references/orientation flags and fix references |
| Expander output | Clones unchanged logical legs and selected published segments | Moves unchanged logical legs; constructs only selected output segments |

No `Arc`, shared ownership registry, or new borrowed navigation-model hierarchy
is required. The candidate collection simply stays alive through expansion.
`SolvedIdent`, `CandidateWithState` and `ConstructedLeg` now express those borrows
with lifetimes. `Solver::new` accepts a candidate slice; `Expander::expand` consumes
the expander. The public route parser still returns an owned `Vec<ResolvedLeg>`.

## Copies deliberately retained

Construction copies selected endpoint fixes and the route identifier into a
logical leg. Successful expansion copies selected segment endpoints and
identifiers into the final result. Most fix variants contain fixed-size data;
unknown/reference variants may own heap data. Keeping the logical leg owned
makes fallback handling and the constructor interface straightforward.

The breadth-first search still creates vectors for its frontier, visited set and
path prefixes. Those vectors contain references and small orientation flags,
rather than owned navigation objects. A predecessor arena could remove prefix
copies, but would add indexing and reconstruction code. That change is deferred
until profiling shows these small copies matter.

Navigation loading also retains small airport-key and record-string copies, and
copies adjacent endpoints into independently owned procedure segments. These
occur while building the candidate data, rather than once per solver branch.
They are kept to preserve the existing navigation-service representation.

## Verification on 2026-10-06

- All 26 flight-plan-v2 tests pass. Coverage includes saved predecessor
  reconstruction, forward/reverse airway expansion, published-order tie breaking,
  directed SID/STAR common legs, cycles and disconnected sections.
- The real-navdata `ELKUR W40 YQG` regression expands into six connected segments.
- The first 100 rows of `data/routes.csv`, using a temporary indexed copy of cycle
  2610 revision 002, all return nonempty routes with correct departure/arrival,
  continuous segment boundaries and valid known-fix coordinates.
- Per-row segment counts, unknown-fix sets, unknown-leg sets and unexpanded-airway
  diagnostics match the previous run for all 100 rows. This comparison does not
  establish byte-for-byte equality of every output segment.

The initial sample timing was **not a controlled clone comparison**: at that
point the working tree's solver retained every sorted alternative in a priority
group because the final `.next()` was commented out. The previous benchmark
retained only one. That behavior was preserved during the ownership change.

| Mean time per route, first 100 rows (debug build) | Previous run | Current run |
| --- | ---: | ---: |
| Total | 4.791 ms | 67.977 ms |
| Candidate resolution | 4.384 ms | 4.845 ms |
| Solver | 0.151 ms | 62.731 ms |
| Constructor | 0.027 ms | 0.054 ms |
| Expander | 0.210 ms | 0.321 ms |

The large solver increase reflects the broader search and prevents attributing
those sample timing differences to the ownership refactor. No measured
memory-reduction claim is made. Initial sample timing and validation artifacts
are saved under `/tmp/route-v2-clones-2026-10-06/`.

## Full-corpus performance retry on 2026-10-06

The solver's `.next()` has been restored. This run uses the same corpus, cycle
2610 revision 002, ten temporary identifier indexes and normal Cargo debug
build as the full pre-refactor benchmark. It includes all 20,877 rows, including
duplicates. Routes run sequentially with a shared service, using stage observers
without formatting the developer trace. Per-route timing excludes CSV reading,
service initialization and route-string construction. Wall time includes basic
result checks and progress logging, but excludes compilation and report output.

Environment: macOS arm64, Rust 1.97.1. Navigation and parser source hashes remained
unchanged throughout the run. Original CSV and navigation database were unchanged.
The previous implementation was not rerun; its recorded full-corpus results are
used as the baseline, so cache and machine-load differences remain possible.

| Mean time | Before clone refactor | After clone refactor | Change |
| --- | ---: | ---: | ---: |
| Total per route | 6.2685 ms | 6.4384 ms | +2.7% |
| Lexer | 0.0121 ms | 0.0119 ms | -1.2% |
| Parser | 0.0107 ms | 0.0110 ms | +3.3% |
| Candidate resolver | 5.7384 ms | 5.8718 ms | +2.3% |
| Solver | 0.2011 ms | 0.1267 ms | -37.0% |
| Constructor | 0.0336 ms | 0.0112 ms | -66.6% |
| Expander | 0.2718 ms | 0.3967 ms | +46.0% |

| Full-pipeline metric | Before clone refactor | After clone refactor |
| --- | ---: | ---: |
| Wall time | 131.520 s | 135.113 s |
| Throughput | 158.74 routes/s | 154.52 routes/s |
| Median | 5.735 ms | 5.902 ms |
| p95 | 12.268 ms | 12.643 ms |
| p99 | 15.392 ms | 16.065 ms |
| Maximum | 49.409 ms | 55.345 ms |

Overall latency is 2.7% higher in this run. Solver and constructor means decrease
by 37.0% and 66.6%, respectively; the expander mean increases by 46.0%.
Removing deep clones therefore did not yield an overall speedup in this
measurement. Candidate resolution still accounts for approximately 91.2% of
mean total latency. The cause of the expansion increase was not isolated by this
benchmark; it requires focused profiling before attributing it to a specific
search operation.

A separate validation pass (145.471 s, excluded from the performance table)
checks nonempty output, correct departure/arrival identifiers, exact continuity
of adjacent segments, valid coordinates for known fixes, and known-airway
connections retained without expansion. The real-data W40 regression also passes.

- All 20,877 routes return successfully; no errors, panics, empty routes,
  disconnected boundaries, incorrect endpoints or invalid known-fix coordinates.
- Both new passes return 615,846 segments, matching the baseline per row.
- Unknown identifiers remain in 262 rows: 261 with unknown fixes, two with unknown
  legs, one overlapping. Counts match the baseline.
- Known-airway fallbacks remain at 1,356 connections across 1,109 rows; per-row
  diagnostics match the baseline.
- Both new passes agree on all recorded outcomes. One route differs from the
  baseline in its unknown fix/leg interpretation: CSV row 20018,
  `ZJHK YLW LGG LG HT GPL SQG SYX ZJSY`. Previously, unknown fixes were `SQG,YLW`
  and unknown legs `GPL,LGG`; now they are `GPL,YLW` and `LGG,SQG`.
  Forty additional replays consistently return the new interpretation. This
  performance test does not establish the cause of that difference.

Artifacts:

- [Full per-route timings](/tmp/route-v2-clones-retry-2026-10-06/timings.tsv)
- [Full correctness and airway validation](/tmp/route-v2-clones-retry-2026-10-06/validation.tsv)
- [Timing runner](/tmp/route-v2-clones-retry-2026-10-06/timing-harness.rs)
- [Validation runner](/tmp/route-v2-clones-retry-2026-10-06/validation-harness.rs)
- [Ambiguous-route replay results](/tmp/route-v2-clones-retry-2026-10-06/ambiguous-route.tsv)

## Repeat with `.next()` explicitly verified active

A further full-corpus debug run on 2026-10-06 explicitly asserted that the
candidate-pruning `.next()` follows the fallback/distance sort, both before
compilation and after the benchmark. All parser and navigation source hashes
remained unchanged during the run. No solver code was edited for this repeat.
The same routes, navigation data and ten temporary indexes were used.

| Metric | Previous retry | Verified repeat |
| --- | ---: | ---: |
| Wall time | 135.113 s | 133.569 s |
| Throughput | 154.52 routes/s | 156.30 routes/s |
| Mean route latency | 6.438 ms | 6.362 ms |
| Median | 5.902 ms | 5.849 ms |
| p95 | 12.643 ms | 12.425 ms |
| p99 | 16.065 ms | 15.535 ms |
| Maximum | 55.345 ms | 59.672 ms |
| Resolver mean | 5.872 ms | 5.802 ms |
| Solver mean | 0.127 ms | 0.123 ms |
| Constructor mean | 0.0112 ms | 0.0110 ms |
| Expander mean | 0.397 ms | 0.394 ms |

The repeat is 1.2% faster on mean route latency than the previous retry, and
1.5% slower than the recorded pre-refactor mean of 6.269 ms. The solver and
constructor improvements and expansion regression persist with `.next()` active.
These repeats still do not isolate cache or background machine-load effects.

All 20,877 rows pass nonempty-output, endpoint and exact segment-continuity
checks. Those checks run after the per-route timer is stopped. No exceptions,
panics or invalid known-fix coordinates occur. There are 615,846 output segments
and 262 rows with existing unknown identifiers. Per-row status, segment count,
unknown-fix/leg sets and coordinate validity match the previous retry in every
row. The heavier airway-fallback diagnostic was not repeated in this timed pass.

Artifacts:

- [Per-route timings and outcomes](/tmp/route-v2-next-active-2026-10-06/timings.tsv)
- [Pruning/source verification and statistics](/tmp/route-v2-next-active-2026-10-06/summary.json)
- [Solver source used for the run](/tmp/route-v2-next-active-2026-10-06/solver.rs)
- [Timing runner](/tmp/route-v2-next-active-2026-10-06/timing-harness.rs)
