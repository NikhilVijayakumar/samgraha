use anyhow::Result;
use schemas::enrichment::{EnrichmentArtifact, EnrichmentType};
use schemas::document::Document;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SummaryOutput {
    pub summary: String,
    pub key_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordsOutput {
    pub keywords: Vec<String>,
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingOutput {
    pub vector: Vec<f64>,
    pub dimensions: usize,
}

#[derive(Debug, Clone)]
pub struct GlossaryOutput {
    pub term: String,
    pub definition: String,
}

pub trait EnrichmentProvider: Send + Sync {
    fn name(&self) -> &str;

    fn summarize(&self, document: &Document) -> Result<SummaryOutput>;

    fn keywords(&self, document: &Document) -> Result<KeywordsOutput>;

    fn embed(&self, document: &Document) -> Result<EmbeddingOutput>;

    fn glossary(&self, documents: &[Document]) -> Result<Vec<GlossaryOutput>>;

    fn enrich(&self, document: &Document, enrichment_type: &EnrichmentType) -> Result<EnrichmentArtifact> {
        match enrichment_type {
            EnrichmentType::Summary => {
                let out = self.summarize(document)?;
                Ok(EnrichmentArtifact {
                    document_id: document.id,
                    artifact_type: EnrichmentType::Summary,
                    content: serde_json::to_string(&out)?,
                    provider: self.name().to_string(),
                    model: None,
                    created_at: chrono_now(),
                })
            }
            EnrichmentType::Keywords => {
                let out = self.keywords(document)?;
                Ok(EnrichmentArtifact {
                    document_id: document.id,
                    artifact_type: EnrichmentType::Keywords,
                    content: serde_json::to_string(&out)?,
                    provider: self.name().to_string(),
                    model: None,
                    created_at: chrono_now(),
                })
            }
            EnrichmentType::Embedding => {
                let out = self.embed(document)?;
                Ok(EnrichmentArtifact {
                    document_id: document.id,
                    artifact_type: EnrichmentType::Embedding,
                    content: serde_json::to_string(&out)?,
                    provider: self.name().to_string(),
                    model: None,
                    created_at: chrono_now(),
                })
            }
            EnrichmentType::Glossary => {
                anyhow::bail!("Glossary enrichment requires batch processing")
            }
        }
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

pub type BoxedProvider = Box<dyn EnrichmentProvider>;
