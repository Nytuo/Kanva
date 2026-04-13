use axum::{
    extract::{Json, Path, State},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::users as user_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/profile", get(get_profile).put(update_profile))
        .route("/avatar", post(upload_avatar))
        .route("/account", delete(delete_account))
        .route("/preferences", get(get_preferences).put(update_preferences))
        .route("/password", put(change_password))
        .route("/notifications", get(list_notifications))
        .route("/notifications/read-all", put(mark_all_notifications_read))
        .route("/notifications/:id/read", put(mark_notification_read))
        .route("/search", get(search_users))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserPreferencesResponse {
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub default_board_view: String,
    pub compact_mode: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub email_notifications: Option<bool>,
    pub push_notifications: Option<bool>,
    pub default_board_view: Option<String>,
    pub compact_mode: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SearchUsersQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = user_service::get_profile(&state, auth_user.user_id).await?;
    Ok(Json(profile))
}

async fn update_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = user_service::update_profile(&state, auth_user.user_id, req).await?;
    Ok(Json(profile))
}

async fn get_preferences(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserPreferencesResponse>, AppError> {
    let prefs = user_service::get_preferences(&state, auth_user.user_id).await?;
    Ok(Json(prefs))
}

async fn update_preferences(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdatePreferencesRequest>,
) -> Result<Json<UserPreferencesResponse>, AppError> {
    let prefs = user_service::update_preferences(&state, auth_user.user_id, req).await?;
    Ok(Json(prefs))
}

async fn list_notifications(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<NotificationResponse>>, AppError> {
    let notifications = user_service::list_notifications(&state, auth_user.user_id).await?;
    Ok(Json(notifications))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    user_service::mark_notification_read(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Notification marked as read"})))
}

async fn mark_all_notifications_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    user_service::mark_all_notifications_read(&state, auth_user.user_id).await?;
    Ok(Json(serde_json::json!({"message": "All notifications marked as read"})))
}

async fn search_users(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    axum::extract::Query(query): axum::extract::Query<SearchUsersQuery>,
) -> Result<Json<Vec<UserSearchResult>>, AppError> {
    let users = user_service::search_users(&state, &query.q).await?;
    Ok(Json(users))
}

async fn change_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.new_password.len() < 8 {
        return Err(AppError::BadRequest("New password must be at least 8 characters".to_string()));
    }
    user_service::change_password(&state, auth_user.user_id, &req.current_password, &req.new_password).await?;
    Ok(Json(serde_json::json!({"message": "Password changed successfully"})))
}

async fn upload_avatar(
    State(state): State<AppState>,
    auth_user: AuthUser,
    multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let avatar_url = user_service::upload_avatar(&state, auth_user.user_id, multipart).await?;
    Ok(Json(serde_json::json!({"avatar_url": avatar_url})))
}

async fn delete_account(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    user_service::delete_account(&state, auth_user.user_id).await?;
    Ok(Json(serde_json::json!({"message": "Account deleted"})))
}
