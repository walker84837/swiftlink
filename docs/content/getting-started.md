# Getting started

This guide helps you run Swiftlink locally for the first time.

## Requirements

- Rust (stable)
- PostgreSQL or SQLite

## Clone the repository

```sh
git clone https://github.com/walker84837/swiftlink.git
cd swiftlink
````

## Build everything

```sh
cargo build --workspace --release
```

## Run the server

```sh
./target/release/swiftlink-server --config example/config.toml
```

The server listens on port `8080` by default.

## Try it out

Use the CLI to create a short link:

```sh
./target/release/swiftclient create --url https://example.com
```

You should now be able to visit:

```
http://localhost:8080/<CODE>
```
