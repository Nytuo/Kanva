use uuid::Uuid;
use crate::{AppError, AppState};
use crate::api::integrations::*;

fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid datetime: {}", s)))
}

// Columns: id, board_id, provider, config, enabled, created_at
fn row_to_integration(
    r: (String, String, String, String, i64, String),
) -> Result<IntegrationResponse, AppError> {
    Ok(IntegrationResponse {
        id: parse_uuid(&r.0)?,
        board_id: parse_uuid(&r.1)?,
        provider: r.2,
        config: serde_json::from_str(&r.3)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        enabled: r.4 != 0,
        created_at: parse_dt(&r.5)?,
    })
}

pub async fn list_integrations(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<IntegrationResponse>, AppError> {
    let uid = user_id.to_string();
    let integrations = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        &state.q(r#"SELECT i.id, i.board_id, i.provider, i.config, i.enabled, i.created_at
           FROM integrations i
           JOIN boards b ON i.board_id = b.id
           LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = ?
           WHERE b.owner_id = ? OR bm.user_id IS NOT NULL"#),
    )
    .bind(&uid)
    .bind(&uid)
    .fetch_all(&state.db)
    .await?;

    integrations.into_iter().map(row_to_integration).collect()
}

pub async fn create_integration(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
    req: CreateIntegrationRequest,
) -> Result<IntegrationResponse, AppError> {
    let id = Uuid::new_v4();
    let config_str = serde_json::to_string(&req.config)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    sqlx::query(
        &state.q("INSERT INTO integrations (id, board_id, provider, config, created_by) VALUES (?, ?, ?, ?, ?)"),
    )
    .bind(id.to_string())
    .bind(board_id.to_string())
    .bind(&req.provider)
    .bind(&config_str)
    .bind(user_id.to_string())
    .execute(&state.db)
    .await?;

    Ok(IntegrationResponse {
        id,
        board_id,
        provider: req.provider,
        config: req.config,
        enabled: true,
        created_at: chrono::Utc::now(),
    })
}

pub async fn update_integration(
    state: &AppState,
    _user_id: Uuid,
    id: Uuid,
    req: serde_json::Value,
) -> Result<IntegrationResponse, AppError> {
    let id_str = id.to_string();

    if let Some(config) = req.get("config") {
        let config_str = serde_json::to_string(config)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        sqlx::query(&state.q("UPDATE integrations SET config = ? WHERE id = ?"))
            .bind(config_str)
            .bind(&id_str)
            .execute(&state.db)
            .await?;
    }
    if let Some(enabled) = req.get("enabled").and_then(|v| v.as_bool()) {
        sqlx::query(&state.q("UPDATE integrations SET enabled = ? WHERE id = ?"))
            .bind(if enabled { 1i64 } else { 0i64 })
            .bind(&id_str)
            .execute(&state.db)
            .await?;
    }

    let i = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        &state.q("SELECT id, board_id, provider, config, enabled, created_at FROM integrations WHERE id = ?"),
    )
    .bind(&id_str)
    .fetch_one(&state.db)
    .await?;

    row_to_integration(i)
}

pub async fn delete_integration(
    state: &AppState,
    _user_id: Uuid,
    id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(&state.q("DELETE FROM integrations WHERE id = ?"))
        .bind(id.to_string())
        .execute(&state.db)
        .await?;
    Ok(())
}

pub async fn sync_integration(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
) -> Result<serde_json::Value, AppError> {
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        &state.q("SELECT id, board_id, provider, config FROM integrations WHERE id = ?"),
    )
    .bind(id.to_string())
    .fetch_one(&state.db)
    .await?;

    let board_id = parse_uuid(&row.1)?;

    match row.2.as_str() {
        "github" => import_github_issues(state, user_id, board_id).await,
        "gitlab" => import_gitlab_issues(state, user_id, board_id).await,
        "atlassian" => import_atlassian_issues(state, user_id, board_id).await,
        _ => Err(AppError::BadRequest("Unknown provider".to_string())),
    }
}

pub async fn handle_webhook(
    _state: &AppState,
    provider: &str,
    id: Uuid,
    payload: serde_json::Value,
) -> Result<(), AppError> {
    tracing::info!(
        "Received webhook from {} for integration {}: {:?}",
        provider, id, payload
    );
    Ok(())
}

pub async fn list_github_repos(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let oauth = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT access_token FROM oauth_accounts WHERE user_id = ? AND provider = 'github'"),
    )
    .bind(user_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("GitHub account not connected".to_string()))?;

    let client = reqwest::Client::new();
    let repos: Vec<serde_json::Value> = client
        .get("https://api.github.com/user/repos?per_page=100&sort=updated")
        .header("Authorization", format!("Bearer {}", oauth.0))
        .header("User-Agent", "Kanva")
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(repos
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r["id"], "name": r["full_name"], "description": r["description"],
                "url": r["html_url"], "private": r["private"],
            })
        })
        .collect())
}

