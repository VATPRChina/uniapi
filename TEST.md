# Testing

There is E2E testing for each API endpoint. It is still missing test coverage on some endpoints.

## Writing tests

Place the test for each API endpoint in `/e2e/src/**/*.test.ts`. The test folder structure should match the API endpoint's URL. For example, test for GET `/api/atc/applications/review-sheet` should be placed in `/e2e/src/atc/applications/review-sheet.test.ts` (prefix `/api` is stripped).

If a user is needed, check `getClient` from `/e2e/lib/backend.ts`. Use Vitest's `extend` to establish a shared context for different roles used in the same test file.

If some common data setup is required for the API endpoint (e.g. `/api/events` requires a present event), use Vitest's `extend` to establish a shared context to avoid duplication in setting up the data.

## Checklist

| Method | Endpoint                                                   | Status  |
| ------ | ---------------------------------------------------------- | ------- |
| GET    | `/`                                                        | MISSING |
| GET    | `/api/atc/applications/{id}`                               | MISSING |
| PUT    | `/api/atc/applications/{id}`                               | MISSING |
| GET    | `/api/atc/applications/sheet`                              | MISSING |
| GET    | `/api/atc/applications/review-sheet`                       | MISSING |
| PUT    | `/api/atc/applications/{id}/review`                        | MISSING |
| GET    | `/api/atc/applications`                                    | MISSING |
| POST   | `/api/atc/applications`                                    | MISSING |
| GET    | `/api/atc/controllers`                                     | MISSING |
| PUT    | `/api/atc/trainings/{id}/record`                           | MISSING |
| DELETE | `/api/atc/trainings/{id}`                                  | MISSING |
| GET    | `/api/atc/trainings/{id}`                                  | MISSING |
| PUT    | `/api/atc/trainings/{id}`                                  | MISSING |
| GET    | `/api/atc/trainings/active`                                | MISSING |
| PUT    | `/api/atc/trainings/applications/{id}/response`            | MISSING |
| GET    | `/api/atc/trainings/applications/{id}/responses`           | MISSING |
| DELETE | `/api/atc/trainings/applications/{id}`                     | MISSING |
| GET    | `/api/atc/trainings/applications/{id}`                     | MISSING |
| PUT    | `/api/atc/trainings/applications/{id}`                     | MISSING |
| GET    | `/api/atc/trainings/applications`                          | MISSING |
| POST   | `/api/atc/trainings/applications`                          | MISSING |
| GET    | `/api/atc/trainings/finished`                              | MISSING |
| GET    | `/api/atc/trainings/record-sheet`                          | MISSING |
| POST   | `/api/atc/trainings`                                       | MISSING |
| GET    | `/api/compat/euroscope/metar/{icao}`                       | MISSING |
| GET    | `/api/compat/euroscope/metar/metar.php`                    | MISSING |
| GET    | `/api/compat/online-status`                                | MISSING |
| GET    | `/api/compat/trackaudio/mandatory_version`                 | MISSING |
| GET    | `/api/compat/vplaaf/areas.json`                            | MISSING |
| POST   | `/api/events/{event_id}/airspaces`                         | OK      |
| DELETE | `/api/events/{event_id}/controllers/{position_id}/booking` | MISSING |
| PUT    | `/api/events/{event_id}/controllers/{position_id}/booking` | MISSING |
| DELETE | `/api/events/{event_id}/controllers/{position_id}`         | MISSING |
| PUT    | `/api/events/{event_id}/controllers/{position_id}`         | MISSING |
| GET    | `/api/events/{event_id}/controllers`                       | MISSING |
| POST   | `/api/events/{event_id}/controllers`                       | MISSING |
| DELETE | `/api/events/{event_id}/slots/{slot_id}/booking`           | MISSING |
| PUT    | `/api/events/{event_id}/slots/{slot_id}/booking`           | MISSING |
| GET    | `/api/events/{event_id}/slots/bookings.csv`                | MISSING |
| GET    | `/api/events/{event_id}/slots`                             | OK      |
| POST   | `/api/events/{event_id}/slots`                             | OK      |
| GET    | `/api/events/{id}`                                         | OK      |
| PUT    | `/api/events/{id}`                                         | OK      |
| GET    | `/api/events/past`                                         | OK      |
| GET    | `/api/events`                                              | OK      |
| POST   | `/api/events`                                              | OK      |
| GET    | `/api/flights/active`                                      | MISSING |
| GET    | `/api/flights/by-callsign/{callsign}/route`                | MISSING |
| GET    | `/api/flights/by-callsign/{callsign}/warnings`             | MISSING |
| GET    | `/api/flights/by-callsign/{callsign}`                      | MISSING |
| GET    | `/api/flights/mine`                                        | MISSING |
| GET    | `/api/flights/temporary/by-plan/warnings`                  | MISSING |
| GET    | `/api/navdata/preferred-routes/{id}`                       | MISSING |
| PUT    | `/api/navdata/preferred-routes/{id}`                       | MISSING |
| GET    | `/api/navdata/preferred-routes`                            | MISSING |
| POST   | `/api/navdata/preferred-routes`                            | MISSING |
| GET    | `/api/sectors/current/permission`                          | MISSING |
| DELETE | `/api/session`                                             | MISSING |
| GET    | `/api/session`                                             | OK      |
| POST   | `/api/storage/images`                                      | MISSING |
| PUT    | `/api/users/{id}/atc/status`                               | MISSING |
| PUT    | `/api/users/{id}/roles`                                    | MISSING |
| GET    | `/api/users/me/atc/status`                                 | MISSING |
| GET    | `/api/users/me`                                            | MISSING |
| GET    | `/api/users`                                               | MISSING |
| POST   | `/auth/__unsafe_assume_user`                               | OK      |
| GET    | `/auth/authorize`                                          | OK      |
| GET    | `/auth/callback/vatsim`                                    | MISSING |
| POST   | `/auth/device_authorization`                               | OK      |
| GET    | `/auth/device`                                             | MISSING |
| GET    | `/auth/login`                                              | MISSING |
| POST   | `/auth/token`                                              | OK      |
| GET    | `/health`                                                  | OK      |
| GET    | `/openapi.json`                                            | MISSING |
