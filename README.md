# Swiftlink

> Blazingly fast, memory-efficient and safe URL shortener to fit **you**.

Swiftlink is a URL shortener built with Rust and [Actix](https://actix.rs). It enabled you to create short links from long URLs and provides a simple API for managing these links. The service uses PostgreSQL for database operations.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
  - [Configuration](#configuration)
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

## Contributions

Contributions are welcome! Please feel free to submit issues and pull requests.

### Roadmap

- [ ] Add authentication support (JWT or similar).
- [ ] Make project logo for favicon.ico.
- [ ] Add /api/info endpoint.
- [ ] Deduplicate links.
- [ ] Check if links exist before creating.

## License

Dual-licensed under Apache v2 or MIT.
