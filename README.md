# VATPRC UniAPI

This project targets providing an universal API server for all the services VATPRC requires.

## Architecture

TODO.

## Development

The application binary supports these commands:

```sh
vatprc-uniapi              # start the web application
vatprc-uniapi run          # start the web application
vatprc-uniapi openapi      # save the specification to openapi.json
vatprc-uniapi openapi -o api.json
vatprc-uniapi migrate      # apply pending database migrations
vatprc-uniapi route-v2 "ZBAA ELKUR W40 YQG ZSPD" # print the complete parser trace
```

`route-v2` accepts a complete route including departure and arrival. It runs the
same v2 pipeline as `GET /api/flights/route/v2` and prints a plain-text trace of
the lexer, parser, candidate resolver, solver, constructor, and expander.
The trace includes every token, amendment, parser error, resolved candidate,
surviving solver state (distance, recovery count, position, and predecessor),
and constructed/expanded leg with coordinates and direction restrictions.
Existing v2 diagnostic logs (such as rejected transitions) are also printed;
`RUST_LOG` can override the default v2 trace filter.
Unknown fixes and legs are explicitly marked. It uses the configured navdata
paths and requires no running HTTP server or PostgreSQL connection. Invalid
routes exit with an error after printing the completed stages.

The final CLI stage lists mainland route portions, matched preferred-route names,
and original leg indices. Border crossings are checked with both mainland and
foreign endpoints; unmatched portions are explicitly marked. This stage uses
the mainland FIR records and preferred-route CSV configured below.

To use other local navigation data files:

```sh
cargo run -- route-v2 "ZBAA ELKUR W40 YQG ZSPD" \
  --navdata "data/NavigraphDFDv2-2604.1.0.db?mode=ro" \
  --preferred-routes data/Route-Server.csv
```

### Database

Preferred-route segment selection reads the nine mainland FIR boundaries from
the configured navigation database's ARINC 424 `tbl_uf_fir_uir` records.
International flights require ZBPE, ZGZU, ZHWH, ZJSA, ZLHW, ZPKM, ZSHA,
ZWUQ and ZYSH; regional/sample databases may not contain all nine. FIR and
combined FIR/UIR records are included; delegated airspace and UIR-only records
are excluded. Great-circle and rhumb-line boundaries are supported; missing,
malformed or unsupported boundary data returns a navigation-data error.

`RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs)` returns
one `Segment` per connected mainland visit, borrowing the original legs and
retaining their indices. Each segment exposes airport, enroute, and border
entry/exit endpoints. Unknown fix chains stay in the path; crossing endpoints
are the known fixes on either side of the chain. The first and last visits
retain their respective departure and arrival airports, when present.

`PreferredRouteMatcher::new(&navdata).match_segment(&segment).await?` returns
an optional preferred route. An airport-to-airport match takes priority. If
that fails, the matcher tries departure airport or either entry endpoint
against arrival airport or either exit endpoint. Routes that start/end at an
enroute fix retain that endpoint. Public routes precede private routes within
each priority tier. CSV `Waypoint1`/`Waypoint2` supply endpoints when `Dep`/`Arr`
are empty. Matching compares the complete expanded enroute path, from the first
fix after SID to the last fix before STAR, ignoring airport connections.
Unknown legs cannot match a designated path. The batch `find_matching_route`
method also returns the exact covered leg range for segment validation.

The validations endpoint checks each matched portion's altitude restrictions
and suppresses route warnings only on its covered legs. Foreign and unmatched
portions keep their own warnings, with indices referring to the original route.

This server requires a local PostgreSQL server. Please start one and specify the
following in `settings.local.toml`.

```
[database]
url = "postgres://postgres:password@localhost/vatprc"
```

And, to migrate the database:

```
cargo run -- migrate
```

To create a new migration:

```
just new-migration MigrationName
```

### Discord bot

The service can optionally run a Discord bot alongside the HTTP API. It is
disabled by default; enable it in local settings and provide a bot token:

```toml
[discord]
enabled = true
token = "<discord-bot-token>"
```

When connected, the bot registers global `/ping` and `/metar` commands. `/ping`
replies with `pong`; `/metar icao:<airport>` replies with the latest METAR text
for the requested airport.

### Email notifications

ATC application review and training application response notifications can be
sent through an SMTP server. Email is disabled by default. Enable it in local
settings (or provide the equivalent `APP_EMAIL__*` environment variables):

```toml
[email]
enabled = true

[email.smtp]
server = "smtp.example.com"
port = 587
username = "smtp-user"
password = "smtp-password"
from = "VATPRC <no-reply@example.com>"
```

The SMTP connection uses STARTTLS.

### Flight plan validation updates

Connect to `ws(s)://<host>/api/flights/warnings/streaming` to receive flight
plan validation results for all active flights. The server sends the current
results as a JSON object keyed by callsign in the first text message, checks all
flight plans again every 30 seconds, and sends another complete snapshot only
when the results change. This includes flights appearing and disappearing. The
existing HTTP validation endpoints remain available for request-based checks.

Every validation also runs the v2 parser and validator in a background shadow
task. Responses continue to use v1, including v1 errors. If the results differ,
the server logs `flight validation shadow mismatch` at WARN with the callsign,
complete route, and both results. Comparisons include warning order, codes,
parameters, fields, and field indices; failures are compared by error message.
V2 failures do not affect the response.

## Testing

There is E2E testing for each API endpoint. It is still missing test coverage on some endpoints.

### Writing tests

Place the test for each API endpoint in `/e2e/src/**/*.test.ts`. The test folder structure should match the API endpoint's URL. For example, test for GET `/api/atc/applications/review-sheet` should be placed in `/e2e/src/atc/applications/review-sheet.test.ts` (prefix `/api` is stripped).

If a user is needed, check `getClient` from `/e2e/lib/backend.ts`. Use Vitest's `extend` to establish a shared context for different roles used in the same test file.

If some common data setup is required for the API endpoint (e.g. `/api/events` requires a present event), use Vitest's `extend` to establish a shared context to avoid duplication in setting up the data.

## License

    VATPRC UniAPI - Universal API endpoint for VATPRC
    Copyright (C) 2024 VATPRC Staff

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program. If not, see <https://www.gnu.org/licenses/>.

### Contributor License Agreement

    By contributing to the repository, in addition to the open-source license attached
    to the repository, the contributors are additionally granting VATPRC staffs an
    unrevokeable right to use the code freely for any purposes related to VATSIM or
    VATPRC.
