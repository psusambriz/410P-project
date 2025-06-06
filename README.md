### Features

* **RESTful API:** The primary programming language
* **Database Integration:** Stores quote using SQLite via 'sqlx'
* **Server-Side Rendered UI (Askama):** Basic HTML interface to display quotes and interact with the API
* **Automatic Database Initialization:** Populates database from a 'quotes.json' file 

### Backend

* **Rust:** The primary programming language
* **Axum:** A web application framework for Rust
* **SQLx:** Asynchronous SQL database
* **Tokio:** Asynchronous runtime for Rust
* **Askama:** Server-side rendering
* **Clap:** Command-line argument parser
* **Utoipa:** OpenAPI (Swagger) documentation generation
* **Tower-HTTP:** HTTP middleware for Axum

## Prerequisites

Ensure you have the following installed:
* **Rust & Cargo:**
```bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
```
(Follow the on-screen instructions)
* **SQLx CLI:**
```bash
cargo install sqlx-cli --no-default-features --features rustls,sqlite
```

## Getting Started

