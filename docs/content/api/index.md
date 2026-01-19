# API Overview

Swiftlink exposes a simple REST API for managing short links. The API is designed to be lightweight, fast, and easy to integrate with any programming language.

## Architecture

The API is defined in the `swiftlink-api` crate, which serves as the single source of truth for:
- Request and response types
- Error handling
- Client implementations (both async and blocking)

This ensures type safety and consistency between the server and client libraries.

## Base URL

All API endpoints are relative to your Swiftlink server's base URL:
```
https://your-domain.com/api/
```

## Authentication

Swiftlink uses a simple bearer token authentication system:

### Public Endpoints
- `POST /api/create` - Create short links
- `GET /api/info/{code}` - Get link information  
- `GET /{code}` - Redirect to original URL

### Protected Endpoints
- `DELETE /{code}` - Delete short links (requires bearer token)

### Bearer Token Usage
For protected endpoints, include the token in the `Authorization` header:

```http
Authorization: Bearer your-secure-token-here
```

The bearer token is configured in the server's configuration file under `base.bearer_token`. If not set, the server generates one at startup and logs it.

## Content Types

All API requests and responses use JSON format:

```http
Content-Type: application/json
```

## Error Handling

The API returns standard HTTP status codes:

- `200 OK` - Request successful
- `302 Found` - Redirect successful
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Missing or invalid authentication
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

Error responses include descriptive messages:

```json
{
  "error": "Invalid URL format"
}
```

## Rate Limiting

Currently, Swiftlink does not implement rate limiting. Consider adding reverse proxy rate limiting for production deployments.

## CORS

The server does not include built-in CORS headers. For web applications, configure CORS at your reverse proxy level.

## Using the Client Library

Swiftlink provides official client libraries for Rust:

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

## Endpoint Reference

For detailed endpoint documentation including request/response examples, see the [Endpoints documentation](endpoints.md).
