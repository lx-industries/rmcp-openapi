use serde_json::{Value, json};
use std::collections::HashMap;

use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use openapiv3::{Operation, Parameter, ParameterData, ReferenceOr, Schema, SchemaKind};

/// Tool generator for creating MCP tools from OpenAPI operations
pub struct ToolGenerator;

impl ToolGenerator {
    /// Generate tool metadata from an OpenAPI operation
    pub fn generate_tool_metadata(
        operation: &Operation,
        method: String,
        path: String,
    ) -> Result<ToolMetadata, OpenApiError> {
        let name = operation.operation_id.clone().unwrap_or_else(|| {
            format!(
                "{}_{}",
                method,
                path.replace('/', "_").replace(['{', '}'], "")
            )
        });

        // Build description from summary and description
        let description = Self::build_description(operation, &method, &path);

        // Generate parameter schema
        let parameters = Self::generate_parameter_schema(&operation.parameters, &method)?;

        Ok(ToolMetadata {
            name,
            description,
            parameters,
            method,
            path,
        })
    }

    /// Build a comprehensive description for the tool
    fn build_description(operation: &Operation, method: &str, path: &str) -> String {
        match (&operation.summary, &operation.description) {
            (Some(summary), Some(desc)) => {
                format!(
                    "{}\n\n{}\n\nEndpoint: {} {}",
                    summary,
                    desc,
                    method.to_uppercase(),
                    path
                )
            }
            (Some(summary), None) => {
                format!(
                    "{}\n\nEndpoint: {} {}",
                    summary,
                    method.to_uppercase(),
                    path
                )
            }
            (None, Some(desc)) => {
                format!("{}\n\nEndpoint: {} {}", desc, method.to_uppercase(), path)
            }
            (None, None) => {
                format!("API endpoint: {} {}", method.to_uppercase(), path)
            }
        }
    }

    /// Generate JSON Schema for tool parameters
    fn generate_parameter_schema(
        parameters: &[ReferenceOr<Parameter>],
        method: &str,
    ) -> Result<Value, OpenApiError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        // Group parameters by location
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();
        let mut cookie_params = Vec::new();

        for param_ref in parameters {
            match param_ref {
                ReferenceOr::Item(param) => match param {
                    Parameter::Query { parameter_data, .. } => query_params.push(parameter_data),
                    Parameter::Header { parameter_data, .. } => header_params.push(parameter_data),
                    Parameter::Path { parameter_data, .. } => path_params.push(parameter_data),
                    Parameter::Cookie { parameter_data, .. } => cookie_params.push(parameter_data),
                },
                ReferenceOr::Reference { .. } => {
                    // For now, skip reference parameters - could be implemented later
                    continue;
                }
            }
        }

        // Process path parameters (always required)
        for param_data in path_params {
            let param_schema = Self::convert_parameter_schema(param_data, "path")?;
            properties.insert(param_data.name.clone(), param_schema);
            required.push(param_data.name.clone());
        }

        // Process query parameters
        for param_data in &query_params {
            let param_schema = Self::convert_parameter_schema(param_data, "query")?;
            properties.insert(param_data.name.clone(), param_schema);
            if param_data.required {
                required.push(param_data.name.clone());
            }
        }

