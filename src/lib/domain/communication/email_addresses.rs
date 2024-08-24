//! Email addresses module.

mod email_address;
mod errors;
mod service;

pub use email_address::{EmailAddress, EmailAddressError};
pub use errors::EmailConfirmationError;
pub use service::{EmailAddressService, EmailAddressServiceImpl, EmailConfirmationType};

#[cfg(test)]
pub mod tests {
    pub use super::service::MockEmailAddressService;
}
