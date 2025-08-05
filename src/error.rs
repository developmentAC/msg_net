use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Text processing error: {0}")]
    TextProcessing(String),
    
    #[error("Entity extraction error: {0}")]
    EntityExtraction(String),
    
    #[error("Graph building error: {0}")]
    GraphBuilding(String),
    
    #[error("Export error: {0}")]
    Export(String),
    
    #[error("Web interface error: {0}")]
    WebInterface(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, GraphError>;
