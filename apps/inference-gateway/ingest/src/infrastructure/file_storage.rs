use crate::domain::entity::Issue;
use crate::domain::repository::FileRepository;
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct JsonlFileRepo;

impl FileRepository for JsonlFileRepo {
    fn read_issues(&self, path: &str) -> Result<Vec<Issue>> {
        let file = File::open(path).map_err(|e| {
            let cwd = std::env::current_dir().ok();
            anyhow::anyhow!(
                "Failed to open file at '{}': {} (Current directory: {:?})",
                path,
                e,
                cwd
            )
        })?;
        let reader = BufReader::new(file);
        let mut issues = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(&line)?;
            issues.push(issue);
        }

        Ok(issues)
    }
}
