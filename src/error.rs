use rmcp::model::{ErrorCode, ErrorData};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenApiError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OpenAPI spec error: {0}")]
    Spec(String),
    #[error("Tool generation error: {0}")]
    ToolGeneration(String),
    #[error("Parameter validation error: {0}")]
    Validation(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Invalid parameter: {parameter} - {reason}")]
    InvalidParameter { parameter: String, reason: String },
    #[error("Invalid parameter location: {0}")]
    InvalidParameterLocation(String),
}

impl From<OpenApiError> for ErrorData {
    fn from(err: OpenApiError) -> Self {
        match err {
            OpenApiError::ToolNotFound(tool_name) => ErrorData::new(
                ErrorCode(-32601),
                format!("Tool '{tool_name}' not found"),
                None,
            ),
            OpenApiError::InvalidParameter { parameter, reason } => ErrorData::new(
                ErrorCode(-32602),
                format!("Invalid parameter '{parameter}': {reason}"),
                None,
            ),
            OpenApiError::Validation(msg) => {
                ErrorData::new(ErrorCode(-32602), format!("Validation error: {msg}"), None)
            }
            OpenApiError::HttpRequest(e) => {
                ErrorData::new(ErrorCode(-32000), format!("HTTP request failed: {e}"), None)
            }
            OpenApiError::Http(msg) => {
                ErrorData::new(ErrorCode(-32000), format!("HTTP error: {msg}"), None)
            }
            OpenApiError::Spec(msg) => ErrorData::new(
                ErrorCode(-32700),
                format!("OpenAPI spec error: {msg}"),
                None,
            ),
            OpenApiError::Json(e) => {
                ErrorData::new(ErrorCode(-32700), format!("JSON parsing error: {e}"), None)
            }
            _ => ErrorData::new(ErrorCode(-32000), err.to_string(), None),
        }
    }
}
