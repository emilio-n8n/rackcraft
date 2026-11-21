//! Engine error type.

use thiserror::Error;

/// Errors produced by the pure document engine.
#[derive(Debug, Error)]
pub enum EngineError {
    /// The JSON document could not be serialized or deserialized.
    #[error("document JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// The document was produced by an incompatible future schema.
    #[error("unsupported document schema {found}; this build supports {supported}")]
    UnsupportedSchema {
        /// Version found in the file.
        found: u32,
        /// Newest version understood by this build.
        supported: u32,
    },
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, EngineError>;
