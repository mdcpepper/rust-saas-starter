use askama::Template;

#[derive(Debug, Template)]
#[template(path = "auth/email_confirmed.html")]
pub struct EmailConfirmedTemplate;
