use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use uuid::Uuid;

use crate::{AppError, AppState};
use crate::api::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::middleware::auth as jwt;
use crate::models::user::{User, empty_to_none};

/// Parse a UUID string, returning an internal error on failure.
fn parse_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid UUID: {}", s)))
}

pub async fn register(state: &AppState, req: RegisterRequest) -> Result<AuthResponse, AppError> {
    // Check if email or username already exists
    let existing = sqlx::query_scalar::<_, i64>(
        &state.q("SELECT COUNT(*) FROM users WHERE email = ? OR username = ?")
    )
    .bind(&req.email)
    .bind(&req.username)
    .fetch_one(&state.db)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict("Email or username already exists".to_string()));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?
        .to_string();

    // Create user — insert then fetch by id
    let new_id = Uuid::new_v4();
    sqlx::query(
        &state.q("INSERT INTO users (id, email, username, display_name, password_hash) VALUES (?, ?, ?, ?, ?)")
    )
    .bind(new_id.to_string())
    .bind(&req.email)
    .bind(&req.username)
    .bind(&req.display_name)
    .bind(&password_hash)
    .execute(&state.db)
    .await?;

    let user = get_user_by_id(state, new_id).await?;

    // Create default preferences
    sqlx::query(&state.q("INSERT INTO user_preferences (user_id) VALUES (?) ON CONFLICT DO NOTHING"))
        .bind(new_id.to_string())
        .execute(&state.db)
        .await?;

    // Generate tokens
    let user_uuid = parse_uuid(&user.id)?;
    let access_token = jwt::create_token(
        user_uuid,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    let refresh_token = generate_refresh_token(state, user_uuid).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user_uuid,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: empty_to_none(user.avatar_url),
            bio: empty_to_none(user.bio),
        },
    })
}

pub async fn login(state: &AppState, req: LoginRequest) -> Result<AuthResponse, AppError> {
    let user = sqlx::query_as::<_, User>(&state.q("SELECT id, email, username, display_name, COALESCE(password_hash,'') AS password_hash, COALESCE(avatar_url,'') AS avatar_url, COALESCE(bio,'') AS bio, is_active, is_verified, created_at, updated_at FROM users WHERE email = ?"))
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if user.is_active == 0 {
        return Err(AppError::Forbidden);
    }

    let password_hash = if user.password_hash.is_empty() {
        return Err(AppError::BadRequest(
            "Account uses OAuth login. Please use the appropriate OAuth provider.".to_string(),
        ));
    } else {
        &user.password_hash
    };

    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hash parse failed: {}", e)))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let user_uuid = parse_uuid(&user.id)?;
    let access_token = jwt::create_token(
        user_uuid,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    let refresh_token = generate_refresh_token(state, user_uuid).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user_uuid,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: empty_to_none(user.avatar_url),
            bio: empty_to_none(user.bio),
        },
    })
}

pub async fn refresh_token(state: &AppState, token: &str) -> Result<AuthResponse, AppError> {
    let token_hash = hash_token(token);

    let stored = sqlx::query_as::<_, crate::models::user::RefreshToken>(
        &state.q("SELECT id, user_id, token_hash, expires_at, created_at, COALESCE(revoked_at,'') AS revoked_at FROM refresh_tokens WHERE token_hash = ? AND revoked_at IS NULL AND expires_at > CURRENT_TIMESTAMP")
    )
    .bind(&token_hash)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    // Revoke old token
    sqlx::query(&state.q("UPDATE refresh_tokens SET revoked_at = CURRENT_TIMESTAMP WHERE id = ?"))
        .bind(&stored.id)
        .execute(&state.db)
        .await?;

    let user_uuid = parse_uuid(&stored.user_id)?;
    let user = get_user_by_id(state, user_uuid).await?;

    let user_uuid2 = parse_uuid(&user.id)?;
    let access_token = jwt::create_token(
        user_uuid2,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    let new_refresh_token = generate_refresh_token(state, user_uuid2).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user_uuid2,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: empty_to_none(user.avatar_url),
            bio: empty_to_none(user.bio),
        },
    })
}

pub async fn logout(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(&state.q("UPDATE refresh_tokens SET revoked_at = CURRENT_TIMESTAMP WHERE user_id = ? AND revoked_at IS NULL"))
        .bind(user_id.to_string())
        .execute(&state.db)
        .await?;
    Ok(())
}

pub async fn get_user_by_id(state: &AppState, user_id: Uuid) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(&state.q("SELECT id, email, username, display_name, COALESCE(password_hash,'') AS password_hash, COALESCE(avatar_url,'') AS avatar_url, COALESCE(bio,'') AS bio, is_active, is_verified, created_at, updated_at FROM users WHERE id = ?"))
        .bind(user_id.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))
}

