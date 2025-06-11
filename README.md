
# Kanye Quotes Web App

A lightweight web application built with Rust that fetches and displays iconic quotes from Kanye West. It features a RESTful API, server-side rendering with Askama, and SQLite database integration.

## Features

- Built with **Rust** and **Axum**
- Uses **SQLite** and **SQLx** for database interactions
- Renders UI using **Askama** templating engine
- Swagger documentation powered by **utoipa**
- Command-line configuration with **Clap**
- Asynchronous runtime via **Tokio**
- Automatically seeds the database from a `quotes.json` file

## Tech Stack

- **Rust** - Systems programming language
- **Axum** - Web framework for building async APIs
- **SQLx** - Async, pure Rust SQL crate
- **Tokio** - Asynchronous runtime
- **Askama** - Type-safe template engine
- **Clap** - Command-line argument parsing
- **Utoipa** - OpenAPI documentation generator
- **Tower-HTTP** - HTTP middleware for Axum

## Prerequisites

Make sure you have the following installed:

- [Rust & Cargo](https://rustup.rs/)
- `sqlx-cli` with SQLite support:
  ```bash
  cargo install sqlx-cli --no-default-features --features rustls,sqlite
  ```

## Getting Started

### Clone the Repo

```bash
git clone https://github.com/psusambriz/410P-project.git
cd 410P-project
```

### Setup and Run

```bash
# Remove old artifacts
rm -rf db .sqlx

# Create DB directory
mkdir db

# Set the environment variable
export DATABASE_URL="sqlite:db/quotes.db"

# Create the database
sqlx database create

# Run migrations
sqlx migrate run

# Prepare SQLx offline cache
cargo sqlx prepare

# Build and run the app
SQLX_OFFLINE=true cargo run -- --init-from quotes.json
```
## License

This project is licensed under the MIT License.
