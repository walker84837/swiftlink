# Server Setup

This guide covers setting up and deploying the Swiftlink server.

## Prerequisites

- **Rust** 1.85.0 or later
- **Database**: PostgreSQL (recommended) or SQLite
- **System**: Linux, macOS, or Windows

## Building the Server

```bash
# Clone the repository
git clone https://github.com/walker84837/swiftlink.git
cd swiftlink

# Build the server
cargo build --release -p swiftlink-server

# The binary will be at target/release/swiftlink-server
```

## Database Setup

### PostgreSQL (Recommended)

1. **Install PostgreSQL**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install postgresql postgresql-contrib
   
   # macOS (with Homebrew)
   brew install postgresql
   brew services start postgresql
   ```

2. **Create Database**:
   ```sql
   -- Connect to PostgreSQL
   psql -U postgres
   
   -- Create database and user
   CREATE DATABASE swiftlink_db;
   CREATE USER swiftlink_user WITH PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE swiftlink_db TO swiftlink_user;
   ```

3. **Test Connection**:
   ```bash
   psql -h localhost -U swiftlink_user -d swiftlink_db
   ```

### SQLite

For development or small deployments:

```bash
# No setup required - just specify a file path in the configuration
```

## Configuration

Create a configuration file `config.toml`:

### Minimal PostgreSQL Configuration

```toml
[base]
port = 8080
code_size = 6

[database]
database_type = "postgres"
username = "swiftlink_user"
password = "your_password"
host = "localhost"
port = 5432
database = "swiftlink_db"
max_connections = 5
```

### Minimal SQLite Configuration

```toml
[base]
port = 8080
code_size = 6

[database]
database_type = "sqlite"
database = "./swiftlink.db"
max_connections = 5
```

### Security Configuration

For production use, add a bearer token for DELETE operations:

```toml
[base]
# Generate a secure token, e.g. with: openssl rand -hex 20
bearer_token = "your-secure-bearer-token-here"
```

!!! warning "Security"
    The `bearer_token` is required for DELETE operations. If not specified, the server will generate one at startup and log it. For production, always set a secure token in your configuration.

## Running the Server

### Development

```bash
# From the repository root
cargo run -p swiftlink-server -- --config config.toml
```

### Production

```bash
# Using the release binary
./target/release/swiftlink-server --config /path/to/config.toml
```

### Systemd Service (Linux)

Create `/etc/systemd/system/swiftlink.service`:

```ini
[Unit]
Description=Swiftlink URL Shortener
After=network.target

[Service]
Type=simple
User=swiftlink
WorkingDirectory=/opt/swiftlink
ExecStart=/opt/swiftlink/swiftlink-server --config /opt/swiftlink/config.toml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable swiftlink
sudo systemctl start swiftlink
```

## Docker Deployment

Create a `Dockerfile`:

```dockerfile
FROM rust:1.85.0 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p swiftlink-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/swiftlink-server /usr/local/bin/
EXPOSE 8080
CMD ["swiftlink-server", "--config", "/config/config.toml"]
```

Build and run:

```bash
docker build -t swiftlink .
docker run -p 8080:8080 -v $(pwd)/config.toml:/config/config.toml swiftlink
```

## Reverse Proxy Setup

### Nginx

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Apache

```apache
<VirtualHost *:80>
    ServerName your-domain.com
    ProxyPreserveHost On
    ProxyRequests Off
    ProxyPass / http://localhost:8080/
    ProxyPassReverse / http://localhost:8080/
</VirtualHost>
```

## Monitoring and Logging

The server uses structured logging. Configure log levels:

```bash
# Debug level
./swiftlink-server --config config.toml --log-level debug

# Info level (default)
./swiftlink-server --config config.toml --log-level info

# Error level only
./swiftlink-server --config config.toml --log-level error
```

Logs include:
- Link creation/deletion events
- Database connection issues
- HTTP request errors
- Configuration warnings

## Performance Tuning

### Database Connections

Adjust `max_connections` based on expected load:

```toml
[database]
max_connections = 20  # For high-traffic sites
```

### Code Length

 shorter codes = faster lookups but higher collision risk:

```toml
[base]
code_size = 4  # For high-throughput, low-concurrency use
```

## Troubleshooting

### Common Issues

1. **Database Connection Failed**:
   - Verify database credentials
   - Check network connectivity
   - Ensure database is running

2. **Port Already in Use**:
   - Change the port in configuration
   - Check for conflicting services

3. **Permission Denied**:
   - Ensure the config file is readable
   - Check file permissions for SQLite database

### Health Check

Verify the server is running:

```bash
curl http://localhost:8080/api/create -d '{"url":"https://example.com"}' -H "Content-Type: application/json"
```

For more configuration options, see the [Configuration Guide](configuration.md).