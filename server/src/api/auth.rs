use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{AppState, AppError};
use crate::services::auth as auth_service;
use crate::middleware::auth::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/oauth/:provider", get(oauth_redirect))
        .route("/oauth/:provider/callback", get(oauth_callback))
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[validate(length(min = 1, max = 255))]
    pub display_name: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let response = auth_service::register(&state, req).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = auth_service::login(&state, req).await?;
    Ok(Json(response))
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = auth_service::refresh_token(&state, &req.refresh_token).await?;
    Ok(Json(response))
}

async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_service::logout(&state, auth_user.user_id).await?;
    Ok(Json(serde_json::json!({"message": "Logged out successfully"})))
}

async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = auth_service::get_user_by_id(&state, auth_user.user_id).await?;
    let user_uuid = uuid::Uuid::parse_str(&user.id)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid user UUID")))?;
    Ok(Json(UserResponse {
        id: user_uuid,
        email: user.email,
        username: user.username,
        display_name: user.display_name,
        avatar_url: crate::models::user::empty_to_none(user.avatar_url),
        bio: crate::models::user::empty_to_none(user.bio),
    }))
}

async fn oauth_redirect(
    State(state): State<AppState>,
    axum::extract::Path(provider): axum::extract::Path<String>,
) -> Result<axum::response::Redirect, AppError> {
    let url = auth_service::get_oauth_redirect_url(&state, &provider)?;
    Ok(axum::response::Redirect::temporary(&url))
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

async fn oauth_callback(
    State(state): State<AppState>,
    axum::extract::Path(provider): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<OAuthCallbackQuery>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = auth_service::handle_oauth_callback(&state, &provider, &query.code).await?;
    Ok(Json(response))
}
