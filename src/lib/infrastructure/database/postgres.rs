//! Postgres module

use clap::Parser;
use sqlx::PgPool;
use thiserror::Error;

use PostgresDatabaseError::*;

/// Postgres database error
#[derive(Debug, Error)]
pub enum PostgresDatabaseError {
    /// Empty connection string
    #[error("Empty connection string")]
    EmptyConnectionString,

    /// Invalid connection string
    #[error("Invalid connection string")]
    InvalidConnectionString,

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(sqlx::Error),
}

/// Database connection
#[derive(Debug, Clone)]
pub struct PostgresDatabase {
    /// The database connection pool
    pub pool: PgPool,
}

impl PostgresDatabase {
    /// Create a new database connection
    pub async fn new(connection_string: &str) -> Result<Self, PostgresDatabaseError> {
        if connection_string.is_empty() {
            return Err(EmptyConnectionString);
        }

        if !connection_string.starts_with("postgres://") {
            return Err(InvalidConnectionString);
        }

        Ok(Self {
            pool: PgPool::connect(connection_string)
                .await
                .map_err(|err| ConnectionError(err))?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blank_connection_string_returns_error() {
        let result = PostgresDatabase::new("").await;
        assert!(matches!(
            result,
            Err(PostgresDatabaseError::EmptyConnectionString)
        ));
    }

    #[tokio::test]
    async fn test_invalid_connection_string_returns_error() {
        let result = PostgresDatabase::new("invalid").await;
        assert!(matches!(
            result,
            Err(PostgresDatabaseError::InvalidConnectionString)
        ));
    }
}
