use rmcp::model::{Tool, ToolAnnotations};
use serde_json::Value;
use std::sync::Arc;

/// Internal metadata for tools generated from OpenAPI operations.
///
/// This struct contains all the information needed to execute HTTP requests
/// and is used internally by the OpenAPI server. It includes fields that are
/// not part of the MCP specification but are necessary for HTTP execution.
///
/// For MCP compliance, this struct is converted to `rmcp::model::Tool` using
/// the `From` trait implementation, which only includes MCP-compliant fields.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolMetadata {
    /// Tool name - exposed to MCP clients
    pub name: String,
    /// Tool title - human-readable display name exposed to MCP clients
    pub title: Option<String>,
    /// Tool description - exposed to MCP clients  
    pub description: String,
    /// Input parameters schema - exposed to MCP clients as `inputSchema`
    pub parameters: Value,
    /// Output schema - exposed to MCP clients as `outputSchema`
    pub output_schema: Option<Value>,
    /// HTTP method (GET, POST, etc.) - internal only, not exposed to MCP
    pub method: String,
    /// URL path for the API endpoint - internal only, not exposed to MCP
    pub path: String,
}

impl ToolMetadata {
    /// Check if a parameter is required according to the tool metadata
    pub fn is_parameter_required(&self, param_name: &str) -> bool {
        self.parameters
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().any(|v| v.as_str() == Some(param_name)))
            .unwrap_or(false)
    }

    /// Check if a parameter is an array type according to the tool metadata
    pub fn is_parameter_array_type(&self, param_name: &str) -> bool {
        self.parameters
            .get("properties")
            .and_then(|props| props.get(param_name))
            .and_then(|param| param.get("type"))
            .and_then(|type_val| type_val.as_str())
            .map(|type_str| type_str == "array")
            .unwrap_or(false)
    }

    /// Check if a parameter has a default value according to the tool metadata
    pub fn has_parameter_default(&self, param_name: &str) -> bool {
        self.parameters
            .get("properties")
            .and_then(|props| props.get(param_name))
            .map(|param| param.get("default").is_some())
            .unwrap_or(false)
    }

    /// Check if a JSON value is an empty array
    pub fn is_empty_array(value: &Value) -> bool {
        matches!(value, Value::Array(arr) if arr.is_empty())
    }

    /// Determine if an empty array parameter should be omitted from the HTTP request
    pub fn should_omit_empty_array_parameter(&self, param_name: &str, value: &Value) -> bool {
        // Only omit if:
        // 1. Parameter is not required
        // 2. Parameter is array type
        // 3. Value is empty array
        // 4. Parameter has no explicit default value
        !self.is_parameter_required(param_name)
            && self.is_parameter_array_type(param_name)
            && Self::is_empty_array(value)
            && !self.has_parameter_default(param_name)
    }
}

