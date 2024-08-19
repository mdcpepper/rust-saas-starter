//! OpenAPI module

use utoipa::OpenApi;

use crate::infrastructure::http::{errors::ErrorResponse, handlers::v1::*};

#[derive(Debug, OpenApi)]
#[openapi(
    info(title = "SaaS Starter"),
    paths(
        auth::create_user::handler,
        auth::get_user_by_id::handler,
        auth::send_email_confirmation::handler,
        uptime::handler
    ),
    components(schemas(
        auth::create_user::CreateUserBody,
        auth::create_user::CreateUserResponse,
        auth::get_user_by_id::GetUserByIdResponse,
        auth::send_email_confirmation::SendEmailConfirmationResponse,
        uptime::UptimeResponse,
        ErrorResponse,
    ))
)]
pub struct ApiDocs;
