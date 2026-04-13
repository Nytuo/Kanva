use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::lists::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

fn list_to_response(list: &crate::models::board::List, card_count: i64) -> Result<ListResponse, AppError> {
    Ok(ListResponse {
        id: parse_uuid(&list.id)?,
        board_id: parse_uuid(&list.board_id)?,
        title: list.title.clone(),
        position: list.position,
        is_archived: list.is_archived != 0,
        card_count,
        created_at: parse_dt(&list.created_at)?,
        updated_at: parse_dt(&list.updated_at)?,
    })
}

pub async fn create_list(
    state: &AppState,
    user_id: Uuid,
    req: CreateListRequest,
) -> Result<ListResponse, AppError> {
    crate::services::boards::get_board(state, user_id, req.board_id).await?;

    let position = if let Some(pos) = req.position {
        pos
    } else {
        sqlx::query_scalar::<_, i32>(
            &state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM lists WHERE board_id = ?")
        )
        .bind(req.board_id.to_string())
        .fetch_one(&state.db)
        .await?
    };

    let list_id = Uuid::new_v4();
    sqlx::query(
        &state.q("INSERT INTO lists (id, board_id, title, position) VALUES (?, ?, ?, ?)")
    )
    .bind(list_id.to_string())
    .bind(req.board_id.to_string())
    .bind(&req.title)
    .bind(position)
    .execute(&state.db)
    .await?;

    let list = sqlx::query_as::<_, crate::models::board::List>(
        &state.q("SELECT id, board_id, title, position, is_archived, created_at, updated_at FROM lists WHERE id = ?")
    )
    .bind(list_id.to_string())
    .fetch_one(&state.db)
    .await?;

    let metadata_json = serde_json::to_string(&serde_json::json!({"list_title": req.title})).unwrap_or_default();
    sqlx::query(
        &state.q("INSERT INTO activity_log (id, board_id, user_id, action, metadata) VALUES (?, ?, ?, 'list_created', ?)")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(req.board_id.to_string())
    .bind(user_id.to_string())
    .bind(&metadata_json)
    .execute(&state.db)
    .await?;

    list_to_response(&list, 0)
}

pub async fn update_list(
    state: &AppState,
    _user_id: Uuid,
    list_id: Uuid,
    req: UpdateListRequest,
) -> Result<ListResponse, AppError> {
    if let Some(title) = &req.title {
        sqlx::query(&state.q("UPDATE lists SET title = ? WHERE id = ?"))
            .bind(title)
            .bind(list_id.to_string())
            .execute(&state.db)
            .await?;
    }

    let list = sqlx::query_as::<_, crate::models::board::List>(
        &state.q("SELECT id, board_id, title, position, is_archived, created_at, updated_at FROM lists WHERE id = ?")
    )
    .bind(list_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("List not found".to_string()))?;

    let card_count = sqlx::query_scalar::<_, i64>(
        &state.q("SELECT COUNT(*) FROM cards WHERE list_id = ?")
    )
    .bind(list_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    list_to_response(&list, card_count)
}

pub async fn delete_list(state: &AppState, _user_id: Uuid, list_id: Uuid) -> Result<(), AppError> {
    sqlx::query(&state.q("DELETE FROM lists WHERE id = ?"))
        .bind(list_id.to_string())
        .execute(&state.db)
        .await?;
    Ok(())
}

pub async fn move_list(
    state: &AppState,
    _user_id: Uuid,
    list_id: Uuid,
    new_position: i32,
) -> Result<ListResponse, AppError> {
    let list = sqlx::query_as::<_, crate::models::board::List>(
        &state.q("SELECT id, board_id, title, position, is_archived, created_at, updated_at FROM lists WHERE id = ?")
    )
    .bind(list_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("List not found".to_string()))?;

    let old_position = list.position;
    let board_id_str = list.board_id.clone();

    if new_position > old_position {
        sqlx::query(
            &state.q("UPDATE lists SET position = position - 1 WHERE board_id = ? AND position > ? AND position <= ?")
        )
        .bind(&board_id_str)
        .bind(old_position)
        .bind(new_position)
        .execute(&state.db)
        .await?;
    } else if new_position < old_position {
        sqlx::query(
            &state.q("UPDATE lists SET position = position + 1 WHERE board_id = ? AND position >= ? AND position < ?")
        )
        .bind(&board_id_str)
        .bind(new_position)
        .bind(old_position)
        .execute(&state.db)
        .await?;
    }

    sqlx::query(&state.q("UPDATE lists SET position = ? WHERE id = ?"))
        .bind(new_position)
        .bind(list_id.to_string())
        .execute(&state.db)
        .await?;

    let updated = sqlx::query_as::<_, crate::models::board::List>(
        &state.q("SELECT id, board_id, title, position, is_archived, created_at, updated_at FROM lists WHERE id = ?")
    )
    .bind(list_id.to_string())
    .fetch_one(&state.db)
    .await?;

    let card_count = sqlx::query_scalar::<_, i64>(
        &state.q("SELECT COUNT(*) FROM cards WHERE list_id = ?")
    )
    .bind(list_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    list_to_response(&updated, card_count)
}

pub async fn archive_list(state: &AppState, _user_id: Uuid, list_id: Uuid) -> Result<(), AppError> {
    sqlx::query(&state.q("UPDATE lists SET is_archived = CASE WHEN is_archived = 1 THEN 0 ELSE 1 END WHERE id = ?"))
        .bind(list_id.to_string())
        .execute(&state.db)
        .await?;
    Ok(())
}
