use anyhow::Result;
use schemas::config::AiConfigSection;
use crate::traits::BoxedProvider;
use crate::RuleBasedProvider;

pub fn create_provider(config: &AiConfigSection) -> Result<BoxedProvider> {
    match config.provider.as_deref() {
        Some("rule-based") | None => Ok(Box::new(RuleBasedProvider::new())),
        Some(other) => {
            tracing::warn!("Provider '{}' not implemented, falling back to rule-based", other);
            Ok(Box::new(RuleBasedProvider::new()))
        }
    }
}

pub fn create_default_provider() -> BoxedProvider {
    Box::new(RuleBasedProvider::new())
}
