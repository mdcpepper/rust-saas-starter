use askama::Template;

#[derive(Debug, Template)]
#[template(path = "errors/internal_server_error.html")]
pub struct InternalServerErrorTemplate;
