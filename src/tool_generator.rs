use serde_json::{Value, json};
use std::collections::HashMap;

use crate::error::OpenApiError;
use crate::openapi_spec::{OpenApiOperation, OpenApiParameter};
use crate::server::ToolMetadata;

/// Tool generator for creating MCP tools from OpenAPI operations
pub struct ToolGenerator;

impl ToolGenerator {
    /// Generate tool metadata from an OpenAPI operation
    pub fn generate_tool_metadata(
        operation: &OpenApiOperation,
    ) -> Result<ToolMetadata, OpenApiError> {
        let name = operation.operation_id.clone();

        // Build description from summary and description
        let description = Self::build_description(operation);

        // Generate parameter schema
        let parameters = Self::generate_parameter_schema(&operation.parameters, &operation.method)?;

        Ok(ToolMetadata {
            name,
            description,
            parameters,
            method: operation.method.clone(),
            path: operation.path.clone(),
        })
    }

    /// Build a comprehensive description for the tool
    fn build_description(operation: &OpenApiOperation) -> String {
        match (&operation.summary, &operation.description) {
            (Some(summary), Some(desc)) => {
                format!(
                    "{}\n\n{}\n\nEndpoint: {} {}",
                    summary,
                    desc,
                    operation.method.to_uppercase(),
                    operation.path
                )
            }
            (Some(summary), None) => {
                format!(
                    "{}\n\nEndpoint: {} {}",
                    summary,
                    operation.method.to_uppercase(),
                    operation.path
                )
            }
            (None, Some(desc)) => {
                format!(
                    "{}\n\nEndpoint: {} {}",
                    desc,
                    operation.method.to_uppercase(),
                    operation.path
                )
            }
            (None, None) => {
                format!(
                    "API endpoint: {} {}",
                    operation.method.to_uppercase(),
                    operation.path
                )
            }
        }
    }

    /// Generate JSON Schema for tool parameters
    fn generate_parameter_schema(
        parameters: &[OpenApiParameter],
        method: &str,
    ) -> Result<Value, OpenApiError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        // Group parameters by location
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();
        let mut cookie_params = Vec::new();
        let mut body_params = Vec::new();

        for param in parameters {
            match param.location {
                crate::openapi_spec::ParameterLocation::Path => path_params.push(param),
                crate::openapi_spec::ParameterLocation::Query => query_params.push(param),
                crate::openapi_spec::ParameterLocation::Header => header_params.push(param),
                crate::openapi_spec::ParameterLocation::Cookie => cookie_params.push(param),
                crate::openapi_spec::ParameterLocation::FormData => body_params.push(param),
            }
        }

        // Process path parameters (always required)
        for param in path_params {
            let param_schema = Self::convert_parameter_schema(param)?;
            properties.insert(param.name.clone(), param_schema);
            required.push(param.name.clone());
        }

        // Process query parameters
        for param in &query_params {
            let param_schema = Self::convert_parameter_schema(param)?;
            properties.insert(param.name.clone(), param_schema);
            if param.required {
                required.push(param.name.clone());
            }
        }

