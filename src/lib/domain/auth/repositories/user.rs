//! User repository module

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::domain::auth::models::user::{CreateUserError, CreateUserRequest};

/// User repository
#[automock]
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    /// Create a new user
    async fn create_user(&self, req: &CreateUserRequest) -> Result<Uuid, CreateUserError>;
}
