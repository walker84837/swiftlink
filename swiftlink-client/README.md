# swiftlink-client

> Command-line tool for managing Swiftlink short links.

## Install

```bash
cargo install swiftlink-client
```

## Usage

```bash
swiftlink-client <BASE_URL> <COMMAND>
```

### Commands

| Command | Description | Example |
|---------|-------------|---------|
| `create <URL>` | Create a new short link | `swiftlink-client http://localhost:8080 create "https://example.com/long/path"` |
| `info <CODE>` | Get link metadata | `swiftlink-client http://localhost:8080 info abc123` |
| `delete <CODE> --token <TOKEN>` | Delete a link (requires auth) | `swiftlink-client http://localhost:8080 delete abc123 --token my-bearer-token` |

### Examples

```bash
# Create a short link
$ swiftlink-client http://localhost:8080 create "https://github.com/walker84837/swiftlink"
Short link created: Xy7kP9

# Get link info
$ swiftlink-client http://localhost:8080 info Xy7kP9
Link info for Xy7kP9: URL = https://github.com/walker84837/swiftlink, Created At = 1699999999

# Delete a link (requires bearer token from server config)
$ swiftlink-client http://localhost:8080 delete Xy7kP9 --token aB3xY9kQ2m
Link Xy7kP9 deleted.
```

## Configuration

The `BASE_URL` should point to your Swiftlink server (e.g., `http://localhost:8080` or `https://links.example.com`).

For `delete`, the bearer token must match the `bearer_token` in the server's config.

## License

Dual-licensed under Apache-2.0 and MIT, at your option.