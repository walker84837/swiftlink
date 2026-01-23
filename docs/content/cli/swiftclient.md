# Swiftclient CLI

`swiftclient` is the command-line interface for interacting with a Swiftlink server. It provides convenient commands for creating, managing, and retrieving information about short links.

## Installation

The CLI is built as part of the Swiftlink workspace:

```bash
cargo build --release -p swiftclient
```

The binary will be available at `target/release/swiftclient`.

## Usage

All commands require the base URL of your Swiftlink server:

```bash
swiftclient --base-url https://your-swiftlink-domain.com <command>
```

### Commands

#### Create a short link

```bash
swiftclient --base-url https://example.com create https://long-url-to-shorten.com/with/parameters
```

**Example:**
```bash
$ swiftclient --base-url http://localhost:8080 create https://github.com/walker84837/swiftlink
Short link created: abc123
```

#### Get link information

```bash
swiftclient --base-url https://example.com info <code>
```

**Example:**
```bash
$ swiftclient --base-url http://localhost:8080 info abc123
Link info for abc123: URL = https://github.com/walker84837/swiftlink, Created At = 1641024000
```

#### Delete a short link

```bash
swiftclient --base-url https://example.com delete <code> --token <bearer-token>
```

**Example:**
```bash
$ swiftclient --base-url http://localhost:8080 delete abc123 --token your-secret-token
Link abc123 deleted.
```

!!! note "Authentication"
    The `delete` command requires a bearer token that must be configured in your Swiftlink server's configuration file. The token is specified using the `bearer_token` field in the `[base]` section.

### Command Reference

| Command | Description | Required Arguments | Optional Arguments |
|:--------|:------------|:-------------------|:-------------------|
| `create` | Create a new short link | `url` - The URL to shorten | None |
| `info` | Get information about a short link | `code` - The short link code | None |
| `delete` | Delete a short link | `code` - The short link code | `--token` - Bearer token for authentication |

### Global Options

| Option | Short | Description | Required |
|:-------|:------|:-------------|:---------|
| `--base-url` | `-b` | Base URL of the Swiftlink server | Yes |

### Examples

**Basic workflow:**
```bash
# Create a short link
swiftclient --base-url http://localhost:8080 create https://example.com/very/long/url

# Get information about it
swiftclient --base-url http://localhost:8080 info abc123

# Delete it (requires token)
swiftclient --base-url http://localhost:8080 delete abc123 --token my-secret-token
```

**Using with a remote server:**
```bash
swiftclient --base-url https://link.shortener.com create https://blog.example.com/my-article
```

### Error Handling

The CLI provides clear error messages for common issues:

- Network connectivity problems
- Invalid URLs
- Missing or expired tokens
- Non-existent short links

For API-specific error codes and responses, see the [API documentation](../api/endpoints.md).
