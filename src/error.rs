//! Error handling for the OpenAPI MCP server.
//!
//! This module provides structured error types that distinguish between validation errors
//! (which return as MCP protocol errors) and execution errors (which appear in tool output schemas).
//!
//! # Error Categories
//!
//! ## Validation Errors (MCP Protocol Errors)
//! These errors occur before tool execution and are returned as MCP protocol errors (Err(ErrorData)).
//! They do NOT have JsonSchema derive to prevent them from appearing in tool output schemas.
//!
//! - **ToolNotFound**: Requested tool doesn't exist
//! - **InvalidParameters**: Parameter validation failed (unknown names, missing required, constraint violations)
//! - **RequestConstructionError**: Failed to construct the HTTP request
//!
//! ## Execution Errors (Tool Output Errors)
//! These errors occur during tool execution and are returned as structured content in the tool response.
//! They have JsonSchema derive so they can appear in tool output schemas.
//!
//! - **HttpError**: HTTP error response from the API (4xx, 5xx status codes)
//! - **NetworkError**: Network/connection failures (timeout, DNS, connection refused)
//! - **ResponseParsingError**: Failed to parse the response
//!
//! # Error Type Examples
//!
//! ## InvalidParameter (Validation Error)
//! ```json
//! {
//!   "type": "invalid-parameter",
//!   "parameter": "pet_id",
//!   "suggestions": ["petId"],
//!   "valid_parameters": ["petId", "status"]
//! }
//! ```
//!
//! ## ConstraintViolation (Validation Error)
//! ```json
//! {
//!   "type": "constraint-violation",
//!   "parameter": "age",
//!   "message": "Parameter 'age' must be between 0 and 150",
//!   "field_path": "age",
//!   "actual_value": 200,
//!   "expected_type": "integer",
//!   "constraints": [
//!     {"type": "minimum", "value": 0, "exclusive": false},
//!     {"type": "maximum", "value": 150, "exclusive": false}
//!   ]
//! }
//! ```
//!
//! ## HttpError (Execution Error)
//! ```json
//! {
//!   "type": "http-error",
//!   "status": 404,
//!   "message": "Pet not found",
//!   "details": {"error": "NOT_FOUND", "pet_id": 123}
//! }
//! ```
//!
//! ## NetworkError (Execution Error)
//! ```json
//! {
//!   "type": "network-error",
//!   "message": "Request timeout after 30 seconds",
//!   "category": "timeout"
//! }
//! ```
//!
//! # Structured Error Responses
//!
//! For tools with output schemas, execution errors are wrapped in the standard response structure:
//! ```json
//! {
//!   "status": 404,
//!   "body": {
//!     "error": {
//!       "type": "http-error",
//!       "status": 404,
//!       "message": "Pet not found"
//!     }
//!   }
//! }
//! ```
//!
//! Validation errors are returned as MCP protocol errors:
//! ```json
//! {
//!   "code": -32602,
//!   "message": "Validation failed with 1 error",
//!   "data": {
//!     "type": "validation-errors",
//!     "violations": [
//!       {
//!         "type": "invalid-parameter",
//!         "parameter": "pet_id",
//!         "suggestions": ["petId"],
//!         "valid_parameters": ["petId", "status"]
//!       }
//!     ]
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

