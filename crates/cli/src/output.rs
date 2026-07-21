use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn format_output<T: Serialize>(data: &T, _format: &OutputFormat) -> String {
    serde_json::to_string_pretty(data).unwrap_or_default()
}
