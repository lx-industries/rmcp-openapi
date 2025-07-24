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
//! Requested tool doesn't exist. Includes the tool name that was not found and suggestions for similar tool names.
//!
//! Example without suggestions:
//! ```json
//! {
//!   "type": "tool-not-found",
//!   "tool_name": "unknownTool",
//!   "suggestions": []
//! }
//! ```
//!
//! Example with suggestions (e.g., typo in tool name):
//! ```json
//! {
//!   "type": "tool-not-found",
//!   "tool_name": "getPetByID",
//!   "suggestions": ["getPetById", "getPetsByStatus"]
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
use serde_json::{Value, json};
use thiserror::Error;

/// Individual validation constraint that was violated
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ValidationConstraint {
    /// Minimum value constraint (for numbers)
    Minimum {
        /// The minimum value
        value: f64,
        /// Whether the minimum is exclusive
        exclusive: bool,
    },
    /// Maximum value constraint (for numbers)
    Maximum {
        /// The maximum value
        value: f64,
        /// Whether the maximum is exclusive
        exclusive: bool,
    },
    /// Minimum length constraint (for strings/arrays)
    MinLength {
        /// The minimum length
        value: usize,
    },
    /// Maximum length constraint (for strings/arrays)
    MaxLength {
        /// The maximum length
        value: usize,
    },
    /// Pattern constraint (for strings)
    Pattern {
        /// The regex pattern that must be matched
        pattern: String,
    },
    /// Enum values constraint
    EnumValues {
        /// The allowed enum values
        values: Vec<Value>,
    },
    /// Format constraint (e.g., "date-time", "email", "uri")
    Format {
        /// The expected format
        format: String,
    },
    /// Multiple of constraint (for numbers)
    MultipleOf {
        /// The value that the number must be a multiple of
        value: f64,
    },
    /// Minimum number of items constraint (for arrays)
    MinItems {
        /// The minimum number of items
        value: usize,
    },
    /// Maximum number of items constraint (for arrays)
    MaxItems {
        /// The maximum number of items
        value: usize,
    },
    /// Unique items constraint (for arrays)
    UniqueItems,
    /// Minimum number of properties constraint (for objects)
    MinProperties {
        /// The minimum number of properties
        value: usize,
    },
    /// Maximum number of properties constraint (for objects)
    MaxProperties {
        /// The maximum number of properties
        value: usize,
    },
    /// Constant value constraint
    ConstValue {
        /// The exact value that must match
        value: Value,
    },
    /// Required properties constraint (for objects)
    Required {
        /// The required property names
        properties: Vec<String>,
    },
}

/// Individual validation error types
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ValidationError {
    /// Invalid parameter error with suggestions
    InvalidParameter {
        /// The parameter name that was invalid
        parameter: String,
        /// Suggested correct parameter names
        suggestions: Vec<String>,
        /// All valid parameter names for this tool
        valid_parameters: Vec<String>,
    },
    /// Missing required parameter
    MissingRequiredParameter {
        /// Name of the missing parameter
        parameter: String,
        /// Description of the parameter from OpenAPI
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Expected type of the parameter
        expected_type: String,
    },
    /// Constraint violation (e.g., type mismatches, pattern violations)
    ConstraintViolation {
        /// Name of the parameter that failed validation
        parameter: String,
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
        /// Specific constraints that were violated
        #[serde(skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<ValidationConstraint>,
    },
}

#[derive(Debug, Error)]
pub enum OpenApiError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("OpenAPI spec error: {0}")]
    Spec(String),
    #[error("Tool generation error: {0}")]
    ToolGeneration(String),
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
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    ToolCall(#[from] ToolCallError),
}

impl From<ToolCallError> for ErrorData {
    fn from(err: ToolCallError) -> Self {
        match err {
            ToolCallError::ToolNotFound {
                tool_name,
                suggestions,
            } => {
                let data = if suggestions.is_empty() {
                    None
                } else {
                    Some(json!({
                        "suggestions": suggestions
                    }))
                };
                ErrorData::new(
                    ErrorCode(-32601),
                    format!("Tool '{tool_name}' not found"),
                    data,
                )
            }
            ToolCallError::ValidationErrors { .. } => {
                ErrorData::new(ErrorCode(-32602), err.to_string(), None)
            }
            ToolCallError::HttpError { .. } | ToolCallError::HttpRequestError { .. } => {
                ErrorData::new(ErrorCode(-32000), err.to_string(), None)
            }
            ToolCallError::JsonError { .. } => {
                ErrorData::new(ErrorCode(-32700), err.to_string(), None)
            }
        }
    }
}

impl From<OpenApiError> for ErrorData {
    fn from(err: OpenApiError) -> Self {
        match err {
            OpenApiError::Spec(msg) => ErrorData::new(
                ErrorCode(-32700),
                format!("OpenAPI spec error: {msg}"),
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
            OpenApiError::Json(e) => {
                ErrorData::new(ErrorCode(-32700), format!("JSON parsing error: {e}"), None)
            }
            OpenApiError::ToolCall(e) => e.into(),
            _ => ErrorData::new(ErrorCode(-32000), err.to_string(), None),
        }
    }
}

/// Error that can occur during tool execution
#[derive(Debug, Serialize, JsonSchema, Error)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[schemars(tag = "type", rename_all = "kebab-case")]
pub enum ToolCallError {
    /// Multiple validation errors
    #[error("Validation failed with multiple errors")]
    #[serde(rename = "validation-errors")]
    ValidationErrors {
        /// List of validation errors
        violations: Vec<ValidationError>,
    },

