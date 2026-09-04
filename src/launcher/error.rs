use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("this launcher is not built for a Mojang-supported platform")]
    UnsupportedPlatform,
    #[error("Mojang metadata does not contain {kind} `{key}` for platform `{platform}`")]
    MissingMetadata {
        kind: &'static str,
        key: String,
        platform: String,
    },
    #[error("unsafe path in Mojang metadata: {0}")]
    UnsafePath(String),
    #[error("download integrity check failed for {path}: expected {expected}, got {actual}")]
    Integrity {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON from Mojang: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid native library archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("cannot determine an OS-appropriate application data directory")]
    DataDirectory,
}

pub type Result<T> = std::result::Result<T, Error>;
