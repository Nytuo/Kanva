use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String, // COALESCE(description, '')
    pub avatar_url: String,  // COALESCE(avatar_url, '')
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMember {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamInvite {
    pub id: String,
    pub team_id: String,
    pub email: String,
    pub role: String,
    pub invited_by: String,
    pub token: String,
    pub expires_at: String,
    pub accepted_at: String, // COALESCE(accepted_at, '')
    pub created_at: String,
}
