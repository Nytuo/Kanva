use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::cards::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

/// Resolve the board a card lives on, erroring 404 if the card doesn't exist.
async fn resolve_card_board(state: &AppState, card_id: Uuid) -> Result<Uuid, AppError> {
    let board_id_str = sqlx::query_scalar::<_, String>(
        &state.q("SELECT l.board_id FROM cards c JOIN lists l ON c.list_id = l.id WHERE c.id = ?")
    )
    .bind(card_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Card not found".to_string()))?;
    parse_uuid(&board_id_str)
}

/// Every card mutation/read must go through this — cards themselves don't carry
/// membership info, only their board does. Without this check any authenticated
/// user could read/edit/delete any card on any (including private) board.
pub(crate) async fn check_card_access(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<(), AppError> {
    let board_id = resolve_card_board(state, card_id).await?;
    crate::services::boards::check_board_access(state, user_id, board_id, "").await
}

async fn check_list_access(state: &AppState, user_id: Uuid, list_id: Uuid) -> Result<(), AppError> {
    let board_id_str = sqlx::query_scalar::<_, String>(&state.q("SELECT board_id FROM lists WHERE id = ?"))
        .bind(list_id.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("List not found".to_string()))?;
    let board_id = parse_uuid(&board_id_str)?;
    crate::services::boards::check_board_access(state, user_id, board_id, "").await
}

pub async fn create_card(
    state: &AppState,
    user_id: Uuid,
    req: CreateCardRequest,
) -> Result<CardResponse, AppError> {
    check_list_access(state, user_id, req.list_id).await?;

    let position = if let Some(pos) = req.position {
        pos
    } else {
        sqlx::query_scalar::<_, i32>(
            &state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM cards WHERE list_id = ?")
        )
        .bind(req.list_id.to_string())
        .fetch_one(&state.db)
        .await?
    };

    let priority = req.priority.as_deref().unwrap_or("none");
    let card_id = Uuid::new_v4();

    sqlx::query(
        &state.q(r#"INSERT INTO cards (id, list_id, title, description, position, priority, due_date, start_date, cover_color, estimated_hours, created_by)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
    )
    .bind(card_id.to_string())
    .bind(req.list_id.to_string())
    .bind(&req.title)
    .bind(&req.description)
    .bind(position)
    .bind(priority)
    .bind(req.due_date.map(|d| d.to_rfc3339()))
    .bind(req.start_date.map(|d| d.to_rfc3339()))
    .bind(&req.cover_color)
    .bind(req.estimated_hours.map(|h| h.to_string()))
    .bind(user_id.to_string())
    .execute(&state.db)
    .await?;

    if let Some(assignee_ids) = &req.assignee_ids {
        for assignee_id in assignee_ids {
            sqlx::query(&state.q("INSERT INTO card_assignees (card_id, user_id) VALUES (?, ?) ON CONFLICT DO NOTHING"))
                .bind(card_id.to_string())
                .bind(assignee_id.to_string())
                .execute(&state.db)
                .await?;
        }
    }

    if let Some(label_ids) = &req.label_ids {
        for label_id in label_ids {
            sqlx::query(&state.q("INSERT INTO card_labels (card_id, label_id) VALUES (?, ?) ON CONFLICT DO NOTHING"))
                .bind(card_id.to_string())
                .bind(label_id.to_string())
                .execute(&state.db)
                .await?;
        }
    }

    let board_id_str = sqlx::query_scalar::<_, String>(
        &state.q("SELECT board_id FROM lists WHERE id = ?")
    )
    .bind(req.list_id.to_string())
    .fetch_one(&state.db)
    .await?;

    let metadata_json = serde_json::to_string(&serde_json::json!({"card_title": req.title})).unwrap_or_default();
    sqlx::query(
        &state.q("INSERT INTO activity_log (id, board_id, card_id, user_id, action, metadata) VALUES (?, ?, ?, ?, 'card_created', ?)")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&board_id_str)
    .bind(card_id.to_string())
    .bind(user_id.to_string())
    .bind(&metadata_json)
    .execute(&state.db)
    .await?;

    get_card(state, user_id, card_id).await
}

pub async fn get_card(
    state: &AppState,
    user_id: Uuid,
    card_id: Uuid,
) -> Result<CardResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    // (id, list_id, title, description, position, priority, due_date, start_date,
    //  completed_at, is_archived, cover_color, cover_image_url, created_by, created_at, updated_at)
    let card = sqlx::query_as::<_, (String, String, String, String, i32, String, String, String, String, i64, String, String, String, String, String)>(
        &state.q(r#"SELECT id, list_id, title, COALESCE(description,'') AS description, position, priority,
                   COALESCE(due_date,'') AS due_date, COALESCE(start_date,'') AS start_date,
                   COALESCE(completed_at,'') AS completed_at, is_archived,
                   COALESCE(cover_color,'') AS cover_color, COALESCE(cover_image_url,'') AS cover_image_url,
                   created_by, created_at, updated_at
           FROM cards WHERE id = ?"#)
    )
    .bind(card_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Card not found".to_string()))?;

    let assignees = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q(r#"SELECT u.id, u.username, u.display_name, COALESCE(u.avatar_url,'') AS avatar_url
           FROM card_assignees ca JOIN users u ON ca.user_id = u.id WHERE ca.card_id = ?"#)
    )
    .bind(card_id.to_string())
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|a| Ok(AssigneeResponse {
        user_id: parse_uuid(&a.0)?, username: a.1, display_name: a.2,
        avatar_url: crate::models::user::empty_to_none(a.3),
    }))
    .collect::<Result<Vec<_>, AppError>>()?;

    let labels = sqlx::query_as::<_, (String, String, String)>(
        &state.q("SELECT l.id, l.name, l.color FROM card_labels cl JOIN labels l ON cl.label_id = l.id WHERE cl.card_id = ?")
    )
    .bind(card_id.to_string())
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|l| Ok(crate::api::boards::LabelResponse { id: parse_uuid(&l.0)?, name: l.1, color: l.2 }))
    .collect::<Result<Vec<_>, AppError>>()?;

    // (id, title, position)
    let checklists_raw = sqlx::query_as::<_, (String, String, i32)>(
        &state.q("SELECT id, title, position FROM checklists WHERE card_id = ? ORDER BY position")
    )
    .bind(card_id.to_string())
    .fetch_all(&state.db)
    .await?;

    let mut checklists = Vec::new();
    for cl in checklists_raw {
        let cl_id_str = cl.0.clone();
        // (id, title, is_checked, position, assigned_to, due_date)
        let items = sqlx::query_as::<_, (String, String, i64, i32, String, String)>(
            &state.q("SELECT id, title, is_checked, position, COALESCE(assigned_to,'') AS assigned_to, COALESCE(due_date,'') AS due_date FROM checklist_items WHERE checklist_id = ? ORDER BY position")
        )
        .bind(&cl_id_str)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|i| Ok(ChecklistItemResponse {
            id: parse_uuid(&i.0)?,
            title: i.1,
            is_checked: i.2 != 0,
            position: i.3,
            assigned_to: if i.4.is_empty() { None } else { Some(parse_uuid(&i.4)?) },
            due_date: if i.5.is_empty() { None } else { parse_dt(&i.5).map(Some)? },
        }))
        .collect::<Result<Vec<_>, AppError>>()?;

        checklists.push(ChecklistResponse { id: parse_uuid(&cl.0)?, title: cl.1, position: cl.2, items });
    }

    // (id, user_id, username, display_name, avatar_url, content, edited_at, created_at)
    let comments = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
        &state.q(r#"SELECT c.id, c.user_id, u.username, u.display_name,
               COALESCE(u.avatar_url,'') AS avatar_url, c.content,
               COALESCE(c.edited_at,'') AS edited_at, c.created_at
           FROM comments c JOIN users u ON c.user_id = u.id WHERE c.card_id = ? ORDER BY c.created_at"#)
    )
    .bind(card_id.to_string())
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|c| Ok(CommentResponse {
        id: parse_uuid(&c.0)?,
        user_id: parse_uuid(&c.1)?,
        username: c.2,
        display_name: c.3,
        avatar_url: crate::models::user::empty_to_none(c.4),
        content: c.5,
        edited_at: if c.6.is_empty() { None } else { parse_dt(&c.6).map(Some)? },
        created_at: parse_dt(&c.7)?,
    }))
    .collect::<Result<Vec<_>, AppError>>()?;

    // (id, filename, file_url, file_size, mime_type, created_at)
    let attachments = sqlx::query_as::<_, (String, String, String, i64, String, String)>(
        &state.q("SELECT id, filename, file_url, file_size, COALESCE(mime_type,'') AS mime_type, created_at FROM attachments WHERE card_id = ?")
    )
    .bind(card_id.to_string())
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|a| Ok(AttachmentResponse {
        id: parse_uuid(&a.0)?,
        filename: a.1,
        file_url: a.2,
        file_size: a.3,
        mime_type: crate::models::user::empty_to_none(a.4),
        created_at: parse_dt(&a.5)?,
    }))
    .collect::<Result<Vec<_>, AppError>>()?;

    Ok(CardResponse {
        id: parse_uuid(&card.0)?,
        list_id: parse_uuid(&card.1)?,
        title: card.2,
        description: crate::models::user::empty_to_none(card.3),
        position: card.4,
        priority: card.5,
        due_date: if card.6.is_empty() { None } else { parse_dt(&card.6).map(Some)? },
        start_date: if card.7.is_empty() { None } else { parse_dt(&card.7).map(Some)? },
        completed_at: if card.8.is_empty() { None } else { parse_dt(&card.8).map(Some)? },
        is_archived: card.9 != 0,
        cover_color: crate::models::user::empty_to_none(card.10),
        cover_image_url: crate::models::user::empty_to_none(card.11),
        estimated_hours: None,
        actual_hours: None,
        created_by: parse_uuid(&card.12)?,
        assignees,
        labels,
        checklists,
        comments,
        attachments,
        custom_field_values: vec![],
        created_at: parse_dt(&card.13)?,
        updated_at: parse_dt(&card.14)?,
    })
}

