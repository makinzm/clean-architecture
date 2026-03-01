use crate::domain::entity::Issue;
use crate::domain::repository::SearchRepository;
use anyhow::{Result, anyhow};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
};

pub struct QdrantSearch {
    client: Qdrant,
    collection_name: String,
}

impl QdrantSearch {
    pub async fn new(client: Qdrant, collection_name: String, vector_size: u64) -> Result<Self> {
        let me = Self {
            client,
            collection_name,
        };

        // Ensure collection exists, and verify the vector dimension matches.
        match me.client.collection_info(&me.collection_name).await {
            Ok(info) => {
                if let Some(existing) = extract_vector_size(&info)
                    && existing != vector_size
                {
                    return Err(anyhow!(
                        "Qdrant collection '{}' already exists but has a different vector dimension: existing={}, embed_model={}\n\
                         Fix: delete/recreate the collection (curl -X DELETE http://localhost:6333/collections/{}) or use the same OLLAMA_EMBED_MODEL as before.\n\
                         Note: make sure you are deleting the SAME Qdrant instance you ingest into (check QDRANT_URL).",
                        me.collection_name,
                        existing,
                        vector_size,
                        me.collection_name
                    ));
                }
            }
            Err(_) => {
                me.client
                    .create_collection(
                        CreateCollectionBuilder::new(&me.collection_name).vectors_config(
                            VectorParamsBuilder::new(vector_size, Distance::Cosine),
                        ),
                    )
                    .await?;
            }
        }

        Ok(me)
    }
}

fn extract_vector_size(info: &qdrant_client::qdrant::GetCollectionInfoResponse) -> Option<u64> {
    let config = info.result.as_ref()?.config.as_ref()?;
    let params = config.params.as_ref()?;
    let vectors_config = params.vectors_config.as_ref()?;

    match vectors_config.config.as_ref()? {
        qdrant_client::qdrant::vectors_config::Config::Params(p) => Some(p.size),
        qdrant_client::qdrant::vectors_config::Config::ParamsMap(m) => {
            m.map.values().next().map(|p| p.size)
        }
    }
}

#[async_trait::async_trait]
impl SearchRepository for QdrantSearch {
    async fn upsert_issues(&self, issues: &[(Issue, Vec<f32>)]) -> Result<()> {
        let mut points = Vec::new();

        for (issue, vector) in issues {
            let payload = serde_json::to_value(issue)?;
            let payload_map: std::collections::HashMap<String, qdrant_client::qdrant::Value> =
                match payload {
                    serde_json::Value::Object(map) => {
                        map.into_iter().map(|(k, v)| (k, v.into())).collect()
                    }
                    _ => return Err(anyhow::anyhow!("Invalid issue payload")),
                };

            points.push(PointStruct::new(
                issue.id as u64,
                vector.clone(),
                payload_map,
            ));
        }

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points).wait(true))
            .await?;

        Ok(())
    }
}
