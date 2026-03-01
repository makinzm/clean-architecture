use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    #[serde(rename = "number")]
    pub id: i64,
    pub title: String,
    pub body: Option<String>,
}
