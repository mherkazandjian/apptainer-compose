use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    ComposeFile(#[from] ComposeFileError),

    #[error("{0}")]
    Apptainer(#[from] ApptainerError),

    #[error("{0}")]
    State(#[from] StateError),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum ComposeFileError {
    #[error("compose file not found: {path}")]
    NotFound { path: PathBuf },

    #[error("parse error in {file}: {message}")]
    ParseError { file: PathBuf, message: String },

    #[error("service '{service}' references unknown network '{network}'")]
    UnknownNetwork { service: String, network: String },

    #[error("service '{service}' references unknown volume '{volume}'")]
    UnknownVolume { service: String, volume: String },

    #[error("circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },

    #[error("service '{service}' has neither 'image' nor 'build' specified")]
    NoImageOrBuild { service: String },

    #[error("unsupported feature '{feature}': {suggestion}")]
    UnsupportedFeature { feature: String, suggestion: String },

    #[error("variable interpolation error: {0}")]
    Interpolation(String),
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum ApptainerError {
    #[error("apptainer not found on PATH. Install from https://apptainer.org")]
    NotFound,

    #[error("apptainer command failed: {command}\n{stderr}")]
    CommandFailed { command: String, stderr: String },

    #[error("failed to pull image '{uri}': {reason}")]
    PullFailed { uri: String, reason: String },

    #[error("failed to start instance '{name}': {reason}")]
    StartFailed { name: String, reason: String },

    #[error("failed to stop instance '{name}': {reason}")]
    StopFailed { name: String, reason: String },

    #[error("instance '{name}' not found")]
    InstanceNotFound { name: String },
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum StateError {
    #[error("failed to read state file: {0}")]
    ReadFailed(String),

    #[error("failed to write state file: {0}")]
    WriteFailed(String),

    #[error("failed to acquire lock: {0}")]
    LockFailed(String),

    #[error("state file corrupted: {0}")]
    Corrupted(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
