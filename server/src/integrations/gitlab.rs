// GitLab integration helpers

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabWebhookPayload {
    pub object_kind: String,
    pub object_attributes: Option<serde_json::Value>,
    pub project: Option<GitLabProject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: i64,
    pub path_with_namespace: String,
    pub web_url: String,
}
