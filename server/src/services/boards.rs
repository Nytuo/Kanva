use uuid::Uuid;

use crate::{AppError, AppState};
use crate::api::boards::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

pub async fn list_boards(
    state: &AppState,
    user_id: Uuid,
    query: ListBoardsQuery,
) -> Result<Vec<BoardSummaryResponse>, AppError> {
    let uid = user_id.to_string();
    let mut sql = String::from(
        r#"SELECT b.id, b.title, COALESCE(b.description,'') AS description, b.visibility,
                  COALESCE(b.background_color,'') AS background_color,
                  COALESCE(b.background_image_url,'') AS background_image_url,
                  b.is_starred, b.is_archived, b.owner_id, COALESCE(b.team_id,'') AS team_id, b.created_at
           FROM boards b
           LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = ?
           LEFT JOIN team_members tm ON b.team_id = tm.team_id AND tm.user_id = ?
           WHERE (b.owner_id = ? OR bm.user_id IS NOT NULL OR b.visibility = 'public'
                  OR (b.visibility = 'team' AND tm.user_id IS NOT NULL))"#
    );

    if query.team_id.is_some() {
        sql.push_str(" AND b.team_id = ?");
    }
    if query.archived.is_some() {
        sql.push_str(" AND b.is_archived = ?");
    }
    if query.starred.is_some() {
        sql.push_str(" AND b.is_starred = ?");
    }
    sql.push_str(" ORDER BY b.is_starred DESC, b.updated_at DESC");
    let sql = state.q(&sql);

    let mut q = sqlx::query_as::<_, (String, String, String, String, String, String, i64, i64, String, String, String)>(&sql);
    q = q.bind(&uid).bind(&uid).bind(&uid);
    if let Some(tid) = query.team_id { q = q.bind(tid.to_string()); }
    if let Some(arch) = query.archived { q = q.bind(if arch { 1i64 } else { 0i64 }); }
    if let Some(star) = query.starred { q = q.bind(if star { 1i64 } else { 0i64 }); }

    let boards = q.fetch_all(&state.db).await?;

    let mut results = Vec::new();
    for b in boards {
        let board_id_str = b.0.clone();
        let member_count = sqlx::query_scalar::<_, i64>(
            &state.q("SELECT COUNT(*) FROM board_members WHERE board_id = ?")
        )
        .bind(&board_id_str)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

        let card_count = sqlx::query_scalar::<_, i64>(
            &state.q("SELECT COUNT(*) FROM cards c JOIN lists l ON c.list_id = l.id WHERE l.board_id = ?")
        )
        .bind(&board_id_str)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

        results.push(BoardSummaryResponse {
            id: parse_uuid(&b.0)?,
            title: b.1,
            description: crate::models::user::empty_to_none(b.2),
            visibility: b.3,
            background_color: crate::models::user::empty_to_none(b.4),
            background_image_url: crate::models::user::empty_to_none(b.5),
            is_starred: b.6 != 0,
            is_archived: b.7 != 0,
            owner_id: parse_uuid(&b.8)?,
            team_id: if b.9.is_empty() { None } else { Some(parse_uuid(&b.9)?) },
            member_count,
            card_count,
            created_at: parse_dt(&b.10)?,
        });
    }

    Ok(results)
}

