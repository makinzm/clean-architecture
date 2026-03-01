use crate::domain::repository::{
    EmbeddingRepository, FileRepository, IngestionStatus, JobRepository, SearchRepository,
};
use anyhow::Result;
use chrono::Local;
use std::sync::Arc;
use std::time::Instant;

fn is_ollama_context_length_error(err: &anyhow::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("exceeds the context length")
        || msg.contains("context length")
        || msg.contains("input length")
}

async fn embed_with_truncation_fallback(
    embed_repo: &dyn EmbeddingRepository,
    input: &str,
) -> anyhow::Result<Vec<f32>> {
    // Try progressively smaller inputs only when Ollama complains about context length.
    // This keeps ingestion moving even if some issues are extremely long.
    const LIMITS: [usize; 6] = [500, 320, 200, 120, 80, 40];

    let mut last_err: Option<anyhow::Error> = None;
    for limit in LIMITS {
        let truncated: String = input.chars().take(limit).collect();
        match embed_repo.embed_text(&truncated).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if is_ollama_context_length_error(&e) {
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Embedding failed")))
}

pub struct IngestUsecase {
    file_repo: Arc<dyn FileRepository>,
    embed_repo: Arc<dyn EmbeddingRepository>,
    search_repo: Arc<dyn SearchRepository>,
    job_repo: Arc<dyn JobRepository>,
    embed_model: String,
    commit_hash: String,
}

impl IngestUsecase {
    pub fn new(
        file_repo: Arc<dyn FileRepository>,
        embed_repo: Arc<dyn EmbeddingRepository>,
        search_repo: Arc<dyn SearchRepository>,
        job_repo: Arc<dyn JobRepository>,
        embed_model: String,
        commit_hash: String,
    ) -> Self {
        Self {
            file_repo,
            embed_repo,
            search_repo,
            job_repo,
            embed_model,
            commit_hash,
        }
    }

    pub async fn execute(&self, job_id: String, path: &str) -> Result<()> {
        use futures::{StreamExt, stream};

        let start_time = Instant::now();
        let mut last_report_time = Instant::now();
        let mut last_report_count = 0;

        // 1. Initial report (Start)
        self.job_repo
            .record_status(IngestionStatus {
                job_id: job_id.clone(),
                total_count: 0,
                processed_count: 0,
                elapsed_seconds: 0.0,
                throughput_rps: 0.0,
                percentage: 0.0,
                timestamp: Local::now().to_rfc3339(),
                is_completed: false,
                embedding_model: self.embed_model.clone(),
                commit_hash: self.commit_hash.clone(),
            })
            .await?;

        let issues = self.file_repo.read_issues(path)?;
        let total_count = issues.len();

        // 2. Report total count as soon as known
        self.job_repo
            .record_status(IngestionStatus {
                job_id: job_id.clone(),
                total_count,
                processed_count: 0,
                elapsed_seconds: start_time.elapsed().as_secs_f64(),
                throughput_rps: 0.0,
                percentage: 0.0,
                timestamp: Local::now().to_rfc3339(),
                is_completed: false,
                embedding_model: self.embed_model.clone(),
                commit_hash: self.commit_hash.clone(),
            })
            .await?;

        let batch_size = 100;
        let concurrency_limit = 50;
        let mut processed_count = 0;
        let mut data_to_upsert = Vec::with_capacity(batch_size);

        // Reporting thresholds
        let time_threshold = std::time::Duration::from_secs(30);
        let count_threshold = 2000;

        // Create a stream of embedding tasks
        let mut stream = stream::iter(issues)
            .enumerate()
            .map(|(i, issue)| {
                let embed_repo = self.embed_repo.clone();
                async move {
                    let body = issue.body.as_deref().unwrap_or("");
                    let input = format!("{}\n{}", issue.title, body);
                    let res = embed_with_truncation_fallback(embed_repo.as_ref(), &input).await;
                    (i, issue, res)
                }
            })
            .buffer_unordered(concurrency_limit);

        while let Some((i, issue, res)) = stream.next().await {
            match res {
                Ok(embedding) => {
                    data_to_upsert.push((issue, embedding));
                    processed_count += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to embed issue (index: {}): {}", i, e);
                    continue;
                }
            }

            // Periodic progress report (every 30s or 2000 issues)
            let now = Instant::now();
            let count_since_report = processed_count - last_report_count;
            if now.duration_since(last_report_time) >= time_threshold
                || count_since_report >= count_threshold
            {
                let elapsed = start_time.elapsed().as_secs_f64();
                let throughput = if elapsed > 0.0 {
                    processed_count as f64 / elapsed
                } else {
                    0.0
                };
                let percentage = if total_count > 0 {
                    (processed_count as f64 / total_count as f64) * 100.0
                } else {
                    0.0
                };

                self.job_repo
                    .record_status(IngestionStatus {
                        job_id: job_id.clone(),
                        total_count,
                        processed_count,
                        elapsed_seconds: elapsed,
                        throughput_rps: throughput,
                        percentage,
                        timestamp: Local::now().to_rfc3339(),
                        is_completed: false,
                        embedding_model: self.embed_model.clone(),
                        commit_hash: self.commit_hash.clone(),
                    })
                    .await?;

                last_report_time = now;
                last_report_count = processed_count;
            }

            if data_to_upsert.len() >= batch_size {
                tracing::info!(
                    "Upserting batch (total processed so far: {})...",
                    processed_count
                );
                self.search_repo.upsert_issues(&data_to_upsert).await?;
                data_to_upsert.clear();

                // Check for stop signal
                if self.job_repo.should_stop(&job_id).await? {
                    tracing::warn!(
                        "Stop signal detected for job {}. Stopping gracefully...",
                        job_id
                    );
                    return Ok(());
                }
            }
        }

        if !data_to_upsert.is_empty() {
            tracing::info!(
                "Upserting final batch of {} issues...",
                data_to_upsert.len()
            );
            self.search_repo.upsert_issues(&data_to_upsert).await?;
        }

        let elapsed = start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let throughput = if elapsed_secs > 0.0 {
            processed_count as f64 / elapsed_secs
        } else {
            0.0
        };
        let percentage = if total_count > 0 {
            (processed_count as f64 / total_count as f64) * 100.0
        } else {
            100.0
        };

        let status = IngestionStatus {
            job_id,
            total_count,
            processed_count,
            elapsed_seconds: elapsed_secs,
            throughput_rps: throughput,
            percentage,
            timestamp: Local::now().to_rfc3339(),
            is_completed: true,
            embedding_model: self.embed_model.clone(),
            commit_hash: self.commit_hash.clone(),
        };

        self.job_repo.record_status(status).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Issue;
    use crate::domain::repository::{
        MockEmbeddingRepository, MockFileRepository, MockJobRepository, MockSearchRepository,
    };

    #[tokio::test]
    async fn test_ingest_usecase() {
        let mut mock_file = MockFileRepository::new();
        let mut mock_embed = MockEmbeddingRepository::new();
        let mut mock_search = MockSearchRepository::new();

        let issues = vec![Issue {
            id: 1,
            title: "t1".into(),
            body: Some("b1".into()),
        }];

        mock_file
            .expect_read_issues()
            .returning(move |_| Ok(issues.clone()));
        mock_embed
            .expect_embed_text()
            .returning(|_| Ok(vec![0.1; 128]));
        mock_search.expect_upsert_issues().returning(|_| Ok(()));

        let mut mock_job = MockJobRepository::new();
        // Expect THREE calls:
        // 1. Initial start (total=0)
        // 2. Count known (total=1)
        // 3. Final completion (total=1, processed=1)
        mock_job
            .expect_record_status()
            .times(3)
            .returning(|_| Ok(()));
        mock_job.expect_should_stop().returning(|_| Ok(false));

        let usecase = IngestUsecase::new(
            Arc::new(mock_file),
            Arc::new(mock_embed),
            Arc::new(mock_search),
            Arc::new(mock_job),
        );

        let result = usecase.execute("job1".into(), "fake_path").await;
        assert!(result.is_ok());
    }
}
