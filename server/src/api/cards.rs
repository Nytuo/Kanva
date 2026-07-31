use axum::{
    extract::{DefaultBodyLimit, Json, Path, State},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::cards as card_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_card))
        .route("/:id", get(get_card).put(update_card).delete(delete_card))
        .route("/:id/move", post(move_card))
        .route("/:id/assign", post(assign_card))
        .route("/:id/unassign/:user_id", delete(unassign_card))
        .route("/:id/labels/:label_id", post(add_label).delete(remove_label))
        .route("/:id/comments", get(list_comments).post(create_comment))
        .route("/:id/comments/:comment_id", put(update_comment).delete(delete_comment))
        .route("/:id/checklists", get(list_checklists).post(create_checklist))
        .route("/:id/checklists/:checklist_id", put(update_checklist).delete(delete_checklist))
        .route("/:id/checklists/:checklist_id/items", post(create_checklist_item))
        .route("/:id/checklists/:checklist_id/items/:item_id", put(update_checklist_item).delete(delete_checklist_item))
        .route(
            "/:id/attachments",
            // Body-size enforcement against config.max_upload_size_mb happens in
            // the service layer; this just raises axum's 2MB default so uploads
            // up to our largest configured limit (standalone: 50MB) aren't rejected
            // before they even reach that check.
            get(list_attachments)
                .post(upload_attachment)
                .layer(DefaultBodyLimit::max(100 * 1024 * 1024)),
        )
        .route("/:id/attachments/:attachment_id", delete(delete_attachment))
        .route("/:id/custom-fields", get(get_custom_field_values).put(set_custom_field_value))
        .route("/:id/archive", post(archive_card))
        .route("/:id/complete", post(complete_card))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCardRequest {
    pub list_id: Uuid,
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub description: Option<String>,
    pub position: Option<i32>,
    pub priority: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub assignee_ids: Option<Vec<Uuid>>,
    pub label_ids: Option<Vec<Uuid>>,
    pub cover_color: Option<String>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCardRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    /// Send an ISO datetime string to set, or empty string "" to clear
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub cover_color: Option<String>,
    pub cover_image_url: Option<String>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct MoveCardRequest {
    pub list_id: Uuid,
    pub position: i32,
}

#[derive(Debug, Serialize)]
pub struct CardResponse {
    pub id: Uuid,
    pub list_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub priority: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_archived: bool,
    pub cover_color: Option<String>,
    pub cover_image_url: Option<String>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub created_by: Uuid,
    pub assignees: Vec<AssigneeResponse>,
    pub labels: Vec<super::boards::LabelResponse>,
    pub checklists: Vec<ChecklistResponse>,
    pub comments: Vec<CommentResponse>,
    pub attachments: Vec<AttachmentResponse>,
    pub custom_field_values: Vec<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AssigneeResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChecklistResponse {
    pub id: Uuid,
    pub title: String,
    pub position: i32,
    pub items: Vec<ChecklistItemResponse>,
}

#[derive(Debug, Serialize)]
pub struct ChecklistItemResponse {
    pub id: Uuid,
    pub title: String,
    pub is_checked: bool,
    pub position: i32,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub content: String,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub filename: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

async fn create_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateCardRequest>,
) -> Result<Json<CardResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let card = card_service::create_card(&state, auth_user.user_id, req).await?;
    Ok(Json(card))
}

async fn get_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CardResponse>, AppError> {
    let card = card_service::get_card(&state, auth_user.user_id, id).await?;
    Ok(Json(card))
}

async fn update_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<CardResponse>, AppError> {
    let card = card_service::update_card(&state, auth_user.user_id, id, req).await?;
    Ok(Json(card))
}

async fn delete_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::delete_card(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Card deleted"})))
}

async fn move_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveCardRequest>,
) -> Result<Json<CardResponse>, AppError> {
    let card = card_service::move_card(&state, auth_user.user_id, id, req).await?;
    Ok(Json(card))
}

#[derive(Debug, Deserialize)]
pub struct AssignCardRequest {
    pub user_id: Uuid,
}

async fn assign_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::assign_card(&state, auth_user.user_id, id, req.user_id).await?;
    Ok(Json(serde_json::json!({"message": "Card assigned"})))
}

async fn unassign_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::unassign_card(&state, auth_user.user_id, id, user_id).await?;
    Ok(Json(serde_json::json!({"message": "Card unassigned"})))
}

