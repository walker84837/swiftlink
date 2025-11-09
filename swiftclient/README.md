# swiftclient

> A CLI client for Swiftlink.

swiftclient is a command-line interface (CLI) tool for interacting with the Swiftlink URL shortening service. It allows users to:
- create new short links
- retrieve information about existing links
- and delete links
all from the comfort of their terminal.

## Features

As listed before:
- **Create shortened links**: Shorten long URLs into concise, shareable Swiftlink codes.
- **Get link information**: Retrieve the original URL and creation timestamp for any Swiftlink code.
- **Delete short links**: Remove unwanted short links from the Swiftlink service (requires authentication, of course).

## Usage

swiftclient requires a `--base-url` argument to specify the address of the Swiftlink server it should connect to.

### Create a new short link

To create a new short link, use the `create` subcommand followed by the URL you wish to shorten:

```bash
swiftclient --base-url http://localhost:8080 create <YOUR_LONG_URL>
```

Example:
```bash
swiftclient --base-url http://go.example.com create https://www.example.com/very/long/path/to/a/resource
# Output: Short link created: <generated_code>
```

### Get information about a short link

To retrieve details about an existing short link, use the `info` subcommand with the link's code:

```bash
swiftclient --base-url http://localhost:8080 info <SHORT_CODE>
```

Example:
```bash
swiftclient --base-url http://localhost:8080 info abcdef
# Output: Link info for abcdef: URL = https://www.example.com/..., Created At = 1678886400
```

### Delete a short link

To delete a short link, use the `delete` subcommand with the link's code and a bearer token for authentication. The bearer token is configured on the Swiftlink server.

```bash
swiftclient --base-url http://localhost:8080 delete <SHORT_CODE> --token <YOUR_BEARER_TOKEN>
```

Example:
```bash
swiftclient --base-url http://localhost:8080 delete abcdef --token YOUR_SECRET_TOKEN
# Output: Link abcdef deleted.
```

## Installation

To build and install swiftclient, ensure you have Rust and Cargo installed (with [rustup](https://rustup.rs)). Then, navigate to the `swiftclient` directory and run:

```bash
cargo install --path .
```

This will install the `swiftclient` executable to your Cargo bin directory, typically `~/.cargo/bin`.

## License

This CLI is dual-licensed under the:
- [Apache-2.0](../LICENSE-APACHE)
- [MIT License](../LICENSE-MIT)
either at your own choice.
