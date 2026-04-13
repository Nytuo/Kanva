use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Board {
    pub id: String,
    pub title: String,
    pub description: String, // COALESCE(description, '')
    pub visibility: String,
    pub background_color: String,     // COALESCE(background_color, '')
    pub background_image_url: String, // COALESCE(background_image_url, '')
    pub is_starred: i64,
    pub is_archived: i64,
    pub owner_id: String,
    pub team_id: String, // COALESCE(team_id, '')
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BoardMember {
    pub id: String,
    pub board_id: String,
    pub user_id: String,
    pub role: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct List {
    pub id: String,
    pub board_id: String,
    pub title: String,
    pub position: i32,
    pub is_archived: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Label {
    pub id: String,
    pub board_id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}
