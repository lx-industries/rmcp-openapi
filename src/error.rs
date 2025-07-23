use rmcp::model::{ErrorCode, ErrorData};
use schemars::JsonSchema;
use serde::Serialize;
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
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("MCP error: {0}")]
    McpError(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error(transparent)]
    ToolCall(#[from] ToolCallError),
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
            OpenApiError::ToolCall(e) => {
                // Map ToolCallError based on its content
                let code = if e.message.contains("not found") {
                    ErrorCode(-32601)
                } else if e.message.contains("parameter") || e.message.contains("Validation") {
                    ErrorCode(-32602)
                } else if e.message.contains("HTTP") {
                    ErrorCode(-32000)
                } else if e.message.contains("JSON") {
                    ErrorCode(-32700)
                } else {
                    ErrorCode(-32000)
                };
                ErrorData::new(code, e.message.clone(), None)
            }
            _ => ErrorData::new(ErrorCode(-32000), err.to_string(), None),
        }
    }
}

/// Error type specifically for tool execution that provides structured error information
#[derive(Debug, Serialize, JsonSchema, Error)]
#[error("{message}")]
#[schemars(inline)]
pub struct ToolCallError {
    /// Human-readable error message
    pub message: String,
    /// Machine-readable error details
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    #[schemars(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
}

/// Structured error details for different error scenarios
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[schemars(tag = "type", rename_all = "kebab-case")]
pub enum ErrorDetails {
    /// Invalid parameter error with suggestions
    InvalidParameter {
        /// The parameter name that was invalid
        parameter: String,
        /// Suggested correct parameter names
        suggestions: Vec<String>,
        /// All valid parameter names for this tool
        valid_parameters: Vec<String>,
    },
    // Future: Add more error detail variants as needed
}

impl ToolCallError {
    /// Create an invalid parameter error with suggestions
    pub fn invalid_parameter(
        parameter: String,
        suggestions: Vec<String>,
        valid_parameters: Vec<String>,
    ) -> Self {
        let message = if suggestions.is_empty() {
            format!(
                "Unknown parameter '{}'. Valid parameters are: {}",
                parameter,
                valid_parameters.join(", ")
            )
        } else if suggestions.len() == 1 {
            format!(
                "Unknown parameter '{}'. Did you mean '{}'?",
                parameter, suggestions[0]
            )
        } else {
            format!(
                "Unknown parameter '{}'. Did you mean one of these? {}",
                parameter,
                suggestions.join(", ")
            )
        };

        Self {
            message,
            details: Some(ErrorDetails::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
            }),
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found(tool_name: String) -> Self {
        Self {
            message: format!("Tool '{tool_name}' not found"),
            details: None,
        }
    }

    /// Create a validation error
    pub fn validation_error(msg: String) -> Self {
        Self {
            message: format!("Validation error: {msg}"),
            details: None,
        }
    }

    /// Create an HTTP error
    pub fn http_error(status: u16, msg: String) -> Self {
        Self {
            message: format!("HTTP {status} error: {msg}"),
            details: None,
        }
    }

    /// Create an HTTP request error
    pub fn http_request_error(msg: String) -> Self {
        Self {
            message: format!("HTTP request failed: {msg}"),
            details: None,
        }
    }

    /// Create a JSON parsing error
    pub fn json_error(msg: String) -> Self {
        Self {
            message: format!("JSON parsing error: {msg}"),
            details: None,
        }
    }

    /// Create an invalid parameter location error
    pub fn invalid_parameter_location(msg: String) -> Self {
        Self {
            message: format!("Invalid parameter location: {msg}"),
            details: None,
        }
    }
}

/// Error response structure for tool execution failures
#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    /// Error information
    pub error: ToolCallError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_tool_call_error_serialization_with_details() {
        let error = ToolCallError::invalid_parameter(
            "pet_id".to_string(),
            vec!["petId".to_string()],
            vec!["petId".to_string(), "timeout_seconds".to_string()],
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_serialization_without_details() {
        let error = ToolCallError::tool_not_found("unknownTool".to_string());

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_multiple_suggestions() {
        let error = ToolCallError::invalid_parameter(
            "pet_i".to_string(),
            vec!["petId".to_string(), "petInfo".to_string()],
            vec![
                "petId".to_string(),
                "petInfo".to_string(),
                "timeout".to_string(),
            ],
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_no_suggestions() {
        let error = ToolCallError::invalid_parameter(
            "completely_wrong".to_string(),
            vec![],
            vec!["petId".to_string(), "timeout".to_string()],
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation() {
        let error = ToolCallError::validation_error("Missing required field".to_string());
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_http_error() {
        let error = ToolCallError::http_error(404, "Not found".to_string());
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_http_request() {
        let error = ToolCallError::http_request_error("Connection timeout".to_string());
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_json() {
        let error = ToolCallError::json_error("Invalid JSON".to_string());
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_invalid_location() {
        let error = ToolCallError::invalid_parameter_location("body".to_string());
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ToolCallError::invalid_parameter(
            "test_param".to_string(),
            vec!["testParam".to_string()],
            vec!["testParam".to_string(), "otherParam".to_string()],
        );

        let response = ErrorResponse { error };
        let serialized = serde_json::to_value(&response).unwrap();
        assert_json_snapshot!(serialized);
    }
}
