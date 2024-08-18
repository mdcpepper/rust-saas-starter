//! Application state module

use std::sync::Arc;

use std::fmt;

use chrono::{DateTime, Utc};

use crate::domain::auth::services::user::UserManagement;

/// Global application state
#[derive(Clone)]
pub struct AppState<US: UserManagement> {
    /// The time the server started
    pub start_time: DateTime<Utc>,

    /// User service
    pub users: Arc<US>,
}

impl<US> fmt::Debug for AppState<US>
where
    US: UserManagement,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("start_time", &self.start_time)
            .field("users", &"UserService")
            .finish()
    }
}

#[cfg(test)]
pub type MockAppState = AppState<UserService<MockUserRepository>>;

#[cfg(test)]
use crate::domain::auth::{repositories::user::MockUserRepository, services::user::UserService};

#[cfg(test)]
pub fn get_test_state(user_repo: MockUserRepository) -> AppState<UserService<MockUserRepository>> {
    let user_service = UserService::new(Arc::new(user_repo));
    AppState {
        start_time: Utc::now(),
        users: Arc::new(user_service),
    }
}