pub async fn update_card(
    state: &AppState,
    user_id: Uuid,
    card_id: Uuid,
    req: UpdateCardRequest,
) -> Result<CardResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    if let Some(title) = &req.title {
        sqlx::query(&state.q("UPDATE cards SET title = ? WHERE id = ?")).bind(title).bind(card_id.to_string()).execute(&state.db).await?;
    }
    if let Some(desc) = &req.description {
        sqlx::query(&state.q("UPDATE cards SET description = ? WHERE id = ?")).bind(desc).bind(card_id.to_string()).execute(&state.db).await?;
    }
    if let Some(priority) = &req.priority {
        sqlx::query(&state.q("UPDATE cards SET priority = ? WHERE id = ?")).bind(priority).bind(card_id.to_string()).execute(&state.db).await?;
    }
    if let Some(due) = &req.due_date {
        if due.is_empty() {
            sqlx::query(&state.q("UPDATE cards SET due_date = '' WHERE id = ?")).bind(card_id.to_string()).execute(&state.db).await?;
        } else {
            sqlx::query(&state.q("UPDATE cards SET due_date = ? WHERE id = ?")).bind(due).bind(card_id.to_string()).execute(&state.db).await?;
        }
    }
    if let Some(start) = &req.start_date {
        if start.is_empty() {
            sqlx::query(&state.q("UPDATE cards SET start_date = '' WHERE id = ?")).bind(card_id.to_string()).execute(&state.db).await?;
        } else {
            sqlx::query(&state.q("UPDATE cards SET start_date = ? WHERE id = ?")).bind(start).bind(card_id.to_string()).execute(&state.db).await?;
        }
    }
    if let Some(color) = &req.cover_color {
        sqlx::query(&state.q("UPDATE cards SET cover_color = ? WHERE id = ?")).bind(color).bind(card_id.to_string()).execute(&state.db).await?;
    }

    get_card(state, user_id, card_id).await
}

