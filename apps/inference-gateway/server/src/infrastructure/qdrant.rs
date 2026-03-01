use anyhow::Result;
use qdrant_client::Qdrant;

use crate::domain::entity::Issue;
use crate::domain::repository::SearchRepository;

pub struct QdrantSearch {
    _client: Qdrant,
    collection_name: String,
}

impl QdrantSearch {
    pub fn new(client: Qdrant, collection_name: String) -> Self {
        Self {
            _client: client,
            collection_name,
        }
    }

    // A real implementation would convert `query` to a dense vector using an embedding model.
    // Here we simulate the embedding for demonstration.
}

#[async_trait::async_trait]
impl SearchRepository for QdrantSearch {
    #[tracing::instrument(name = "Qdrant Vector Search", skip(self))]
    async fn search_issues(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<Issue>> {
        use qdrant_client::qdrant::SearchPoints;

        let search_request = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: query_vector,
            limit: limit as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let result = self._client.search_points(search_request).await?;
        let mut issues = Vec::new();

        for point in result.result {
            let id = match point.id {
                Some(p_id) => match p_id.point_id_options {
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(n)) => n.to_string(),
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(s)) => s,
                    None => "".to_string(),
                },
                None => "".to_string(),
            };

            let problem = point
                .payload
                .get("problem")
                .and_then(|v| match &v.kind {
                    Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let solution = point
                .payload
                .get("solution")
                .and_then(|v| match &v.kind {
                    Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            issues.push(Issue {
                id,
                problem,
                solution,
            });
        }

        Ok(issues)
    }

    async fn upsert_issues(&self, _issues: &[(Issue, Vec<f32>)]) -> Result<()> {
        // Implement if server needs to sync back, but for now we have a separate ingest tool.
        // We'll leave it as a no-op or return an error if not expected here.
        Ok(())
    }
}
