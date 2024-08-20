use askama::Template;

#[derive(Debug, Template)]
#[template(path = "errors/unprocessable.html")]
pub struct UnprocessableEntityErrorTemplate;
