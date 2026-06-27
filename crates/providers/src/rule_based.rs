use crate::traits::{
    EmbeddingOutput, EnrichmentProvider, GlossaryOutput, KeywordsOutput, SummaryOutput,
};
use anyhow::Result;
use schemas::document::Document;

pub struct RuleBasedProvider;

impl RuleBasedProvider {
    pub fn new() -> Self {
        Self
    }
}

impl EnrichmentProvider for RuleBasedProvider {
    fn name(&self) -> &str {
        "rule-based"
    }

    fn summarize(&self, document: &Document) -> Result<SummaryOutput> {
        let body = document.body.raw();
        let first_line = body.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        let summary = if first_line.starts_with('#') {
            first_line.trim_start_matches('#').trim().to_string()
        } else {
            truncate(body, 200)
        };
        Ok(SummaryOutput {
            summary,
            key_points: extract_key_points(body),
        })
    }

    fn keywords(&self, document: &Document) -> Result<KeywordsOutput> {
        let mut words: Vec<String> = document
            .body
            .raw()
            .split_whitespace()
            .filter(|w| w.len() > 4)
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| !STOP_WORDS.contains(&w.as_str()))
            .collect();
        words.sort();
        words.dedup();
        words.truncate(20);
        Ok(KeywordsOutput {
            keywords: words,
            relevance: 1.0,
        })
    }

    fn embed(&self, _document: &Document) -> Result<EmbeddingOutput> {
        Ok(EmbeddingOutput {
            vector: vec![],
            dimensions: 0,
        })
    }

    fn glossary(&self, documents: &[Document]) -> Result<Vec<GlossaryOutput>> {
        let mut terms = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for doc in documents {
            for line in doc.body.raw().lines() {
                let lower = line.trim().to_lowercase();
                if lower.starts_with("### ") || lower.starts_with("#### ") {
                    let term = line.trim_start_matches('#').trim().to_string();
                    if !seen.contains(&term) {
                        seen.insert(term.clone());
                        terms.push(GlossaryOutput {
                            term,
                            definition: String::new(),
                        });
                    }
                }
            }
        }
        Ok(terms)
    }
}

fn extract_key_points(body: &str) -> Vec<String> {
    body.lines()
        .filter(|l| l.starts_with("- ") || l.starts_with("* "))
        .take(5)
        .map(|l| {
            l.trim_start_matches("- ")
                .trim_start_matches("* ")
                .to_string()
        })
        .collect()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

const STOP_WORDS: &[&str] = &[
    "this", "that", "with", "from", "have", "been", "will", "would", "should", "their", "there",
    "which", "about", "into", "could", "other", "after", "first", "second", "third", "shall",
    "must", "each", "than", "then",
];