pub async fn create_board(
    state: &AppState,
    user_id: Uuid,
    req: CreateBoardRequest,
) -> Result<BoardResponse, AppError> {
    let visibility = req.visibility.as_deref().unwrap_or("private");
    let board_id = Uuid::new_v4();

    sqlx::query(
        &state.q(r#"INSERT INTO boards (id, title, description, visibility, background_color, background_image_url, owner_id, team_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#)
    )
    .bind(board_id.to_string())
    .bind(&req.title)
    .bind(&req.description)
    .bind(visibility)
    .bind(&req.background_color)
    .bind(&req.background_image_url)
    .bind(user_id.to_string())
    .bind(req.team_id.map(|id| id.to_string()))
    .execute(&state.db)
    .await?;

    sqlx::query(
        &state.q("INSERT INTO board_members (id, board_id, user_id, role) VALUES (?, ?, ?, 'owner')")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(board_id.to_string())
    .bind(user_id.to_string())
    .execute(&state.db)
    .await?;

    let default_labels = vec![
        ("Bug", "#ef4444"),
        ("Feature", "#3b82f6"),
        ("Enhancement", "#8b5cf6"),
        ("Documentation", "#06b6d4"),
        ("High Priority", "#f97316"),
        ("Low Priority", "#22c55e"),
    ];

    for (name, color) in default_labels {
        sqlx::query(&state.q("INSERT INTO labels (id, board_id, name, color) VALUES (?, ?, ?, ?)"))
            .bind(Uuid::new_v4().to_string())
            .bind(board_id.to_string())
            .bind(name)
            .bind(color)
            .execute(&state.db)
            .await?;
    }

    sqlx::query(
        &state.q("INSERT INTO activity_log (id, board_id, user_id, action) VALUES (?, ?, ?, 'board_created')")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(board_id.to_string())
    .bind(user_id.to_string())
    .execute(&state.db)
    .await?;

    get_board(state, user_id, board_id).await
}

pub async fn get_board(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<BoardResponse, AppError> {
    let board = sqlx::query_as::<_, crate::models::board::Board>(
        &state.q(r#"SELECT id, title, COALESCE(description,'') AS description, visibility, COALESCE(background_color,'') AS background_color, COALESCE(background_image_url,'') AS background_image_url,
                  is_starred, is_archived, owner_id, COALESCE(team_id,'') AS team_id, created_at, updated_at
           FROM boards WHERE id = ?"#)
    )
    .bind(board_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Board not found".to_string()))?;

    check_board_access(state, user_id, board_id, &board.visibility).await?;

    let lists = sqlx::query_as::<_, crate::models::board::List>(
        &state.q("SELECT id, board_id, title, position, is_archived, created_at, updated_at FROM lists WHERE board_id = ? AND is_archived = 0 ORDER BY position")
    )
    .bind(board_id.to_string())
    .fetch_all(&state.db)
    .await?;

    let mut list_responses = Vec::new();
    for list in &lists {
        let list_id_str = list.id.clone();
        // (id, title, position, priority, due_date, cover_color)
        let cards = sqlx::query_as::<_, (String, String, i32, String, String, String)>(
            &state.q(r#"SELECT c.id, c.title, c.position, c.priority,
                   COALESCE(c.due_date,'') AS due_date, COALESCE(c.cover_color,'') AS cover_color
               FROM cards c WHERE c.list_id = ? AND c.is_archived = 0 ORDER BY c.position"#)
        )
        .bind(&list_id_str)
        .fetch_all(&state.db)
        .await?;

        let mut card_responses = Vec::new();
        for card in cards {
            let card_id_str = card.0.clone();
            let assignee_count = sqlx::query_scalar::<_, i64>(
                &state.q("SELECT COUNT(*) FROM card_assignees WHERE card_id = ?")
            )
            .bind(&card_id_str)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

            let comment_count = sqlx::query_scalar::<_, i64>(
                &state.q("SELECT COUNT(*) FROM comments WHERE card_id = ?")
            )
            .bind(&card_id_str)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

            let label_id_strs: Vec<String> = sqlx::query_scalar(
                &state.q("SELECT label_id FROM card_labels WHERE card_id = ?")
            )
            .bind(&card_id_str)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let label_ids: Vec<Uuid> = label_id_strs.iter()
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect();

            let due_date = if card.4.is_empty() { None } else {
                chrono::DateTime::parse_from_rfc3339(&card.4)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            };

            card_responses.push(CardSummaryResponse {
                id: parse_uuid(&card.0)?,
                title: card.1,
                position: card.2,
                priority: card.3,
                due_date,
                assignee_count,
                comment_count,
                checklist_progress: None,
                label_ids,
                cover_color: crate::models::user::empty_to_none(card.5),
            });
        }

        list_responses.push(ListWithCardsResponse {
            id: parse_uuid(&list.id)?,
            title: list.title.clone(),
            position: list.position,
            cards: card_responses,
        });
    }

    let labels = list_labels(state, user_id, board_id).await?;
    let members = list_board_members(state, user_id, board_id).await?;

    Ok(BoardResponse {
        id: parse_uuid(&board.id)?,
        title: board.title,
        description: crate::models::user::empty_to_none(board.description),
        visibility: board.visibility,
        background_color: crate::models::user::empty_to_none(board.background_color),
        background_image_url: crate::models::user::empty_to_none(board.background_image_url),
        is_starred: board.is_starred != 0,
        is_archived: board.is_archived != 0,
        owner_id: parse_uuid(&board.owner_id)?,
        team_id: if board.team_id.is_empty() { None } else { Some(parse_uuid(&board.team_id)?) },
        lists: list_responses,
        labels,
        members,
        created_at: parse_dt(&board.created_at)?,
        updated_at: parse_dt(&board.updated_at)?,
    })
}

pub async fn update_board(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    req: UpdateBoardRequest,
) -> Result<BoardResponse, AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    let mut updates = Vec::new();

    if req.title.is_some() { updates.push("title = ?"); }
    if req.description.is_some() { updates.push("description = ?"); }
    if req.visibility.is_some() { updates.push("visibility = ?"); }
    if req.background_color.is_some() { updates.push("background_color = ?"); }
    if req.background_image_url.is_some() { updates.push("background_image_url = ?"); }

    if updates.is_empty() {
        return get_board(state, user_id, board_id).await;
    }

    let query_raw = format!("UPDATE boards SET {} WHERE id = ?", updates.join(", "));
    let query = state.q(&query_raw);

    let mut q = sqlx::query(&query);
    if let Some(ref title) = req.title { q = q.bind(title); }
    if let Some(ref desc) = req.description { q = q.bind(desc); }
    if let Some(ref vis) = req.visibility { q = q.bind(vis); }
    if let Some(ref bg) = req.background_color { q = q.bind(bg); }
    if let Some(ref img) = req.background_image_url { q = q.bind(img); }
    q = q.bind(board_id.to_string());

    q.execute(&state.db).await?;

    get_board(state, user_id, board_id).await
}

pub async fn delete_board(state: &AppState, user_id: Uuid, board_id: Uuid) -> Result<(), AppError> {
    let board = sqlx::query_as::<_, crate::models::board::Board>(
        &state.q(r#"SELECT id, title, COALESCE(description,'') AS description, visibility, COALESCE(background_color,'') AS background_color, COALESCE(background_image_url,'') AS background_image_url,
                  is_starred, is_archived, owner_id, COALESCE(team_id,'') AS team_id, created_at, updated_at
           FROM boards WHERE id = ?"#)
    )
    .bind(board_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Board not found".to_string()))?;

    if board.owner_id != user_id.to_string() {
        return Err(AppError::Forbidden);
    }

    sqlx::query(&state.q("DELETE FROM boards WHERE id = ?"))
        .bind(board_id.to_string())
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn list_board_members(
    state: &AppState,
    _user_id: Uuid,
    board_id: Uuid,
) -> Result<Vec<BoardMemberResponse>, AppError> {
    let members = sqlx::query_as::<_, (String, String, String, String, String)>(
        &state.q(r#"SELECT u.id, u.username, u.display_name, COALESCE(u.avatar_url,'') AS avatar_url, bm.role
           FROM board_members bm JOIN users u ON bm.user_id = u.id
           WHERE bm.board_id = ?"#)
    )
    .bind(board_id.to_string())
    .fetch_all(&state.db)
    .await?;

    members
        .into_iter()
        .map(|m| Ok(BoardMemberResponse {
            user_id: parse_uuid(&m.0)?,
            username: m.1,
            display_name: m.2,
            avatar_url: crate::models::user::empty_to_none(m.3),
            role: m.4,
        }))
        .collect()
}

pub async fn add_board_member(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    req: AddBoardMemberRequest,
) -> Result<BoardMemberResponse, AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    let role = req.role.as_deref().unwrap_or("member");
    sqlx::query(
        &state.q("INSERT INTO board_members (id, board_id, user_id, role) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(board_id.to_string())
    .bind(req.user_id.to_string())
    .bind(role)
    .execute(&state.db)
    .await?;

    let user = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q("SELECT id, username, display_name, COALESCE(avatar_url,'') AS avatar_url FROM users WHERE id = ?")
    )
    .bind(req.user_id.to_string())
    .fetch_one(&state.db)
    .await?;

    Ok(BoardMemberResponse {
        user_id: parse_uuid(&user.0)?,
        username: user.1,
        display_name: user.2,
        avatar_url: crate::models::user::empty_to_none(user.3),
        role: role.to_string(),
    })
}

pub async fn remove_board_member(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    member_user_id: Uuid,
) -> Result<(), AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    sqlx::query(&state.q("DELETE FROM board_members WHERE board_id = ? AND user_id = ?"))
        .bind(board_id.to_string())
        .bind(member_user_id.to_string())
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn list_labels(
    state: &AppState,
    _user_id: Uuid,
    board_id: Uuid,
) -> Result<Vec<LabelResponse>, AppError> {
    let labels = sqlx::query_as::<_, crate::models::board::Label>(
        &state.q("SELECT id, board_id, name, color, created_at FROM labels WHERE board_id = ? ORDER BY created_at")
    )
    .bind(board_id.to_string())
    .fetch_all(&state.db)
    .await?;

    labels
        .into_iter()
        .map(|l| Ok(LabelResponse {
            id: parse_uuid(&l.id)?,
            name: l.name,
            color: l.color,
        }))
        .collect()
}

pub async fn create_label(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    req: CreateLabelRequest,
) -> Result<LabelResponse, AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    let label_id = Uuid::new_v4();
    sqlx::query(
        &state.q("INSERT INTO labels (id, board_id, name, color) VALUES (?, ?, ?, ?)")
    )
    .bind(label_id.to_string())
    .bind(board_id.to_string())
    .bind(&req.name)
    .bind(&req.color)
    .execute(&state.db)
    .await?;

    let label = sqlx::query_as::<_, crate::models::board::Label>(
        &state.q("SELECT id, board_id, name, color, created_at FROM labels WHERE id = ?")
    )
    .bind(label_id.to_string())
    .fetch_one(&state.db)
    .await?;

    Ok(LabelResponse {
        id: parse_uuid(&label.id)?,
        name: label.name,
        color: label.color,
    })
}

pub async fn update_label(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    label_id: Uuid,
    req: CreateLabelRequest,
) -> Result<LabelResponse, AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    sqlx::query(
        &state.q("UPDATE labels SET name = ?, color = ? WHERE id = ? AND board_id = ?")
    )
    .bind(&req.name)
    .bind(&req.color)
    .bind(label_id.to_string())
    .bind(board_id.to_string())
    .execute(&state.db)
    .await?;

    let label = sqlx::query_as::<_, crate::models::board::Label>(
        &state.q("SELECT id, board_id, name, color, created_at FROM labels WHERE id = ? AND board_id = ?")
    )
    .bind(label_id.to_string())
    .bind(board_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Label not found".to_string()))?;

    Ok(LabelResponse {
        id: parse_uuid(&label.id)?,
        name: label.name,
        color: label.color,
    })
}

pub async fn delete_label(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    label_id: Uuid,
) -> Result<(), AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    sqlx::query(&state.q("DELETE FROM labels WHERE id = ? AND board_id = ?"))
        .bind(label_id.to_string())
        .bind(board_id.to_string())
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn get_board_activity(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    check_board_access(state, user_id, board_id, "private").await?;

    // (id, card_id, user_id, action, metadata, created_at)
    let activities = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        &state.q(r#"SELECT al.id, COALESCE(al.card_id,'') AS card_id, al.user_id, al.action,
               COALESCE(al.metadata,'') AS metadata, al.created_at
           FROM activity_log al WHERE al.board_id = ? ORDER BY al.created_at DESC LIMIT 50"#)
    )
    .bind(board_id.to_string())
    .fetch_all(&state.db)
    .await?;

    Ok(activities
        .into_iter()
        .map(|a| {
            let metadata: serde_json::Value = if a.4.is_empty() { serde_json::Value::Null } else {
                serde_json::from_str(&a.4).unwrap_or(serde_json::Value::Null)
            };
            serde_json::json!({
                "id": a.0,
                "card_id": if a.1.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(a.1) },
                "user_id": a.2,
                "action": a.3,
                "metadata": metadata,
                "created_at": a.5,
            })
        })
        .collect())
}

pub async fn archive_board(state: &AppState, user_id: Uuid, board_id: Uuid) -> Result<(), AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    sqlx::query(&state.q("UPDATE boards SET is_archived = CASE WHEN is_archived = 1 THEN 0 ELSE 1 END WHERE id = ?"))
        .bind(board_id.to_string())
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn toggle_star(state: &AppState, user_id: Uuid, board_id: Uuid) -> Result<bool, AppError> {
    check_board_access(state, user_id, board_id, "private").await?;

    sqlx::query(&state.q("UPDATE boards SET is_starred = CASE WHEN is_starred = 1 THEN 0 ELSE 1 END WHERE id = ?"))
        .bind(board_id.to_string())
        .execute(&state.db)
        .await?;

    let is_starred = sqlx::query_scalar::<_, i64>(
        &state.q("SELECT is_starred FROM boards WHERE id = ?")
    )
    .bind(board_id.to_string())
    .fetch_one(&state.db)
    .await?;

    Ok(is_starred != 0)
}

pub async fn list_custom_fields(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    check_board_access(state, user_id, board_id, "private").await?;

    let fields = sqlx::query_as::<_, (String, String, String, String, i32)>(
        &state.q("SELECT id, name, field_type, options, position FROM board_custom_fields WHERE board_id = ? ORDER BY position")
    )
    .bind(board_id.to_string())
    .fetch_all(&state.db)
    .await?;

    Ok(fields
        .into_iter()
        .map(|f| {
            let options: serde_json::Value = serde_json::from_str(&f.3).unwrap_or(serde_json::Value::Null);
            serde_json::json!({
                "id": f.0, "name": f.1, "field_type": f.2, "options": options, "position": f.4,
            })
        })
        .collect())
}

pub async fn create_custom_field(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    req: CreateCustomFieldRequest,
) -> Result<serde_json::Value, AppError> {
    check_board_write_access(state, user_id, board_id).await?;

    let position = sqlx::query_scalar::<_, i32>(
        &state.q("SELECT COALESCE(MAX(position), -1) + 1 FROM board_custom_fields WHERE board_id = ?")
    )
    .bind(board_id.to_string())
    .fetch_one(&state.db)
    .await?;

    let options_json = serde_json::to_string(&req.options.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
    let field_id = Uuid::new_v4();

    sqlx::query(
        &state.q("INSERT INTO board_custom_fields (id, board_id, name, field_type, options, position) VALUES (?, ?, ?, ?, ?, ?)")
    )
    .bind(field_id.to_string())
    .bind(board_id.to_string())
    .bind(&req.name)
    .bind(&req.field_type)
    .bind(&options_json)
    .bind(position)
    .execute(&state.db)
    .await?;

    let field = sqlx::query_as::<_, (String, String, String, String, i32)>(
        &state.q("SELECT id, name, field_type, options, position FROM board_custom_fields WHERE id = ?")
    )
    .bind(field_id.to_string())
    .fetch_one(&state.db)
    .await?;

    let options: serde_json::Value = serde_json::from_str(&field.3).unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({
        "id": field.0, "name": field.1, "field_type": field.2, "options": options, "position": field.4,
    }))
}

/// Built-in board templates (no DB rows needed)
fn builtin_templates() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Kanban Board",
            "description": "Classic kanban workflow with To Do, In Progress, and Done columns",
            "is_builtin": true,
            "background_color": "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
            "lists": ["To Do", "In Progress", "Review", "Done"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000002",
            "name": "Project Management",
            "description": "Full project lifecycle from planning to launch",
            "is_builtin": true,
            "background_color": "linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)",
            "lists": ["Backlog", "Planning", "In Development", "Testing", "Staging", "Deployed"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000003",
            "name": "Bug Tracker",
            "description": "Track and resolve bugs through triage to resolution",
            "is_builtin": true,
            "background_color": "#ef4444",
            "lists": ["Reported", "Triaged", "In Progress", "Fixed", "Verified", "Closed"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000004",
            "name": "Sprint Board",
            "description": "Agile sprint planning and execution",
            "is_builtin": true,
            "background_color": "linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)",
            "lists": ["Sprint Backlog", "In Progress", "In Review", "QA", "Done"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000005",
            "name": "Content Pipeline",
            "description": "Manage content creation from ideation to publishing",
            "is_builtin": true,
            "background_color": "linear-gradient(135deg, #fa709a 0%, #fee140 100%)",
            "lists": ["Ideas", "Writing", "Editing", "Design", "Review", "Published"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000006",
            "name": "Design Process",
            "description": "End-to-end design workflow",
            "is_builtin": true,
            "background_color": "linear-gradient(135deg, #a18cd1 0%, #fbc2eb 100%)",
            "lists": ["Research", "Wireframes", "Visual Design", "Prototyping", "Handoff"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000007",
            "name": "Personal Tasks",
            "description": "Simple personal task management",
            "is_builtin": true,
            "background_color": "#06b6d4",
            "lists": ["Inbox", "Today", "This Week", "Someday", "Done"]
        }),
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000008",
            "name": "Blank Board",
            "description": "Start from scratch with an empty board",
            "is_builtin": true,
            "background_color": "#3b82f6",
            "lists": []
        }),
    ]
}

pub async fn list_templates(state: &AppState, user_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    // Start with built-in templates
    let mut result = builtin_templates();

    // Append user/public templates from DB
    let templates = sqlx::query_as::<_, (String, String, String, i64, String)>(
        &state.q("SELECT id, name, COALESCE(description,'') AS description, is_public, created_at FROM board_templates WHERE is_public = 1 OR created_by = ?")
    )
    .bind(user_id.to_string())
    .fetch_all(&state.db)
    .await?;

    for t in templates {
        result.push(serde_json::json!({
            "id": t.0, "name": t.1,
            "description": if t.2.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(t.2) },
            "is_builtin": false, "is_public": t.3, "created_at": t.4,
        }));
    }

    Ok(result)
}

pub async fn create_template(
    state: &AppState,
    user_id: Uuid,
    req: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let name = req["name"].as_str().ok_or(AppError::Validation("name is required".to_string()))?;
    let description = req["description"].as_str();
    let board_id = req["board_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("board_id is required".to_string()))?;

    let board = get_board(state, user_id, board_id).await?;
    let template_data = serde_json::to_string(&board).unwrap_or_default();
    let template_id = Uuid::new_v4();

    sqlx::query(
        &state.q("INSERT INTO board_templates (id, name, description, template_data, created_by) VALUES (?, ?, ?, ?, ?)")
    )
    .bind(template_id.to_string())
    .bind(name)
    .bind(description)
    .bind(&template_data)
    .bind(user_id.to_string())
    .execute(&state.db)
    .await?;

    let template = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q("SELECT id, name, COALESCE(description,'') AS description, created_at FROM board_templates WHERE id = ?")
    )
    .bind(template_id.to_string())
    .fetch_one(&state.db)
    .await?;

    Ok(serde_json::json!({
        "id": template.0, "name": template.1,
        "description": if template.2.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(template.2) },
        "created_at": template.3,
    }))
}

pub async fn create_from_template(
    state: &AppState,
    user_id: Uuid,
    template_id: Uuid,
    req: serde_json::Value,
) -> Result<BoardResponse, AppError> {
    let title = req["title"].as_str().unwrap_or("New Board from Template");

    // Check if it's a built-in template
    let builtin = builtin_templates().into_iter().find(|t| {
        t["id"].as_str().map(|s| Uuid::parse_str(s).ok()) == Some(Some(template_id))
    });

    if let Some(tmpl) = builtin {
        let bg_color = tmpl["background_color"].as_str().map(String::from);
        let create_req = CreateBoardRequest {
            title: title.to_string(),
            description: req["description"].as_str().map(String::from),
            visibility: req["visibility"].as_str().map(String::from).or(Some("private".to_string())),
            team_id: req["team_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
            background_color: req["background_color"].as_str().map(String::from).or(bg_color),
            background_image_url: req["background_image_url"].as_str().map(String::from),
        };

        let board = create_board(state, user_id, create_req).await?;

        // Create lists from the template
        if let Some(lists) = tmpl["lists"].as_array() {
            for (i, list_name) in lists.iter().enumerate() {
                if let Some(name) = list_name.as_str() {
                    sqlx::query(
                        &state.q("INSERT INTO lists (id, board_id, title, position) VALUES (?, ?, ?, ?)")
                    )
                    .bind(Uuid::new_v4().to_string())
                    .bind(board.id.to_string())
                    .bind(name)
                    .bind(i as i32)
                    .execute(&state.db)
                    .await?;
                }
            }
        }

        // Re-fetch to include the lists
        return get_board(state, user_id, board.id).await;
    }

    // User-created template from DB
    let template = sqlx::query_as::<_, (String, String)>(
        &state.q("SELECT id, template_data FROM board_templates WHERE id = ?")
    )
    .bind(template_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Template not found".to_string()))?;

    // Parse template_data to extract lists and background
    let tmpl_data: serde_json::Value = serde_json::from_str(&template.1).unwrap_or_default();
    let bg_color = tmpl_data["background_color"].as_str().map(String::from);
    let bg_image = tmpl_data["background_image_url"].as_str().map(String::from);

    let create_req = CreateBoardRequest {
        title: title.to_string(),
        description: req["description"].as_str().map(String::from),
        visibility: req["visibility"].as_str().map(String::from).or(Some("private".to_string())),
        team_id: req["team_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        background_color: req["background_color"].as_str().map(String::from).or(bg_color),
        background_image_url: req["background_image_url"].as_str().map(String::from).or(bg_image),
    };

    let board = create_board(state, user_id, create_req).await?;

    // Restore lists from template_data
    if let Some(lists) = tmpl_data["lists"].as_array() {
        for (i, list_val) in lists.iter().enumerate() {
            let list_title = list_val["title"].as_str().unwrap_or("Untitled");
            sqlx::query(
                &state.q("INSERT INTO lists (id, board_id, title, position) VALUES (?, ?, ?, ?)")
            )
            .bind(Uuid::new_v4().to_string())
            .bind(board.id.to_string())
            .bind(list_title)
            .bind(i as i32)
            .execute(&state.db)
            .await?;
        }
    }

    get_board(state, user_id, board.id).await
}

// Helper functions

pub(crate) async fn check_board_access(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    _visibility: &str,
) -> Result<(), AppError> {
    let uid = user_id.to_string();
    let bid = board_id.to_string();
    let has_access = sqlx::query_scalar::<_, i64>(
        &state.q(r#"SELECT COUNT(*) FROM boards b
            LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = ?
            LEFT JOIN team_members tm ON b.team_id = tm.team_id AND tm.user_id = ?
            WHERE b.id = ? AND (b.owner_id = ? OR bm.user_id IS NOT NULL OR b.visibility = 'public'
                  OR (b.visibility = 'team' AND tm.user_id IS NOT NULL))"#)
    )
    .bind(&uid)
    .bind(&uid)
    .bind(&bid)
    .bind(&uid)
    .fetch_one(&state.db)
    .await?;

    if has_access == 0 {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

async fn check_board_write_access(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<(), AppError> {
    let uid = user_id.to_string();
    let bid = board_id.to_string();
    let has_access = sqlx::query_scalar::<_, i64>(
        &state.q(r#"SELECT COUNT(*) FROM boards b
            LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = ?
            LEFT JOIN team_members tm ON b.team_id = tm.team_id AND tm.user_id = ?
            WHERE b.id = ? AND (b.owner_id = ?
                  OR (bm.user_id IS NOT NULL AND bm.role IN ('owner', 'admin', 'member'))
                  OR (b.visibility = 'team' AND tm.user_id IS NOT NULL))"#)
    )
    .bind(&uid)
    .bind(&uid)
    .bind(&bid)
    .bind(&uid)
    .fetch_one(&state.db)
    .await?;

    if has_access == 0 {
        return Err(AppError::Forbidden);
    }

    Ok(())
}
