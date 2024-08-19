//! Postgres implementation of the UserRepository trait

use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    error::ErrorKind::UniqueViolation,
    query, query_as,
    Error::{Database, RowNotFound},
};
use uuid::Uuid;

use crate::{
    domain::auth::{
        errors::{CreateUserError, GetUserByIdError},
        models::user::{NewUser, User},
        repositories::user::UserRepository,
        value_objects::email_address::EmailAddress,
    },
    infrastructure::db::postgres::PostgresDatabase,
};

struct UserRecord {
    id: Uuid,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRecord> for User {
    type Error = Error;

    fn try_from(record: UserRecord) -> Result<Self, Self::Error> {
        Ok(User {
            id: record.id,
            email: EmailAddress::new_unchecked(record.email.as_ref()),
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

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
        .map_err(|err| match err {
            Database(db_err) => match db_err.kind() {
                UniqueViolation => CreateUserError::DuplicateUser {
                    email: req.email().clone(),
                },
                _ => CreateUserError::UnknownError(anyhow!("Unknown database error: {:?}", db_err)),
            },
            _ => CreateUserError::UnknownError(anyhow!("Unknown database error: {:?}", err)),
        })?;

        Ok(result.id)
    }

    #[mutants::skip]
    async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError> {
        Ok(query_as!(
            UserRecord,
            r#"
            SELECT id, email, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            RowNotFound => GetUserByIdError::UserNotFound(*id),
            _ => GetUserByIdError::UnknownError(anyhow!("Unknown database error: {:?}", err)),
        })?
        .try_into()?)
    }
}
