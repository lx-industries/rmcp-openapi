//! Error handling for the OpenAPI MCP server.
//!
//! This module provides structured error types to help clients understand and potentially fix issues.
//! Errors are returned as a typed enum with specific fields for each error type.
//!
//! # Error Types
//!
//! ## InvalidParameter
//! Unknown or misspelled parameter names. Includes parameter name, suggestions for typos, and list of valid parameters.
//!
//! Example:
//! ```json
//! {
//!   "type": "invalid-parameter",
//!   "parameter": "pet_id",
//!   "suggestions": ["petId"],
//!   "valid_parameters": ["petId", "status"]
//! }
//! ```
//!
//! ## ValidationError
//! Parameter value validation failures. Includes descriptive message, field path, actual value,
//! expected type, and constraint details.
//!
//! Example for numeric constraints:
//! ```json
//! {
//!   "type": "validation-error",
//!   "message": "Parameter 'age' must be between 0 and 150",
//!   "field_path": "age",
//!   "actual_value": 200,
//!   "expected_type": "integer",
//!   "constraints": {
//!     "minimum": 0,
//!     "maximum": 150
//!   }
//! }
//! ```
//!
//! Example for array constraints:
//! ```json
//! {
//!   "type": "validation-error",
//!   "message": "Array has 1 items but minimum is 2",
//!   "field_path": "tags",
//!   "actual_value": ["tag1"],
//!   "expected_type": "array",
//!   "constraints": {
//!     "min_items": 2,
//!     "unique_items": true
//!   }
//! }
//! ```
//!
//! Example for const constraint:
//! ```json
//! {
//!   "type": "validation-error",
//!   "message": "\"staging\" is not equal to const \"production\"",
//!   "field_path": "environment",
//!   "actual_value": "staging",
//!   "expected_type": "string",
//!   "constraints": {
//!     "const_value": "production"
//!   }
//! }
//! ```
//!
//! ## MissingRequiredParameter
//! Required parameter not provided. Includes parameter name, description, and expected type.
//!
//! Example:
//! ```json
//! {
//!   "type": "missing-required-parameter",
//!   "parameter": "petId",
//!   "expected_type": "integer"
//! }
//! ```
//!
//! ## ToolNotFound
//! Requested tool doesn't exist. Includes the tool name that was not found.
//!
//! Example:
//! ```json
//! {
//!   "type": "tool-not-found",
//!   "tool_name": "unknownTool"
//! }
//! ```
//!
//! ## HttpError
//! HTTP error responses from the API. Includes status code and error message.
//!
//! Example:
//! ```json
//! {
//!   "type": "http-error",
//!   "status": 404,
//!   "message": "Pet not found"
//! }
//! ```
//!
//! ## HttpRequestError
//! Network/connection failures. Includes description of the request failure.
//!
//! Example:
//! ```json
//! {
//!   "type": "http-request-error",
//!   "message": "Connection timeout"
//! }
//! ```
//!
//! ## JsonError
//! JSON parsing failures. Includes description of what failed to parse.
//!
//! Example:
//! ```json
//! {
//!   "type": "json-error",
//!   "message": "Invalid JSON in response body"
//! }
//! ```
//!
//! # Structured Error Responses
//!
//! For tools with output schemas, errors are wrapped in the same structure as successful responses:
//! ```json
//! {
//!   "status": 400,
//!   "body": {
//!     "error": {
//!       "type": "invalid-parameter",
//!       "parameter": "pet_id",
//!       "suggestions": ["petId"],
//!       "valid_parameters": ["petId", "status"]
//!     }
//!   }
//! }
//! ```
//!
//! This consistent structure allows clients to:
//! - Programmatically handle different error types
//! - Provide helpful feedback to users
//! - Automatically fix certain errors (e.g., typos in parameter names)
//! - Retry requests with corrected parameters

