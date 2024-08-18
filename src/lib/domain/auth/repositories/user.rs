//! User repository module

use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

use crate::domain::auth::models::user::{CreateUserError, NewUser};

/// User repository
#[async_trait]
pub trait UserRepository: Clone + Send + Sync + 'static {
    /// Create a new user
    async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;
}

mock! {
    pub UserRepository {}

    impl Clone for UserRepository {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl UserRepository for UserRepository {
        async fn create_user(&self, req: &NewUser) -> Result<Uuid, CreateUserError>;
    }
}
