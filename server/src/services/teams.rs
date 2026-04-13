use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::teams::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

fn team_model_to_response(team: &crate::models::team::Team, member_count: i64, board_count: i64) -> Result<TeamResponse, AppError> {
    Ok(TeamResponse {
        id: parse_uuid(&team.id)?,
        name: team.name.clone(),
        slug: team.slug.clone(),
        description: crate::models::user::empty_to_none(team.description.clone()),
        avatar_url: crate::models::user::empty_to_none(team.avatar_url.clone()),
        member_count,
        board_count,
        created_by: parse_uuid(&team.created_by)?,
        created_at: parse_dt(&team.created_at)?,
    })
}

pub async fn list_teams(state: &AppState, user_id: Uuid) -> Result<Vec<TeamResponse>, AppError> {
    let teams = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        &state.q(r#"SELECT t.id, t.name, t.slug, COALESCE(t.description,'') AS description, COALESCE(t.avatar_url,'') AS avatar_url, t.created_by, t.created_at
           FROM teams t JOIN team_members tm ON t.id = tm.team_id WHERE tm.user_id = ?"#)
    )
    .bind(user_id.to_string())
    .fetch_all(&state.db)
    .await?;

    let mut results = Vec::new();
    for t in teams {
        let team_id_str = t.0.clone();
        let member_count = sqlx::query_scalar::<_, i64>(&state.q("SELECT COUNT(*) FROM team_members WHERE team_id = ?"))
            .bind(&team_id_str).fetch_one(&state.db).await.unwrap_or(0);
        let board_count = sqlx::query_scalar::<_, i64>(&state.q("SELECT COUNT(*) FROM boards WHERE team_id = ?"))
            .bind(&team_id_str).fetch_one(&state.db).await.unwrap_or(0);
        results.push(TeamResponse {
            id: parse_uuid(&t.0)?,
            name: t.1,
            slug: t.2,
            description: crate::models::user::empty_to_none(t.3),
            avatar_url: crate::models::user::empty_to_none(t.4),
            member_count,
            board_count,
            created_by: parse_uuid(&t.5)?,
            created_at: parse_dt(&t.6)?,
        });
    }
    Ok(results)
}

pub async fn create_team(state: &AppState, user_id: Uuid, req: CreateTeamRequest) -> Result<TeamResponse, AppError> {
    let slug = req.name.to_lowercase().replace(' ', "-").chars().filter(|c| c.is_alphanumeric() || *c == '-').collect::<String>();
    let team_id = Uuid::new_v4();

    sqlx::query(
        &state.q("INSERT INTO teams (id, name, slug, description, created_by) VALUES (?, ?, ?, ?, ?)")
    )
    .bind(team_id.to_string()).bind(&req.name).bind(&slug).bind(&req.description).bind(user_id.to_string())
    .execute(&state.db).await?;

    sqlx::query(&state.q("INSERT INTO team_members (id, team_id, user_id, role) VALUES (?, ?, ?, 'owner')"))
        .bind(Uuid::new_v4().to_string()).bind(team_id.to_string()).bind(user_id.to_string()).execute(&state.db).await?;

    let team = sqlx::query_as::<_, crate::models::team::Team>(&state.q("SELECT id, name, slug, COALESCE(description,'') AS description, COALESCE(avatar_url,'') AS avatar_url, created_by, created_at, updated_at FROM teams WHERE id = ?"))
        .bind(team_id.to_string()).fetch_one(&state.db).await?;

    team_model_to_response(&team, 1, 0)
}

pub async fn get_team(state: &AppState, user_id: Uuid, team_id: Uuid) -> Result<TeamResponse, AppError> {
    let team = sqlx::query_as::<_, crate::models::team::Team>(&state.q("SELECT id, name, slug, COALESCE(description,'') AS description, COALESCE(avatar_url,'') AS avatar_url, created_by, created_at, updated_at FROM teams WHERE id = ?"))
        .bind(team_id.to_string()).fetch_optional(&state.db).await?
        .ok_or(AppError::NotFound("Team not found".to_string()))?;

    let is_member = sqlx::query_scalar::<_, i64>(
        &state.q("SELECT COUNT(*) FROM team_members WHERE team_id = ? AND user_id = ?")
    )
    .bind(team_id.to_string()).bind(user_id.to_string()).fetch_one(&state.db).await? != 0;

    if !is_member { return Err(AppError::Forbidden); }

    let member_count = sqlx::query_scalar::<_, i64>(&state.q("SELECT COUNT(*) FROM team_members WHERE team_id = ?"))
        .bind(team_id.to_string()).fetch_one(&state.db).await.unwrap_or(0);
    let board_count = sqlx::query_scalar::<_, i64>(&state.q("SELECT COUNT(*) FROM boards WHERE team_id = ?"))
        .bind(team_id.to_string()).fetch_one(&state.db).await.unwrap_or(0);

    team_model_to_response(&team, member_count, board_count)
}

