use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path `{0}` is outside repository root `{1}`")]
    PathOutsideRoot(PathBuf, PathBuf),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),
}
