# VATPRC UniAPI

This project targets providing an universal API server for all the services VATPRC requires.

## Architecture

TODO.

## Development

```
just run
--- OR ---
just watch
```

### Database

This server requires a local PostgreSQL server. Please start one and specify the
following in `Net.Vatprc.Uniapi/appsettings.Local.toml`.

```
[ConnectionStrings]
VATPRCContext = "Host=localhost;Port=5432;Username=postgres;Password=;Database=vatprc"
```

And, to migrate the database:

```
just db-update
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
