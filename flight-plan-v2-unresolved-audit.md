# Flight plan v2 unresolved identifier audit

Audited on 2026-10-04 using the requested latest data:

- CSV: `/Users/xfoxfu/Downloads/Route.csv` (20,877 rows; 20,742 unique complete routes).
- Database: `/Users/xfoxfu/Downloads/NavigraphDFDv2-2610.2.0/NavigraphDFDv2-2610.2.0.db`.
- Database header: cycle 2610, revision 002, Jeppesen; built 2026-09-30.
- Benchmarked against an indexed temporary copy; downloaded source files were unchanged.

## Result

All 20,877 inputs completed the shared parsing/solving/construction/expansion
pipeline without parser errors, panics, timeouts, incorrect endpoints, or
disconnected segments. **20,615 pass strict unresolved-identifier checks; 262
fail**. Of those failures, 261 contain unknown fixes and two contain unknown
connections; one route belongs to both groups.

There are **37 distinct unresolved identifiers** in the output. Direct database
checks found no matching fix or connection records for any of them, including
checks with identifier case and surrounding whitespace normalized. Missing
records therefore explain the remaining failures; they are not failed conversions
of known database records.

Cycle 2610 supplies airport records for ZHLY and ZBUH, resolving 193 previously
affected rows. Switching data cannot resolve the remaining identifiers when they
have no record in the newer source.

## Bugs found and fixed

### Solver preferred a shorter recovery history

The initial cycle 2609 result contained W45 as an unknown fix in 23 rows, even
though W45 exists as an airway. The resolver supplied that known airway candidate,
but it also supplied the intended recovery alternatives. When histories converged
on a later fix, the solver retained the shortest distance within each candidate
kind, without accounting for earlier recovery choices. With a missing departure,
a history treating LYA as an unknown connection and W45 as an unknown point could
beat the known LYA/W45 history on distance.

State now tracks cumulative fallback interpretations. Pruning compares that count
before distance within a candidate kind, retaining existing kind priorities.
The regression test includes an intentionally much longer known history. Rerunning
all 455 originally affected rows against cycle 2609 produced **zero W45 unknown
fixes**, down from 23.

### Corpus validation counted recovery as success

The original harness accepted unknown fix results. Checking only unknown fixes
also missed unknown connecting legs: the VFR route `TLG LIS NYW GB BAG WC SJ`
could contain unknown connection names with no `AnyFix::Unknown` in its output.

Validation now rejects either an unknown fix or a connection name absent from
the airway/SID/STAR tables. Known connection names are loaded before timing,
so these checks introduce no database queries into measured pipeline latency.
`--allow-unresolved` explicitly opts into the previous recovery-success behavior.
Production recovery remains available; the test runner reports it as a failure
instead of hiding missing navigation data.

## Remaining data gaps

The six fix-source tables checked are `tbl_pa_airports`, `tbl_d_vhfnavaids`,
`tbl_db_enroute_ndbnavaids`, `tbl_pn_terminal_ndbnavaids`,
`tbl_ea_enroute_waypoints`, and `tbl_pc_terminal_waypoints`. VHF identifiers use
the resolver's `coalesce(navaid_identifier, dme_ident)` expression. Connection
checks cover `tbl_er_enroute_airways`, `tbl_pd_sids`, and `tbl_pe_stars`.

HSSS and YBBB occur in FIR metadata, not airport or fix records. A FIR record does
not provide an airport candidate. Three endpoints are Chinese place labels
(神户, 阿皮亚, 塔姆奇) rather than airport identifiers. The CSV contains no
coordinates or alias mapping for those entries. No coordinates or replacements
were invented to make validation pass.

The table counts routes containing each name, so its counts must not be summed
as a route total. Fix/connection columns describe the chosen recovery interpretation,
not the intended aviation meaning. In particular, missing VFR points may appear
as unknown connecting legs.

