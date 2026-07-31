use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::notes::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

// (id, owner_id, board_id, title, content, position, is_pinned, created_at, updated_at)
type NoteRow = (String, String, String, String, String, i32, i64, String, String);

fn row_to_response(row: NoteRow) -> Result<NoteResponse, AppError> {
    Ok(NoteResponse {
        id: parse_uuid(&row.0)?,
        owner_id: parse_uuid(&row.1)?,
        board_id: if row.2.is_empty() { None } else { Some(parse_uuid(&row.2)?) },
        title: row.3,
        content: row.4,
        position: row.5,
        is_pinned: row.6 != 0,
        created_at: parse_dt(&row.7)?,
        updated_at: parse_dt(&row.8)?,
    })
}

const SELECT_NOTE: &str = r#"SELECT id, owner_id, COALESCE(board_id,'') AS board_id, title, content, position, is_pinned, created_at, updated_at FROM notes"#;

/// Notes on a board are shared with everyone who can see that board; global
/// notes (board_id IS NULL) are private to their owner.
async fn check_note_access(state: &AppState, user_id: Uuid, board_id: &Option<Uuid>, owner_id: Uuid) -> Result<(), AppError> {
    match board_id {
        Some(bid) => crate::services::boards::check_board_access(state, user_id, *bid, "").await,
        None => {
            if owner_id == user_id {
                Ok(())
            } else {
                Err(AppError::Forbidden)
            }
        }
    }
}

pub async fn list_notes(state: &AppState, user_id: Uuid, board_id: Option<Uuid>) -> Result<Vec<NoteResponse>, AppError> {
    let rows = if let Some(bid) = board_id {
        crate::services::boards::check_board_access(state, user_id, bid, "").await?;
        sqlx::query_as::<_, NoteRow>(
            &state.q(&format!("{SELECT_NOTE} WHERE board_id = ? ORDER BY is_pinned DESC, position, updated_at DESC"))
        )
        .bind(bid.to_string())
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, NoteRow>(
            &state.q(&format!("{SELECT_NOTE} WHERE owner_id = ? AND board_id IS NULL ORDER BY is_pinned DESC, position, updated_at DESC"))
        )
        .bind(user_id.to_string())
        .fetch_all(&state.db)
        .await?
    };

    rows.into_iter().map(row_to_response).collect()
}

pub async fn create_note(state: &AppState, user_id: Uuid, req: CreateNoteRequest) -> Result<NoteResponse, AppError> {
    if let Some(bid) = req.board_id {
        crate::services::boards::check_board_access(state, user_id, bid, "").await?;
    }

    let position = if let Some(bid) = req.board_id {
        sqlx::query_scalar::<_, i32>(&state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM notes WHERE board_id = ?"))
            .bind(bid.to_string())
            .fetch_one(&state.db)
            .await?
    } else {
        sqlx::query_scalar::<_, i32>(&state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM notes WHERE owner_id = ? AND board_id IS NULL"))
            .bind(user_id.to_string())
            .fetch_one(&state.db)
            .await?
    };

    let id = Uuid::new_v4();
    let title = req.title.unwrap_or_else(|| "Untitled".to_string());
    let content = req.content.unwrap_or_default();

    sqlx::query(
        &state.q("INSERT INTO notes (id, owner_id, board_id, title, content, position) VALUES (?, ?, ?, ?, ?, ?)")
    )
    .bind(id.to_string())
    .bind(user_id.to_string())
    .bind(req.board_id.map(|b| b.to_string()))
    .bind(&title)
    .bind(&content)
    .bind(position)
    .execute(&state.db)
    .await?;

    get_note(state, user_id, id).await
}

pub async fn get_note(state: &AppState, user_id: Uuid, note_id: Uuid) -> Result<NoteResponse, AppError> {
    let row = sqlx::query_as::<_, NoteRow>(&state.q(&format!("{SELECT_NOTE} WHERE id = ?")))
        .bind(note_id.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Note not found".to_string()))?;

    let board_id = if row.2.is_empty() { None } else { Some(parse_uuid(&row.2)?) };
    let owner_id = parse_uuid(&row.1)?;
    check_note_access(state, user_id, &board_id, owner_id).await?;

    row_to_response(row)
}

pub async fn update_note(state: &AppState, user_id: Uuid, note_id: Uuid, req: UpdateNoteRequest) -> Result<NoteResponse, AppError> {
    let existing = sqlx::query_as::<_, NoteRow>(&state.q(&format!("{SELECT_NOTE} WHERE id = ?")))
        .bind(note_id.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Note not found".to_string()))?;

    let board_id = if existing.2.is_empty() { None } else { Some(parse_uuid(&existing.2)?) };
    let owner_id = parse_uuid(&existing.1)?;
    check_note_access(state, user_id, &board_id, owner_id).await?;

    let mut updates = vec!["updated_at = ?".to_string()];
    if req.title.is_some() { updates.push("title = ?".to_string()); }
    if req.content.is_some() { updates.push("content = ?".to_string()); }
    if req.position.is_some() { updates.push("position = ?".to_string()); }
    if req.is_pinned.is_some() { updates.push("is_pinned = ?".to_string()); }

    let query_raw = format!("UPDATE notes SET {} WHERE id = ?", updates.join(", "));
    let query = state.q(&query_raw);
    let mut q = sqlx::query(&query).bind(chrono::Utc::now().to_rfc3339());
    if let Some(ref title) = req.title { q = q.bind(title); }
    if let Some(ref content) = req.content { q = q.bind(content); }
    if let Some(position) = req.position { q = q.bind(position); }
    if let Some(pinned) = req.is_pinned { q = q.bind(if pinned { 1i64 } else { 0i64 }); }
    q.bind(note_id.to_string()).execute(&state.db).await?;

    get_note(state, user_id, note_id).await
}

pub async fn delete_note(state: &AppState, user_id: Uuid, note_id: Uuid) -> Result<(), AppError> {
    let existing = sqlx::query_as::<_, NoteRow>(&state.q(&format!("{SELECT_NOTE} WHERE id = ?")))
        .bind(note_id.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Note not found".to_string()))?;

    let board_id = if existing.2.is_empty() { None } else { Some(parse_uuid(&existing.2)?) };
    let owner_id = parse_uuid(&existing.1)?;
    check_note_access(state, user_id, &board_id, owner_id).await?;

    sqlx::query(&state.q("DELETE FROM notes WHERE id = ?"))
        .bind(note_id.to_string())
        .execute(&state.db)
        .await?;
    Ok(())
}