pub fn get_oauth_redirect_url(state: &AppState, provider: &str) -> Result<String, AppError> {
    match provider {
        "github" => {
            let url = format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user,user:email,repo",
                state.config.github_client_id,
                state.config.github_redirect_uri,
            );
            Ok(url)
        }
        "gitlab" => {
            let url = format!(
                "https://gitlab.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read_user+read_api",
                state.config.gitlab_client_id,
                state.config.gitlab_redirect_uri,
            );
            Ok(url)
        }
        "atlassian" => {
            let url = format!(
                "https://auth.atlassian.com/authorize?audience=api.atlassian.com&client_id={}&scope=read%3Ajira-work%20manage%3Ajira-project%20read%3Ajira-user&redirect_uri={}&response_type=code&prompt=consent",
                state.config.atlassian_client_id,
                state.config.atlassian_redirect_uri,
            );
            Ok(url)
        }
        _ => Err(AppError::BadRequest(format!("Unknown provider: {}", provider))),
    }
}

pub async fn handle_oauth_callback(
    state: &AppState,
    provider: &str,
    code: &str,
) -> Result<AuthResponse, AppError> {
    let (provider_user_id, email, display_name, avatar_url, access_token, refresh_token_val) =
        match provider {
            "github" => exchange_github_code(state, code).await?,
            "gitlab" => exchange_gitlab_code(state, code).await?,
            "atlassian" => exchange_atlassian_code(state, code).await?,
            _ => return Err(AppError::BadRequest(format!("Unknown provider: {}", provider))),
        };

    // Check if OAuth account already exists
    let existing_oauth = sqlx::query_as::<_, crate::models::user::OAuthAccount>(
        &state.q("SELECT id, user_id, provider, provider_user_id, access_token, COALESCE(refresh_token,'') AS refresh_token, COALESCE(token_expires_at,'') AS token_expires_at, COALESCE(scopes,'') AS scopes, created_at, updated_at FROM oauth_accounts WHERE provider = ? AND provider_user_id = ?")
    )
    .bind(provider)
    .bind(&provider_user_id)
    .fetch_optional(&state.db)
    .await?;

    let user = if let Some(oauth) = existing_oauth {
        // Update tokens
        sqlx::query(
            &state.q("UPDATE oauth_accounts SET access_token = ?, refresh_token = ? WHERE id = ?")
        )
        .bind(&access_token)
        .bind(&refresh_token_val)
        .bind(&oauth.id)
        .execute(&state.db)
        .await?;

        let uid = parse_uuid(&oauth.user_id)?;
        get_user_by_id(state, uid).await?
    } else {
        // Check if user with this email exists
        let existing_user = sqlx::query_as::<_, User>(
            &state.q("SELECT id, email, username, display_name, COALESCE(password_hash,'') AS password_hash, COALESCE(avatar_url,'') AS avatar_url, COALESCE(bio,'') AS bio, is_active, is_verified, created_at, updated_at FROM users WHERE email = ?")
        )
        .bind(&email)
        .fetch_optional(&state.db)
        .await?;

        let user = if let Some(user) = existing_user {
            user
        } else {
            let username = email.split('@').next().unwrap_or("user").to_string();
            let unique_username = format!("{}_{}", username, &Uuid::new_v4().to_string()[..8]);
            let new_id = Uuid::new_v4();

            sqlx::query(
                &state.q("INSERT INTO users (id, email, username, display_name, avatar_url, is_verified) VALUES (?, ?, ?, ?, ?, 1)")
            )
            .bind(new_id.to_string())
            .bind(&email)
            .bind(&unique_username)
            .bind(&display_name)
            .bind(&avatar_url)
            .execute(&state.db)
            .await?;

            get_user_by_id(state, new_id).await?
        };

        let uid = parse_uuid(&user.id)?;

        // Create OAuth account link
        sqlx::query(
            &state.q("INSERT INTO oauth_accounts (id, user_id, provider, provider_user_id, access_token, refresh_token) VALUES (?, ?, ?, ?, ?, ?)")
        )
        .bind(Uuid::new_v4().to_string())
        .bind(uid.to_string())
        .bind(provider)
        .bind(&provider_user_id)
        .bind(&access_token)
        .bind(&refresh_token_val)
        .execute(&state.db)
        .await?;

        // Create preferences if new user
        sqlx::query(&state.q("INSERT INTO user_preferences (user_id) VALUES (?) ON CONFLICT DO NOTHING"))
            .bind(uid.to_string())
            .execute(&state.db)
            .await?;

        user
    };

    let user_uuid = parse_uuid(&user.id)?;
    let jwt_token = jwt::create_token(
        user_uuid,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    let refresh = generate_refresh_token(state, user_uuid).await?;

    Ok(AuthResponse {
        access_token: jwt_token,
        refresh_token: refresh,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user_uuid,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: empty_to_none(user.avatar_url),
            bio: empty_to_none(user.bio),
        },
    })
}

