//! OpenAPI 3.0.0 module

use utoipa::OpenApi;

use crate::infrastructure::http::{errors::ErrorResponse, handlers::v1::*};

#[derive(Debug, OpenApi)]
#[openapi(
    info(title = "SaaS Starter"),
    paths(auth::create_user::handler, uptime::handler),
    components(schemas(
        auth::create_user::CreateUserBody,
        auth::create_user::CreateUserResponse,
        uptime::UptimeResponse,
        ErrorResponse,
    ))
)]
pub struct ApiDocs;
