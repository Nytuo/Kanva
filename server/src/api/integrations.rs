use axum::{
    extract::{Json, Path, State},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::integrations as integration_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_integrations))
        .route("/board/:board_id", post(create_integration))
        .route("/:id", put(update_integration).delete(delete_integration))
        .route("/:id/sync", post(sync_integration))
        .route("/webhook/:provider/:id", post(handle_webhook))
        // GitHub-specific
        .route("/github/repos", get(list_github_repos))
        .route("/github/issues/:board_id", get(import_github_issues))
        // GitLab-specific
        .route("/gitlab/projects", get(list_gitlab_projects))
        .route("/gitlab/issues/:board_id", get(import_gitlab_issues))
        // Atlassian-specific
        .route("/atlassian/projects", get(list_atlassian_projects))
        .route("/atlassian/issues/:board_id", get(import_atlassian_issues))
}

#[derive(Debug, Deserialize)]
pub struct CreateIntegrationRequest {
    pub provider: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct IntegrationResponse {
    pub id: Uuid,
    pub board_id: Uuid,
    pub provider: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_integrations(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<IntegrationResponse>>, AppError> {
    let integrations = integration_service::list_integrations(&state, auth_user.user_id).await?;
    Ok(Json(integrations))
}

async fn create_integration(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(board_id): Path<Uuid>,
    Json(req): Json<CreateIntegrationRequest>,
) -> Result<Json<IntegrationResponse>, AppError> {
    let integration = integration_service::create_integration(&state, auth_user.user_id, board_id, req).await?;
    Ok(Json(integration))
}

async fn update_integration(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<IntegrationResponse>, AppError> {
    let integration = integration_service::update_integration(&state, auth_user.user_id, id, req).await?;
    Ok(Json(integration))
}

async fn delete_integration(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    integration_service::delete_integration(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Integration deleted"})))
}

async fn sync_integration(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = integration_service::sync_integration(&state, auth_user.user_id, id).await?;
    Ok(Json(result))
}

async fn handle_webhook(
    State(state): State<AppState>,
    Path((provider, id)): Path<(String, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    integration_service::handle_webhook(&state, &provider, id, payload).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

async fn list_github_repos(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let repos = integration_service::list_github_repos(&state, auth_user.user_id).await?;
    Ok(Json(repos))
}

async fn import_github_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(board_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = integration_service::import_github_issues(&state, auth_user.user_id, board_id).await?;
    Ok(Json(result))
}

async fn list_gitlab_projects(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let projects = integration_service::list_gitlab_projects(&state, auth_user.user_id).await?;
    Ok(Json(projects))
}

async fn import_gitlab_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(board_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = integration_service::import_gitlab_issues(&state, auth_user.user_id, board_id).await?;
    Ok(Json(result))
}

async fn list_atlassian_projects(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let projects = integration_service::list_atlassian_projects(&state, auth_user.user_id).await?;
    Ok(Json(projects))
}

async fn import_atlassian_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(board_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = integration_service::import_atlassian_issues(&state, auth_user.user_id, board_id).await?;
    Ok(Json(result))
}
