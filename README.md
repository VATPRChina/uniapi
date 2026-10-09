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

To use other local navigation data files:

```sh
cargo run -- route-v2 "ZBAA ELKUR W40 YQG ZSPD" \
  --navdata "data/NavigraphDFDv2-2604.1.0.db?mode=ro" \
  --preferred-routes data/Route-Server.csv
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

### Discord event posts

Set `discord.event_forum_channel_id` to a Discord forum channel ID, in addition
to enabling the bot and configuring its token. If the forum requires tags, set
`discord.event_forum_tag_ids` to the permitted tag IDs. The bot needs View Channel,
Send Messages, Embed Links, Read Message History, and Send Messages in Threads
permissions there. Locked posts additionally require Manage Threads to reopen.

Event coordinators can publish a saved website event from its edit dialog. The
service stores the forum thread and guild IDs and returns `discord_message` in the
event API. `PUT /api/events/{id}/discord` creates the post once, or synchronizes
its existing title and starter message. Event updates automatically synchronize
linked posts. A failed sync preserves the website changes and exposes
`discord_message.status = OutOfSync`; the editor offers a retry. Unpublished events are never
automatically published. Long descriptions are shortened to Discord's embed
limit, with a link to the full website event. Mentions are disabled.

Run database migrations before deploying this version. Discord associations live
in `event_discord_message`, keyed by `event_id`. Its `status` column is a text enum
restricted to `Sync` and `OutOfSync`; `synced_at` records the last successful sync.
Failed updates retain that timestamp. First publication failures create no
association and return an API error.

Discord requests run after website changes are committed, without holding a
transaction or row lock. Concurrent first publications can create duplicate posts;
there is no cross-system atomicity. Local tests use a mock Discord HTTP server.

The API exposes `discord_message` with string `guild_id` and `message_id`
snowflakes, `status`, and `synced_at`.
