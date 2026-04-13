use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Card {
    pub id: String,
    pub list_id: String,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub priority: String,
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub completed_at: Option<String>,
    pub is_archived: i64,
    pub cover_color: Option<String>,
    pub cover_image_url: Option<String>,
    pub estimated_hours: Option<String>,
    pub actual_hours: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: String,
    pub card_id: String,
    pub user_id: String,
    pub content: String,
    pub edited_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Checklist {
    pub id: String,
    pub card_id: String,
    pub title: String,
    pub position: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChecklistItem {
    pub id: String,
    pub checklist_id: String,
    pub title: String,
    pub is_checked: i64,
    pub position: i32,
    pub assigned_to: Option<String>,
    pub due_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: String,
    pub card_id: String,
    pub user_id: String,
    pub filename: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub created_at: String,
}
