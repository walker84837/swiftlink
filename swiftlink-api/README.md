# swiftlink-api

> Shared API contract and client SDK for the Swiftlink URL shortener.

The core library for interacting with the Swiftlink server, providing types, error handling, and both async/blocking clients.

## Quick start

Add to `Cargo.toml`:

```bash
cargo add swiftlink-api --features async # or blocking
```

## Examples

**Async client** (feature = `"async"`):
```rust
use swiftlink_api::{AsyncSwiftlinkClient, SwiftlinkClientError};

async fn create_link() -> Result<(), SwiftlinkClientError> {
    let client = AsyncSwiftlinkClient::new("http://localhost:8080");
    let response = client.create_link("https://example.com/very/long/path").await?;
    println!("Created: {}", response.code);
    Ok(())
}
```

**Blocking client** (feature = `"blocking"`):
```rust
use swiftlink_api::{BlockingSwiftlinkClient, SwiftlinkClientError};

fn get_link() -> Result<(), SwiftlinkClientError> {
    let client = BlockingSwiftlinkClient::new("http://localhost:8080");
    let info = client.get_link_info("abc123")?;
    println!("URL: {}", info.url);
    Ok(())
}
```

## Features

| Feature | Description |
|---------|-------------|
| `async` | Enables `AsyncSwiftlinkClient` (requires `reqwest` + `tokio`) |
| `blocking` | Enables `BlockingSwiftlinkClient` (requires `reqwest`) |

Enable one or both. Default: none.

## License

Dual-licensed under Apache-2.0 and MIT, at your option.
