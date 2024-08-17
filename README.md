# Rust SaaS Starter Kit

This is a starter kit for building a SaaS application with Rust and Postgres.

It is a work in progress and is **not yet ready for production use**.

Based partly on https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust

## Requirements

- Rust
- Docker
- Docker Compose
- SQLx CLI

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
sqlx migrate run
```

The database can be viewed at http://localhost:8888.
