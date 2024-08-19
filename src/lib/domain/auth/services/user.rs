//! User service module

use std::sync::Arc;

use anyhow::Result;
use askama::Template;
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[cfg(test)]
use mockall::mock;

use crate::domain::{
    auth::{
        emails::confirm_email_address::ConfirmEmailAddressTemplate,
        errors::{CreateUserError, EmailConfirmationError, GetUserByIdError},
        models::user::{NewUser, User},
        repositories::user::UserRepository,
    },
    comms::mailer::Mailer,
};

/// User service
#[async_trait]
pub trait UserService: Clone + Send + Sync + 'static {
    /// Creates a new user based on the provided request details.
    ///
    /// # Arguments
    /// * `req` - A reference to a [`CreateUserRequest`] containing the user details.
    ///
    /// # Returns
    /// A [`Result`] which is [`Ok`] containing the user's UUID if the user is successfully created,
    /// or an [`Err`] containing a [`CreateUserError`] if the user cannot be created.
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;

    /// Retrieves a user by their ID.
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to retrieve.
    ///
    /// # Returns
    /// A [`Result`] which is [`Ok`] containing the [`User`] if found,
    /// or an [`Err`] containing a [`GetUserError`] if the user cannot be found.
    async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError>;

    /// Sends an email confirmation to the user.
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to send the email confirmation to.
    ///
    /// # Returns
    /// A [`Result`] which is [`Ok`] if the email confirmation was sent successfully,
    /// or an [`Err`] containing an [`EmailConfirmationError`] if the email confirmation could not be sent.
    async fn send_email_confirmation(
        &self,
        user_id: &Uuid,
        base_url: &str,
    ) -> Result<DateTime<Utc>, EmailConfirmationError>;
}

#[cfg(test)]
mock! {
    pub UserService {}

    impl Clone for UserService {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl UserService for UserService {
        async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;
        async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError>;
        async fn send_email_confirmation(&self, user_id: &Uuid, base_url: &str) -> Result<DateTime<Utc>, EmailConfirmationError>;
    }
}

/// User service implementation
#[derive(Debug, Clone)]
pub struct UserServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    repo: Arc<R>,
    mailer: Arc<M>,
}

impl<R, M> UserServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    /// Create a new user service
    pub fn new(repo: Arc<R>, mailer: Arc<M>) -> Self {
        Self { repo, mailer }
    }
}

impl<R, M> UserServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    async fn generate_email_confirmation_token(
        &self,
        user_id: &Uuid,
    ) -> Result<(String, DateTime<Utc>)> {
        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let data = format!("{}{}{}", user_id, salt, Utc::now().timestamp());
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hash_result = hasher.finalize();
        let token = URL_SAFE.encode(hash_result);

        self.repo
            .update_email_confirmation_token(user_id, &token)
            .await?;

        Ok((token, Utc::now() + Duration::hours(24)))
    }
}

