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

### Route regions

Expanded routes distinguish mainland, Hong Kong, and international segments.
Hong Kong coverage comes from the current nav database's `VHHK` FIR records
(`tbl_uf_fir_uir`, indicators F/B), with great-circle and rhumb-line boundary
edges handled by the shared FIR loader. Existing mainland cuts take priority:
some mainland-coded corridors, such as M503 beyond LELIM, overlap the horizontal
Hong Kong FIR. The remaining route is classified geographically, without a
production list of Hong Kong entry/exit fix names.

Each expanded leg is classified using its great-circle midpoint. This cuts
published airways at their crossing fixes without assigning an extra leg to
Hong Kong merely because its endpoint lies on the FIR boundary. Route slices
preserve original global leg indices. A direct leg crossing between fixes
remains indivisible and is assigned by its midpoint; no synthetic crossing fix
is inserted. Unknown-fix chains use their nearest surrounding known positions.
Missing Hong Kong FIR coverage disables this classification independently of
mainland coverage. Hong Kong segments skip mainland preferred-route and scoped
validation checks.

The optional navdata integration tests compare common routes against
[Hong Kong AIP ENR 1.10](https://www.ais.gov.hk/eaip_20261001/2026-10-01-000000/html/eAIP/VH-ENR-1.10-en-US.html#i24049):

```sh
VATPRC_NAVDATA_PATH=data/navdata_naip.db cargo test aip_hong_kong -- --ignored
```

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

## Testing

Test routes are defined in `assets/test/routes.sql`. CI generates
`data/routes.db` from this SQL for both Rust and E2E tests. For local test setup,
create a new test database with:

```sh
mkdir -p data
sqlite3 -bail data/routes.db < assets/test/routes.sql
```

The database must not already contain route tables. Generated databases stay
under the ignored `data/` directory; no binary fixture is required.

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
