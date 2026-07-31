use axum::{
    extract::{Json, Path, Query, State},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::notes as note_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notes).post(create_note))
        .route("/:id", get(get_note).put(update_note).delete(delete_note))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNoteRequest {
    /// Omit for a global (not tied to a board) note.
    pub board_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNoteRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub content: Option<String>,
    pub position: Option<i32>,
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotesQuery {
    /// Filter to a single board's notes. Omit to list the caller's global notes.
    pub board_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub board_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub position: i32,
    pub is_pinned: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

async fn list_notes(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListNotesQuery>,
) -> Result<Json<Vec<NoteResponse>>, AppError> {
    let notes = note_service::list_notes(&state, auth_user.user_id, query.board_id).await?;
    Ok(Json(notes))
}

async fn create_note(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateNoteRequest>,
) -> Result<Json<NoteResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let note = note_service::create_note(&state, auth_user.user_id, req).await?;
    Ok(Json(note))
}

async fn get_note(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<NoteResponse>, AppError> {
    let note = note_service::get_note(&state, auth_user.user_id, id).await?;
    Ok(Json(note))
}

async fn update_note(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<NoteResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let note = note_service::update_note(&state, auth_user.user_id, id, req).await?;
    Ok(Json(note))
}

async fn delete_note(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    note_service::delete_note(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Note deleted"})))
}
