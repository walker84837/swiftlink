# Using the Rust SDK

Swiftlink provides an official SDK for Rust. It includes two client libraries: an async client and a blocking client (in case your app uses [Tokio](https://tokio.rs)).

## Code Examples

### Async Client

```rust
use swiftlink_api::AsyncSwiftlinkClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AsyncSwiftlinkClient::new("https://your-domain.com");
    
    let response = client.create_link("https://example.com").await?;
    println!("Short code: {}", response.code);
    
    Ok(())
}
```

### Blocking Client

```rust
use swiftlink_api::BlockingSwiftlinkClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BlockingSwiftlinkClient::new("https://your-domain.com");
    
    let response = client.create_link("https://example.com")?;
    println!("Short code: {}", response.code);
    
    Ok(())
}
```

## Data Types

The SDK contains the following data types which represent a request and response for each endpoint. Here's a few of hem (derives and docs omitted for simplicity):

### Request Types

```rust
pub struct CreateLinkRequest {
    pub url: String,
}
```

### Response Types

```rust
pub struct CreateLinkResponse {
    pub code: String,
    pub url: String,
}

pub struct InfoResponse {
    pub code: String,
    pub url: String,
    pub created_at: i64,
}
```