        // Process header parameters (optional by default unless explicitly required)
        for param in &header_params {
            let mut param_schema = Self::convert_parameter_schema(param)?;

            // Add location metadata for headers
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("header"));
            }

            properties.insert(format!("header_{}", param.name), param_schema);
            if param.required {
                required.push(format!("header_{}", param.name));
            }
        }

        // Process cookie parameters (rare, but supported)
        for param in &cookie_params {
            let mut param_schema = Self::convert_parameter_schema(param)?;

            // Add location metadata for cookies
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("cookie"));
            }

            properties.insert(format!("cookie_{}", param.name), param_schema);
            if param.required {
                required.push(format!("cookie_{}", param.name));
            }
        }

        // Process request body parameters (for POST/PUT operations)
        for param in &body_params {
            let mut param_schema = Self::convert_parameter_schema(param)?;

            // Add location metadata for body
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("body"));
                obj.insert("x-content-type".to_string(), json!("application/json"));
            }

            properties.insert(format!("body_{}", param.name), param_schema);
            if param.required {
                required.push(format!("body_{}", param.name));
            }
        }

        // Add request body parameter for operations that typically need it
        if body_params.is_empty()
            && ["post", "put", "patch"].contains(&method.to_lowercase().as_str())
        {
            properties.insert(
                "request_body".to_string(),
                json!({
                    "type": "object",
                    "description": "Request body data (JSON)",
                    "additionalProperties": true,
                    "x-location": "body",
                    "x-content-type": "application/json"
                }),
            );
        }

        // Add special parameters for request configuration
        if !query_params.is_empty() || !header_params.is_empty() || !cookie_params.is_empty() {
            // Add optional timeout parameter
            properties.insert(
                "timeout_seconds".to_string(),
                json!({
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "minimum": 1,
                    "maximum": 300,
                    "default": 30
                }),
            );
        }

        Ok(json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false
        }))
    }

    /// Convert OpenAPI parameter schema to JSON Schema for MCP tools
    fn convert_parameter_schema(param: &OpenApiParameter) -> Result<Value, OpenApiError> {
        let mut schema = param.schema.clone();

        // Ensure we have a valid schema object
        if !schema.is_object() {
            schema = json!({
                "type": param.param_type
            });
        }

        let mut result = serde_json::Map::new();

        // Copy type information
        if let Some(param_type) = schema.get("type") {
            result.insert("type".to_string(), param_type.clone());
        } else {
            result.insert("type".to_string(), json!(param.param_type));
        }

        // Add description
        if let Some(desc) = &param.description {
            result.insert("description".to_string(), json!(desc));
        } else {
            result.insert(
                "description".to_string(),
                json!(format!("{} parameter", param.name)),
            );
        }

        // Handle array types
        if param.param_type == "array"
            || schema.get("type").and_then(|v| v.as_str()) == Some("array")
        {
            if let Some(items) = schema.get("items") {
                result.insert("items".to_string(), items.clone());
            } else {
                result.insert("items".to_string(), json!({"type": "string"}));
            }
        }

        // Copy additional constraints
        for key in [
            "minimum",
            "maximum",
            "minLength",
            "maxLength",
            "pattern",
            "enum",
            "format",
        ] {
            if let Some(constraint) = schema.get(key) {
                result.insert(key.to_string(), constraint.clone());
            }
        }

        // Add parameter location metadata
        result.insert(
            "x-parameter-location".to_string(),
            json!(param.location.to_string()),
        );
        result.insert("x-parameter-required".to_string(), json!(param.required));

        Ok(Value::Object(result))
    }

    /// Extract parameter values from MCP tool call arguments
    pub fn extract_parameters(
        tool_metadata: &ToolMetadata,
        arguments: &Value,
    ) -> Result<ExtractedParameters, OpenApiError> {
        let args = arguments
            .as_object()
            .ok_or_else(|| OpenApiError::Validation("Arguments must be an object".to_string()))?;

        let mut path_params = HashMap::new();
        let mut query_params = HashMap::new();
        let mut header_params = HashMap::new();
        let mut cookie_params = HashMap::new();
        let mut body_params = HashMap::new();
        let mut config = RequestConfig::default();

        // Extract timeout if provided
        if let Some(timeout) = args.get("timeout_seconds").and_then(|v| v.as_u64()) {
            config.timeout_seconds = timeout as u32;
        }

        // Process each argument
        for (key, value) in args {
            if key == "timeout_seconds" {
                continue; // Already processed
            }

            // Handle special request_body parameter
            if key == "request_body" {
                body_params.insert("request_body".to_string(), value.clone());
                continue;
            }

            // Determine parameter location from the tool metadata
            let location = Self::get_parameter_location(tool_metadata, key)?;

            match location.as_str() {
                "path" => {
                    path_params.insert(key.clone(), value.clone());
                }
                "query" => {
                    query_params.insert(key.clone(), value.clone());
                }
                "header" => {
                    // Remove "header_" prefix if present
                    let header_name = if key.starts_with("header_") {
                        key.strip_prefix("header_").unwrap_or(key).to_string()
                    } else {
                        key.clone()
                    };
                    header_params.insert(header_name, value.clone());
                }
                "cookie" => {
                    // Remove "cookie_" prefix if present
                    let cookie_name = if key.starts_with("cookie_") {
                        key.strip_prefix("cookie_").unwrap_or(key).to_string()
                    } else {
                        key.clone()
                    };
                    cookie_params.insert(cookie_name, value.clone());
                }
                "body" => {
                    // Remove "body_" prefix if present
                    let body_name = if key.starts_with("body_") {
                        key.strip_prefix("body_").unwrap_or(key).to_string()
                    } else {
                        key.clone()
                    };
                    body_params.insert(body_name, value.clone());
                }
                _ => {
                    return Err(OpenApiError::ToolGeneration(format!(
                        "Unknown parameter location for parameter: {key}"
                    )));
                }
            }
        }

        let extracted = ExtractedParameters {
            path: path_params,
            query: query_params,
            headers: header_params,
            cookies: cookie_params,
            body: body_params,
            config,
        };

        // Validate parameters against tool metadata
        Self::validate_parameters(tool_metadata, &extracted)?;

        Ok(extracted)
    }

    /// Get parameter location from tool metadata
    fn get_parameter_location(
        tool_metadata: &ToolMetadata,
        param_name: &str,
    ) -> Result<String, OpenApiError> {
        let properties = tool_metadata
            .parameters
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or_else(|| {
                OpenApiError::ToolGeneration("Invalid tool parameters schema".to_string())
            })?;

        if let Some(param_schema) = properties.get(param_name) {
            if let Some(location) = param_schema
                .get("x-parameter-location")
                .and_then(|v| v.as_str())
            {
                return Ok(location.to_string());
            }
        }

        // Fallback: infer from parameter name prefix
        if param_name.starts_with("header_") {
            Ok("header".to_string())
        } else if param_name.starts_with("cookie_") {
            Ok("cookie".to_string())
        } else if param_name.starts_with("body_") {
            Ok("body".to_string())
        } else {
            // Default to query for unknown parameters
            Ok("query".to_string())
        }
    }

    /// Validate extracted parameters against tool metadata
    fn validate_parameters(
        tool_metadata: &ToolMetadata,
        extracted: &ExtractedParameters,
    ) -> Result<(), OpenApiError> {
        let schema = &tool_metadata.parameters;

        // Get required parameters from schema
        let required_params = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<std::collections::HashSet<_>>()
            })
            .unwrap_or_default();

        let properties = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or_else(|| {
                OpenApiError::Validation("Tool schema missing properties".to_string())
            })?;

        // Check all required parameters are provided
        for required_param in &required_params {
            let param_found = extracted.path.contains_key(*required_param)
                || extracted.query.contains_key(*required_param)
                || extracted
                    .headers
                    .contains_key(&required_param.replace("header_", ""))
                || extracted
                    .cookies
                    .contains_key(&required_param.replace("cookie_", ""))
                || extracted
                    .body
                    .contains_key(&required_param.replace("body_", ""))
                || (*required_param == "request_body"
                    && extracted.body.contains_key("request_body"));

            if !param_found {
                return Err(OpenApiError::InvalidParameter {
                    parameter: required_param.to_string(),
                    reason: "Required parameter is missing".to_string(),
                });
            }
        }

        // Validate parameter types and constraints
        for (param_name, param_value) in extracted
            .path
            .iter()
            .chain(extracted.query.iter())
            .chain(extracted.headers.iter())
            .chain(extracted.cookies.iter())
            .chain(extracted.body.iter())
        {
            if let Some(param_schema) = properties
                .get(param_name)
                .or_else(|| properties.get(&format!("header_{param_name}")))
                .or_else(|| properties.get(&format!("cookie_{param_name}")))
                .or_else(|| properties.get(&format!("body_{param_name}")))
            {
                Self::validate_parameter_value(param_name, param_value, param_schema)?;
            }
        }

        Ok(())
    }

    /// Validate a single parameter value against its schema
    fn validate_parameter_value(
        param_name: &str,
        param_value: &Value,
        param_schema: &Value,
    ) -> Result<(), OpenApiError> {
        let expected_type = param_schema.get("type").and_then(|t| t.as_str());

        match expected_type {
            Some("string") => {
                if !param_value.is_string() {
                    return Err(OpenApiError::InvalidParameter {
                        parameter: param_name.to_string(),
                        reason: "Expected string value".to_string(),
                    });
                }

                // Check string constraints
                if let Some(value_str) = param_value.as_str() {
                    if let Some(min_length) = param_schema.get("minLength").and_then(|v| v.as_u64())
                    {
                        if value_str.len() < min_length as usize {
                            return Err(OpenApiError::InvalidParameter {
                                parameter: param_name.to_string(),
                                reason: format!("String too short, minimum length is {min_length}"),
                            });
                        }
                    }

                    if let Some(max_length) = param_schema.get("maxLength").and_then(|v| v.as_u64())
                    {
                        if value_str.len() > max_length as usize {
                            return Err(OpenApiError::InvalidParameter {
                                parameter: param_name.to_string(),
                                reason: format!("String too long, maximum length is {max_length}"),
                            });
                        }
                    }

                    if let Some(pattern) = param_schema.get("pattern").and_then(|v| v.as_str()) {
                        if let Ok(regex) = regex::Regex::new(pattern) {
                            if !regex.is_match(value_str) {
                                return Err(OpenApiError::InvalidParameter {
                                    parameter: param_name.to_string(),
                                    reason: format!("String does not match pattern: {pattern}"),
                                });
                            }
                        }
                    }

                    if let Some(enum_values) = param_schema.get("enum").and_then(|v| v.as_array()) {
                        let valid_values: Vec<&str> =
                            enum_values.iter().filter_map(|v| v.as_str()).collect();
                        if !valid_values.contains(&value_str) {
                            return Err(OpenApiError::InvalidParameter {
                                parameter: param_name.to_string(),
                                reason: format!(
                                    "Invalid enum value. Valid values: {valid_values:?}"
                                ),
                            });
                        }
                    }
                }
            }
            Some("integer") | Some("number") => {
                if !param_value.is_number() {
                    return Err(OpenApiError::InvalidParameter {
                        parameter: param_name.to_string(),
                        reason: "Expected numeric value".to_string(),
                    });
                }

                if let Some(value_num) = param_value.as_f64() {
                    if let Some(minimum) = param_schema.get("minimum").and_then(|v| v.as_f64()) {
                        if value_num < minimum {
                            return Err(OpenApiError::InvalidParameter {
                                parameter: param_name.to_string(),
                                reason: format!("Value {value_num} is below minimum {minimum}"),
                            });
                        }
                    }

                    if let Some(maximum) = param_schema.get("maximum").and_then(|v| v.as_f64()) {
                        if value_num > maximum {
                            return Err(OpenApiError::InvalidParameter {
                                parameter: param_name.to_string(),
                                reason: format!("Value {value_num} is above maximum {maximum}"),
                            });
                        }
                    }
                }
            }
            Some("boolean") => {
                if !param_value.is_boolean() {
                    return Err(OpenApiError::InvalidParameter {
                        parameter: param_name.to_string(),
                        reason: "Expected boolean value".to_string(),
                    });
                }
            }
            Some("array") => {
                if !param_value.is_array() {
                    return Err(OpenApiError::InvalidParameter {
                        parameter: param_name.to_string(),
                        reason: "Expected array value".to_string(),
                    });
                }
            }
            Some("object") => {
                if !param_value.is_object() {
                    return Err(OpenApiError::InvalidParameter {
                        parameter: param_name.to_string(),
                        reason: "Expected object value".to_string(),
                    });
                }
            }
            _ => {
                // No specific type validation, allow any value
            }
        }

        Ok(())
    }
}

