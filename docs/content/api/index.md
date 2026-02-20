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

Swiftlink includes built-in rate limiting to prevent abuse and ensure fair usage. Rate limits are enforced per-client IP address and per-route pattern.

### Rate Limit Headers

When a request is rate-limited, the server responds with `429 Too Many Requests` and includes headers indicating the rate limit status:

- `X-RateLimit-Limit` - Maximum requests allowed in the window
- `X-RateLimit-Remaining` - Requests remaining in the current window
- `X-RateLimit-Reset` - Unix timestamp when the rate limit resets

### Configuration

Rate limiting is configured in the server's configuration file:

| Setting | Description | Default |
|---------|-------------|---------|
| `enabled` | Enable/disable rate limiting | `true` |
| `max_requests` | Maximum requests per window | `10` |
| `window_seconds` | Time window in seconds | `60` |
| `trust_proxy_headers` | Trust X-Forwarded-For headers | `false` |
| `max_tracked_clients` | Maximum unique clients to track | `10000` |

Example configuration:

```toml
[base.rate_limit]
enabled = true
max_requests = 10
window_seconds = 60
trust_proxy_headers = false
max_tracked_clients = 10000
```

!!! note
    When running behind a reverse proxy (like Nginx or Caddy), set `trust_proxy_headers = true` to ensure rate limiting works correctly based on the original client IP.

## CORS

[TODO](../roadmap.md)

## Endpoint Reference

For detailed endpoint documentation including request/response examples, see the [Endpoints documentation](endpoints.md).