pub async fn update_team(state: &AppState, user_id: Uuid, team_id: Uuid, req: UpdateTeamRequest) -> Result<TeamResponse, AppError> {
    check_team_admin(state, user_id, team_id).await?;

    if let Some(name) = &req.name {
        sqlx::query(&state.q("UPDATE teams SET name = ? WHERE id = ?")).bind(name).bind(team_id.to_string()).execute(&state.db).await?;
    }
    if let Some(desc) = &req.description {
        sqlx::query(&state.q("UPDATE teams SET description = ? WHERE id = ?")).bind(desc).bind(team_id.to_string()).execute(&state.db).await?;
    }
    if let Some(avatar) = &req.avatar_url {
        sqlx::query(&state.q("UPDATE teams SET avatar_url = ? WHERE id = ?")).bind(avatar).bind(team_id.to_string()).execute(&state.db).await?;
    }

    get_team(state, user_id, team_id).await
}

pub async fn delete_team(state: &AppState, user_id: Uuid, team_id: Uuid) -> Result<(), AppError> {
    let team = sqlx::query_as::<_, crate::models::team::Team>(&state.q("SELECT id, name, slug, COALESCE(description,'') AS description, COALESCE(avatar_url,'') AS avatar_url, created_by, created_at, updated_at FROM teams WHERE id = ?"))
        .bind(team_id.to_string()).fetch_optional(&state.db).await?
        .ok_or(AppError::NotFound("Team not found".to_string()))?;

    if team.created_by != user_id.to_string() { return Err(AppError::Forbidden); }

    sqlx::query(&state.q("DELETE FROM teams WHERE id = ?")).bind(team_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn list_members(state: &AppState, user_id: Uuid, team_id: Uuid) -> Result<Vec<TeamMemberResponse>, AppError> {
    let is_member = sqlx::query_scalar::<_, i64>(
        &state.q("SELECT COUNT(*) FROM team_members WHERE team_id = ? AND user_id = ?")
    )
    .bind(team_id.to_string()).bind(user_id.to_string()).fetch_one(&state.db).await? != 0;
    if !is_member { return Err(AppError::Forbidden); }

    let members = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        &state.q(r#"SELECT u.id, u.username, u.display_name, COALESCE(u.avatar_url,'') AS avatar_url, tm.role, tm.joined_at
           FROM team_members tm JOIN users u ON tm.user_id = u.id WHERE tm.team_id = ?"#)
    )
    .bind(team_id.to_string()).fetch_all(&state.db).await?;

    members.into_iter().map(|m| Ok(TeamMemberResponse {
        user_id: parse_uuid(&m.0)?,
        username: m.1,
        display_name: m.2,
        avatar_url: crate::models::user::empty_to_none(m.3),
        role: m.4,
        joined_at: parse_dt(&m.5)?,
    })).collect()
}

pub async fn add_member(state: &AppState, user_id: Uuid, team_id: Uuid, req: InviteMemberRequest) -> Result<TeamMemberResponse, AppError> {
    check_team_admin(state, user_id, team_id).await?;

    let target_user = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q("SELECT id, username, display_name, COALESCE(avatar_url,'') AS avatar_url FROM users WHERE email = ?")
    )
    .bind(&req.email).fetch_optional(&state.db).await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    let role = req.role.as_deref().unwrap_or("member");
    sqlx::query(&state.q("INSERT INTO team_members (id, team_id, user_id, role) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING"))
        .bind(Uuid::new_v4().to_string()).bind(team_id.to_string()).bind(&target_user.0).bind(role).execute(&state.db).await?;

    Ok(TeamMemberResponse {
        user_id: parse_uuid(&target_user.0)?,
        username: target_user.1,
        display_name: target_user.2,
        avatar_url: crate::models::user::empty_to_none(target_user.3),
        role: role.to_string(),
        joined_at: chrono::Utc::now(),
    })
}

pub async fn update_member_role(state: &AppState, user_id: Uuid, team_id: Uuid, target_user_id: Uuid, role: String) -> Result<TeamMemberResponse, AppError> {
    check_team_admin(state, user_id, team_id).await?;

    sqlx::query(&state.q("UPDATE team_members SET role = ? WHERE team_id = ? AND user_id = ?"))
        .bind(&role).bind(team_id.to_string()).bind(target_user_id.to_string()).execute(&state.db).await?;

    let user = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q("SELECT id, username, display_name, COALESCE(avatar_url,'') AS avatar_url FROM users WHERE id = ?")
    )
    .bind(target_user_id.to_string()).fetch_one(&state.db).await?;

    Ok(TeamMemberResponse {
        user_id: parse_uuid(&user.0)?,
        username: user.1,
        display_name: user.2,
        avatar_url: crate::models::user::empty_to_none(user.3),
        role,
        joined_at: chrono::Utc::now(),
    })
}

