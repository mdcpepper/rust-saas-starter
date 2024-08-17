//! User repository module

use std::future::Future;

use mockall::automock;
use uuid::Uuid;

use crate::domain::auth::models::user::{CreateUserError, CreateUserRequest};

/// User repository
#[automock]
pub trait UserRepository: Send + Sync + 'static {
    /// Create a new user
    fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<Uuid, CreateUserError>> + Send;
}