        // Process header parameters (optional by default unless explicitly required)
        for param_data in &header_params {
            let mut param_schema = Self::convert_parameter_schema(param_data, "header")?;

            // Add location metadata for headers
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("header"));
            }

            properties.insert(format!("header_{}", param_data.name), param_schema);
            if param_data.required {
                required.push(format!("header_{}", param_data.name));
            }
        }

        // Process cookie parameters (rare, but supported)
        for param_data in &cookie_params {
            let mut param_schema = Self::convert_parameter_schema(param_data, "cookie")?;

            // Add location metadata for cookies
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("cookie"));
            }

            properties.insert(format!("cookie_{}", param_data.name), param_schema);
            if param_data.required {
                required.push(format!("cookie_{}", param_data.name));
            }
        }

        // Add request body parameter for operations that typically need it
        if ["post", "put", "patch"].contains(&method.to_lowercase().as_str()) {
            properties.insert(
                "request_body".to_string(),
                json!({
                    "type": "object",
                    "description": "Request body data (JSON)",
                    "additionalProperties": true,
                    "x-location": "body",
                    "x-content-type": mime::APPLICATION_JSON.as_ref()
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
    fn convert_parameter_schema(
        param_data: &ParameterData,
        location: &str,
    ) -> Result<Value, OpenApiError> {
        let mut result = serde_json::Map::new();

        // Handle the parameter schema
        match &param_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
                match schema_ref {
                    ReferenceOr::Item(schema) => {
                        Self::convert_schema_to_json_schema(schema, &mut result)?;
                    }
                    ReferenceOr::Reference { .. } => {
                        // For now, default to string for references
                        result.insert("type".to_string(), json!("string"));
                    }
                }
            }
            openapiv3::ParameterSchemaOrContent::Content(_) => {
                // For content parameters, default to object
                result.insert("type".to_string(), json!("object"));
            }
        }

        // Add description
        if let Some(desc) = &param_data.description {
            result.insert("description".to_string(), json!(desc));
        } else {
            result.insert(
                "description".to_string(),
                json!(format!("{} parameter", param_data.name)),
            );
        }

        // Add parameter location metadata
        result.insert("x-parameter-location".to_string(), json!(location));
        result.insert(
            "x-parameter-required".to_string(),
            json!(param_data.required),
        );

        Ok(Value::Object(result))
    }

    /// Convert openapiv3::Schema to JSON Schema properties
    fn convert_schema_to_json_schema(
        schema: &Schema,
        result: &mut serde_json::Map<String, Value>,
    ) -> Result<(), OpenApiError> {
        match &schema.schema_kind {
            SchemaKind::Type(type_) => match type_ {
                openapiv3::Type::String(string_type) => {
                    result.insert("type".to_string(), json!("string"));
                    if let Some(min_length) = string_type.min_length {
                        result.insert("minLength".to_string(), json!(min_length));
                    }
                    if let Some(max_length) = string_type.max_length {
                        result.insert("maxLength".to_string(), json!(max_length));
                    }
                    if let Some(pattern) = &string_type.pattern {
                        result.insert("pattern".to_string(), json!(pattern));
                    }
                    if let openapiv3::VariantOrUnknownOrEmpty::Item(format) = &string_type.format {
                        result.insert("format".to_string(), json!(format!("{:?}", format)));
                    }
                }
                openapiv3::Type::Number(number_type) => {
                    result.insert("type".to_string(), json!("number"));
                    if let Some(minimum) = number_type.minimum {
                        result.insert("minimum".to_string(), json!(minimum));
                    }
                    if let Some(maximum) = number_type.maximum {
                        result.insert("maximum".to_string(), json!(maximum));
                    }
                    if let openapiv3::VariantOrUnknownOrEmpty::Item(format) = &number_type.format {
                        result.insert("format".to_string(), json!(format!("{:?}", format)));
                    }
                }
                openapiv3::Type::Integer(integer_type) => {
                    result.insert("type".to_string(), json!("integer"));
                    if let Some(minimum) = integer_type.minimum {
                        result.insert("minimum".to_string(), json!(minimum));
                    }
                    if let Some(maximum) = integer_type.maximum {
                        result.insert("maximum".to_string(), json!(maximum));
                    }
                    if let openapiv3::VariantOrUnknownOrEmpty::Item(format) = &integer_type.format {
                        result.insert("format".to_string(), json!(format!("{:?}", format)));
                    }
                }
                openapiv3::Type::Boolean(_) => {
                    result.insert("type".to_string(), json!("boolean"));
                }
                openapiv3::Type::Array(array_type) => {
                    result.insert("type".to_string(), json!("array"));
                    if let Some(items) = &array_type.items {
                        match items {
                            ReferenceOr::Item(item_schema) => {
                                let mut items_result = serde_json::Map::new();
                                Self::convert_schema_to_json_schema(
                                    item_schema,
                                    &mut items_result,
                                )?;
                                result.insert("items".to_string(), Value::Object(items_result));
                            }
                            ReferenceOr::Reference { .. } => {
                                result.insert("items".to_string(), json!({"type": "string"}));
                            }
                        }
                    } else {
                        result.insert("items".to_string(), json!({"type": "string"}));
                    }
                }
                openapiv3::Type::Object(_) => {
                    result.insert("type".to_string(), json!("object"));
                    result.insert("additionalProperties".to_string(), json!(true));
                }
            },
            SchemaKind::OneOf { .. } | SchemaKind::AllOf { .. } | SchemaKind::AnyOf { .. } => {
                // For complex schema types, default to object
                result.insert("type".to_string(), json!("object"));
            }
            SchemaKind::Not { .. } => {
                // For not schema, default to string
                result.insert("type".to_string(), json!("string"));
            }
            SchemaKind::Any(_) => {
                // For any schema, allow any type
                result.insert("type".to_string(), json!("object"));
            }
        }

        // Handle enum values - in openapiv3 this is typically handled in the schema_kind
        // For now, we'll skip enum handling as it's more complex in openapiv3

        Ok(())
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

        let _properties = schema
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
            content_type: mime::APPLICATION_JSON.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openapiv3::*;
    use serde_json::{Value, json};

    fn validate_tool_against_mcp_schema(metadata: &ToolMetadata) {
        let schema_content = std::fs::read_to_string("schema/2025-03-26/schema.json")
            .expect("Failed to read MCP schema file");
        let full_schema: Value =
            serde_json::from_str(&schema_content).expect("Failed to parse MCP schema JSON");

        // Create a schema that references the Tool definition from the full schema
        let tool_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "definitions": full_schema.get("definitions"),
            "$ref": "#/definitions/Tool"
        });

        let validator =
            jsonschema::validator_for(&tool_schema).expect("Failed to compile MCP Tool schema");

        // Convert ToolMetadata to MCP Tool format
        let mcp_tool = json!({
            "name": metadata.name,
            "description": metadata.description,
            "inputSchema": metadata.parameters
        });

        // Validate the generated tool against MCP schema
        let errors: Vec<String> = validator
            .iter_errors(&mcp_tool)
            .map(|e| e.to_string())
            .collect();

        if !errors.is_empty() {
            panic!("Generated tool failed MCP schema validation: {errors:?}");
        }
    }

    #[test]
    fn test_petstore_get_pet_by_id() {
        let mut operation = Operation {
            operation_id: Some("getPetById".to_string()),
            summary: Some("Find pet by ID".to_string()),
            description: Some("Returns a single pet".to_string()),
            ..Default::default()
        };

        // Create a path parameter
        let param_data = ParameterData {
            name: "petId".to_string(),
            description: Some("ID of pet to return".to_string()),
            required: true,
            deprecated: None,
            format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                schema_data: SchemaData::default(),
                schema_kind: SchemaKind::Type(Type::Integer(IntegerType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64),
                    minimum: Some(1),
                    maximum: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    multiple_of: None,
                    enumeration: Vec::new(),
                })),
            })),
            example: None,
            examples: indexmap::IndexMap::new(),
            extensions: indexmap::IndexMap::new(),
            explode: None,
        };

        operation
            .parameters
            .push(ReferenceOr::Item(Parameter::Path {
                parameter_data: param_data,
                style: Default::default(),
            }));

        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pet/{petId}".to_string(),
        )
        .unwrap();

        assert_eq!(metadata.name, "getPetById");
        assert_eq!(metadata.method, "get");
        assert_eq!(metadata.path, "/pet/{petId}");
        assert!(metadata.description.contains("Find pet by ID"));

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }
}
