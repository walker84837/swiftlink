# swiftlink-api

> A simple library featuring a client and API types for Swiftlink server.

The core API library for interacting with the Swiftlink URL shortening service, providing common data structures, error handling, and client implementations for interacting with the server.

## Examples

**Example of using the async client to create a link**:
```rust
#[cfg(feature = "async")]
async fn example_async_create_link() -> Result<(), swiftlink_api::SwiftlinkClientError> {
    let client = swiftlink_api::AsyncSwiftlinkClient::new("http://localhost:8080");
    let response = client.create_link("https://www.example.com/very/long/path").await?;
    println!("Created short link: {}", response.code);
    Ok(())
}
```

**Example of using the blocking client to get link info**:
```rust
#[cfg(feature = "blocking")]
fn example_blocking_get_link_info() -> Result<(), swiftlink_api::SwiftlinkClientError> {
    let client = swiftlink_api::BlockingSwiftlinkClient::new("http://localhost:8080");
    let response = client.get_link_info("some_code")?;
    println!("Link info: URL = {}, Created At = {}", response.url, response.created_at);
    Ok(())
}
```

## Features

The `swiftlink-api` crate provides the following key features:

- **Common data structures**: Defines shared `struct`s for API requests and responses, such as `CreateLinkRequest`, `CreateLinkResponse`, and `InfoResponse`, ensuring type safety and consistency across the ecosystem.
- **Client features**: The client allows you to:
  - Creating new short links (`/api/create`).
  - Retrieving information about existing links (`/api/info/{code}`).
  - Resolving short links to their original URLs via redirection (`/{code}`).
  - Deleting short links (`/{code}` with DELETE method), including bearer token authentication.
  - Switch between asynchronous and blocking:
      - *Asynchronous Client*: Offers `AsyncSwiftlinkClient` for non-blocking API calls, ideal for high-performance applications.
      - *Blocking client*: Provides `BlockingSwiftlinkClient` for synchronous API calls, suitable for simpler scripts or environments where async is not supported.
- **Specialized errors**: Includes an `SwiftlinkClientError` enum to manage various error conditions, providing clear error messages.
- **Unified API**: Re-exports core components, allowing users to easily access both async and blocking client implementations and common types through a single, consistent interface.

## Roadmap

- [ ] ...
- [ ] ...
- [ ] ...

## License

This project is licensed under the MIT and Apache 2.0 license, either at your own choice.
