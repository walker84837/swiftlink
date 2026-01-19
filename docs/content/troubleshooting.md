# Troubleshooting

This guide helps you diagnose and resolve common issues with Swiftlink.

## Common Issues

### Server Issues

#### Server Won't Start

**Symptoms**: Error message when running `swiftlink-server`

**Possible Causes**:

1. **Port Already in Use**
   ```bash
   # Check what's using the port
   netstat -tulpn | grep :8080
   # or
   lsof -i :8080
   ```
   
   **Solution**: Change the port in `config.toml`:
   ```toml
   [base]
   port = 8081
   ```

2. **Database Connection Failed**
   
   **PostgreSQL**:
   ```bash
   # Check if PostgreSQL is running
   sudo systemctl status postgresql
   
   # Test connection
   psql -h localhost -U your_user -d swiftlink_db
   ```
   
   **SQLite**:
   ```bash
   # Check file permissions
   ls -la swiftlink.db
   # Ensure writable
   chmod 664 swiftlink.db
   ```

3. **Configuration File Not Found**
   
   **Solution**: Create or specify the config file:
   ```bash
   # Create default config
   mkdir -p ~/.config/swiftlink
   cp example/config.toml ~/.config/swiftlink/
   
   # Or specify path explicitly
   ./swiftlink-server --config /path/to/config.toml
   ```

#### Server is Slow

**Possible Causes**:

1. **Database Performance**
   
   **PostgreSQL**:
   ```sql
   -- Check connection count
   SELECT count(*) FROM pg_stat_activity;
   
   -- Create indexes if missing
   CREATE INDEX IF NOT EXISTS idx_links_code ON links(code);
   CREATE INDEX IF NOT EXISTS idx_links_url ON links(url);
   ```
   
2. **High Load**
   
   **Solution**: Adjust configuration:
   ```toml
   [database]
   max_connections = 20
   ```

#### Memory Usage High

**Diagnosis**:
```bash
# Check memory usage
ps aux | grep swiftlink-server
```

**Solutions**:
- Reduce `max_connections` in config
- Monitor for memory leaks
- Use `ulimit` to limit memory

### API Issues

#### 404 Not Found on Short Links

**Possible Causes**:

1. **Link Never Existed**
   ```bash
   # Check if code exists in database
   # PostgreSQL:
   psql -d swiftlink_db -c "SELECT * FROM links WHERE code = 'yourcode';"
   
   # SQLite:
   sqlite3 swiftlink.db "SELECT * FROM links WHERE code = 'yourcode';"
   ```

2. **Case Sensitivity**
   
   **Solution**: Check the exact code case:
   ```bash
   ./swiftclient --base-url http://localhost:8080 info YourCode
   ```

#### 400 Bad Request

**Common Causes**:

1. **Invalid URL Format**
   
   **Valid URLs**:
   ```
   https://example.com
   http://example.com/path
   ftp://files.example.com
   ```
   
   **Invalid URLs**:
   ```
   not-a-url
   example.com  # Missing protocol
   ```

2. **Malformed JSON**
   
   **Check JSON syntax**:
   ```bash
   # Use jq to validate
   echo '{"url":"https://example.com"}' | jq .
   ```

#### 401 Unauthorized

**Cause**: Missing or invalid bearer token for DELETE operations

**Solution**:
```bash
# Check server logs for generated token (if not set)
# Or set token in config.toml
./swiftclient --base-url http://localhost:8080 delete CODE --token your-token
```

### CLI Issues

#### Command Not Found

**Causes**:
1. Binary not in PATH
2. Binary not built/installed

**Solutions**:
```bash
# Add to PATH (temporary)
export PATH=$PATH:$(pwd)/target/release

# Add to shell profile (permanent)
echo 'export PATH=$PATH:$(pwd)/target/release' >> ~/.bashrc
source ~/.bashrc

# Or install globally
sudo cp target/release/swiftclient /usr/local/bin/
```

#### Connection Refused

**Cause**: Server not running or wrong address

**Solution**:
```bash
# Check if server is running
ps aux | grep swiftlink-server

# Test connectivity
curl http://localhost:8080/api/info/test

# Check if port is correct
netstat -tulpn | grep :8080
```

