# Getting Started

This guide will show you how to set up and run Swiftlink locally for the first time. Swiftlink is easy for both new users and system administrators.

## Requirements

Before you start, make sure you have the following installed on your system:

- Rust (stable channel): a fast and safe programming language, and Swiftlink is built with it;
- PostgreSQL or SQLite: used to store the short link information.

### Rust

You'll need the Rust compiler and Cargo, which is Rust's package manager and build tool.

If you don't have Rust installed, using rustup--the official Rust toolchain installer--is the recommended way.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, verify with:

```sh
rustc --version
```

and

```sh
cargo --version
```

### Databases

If you don't know what SQLite or PostgreSQL are:

  * SQLite is a simple file-based, self-contained database, and requires no configuration.
  * PostgreSQL is a powerful relational database, better-suited for production environments. Going this route requires more configuration.

### Installation

#### From Source: Cloning the Repository

First, you need to obtain the Swiftlink source code. Open your terminal or command prompt and execute the following commands:

```sh
git clone https://github.com/walker84837/swiftlink.git
cd swiftlink
```

This will download the entire Swiftlink project to your current directory and then navigate you into the newly created swiftlink directory.

**Compiling**:

Swiftlink's codebase is a Rust workspace, which means it's divided into modules. If you use the cargo build command with the --workspace option, it will compile all of them.

```sh
cargo build --workspace --release
```

Here's a breakdown of what this command does:

- `cargo build`: This is the standard command to compile a Rust project.
- `--workspace`: This flag tells Cargo to build all the crates defined in the workspace section of Cargo.toml. In Swiftlink, this includes:
  * `swiftlink-server`: The core web server responsible for link shortening.
  * `swiftclient`: A CLI tool for interacting with the Swiftlink server to create, retrieve information about, or delete short links, without accessing the database.
  * `swiftlink-api`: A Rust library that defines data structures and API contracts used by swiftclient. It acts as an SDK for Rust applications.
- `--release`: It optimizes the build for performance and enables optimization. These optimizations make it extremely fast, making it perfect for deployment or benchmarking.

When all three finish compiling, they'll be placed in `./target/release`, or `./target/debug` if you didn't add `--release`. If you downloaded a pre-built version, you should remove `./target/.../` and leave just `./swiftlink-server`, for example.

#### Run the Server

Once the code compiles, you can start the Swiftlink server.

The server requires a configuration file to specify its behavior, such as:
- the database to use;
- the port to listen on;
- any authentication tokens.

For the sake of running the server the first time, we'll use the default one:

```sh
./target/release/swiftlink-server --config example/config.toml
```

The config.toml file provides a very simple configuration that's perfect for getting started quickly. It contains fundamental server settings like:
  * The number of characters for generated short codes;
  * The network port the server will listen on (default is 8080); If this port is already in use, you can change it in the config.toml file.
  * A simple token for authenticating API requests, primarily for actions like deleting links.

It also contains configuration for the database the defaults are:
   * SQLite database in "swiftlink.db" in the server's working directory.

**Try it out with the CLI**:

With the server running, you can now use the `swiftclient` utility to interact with it and create your first short link.

Open another terminal window, while keeping the server running.

```sh
./target/release/swiftclient create --url https://example.com --base-url http://localhost:8080
```

This creates a new shortened URL which redirects to <https://example.com>, sending a your local "demo" server (`http://localhost:8080`) to create a link.

The server will generate a unique short code for your URL, your client will receive it and print it to the console.

**Visit your Short Link**:

Finally, you can test your newly created short link by visiting it in your web browser.

Take the generated short code from the previous step and append it to your server's base URL:

`http://localhost:8080/<YOUR_GENERATED_CODE>`

!!! note
    Replace `<YOUR_GENERATED_CODE>` with the actual code displayed by swiftclient.

When you navigate to this URL in your browser, the server will redirect you to the original long URL (<https://example.com> in this case).

Congratulations! You have successfully set up, run, and used Swiftlink locally.