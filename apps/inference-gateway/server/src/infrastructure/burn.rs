use anyhow::{Result, anyhow};
use ndarray::Array2;
use ort::session::Session;
use ort::value::Value;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;

use crate::domain::entity::{Issue, RankedIssue};
use crate::domain::repository::RankingRepository;

pub struct BurnRanker {
    session: Arc<Mutex<Session>>,
    tokenizer: Tokenizer,
}

impl BurnRanker {
    pub fn new(model_path: &str, tokenizer_path: &str) -> Result<Self> {
        let session = Session::builder()?.commit_from_file(model_path)?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            tokenizer,
        })
    }
}

#[async_trait::async_trait]
impl RankingRepository for BurnRanker {
    #[tracing::instrument(name = "Burn ONNX Ranking", skip(self, issues))]
    async fn rank_issues(&self, query: &str, issues: Vec<Issue>) -> Result<Vec<RankedIssue>> {
        let mut ranked = Vec::with_capacity(issues.len());

        for issue in issues {
            let body = issue.body.as_deref().unwrap_or("");
            let context = format!("{}\n{}", issue.title, body);
            let encoding = self
                .tokenizer
                .encode((query.to_string(), context), true)
                .map_err(|e| anyhow!("Tokenization error: {}", e))?;

            let input_ids = encoding.get_ids();
            let attention_mask = encoding.get_attention_mask();
            let seq_len = input_ids.len();

            let input_ids_array = Array2::from_shape_vec(
                (1, seq_len),
                input_ids.iter().map(|&x| x as i64).collect(),
            )?;
            let attention_mask_array = Array2::from_shape_vec(
                (1, seq_len),
                attention_mask.iter().map(|&x| x as i64).collect(),
            )?;

            let input_ids_value = Value::from_array((
                vec![1, seq_len],
                input_ids_array.into_raw_vec_and_offset().0,
            ))?;
            let attention_mask_value = Value::from_array((
                vec![1, seq_len],
                attention_mask_array.into_raw_vec_and_offset().0,
            ))?;

            // In ort 2.0 rc.11, the macro might return Result or Vec depending on features or specific version.
            // If it returns Vec, we don't use ?.
            // The error said "the ? operator cannot be applied to type Vec".
            // So we remove the ? from the macro result.
            let session_inputs = ort::inputs![
                "input_ids" => input_ids_value,
                "attention_mask" => attention_mask_value,
            ];

            let mut session = self
                .session
                .lock()
                .map_err(|_| anyhow!("Failed to lock session"))?;
            let outputs = session.run(session_inputs)?;

            // Cross-encoders usually output a single logit/score at index 0
            let output_tensor = outputs
                .get("logits")
                .ok_or_else(|| anyhow!("Failed to find 'logits' output"))?;

            let logits = output_tensor.try_extract_tensor::<f32>()?;
            // logits is (&Shape, &[f32]) in this version
            let score = logits.1[0];

            ranked.push(RankedIssue { issue, score });
        }

        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ranked)
    }
}