use rmcp::model::{ErrorCode, ErrorData};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// Validation constraints that were violated
#[derive(Debug, Serialize, JsonSchema)]
pub struct ValidationConstraints {
    /// Minimum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Whether minimum is exclusive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,
    /// Whether maximum is exclusive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,
    /// Minimum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Pattern the string must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Allowed enum values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    /// Expected format (e.g., "date-time", "email", "uri")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Number must be a multiple of this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    /// Minimum number of items (for arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    /// Maximum number of items (for arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    /// Whether array items must be unique
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    /// Minimum number of properties (for objects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,
    /// Maximum number of properties (for objects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,
    /// Exact value that must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub const_value: Option<Value>,
    /// Required properties (for objects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

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
                // Map ToolCallError based on its variant
                let (code, message) = match &e {
                    ToolCallError::ToolNotFound { .. } => (ErrorCode(-32601), e.to_string()),
                    ToolCallError::InvalidParameter { .. }
                    | ToolCallError::ValidationError { .. }
                    | ToolCallError::MissingRequiredParameter { .. } => {
                        (ErrorCode(-32602), e.to_string())
                    }
                    ToolCallError::HttpError { .. } | ToolCallError::HttpRequestError { .. } => {
                        (ErrorCode(-32000), e.to_string())
                    }
                    ToolCallError::JsonError { .. } => (ErrorCode(-32700), e.to_string()),
                };
                ErrorData::new(code, message, None)
            }
            _ => ErrorData::new(ErrorCode(-32000), err.to_string(), None),
        }
    }
}

/// Helper function to format parameter suggestions
fn format_suggestions(suggestions: &[String], valid_parameters: &[String]) -> String {
    if suggestions.is_empty() {
        format!("Valid parameters are: {}", valid_parameters.join(", "))
    } else if suggestions.len() == 1 {
        format!("Did you mean '{}'?", suggestions[0])
    } else {
        format!("Did you mean one of these? {}", suggestions.join(", "))
    }
}

/// Error that can occur during tool execution
#[derive(Debug, Serialize, JsonSchema, Error)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[schemars(tag = "type", rename_all = "kebab-case")]
pub enum ToolCallError {
    /// Invalid parameter error with suggestions
    #[error(
        "Unknown parameter '{parameter}'. {}",
        format_suggestions(suggestions, valid_parameters)
    )]
    #[serde(rename = "invalid-parameter")]
    InvalidParameter {
        /// The parameter name that was invalid
        parameter: String,
        /// Suggested correct parameter names
        suggestions: Vec<String>,
        /// All valid parameter names for this tool
        valid_parameters: Vec<String>,
    },

    /// Tool not found error
    #[error("Tool '{tool_name}' not found")]
    #[serde(rename = "tool-not-found")]
    ToolNotFound {
        /// Name of the tool that was not found
        tool_name: String,
        // TODO: Future enhancement - add available_tools and suggestions
    },

    /// Validation error (e.g., type mismatches, constraint violations)
    #[error("Validation error: {message}")]
    #[serde(rename = "validation-error")]
    ValidationError {
        /// Description of what validation failed
        message: String,
        /// Path to the field that failed validation (e.g., "address.street")
        #[serde(skip_serializing_if = "Option::is_none")]
        field_path: Option<String>,
        /// The actual value that failed validation
        #[serde(skip_serializing_if = "Option::is_none")]
        actual_value: Option<Box<Value>>,
        /// Expected type or format
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_type: Option<String>,
        /// Constraint details that were violated
        #[serde(skip_serializing_if = "Option::is_none")]
        constraints: Option<Box<ValidationConstraints>>,
    },

    /// Missing required parameter
    #[error("Missing required parameter '{parameter}' of type {expected_type}")]
    #[serde(rename = "missing-required-parameter")]
    MissingRequiredParameter {
        /// Name of the missing parameter
        parameter: String,
        /// Description of the parameter from OpenAPI
        description: Option<String>,
        /// Expected type of the parameter
        expected_type: String,
    },

    /// HTTP error response from the API
    #[error("HTTP {status} error: {message}")]
    #[serde(rename = "http-error")]
    HttpError {
        /// HTTP status code
        status: u16,
        /// Error message or response body
        message: String,
        // TODO: Future enhancement - add structured error details for actionable errors
    },

    /// HTTP request failed (network, connection, timeout)
    #[error("HTTP request failed: {message}")]
    #[serde(rename = "http-request-error")]
    HttpRequestError {
        /// Description of the request failure
        message: String,
    },

    /// JSON parsing/serialization error
    #[error("JSON parsing error: {message}")]
    #[serde(rename = "json-error")]
    JsonError {
        /// Description of the JSON error
        message: String,
    },
}

