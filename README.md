# Rust SaaS Starter Kit

This is a starter kit for building a SaaS application with Rust and Postgres.

It is a work in progress and is **not yet ready for production use**.

Based partly on [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust).

## Requirements

- Rust
- Docker
- Docker Compose

## Setup

1. Create a `.env` file:

```bash
cp .env.example .env
```

2. Start the database:

```bash
docker-compose up -d
```

3. Run the migrations:

```bash
cargo install sqlx-cli
cargo build
sqlx migrate run
```

The database can be viewed using Adminer at http://localhost:8888. Adminer is a lightweight database management tool that provides a web interface for interacting with the database.
