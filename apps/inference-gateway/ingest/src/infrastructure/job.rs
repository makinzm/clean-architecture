use crate::domain::repository::{IngestionStatus, JobRepository};
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;

pub struct FileJobRepository {
    status_dir: PathBuf,
}

impl FileJobRepository {
    pub fn new(status_dir: PathBuf) -> Self {
        Self { status_dir }
    }
}

#[async_trait]
impl JobRepository for FileJobRepository {
    async fn record_status(&self, status: IngestionStatus) -> Result<()> {
        if !self.status_dir.exists() {
            fs::create_dir_all(&self.status_dir)?;
        }

        let file_path = self.status_dir.join(format!("job_{}.jsonl", status.job_id));

        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let json = serde_json::to_string(&status)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    async fn should_stop(&self, job_id: &str) -> Result<bool> {
        let stop_file = self.status_dir.join(format!("stop_{}", job_id));
        Ok(stop_file.exists())
    }
}