    /// Tool not found error
    #[error("Tool '{tool_name}' not found")]
    #[serde(rename = "tool-not-found")]
    ToolNotFound {
        /// Name of the tool that was not found
        tool_name: String,
        /// Suggested tool names based on similarity
        suggestions: Vec<String>,
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
    /// Create a validation errors collection
    pub fn validation_errors(violations: Vec<ValidationError>) -> Self {
        Self::ValidationErrors { violations }
    }

    /// Create an invalid parameter error with suggestions
    pub fn invalid_parameter(
        parameter: String,
        suggestions: Vec<String>,
        valid_parameters: Vec<String>,
    ) -> Self {
        Self::ValidationErrors {
            violations: vec![ValidationError::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
            }],
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found(tool_name: String, suggestions: Vec<String>) -> Self {
        Self::ToolNotFound {
            tool_name,
            suggestions,
        }
    }

    /// Create a validation error with only a message
    pub fn validation_error(msg: String) -> Self {
        Self::ValidationErrors {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: String::new(),
                message: msg,
                field_path: None,
                actual_value: None,
                expected_type: None,
                constraints: vec![],
            }],
        }
    }

    /// Create a detailed validation error
    pub fn validation_error_detailed(
        message: String,
        field_path: Option<String>,
        actual_value: Option<Value>,
        expected_type: Option<String>,
        constraints: Vec<ValidationConstraint>,
    ) -> Self {
        // Extract parameter name from field_path or use empty string
        let parameter = field_path
            .as_ref()
            .map(|path| {
                path.split('.')
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('/')
                    .to_string()
            })
            .unwrap_or_default();

        Self::ValidationErrors {
            violations: vec![ValidationError::ConstraintViolation {
                parameter,
                message,
                field_path,
                actual_value: actual_value.map(Box::new),
                expected_type,
                constraints,
            }],
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
        Self::ValidationErrors {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: String::new(),
                message: format!("Invalid parameter location: {msg}"),
                field_path: None,
                actual_value: None,
                expected_type: None,
                constraints: vec![],
            }],
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
        let error = ToolCallError::tool_not_found("unknownTool".to_string(), vec![]);

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_serialization_with_suggestions() {
        let error = ToolCallError::tool_not_found(
            "getPetByID".to_string(),
            vec!["getPetById".to_string(), "getPetsByStatus".to_string()],
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_eq!(serialized["type"], "tool-not-found");
        assert_eq!(serialized["tool_name"], "getPetByID");
        assert_eq!(
            serialized["suggestions"],
            json!(["getPetById", "getPetsByStatus"])
        );
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
        let constraints = vec![
            ValidationConstraint::Minimum {
                value: 0.0,
                exclusive: false,
            },
            ValidationConstraint::Maximum {
                value: 150.0,
                exclusive: false,
            },
        ];

        let error = ToolCallError::validation_error_detailed(
            "Parameter 'age' must be between 0 and 150".to_string(),
            Some("age".to_string()),
            Some(json!(200)),
            Some("integer".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_enum() {
        let constraints = vec![ValidationConstraint::EnumValues {
            values: vec![json!("available"), json!("pending"), json!("sold")],
        }];

        let error = ToolCallError::validation_error_detailed(
            "Parameter 'status' must be one of: available, pending, sold".to_string(),
            Some("status".to_string()),
            Some(json!("unknown")),
            Some("string".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_format() {
        let constraints = vec![ValidationConstraint::Format {
            format: "email".to_string(),
        }];

        let error = ToolCallError::validation_error_detailed(
            "Invalid email format".to_string(),
            Some("contact.email".to_string()),
            Some(json!("not-an-email")),
            Some("string".to_string()),
            constraints,
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
        let constraints = vec![ValidationConstraint::MultipleOf { value: 3.0 }];

        let error = ToolCallError::validation_error_detailed(
            "10.5 is not a multiple of 3".to_string(),
            Some("price".to_string()),
            Some(json!(10.5)),
            Some("number".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_min_items() {
        let constraints = vec![ValidationConstraint::MinItems { value: 2 }];

        let error = ToolCallError::validation_error_detailed(
            "Array has 1 items but minimum is 2".to_string(),
            Some("tags".to_string()),
            Some(json!(["tag1"])),
            Some("array".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_max_items() {
        let constraints = vec![ValidationConstraint::MaxItems { value: 3 }];

        let error = ToolCallError::validation_error_detailed(
            "Array has 4 items but maximum is 3".to_string(),
            Some("categories".to_string()),
            Some(json!(["a", "b", "c", "d"])),
            Some("array".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_unique_items() {
        let constraints = vec![ValidationConstraint::UniqueItems];

        let error = ToolCallError::validation_error_detailed(
            "Array items [1, 2, 2, 3] are not unique".to_string(),
            Some("numbers".to_string()),
            Some(json!([1, 2, 2, 3])),
            Some("array".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_min_properties() {
        let constraints = vec![ValidationConstraint::MinProperties { value: 3 }];

        let error = ToolCallError::validation_error_detailed(
            "Object has 2 properties but minimum is 3".to_string(),
            Some("metadata".to_string()),
            Some(json!({"name": "test", "version": "1.0"})),
            Some("object".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_max_properties() {
        let constraints = vec![ValidationConstraint::MaxProperties { value: 2 }];

        let error = ToolCallError::validation_error_detailed(
            "Object has 3 properties but maximum is 2".to_string(),
            Some("config".to_string()),
            Some(json!({"a": 1, "b": 2, "c": 3})),
            Some("object".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_const() {
        let constraints = vec![ValidationConstraint::ConstValue {
            value: json!("production"),
        }];

        let error = ToolCallError::validation_error_detailed(
            r#""staging" is not equal to const "production""#.to_string(),
            Some("environment".to_string()),
            Some(json!("staging")),
            Some("string".to_string()),
            constraints,
        );

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }
}