async fn add_label(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::add_label(&state, auth_user.user_id, id, label_id).await?;
    Ok(Json(serde_json::json!({"message": "Label added"})))
}

async fn remove_label(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::remove_label(&state, auth_user.user_id, id, label_id).await?;
    Ok(Json(serde_json::json!({"message": "Label removed"})))
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

async fn list_comments(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CommentResponse>>, AppError> {
    let comments = card_service::list_comments(&state, auth_user.user_id, id).await?;
    Ok(Json(comments))
}

async fn create_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let comment = card_service::create_comment(&state, auth_user.user_id, id, req.content).await?;
    Ok(Json(comment))
}

async fn update_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, comment_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let comment = card_service::update_comment(&state, auth_user.user_id, id, comment_id, req.content).await?;
    Ok(Json(comment))
}

async fn delete_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::delete_comment(&state, auth_user.user_id, id, comment_id).await?;
    Ok(Json(serde_json::json!({"message": "Comment deleted"})))
}

#[derive(Debug, Deserialize)]
pub struct CreateChecklistRequest {
    pub title: String,
}

async fn list_checklists(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ChecklistResponse>>, AppError> {
    let checklists = card_service::list_checklists(&state, auth_user.user_id, id).await?;
    Ok(Json(checklists))
}

async fn create_checklist(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateChecklistRequest>,
) -> Result<Json<ChecklistResponse>, AppError> {
    let checklist = card_service::create_checklist(&state, auth_user.user_id, id, req.title).await?;
    Ok(Json(checklist))
}

async fn update_checklist(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, checklist_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreateChecklistRequest>,
) -> Result<Json<ChecklistResponse>, AppError> {
    let checklist = card_service::update_checklist(&state, auth_user.user_id, id, checklist_id, req.title).await?;
    Ok(Json(checklist))
}

async fn delete_checklist(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, checklist_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::delete_checklist(&state, auth_user.user_id, id, checklist_id).await?;
    Ok(Json(serde_json::json!({"message": "Checklist deleted"})))
}

#[derive(Debug, Deserialize)]
pub struct CreateChecklistItemRequest {
    pub title: String,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

async fn create_checklist_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, checklist_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreateChecklistItemRequest>,
) -> Result<Json<ChecklistItemResponse>, AppError> {
    let item = card_service::create_checklist_item(&state, auth_user.user_id, id, checklist_id, req).await?;
    Ok(Json(item))
}

#[derive(Debug, Deserialize)]
pub struct UpdateChecklistItemRequest {
    pub title: Option<String>,
    pub is_checked: Option<bool>,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

async fn update_checklist_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, checklist_id, item_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(req): Json<UpdateChecklistItemRequest>,
) -> Result<Json<ChecklistItemResponse>, AppError> {
    let item = card_service::update_checklist_item(&state, auth_user.user_id, id, checklist_id, item_id, req).await?;
    Ok(Json(item))
}

async fn delete_checklist_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, checklist_id, item_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::delete_checklist_item(&state, auth_user.user_id, id, checklist_id, item_id).await?;
    Ok(Json(serde_json::json!({"message": "Checklist item deleted"})))
}

async fn list_attachments(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AttachmentResponse>>, AppError> {
    let attachments = card_service::list_attachments(&state, auth_user.user_id, id).await?;
    Ok(Json(attachments))
}

async fn upload_attachment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    multipart: axum::extract::Multipart,
) -> Result<Json<AttachmentResponse>, AppError> {
    let attachment = card_service::upload_attachment(&state, auth_user.user_id, id, multipart).await?;
    Ok(Json(attachment))
}

async fn delete_attachment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::delete_attachment(&state, auth_user.user_id, id, attachment_id).await?;
    Ok(Json(serde_json::json!({"message": "Attachment deleted"})))
}

async fn get_custom_field_values(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let values = card_service::get_custom_field_values(&state, auth_user.user_id, id).await?;
    Ok(Json(values))
}

#[derive(Debug, Deserialize)]
pub struct SetCustomFieldValueRequest {
    pub field_id: Uuid,
    pub value: serde_json::Value,
}

async fn set_custom_field_value(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<SetCustomFieldValueRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let value = card_service::set_custom_field_value(&state, auth_user.user_id, id, req).await?;
    Ok(Json(value))
}

async fn archive_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::archive_card(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Card archived"})))
}

async fn complete_card(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    card_service::complete_card(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Card completed"})))
}
