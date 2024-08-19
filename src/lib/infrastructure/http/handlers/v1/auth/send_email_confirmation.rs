//! Send email confirmation email handler

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::auth::services::user::UserService,
    infrastructure::http::{errors::ApiError, state::AppState},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendEmailConfirmationResponse {
    success: bool,
}

/// Send an email confirmation email
#[utoipa::path(
    post,
    operation_id = "send_email_confirmation",
    tag = "Auth",
    path = "/api/v1/users/{id}/email-confirmation/send",
    params(
        ("id" = Uuid, Path, description = "The UUID of the user", example = "550e8400-e29b-41d4-a716-446655440000"),
    ),
    responses(
        (status = StatusCode::CREATED, description = "Email confirmation sent", body = SendEmailConfirmationResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = ErrorResponse, example = json!({ "error": "User with id \"550e8400-e29b-41d4-a716-446655440000\" not found" })),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = ErrorResponse, example = json!({ "error": "Failed to send email confirmation: <error>" })),
    )
)]
pub async fn handler<U: UserService>(
    State(state): State<AppState<U>>,
    Path(user_id): Path<Uuid>,
) -> Result<(StatusCode, Json<SendEmailConfirmationResponse>), ApiError> {
    state
        .users
        .send_email_confirmation(&user_id, &state.config.base_url)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(SendEmailConfirmationResponse { success: true }),
    ))
}