pub async fn remove_member(state: &AppState, user_id: Uuid, team_id: Uuid, target_user_id: Uuid) -> Result<(), AppError> {
    check_team_admin(state, user_id, team_id).await?;
    sqlx::query(&state.q("DELETE FROM team_members WHERE team_id = ? AND user_id = ?"))
        .bind(team_id.to_string()).bind(target_user_id.to_string()).execute(&state.db).await?;
    Ok(())
}

pub async fn invite_member(state: &AppState, user_id: Uuid, team_id: Uuid, req: InviteMemberRequest) -> Result<serde_json::Value, AppError> {
    check_team_admin(state, user_id, team_id).await?;

    let token: String = rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
        .take(32).map(char::from).collect();

    let role = req.role.as_deref().unwrap_or("member");
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    sqlx::query(
        &state.q("INSERT INTO team_invites (id, team_id, email, role, invited_by, token, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(team_id.to_string())
    .bind(&req.email)
    .bind(role)
    .bind(user_id.to_string())
    .bind(&token)
    .bind(expires_at.to_rfc3339())
    .execute(&state.db).await?;

    Ok(serde_json::json!({"token": token, "expires_at": expires_at}))
}

pub async fn accept_invite(state: &AppState, user_id: Uuid, token: &str) -> Result<TeamResponse, AppError> {
    let invite = sqlx::query_as::<_, crate::models::team::TeamInvite>(
        &state.q(r#"SELECT id, team_id, email, role, invited_by, token, expires_at, COALESCE(accepted_at,'') AS accepted_at, created_at
           FROM team_invites WHERE token = ? AND accepted_at IS NULL AND expires_at > CURRENT_TIMESTAMP"#)
    )
    .bind(token).fetch_optional(&state.db).await?
    .ok_or(AppError::NotFound("Invite not found or expired".to_string()))?;

    let user = sqlx::query_as::<_, crate::models::user::User>(&state.q("SELECT id, email, username, display_name, COALESCE(password_hash,'') AS password_hash, COALESCE(avatar_url,'') AS avatar_url, COALESCE(bio,'') AS bio, is_active, is_verified, created_at, updated_at FROM users WHERE id = ?"))
        .bind(user_id.to_string()).fetch_one(&state.db).await?;

    if user.email != invite.email {
        return Err(AppError::Forbidden);
    }

    let invite_team_id = parse_uuid(&invite.team_id)?;

    sqlx::query(&state.q("INSERT INTO team_members (id, team_id, user_id, role) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING"))
        .bind(Uuid::new_v4().to_string())
        .bind(&invite.team_id)
        .bind(user_id.to_string())
        .bind(&invite.role)
        .execute(&state.db).await?;

    sqlx::query(&state.q("UPDATE team_invites SET accepted_at = CURRENT_TIMESTAMP WHERE id = ?"))
        .bind(&invite.id).execute(&state.db).await?;

    get_team(state, user_id, invite_team_id).await
}

async fn check_team_admin(state: &AppState, user_id: Uuid, team_id: Uuid) -> Result<(), AppError> {
    let role = sqlx::query_scalar::<_, String>(
        &state.q("SELECT role FROM team_members WHERE team_id = ? AND user_id = ?")
    )
    .bind(team_id.to_string()).bind(user_id.to_string()).fetch_optional(&state.db).await?
    .ok_or(AppError::Forbidden)?;

    if role != "owner" && role != "admin" {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
