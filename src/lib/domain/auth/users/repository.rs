//! User repository module

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::mock;

use crate::domain::{
    auth::users::{
        errors::{CreateUserError, GetUserByIdError, UpdateUserError},
        NewUser, User,
    },
    communication::email_addresses::EmailAddress,
};

/// User repository
#[async_trait]
pub trait UserRepository: Clone + Send + Sync + 'static {
    /// Create a new user
    async fn create_user(&self, user: &NewUser) -> Result<Uuid, CreateUserError>;

    /// Get a user by their ID
    async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError>;

    /// Update the email confirmation token for a user
    async fn initialize_email_confirmation<'a>(
        &self,
        user_id: &Uuid,
        token: &str,
        new_email: Option<&'a EmailAddress>,
    ) -> Result<(), UpdateUserError>;

    /// Update the email confirmed date for a user
    async fn complete_email_confirmation<'a>(
        &self,
        user_id: &Uuid,
        new_email: Option<&'a EmailAddress>,
    ) -> Result<(), UpdateUserError>;
}

#[cfg(test)]
mock! {
    pub UserRepository {}

    impl Clone for UserRepository {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl UserRepository for UserRepository {
        async fn create_user(&self, user: &NewUser) -> Result<Uuid, CreateUserError>;
        async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError>;
        async fn initialize_email_confirmation<'a>(
            &self,
            user_id: &Uuid,
            token: &str,
            new_email: Option<&'a EmailAddress>,
        ) -> Result<(), UpdateUserError>;
        async fn complete_email_confirmation<'a>(&self, user_id: &Uuid, new_email: Option<&'a EmailAddress>) -> Result<(), UpdateUserError>;
    }
}
