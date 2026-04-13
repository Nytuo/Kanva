use axum::{
    extract::{Json, Path, State},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::teams as team_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_teams).post(create_team))
        .route("/:id", get(get_team).put(update_team).delete(delete_team))
        .route("/:id/members", get(list_members).post(add_member))
        .route("/:id/members/:user_id", put(update_member_role).delete(remove_member))
        .route("/:id/invite", post(invite_member))
        .route("/invite/:token/accept", post(accept_invite))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub member_count: i64,
    pub board_count: i64,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

async fn list_teams(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<TeamResponse>>, AppError> {
    let teams = team_service::list_teams(&state, auth_user.user_id).await?;
    Ok(Json(teams))
}

async fn create_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let team = team_service::create_team(&state, auth_user.user_id, req).await?;
    Ok(Json(team))
}

async fn get_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamResponse>, AppError> {
    let team = team_service::get_team(&state, auth_user.user_id, id).await?;
    Ok(Json(team))
}

async fn update_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<Json<TeamResponse>, AppError> {
    let team = team_service::update_team(&state, auth_user.user_id, id, req).await?;
    Ok(Json(team))
}

async fn delete_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    team_service::delete_team(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Team deleted"})))
}

async fn list_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TeamMemberResponse>>, AppError> {
    let members = team_service::list_members(&state, auth_user.user_id, id).await?;
    Ok(Json(members))
}

async fn add_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<InviteMemberRequest>,
) -> Result<Json<TeamMemberResponse>, AppError> {
    let member = team_service::add_member(&state, auth_user.user_id, id, req).await?;
    Ok(Json(member))
}

async fn update_member_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<TeamMemberResponse>, AppError> {
    let member = team_service::update_member_role(&state, auth_user.user_id, id, user_id, req.role).await?;
    Ok(Json(member))
}

async fn remove_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    team_service::remove_member(&state, auth_user.user_id, id, user_id).await?;
    Ok(Json(serde_json::json!({"message": "Member removed"})))
}

async fn invite_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<InviteMemberRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let invite = team_service::invite_member(&state, auth_user.user_id, id, req).await?;
    Ok(Json(invite))
}

async fn accept_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(token): Path<String>,
) -> Result<Json<TeamResponse>, AppError> {
    let team = team_service::accept_invite(&state, auth_user.user_id, &token).await?;
    Ok(Json(team))
}