async fn exchange_github_code(
    state: &AppState,
    code: &str,
) -> Result<(String, String, String, Option<String>, String, Option<String>), AppError> {
    let client = reqwest::Client::new();

    let token_res: serde_json::Value = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": state.config.github_client_id,
            "client_secret": state.config.github_client_secret,
            "code": code,
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub token exchange failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub token parse failed: {}", e)))?;

    let access_token = token_res["access_token"]
        .as_str()
        .ok_or(AppError::Internal(anyhow::anyhow!("No access token in GitHub response")))?
        .to_string();

    let user_res: serde_json::Value = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "Kanva")
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub user fetch failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub user parse failed: {}", e)))?;

    let provider_user_id = user_res["id"].to_string();
    let display_name = user_res["name"]
        .as_str()
        .or(user_res["login"].as_str())
        .unwrap_or("GitHub User")
        .to_string();
    let avatar_url = user_res["avatar_url"].as_str().map(String::from);

    // Fetch email
    let email = if let Some(email) = user_res["email"].as_str() {
        email.to_string()
    } else {
        let emails_res: Vec<serde_json::Value> = client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Kanva")
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub emails fetch failed: {}", e)))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub emails parse failed: {}", e)))?;

        emails_res
            .iter()
            .find(|e| e["primary"].as_bool() == Some(true))
            .or(emails_res.first())
            .and_then(|e| e["email"].as_str())
            .ok_or(AppError::Internal(anyhow::anyhow!("No email found for GitHub user")))?
            .to_string()
    };

    Ok((provider_user_id, email, display_name, avatar_url, access_token, None))
}

async fn exchange_gitlab_code(
    state: &AppState,
    code: &str,
) -> Result<(String, String, String, Option<String>, String, Option<String>), AppError> {
    let client = reqwest::Client::new();

    let token_res: serde_json::Value = client
        .post("https://gitlab.com/oauth/token")
        .json(&serde_json::json!({
            "client_id": state.config.gitlab_client_id,
            "client_secret": state.config.gitlab_client_secret,
            "code": code,
            "grant_type": "authorization_code",
            "redirect_uri": state.config.gitlab_redirect_uri,
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitLab token exchange failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitLab token parse failed: {}", e)))?;

    let access_token = token_res["access_token"]
        .as_str()
        .ok_or(AppError::Internal(anyhow::anyhow!("No access token in GitLab response")))?
        .to_string();
    let refresh_token = token_res["refresh_token"].as_str().map(String::from);

    let user_res: serde_json::Value = client
        .get("https://gitlab.com/api/v4/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitLab user fetch failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitLab user parse failed: {}", e)))?;

    let provider_user_id = user_res["id"].to_string();
    let email = user_res["email"]
        .as_str()
        .ok_or(AppError::Internal(anyhow::anyhow!("No email in GitLab response")))?
        .to_string();
    let display_name = user_res["name"]
        .as_str()
        .unwrap_or("GitLab User")
        .to_string();
    let avatar_url = user_res["avatar_url"].as_str().map(String::from);

    Ok((provider_user_id, email, display_name, avatar_url, access_token, refresh_token))
}

async fn exchange_atlassian_code(
    state: &AppState,
    code: &str,
) -> Result<(String, String, String, Option<String>, String, Option<String>), AppError> {
    let client = reqwest::Client::new();

    let token_res: serde_json::Value = client
        .post("https://auth.atlassian.com/oauth/token")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": state.config.atlassian_client_id,
            "client_secret": state.config.atlassian_client_secret,
            "code": code,
            "redirect_uri": state.config.atlassian_redirect_uri,
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Atlassian token exchange failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Atlassian token parse failed: {}", e)))?;

    let access_token = token_res["access_token"]
        .as_str()
        .ok_or(AppError::Internal(anyhow::anyhow!("No access token in Atlassian response")))?
        .to_string();
    let refresh_token = token_res["refresh_token"].as_str().map(String::from);

    let user_res: serde_json::Value = client
        .get("https://api.atlassian.com/me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Atlassian user fetch failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Atlassian user parse failed: {}", e)))?;

    let provider_user_id = user_res["account_id"]
        .as_str()
        .ok_or(AppError::Internal(anyhow::anyhow!("No account_id in Atlassian response")))?
        .to_string();
    let email = user_res["email"]
        .as_str()
        .ok_or(AppError::Internal(anyhow::anyhow!("No email in Atlassian response")))?
        .to_string();
    let display_name = user_res["name"]
        .as_str()
        .unwrap_or("Atlassian User")
        .to_string();
    let avatar_url = user_res["picture"].as_str().map(String::from);

    Ok((provider_user_id, email, display_name, avatar_url, access_token, refresh_token))
}

async fn generate_refresh_token(state: &AppState, user_id: Uuid) -> Result<String, AppError> {
    use rand::Rng;
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let token_hash = hash_token(&token);
    let expires_at = chrono::Utc::now()
        + chrono::Duration::days(state.config.refresh_token_expiry_days);

    sqlx::query(
        &state.q("INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at) VALUES (?, ?, ?, ?)")
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .bind(&token_hash)
    .bind(expires_at.to_rfc3339())
    .execute(&state.db)
    .await?;

    Ok(token)
}

fn hash_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
