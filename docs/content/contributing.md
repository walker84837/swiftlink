# Contributing to Swiftlink

Swiftlink is free and open-source software, and contributions are welcome! This guide will help you get started with contributing to the project.

## Ways to Contribute

There's mulitple ways you can contribute to Swiftlink. This section will list a few ways to do so, along with some examples of contributions for every category.

## Non-code changes

If you're not a developer or know Rust, contributions to Swiftlink don't need to be code-related.

### Improving Documentation

- Fix typos and grammatical errors
- Add missing examples
- Improve explanation of concepts
- Update outdated information
- Styling and readability improvements

## Code-related changes

### Implementing New Features

1. Open an issue to discuss the proposed feature
2. Get feedback from maintainers
3. Follow the implementation guidelines below
4. Submit a pull request

### Refactoring and Cleanup

- Improve code organization
- Optimize performance
- Remove deprecated code
- Enhance error handling

### Development Setup

#### Prerequisites

- **Rust** 1.85.0 or later
- **Database**: PostgreSQL or SQLite
- **Git**

#### Getting Started

1. **Clone the repository**:
   ```bash
   git clone https://github.com/walker84837/swiftlink.git
   cd swiftlink
   ```

2. **Set up the development environment**:
   ```bash
   # Install Rust toolchain
   rustup update stable
   rustup component add rustfmt clippy
   
   # Build the workspace
   cargo build --workspace
   ```

3. **Run tests**:
   ```bash
   cargo test --workspace
   ```

4. **Check formatting**:
   ```bash
   cargo fmt --all --check
   ```

5. **Run clippy**:
   ```bash
   cargo clippy --workspace
   ```

#### Database Setup for Development

For testing with a local database:

**PostgreSQL**:

1. Install PostgreSQL:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install postgresql postgresql-contrib
   
   # macOS (with Homebrew)
   brew install postgresql
   brew services start postgresql
   ```

2. Create a database and user:
   ```sql
   psql -U postgres
   
   CREATE DATABASE swiftlink_db;
   CREATE USER swiftlink_user WITH PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE swiftlink_db TO swiftlink_user;
   ```

3. Configure the server to use PostgreSQL (see `example/config.toml`)

**SQLite**:

SQLite requires no setup - just specify a file path in the configuration. The default `example/config.toml` uses SQLite.

#### Reporting Bugs

- Use the [GitHub issue tracker](https://github.com/walker84837/swiftlink/issues)
- Include:
  - Swiftlink version
  - Operating system and Rust version
  - Steps to reproduce
  - Expected vs actual behavior
  - Any relevant logs or configuration

## Code Structure

Swiftlink is organized as a Cargo workspace with three main crates:

### `swiftlink-server` - HTTP Server

The core web server that handles URL shortening, redirects, and the REST API. Built with Actix-web, it handles HTTP requests, manages database connections, and serves the short link functionality.

### `swiftlink-api` - Shared Library

A Rust library containing request/response types and client implementations (both async and blocking). Used by `swiftclient` and can be used by other Rust applications to interact with a Swiftlink server.

### `swiftclient` - CLI Tool

A command-line interface for interacting with a Swiftlink server. Allows creating, retrieving info about, and deleting short links without direct database access.

## Development Guidelines

### Code Style

- Follow Rust's standard formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Write clear, descriptive variable and function names
- Add comments for complex logic
- Document public APIs with `///` doc comments
- Avoid `.unwrap()` and `.expect()` and actaully handle errors.

### Testing

- Write unit tests for new functionality where applicable
- Integration tests should cover API endpoints
- Ensure all tests pass before submitting PR
- Use `cargo test --workspace` to run all tests

### Commit Messages

Follow the [Conventional Commits](https://conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Examples**:

- `feat(api): add bulk endpoint for creating multiple links`
- `fix: handle database connection errors gracefully`
- `docs(cli): update installation instructions`
- `refactor(client): remove duplicate code in error handling`

## Pull Request Process

### Before Submitting

1. **Create a new branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and ensure they follow the guidelines

3. **Run all checks**:
   ```bash
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   ```

4. **Update documentation** if needed

### Submitting the PR

1. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a pull request** with:
   - Clear title describing the change
   - Detailed description of *why* the change is needed
   - Steps to test (if applicable)
   - Any breaking changes or migration notes

3. **Link any related issues** in the PR description

## Proposing Bug Fixes

1. **Reproduce the bug** with minimal steps
2. **Add a test** that fails before the fix
3. **Implement the fix**
4. **Ensure all tests pass**
5. **Update documentation** if behavior changed
6. **Submit PR** with `fix:` prefix

## Feature Development

1. **Open an issue** to discuss the feature idea
2. **Get consensus** on the approach
3. **Break down** into smaller tasks if needed
4. **Implement incrementally**
5. **Test thoroughly**
6. **Update documentation**
7. **Submit PR** with `feat:` prefix

## Considerations When Adding Features

### Performance

- Optimize for common use cases
- Be mindful of large URL lists
- Consider thread safety in shared code
- Don't let performance degrade on errors

### Security

- Always validate external input
- Never expose tokens or secrets
- Use parameterized queries (SQLx handles this)
- Consider for public deployments

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For general questions
- **Documentation**: Check existing docs first
- **Code Comments**: Read inline documentation

## License

By contributing to Swiftlink, you agree that your contributions will be licensed under the same license as the project (Apache-2.0 OR MIT).
