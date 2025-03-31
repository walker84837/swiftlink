# Swiftlink

> Blazingly fast, memory-efficient and safe URL shortener to fit **you**.

Swiftlink is a URL shortener built with Rust and [Actix](https://actix.rs). It enabled you to create short links from long URLs and provides a simple API for managing these links. The service uses PostgreSQL for database operations.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
  - [Configuration](#configuration)
- [API](#api)
  - [Endpoints](#endpoints)
- [Contributions](#contributions)
  - [Roadmap](#roadmap)
- [License](#license)

## Features

- Create short links from long URLs.
- Customizability in mind, from database options to the web server.
- Database configuration for PostgreSQL.
- Flexible logging with configurable log levels.

## Usage

```sh
./swiftlink --config path/to/config.toml --log-level Info
```

### Configuration

The configuration file is in TOML format and should be placed wherever you want, given you specify the right path. We have an [example](example/config.toml) can config file.

| Value          | Description                                 | Default          |
|----------------|---------------------------------------------|------------------|
| `code_size`    | Length of the generated short link code    | 6                |
| `port`         | Port for the web server to listen on       | 8080             |
| `username`     | Database username                          | `postgres`       |
| `password`     | Database password                          | `password`       |
| `host`         | Database host                              | `localhost`      |
| `port`         | Database port                              | 5432             |
| `database`     | Database name                              | `swiftlink_db`   |
| `max_connections` | Maximum database connections             | 5                |

## API

This is the API for Swiftlink to create short links, retrieve information about existing links, and redirect to the original URL.

### Endpoints

| Endpoint | URL | Method | Request Body | Response | Error Handling |
| --- | --- | --- | --- | --- | --- |
| Create link | `/api/create` | POST | `application/json`: `{url}` | `application/json`: `{code, url}` | `400 Bad Request`, `500 Internal Server Error` |
| Get link info | `/api/info/{code}` | GET | N/A | `application/json`: `{code, created_at, url}` | `404 Not Found` |
| Redirect | `/{code}` | GET | N/A | `302 Found` with `Location` header | `404 Not Found` |

## Contributions

Contributions are welcome! Please feel free to submit issues and pull requests.

### Roadmap

**Functionality**:
- [ ] Add authentication support (JWT or similar).
- [X] Add /api/info endpoint.
- [X] Deduplicate links.
- [X] Check if links exist before creating.

**Look and feel**:
- [ ] Make project logo for favicon.ico.
- [ ] Add landing page:
  - [ ] Information about the project
  - [ ] Simple usage with cURL
  - [ ] Templating (handlebars) for the landing page (domain, port, etc)

## License

Dual-licensed under Apache v2 or MIT.
