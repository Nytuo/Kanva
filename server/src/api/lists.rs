use axum::{
    extract::{Json, Path, State},
    routing::{post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::lists as list_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_list))
        .route("/:id", put(update_list).delete(delete_list))
        .route("/:id/move", post(move_list))
        .route("/:id/archive", post(archive_list))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateListRequest {
    pub board_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateListRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveListRequest {
    pub position: i32,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub id: Uuid,
    pub board_id: Uuid,
    pub title: String,
    pub position: i32,
    pub is_archived: bool,
    pub card_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

async fn create_list(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateListRequest>,
) -> Result<Json<ListResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let list = list_service::create_list(&state, auth_user.user_id, req).await?;
    Ok(Json(list))
}

async fn update_list(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateListRequest>,
) -> Result<Json<ListResponse>, AppError> {
    let list = list_service::update_list(&state, auth_user.user_id, id, req).await?;
    Ok(Json(list))
}

async fn delete_list(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    list_service::delete_list(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "List deleted"})))
}

async fn move_list(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveListRequest>,
) -> Result<Json<ListResponse>, AppError> {
    let list = list_service::move_list(&state, auth_user.user_id, id, req.position).await?;
    Ok(Json(list))
}

async fn archive_list(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    list_service::archive_list(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "List archived"})))
}
