use thiserror::Error;

pub type ToolSearchResultType<T> = std::result::Result<T, ToolSearchError>;

/// Errors that can occur during tool search operations
#[derive(Error, Debug)]
pub enum ToolSearchError {
    #[error("Transport error: {0}")]
    Transport(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Unsupported transport: {0}")]
    UnsupportedTransport(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<rmcp::ErrorData> for ToolSearchError {
    fn from(err: rmcp::ErrorData) -> Self {
        ToolSearchError::McpProtocol(err.to_string())
    }
}

impl From<rmcp::service::ServiceError> for ToolSearchError {
    fn from(err: rmcp::service::ServiceError) -> Self {
        ToolSearchError::McpProtocol(err.to_string())
    }
}