pub async fn delete_card(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("DELETE FROM cards WHERE id = ?")).bind(card_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn move_card(
    state: &AppState,
    user_id: Uuid,
    card_id: Uuid,
    req: MoveCardRequest,
) -> Result<CardResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;
    // Destination list may be on a different board — check that one too.
    check_list_access(state, user_id, req.list_id).await?;

    // (list_id, position)
    let card = sqlx::query_as::<_, (String, i32)>(
        &state.q("SELECT list_id, position FROM cards WHERE id = ?")
    )
    .bind(card_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Card not found".to_string()))?;

    let old_list_id = card.0;

    sqlx::query(&state.q("UPDATE cards SET position = position - 1 WHERE list_id = ? AND position > ?"))
        .bind(&old_list_id)
        .bind(card.1)
        .execute(&state.db)
        .await?;

    sqlx::query(&state.q("UPDATE cards SET position = position + 1 WHERE list_id = ? AND position >= ?"))
        .bind(req.list_id.to_string())
        .bind(req.position)
        .execute(&state.db)
        .await?;

    sqlx::query(&state.q("UPDATE cards SET list_id = ?, position = ? WHERE id = ?"))
        .bind(req.list_id.to_string())
        .bind(req.position)
        .bind(card_id.to_string())
        .execute(&state.db)
        .await?;

    let board_id_str = sqlx::query_scalar::<_, String>(&state.q("SELECT board_id FROM lists WHERE id = ?"))
        .bind(req.list_id.to_string())
        .fetch_one(&state.db)
        .await?;

    sqlx::query(&state.q("INSERT INTO activity_log (id, board_id, card_id, user_id, action) VALUES (?, ?, ?, ?, 'card_moved')"))
        .bind(Uuid::new_v4().to_string())
        .bind(&board_id_str)
        .bind(card_id.to_string())
        .bind(user_id.to_string())
        .execute(&state.db)
        .await?;

    get_card(state, user_id, card_id).await
}

pub async fn assign_card(state: &AppState, user_id: Uuid, card_id: Uuid, assignee_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("INSERT INTO card_assignees (card_id, user_id) VALUES (?, ?) ON CONFLICT DO NOTHING"))
        .bind(card_id.to_string()).bind(assignee_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn unassign_card(state: &AppState, user_id: Uuid, card_id: Uuid, assignee_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("DELETE FROM card_assignees WHERE card_id = ? AND user_id = ?"))
        .bind(card_id.to_string()).bind(assignee_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn add_label(state: &AppState, user_id: Uuid, card_id: Uuid, label_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("INSERT INTO card_labels (card_id, label_id) VALUES (?, ?) ON CONFLICT DO NOTHING"))
        .bind(card_id.to_string()).bind(label_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn remove_label(state: &AppState, user_id: Uuid, card_id: Uuid, label_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("DELETE FROM card_labels WHERE card_id = ? AND label_id = ?"))
        .bind(card_id.to_string()).bind(label_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn list_comments(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<Vec<CommentResponse>, AppError> {
    check_card_access(state, user_id, card_id).await?;
    let comments = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
        &state.q(r#"SELECT c.id, c.user_id, u.username, u.display_name,
               COALESCE(u.avatar_url,'') AS avatar_url, c.content,
               COALESCE(c.edited_at,'') AS edited_at, c.created_at
           FROM comments c JOIN users u ON c.user_id = u.id WHERE c.card_id = ? ORDER BY c.created_at"#)
    )
    .bind(card_id.to_string())
    .fetch_all(&state.db)
    .await?;

    comments.into_iter().map(|c| Ok(CommentResponse {
        id: parse_uuid(&c.0)?,
        user_id: parse_uuid(&c.1)?,
        username: c.2,
        display_name: c.3,
        avatar_url: crate::models::user::empty_to_none(c.4),
        content: c.5,
        edited_at: if c.6.is_empty() { None } else { parse_dt(&c.6).map(Some)? },
        created_at: parse_dt(&c.7)?,
    })).collect()
}

pub async fn create_comment(state: &AppState, user_id: Uuid, card_id: Uuid, content: String) -> Result<CommentResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let comment_id = Uuid::new_v4();
    sqlx::query(
        &state.q("INSERT INTO comments (id, card_id, user_id, content) VALUES (?, ?, ?, ?)")
    )
    .bind(comment_id.to_string()).bind(card_id.to_string()).bind(user_id.to_string()).bind(&content)
    .execute(&state.db).await?;

    let user = sqlx::query_as::<_, (String, String, String)>(
        &state.q("SELECT username, display_name, COALESCE(avatar_url,'') AS avatar_url FROM users WHERE id = ?")
    )
    .bind(user_id.to_string()).fetch_one(&state.db).await?;

    Ok(CommentResponse {
        id: comment_id, user_id, username: user.0, display_name: user.1,
        avatar_url: crate::models::user::empty_to_none(user.2),
        content, edited_at: None, created_at: chrono::Utc::now(),
    })
}

pub async fn update_comment(state: &AppState, user_id: Uuid, card_id: Uuid, comment_id: Uuid, content: String) -> Result<CommentResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    // Only the comment's author may edit it, even if others can see the card.
    let updated = sqlx::query(&state.q("UPDATE comments SET content = ?, edited_at = CURRENT_TIMESTAMP WHERE id = ? AND card_id = ? AND user_id = ?"))
        .bind(&content).bind(comment_id.to_string()).bind(card_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::Forbidden);
    }

    let user = sqlx::query_as::<_, (String, String, String)>(
        &state.q("SELECT username, display_name, COALESCE(avatar_url,'') AS avatar_url FROM users WHERE id = ?")
    )
    .bind(user_id.to_string()).fetch_one(&state.db).await?;

    Ok(CommentResponse {
        id: comment_id, user_id, username: user.0, display_name: user.1,
        avatar_url: crate::models::user::empty_to_none(user.2),
        content, edited_at: Some(chrono::Utc::now()), created_at: chrono::Utc::now(),
    })
}

pub async fn delete_comment(state: &AppState, user_id: Uuid, card_id: Uuid, comment_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;

    let deleted = sqlx::query(&state.q("DELETE FROM comments WHERE id = ? AND card_id = ? AND user_id = ?"))
        .bind(comment_id.to_string()).bind(card_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    if deleted.rows_affected() == 0 {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub async fn list_checklists(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<Vec<ChecklistResponse>, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let cls = sqlx::query_as::<_, (String, String, i32)>(
        &state.q("SELECT id, title, position FROM checklists WHERE card_id = ? ORDER BY position")
    )
    .bind(card_id.to_string()).fetch_all(&state.db).await?;

    let mut result = Vec::new();
    for cl in cls {
        let cl_id_str = cl.0.clone();
        let items = sqlx::query_as::<_, (String, String, i64, i32, String, String)>(
            &state.q("SELECT id, title, is_checked, position, COALESCE(assigned_to,'') AS assigned_to, COALESCE(due_date,'') AS due_date FROM checklist_items WHERE checklist_id = ? ORDER BY position")
        )
        .bind(&cl_id_str).fetch_all(&state.db).await?
        .into_iter()
        .map(|i| Ok(ChecklistItemResponse {
            id: parse_uuid(&i.0)?,
            title: i.1,
            is_checked: i.2 != 0,
            position: i.3,
            assigned_to: if i.4.is_empty() { None } else { Some(parse_uuid(&i.4)?) },
            due_date: if i.5.is_empty() { None } else { parse_dt(&i.5).map(Some)? },
        }))
        .collect::<Result<Vec<_>, AppError>>()?;
        result.push(ChecklistResponse { id: parse_uuid(&cl.0)?, title: cl.1, position: cl.2, items });
    }
    Ok(result)
}

pub async fn create_checklist(state: &AppState, user_id: Uuid, card_id: Uuid, title: String) -> Result<ChecklistResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let pos = sqlx::query_scalar::<_, i32>(
        &state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM checklists WHERE card_id = ?")
    )
    .bind(card_id.to_string()).fetch_one(&state.db).await?;

    let id = Uuid::new_v4();
    sqlx::query(
        &state.q("INSERT INTO checklists (id, card_id, title, position) VALUES (?, ?, ?, ?)")
    )
    .bind(id.to_string()).bind(card_id.to_string()).bind(&title).bind(pos).execute(&state.db).await?;

    Ok(ChecklistResponse { id, title, position: pos, items: vec![] })
}

pub async fn update_checklist(state: &AppState, user_id: Uuid, card_id: Uuid, checklist_id: Uuid, title: String) -> Result<ChecklistResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let updated = sqlx::query(&state.q("UPDATE checklists SET title = ? WHERE id = ? AND card_id = ?"))
        .bind(&title).bind(checklist_id.to_string()).bind(card_id.to_string()).execute(&state.db).await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("Checklist not found".to_string()));
    }

    let items = sqlx::query_as::<_, (String, String, i64, i32, String, String)>(
        &state.q("SELECT id, title, is_checked, position, COALESCE(assigned_to,'') AS assigned_to, COALESCE(due_date,'') AS due_date FROM checklist_items WHERE checklist_id = ? ORDER BY position")
    )
    .bind(checklist_id.to_string()).fetch_all(&state.db).await?
    .into_iter()
    .map(|i| Ok(ChecklistItemResponse {
        id: parse_uuid(&i.0)?,
        title: i.1,
        is_checked: i.2 != 0,
        position: i.3,
        assigned_to: if i.4.is_empty() { None } else { Some(parse_uuid(&i.4)?) },
        due_date: if i.5.is_empty() { None } else { parse_dt(&i.5).map(Some)? },
    }))
    .collect::<Result<Vec<_>, AppError>>()?;
    Ok(ChecklistResponse { id: checklist_id, title, position: 0, items })
}

pub async fn delete_checklist(state: &AppState, user_id: Uuid, card_id: Uuid, checklist_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("DELETE FROM checklists WHERE id = ? AND card_id = ?"))
        .bind(checklist_id.to_string()).bind(card_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn create_checklist_item(state: &AppState, user_id: Uuid, card_id: Uuid, checklist_id: Uuid, req: CreateChecklistItemRequest) -> Result<ChecklistItemResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    // Make sure the checklist actually belongs to this card.
    sqlx::query_scalar::<_, String>(&state.q("SELECT id FROM checklists WHERE id = ? AND card_id = ?"))
        .bind(checklist_id.to_string()).bind(card_id.to_string())
        .fetch_optional(&state.db).await?
        .ok_or(AppError::NotFound("Checklist not found".to_string()))?;

    let pos = sqlx::query_scalar::<_, i32>(
        &state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM checklist_items WHERE checklist_id = ?")
    )
    .bind(checklist_id.to_string()).fetch_one(&state.db).await?;

    let id = Uuid::new_v4();
    sqlx::query(
        &state.q("INSERT INTO checklist_items (id, checklist_id, title, position, assigned_to, due_date) VALUES (?, ?, ?, ?, ?, ?)")
    )
    .bind(id.to_string())
    .bind(checklist_id.to_string())
    .bind(&req.title)
    .bind(pos)
    .bind(req.assigned_to.map(|u| u.to_string()))
    .bind(req.due_date.map(|d| d.to_rfc3339()))
    .execute(&state.db).await?;

    Ok(ChecklistItemResponse {
        id,
        title: req.title,
        is_checked: false,
        position: pos,
        assigned_to: req.assigned_to,
        due_date: req.due_date,
    })
}

pub async fn update_checklist_item(state: &AppState, user_id: Uuid, card_id: Uuid, checklist_id: Uuid, item_id: Uuid, req: UpdateChecklistItemRequest) -> Result<ChecklistItemResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    sqlx::query_scalar::<_, String>(&state.q("SELECT id FROM checklists WHERE id = ? AND card_id = ?"))
        .bind(checklist_id.to_string()).bind(card_id.to_string())
        .fetch_optional(&state.db).await?
        .ok_or(AppError::NotFound("Checklist not found".to_string()))?;

    if let Some(title) = &req.title {
        sqlx::query(&state.q("UPDATE checklist_items SET title = ? WHERE id = ? AND checklist_id = ?")).bind(title).bind(item_id.to_string()).bind(checklist_id.to_string()).execute(&state.db).await?;
    }
    if let Some(checked) = req.is_checked {
        sqlx::query(&state.q("UPDATE checklist_items SET is_checked = ? WHERE id = ? AND checklist_id = ?")).bind(if checked { 1i64 } else { 0i64 }).bind(item_id.to_string()).bind(checklist_id.to_string()).execute(&state.db).await?;
    }

    let item = sqlx::query_as::<_, (String, String, i64, i32, String, String)>(
        &state.q("SELECT id, title, is_checked, position, COALESCE(assigned_to,'') AS assigned_to, COALESCE(due_date,'') AS due_date FROM checklist_items WHERE id = ? AND checklist_id = ?")
    )
    .bind(item_id.to_string()).bind(checklist_id.to_string()).fetch_optional(&state.db).await?
    .ok_or(AppError::NotFound("Checklist item not found".to_string()))?;

    Ok(ChecklistItemResponse {
        id: parse_uuid(&item.0)?,
        title: item.1,
        is_checked: item.2 != 0,
        position: item.3,
        assigned_to: if item.4.is_empty() { None } else { Some(parse_uuid(&item.4)?) },
        due_date: if item.5.is_empty() { None } else { parse_dt(&item.5).map(Some)? },
    })
}

pub async fn delete_checklist_item(state: &AppState, user_id: Uuid, card_id: Uuid, checklist_id: Uuid, item_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;

    sqlx::query_scalar::<_, String>(&state.q("SELECT id FROM checklists WHERE id = ? AND card_id = ?"))
        .bind(checklist_id.to_string()).bind(card_id.to_string())
        .fetch_optional(&state.db).await?
        .ok_or(AppError::NotFound("Checklist not found".to_string()))?;

    sqlx::query(&state.q("DELETE FROM checklist_items WHERE id = ? AND checklist_id = ?"))
        .bind(item_id.to_string()).bind(checklist_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn list_attachments(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<Vec<AttachmentResponse>, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let attachments = sqlx::query_as::<_, (String, String, String, i64, String, String)>(
        &state.q("SELECT id, filename, file_url, file_size, COALESCE(mime_type,'') AS mime_type, created_at FROM attachments WHERE card_id = ?")
    )
    .bind(card_id.to_string()).fetch_all(&state.db).await?;

    attachments.into_iter().map(|a| Ok(AttachmentResponse {
        id: parse_uuid(&a.0)?,
        filename: a.1,
        file_url: a.2,
        file_size: a.3,
        mime_type: crate::models::user::empty_to_none(a.4),
        created_at: parse_dt(&a.5)?,
    })).collect()
}

/// Keep only the filename component and strip anything that could be used for
/// path traversal (`/`, `\`, `..`) or a hidden/absolute path. The stored file is
/// always prefixed with a fresh UUID, so collisions aren't a concern — this is
/// purely to stop a crafted multipart filename from escaping `upload_dir`.
fn sanitize_filename(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = base
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let cleaned = cleaned.trim_start_matches('.').trim();
    if cleaned.is_empty() {
        "file".to_string()
    } else {
        cleaned.chars().take(255).collect()
    }
}

pub async fn upload_attachment(
    state: &AppState,
    user_id: Uuid,
    card_id: Uuid,
    mut multipart: axum::extract::Multipart,
) -> Result<AttachmentResponse, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let max_bytes = state.config.max_upload_size_mb * 1024 * 1024;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let filename = sanitize_filename(field.file_name().unwrap_or("file"));
        let mime_type = field.content_type().map(String::from);
        let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
        let file_size = data.len() as i64;

        if data.len() > max_bytes {
            return Err(AppError::BadRequest(format!(
                "File exceeds the {} MB upload limit",
                state.config.max_upload_size_mb
            )));
        }

        let upload_dir = &state.config.upload_dir;
        tokio::fs::create_dir_all(upload_dir).await.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let file_id = Uuid::new_v4();
        let file_path = format!("{}/{}-{}", upload_dir, file_id, filename);
        tokio::fs::write(&file_path, &data).await.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        let file_url = format!("/uploads/{}-{}", file_id, filename);
        let attachment_id = Uuid::new_v4();

        sqlx::query(
            &state.q("INSERT INTO attachments (id, card_id, user_id, filename, file_url, file_size, mime_type) VALUES (?, ?, ?, ?, ?, ?, ?)")
        )
        .bind(attachment_id.to_string()).bind(card_id.to_string()).bind(user_id.to_string())
        .bind(&filename).bind(&file_url).bind(file_size).bind(&mime_type)
        .execute(&state.db).await?;

        return Ok(AttachmentResponse {
            id: attachment_id, filename, file_url, file_size, mime_type, created_at: chrono::Utc::now(),
        });
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}

pub async fn delete_attachment(state: &AppState, user_id: Uuid, card_id: Uuid, attachment_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("DELETE FROM attachments WHERE id = ? AND card_id = ?"))
        .bind(attachment_id.to_string()).bind(card_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn get_custom_field_values(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let values = sqlx::query_as::<_, (String, String, String)>(
        &state.q("SELECT id, field_id, value FROM card_custom_field_values WHERE card_id = ?")
    )
    .bind(card_id.to_string()).fetch_all(&state.db).await?;

    Ok(values.into_iter().map(|v| {
        let value: serde_json::Value = serde_json::from_str(&v.2).unwrap_or(serde_json::Value::String(v.2.clone()));
        serde_json::json!({
            "id": v.0, "field_id": v.1, "value": value,
        })
    }).collect())
}

pub async fn set_custom_field_value(state: &AppState, user_id: Uuid, card_id: Uuid, req: SetCustomFieldValueRequest) -> Result<serde_json::Value, AppError> {
    check_card_access(state, user_id, card_id).await?;

    let value_json = serde_json::to_string(&req.value).unwrap_or_default();
    sqlx::query(
        &state.q(r#"INSERT INTO card_custom_field_values (id, card_id, field_id, value) VALUES (?, ?, ?, ?)
           ON CONFLICT (card_id, field_id) DO UPDATE SET value = excluded.value"#)
    )
    .bind(Uuid::new_v4().to_string()).bind(card_id.to_string()).bind(req.field_id.to_string()).bind(&value_json)
    .execute(&state.db).await?;

    Ok(serde_json::json!({"card_id": card_id, "field_id": req.field_id, "value": req.value}))
}

pub async fn archive_card(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(&state.q("UPDATE cards SET is_archived = CASE WHEN is_archived = 1 THEN 0 ELSE 1 END WHERE id = ?"))
        .bind(card_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn complete_card(state: &AppState, user_id: Uuid, card_id: Uuid) -> Result<(), AppError> {
    check_card_access(state, user_id, card_id).await?;
    sqlx::query(
        &state.q("UPDATE cards SET completed_at = CASE WHEN completed_at IS NULL THEN CURRENT_TIMESTAMP ELSE NULL END WHERE id = ?")
    )
    .bind(card_id.to_string()).execute(&state.db).await?;
    Ok(())
}
