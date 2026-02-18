use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiffusionError {
    #[error("Model loading failed: {0}")]
    ModelLoad(String),
    
    #[error("Inference failed: {0}")]
    Inference(String),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Queue full")]
    QueueFull,
    
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, DiffusionError>;
