#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    pub point_id: u64,
    pub repo_name: String,
    pub html_url: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedIssue {
    pub issue: Issue,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub original_query: String,
    pub top_issues: Vec<RankedIssue>,
    pub llm_advice: String,
}
