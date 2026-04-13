// GitHub integration helpers
// Webhook processing, issue sync, PR sync, etc.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub issue: Option<GitHubIssue>,
    pub pull_request: Option<serde_json::Value>,
    pub repository: Option<GitHubRepo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub assignees: Vec<GitHubUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub full_name: String,
}
