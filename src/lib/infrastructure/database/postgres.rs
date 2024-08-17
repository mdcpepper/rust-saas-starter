//! Postgres module

use anyhow::Result;
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
    pub async fn new(connection_string: &str) -> Result<Self> {
        Ok(Self {
            pool: PgPool::connect(connection_string).await?,
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
