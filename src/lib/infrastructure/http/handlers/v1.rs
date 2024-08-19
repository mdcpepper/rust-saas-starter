//! Version 1 of the API

use axum::{
    routing::{get, post},
    Json, Router,
};
use utoipa::OpenApi;

use crate::{
    domain::auth::services::user::UserServiceImpl,
    infrastructure::http::{open_api::ApiDocs, state::AppState},
};

pub mod auth;
pub mod stoplight;
pub mod uptime;

/// Create the router for version 1 of the API
pub fn router<US: UserServiceImpl>() -> Router<AppState<US>> {
    Router::new()
        .route("/", get(stoplight::handler))
        .route("/openapi.json", get(Json(ApiDocs::openapi())))
        .route("/uptime", get(uptime::handler))
        .route("/users", post(auth::create_user::handler))
}
