#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    pub id: String,
    pub problem: String,
    pub solution: String, // From the merged PR
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
