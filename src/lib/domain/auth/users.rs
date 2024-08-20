//! This module contains the user model and its related functions.

mod password;
mod repository;
mod service;
mod user;

pub mod errors;

pub use password::{Password, PasswordError};
pub use repository::UserRepository;
pub use service::{UserService, UserServiceImpl};
pub use user::{NewUser, User};

#[cfg(test)]
pub mod tests {
    pub use super::repository::MockUserRepository;
    pub use super::service::MockUserService;
}
