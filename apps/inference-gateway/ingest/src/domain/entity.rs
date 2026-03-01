use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub repo_name: String,
    pub html_url: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
}
