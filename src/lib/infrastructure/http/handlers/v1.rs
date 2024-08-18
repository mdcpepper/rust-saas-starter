use axum::{
    routing::{get, post},
    Json, Router,
};
use utoipa::OpenApi;

use crate::{
    domain::auth::services::user::UserManagement,
    infrastructure::http::{open_api::ApiDocs, state::AppState},
};

pub mod auth;
pub mod stoplight;
pub mod uptime;

pub fn router<US: UserManagement>() -> Router<AppState<US>> {
    Router::new()
        .route("/", get(stoplight::handler))
        .route("/openapi.json", get(Json(ApiDocs::openapi())))
        .route("/uptime", get(uptime::handler))
        .route("/users", post(auth::create_user::handler))
}
