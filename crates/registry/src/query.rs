use anyhow::Result;
use schemas::search::{SearchQuery, SearchResult, SearchResponse, RetrievalLevel};
use crate::store::RegistryStore;

impl RegistryStore {
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResponse> {
        let start = std::time::Instant::now();
        let all_docs = self.get_all_documents()?;
        let query_lower = query.query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SearchResult> = all_docs
            .iter()
            .filter(|doc| {
                if let Some(ref domain) = query.domain {
                    if doc.standard != *domain {
                        return false;
                    }
                }
                true
            })
            .filter_map(|doc| {
                let body_lower = doc.body.raw().to_lowercase();
                let title_lower = doc.title.to_lowercase();

                let mut score = 0.0;
                for term in &query_terms {
                    if title_lower.contains(term) {
                        score += 10.0;
                    }
                    let count = body_lower.matches(term).count();
                    score += count as f64;
                }

                if score == 0.0 && !query_terms.is_empty() {
                    return None;
                }

                let snippet = match query.level {
                    RetrievalLevel::Metadata => None,
                    RetrievalLevel::Summary => {
                        Some(doc.body.raw().lines().find(|l| !l.trim().is_empty()).unwrap_or("").to_string())
                    }
                    _ => doc.body.raw().lines()
                        .find(|l| query_terms.iter().any(|t| l.to_lowercase().contains(t)))
                        .map(|l| l.trim().to_string()),
                };

                Some(SearchResult {
                    document_id: doc.id,
                    path: doc.path.as_str().to_string(),
                    title: doc.title.clone(),
                    domain: doc.standard.clone(),
                    score,
                    snippet,
                    metadata: std::collections::HashMap::new(),
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(query.max_results);

        let duration = start.elapsed();
        Ok(SearchResponse {
            total_count: results.len(),
            results,
            query: query.query.clone(),
            duration_ms: duration.as_millis() as u64,
        })
    }
}
