//! Postgres module

use clap::Parser;
use sqlx::PgPool;

/// Database connection
#[derive(Debug, Clone)]
pub struct PostgresDatabase {
    /// The database connection pool
    pub pool: PgPool,
}

impl PostgresDatabase {
    /// Create a new database connection
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new database connection with a URL
    pub async fn new_with_url(url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: PgPool::connect(url).await?,
        })
    }

    /// Returns the underlying database connection
    pub fn connection(&self) -> &PgPool {
        &self.pool
    }
}

/// Database connection details
#[derive(Debug, Parser)]
pub struct DatabaseConnectionDetails {
    /// The database connection string
    #[arg(long, env = "DATABASE_URL")]
    pub connection_string: String,
}
