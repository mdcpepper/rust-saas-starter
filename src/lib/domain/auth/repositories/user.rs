//! User repository module

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::mock;

use crate::domain::auth::{
    errors::{CreateUserError, GetUserByIdError},
    models::user::{NewUser, User},
};

/// User repository
#[async_trait]
pub trait UserRepository: Clone + Send + Sync + 'static {
    /// Create a new user
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;

    /// Get a user by their ID
    async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError>;
}

#[cfg(test)]
mock! {
    pub UserRepository {}

    impl Clone for UserRepository {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl UserRepository for UserRepository {
        async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;
        async fn get_user_by_id(&self, id: &Uuid) -> Result<User, GetUserByIdError>;
    }
}
