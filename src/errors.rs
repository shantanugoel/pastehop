use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasteHopError {
    #[error("attach requires one or more paths or --clipboard")]
    MissingAttachInput,
    #[error("could not resolve a remote target; pass --host")]
    MissingTarget,
    #[error("host '{host}' is not approved for uploads")]
    HostDenied { host: String },
    #[error("unable to confirm first upload to '{host}' in a non-interactive session")]
    HostConfirmationUnavailable { host: String },
    #[error("too many files requested: {actual} > {limit}")]
    TooManyFiles { limit: usize, actual: usize },
    #[error("local path is not a file: {0}")]
    InvalidLocalPath(PathBuf),
    #[error("failed to read local path {path}: {source}")]
    ReadLocalPath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("file exceeds single-file size limit ({actual_bytes} > {limit_bytes}): {path}")]
    FileTooLarge {
        path: PathBuf,
        limit_bytes: u64,
        actual_bytes: u64,
    },
    #[error("total request size exceeds limit ({actual_bytes} > {limit_bytes})")]
    TotalSizeTooLarge { limit_bytes: u64, actual_bytes: u64 },
    #[error("clipboard is unavailable: {message}")]
    ClipboardUnavailable { message: String },
    #[error("clipboard does not currently contain an image")]
    ClipboardNotImage,
    #[error("failed to materialize clipboard content: {source}")]
    ClipboardIo {
        #[source]
        source: std::io::Error,
    },
    #[error("failed to encode clipboard image: {source}")]
    ClipboardImageEncoding {
        #[source]
        source: image::ImageError,
    },
    #[error("failed to install or uninstall integration at {path}: {source}")]
    InstallIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("required tool not found in PATH: {0}")]
    MissingTool(String),
    #[error("failed to spawn transport command {command}: {source}")]
    SpawnTransport {
        command: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("transport command failed: {command} (code: {code:?})")]
    TransportFailed { command: PathBuf, code: Option<i32> },
    #[error("hook output failed to serialize")]
    HookSerialization(#[source] serde_json::Error),
    #[error("failed to read config at {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config at {path}: {source}")]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to write config at {path}: {source}")]
    WriteConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to encode config at {path}: {source}")]
    EncodeConfig {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
}

impl PasteHopError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::MissingAttachInput => 2,
            Self::MissingTarget => 2,
            Self::HostDenied { .. } | Self::HostConfirmationUnavailable { .. } => 6,
            Self::TooManyFiles { .. }
            | Self::InvalidLocalPath(_)
            | Self::ReadLocalPath { .. }
            | Self::FileTooLarge { .. }
            | Self::TotalSizeTooLarge { .. }
            | Self::ClipboardUnavailable { .. }
            | Self::ClipboardNotImage
            | Self::ClipboardIo { .. }
            | Self::ClipboardImageEncoding { .. } => 2,
            Self::InstallIo { .. } => 4,
            Self::MissingTool(_) | Self::SpawnTransport { .. } | Self::TransportFailed { .. } => 7,
            Self::HookSerialization(_) => 3,
            Self::ReadConfig { .. }
            | Self::ParseConfig { .. }
            | Self::WriteConfig { .. }
            | Self::EncodeConfig { .. } => 4,
        }
    }
}
