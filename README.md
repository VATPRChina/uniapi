# VATPRC UniAPI

This project targets providing an universal API server for all the services VATPRC requires.

- [ ] Slot (**Ongoing**)
- [ ] ATC Management
- [ ] CORS Proxy
- [ ] Discord Bot

## Architecture

This project follows a modified version of [The Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html).

The `entity` folder contains core business entities, which also serves as database models.

The `usecase` folder contains business logic for the manipulation of entities.

The `external` folder is for external interfaces, including database, REST API, etc. We do not have interface adapters, but instead, the implementation is also considered as an interface to avoid over-designing.