| Identifier | Input role | Rows | Unknown fix rows | Unknown connection rows | First CSV line |
| --- | --- | ---: | ---: | ---: | ---: |
| AGVUR | Route point | 16 | 16 | 0 | 20306 |
| AKNUX | Route point | 4 | 4 | 0 | 20148 |
| AKS | Route point | 8 | 8 | 0 | 20142 |
| AVKES | Route point | 24 | 24 | 0 | 20185 |
| EDOP | Endpoint | 3 | 3 | 0 | 5440 |
| FVHA | Endpoint | 2 | 2 | 0 | 65 |
| GPL | Route point | 1 | 0 | 1 | 20018 |
| GQNA | Endpoint | 2 | 2 | 0 | 12306 |
| HSSS | Endpoint | 6 | 6 | 0 | 77 |
| LAPEN | Route point | 2 | 2 | 0 | 20265 |
| LGG | Route point | 1 | 0 | 1 | 20018 |
| NUSLA | Route point | 14 | 14 | 0 | 20148 |
| NYW | Route point | 1 | 0 | 1 | 20019 |
| OAJL | Endpoint | 1 | 1 | 0 | 18259 |
| OISKI | Route point | 1 | 1 | 0 | 13732 |
| OKBK | Endpoint | 3 | 3 | 0 | 244 |
| POTOT | Route point | 4 | 4 | 0 | 20347 |
| SQG | Route point | 1 | 1 | 0 | 20018 |
| TAREX | Route point | 6 | 6 | 0 | 20219 |
| TLG | Route point | 1 | 0 | 1 | 20019 |
| UAFO | Endpoint | 4 | 4 | 0 | 362 |
| URRR | Endpoint | 2 | 2 | 0 | 6697 |
| UTFF | Endpoint | 1 | 1 | 0 | 18321 |
| UTSA | Endpoint | 7 | 7 | 0 | 1803 |
| UTSS | Endpoint | 5 | 5 | 0 | 9241 |
| UTTT | Endpoint | 40 | 40 | 0 | 397 |
| VDSR | Endpoint | 64 | 64 | 0 | 409 |
| VYEL | Endpoint | 8 | 8 | 0 | 442 |
| WADA | Endpoint | 5 | 5 | 0 | 3764 |
| WARQ | Endpoint | 1 | 1 | 0 | 8968 |
| WC | Route point | 1 | 0 | 1 | 20019 |
| YBBB | Endpoint | 21 | 21 | 0 | 458 |
| YLW | Route point | 1 | 1 | 0 | 20018 |
| ZGNT | Endpoint | 6 | 6 | 0 | 4244 |
| 塔姆奇 | Endpoint | 1 | 1 | 0 | 18186 |
| 神户 | Endpoint | 3 | 3 | 0 | 11733 |
| 阿皮亚 | Endpoint | 1 | 1 | 0 | 6933 |

The full affected-row CSV is `/private/tmp/route-v2-unresolved-2610.csv`, with
original CSV fields, original line numbers, and separate unknown fix/connection
lists. Database audit details are in `/private/tmp/route-v2-unresolved-audit-2610.json`.
The full corpus report is `/private/tmp/route-v2-corpus-2610.json`.

## Timing and validation

Debug build, arm64, sequential processing, indexed temporary navdata copy:

| Metric | Result |
| --- | ---: |
| Wall time | 183.40 s |
| Mean per route | 8.76 ms |
| Median | 7.94 ms |
| p95 | 17.34 ms |
| p99 | 21.95 ms |
| Throughput | 113.83 routes/s |

Stage profiling used 98 successful, evenly spaced rows. Mean lexer/parser
cost was 0.0251 ms; solver cost was
0.1186 ms. Database lookup and expansion dominate.
These are not release-build or original unindexed-database timings.

V2 tests passed **51 tests**, including the new history-selection regression.
The two existing solver debug tests that assert empty output were excluded.

## Reproduce

Run from the repository root:

```sh
cp ~/Downloads/NavigraphDFDv2-2610.2.0/NavigraphDFDv2-2610.2.0.db /tmp/route-v2-indexed-2610.db
sqlite3 /tmp/route-v2-indexed-2610.db < examples/route_v2_indexes.sql
cargo run --offline --example route_v2_corpus -- \
  --csv ~/Downloads/Route.csv \
  --navdata '/tmp/route-v2-indexed-2610.db?mode=ro' \
  --output /tmp/route-v2-corpus-2610.json
```

With these inputs this command exits with an error reporting the 262 unresolved
routes, after saving every row's result. Clearing those failures requires valid
source records or corrected CSV identifiers for the data gaps listed above.