impl From<ToolCallValidationError> for ErrorData {
    fn from(err: ToolCallValidationError) -> Self {
        match err {
            ToolCallValidationError::ToolNotFound {
                ref tool_name,
                ref suggestions,
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
            ToolCallValidationError::InvalidParameters { ref violations } => {
                // Include the full validation error details
                let data = Some(json!({
                    "type": "validation-errors",
                    "violations": violations
                }));
                ErrorData::new(ErrorCode(-32602), err.to_string(), data)
            }
            ToolCallValidationError::RequestConstructionError { ref reason } => {
                // Include construction error details
                let data = Some(json!({
                    "type": "request-construction-error",
                    "reason": reason
                }));
                ErrorData::new(ErrorCode(-32602), err.to_string(), data)
            }
        }
    }
}

impl From<ToolCallError> for ErrorData {
    fn from(err: ToolCallError) -> Self {
        match err {
            ToolCallError::Validation(validation_err) => validation_err.into(),
            ToolCallError::Execution(execution_err) => {
                // Execution errors should not be converted to ErrorData
                // They should be returned as CallToolResult with is_error: true
                // But for backward compatibility, we'll convert them
                match execution_err {
                    ToolCallExecutionError::HttpError {
                        status,
                        ref message,
                        ..
                    } => {
                        let data = Some(json!({
                            "type": "http-error",
                            "status": status,
                            "message": message
                        }));
                        ErrorData::new(ErrorCode(-32000), execution_err.to_string(), data)
                    }
                    ToolCallExecutionError::NetworkError {
                        ref message,
                        ref category,
                    } => {
                        let data = Some(json!({
                            "type": "network-error",
                            "message": message,
                            "category": category
                        }));
                        ErrorData::new(ErrorCode(-32000), execution_err.to_string(), data)
                    }
                    ToolCallExecutionError::ResponseParsingError { ref reason, .. } => {
                        let data = Some(json!({
                            "type": "response-parsing-error",
                            "reason": reason
                        }));
                        ErrorData::new(ErrorCode(-32700), execution_err.to_string(), data)
                    }
                }
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
#[derive(Debug, Error, Serialize)]
#[serde(untagged)]
pub enum ToolCallError {
    /// Validation errors that occur before tool execution
    #[error(transparent)]
    Validation(#[from] ToolCallValidationError),

    /// Execution errors that occur during tool execution
    #[error(transparent)]
    Execution(#[from] ToolCallExecutionError),
}

/// Error response structure for tool execution failures
#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    /// Error information
    pub error: ToolCallExecutionError,
}

/// Validation errors that occur before tool execution
/// These return as Err(ErrorData) with MCP protocol error codes
#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ToolCallValidationError {
    /// Tool not found
    #[error("Tool '{tool_name}' not found")]
    #[serde(rename = "tool-not-found")]
    ToolNotFound {
        /// Name of the tool that was not found
        tool_name: String,
        /// Suggested tool names based on similarity
        suggestions: Vec<String>,
    },

    /// Invalid parameters (unknown names, missing required, constraints)
    #[error("Validation failed with {} error{}", violations.len(), if violations.len() == 1 { "" } else { "s" })]
    #[serde(rename = "validation-errors")]
    InvalidParameters {
        /// List of validation errors
        violations: Vec<ValidationError>,
    },

    /// Request construction failed (JSON serialization for body)
    #[error("Failed to construct request: {reason}")]
    #[serde(rename = "request-construction-error")]
    RequestConstructionError {
        /// Description of the construction failure
        reason: String,
    },
}

/// Execution errors that occur during tool execution
/// These return as Ok(CallToolResult { is_error: true })
#[derive(Debug, Error, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[schemars(tag = "type", rename_all = "kebab-case")]
pub enum ToolCallExecutionError {
    /// HTTP error response from the API
    #[error("HTTP {status} error: {message}")]
    #[serde(rename = "http-error")]
    HttpError {
        /// HTTP status code
        status: u16,
        /// Error message or response body
        message: String,
        /// Optional structured error details from API
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<Value>,
    },

    /// Network/connection failures
    #[error("Network error: {message}")]
    #[serde(rename = "network-error")]
    NetworkError {
        /// Description of the network failure
        message: String,
        /// Error category for better handling
        category: NetworkErrorCategory,
    },

    /// Response parsing failed
    #[error("Failed to parse response: {reason}")]
    #[serde(rename = "response-parsing-error")]
    ResponseParsingError {
        /// Description of the parsing failure
        reason: String,
        /// Raw response body for debugging
        #[serde(skip_serializing_if = "Option::is_none")]
        raw_response: Option<String>,
    },
}

/// Network error categories for better error handling
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkErrorCategory {
    /// Request timeout
    Timeout,
    /// Connection error (DNS, refused, unreachable)
    Connect,
    /// Request construction/sending error
    Request,
    /// Response body error
    Body,
    /// Response decoding error
    Decode,
    /// Other network errors
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;
    use serde_json::json;

    #[test]
    fn test_tool_call_error_serialization_with_details() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::InvalidParameter {
                parameter: "pet_id".to_string(),
                suggestions: vec!["petId".to_string()],
                valid_parameters: vec!["petId".to_string(), "timeout_seconds".to_string()],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_serialization_without_details() {
        let error = ToolCallError::Validation(ToolCallValidationError::ToolNotFound {
            tool_name: "unknownTool".to_string(),
            suggestions: vec![],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_serialization_with_suggestions() {
        let error = ToolCallError::Validation(ToolCallValidationError::ToolNotFound {
            tool_name: "getPetByID".to_string(),
            suggestions: vec!["getPetById".to_string(), "getPetsByStatus".to_string()],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_multiple_suggestions() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::InvalidParameter {
                parameter: "pet_i".to_string(),
                suggestions: vec!["petId".to_string(), "petInfo".to_string()],
                valid_parameters: vec![
                    "petId".to_string(),
                    "petInfo".to_string(),
                    "timeout".to_string(),
                ],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_no_suggestions() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::InvalidParameter {
                parameter: "completely_wrong".to_string(),
                suggestions: vec![],
                valid_parameters: vec!["petId".to_string(), "timeout".to_string()],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::MissingRequiredParameter {
                parameter: "field".to_string(),
                description: Some("Missing required field".to_string()),
                expected_type: "string".to_string(),
            }],
        });
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_detailed() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "age".to_string(),
                message: "Parameter 'age' must be between 0 and 150".to_string(),
                field_path: Some("age".to_string()),
                actual_value: Some(Box::new(json!(200))),
                expected_type: Some("integer".to_string()),
                constraints: vec![
                    ValidationConstraint::Minimum {
                        value: 0.0,
                        exclusive: false,
                    },
                    ValidationConstraint::Maximum {
                        value: 150.0,
                        exclusive: false,
                    },
                ],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_enum() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "status".to_string(),
                message: "Parameter 'status' must be one of: available, pending, sold".to_string(),
                field_path: Some("status".to_string()),
                actual_value: Some(Box::new(json!("unknown"))),
                expected_type: Some("string".to_string()),
                constraints: vec![ValidationConstraint::EnumValues {
                    values: vec![json!("available"), json!("pending"), json!("sold")],
                }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_format() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "email".to_string(),
                message: "Invalid email format".to_string(),
                field_path: Some("contact.email".to_string()),
                actual_value: Some(Box::new(json!("not-an-email"))),
                expected_type: Some("string".to_string()),
                constraints: vec![ValidationConstraint::Format {
                    format: "email".to_string(),
                }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_http_error() {
        let error = ToolCallError::Execution(ToolCallExecutionError::HttpError {
            status: 404,
            message: "Not found".to_string(),
            details: None,
        });
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_http_request() {
        let error = ToolCallError::Execution(ToolCallExecutionError::NetworkError {
            message: "Connection timeout".to_string(),
            category: NetworkErrorCategory::Timeout,
        });
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_json() {
        let error = ToolCallError::Execution(ToolCallExecutionError::ResponseParsingError {
            reason: "Invalid JSON".to_string(),
            raw_response: None,
        });
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_request_construction() {
        let error = ToolCallError::Validation(ToolCallValidationError::RequestConstructionError {
            reason: "Invalid parameter location: body".to_string(),
        });
        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ToolCallExecutionError::HttpError {
            status: 400,
            message: "Bad Request".to_string(),
            details: Some(json!({
                "error": "Invalid parameter",
                "parameter": "test_param"
            })),
        };

        let response = ErrorResponse { error };
        let serialized = serde_json::to_value(&response).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_multiple_of() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "price".to_string(),
                message: "10.5 is not a multiple of 3".to_string(),
                field_path: Some("price".to_string()),
                actual_value: Some(Box::new(json!(10.5))),
                expected_type: Some("number".to_string()),
                constraints: vec![ValidationConstraint::MultipleOf { value: 3.0 }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_min_items() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "tags".to_string(),
                message: "Array has 1 items but minimum is 2".to_string(),
                field_path: Some("tags".to_string()),
                actual_value: Some(Box::new(json!(["tag1"]))),
                expected_type: Some("array".to_string()),
                constraints: vec![ValidationConstraint::MinItems { value: 2 }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_max_items() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "categories".to_string(),
                message: "Array has 4 items but maximum is 3".to_string(),
                field_path: Some("categories".to_string()),
                actual_value: Some(Box::new(json!(["a", "b", "c", "d"]))),
                expected_type: Some("array".to_string()),
                constraints: vec![ValidationConstraint::MaxItems { value: 3 }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_unique_items() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "numbers".to_string(),
                message: "Array items [1, 2, 2, 3] are not unique".to_string(),
                field_path: Some("numbers".to_string()),
                actual_value: Some(Box::new(json!([1, 2, 2, 3]))),
                expected_type: Some("array".to_string()),
                constraints: vec![ValidationConstraint::UniqueItems],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_min_properties() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "metadata".to_string(),
                message: "Object has 2 properties but minimum is 3".to_string(),
                field_path: Some("metadata".to_string()),
                actual_value: Some(Box::new(json!({"name": "test", "version": "1.0"}))),
                expected_type: Some("object".to_string()),
                constraints: vec![ValidationConstraint::MinProperties { value: 3 }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_max_properties() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "config".to_string(),
                message: "Object has 3 properties but maximum is 2".to_string(),
                field_path: Some("config".to_string()),
                actual_value: Some(Box::new(json!({"a": 1, "b": 2, "c": 3}))),
                expected_type: Some("object".to_string()),
                constraints: vec![ValidationConstraint::MaxProperties { value: 2 }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_tool_call_error_validation_const() {
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::ConstraintViolation {
                parameter: "environment".to_string(),
                message: r#""staging" is not equal to const "production""#.to_string(),
                field_path: Some("environment".to_string()),
                actual_value: Some(Box::new(json!("staging"))),
                expected_type: Some("string".to_string()),
                constraints: vec![ValidationConstraint::ConstValue {
                    value: json!("production"),
                }],
            }],
        });

        let serialized = serde_json::to_value(&error).unwrap();
        assert_json_snapshot!(serialized);
    }

    #[test]
    fn test_error_data_conversion_preserves_details() {
        // Test InvalidParameter error conversion
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![ValidationError::InvalidParameter {
                parameter: "page".to_string(),
                suggestions: vec!["page_number".to_string()],
                valid_parameters: vec!["page_number".to_string(), "page_size".to_string()],
            }],
        });

        let error_data: ErrorData = error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        // Check that error details are preserved
        assert!(error_json["data"].is_object(), "Should have data field");
        assert_eq!(
            error_json["data"]["type"].as_str(),
            Some("validation-errors"),
            "Should have validation-errors type"
        );

        // Test Network error conversion
        let network_error = ToolCallError::Execution(ToolCallExecutionError::NetworkError {
            message: "SSL/TLS connection failed - certificate verification error".to_string(),
            category: NetworkErrorCategory::Connect,
        });

        let error_data: ErrorData = network_error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        assert!(error_json["data"].is_object(), "Should have data field");
        assert_eq!(
            error_json["data"]["type"].as_str(),
            Some("network-error"),
            "Should have network-error type"
        );
        assert!(
            error_json["data"]["message"]
                .as_str()
                .unwrap()
                .contains("SSL/TLS"),
            "Should preserve error message"
        );
    }
}