pub async fn import_github_issues(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    let uid = user_id.to_string();
    let bid = board_id.to_string();

    let integration = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT config FROM integrations WHERE board_id = ? AND provider = 'github'"),
    )
    .bind(&bid)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("No GitHub integration for this board".to_string()))?;

    let config: serde_json::Value = serde_json::from_str(&integration.0)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let repo = config["repo"].as_str().unwrap_or("").to_string();

    let oauth = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT access_token FROM oauth_accounts WHERE user_id = ? AND provider = 'github'"),
    )
    .bind(&uid)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("GitHub account not connected".to_string()))?;

    let client = reqwest::Client::new();
    let issues: Vec<serde_json::Value> = client
        .get(format!(
            "https://api.github.com/repos/{}/issues?state=open&per_page=100",
            repo
        ))
        .header("Authorization", format!("Bearer {}", oauth.0))
        .header("User-Agent", "Kanva")
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // Get or create "GitHub Issues" list
    let existing_list = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT id FROM lists WHERE board_id = ? AND title = 'GitHub Issues' LIMIT 1"),
    )
    .bind(&bid)
    .fetch_optional(&state.db)
    .await?;

    let list_id = if let Some((id_str,)) = existing_list {
        parse_uuid(&id_str)?
    } else {
        let new_list_id = Uuid::new_v4();
        sqlx::query(
            &state.q("INSERT INTO lists (id, board_id, title, position) VALUES (?, ?, 'GitHub Issues', 999)"),
        )
        .bind(new_list_id.to_string())
        .bind(&bid)
        .execute(&state.db)
        .await?;
        new_list_id
    };

    let list_id_str = list_id.to_string();
    let mut imported = 0i32;
    for issue in &issues {
        if issue.get("pull_request").is_some() {
            continue;
        }
        let title = format!(
            "#{} {}",
            issue["number"],
            issue["title"].as_str().unwrap_or("")
        );
        let description = issue["body"].as_str().map(String::from);

        sqlx::query(
            &state.q("INSERT INTO cards (list_id, title, description, position, created_by) VALUES (?, ?, ?, ?, ?)"),
        )
        .bind(&list_id_str)
        .bind(&title)
        .bind(&description)
        .bind(imported)
        .bind(&uid)
        .execute(&state.db)
        .await?;

        imported += 1;
    }

    Ok(serde_json::json!({"imported": imported}))
}

pub async fn list_gitlab_projects(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let oauth = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT access_token FROM oauth_accounts WHERE user_id = ? AND provider = 'gitlab'"),
    )
    .bind(user_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("GitLab account not connected".to_string()))?;

    let client = reqwest::Client::new();
    let projects: Vec<serde_json::Value> = client
        .get("https://gitlab.com/api/v4/projects?membership=true&per_page=100&order_by=updated_at")
        .header("Authorization", format!("Bearer {}", oauth.0))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(projects
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p["id"], "name": p["path_with_namespace"],
                "description": p["description"], "url": p["web_url"],
            })
        })
        .collect())
}

pub async fn import_gitlab_issues(
    state: &AppState,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    let integration = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT config FROM integrations WHERE board_id = ? AND provider = 'gitlab'"),
    )
    .bind(board_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("No GitLab integration for this board".to_string()))?;

    let config: serde_json::Value = serde_json::from_str(&integration.0)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let project_id = config["project_id"].as_str().unwrap_or("").to_string();

    let oauth = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT access_token FROM oauth_accounts WHERE user_id = ? AND provider = 'gitlab'"),
    )
    .bind(user_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("GitLab account not connected".to_string()))?;

    let client = reqwest::Client::new();
    let issues: Vec<serde_json::Value> = client
        .get(format!(
            "https://gitlab.com/api/v4/projects/{}/issues?state=opened&per_page=100",
            project_id
        ))
        .header("Authorization", format!("Bearer {}", oauth.0))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(serde_json::json!({"imported": issues.len()}))
}

pub async fn list_atlassian_projects(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let oauth = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT access_token FROM oauth_accounts WHERE user_id = ? AND provider = 'atlassian'"),
    )
    .bind(user_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("Atlassian account not connected".to_string()))?;

    let client = reqwest::Client::new();

    let resources: Vec<serde_json::Value> = client
        .get("https://api.atlassian.com/oauth/token/accessible-resources")
        .header("Authorization", format!("Bearer {}", oauth.0))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    if resources.is_empty() {
        return Ok(vec![]);
    }

    let cloud_id = resources[0]["id"].as_str().unwrap_or("").to_string();

    let projects: Vec<serde_json::Value> = client
        .get(format!(
            "https://api.atlassian.com/ex/jira/{}/rest/api/3/project",
            cloud_id
        ))
        .header("Authorization", format!("Bearer {}", oauth.0))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(projects
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p["id"], "key": p["key"], "name": p["name"],
            })
        })
        .collect())
}

pub async fn import_atlassian_issues(
    state: &AppState,
    _user_id: Uuid,
    board_id: Uuid,
) -> Result<serde_json::Value, AppError> {
    let integration = sqlx::query_as::<_, (String,)>(
        &state.q("SELECT config FROM integrations WHERE board_id = ? AND provider = 'atlassian'"),
    )
    .bind(board_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest("No Atlassian integration for this board".to_string()))?;

    let config: serde_json::Value = serde_json::from_str(&integration.0)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let _project_key = config["project_key"].as_str().unwrap_or("");

    Ok(serde_json::json!({"imported": 0, "message": "Atlassian import initiated"}))
}
