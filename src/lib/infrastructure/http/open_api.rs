//! OpenAPI 3.0.0 module

use utoipa::OpenApi;

use crate::infrastructure::http::{errors::ErrorResponse, handlers::v1::uptime};

#[derive(Debug, OpenApi)]
#[openapi(
    info(title = "SaaS Starter"),
    paths(uptime::handler),
    components(schemas(uptime::UptimeResponse, ErrorResponse))
)]
pub struct ApiDocs;
