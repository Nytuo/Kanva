use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::calendar::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

fn parse_opt_dt_str(s: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>, AppError> {
    if s.is_empty() { Ok(None) } else { parse_dt(s).map(Some) }
}

// (id, user_id, board_id, card_id, title, description, start_time, end_time, all_day, color, recurrence_rule, created_at)
fn row_to_event(r: (String, String, String, String, String, String, String, String, i64, String, String, String)) -> Result<CalendarEventResponse, AppError> {
    Ok(CalendarEventResponse {
        id: parse_uuid(&r.0)?,
        user_id: parse_uuid(&r.1)?,
        board_id: if r.2.is_empty() { None } else { Some(parse_uuid(&r.2)?) },
        card_id: if r.3.is_empty() { None } else { Some(parse_uuid(&r.3)?) },
        title: r.4,
        description: crate::models::user::empty_to_none(r.5),
        start_time: parse_dt(&r.6)?,
        end_time: parse_dt(&r.7)?,
        all_day: r.8 != 0,
        color: crate::models::user::empty_to_none(r.9),
        recurrence_rule: crate::models::user::empty_to_none(r.10),
        created_at: parse_dt(&r.11)?,
    })
}

pub async fn list_events(state: &AppState, user_id: Uuid, query: ListEventsQuery) -> Result<Vec<CalendarEventResponse>, AppError> {
    let uid = user_id.to_string();
    let start_str = query.start.to_rfc3339();
    let end_str = query.end.to_rfc3339();

    let mut sql = String::from(
        r#"SELECT id, user_id, COALESCE(board_id,'') AS board_id, COALESCE(card_id,'') AS card_id,
               title, COALESCE(description,'') AS description, start_time, end_time, all_day,
               COALESCE(color,'') AS color, COALESCE(recurrence_rule,'') AS recurrence_rule, created_at
           FROM calendar_events WHERE user_id = ? AND start_time <= ? AND end_time >= ?"#
    );
    if query.board_id.is_some() {
        sql.push_str(" AND board_id = ?");
    }
    sql.push_str(" ORDER BY start_time");
    let sql = state.q(&sql);

    let mut q = sqlx::query_as::<_, (String, String, String, String, String, String, String, String, i64, String, String, String)>(&sql);
    q = q.bind(&uid).bind(&end_str).bind(&start_str);
    if let Some(bid) = query.board_id { q = q.bind(bid.to_string()); }

    let events = q.fetch_all(&state.db).await?;

    // Also fetch cards with due dates as calendar events
    // Use board_members to include all cards from user's boards (not just assigned ones)
    // Use due_date != '' instead of IS NOT NULL (empty strings stored for cleared dates)
    let card_events = sqlx::query_as::<_, (String, String, String, String, String)>(
        &state.q(r#"SELECT DISTINCT c.id, l.board_id, c.title,
               COALESCE(c.start_date,'') AS start_date, COALESCE(c.due_date,'') AS due_date
           FROM cards c
           JOIN lists l ON c.list_id = l.id
           JOIN board_members bm ON l.board_id = bm.board_id
           WHERE bm.user_id = ? AND c.due_date != ''
           AND c.due_date >= ? AND c.due_date <= ?"#)
    )
    .bind(&uid).bind(&start_str).bind(&end_str)
    .fetch_all(&state.db).await?;

    let mut result: Vec<CalendarEventResponse> = events.into_iter()
        .map(row_to_event)
        .collect::<Result<Vec<_>, AppError>>()?;

    for card in card_events {
        let start = parse_opt_dt_str(&card.3)?
            .or_else(|| parse_opt_dt_str(&card.4).ok().flatten());
        let end = parse_opt_dt_str(&card.4)?
            .or(start);
        if let (Some(start), Some(end)) = (start, end) {
            result.push(CalendarEventResponse {
                id: parse_uuid(&card.0)?,
                user_id,
                board_id: Some(parse_uuid(&card.1)?),
                card_id: Some(parse_uuid(&card.0)?),
                title: card.2,
                description: None,
                start_time: start,
                end_time: end,
                all_day: false,
                color: Some("#3b82f6".to_string()),
                recurrence_rule: None,
                created_at: chrono::Utc::now(),
            });
        }
    }

    result.sort_by_key(|e| e.start_time);
    Ok(result)
}

pub async fn create_event(state: &AppState, user_id: Uuid, req: CreateEventRequest) -> Result<CalendarEventResponse, AppError> {
    let all_day = req.all_day.unwrap_or(false);
    let id = Uuid::new_v4();

    sqlx::query(
        &state.q(r#"INSERT INTO calendar_events (id, user_id, board_id, card_id, title, description, start_time, end_time, all_day, color, recurrence_rule)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
    )
    .bind(id.to_string())
    .bind(user_id.to_string())
    .bind(req.board_id.map(|u| u.to_string()))
    .bind(req.card_id.map(|u| u.to_string()))
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.start_time.to_rfc3339())
    .bind(req.end_time.to_rfc3339())
    .bind(if all_day { 1i64 } else { 0i64 })
    .bind(&req.color)
    .bind(&req.recurrence_rule)
    .execute(&state.db)
    .await?;

    Ok(CalendarEventResponse {
        id,
        user_id,
        board_id: req.board_id,
        card_id: req.card_id,
        title: req.title,
        description: req.description,
        start_time: req.start_time,
        end_time: req.end_time,
        all_day,
        color: req.color,
        recurrence_rule: req.recurrence_rule,
        created_at: chrono::Utc::now(),
    })
}

pub async fn get_event(state: &AppState, user_id: Uuid, event_id: Uuid) -> Result<CalendarEventResponse, AppError> {
    let e = sqlx::query_as::<_, (String, String, String, String, String, String, String, String, i64, String, String, String)>(
        &state.q(r#"SELECT id, user_id, COALESCE(board_id,'') AS board_id, COALESCE(card_id,'') AS card_id,
               title, COALESCE(description,'') AS description, start_time, end_time, all_day,
               COALESCE(color,'') AS color, COALESCE(recurrence_rule,'') AS recurrence_rule, created_at
           FROM calendar_events WHERE id = ? AND user_id = ?"#)
    )
    .bind(event_id.to_string()).bind(user_id.to_string()).fetch_optional(&state.db).await?
    .ok_or(AppError::NotFound("Event not found".to_string()))?;

    row_to_event(e)
}

pub async fn update_event(state: &AppState, user_id: Uuid, event_id: Uuid, req: UpdateEventRequest) -> Result<CalendarEventResponse, AppError> {
    if let Some(title) = &req.title {
        sqlx::query(&state.q("UPDATE calendar_events SET title = ? WHERE id = ? AND user_id = ?"))
            .bind(title).bind(event_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(desc) = &req.description {
        sqlx::query(&state.q("UPDATE calendar_events SET description = ? WHERE id = ? AND user_id = ?"))
            .bind(desc).bind(event_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(start) = req.start_time {
        sqlx::query(&state.q("UPDATE calendar_events SET start_time = ? WHERE id = ? AND user_id = ?"))
            .bind(start.to_rfc3339()).bind(event_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(end) = req.end_time {
        sqlx::query(&state.q("UPDATE calendar_events SET end_time = ? WHERE id = ? AND user_id = ?"))
            .bind(end.to_rfc3339()).bind(event_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(color) = &req.color {
        sqlx::query(&state.q("UPDATE calendar_events SET color = ? WHERE id = ? AND user_id = ?"))
            .bind(color).bind(event_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    }

    get_event(state, user_id, event_id).await
}

pub async fn delete_event(state: &AppState, user_id: Uuid, event_id: Uuid) -> Result<(), AppError> {
    sqlx::query(&state.q("DELETE FROM calendar_events WHERE id = ? AND user_id = ?"))
        .bind(event_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn get_board_calendar(state: &AppState, user_id: Uuid, board_id: Uuid, query: BoardCalendarQuery) -> Result<Vec<CalendarEventResponse>, AppError> {
    list_events(state, user_id, ListEventsQuery {
        start: query.start, end: query.end, board_id: Some(board_id),
    }).await
}
