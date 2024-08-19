//! Postgres implementation of the UserRepository trait

use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{error::ErrorKind::UniqueViolation, query, Error::Database};
use uuid::Uuid;

use crate::{
    domain::auth::{
        models::user::{CreateUserError, NewUser},
        repositories::user::UserRepository,
    },
    infrastructure::db::postgres::PostgresDatabase,
};

#[async_trait]
impl UserRepository for PostgresDatabase {
    #[mutants::skip]
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError> {
        let result = query!(
            r#"
            INSERT INTO users (id, email, password)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            req.id(),
            req.email().to_string(),
            req.password_hash().to_string()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Database(db_error) => match db_error.kind() {
                UniqueViolation => CreateUserError::DuplicateUser {
                    email: req.email().clone(),
                },
                _ => {
                    CreateUserError::UnknownError(anyhow!("Unknown database error: {:?}", db_error))
                }
            },
            _ => CreateUserError::UnknownError(anyhow!("Unknown database error: {:?}", e)),
        })?;

        Ok(result.id)
    }
}
