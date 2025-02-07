//! Postgres implementation of the UserRepository trait

use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, Error::RowNotFound};
use uuid::Uuid;

use crate::{
    domain::{
        auth::users::{
            errors::{CreateUserError, GetUserByIdError, UpdateUserError},
            NewUser, User, UserRepository,
        },
        communication::email_addresses::EmailAddress,
    },
    infrastructure::db::postgres::PostgresDatabase,
};

struct UserRecord {
    id: Uuid,
    email: String,
    new_email: Option<String>,
    email_confirmed_at: Option<DateTime<Utc>>,
    email_confirmation_token: Option<String>,
    email_confirmation_sent_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRecord> for User {
    type Error = Error;

    fn try_from(record: UserRecord) -> Result<Self, Self::Error> {
        Ok(User {
            id: record.id,
            email: EmailAddress::new_unchecked(record.email.as_ref()),
            new_email: record
                .new_email
                .map(|email| EmailAddress::new_unchecked(email.as_ref())),
            email_confirmed_at: record.email_confirmed_at,
            email_confirmation_token: record.email_confirmation_token,
            email_confirmation_sent_at: record.email_confirmation_sent_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

#[async_trait]
impl UserRepository for PostgresDatabase {
    #[mutants::skip]
    async fn create_user(&self, user: &NewUser) -> Result<Uuid, CreateUserError> {
        let result = query!(
            r#"
            INSERT INTO users (id, email, password)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            user.id(),
            user.email().to_string(),
            user.password_hash().to_string()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    #[mutants::skip]
    async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError> {
        Ok(query_as!(
            UserRecord,
            r#"
            SELECT
                id,
                email,
                new_email,
                email_confirmed_at,
                email_confirmation_token,
                email_confirmation_sent_at,
                created_at,
                updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            RowNotFound => GetUserByIdError::UserNotFound,
            _ => GetUserByIdError::UnknownError(anyhow!("Unknown database error: {:?}", err)),
        })?
        .try_into()?)
    }

    #[mutants::skip]
    async fn initialize_email_confirmation<'a>(
        &self,
        user_id: &Uuid,
        token: &str,
        new_email: Option<&'a EmailAddress>,
    ) -> Result<(), UpdateUserError> {
        let new_email: Option<String> = new_email.as_ref().map(|email| email.to_string());

        let mut tx = self.pool.begin().await?;

        if new_email.is_some() {
            let existing_email = query!(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM users
                    WHERE email = $1
                )
                "#,
                new_email,
            )
            .fetch_optional(&mut *tx)
            .await?;

            if existing_email
                .map(|row| row.exists.unwrap_or(false))
                .unwrap_or(false)
            {
                return Err(UpdateUserError::EmailAddressInUse);
            }
        }

        query!(
            r#"
            UPDATE users
            SET email_confirmation_token = $1,
            email_confirmation_sent_at = NOW(),
            new_email = COALESCE($3, new_email)
            WHERE id = $2
            "#,
            token.to_string(),
            user_id,
            new_email,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    #[mutants::skip]
    async fn complete_email_confirmation<'a>(
        &self,
        user_id: &Uuid,
        new_email: Option<&'a EmailAddress>,
    ) -> Result<(), UpdateUserError> {
        query!(
            r#"
            UPDATE users
            SET email_confirmed_at = NOW(),
                email_confirmation_token = NULL,
                email = COALESCE($2, email),
                new_email = NULL
            WHERE id = $1
            "#,
            user_id,
            new_email.map(|email| email.to_string()),
        )
        .execute(&self.pool)
        .await
        .map_err(|err| match err {
            RowNotFound => UpdateUserError::UserNotFound,
            _ => UpdateUserError::UnknownError(anyhow!("Unknown database error: {:?}", err)),
        })?;

        Ok(())
    }
}
