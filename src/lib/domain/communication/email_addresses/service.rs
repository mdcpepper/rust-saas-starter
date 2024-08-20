//! Email address service

use std::sync::Arc;

use anyhow::Result;
use askama::Template;
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use chrono::{DateTime, Duration, Utc};
use constant_time_eq::constant_time_eq;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[cfg(test)]
use mockall::mock;

use crate::domain::{
    auth::{
        emails::confirm_email_address::ConfirmEmailAddressTemplate,
        users::{errors::EmailConfirmationError, User, UserRepository},
    },
    communication::mailer::Mailer,
};

/// Email address service
#[async_trait]
pub trait EmailAddressService: Clone + Send + Sync + 'static {
    /// Sends an email confirmation to the user.
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to send the email confirmation to.
    ///
    /// # Returns
    /// - [`Ok`] with a [`DateTime<Utc>`] representing the token's expiration time if successful.
    /// - [`Err`] containing an [`EmailConfirmationError`] if the email confirmation could not be sent.
    async fn send_email_confirmation(
        &self,
        user: &User,
        base_url: &str,
    ) -> Result<DateTime<Utc>, EmailConfirmationError>;

    /// Confirms the user's email address.
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user to confirm the email address for.
    /// * `token` - The email confirmation token.
    ///
    /// # Returns
    /// A [`Result`] which is [`Ok`] if the email address was confirmed successfully,
    async fn confirm_email(&self, user: &User, token: &str) -> Result<(), EmailConfirmationError>;
}

#[cfg(test)]
mock! {
    pub EmailAddressService {}

    impl Clone for EmailAddressService {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl EmailAddressService for EmailAddressService {
        async fn send_email_confirmation(&self, user: &User, base_url: &str) -> Result<DateTime<Utc>, EmailConfirmationError>;
        async fn confirm_email(&self, user: &User, token: &str) -> Result<(), EmailConfirmationError>;
    }
}

/// Email address service implementation
#[derive(Debug, Clone)]
pub struct EmailAddressServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    user_repo: Arc<R>,
    mailer: Arc<M>,
}

impl<R, M> EmailAddressServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    /// Creates a new email address service.
    pub fn new(user_repo: Arc<R>, mailer: Arc<M>) -> Self {
        Self { user_repo, mailer }
    }

    async fn generate_email_confirmation_token(
        &self,
        user_id: &Uuid,
    ) -> Result<(String, DateTime<Utc>)> {
        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let data = format!("{}{}{}", user_id, salt, Utc::now().timestamp());
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hash_result = hasher.finalize();
        let token = URL_SAFE.encode(hash_result);

        self.user_repo
            .update_email_confirmation_token(user_id, &token)
            .await?;

        Ok((token, Utc::now() + Duration::hours(24)))
    }
}