#### SSL/TLS Errors

**Cause**: HTTPS vs HTTP mismatch

**Solutions**:
```bash
# Use HTTP for local development
swiftclient --base-url http://localhost:8080

# For HTTPS, ensure valid certificate
swiftclient --base-url https://your-domain.com
```

### Database Issues

#### PostgreSQL Connection Failed

**Common Errors**:
- `FATAL: database "swiftlink_db" does not exist`
- `FATAL: password authentication failed for user`
- `connection refused`

**Solutions**:

1. **Create Database**:
   ```sql
   CREATE DATABASE swiftlink_db;
   CREATE USER swiftlink_user WITH PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE swiftlink_db TO swiftlink_user;
   ```

2. **Check Configuration**:
   ```toml
   [database]
   database_type = "postgres"
   username = "swiftlink_user"
   password = "your_password"
   host = "localhost"
   port = 5432
   database = "swiftlink_db"
   ```

3. **Test Connection**:
   ```bash
   psql -h localhost -U swiftlink_user -d swiftlink_db
   ```

#### SQLite Database Locked

**Cause**: Multiple processes trying to write simultaneously

**Solutions**:
- Wait for other processes to finish
- Use `journal_mode=WAL` in SQLite
- Consider PostgreSQL for concurrent access

### Performance Issues

#### Slow Response Times

**Diagnosis**:
```bash
# Check response time
time curl http://localhost:8080/api/info/testcode

# Check database performance
# PostgreSQL:
EXPLAIN ANALYZE SELECT * FROM links WHERE code = 'testcode';
```

**Optimizations**:

1. **Database Indexes**:
   ```sql
   CREATE INDEX IF NOT EXISTS idx_links_code ON links(code);
   ```

2. **Connection Pooling**:
   ```toml
   [database]
   max_connections = 10
   ```

3. **Code Generation**:
   ```toml
   [base]
   code_size = 4  # Shorter codes = faster lookups
   ```

## Debugging Tools

### Server Logs

Enable debug logging:
```bash
./swiftlink-server --config config.toml --log-level debug
```

### Database Queries

**PostgreSQL**:
```sql
-- Enable query logging
ALTER SYSTEM SET log_statement = 'all';
SELECT pg_reload_conf();

-- Monitor activity
SELECT * FROM pg_stat_activity WHERE datname = 'swiftlink_db';
```

### Network Debugging

```bash
# Test connectivity
telnet localhost 8080

# Check DNS
nslookup your-domain.com

# Trace route
traceroute your-domain.com
```

## Getting Help

### Collect Debug Information

When reporting issues, include:

1. **Swiftlink Version**:
   ```bash
   ./swiftlink-server --version
   ```

2. **Rust Version**:
   ```bash
   rustc --version
   cargo --version
   ```

3. **System Information**:
   ```bash
   uname -a
   ```

4. **Configuration**:
   ```bash
   cat config.toml
   # Remove sensitive information before sharing
   ```

5. **Logs**:
   ```bash
   # Collect recent logs
   journalctl -u swiftlink --since "1 hour ago"
   ```

### Common Questions

**Q: How do I reset the database?**
```bash
# PostgreSQL
psql -d swiftlink_db -c "DROP TABLE IF EXISTS links;"

# SQLite
rm swiftlink.db
```

**Q: How do I migrate data?**
```bash
# Export from SQLite
sqlite3 old.db .dump > dump.sql

# Import to PostgreSQL
psql -d swiftlink_db < dump.sql
```

**Q: How do I backup?**
```bash
# PostgreSQL
pg_dump swiftlink_db > backup.sql

# SQLite
cp swiftlink.db swiftlink.db.backup
```

## Report Issues

If you encounter an issue not covered here:

1. Check the [GitHub Issues](https://github.com/walker84837/swiftlink/issues)
2. Create a new issue with:
   - Description of the problem
   - Steps to reproduce
   - Expected vs actual behavior
   - System information
   - Error messages and logs

For community support, see the [Contributing Guide](contributing.md).