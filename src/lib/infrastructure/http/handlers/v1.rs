use axum::{routing::get, Json, Router};
use utoipa::OpenApi;

use crate::{
    domain::auth::services::user::UserManagement,
    infrastructure::http::{open_api::ApiDocs, state::AppState},
};

pub mod stoplight;
pub mod uptime;

pub fn router<US: UserManagement>() -> Router<AppState<US>> {
    Router::new()
        .route("/openapi.json", get(Json(ApiDocs::openapi())))
        .route("/", get(stoplight::handler))
        .route("/uptime", get(uptime::handler))
}