/// Converts internal `ToolMetadata` to MCP-compliant `Tool`.
///
/// This implementation ensures that only MCP-compliant fields are exposed to clients.
/// Internal fields like `method` and `path` are not included in the conversion.
impl From<&ToolMetadata> for Tool {
    fn from(metadata: &ToolMetadata) -> Self {
        // Convert parameters to the expected Arc<Map> format
        let input_schema = if let Value::Object(obj) = &metadata.parameters {
            Arc::new(obj.clone())
        } else {
            Arc::new(serde_json::Map::new())
        };

        // Convert output_schema to the expected Arc<Map> format if present
        let output_schema = metadata.output_schema.as_ref().and_then(|schema| {
            if let Value::Object(obj) = schema {
                Some(Arc::new(obj.clone()))
            } else {
                None
            }
        });

        // Create annotations with title if present
        let annotations = metadata.title.as_ref().map(|title| ToolAnnotations {
            title: Some(title.clone()),
            ..Default::default()
        });

        Tool {
            name: metadata.name.clone().into(),
            description: Some(metadata.description.clone().into()),
            input_schema,
            output_schema,
            annotations,
            // TODO: Consider migration to Tool.title when rmcp supports MCP 2025-06-18 (see issue #26)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_metadata_with_params(parameters: Value) -> ToolMetadata {
        ToolMetadata {
            name: "test_tool".to_string(),
            title: Some("Test Tool".to_string()),
            description: "A test tool".to_string(),
            parameters,
            output_schema: None,
            method: "GET".to_string(),
            path: "/test".to_string(),
        }
    }

    #[test]
    fn test_is_parameter_required() {
        let metadata = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "required_param": {"type": "string"},
                "optional_param": {"type": "string"}
            },
            "required": ["required_param"]
        }));

        assert!(metadata.is_parameter_required("required_param"));
        assert!(!metadata.is_parameter_required("optional_param"));
        assert!(!metadata.is_parameter_required("nonexistent_param"));
    }

    #[test]
    fn test_is_parameter_required_no_required_array() {
        let metadata = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "param": {"type": "string"}
            }
        }));

        assert!(!metadata.is_parameter_required("param"));
    }

    #[test]
    fn test_is_parameter_array_type() {
        let metadata = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "array_param": {"type": "array", "items": {"type": "string"}},
                "string_param": {"type": "string"},
                "number_param": {"type": "number"}
            }
        }));

        assert!(metadata.is_parameter_array_type("array_param"));
        assert!(!metadata.is_parameter_array_type("string_param"));
        assert!(!metadata.is_parameter_array_type("number_param"));
        assert!(!metadata.is_parameter_array_type("nonexistent_param"));
    }

    #[test]
    fn test_has_parameter_default() {
        let metadata = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "with_default": {"type": "string", "default": "test"},
                "with_array_default": {"type": "array", "items": {"type": "string"}, "default": ["item1"]},
                "without_default": {"type": "string"},
                "with_null_default": {"type": "string", "default": null}
            }
        }));

        assert!(metadata.has_parameter_default("with_default"));
        assert!(metadata.has_parameter_default("with_array_default"));
        assert!(!metadata.has_parameter_default("without_default"));
        assert!(metadata.has_parameter_default("with_null_default")); // null is still a default
        assert!(!metadata.has_parameter_default("nonexistent_param"));
    }

    #[test]
    fn test_is_empty_array() {
        assert!(ToolMetadata::is_empty_array(&json!([])));
        assert!(!ToolMetadata::is_empty_array(&json!(["item"])));
        assert!(!ToolMetadata::is_empty_array(&json!("string")));
        assert!(!ToolMetadata::is_empty_array(&json!(42)));
        assert!(!ToolMetadata::is_empty_array(&json!(null)));
    }

    #[test]
    fn test_should_omit_empty_array_parameter() {
        // Test case: optional array without default - should omit
        let metadata1 = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "optional_array": {"type": "array", "items": {"type": "string"}}
            },
            "required": []
        }));
        assert!(metadata1.should_omit_empty_array_parameter("optional_array", &json!([])));
        assert!(!metadata1.should_omit_empty_array_parameter("optional_array", &json!(["item"])));

        // Test case: required array - should not omit
        let metadata2 = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "required_array": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["required_array"]
        }));
        assert!(!metadata2.should_omit_empty_array_parameter("required_array", &json!([])));

        // Test case: optional array with default - should not omit
        let metadata3 = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "optional_array_with_default": {
                    "type": "array",
                    "items": {"type": "string"},
                    "default": ["default_item"]
                }
            },
            "required": []
        }));
        assert!(
            !metadata3.should_omit_empty_array_parameter("optional_array_with_default", &json!([]))
        );

        // Test case: optional non-array - should not omit
        let metadata4 = create_test_metadata_with_params(json!({
            "type": "object",
            "properties": {
                "optional_string": {"type": "string"}
            },
            "required": []
        }));
        assert!(!metadata4.should_omit_empty_array_parameter("optional_string", &json!([])));
    }
}
