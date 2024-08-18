# Rust SaaS Starter Kit

This is a starter kit for building a SaaS application with Rust and Postgres.

It is a work in progress and is **not yet ready for production use**.

Based partly on [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust).

## Requirements

- Rust
- Docker
- Docker Compose

## Generating Self-Signed Certificates

To generate self-signed certificates for local development, you can use the OpenSSL toolkit. Run the following commands:

NOTE: Self-signed certificates should only be used for development purposes. For production environments, obtain certificates from a trusted Certificate Authority (CA).

```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout certs/key.pem -out certs/cert.pem -days 365 -subj "/CN=localhost"
```

## Application Setup

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
sqlx migrate run
```

4. Start the application:

```bash
cargo run --bin server
```

The database can be viewed using Adminer at http://localhost:8888. Adminer is a lightweight database management tool that provides a web interface for interacting with the database.
