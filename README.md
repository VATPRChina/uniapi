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
vatprc-uniapi route-v2 "ZBAA ELKUR W40 YQG ZSPD" # print expanded route JSON
```

`route-v2` accepts a complete route including departure and arrival. It runs the
same v2 pipeline as `GET /api/flights/route/v2` and prints expanded segments as
JSON with fix coordinates (null for unresolved fixes). It uses the configured
navdata paths and requires no running HTTP server or PostgreSQL connection.
Invalid routes exit with an error.

To use other local navigation data files:

```sh
cargo run -- route-v2 "ZBAA ELKUR W40 YQG ZSPD" \
  --navdata "data/NavigraphDFDv2-2604.1.0.db?mode=ro" \
  --preferred-routes data/Route-Server.csv
```

To validate every route in the CSV and measure the v2 pipeline:

```sh
cargo run --offline --example route_v2_corpus -- --output /tmp/route-v2-corpus.json
```

The report records errors, endpoint/continuity checks, unresolved fixes, and
per-route timings. Unknown fixes and unknown connecting legs fail validation by
default; use `--allow-unresolved` to explicitly measure recovery behavior instead.
See [the corpus benchmark](flight-plan-v2-corpus-benchmark.md)
for measured results and instructions for running against an indexed navdata copy.

### Database

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
