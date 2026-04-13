use axum::{
    extract::{Json, Path, Query, State},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::boards as board_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_boards).post(create_board))
        .route("/:id", get(get_board).put(update_board).delete(delete_board))
        .route("/:id/members", get(list_board_members).post(add_board_member))
        .route("/:id/members/:user_id", delete(remove_board_member))
        .route("/:id/labels", get(list_labels).post(create_label))
        .route("/:id/labels/:label_id", put(update_label).delete(delete_label))
        .route("/:id/activity", get(get_board_activity))
        .route("/:id/archive", post(archive_board))
        .route("/:id/star", post(toggle_star))
        .route("/:id/custom-fields", get(list_custom_fields).post(create_custom_field))
        .route("/templates", get(list_templates).post(create_template))
        .route("/templates/:id", post(create_from_template))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBoardRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub team_id: Option<Uuid>,
    pub background_color: Option<String>,
    pub background_image_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBoardRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub background_color: Option<String>,
    pub background_image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BoardResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub visibility: String,
    pub background_color: Option<String>,
    pub background_image_url: Option<String>,
    pub is_starred: bool,
    pub is_archived: bool,
    pub owner_id: Uuid,
    pub team_id: Option<Uuid>,
    pub lists: Vec<ListWithCardsResponse>,
    pub labels: Vec<LabelResponse>,
    pub members: Vec<BoardMemberResponse>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct BoardSummaryResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub visibility: String,
    pub background_color: Option<String>,
    pub background_image_url: Option<String>,
    pub is_starred: bool,
    pub is_archived: bool,
    pub owner_id: Uuid,
    pub team_id: Option<Uuid>,
    pub member_count: i64,
    pub card_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListWithCardsResponse {
    pub id: Uuid,
    pub title: String,
    pub position: i32,
    pub cards: Vec<CardSummaryResponse>,
}

#[derive(Debug, Serialize)]
pub struct CardSummaryResponse {
    pub id: Uuid,
    pub title: String,
    pub position: i32,
    pub priority: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub assignee_count: i64,
    pub comment_count: i64,
    pub checklist_progress: Option<String>,
    pub label_ids: Vec<Uuid>,
    pub cover_color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LabelResponse {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct BoardMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct ListBoardsQuery {
    pub team_id: Option<Uuid>,
    pub visibility: Option<String>,
    pub archived: Option<bool>,
    pub starred: Option<bool>,
}

async fn list_boards(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListBoardsQuery>,
) -> Result<Json<Vec<BoardSummaryResponse>>, AppError> {
    let boards = board_service::list_boards(&state, auth_user.user_id, query).await?;
    Ok(Json(boards))
}

async fn create_board(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateBoardRequest>,
) -> Result<Json<BoardResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let board = board_service::create_board(&state, auth_user.user_id, req).await?;
    Ok(Json(board))
}

async fn get_board(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BoardResponse>, AppError> {
    let board = board_service::get_board(&state, auth_user.user_id, id).await?;
    Ok(Json(board))
}

async fn update_board(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBoardRequest>,
) -> Result<Json<BoardResponse>, AppError> {
    let board = board_service::update_board(&state, auth_user.user_id, id, req).await?;
    Ok(Json(board))
}

async fn delete_board(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    board_service::delete_board(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Board deleted"})))
}

#[derive(Debug, Deserialize)]
pub struct AddBoardMemberRequest {
    pub user_id: Uuid,
    pub role: Option<String>,
}

async fn list_board_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BoardMemberResponse>>, AppError> {
    let members = board_service::list_board_members(&state, auth_user.user_id, id).await?;
    Ok(Json(members))
}

async fn add_board_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AddBoardMemberRequest>,
) -> Result<Json<BoardMemberResponse>, AppError> {
    let member = board_service::add_board_member(&state, auth_user.user_id, id, req).await?;
    Ok(Json(member))
}

async fn remove_board_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    board_service::remove_board_member(&state, auth_user.user_id, id, user_id).await?;
    Ok(Json(serde_json::json!({"message": "Member removed"})))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLabelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub color: String,
}

async fn list_labels(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<LabelResponse>>, AppError> {
    let labels = board_service::list_labels(&state, auth_user.user_id, id).await?;
    Ok(Json(labels))
}

async fn create_label(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Json<LabelResponse>, AppError> {
    let label = board_service::create_label(&state, auth_user.user_id, id, req).await?;
    Ok(Json(label))
}

async fn update_label(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, label_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Json<LabelResponse>, AppError> {
    let label = board_service::update_label(&state, auth_user.user_id, id, label_id, req).await?;
    Ok(Json(label))
}

async fn delete_label(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    board_service::delete_label(&state, auth_user.user_id, id, label_id).await?;
    Ok(Json(serde_json::json!({"message": "Label deleted"})))
}

async fn get_board_activity(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let activity = board_service::get_board_activity(&state, auth_user.user_id, id).await?;
    Ok(Json(activity))
}

async fn archive_board(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    board_service::archive_board(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Board archived"})))
}

async fn toggle_star(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let starred = board_service::toggle_star(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"starred": starred})))
}

async fn list_custom_fields(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let fields = board_service::list_custom_fields(&state, auth_user.user_id, id).await?;
    Ok(Json(fields))
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomFieldRequest {
    pub name: String,
    pub field_type: String,
    pub options: Option<Vec<String>>,
}

async fn create_custom_field(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateCustomFieldRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let field = board_service::create_custom_field(&state, auth_user.user_id, id, req).await?;
    Ok(Json(field))
}

async fn list_templates(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let templates = board_service::list_templates(&state, auth_user.user_id).await?;
    Ok(Json(templates))
}

async fn create_template(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let template = board_service::create_template(&state, auth_user.user_id, req).await?;
    Ok(Json(template))
}

async fn create_from_template(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<BoardResponse>, AppError> {
    let board = board_service::create_from_template(&state, auth_user.user_id, id, req).await?;
    Ok(Json(board))
}
