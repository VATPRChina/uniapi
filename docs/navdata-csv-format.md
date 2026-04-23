# Navdata CSV Format

This document describes the CSV files currently consumed by [`NavadataAdapter`](../../src/Adapters/NavdataAdapter.cs).

## Location

The application reads navdata from S3 using these settings:

- `Navdata:S3:AipPath`: directory-like prefix for per-model CSV files
- `Navdata:S3:RoutePath`: preferred-route CSV file

Per-model files are expected at:

- `{AipPath}/airport.csv`
- `{AipPath}/waypoint.csv`
- `{AipPath}/vhf_navaid.csv`
- `{AipPath}/ndb_navaid.csv`
- `{AipPath}/airway.csv`
- `{AipPath}/airway_fix.csv`
- `{AipPath}/procedure.csv`

## General Rules

- Files must be CSV with a header row.
- Header names are case-sensitive and must match the names below.
- Unused extra columns are allowed.
- Numeric latitude/longitude values are parsed as decimal numbers.
- Empty strings are only allowed where noted.

## `airport.csv`

Used by `FindAirport`.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable airport id. Used by `procedure.csv`. |
| `identifier` | string | Airport ICAO identifier, for example `ZBAA`. |
| `latitude` | double | Decimal degrees. |
| `longitude` | double | Decimal degrees. |

Example:

```csv
id,identifier,latitude,longitude
01JTESTAIRPORT,ZBAA,40.0799,116.6031
```

## `waypoint.csv`

Used by `FindFix`.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `icao_code` | string | Region or area code used to disambiguate fixes. |
| `identifier` | string | Waypoint name. |
| `latitude` | double | Decimal degrees. |
| `longitude` | double | Decimal degrees. |

Example:

```csv
icao_code,identifier,latitude,longitude
ZB,TAMOT,39.812500,116.125000
```

## `vhf_navaid.csv`

Used by `FindFix`.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `icao_code` | string | Region or area code. |
| `vor_identifier` | string | Navaid identifier used in route parsing. |
| `vor_latitude` | double or empty | Rows without both coordinates are ignored. |
| `vor_longitude` | double or empty | Rows without both coordinates are ignored. |

Example:

```csv
icao_code,vor_identifier,vor_latitude,vor_longitude
ZB,CGO,34.519444,113.840278
```

## `ndb_navaid.csv`

Used by `FindFix`.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `icao_code` | string | Region or area code. |
| `identifier` | string | NDB identifier. |
| `latitude` | double | Decimal degrees. |
| `longitude` | double | Decimal degrees. |

Example:

```csv
icao_code,identifier,latitude,longitude
ZB,XX,39.900000,116.300000
```

## `airway.csv`

Used with `airway_fix.csv` to build airway legs.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable airway id referenced by `airway_fix.csv`. |
| `identifier` | string | Airway name, for example `A1`. |

Example:

```csv
id,identifier
01JTESTAIRWAY,A1
```

## `airway_fix.csv`

Used with `airway.csv` to build airway segments in sequence order.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `airway_id` | string | Must match `airway.csv:id`. |
| `sequence_number` | integer | Sort order inside an airway. |
| `fix_identifier` | string | Identifier of the fix at this sequence position. |
| `fix_icao_code` | string | Used to disambiguate duplicate fix identifiers. |
| `directional_restriction` | char | `F` = forward, `B` = backward, anything else = both. |

Notes:

- Each adjacent pair of rows inside one airway becomes one leg.
- `fix_identifier` is resolved against the combined airport, waypoint, VHF navaid, and NDB navaid datasets.
- If a referenced fix cannot be found, that leg is skipped.

Example:

```csv
airway_id,sequence_number,fix_identifier,fix_icao_code,directional_restriction
01JTESTAIRWAY,10,TAMOT,ZB,F
01JTESTAIRWAY,20,SOSDI,ZB,F
01JTESTAIRWAY,30,TONIL,ZB,F
```

## `procedure.csv`

Used by `FindSid` and `FindStar`.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `identifier` | string | Procedure name. |
| `airport_id` | string | Must match `airport.csv:id`. |
| `subsection_code` | char | `D` = SID, `E` = STAR. |

Notes:

- The current implementation only checks existence and returns a `Procedure` with an empty `Legs` list.
- No procedure leg CSV is consumed yet.

Example:

```csv
identifier,airport_id,subsection_code
RENOB7D,01JTESTAIRPORT,D
GITUM1E,01JTESTAIRPORT,E
```

## Preferred Route CSV

The preferred-route file is loaded from `Navdata:S3:RoutePath` rather than from `{AipPath}`.

Required columns:

| Column | Type | Notes |
| --- | --- | --- |
| `Dep` | string | Departure airport. |
| `Arr` | string | Arrival airport. |
| `Name` | string | Currently parsed but not used. |
| `EvenOdd` | string | `SE`, `SO`, `FE`, `FO`, or other. |
| `AltList` | string | Slash-separated altitude list like `S089/F330`. |
| `MinAlt` | string | Integer feet, or empty for `0`. |
| `Route` | string | Raw route string. |
| `Remarks` | string | Free text. |

`EvenOdd` mapping:

- `SE` -> standard even
- `SO` -> standard odd
- `FE` -> flight-level even
- `FO` -> flight-level odd
- anything else -> standard

`AltList` format:

- `Sxxx` means standard altitude, converted through `AltitudeHelper.StandardAltitudesToFlightLevel`
- `Fxxx` means flight level `xxx * 100`

Example:

```csv
Dep,Arr,Name,EvenOdd,AltList,MinAlt,Route,Remarks
ZBAA,ZSPD,SAMPLE,FE,F330/F350,18000,DCT TAMOT A1 SOSDI,Preferred by ACC
```

## Current Assumptions

These are implementation assumptions in the current code and should stay aligned with the data export:

- Filenames are snake_case and match the model names above.
- `procedure.subsection_code` uses `D` for SID and `E` for STAR.
- `airway_fix.directional_restriction` uses `F` and `B`; any other value is treated as bidirectional.
- `airport.identifier` is used as both `IcaoCode` and `Identifier` in the in-memory `Airport` model.
