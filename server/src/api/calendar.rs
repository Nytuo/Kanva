use axum::{
    extract::{Json, Path, Query, State},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, AppError};
use crate::middleware::auth::AuthUser;
use crate::services::calendar as calendar_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events).post(create_event))
        .route("/events/:id", get(get_event).put(update_event).delete(delete_event))
        .route("/board/:board_id", get(get_board_calendar))
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub board_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub all_day: Option<bool>,
    pub color: Option<String>,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub all_day: Option<bool>,
    pub color: Option<String>,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CalendarEventResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub all_day: bool,
    pub color: Option<String>,
    pub recurrence_rule: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_events(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<Vec<CalendarEventResponse>>, AppError> {
    let events = calendar_service::list_events(&state, auth_user.user_id, query).await?;
    Ok(Json(events))
}

async fn create_event(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateEventRequest>,
) -> Result<Json<CalendarEventResponse>, AppError> {
    let event = calendar_service::create_event(&state, auth_user.user_id, req).await?;
    Ok(Json(event))
}

async fn get_event(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarEventResponse>, AppError> {
    let event = calendar_service::get_event(&state, auth_user.user_id, id).await?;
    Ok(Json(event))
}

async fn update_event(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEventRequest>,
) -> Result<Json<CalendarEventResponse>, AppError> {
    let event = calendar_service::update_event(&state, auth_user.user_id, id, req).await?;
    Ok(Json(event))
}

async fn delete_event(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    calendar_service::delete_event(&state, auth_user.user_id, id).await?;
    Ok(Json(serde_json::json!({"message": "Event deleted"})))
}

#[derive(Debug, Deserialize)]
pub struct BoardCalendarQuery {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

async fn get_board_calendar(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(board_id): Path<Uuid>,
    Query(query): Query<BoardCalendarQuery>,
) -> Result<Json<Vec<CalendarEventResponse>>, AppError> {
    let events = calendar_service::get_board_calendar(&state, auth_user.user_id, board_id, query).await?;
    Ok(Json(events))
}
