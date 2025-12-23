# API overview

Swiftlink exposes a simple HTTP API for managing short links.

The API is defined in the `swiftlink-api` crate, which serves as the
single source of truth for request and response types.

## Authentication

- Most endpoints are public
- Deletion requires a bearer token

See the endpoint reference for details.
