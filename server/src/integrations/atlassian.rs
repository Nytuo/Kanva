// Atlassian/Jira integration helpers

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraWebhookPayload {
    pub webhook_event: String,
    pub issue: Option<JiraIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub fields: JiraIssueFields,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssueFields {
    pub summary: String,
    pub description: Option<serde_json::Value>,
    pub status: JiraStatus,
    pub priority: Option<JiraPriority>,
    pub assignee: Option<JiraUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraPriority {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraUser {
    pub account_id: String,
    pub display_name: String,
}