impl ToolCallError {
    /// Create an invalid parameter error with suggestions
    pub fn invalid_parameter(
        parameter: String,
        suggestions: Vec<String>,
        valid_parameters: Vec<String>,
    ) -> Self {
        Self::InvalidParameter {
            parameter,
            suggestions,
            valid_parameters,
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found(tool_name: String) -> Self {
        Self::ToolNotFound { tool_name }
    }

    /// Create a validation error with only a message
    pub fn validation_error(msg: String) -> Self {
        Self::ValidationError {
            message: msg,
            field_path: None,
            actual_value: None,
            expected_type: None,
            constraints: None,
        }
    }

    /// Create a detailed validation error
    pub fn validation_error_detailed(
        message: String,
        field_path: Option<String>,
        actual_value: Option<Value>,
        expected_type: Option<String>,
        constraints: Option<ValidationConstraints>,
    ) -> Self {
        Self::ValidationError {
            message,
            field_path,
            actual_value: actual_value.map(Box::new),
            expected_type,
            constraints: constraints.map(Box::new),
        }
    }

    /// Create an HTTP error
    pub fn http_error(status: u16, msg: String) -> Self {
        Self::HttpError {
            status,
            message: msg,
        }
    }

    /// Create an HTTP request error
    pub fn http_request_error(msg: String) -> Self {
        Self::HttpRequestError { message: msg }
    }

    /// Create a JSON parsing error
    pub fn json_error(msg: String) -> Self {
        Self::JsonError { message: msg }
    }

    /// Create an invalid parameter location error
    pub fn invalid_parameter_location(msg: String) -> Self {
        Self::ValidationError {
            message: format!("Invalid parameter location: {msg}"),
            field_path: None,
            actual_value: None,
            expected_type: None,
            constraints: None,
        }
    }

    /// Get a human-readable message for this error
    pub fn message(&self) -> String {
        self.to_string()
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
    use serde_json::json;

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
    fn test_tool_call_error_validation_detailed() {
        let constraints = ValidationConstraints {
            minimum: Some(0.0),
            maximum: Some(150.0),
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Parameter 'age' must be between 0 and 150".to_string(),
            Some("age".to_string()),
            Some(json!(200)),
            Some("integer".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_enum() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: Some(vec![json!("available"), json!("pending"), json!("sold")]),
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Parameter 'status' must be one of: available, pending, sold".to_string(),
            Some("status".to_string()),
            Some(json!("unknown")),
            Some("string".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_format() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: Some("email".to_string()),
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Invalid email format".to_string(),
            Some("contact.email".to_string()),
            Some(json!("not-an-email")),
            Some("string".to_string()),
            Some(constraints),
        );

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

    #[test]
    fn test_tool_call_error_validation_multiple_of() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: Some(3.0),
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "10.5 is not a multiple of 3".to_string(),
            Some("price".to_string()),
            Some(json!(10.5)),
            Some("number".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_min_items() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: Some(2),
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Array has 1 items but minimum is 2".to_string(),
            Some("tags".to_string()),
            Some(json!(["tag1"])),
            Some("array".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_max_items() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: Some(3),
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Array has 4 items but maximum is 3".to_string(),
            Some("categories".to_string()),
            Some(json!(["a", "b", "c", "d"])),
            Some("array".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_unique_items() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: Some(true),
            min_properties: None,
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Array items [1, 2, 2, 3] are not unique".to_string(),
            Some("numbers".to_string()),
            Some(json!([1, 2, 2, 3])),
            Some("array".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_min_properties() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: Some(3),
            max_properties: None,
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Object has 2 properties but minimum is 3".to_string(),
            Some("metadata".to_string()),
            Some(json!({"name": "test", "version": "1.0"})),
            Some("object".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_max_properties() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: Some(2),
            const_value: None,
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            "Object has 3 properties but maximum is 2".to_string(),
            Some("config".to_string()),
            Some(json!({"a": 1, "b": 2, "c": 3})),
            Some("object".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_const() {
        let constraints = ValidationConstraints {
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            format: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            const_value: Some(json!("production")),
            required: None,
        };

        let error = ToolCallError::validation_error_detailed(
            r#""staging" is not equal to const "production""#.to_string(),
            Some("environment".to_string()),
            Some(json!("staging")),
            Some("string".to_string()),
            Some(constraints),
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }
}
