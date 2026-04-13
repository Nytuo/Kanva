use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Helper: convert empty string (from COALESCE) back to Option<String>
pub fn empty_to_none(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String, // COALESCE(password_hash, '')
    pub avatar_url: String,    // COALESCE(avatar_url, '')
    pub bio: String,           // COALESCE(bio, '')
    pub is_active: i64,
    pub is_verified: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OAuthAccount {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: String,    // COALESCE(refresh_token, '')
    pub token_expires_at: String, // COALESCE(token_expires_at, '')
    pub scopes: String,           // COALESCE(scopes, '')
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
    pub created_at: String,
    pub revoked_at: String, // COALESCE(revoked_at, '')
}