#[async_trait]
impl<R, M> EmailAddressService for EmailAddressServiceImpl<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    async fn send_email_confirmation(
        &self,
        user: &User,
        base_url: &str,
    ) -> Result<DateTime<Utc>, EmailConfirmationError> {
        if user.email_confirmed_at.is_some() {
            return Err(EmailConfirmationError::EmailAlreadyConfirmed);
        }

        let (token, expires_at) = self.generate_email_confirmation_token(&user.id).await?;

        let template = ConfirmEmailAddressTemplate::new(base_url, &user.id, &token);
        let html = css_inline::inline(&template.render()?)?;
        let plain = template.render_plain()?;

        self.mailer
            .send_email(
                &user.email,
                "Please confirm your email address",
                &html,
                &plain,
            )
            .await?;

        Ok(expires_at)
    }

    async fn confirm_email(&self, user: &User, token: &str) -> Result<(), EmailConfirmationError> {
        if user.email_confirmed_at.is_some() {
            return Err(EmailConfirmationError::EmailAlreadyConfirmed);
        }

        let expected_token = user
            .email_confirmation_token
            .as_ref()
            .ok_or_else(|| EmailConfirmationError::ConfirmationTokenMismatch)?;

        let confirmation_sent_at = user
            .email_confirmation_sent_at
            .ok_or_else(|| EmailConfirmationError::ConfirmationTokenMismatch)?;

        let expires_at = confirmation_sent_at + Duration::hours(24);

        if Utc::now() > expires_at {
            return Err(EmailConfirmationError::ConfirmationTokenExpired);
        }

        if !constant_time_eq(token.as_bytes(), expected_token.as_bytes()) {
            return Err(EmailConfirmationError::ConfirmationTokenMismatch);
        }

        self.user_repo.update_email_confirmed(&user.id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use testresult::TestResult;

    use crate::domain::{
        auth::users::tests::MockUserRepository,
        communication::{
            email_addresses::EmailAddress,
            mailer::{tests::MockMailer, MailerError},
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_generate_email_confirmation_token_and_update_user() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut repo = MockUserRepository::new();

        repo.expect_update_email_confirmation_token()
            .times(1)
            .returning(move |_, _| Ok(()));

        let service = EmailAddressServiceImpl::new(Arc::new(repo), Arc::new(MockMailer::new()));

        let (token, expires_at) = service.generate_email_confirmation_token(&user_id).await?;

        assert_eq!(44, token.len());
        assert!(expires_at > Utc::now());

        Ok(())
    }

    #[tokio::test]
    async fn test_send_email_confirmation_success() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserRepository::new();

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            email_confirmed_at: None,
            email_confirmation_token: None,
            email_confirmation_sent_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let expected_user = user.clone();

        users
            .expect_update_email_confirmation_token()
            .times(1)
            .returning(move |_, _| Ok(()));

        let mut mailer = MockMailer::new();

        mailer
            .expect_send_email()
            .times(1)
            .returning(move |_, _, _, _| Ok(()));

        let service = EmailAddressServiceImpl::new(Arc::new(users), Arc::new(mailer));

        let expires_at = service
            .send_email_confirmation(&expected_user, "https://localhost:3443")
            .await?;

        assert!(expires_at > Utc::now());

        Ok(())
    }

    #[tokio::test]
    async fn test_send_confirmation_email_failure() -> TestResult {
        let user = User::default();

        let mut users = MockUserRepository::new();
        let mut mailer = MockMailer::new();

        users
            .expect_update_email_confirmation_token()
            .times(1)
            .returning(|_, _| Ok(()));

        mailer
            .expect_send_email()
            .times(1)
            .returning(|_, _, _, _| Err(MailerError::SendError));

        let service = EmailAddressServiceImpl::new(Arc::new(users), Arc::new(mailer));

        let result = service
            .send_email_confirmation(&user, "https://localhost:3443")
            .await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_email_success() -> TestResult {
        let user_id = Uuid::now_v7();
        let yesterday = Utc::now() - Duration::days(1);

        let mut users = MockUserRepository::new();

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            email_confirmed_at: None,
            email_confirmation_token: Some("token".to_string()),
            email_confirmation_sent_at: Some(yesterday.clone() + Duration::hours(12)),
            created_at: yesterday.clone(),
            updated_at: yesterday.clone(),
        };

        let expected_user = user.clone();

        users
            .expect_update_email_confirmed()
            .times(1)
            .with(eq(user_id.clone()))
            .returning(|_| Ok(()));

        let service = EmailAddressServiceImpl::new(Arc::new(users), Arc::new(MockMailer::new()));

        let result = service.confirm_email(&expected_user, "token").await;

        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_email_incorrect_token() -> TestResult {
        let user_id = Uuid::now_v7();
        let yesterday = Utc::now() - Duration::days(1);

        let mut users = MockUserRepository::new();

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            email_confirmed_at: None,
            email_confirmation_token: Some("token".to_string()),
            email_confirmation_sent_at: Some(yesterday.clone()),
            created_at: yesterday.clone(),
            updated_at: yesterday.clone(),
        };

        let expected_user = user.clone();

        users.expect_update_email_confirmed().times(0);

        let service = EmailAddressServiceImpl::new(Arc::new(users), Arc::new(MockMailer::new()));

        let result = service
            .confirm_email(&expected_user, "incorrect token")
            .await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_email_expired_token() -> TestResult {
        let user_id = Uuid::now_v7();
        let last_week = Utc::now() - Duration::weeks(1);

        let mut users = MockUserRepository::new();

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            email_confirmed_at: None,
            email_confirmation_token: Some("token".to_string()),
            email_confirmation_sent_at: Some(last_week.clone()),
            created_at: last_week.clone(),
            updated_at: last_week.clone(),
        };

        let expected_user = user.clone();

        users.expect_update_email_confirmed().times(0);

        let service = EmailAddressServiceImpl::new(Arc::new(users), Arc::new(MockMailer::new()));

        let result = service.confirm_email(&expected_user, "token").await;

        assert!(result.is_err());

        Ok(())
    }
}
