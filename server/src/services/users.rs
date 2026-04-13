use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::users::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

pub async fn get_profile(state: &AppState, user_id: Uuid) -> Result<serde_json::Value, AppError> {
    let user = crate::services::auth::get_user_by_id(state, user_id).await?;
    Ok(serde_json::json!({
        "id": user.id, "email": user.email, "username": user.username,
        "display_name": user.display_name,
        "avatar_url": crate::models::user::empty_to_none(user.avatar_url),
        "bio": crate::models::user::empty_to_none(user.bio),
        "is_verified": user.is_verified != 0,
        "created_at": user.created_at,
    }))
}

pub async fn update_profile(state: &AppState, user_id: Uuid, req: UpdateProfileRequest) -> Result<serde_json::Value, AppError> {
    if let Some(name) = &req.display_name {
        sqlx::query(&state.q("UPDATE users SET display_name = ? WHERE id = ?")).bind(name).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(bio) = &req.bio {
        sqlx::query(&state.q("UPDATE users SET bio = ? WHERE id = ?")).bind(bio).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(avatar) = &req.avatar_url {
        sqlx::query(&state.q("UPDATE users SET avatar_url = ? WHERE id = ?")).bind(avatar).bind(user_id.to_string()).execute(&state.db).await?;
    }
    get_profile(state, user_id).await
}

pub async fn get_preferences(state: &AppState, user_id: Uuid) -> Result<UserPreferencesResponse, AppError> {
    let prefs = sqlx::query_as::<_, (String, String, String, i64, i64, String, i64)>(
        &state.q("SELECT theme, language, timezone, email_notifications, push_notifications, default_board_view, compact_mode FROM user_preferences WHERE user_id = ?")
    )
    .bind(user_id.to_string()).fetch_optional(&state.db).await?;

    match prefs {
        Some(p) => Ok(UserPreferencesResponse {
            theme: p.0, language: p.1, timezone: p.2, email_notifications: p.3 != 0,
            push_notifications: p.4 != 0, default_board_view: p.5, compact_mode: p.6 != 0,
        }),
        None => {
            sqlx::query(&state.q("INSERT INTO user_preferences (user_id) VALUES (?) ON CONFLICT DO NOTHING"))
                .bind(user_id.to_string()).execute(&state.db).await?;
            Ok(UserPreferencesResponse {
                theme: "system".to_string(), language: "en".to_string(), timezone: "UTC".to_string(),
                email_notifications: true, push_notifications: true,
                default_board_view: "board".to_string(), compact_mode: false,
            })
        }
    }
}

pub async fn update_preferences(state: &AppState, user_id: Uuid, req: UpdatePreferencesRequest) -> Result<UserPreferencesResponse, AppError> {
    if let Some(theme) = &req.theme {
        sqlx::query(&state.q("UPDATE user_preferences SET theme = ? WHERE user_id = ?")).bind(theme).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(lang) = &req.language {
        sqlx::query(&state.q("UPDATE user_preferences SET language = ? WHERE user_id = ?")).bind(lang).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(tz) = &req.timezone {
        sqlx::query(&state.q("UPDATE user_preferences SET timezone = ? WHERE user_id = ?")).bind(tz).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(en) = req.email_notifications {
        sqlx::query(&state.q("UPDATE user_preferences SET email_notifications = ? WHERE user_id = ?")).bind(if en { 1i64 } else { 0i64 }).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(pn) = req.push_notifications {
        sqlx::query(&state.q("UPDATE user_preferences SET push_notifications = ? WHERE user_id = ?")).bind(if pn { 1i64 } else { 0i64 }).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(view) = &req.default_board_view {
        sqlx::query(&state.q("UPDATE user_preferences SET default_board_view = ? WHERE user_id = ?")).bind(view).bind(user_id.to_string()).execute(&state.db).await?;
    }
    if let Some(cm) = req.compact_mode {
        sqlx::query(&state.q("UPDATE user_preferences SET compact_mode = ? WHERE user_id = ?")).bind(if cm { 1i64 } else { 0i64 }).bind(user_id.to_string()).execute(&state.db).await?;
    }
    get_preferences(state, user_id).await
}

pub async fn list_notifications(state: &AppState, user_id: Uuid) -> Result<Vec<NotificationResponse>, AppError> {
    let notifs = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        &state.q("SELECT id, title, message, COALESCE(link,'') AS link, is_read, created_at FROM notifications WHERE user_id = ? ORDER BY created_at DESC LIMIT 50")
    )
    .bind(user_id.to_string()).fetch_all(&state.db).await?;

    notifs.into_iter().map(|n| {
        let id = parse_uuid(&n.0)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&n.5)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime")))?;
        Ok(NotificationResponse {
            id, title: n.1, message: n.2,
            link: crate::models::user::empty_to_none(n.3),
            is_read: n.4 != 0, created_at,
        })
    }).collect()
}

pub async fn mark_notification_read(state: &AppState, user_id: Uuid, notification_id: Uuid) -> Result<(), AppError> {
    sqlx::query(&state.q("UPDATE notifications SET is_read = 1 WHERE id = ? AND user_id = ?"))
        .bind(notification_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn mark_all_notifications_read(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(&state.q("UPDATE notifications SET is_read = 1 WHERE user_id = ? AND is_read = 0"))
        .bind(user_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn search_users(state: &AppState, query: &str) -> Result<Vec<UserSearchResult>, AppError> {
    let pattern = format!("%{}%", query.to_lowercase());
    let users = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q("SELECT id, username, display_name, COALESCE(avatar_url,'') AS avatar_url FROM users WHERE (LOWER(username) LIKE ? OR LOWER(display_name) LIKE ? OR LOWER(email) LIKE ?) AND is_active = 1 LIMIT 20")
    )
    .bind(&pattern).bind(&pattern).bind(&pattern).fetch_all(&state.db).await?;

    users.into_iter().map(|u| {
        let id = parse_uuid(&u.0)?;
        Ok(UserSearchResult {
            id, username: u.1, display_name: u.2,
            avatar_url: crate::models::user::empty_to_none(u.3),
        })
    }).collect()
}

pub async fn change_password(state: &AppState, user_id: Uuid, current_password: &str, new_password: &str) -> Result<(), AppError> {
    let hash_opt: Option<String> = sqlx::query_scalar(
        &state.q("SELECT password_hash FROM users WHERE id = ?")
    )
    .bind(user_id.to_string())
    .fetch_optional(&state.db)
    .await?;

    let hash = hash_opt
        .filter(|s| !s.is_empty())
        .ok_or(AppError::BadRequest(
            "Password-based login not available for this account".to_string(),
        ))?;

    let parsed = PasswordHash::new(&hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hash parse error: {}", e)))?;
    Argon2::default()
        .verify_password(current_password.as_bytes(), &parsed)
        .map_err(|_| AppError::BadRequest("Current password is incorrect".to_string()))?;

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?
        .to_string();

    sqlx::query(&state.q("UPDATE users SET password_hash = ? WHERE id = ?"))
        .bind(&new_hash)
        .bind(user_id.to_string())
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn upload_avatar(
    state: &AppState,
    user_id: Uuid,
    mut multipart: axum::extract::Multipart,
) -> Result<String, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let filename = field.file_name().unwrap_or("avatar").to_string();
        let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;

        if data.len() > 2 * 1024 * 1024 {
            return Err(AppError::BadRequest("File too large. Max 2MB.".to_string()));
        }

        let upload_dir = &state.config.upload_dir;
        tokio::fs::create_dir_all(upload_dir).await.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let ext = filename.rsplit('.').next().unwrap_or("png");
        let file_path = format!("{}/avatar-{}.{}", upload_dir, user_id, ext);
        tokio::fs::write(&file_path, &data).await.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        let avatar_url = format!("/uploads/avatar-{}.{}", user_id, ext);

        sqlx::query(&state.q("UPDATE users SET avatar_url = ? WHERE id = ?"))
            .bind(&avatar_url)
            .bind(user_id.to_string())
            .execute(&state.db)
            .await?;

        return Ok(avatar_url);
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}

pub async fn delete_account(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    // Delete user's data in order (foreign key safe)
    sqlx::query(&state.q("DELETE FROM card_assignees WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM comments WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM notifications WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM calendar_events WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM team_members WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM user_preferences WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM refresh_tokens WHERE user_id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    sqlx::query(&state.q("DELETE FROM users WHERE id = ?")).bind(user_id.to_string()).execute(&state.db).await?;
    Ok(())
}
