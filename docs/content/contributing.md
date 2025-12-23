# Contributing to Swiftlink

Swiftlink is free and open-source software, and contributions are welcome.

## Ways to contribute

- Reporting bugs
- Improving documentation
- Implementing new features
- Refactoring and cleanup

## Development setup

- Rust stable
- PostgreSQL or SQLite

Clone the repository and build the workspace:

```sh
cargo build --workspace
````

## Code structure

* `swiftlink-server` — HTTP server and business logic
* `swiftlink-api` — shared API types and client
* `swiftclient` — command-line interface

## Pull requests

* Keep changes focused
* Run formatting and tests
* Describe *why* the change exists

If in doubt, open an issue first to discuss ideas.
