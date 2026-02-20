# Configuration

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
| `base.bearer_token` | Authentication token for `DELETE` operations | (None - required for DELETE) |
| `base.rate_limit.enabled` | Enable or disable rate limiting | `true` |
| `base.rate_limit.max_requests` | Maximum requests per time window | `10` |
| `base.rate_limit.window_seconds` | Time window for rate limiting in seconds | `60` |
| `base.rate_limit.trust_proxy_headers` | Trust X-Forwarded-For and Forwarded headers | `false` |
| `base.rate_limit.max_tracked_clients` | Maximum unique clients to track (prevents memory abuse) | `10000` |
