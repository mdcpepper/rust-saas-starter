//! Version 1 of the API

use axum::{
    routing::{get, post},
    Json, Router,
};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;

use crate::{
    domain::auth::services::user::UserService,
    infrastructure::http::{open_api::ApiDocs, state::AppState},
};

pub mod auth;
pub mod stoplight;
pub mod uptime;

/// Create the router for version 1 of the API
pub fn router<U: UserService>() -> Router<AppState<U>> {
    Router::new()
        .route("/", get(stoplight::handler))
        .route("/openapi.json", get(Json(ApiDocs::openapi())))
        .route("/uptime", get(uptime::handler))
        .route("/users/:id", get(auth::get_user_by_id::handler))
        .route(
            "/users/:id/email/confirmation",
            post(auth::send_email_confirmation::handler),
        )
        .route(
            "/users/:id/email/confirmation",
            get(auth::confirm_email::handler),
        )
        .route("/users", post(auth::create_user::handler))
        .layer(
            CompressionLayer::new()
                .br(true)
                .deflate(true)
                .gzip(true)
                .zstd(true),
        )
}