/// Extracted parameters from MCP tool call
#[derive(Debug, Clone)]
pub struct ExtractedParameters {
    pub path: HashMap<String, Value>,
    pub query: HashMap<String, Value>,
    pub headers: HashMap<String, Value>,
    pub cookies: HashMap<String, Value>,
    pub body: HashMap<String, Value>,
    pub config: RequestConfig,
}

/// Request configuration options
#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub timeout_seconds: u32,
    pub content_type: String,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            content_type: "application/json".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi_spec::{OpenApiOperation, OpenApiParameter, ParameterLocation};

    #[test]
    fn test_petstore_get_pet_by_id() {
        let operation = OpenApiOperation {
            operation_id: "getPetById".to_string(),
            summary: Some("Find pet by ID".to_string()),
            description: Some("Returns a single pet".to_string()),
            method: "get".to_string(),
            path: "/pet/{petId}".to_string(),
            parameters: vec![OpenApiParameter {
                name: "petId".to_string(),
                location: ParameterLocation::Path,
                description: Some("ID of pet to return".to_string()),
                required: true,
                param_type: "integer".to_string(),
                schema: json!({
                    "type": "integer",
                    "format": "int64",
                    "minimum": 1
                }),
            }],
        };

        let metadata = ToolGenerator::generate_tool_metadata(&operation).unwrap();
        insta::assert_json_snapshot!(metadata);
    }

    #[test]
    fn test_petstore_find_pets_by_status() {
        let operation = OpenApiOperation {
            operation_id: "findPetsByStatus".to_string(),
            summary: Some("Finds Pets by status".to_string()),
            description: Some(
                "Multiple status values can be provided with comma separated strings".to_string(),
            ),
            method: "get".to_string(),
            path: "/pet/findByStatus".to_string(),
            parameters: vec![OpenApiParameter {
                name: "status".to_string(),
                location: ParameterLocation::Query,
                description: Some(
                    "Status values that need to be considered for filter".to_string(),
                ),
                required: true,
                param_type: "array".to_string(),
                schema: json!({
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["available", "pending", "sold"]
                    }
                }),
            }],
        };

        let metadata = ToolGenerator::generate_tool_metadata(&operation).unwrap();
        insta::assert_json_snapshot!(metadata);
    }

    #[test]
    fn test_petstore_add_pet() {
        let operation = OpenApiOperation {
            operation_id: "addPet".to_string(),
            summary: Some("Add a new pet to the store".to_string()),
            description: Some("Add a new pet to the store".to_string()),
            method: "post".to_string(),
            path: "/pet".to_string(),
            parameters: vec![],
        };

        let metadata = ToolGenerator::generate_tool_metadata(&operation).unwrap();
        insta::assert_json_snapshot!(metadata);
    }

    #[test]
    fn test_petstore_update_pet_with_form() {
        let operation = OpenApiOperation {
            operation_id: "updatePetWithForm".to_string(),
            summary: Some("Updates a pet in the store with form data".to_string()),
            description: None,
            method: "post".to_string(),
            path: "/pet/{petId}".to_string(),
            parameters: vec![
                OpenApiParameter {
                    name: "petId".to_string(),
                    location: ParameterLocation::Path,
                    description: Some("ID of pet that needs to be updated".to_string()),
                    required: true,
                    param_type: "integer".to_string(),
                    schema: json!({
                        "type": "integer",
                        "format": "int64"
                    }),
                },
                OpenApiParameter {
                    name: "name".to_string(),
                    location: ParameterLocation::Query,
                    description: Some("Updated name of the pet".to_string()),
                    required: false,
                    param_type: "string".to_string(),
                    schema: json!({
                        "type": "string"
                    }),
                },
                OpenApiParameter {
                    name: "status".to_string(),
                    location: ParameterLocation::Query,
                    description: Some("Updated status of the pet".to_string()),
                    required: false,
                    param_type: "string".to_string(),
                    schema: json!({
                        "type": "string",
                        "enum": ["available", "pending", "sold"]
                    }),
                },
            ],
        };

        let metadata = ToolGenerator::generate_tool_metadata(&operation).unwrap();
        insta::assert_json_snapshot!(metadata);
    }
}
