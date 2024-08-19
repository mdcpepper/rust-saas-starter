//! User service module.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::mock;

use crate::domain::auth::{
    models::user::{CreateUserError, NewUser},
    repositories::user::UserRepository,
};

/// User service
#[async_trait]
pub trait UserServiceImpl: Clone + Send + Sync + 'static {
    /// Creates a new user based on the provided request details.
    ///
    /// # Arguments
    /// * `req` - A reference to a [`CreateUserRequest`] containing the user details.
    ///
    /// # Returns
    /// A [`Result`] which is [`Ok`] containing the user's UUID if the user is successfully created,
    /// or an [`Err`] containing a [`CreateUserError`] if the user cannot be created.
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;
}

#[cfg(test)]
mock! {
    pub UserServiceImpl {}

    impl Clone for UserServiceImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl UserServiceImpl for UserServiceImpl {
        async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;
    }
}

/// User service implementation
#[derive(Debug, Clone)]
pub struct UserService<R>
where
    R: UserRepository,
{
    repo: Arc<R>,
}

impl<R> UserService<R>
where
    R: UserRepository,
{
    /// Create a new user service
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> UserServiceImpl for UserService<R>
where
    R: UserRepository,
{
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError> {
        self.repo.create_user(req).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::anyhow;
    use mockall::predicate::eq;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::domain::auth::{
        models::user::{CreateUserError, NewUser},
        repositories::user::MockUserRepository,
        services::user::{UserService, UserServiceImpl},
        value_objects::{email_address::EmailAddress, password::Password},
    };

    #[tokio::test]
    async fn test_create_user_success() -> TestResult {
        let user_id = Uuid::now_v7();
        let request = NewUser::new(
            user_id,
            EmailAddress::new("email@example.com")?,
            Password::new("correcthorsebatterystaple")?,
        );
        let expected_id = request.id().clone();

        let mut mock = MockUserRepository::new();

        mock.expect_create_user()
            .times(1)
            .with(eq(request.clone()))
            .returning(move |_| Ok(expected_id));

        let service = UserService::new(Arc::new(mock));

        let user_id = service.create_user(&request).await?;

        assert_eq!(&user_id, request.id());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_user_already_exists() -> TestResult {
        let user_id = Uuid::now_v7();
        let request = NewUser::new(
            user_id,
            EmailAddress::new("email@example.com")?,
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

        let service = UserService::new(Arc::new(mock));

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
            EmailAddress::new("email@example.com")?,
            Password::new("correcthorsebatterystaple")?,
        );

        let mut mock = MockUserRepository::new();

        mock.expect_create_user()
            .times(1)
            .with(eq(request.clone()))
            .returning(move |_req| Err(CreateUserError::UnknownError(anyhow!("Unknown error"))));

        let service = UserService::new(Arc::new(mock));

        let result = service.create_user(&request).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CreateUserError::UnknownError { .. })));

        Ok(())
    }
}
