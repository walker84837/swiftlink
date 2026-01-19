# Installation

This guide covers various installation methods for Swiftlink, from pre-built binaries to source compilation.

## Installation Methods

### 1. From Source (Recommended for Development)

Clone and build from the repository:

```bash
# Clone the repository
git clone https://github.com/walker84837/swiftlink.git
cd swiftlink

# Build all components
cargo build --workspace --release

# Binaries will be in target/release/
```

### 2. Pre-built Binaries

Download pre-compiled binaries for your platform (when releases are available):

```bash
# Example for Linux x64
wget https://github.com/walker84837/swiftlink/releases/latest/download/swiftlink-linux-x64.tar.gz
tar -xzf swiftlink-linux-x64.tar.gz
```

### 3. Using Cargo Install

Install directly from crates.io (when published):

```bash
# Install all components
cargo install swiftlink-server swiftclient

# Or install individual components
cargo install swiftlink-server
cargo install swiftclient
```

### 4. Docker

Build from Dockerfile:

```bash
# Build the image
docker build -t swiftlink .

# Run the container
docker run -p 8080:8080 -v $(pwd)/config.toml:/config/config.toml swiftlink
```

## System Requirements

### Minimum Requirements
- **CPU**: 1 core
- **Memory**: 128MB RAM
- **Storage**: 100MB for application + database
- **OS**: Linux, macOS, or Windows

### Recommended Requirements
- **CPU**: 2+ cores
- **Memory**: 512MB RAM
- **Storage**: 1GB for application + database growth
- **Network**: Reliable internet connection

## Dependencies

### Required by Users
- **Database**: PostgreSQL 9.6+ or SQLite 3.0+
- **Reverse Proxy** (optional): Nginx, Apache, or Caddy

### Required for Development
- **Rust**: 1.85.0 or later
- **Cargo**: Comes with Rust
- **Git**: For cloning the repository

## Verification

After installation, verify the components work:

### Server Verification

```bash
# Test server with default config
./swiftlink-server --config example/config.toml

# In another terminal, test the API
curl -X POST http://localhost:8080/api/create \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com"}'
```

### CLI Verification

```bash
# Test CLI with server
./swiftclient --base-url http://localhost:8080 create https://example.com
```

## Directory Structure

After installation, your directory should look like:

```
swiftlink/
├── swiftlink-server      # Main server binary
├── swiftclient          # CLI tool
├── config.toml          # Server configuration
├── example/             # Example configurations
└── database/            # Database files (if using SQLite)
```

## Configuration Files

Copy the example configuration:

```bash
# Create config directory
mkdir -p ~/.config/swiftlink

# Copy example config
cp example/config.toml ~/.config/swiftlink/
```

## Service Setup

### systemd (Linux)

Create `/etc/systemd/system/swiftlink.service`:

```ini
[Unit]
Description=Swiftlink URL Shortener
After=network.target

[Service]
Type=simple
User=swiftlink
Group=swiftlink
WorkingDirectory=/opt/swiftlink
ExecStart=/opt/swiftlink/swiftlink-server --config /etc/swiftlink/config.toml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable swiftlink
sudo systemctl start swiftlink
```

### macOS LaunchAgent

Create `~/Library/LaunchAgents/com.swiftlink.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.swiftlink</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/swiftlink-server</string>
        <string>--config</string>
        <string>/etc/swiftlink/config.toml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load the service:

```bash
launchctl load ~/Library/LaunchAgents/com.swiftlink.plist
```

## Upgrading

### From Source

```bash
cd swiftlink
git pull origin main
cargo build --workspace --release
```

### From Binary

```bash
# Download new binary
# Stop service
sudo systemctl stop swiftlink

# Replace binary
sudo cp new-swiftlink-server /usr/local/bin/

# Start service
sudo systemctl start swiftlink
```

## Post-Installation

1. **Configure Database**: Set up PostgreSQL or SQLite
2. **Set Up Reverse Proxy**: Configure Nginx/Apache for production
3. **Configure DNS**: Point your domain to the server
4. **SSL/TLS**: Set up HTTPS certificates
5. **Monitoring**: Set up logs and monitoring

## Next Steps

After installation, check out:

- [Server Setup Guide](server/setup.md) - Configuration and deployment
- [CLI Usage](cli/swiftclient.md) - Command-line interface
- [API Documentation](api/index.md) - REST API reference
- [Troubleshooting](troubleshooting.md) - Common issues and solutions