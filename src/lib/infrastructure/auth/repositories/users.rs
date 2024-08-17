//! Postgres implementation of the UserRepository trait

use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{error::ErrorKind::UniqueViolation, query, Error::Database};
use uuid::Uuid;

use crate::{
    domain::auth::{
        models::user::{CreateUserError, CreateUserRequest},
        repositories::user::UserRepository,
    },
    infrastructure::database::postgres::PostgresDatabase,
};

#[async_trait]
impl UserRepository for PostgresDatabase {
    #[mutants::skip]
    async fn create_user(&self, req: &CreateUserRequest) -> Result<Uuid, CreateUserError> {
        let result = query!(
            r#"
            INSERT INTO users (id, email)
            VALUES ($1, $2)
            RETURNING id
            "#,
            req.id(),
            req.email().to_string(),
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
