# Swiftlink

> Blazingly fast, memory-efficient and safe URL shortener to fit **you**.

Swiftlink is a modular URL shortening service built with Rust. It consists of three main components:
- a strong Actix-web server (`swiftlink-server`);
- a core API definition and client SDK (`swiftlink-api`);
- and a command-line interface (`swiftclient`).

Swiftlink aims for high performance, security, and full customizability, backed by PostgreSQL (or SQLite) for data persistence. It's **easy to use, easy to configure.**

## Project Structure

* `swiftlink-api`Defines the shared API contract and provides both asynchronous (`async`) and blocking (`blocking`) HTTP client implementations for interacting with the Swiftlink server. This crate serves as the single source of truth for API request and response types.
* `swiftlink-server`An `actix-web` based HTTP server that implements the core URL shortening logic. It handles API requests, interacts with a PostgreSQL database via `sqlx`, and provides redirection functionality.
* `swiftclient`A command-line tool built with `clap` that utilizes the `swiftlink-api`'s blocking client to easily create, query, and delete short links from your terminal.

## Features

* **Modular design**: Clearly separated concerns with dedicated crates for API, server, and client.
* **High performance**: Built with Rust and Actix-web for speed and efficiency.
* **Robust API**: Well-defined endpoints for creating, retrieving information about, and deleting short links.
* **Database support**: Integrates with PostgreSQL (and SQLite) via `sqlx` for reliable data storage.
* **CLI tool**: Convenient command-line client for easy interaction with the service.
* **Customizable**: Flexible configuration options for server and database settings.

## Quick Start

To get Swiftlink up and running locally, follow these steps:

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/walker84837/swiftlink.git
    cd swiftlink
    ```

2.  **Database Setup:**
    Swiftlink uses PostgreSQL (you can also configure it to use SQLite). Ensure you have one of these databases installed.
    For PostgreSQL, create a database (e.g., `swiftlink_db`) and a user with appropriate permissions.
    You can configure database connection details in `example/config.toml`.

3.  **Build the project**:
  ```sh
  cargo build --workspace --release
  ```
  This command will build all three crates (`swiftlink-api`, `swiftlink-server`, `swiftclient`).

4.  **Run the Swiftlink server:**
  ```sh
  ./target/release/swiftlink-server --config example/config.toml
  ```
  The server will start on the port specified in your `config.toml` (default 8080).
  You can then access the API at `http://localhost:8080`.

5.  **Use the Swiftclient CLI:**
  ```sh
  # Create a short link
  ./target/release/swiftclient create --url "https://www.google.com"

  # Get info about a short link (replace CODE with the generated short code)
  ./target/release/swiftclient info --code ABCDEF

  # Delete a short link (requires an authentication token)
  ./target/release/swiftclient delete --code CODE --auth-token YOUR_AUTH_TOKEN
  ```

## API reference

The Swiftlink API allows you to programmatically interact with the URL shortener service. The `swiftlink-api` crate defines the request/response types and provides client implementations.

### Endpoints

| Endpoint      | URL       | Method | Authentication  | Request Body            | Response             | Error Handling          |
| :------------------------ | :-------------------- | :----- | :---------------- | :------------------------------------------ | :----------------------------------------- | :------------------------------------------ |
| Create Link     | `/api/create`   | `POST` | None      | `application/json`: `{ "url": "long_url" }` | `application/json`: `{ "code": "CODE", "url": "long_url" }` | `400 Bad Request`, `500 Internal Server Error` |
| Get Link Information  | `/api/info/{code}`  | `GET`  | None      | N/A               | `application/json`: `{ "code": "CODE", "created_at": "TIMESTAMP", "url": "long_url" }` | `404 Not Found`           |
| Redirect      | `/{code}`     | `GET`  | None      | N/A               | `302 Found` with `Location` header   | `404 Not Found`           |
| Delete Link     | `/api/delete/{code}`  | `DELETE` | Bearer Token  | N/A               | `204 No Content`         | `401 Unauthorized`, `404 Not Found`   |

## Configuration

The `swiftlink-server` uses a TOML configuration file. An example can be found at `example/config.toml`.

| Value     | Description             | Default    |
| :---------------- | :-------------------------------------------- | :----------------- |
| `code_size`   | Length of the generated short link code   | `6`      |
| `port`    | Port for the web server to listen on    | `8080`     |
| `database.username` | Database username         | `postgres`   |
| `database.password` | Database password         | `password`   |
| `database.host` | Database host           | `localhost`    |
| `database.port` | Database port           | `5432`     |
| `database.database` | Database name           | `swiftlink_db`   |
| `database.max_connections` | Maximum database connections   | `5`      |
| `server.auth_token` | Authentication token for `DELETE` operations | (None - required for DELETE) |

## Contributions

Swiftlink is free and open-source. We welcome contributions from the community! Feel free to [contribute](https://github.com/walker84837/swiftlink/contribute)! Please visit our GitHub repository to review the contribution guidelines and submit pull requests. Help us enhance the project for everyone.

## Roadmap

### Functionality

* [ ] Implement authentication support more broadly (e.g., for `POST /api/create`).
* [ ] Address the type duplication in `swiftlink-server` by consistently using types from `swiftlink-api`.
* [ ] Complete the `async` client in `swiftlink-api` by adding the `delete_link` method.
* [ ] Improve error handling and provide more descriptive error responses.
* Landing page:
  * [ ] Simple install with `curl` command and `install.sh` file.

### Look and feel

* Landing page
  * [ ] Make project logo for `favicon.ico`.
  * [ ] Develop a templated and "specialized" page for the `swiftlink-server` with dynamic content (e.g., domain, port) and information on the service that's running.

## License

Dual-licensed under Apache v2 and MIT, either at your option.
