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
| `server.auth_token` | Authentication token for `DELETE` operations | (None - required for DELETE) |