#[async_trait]
impl<R, M> UserService for UserServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError> {
        self.repo.create_user(req).await
    }

    async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError> {
        self.repo.get_user_by_id(id).await
    }

    async fn send_email_confirmation(
        &self,
        user_id: &Uuid,
        base_url: &str,
    ) -> Result<DateTime<Utc>, EmailConfirmationError> {
        let user = self.get_user_by_id(user_id).await?;

        if user.email_confirmed_at.is_some() {
            return Err(EmailConfirmationError::EmailAlreadyConfirmed);
        }

        let (token, expires_at) = self.generate_email_confirmation_token(user_id).await?;

        let template = ConfirmEmailAddressTemplate::new(base_url, user_id, &token).render()?;

        self.mailer
            .send_email(&user.email, "Please confirm your email address", &template)
            .await?;

        Ok(expires_at)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::anyhow;
    use chrono::Utc;
    use mockall::predicate::eq;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::domain::{
        auth::{
            models::user::NewUser, repositories::user::MockUserRepository,
            value_objects::password::Password,
        },
        comms::{mailer::MockMailer, value_objects::email_address::EmailAddress},
    };

    use super::*;

    #[tokio::test]
    async fn test_create_user_success() -> TestResult {
        let user_id = Uuid::now_v7();
        let request = NewUser::new(
            user_id,
            EmailAddress::new_unchecked("email@example.com"),
            Password::new("correcthorsebatterystaple")?,
        );
        let expected_id = request.id().clone();

        let mut mock = MockUserRepository::new();

        mock.expect_create_user()
            .times(1)
            .with(eq(request.clone()))
            .returning(move |_| Ok(expected_id));

        let service = UserServiceImpl::new(Arc::new(mock), Arc::new(MockMailer::new()));

        let user_id = service.create_user(&request).await?;

        assert_eq!(&user_id, request.id());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_user_already_exists() -> TestResult {
        let user_id = Uuid::now_v7();
        let request = NewUser::new(
            user_id,
            EmailAddress::new_unchecked("email@example.com"),
            Password::new("correcthorsebatterystaple")?,
        );
        let email = request.email().clone();

        let mut mock = MockUserRepository::new();

        mock.expect_create_user()
            .times(1)
            .with(eq(request.clone()))
            .returning(move |_req| {
                Err(CreateUserError::DuplicateUser {
                    email: email.clone(),
                })
            });

        let service = UserServiceImpl::new(Arc::new(mock), Arc::new(MockMailer::new()));

        let result = service.create_user(&request).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CreateUserError::DuplicateUser { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_user_unknown_error() -> TestResult {
        let user_id = Uuid::now_v7();
        let request = NewUser::new(
            user_id,
            EmailAddress::new_unchecked("email@example.com"),
            Password::new("correcthorsebatterystaple")?,
        );

        let mut mock = MockUserRepository::new();

        mock.expect_create_user()
            .times(1)
            .with(eq(request.clone()))
            .returning(move |_req| Err(CreateUserError::UnknownError(anyhow!("Unknown error"))));

        let service = UserServiceImpl::new(Arc::new(mock), Arc::new(MockMailer::new()));

        let result = service.create_user(&request).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CreateUserError::UnknownError { .. })));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_by_id_success() -> TestResult {
        let user_id = Uuid::now_v7();

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("mdcpepper@gmail.com"),
            email_confirmed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let expected_user = user.clone();

        let mut repo = MockUserRepository::new();

        repo.expect_get_user_by_id()
            .times(1)
            .with(eq(user_id.clone()))
            .returning(move |_| Ok(user.clone()));

        let service = UserServiceImpl::new(Arc::new(repo), Arc::new(MockMailer::new()));

        let found_user = service.get_user_by_id(&user_id).await?;

        assert_eq!(found_user, expected_user);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut mock = MockUserRepository::new();

        mock.expect_get_user_by_id()
            .times(1)
            .with(eq(user_id.clone()))
            .returning(move |_| Err(GetUserByIdError::UserNotFound(user_id.clone())));

        let service = UserServiceImpl::new(Arc::new(mock), Arc::new(MockMailer::new()));

        let result = service.get_user_by_id(&user_id).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(GetUserByIdError::UserNotFound(id)) if id == user_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_email_confirmation_token_and_update_user() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut repo = MockUserRepository::new();

        repo.expect_update_email_confirmation_token()
            .times(1)
            .returning(move |_, _| Ok(()));

        let service = UserServiceImpl::new(Arc::new(repo), Arc::new(MockMailer::new()));

        let (token, expires_at) = service.generate_email_confirmation_token(&user_id).await?;

        assert_eq!(44, token.len());
        assert!(expires_at > Utc::now());

        Ok(())
    }

    #[tokio::test]
    async fn test_send_email_confirmation_success() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserRepository::new();

        users
            .expect_get_user_by_id()
            .times(1)
            .with(eq(user_id.clone()))
            .returning(move |_| {
                Ok(User {
                    id: user_id.clone(),
                    email: EmailAddress::new_unchecked("email@example.com"),
                    email_confirmed_at: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            });

        users
            .expect_update_email_confirmation_token()
            .times(1)
            .returning(move |_, _| Ok(()));

        let mut mailer = MockMailer::new();

        mailer
            .expect_send_email()
            .times(1)
            .returning(move |_, _, _| Ok(()));

        let service = UserServiceImpl::new(Arc::new(users), Arc::new(mailer));

        let expires_at = service
            .send_email_confirmation(&user_id, "https://localhost:3443")
            .await?;

        assert!(expires_at > Utc::now());

        Ok(())
    }
}
