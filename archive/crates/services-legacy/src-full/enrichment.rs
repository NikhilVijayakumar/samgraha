use anyhow::Result;
use providers::traits::EnrichmentProvider;
use schemas::document::Document;
use schemas::enrichment::{EnrichmentArtifact, EnrichmentProfile};

pub struct EnrichmentService;

impl EnrichmentService {
    pub fn enrich_document(
        provider: &dyn EnrichmentProvider,
        document: &Document,
        profile: &EnrichmentProfile,
    ) -> Result<Vec<EnrichmentArtifact>> {
        let mut artifacts = Vec::new();
        for et in &profile.enabled_types {
            let artifact = provider.enrich(document, et)?;
            artifacts.push(artifact);
        }
        Ok(artifacts)
    }

    pub fn enrich_batch(
        provider: &dyn EnrichmentProvider,
        documents: &[Document],
        profile: &EnrichmentProfile,
    ) -> Result<Vec<EnrichmentArtifact>> {
        let mut all = Vec::new();
        for chunk in documents.chunks(profile.batch_size) {
            for doc in chunk {
                let artifacts = Self::enrich_document(provider, doc, profile)?;
                all.extend(artifacts);
            }
        }
        Ok(all)
    }
}
