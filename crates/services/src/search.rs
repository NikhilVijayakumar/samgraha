use anyhow::Result;
use schemas::document::Document;
use schemas::search::{SearchQuery, SearchResult, SearchResponse, RetrievalLevel};

pub struct SearchService;

impl SearchService {
    pub fn search(
        documents: &[Document],
        query: &SearchQuery,
    ) -> Result<SearchResponse> {
        let start = std::time::Instant::now();
        let query_lower = query.query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SearchResult> = documents
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
                let body_lower = doc.body.to_lowercase();
                let title_lower = doc.title.to_lowercase();

                let mut score = 0.0;
                for term in &query_terms {
                    if title_lower.contains(term) {
                        score += 10.0;
                    }
                    if body_lower.contains(term) {
                        score += 1.0;
                    }
                }

                if score == 0.0 && !query_terms.is_empty() {
                    return None;
                }

                if score > 0.0 || query_terms.is_empty() {
                    let snippet = match query.level {
                        RetrievalLevel::Metadata => None,
                        RetrievalLevel::Summary => Some(
                            doc.body.lines()
                                .find(|l| !l.trim().is_empty())
                                .unwrap_or("")
                                .to_string(),
                        ),
                        RetrievalLevel::Section | RetrievalLevel::Full => {
                            find_relevant_snippet(&doc.body, &query_terms)
                        }
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
                } else {
                    None
                }
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

fn find_relevant_snippet(body: &str, terms: &[&str]) -> Option<String> {
    for line in body.lines() {
        let lower = line.to_lowercase();
        if terms.iter().any(|t| lower.contains(t)) {
            return Some(line.trim().to_string());
        }
    }
    body.lines().next().map(|l| l.trim().to_string())
}
