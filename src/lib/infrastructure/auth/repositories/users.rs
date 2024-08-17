//! Postgres implementation of the UserRepository trait

use anyhow::anyhow;
use sqlx::{error::ErrorKind::UniqueViolation, query};
use uuid::Uuid;

use crate::{
    domain::auth::{
        models::user::{CreateUserError, CreateUserRequest},
        repositories::user::UserRepository,
    },
    infrastructure::database::postgres::PostgresDatabase,
};

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
        .map_err(|e| {
            println!("{:#?}", e);

            match e {
                sqlx::Error::Database(db_error) => match db_error.kind() {
                    UniqueViolation => CreateUserError::Duplicate {
                        email: req.email().clone(),
                    },
                    _ => CreateUserError::UnknownError(anyhow!("Unknown something error")),
                },
                _ => CreateUserError::UnknownError(anyhow!("Unknown something else error")),
            }
        })?;

        Ok(result.id)
    }
}
