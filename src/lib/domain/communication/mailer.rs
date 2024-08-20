//! Mailer module

mod errors;
mod mailer;

pub use errors::MailerError;
pub use mailer::Mailer;

#[cfg(test)]
pub mod tests {
    pub use super::mailer::MockMailer;
}
