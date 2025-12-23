# Endpoints

## API reference

The Swiftlink API allows you to programmatically interact with the URL shortener service. The `swiftlink-api` crate defines the request/response types and provides client implementations.

### Endpoints

| Endpoint      | URL       | Method | Authentication  | Request Body            | Response             | Error Handling          |
| :------------------------ | :-------------------- | :----- | :---------------- | :------------------------------------------ | :----------------------------------------- | :------------------------------------------ |
| Create Link     | `/api/create`   | `POST` | None      | `application/json`: `{ "url": "long_url" }` | `application/json`: `{ "code": "CODE", "url": "long_url" }` | `400 Bad Request`, `500 Internal Server Error` |
| Get Link Information  | `/api/info/{code}`  | `GET`  | None      | N/A               | `application/json`: `{ "code": "CODE", "created_at": "TIMESTAMP", "url": "long_url" }` | `404 Not Found`           |
| Redirect      | `/{code}`     | `GET`  | None      | N/A               | `302 Found` with `Location` header   | `404 Not Found`           |
| Delete Link     | `/api/delete/{code}`  | `DELETE` | Bearer Token  | N/A               | `204 No Content`         | `401 Unauthorized`, `404 Not Found`   |
