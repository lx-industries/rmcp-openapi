use serde_json::{Value, json};
use std::collections::HashMap;

use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use oas3::spec::{
    ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn, Schema, SchemaType,
    SchemaTypeSet,
};

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
        parameters: &[ObjectOrReference<Parameter>],
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
                ObjectOrReference::Object(param) => match &param.location {
                    ParameterIn::Query => query_params.push(param),
                    ParameterIn::Header => header_params.push(param),
                    ParameterIn::Path => path_params.push(param),
                    ParameterIn::Cookie => cookie_params.push(param),
                },
                ObjectOrReference::Ref { .. } => {
                    // Skip references for now
                    continue;
                }
            }
        }

        // Process path parameters (always required)
        for param in path_params {
            let param_schema = Self::convert_parameter_schema(param, "path")?;
            properties.insert(param.name.clone(), param_schema);
            required.push(param.name.clone());
        }

        // Process query parameters
        for param in &query_params {
            let param_schema = Self::convert_parameter_schema(param, "query")?;
            properties.insert(param.name.clone(), param_schema);
            if param.required.unwrap_or(false) {
                required.push(param.name.clone());
            }
        }

        // Process header parameters (optional by default unless explicitly required)
        for param in &header_params {
            let mut param_schema = Self::convert_parameter_schema(param, "header")?;

            // Add location metadata for headers
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("header"));
            }

            properties.insert(format!("header_{}", param.name), param_schema);
            if param.required.unwrap_or(false) {
                required.push(format!("header_{}", param.name));
            }
        }

        // Process cookie parameters (rare, but supported)
        for param in &cookie_params {
            let mut param_schema = Self::convert_parameter_schema(param, "cookie")?;

            // Add location metadata for cookies
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("cookie"));
            }

            properties.insert(format!("cookie_{}", param.name), param_schema);
            if param.required.unwrap_or(false) {
                required.push(format!("cookie_{}", param.name));
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
    fn convert_parameter_schema(param: &Parameter, location: &str) -> Result<Value, OpenApiError> {
        let mut result = serde_json::Map::new();

        // Handle the parameter schema
        if let Some(schema_ref) = &param.schema {
            match schema_ref {
                ObjectOrReference::Object(obj_schema) => {
                    Self::convert_schema_to_json_schema(
                        &Schema::Object(Box::new(ObjectOrReference::Object(obj_schema.clone()))),
                        &mut result,
                    )?;
                }
                ObjectOrReference::Ref { .. } => {
                    // Default to string for references
                    result.insert("type".to_string(), json!("string"));
                }
            }
        } else {
            // Default to string if no schema
            result.insert("type".to_string(), json!("string"));
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

        // Add parameter location metadata
        result.insert("x-parameter-location".to_string(), json!(location));
        result.insert("x-parameter-required".to_string(), json!(param.required));

        Ok(Value::Object(result))
    }

    /// Converts prefixItems (tuple-like arrays) to JSON Schema draft-07 compatible format.
    ///
    /// This handles OpenAPI 3.1 prefixItems which define specific schemas for each array position,
    /// converting them to draft-07 format that MCP tools can understand.
    ///
    /// Conversion strategy:
    /// - If items is `false`, set minItems=maxItems=prefix_items.len() for exact length
    /// - If all prefixItems have same type, use that type for items
    /// - If mixed types, use oneOf with all unique types from prefixItems
    /// - Add descriptive comment about tuple nature
    fn convert_prefix_items_to_draft07(
        prefix_items: &[ObjectOrReference<ObjectSchema>],
        items: &Option<Box<Schema>>,
        result: &mut serde_json::Map<String, Value>,
    ) -> Result<(), OpenApiError> {
        let prefix_count = prefix_items.len();

        // Extract types from prefixItems
        let mut item_types = Vec::new();
        for prefix_item in prefix_items {
            match prefix_item {
                ObjectOrReference::Object(obj_schema) => {
                    if let Some(schema_type) = &obj_schema.schema_type {
                        match schema_type {
                            SchemaTypeSet::Single(SchemaType::String) => item_types.push("string"),
                            SchemaTypeSet::Single(SchemaType::Integer) => {
                                item_types.push("integer")
                            }
                            SchemaTypeSet::Single(SchemaType::Number) => item_types.push("number"),
                            SchemaTypeSet::Single(SchemaType::Boolean) => {
                                item_types.push("boolean")
                            }
                            SchemaTypeSet::Single(SchemaType::Array) => item_types.push("array"),
                            SchemaTypeSet::Single(SchemaType::Object) => item_types.push("object"),
                            _ => item_types.push("string"), // fallback
                        }
                    } else {
                        item_types.push("string"); // fallback
                    }
                }
                ObjectOrReference::Ref { .. } => item_types.push("string"), // fallback for refs
            }
        }

        // Check if items is false (no additional items allowed)
        let items_is_false =
            matches!(items.as_ref().map(|i| i.as_ref()), Some(Schema::Boolean(b)) if !b.0);

        if items_is_false {
            // Exact array length required
            result.insert("minItems".to_string(), json!(prefix_count));
            result.insert("maxItems".to_string(), json!(prefix_count));
        }

        // Determine items schema based on prefixItems types
        let unique_types: std::collections::HashSet<_> = item_types.into_iter().collect();

        if unique_types.len() == 1 {
            // All items have same type
            let item_type = unique_types.into_iter().next().unwrap();
            result.insert("items".to_string(), json!({"type": item_type}));
        } else if unique_types.len() > 1 {
            // Mixed types, use oneOf
            let one_of: Vec<Value> = unique_types
                .into_iter()
                .map(|t| json!({"type": t}))
                .collect();
            result.insert("items".to_string(), json!({"oneOf": one_of}));
        }

        Ok(())
    }

    /// Converts the new oas3 Schema enum (which can be Boolean or Object) to draft-07 format.
    ///
    /// The oas3 crate now supports:
    /// - Schema::Object(ObjectOrReference<ObjectSchema>) - regular object schemas
    /// - Schema::Boolean(BooleanSchema) - true/false schemas for validation control
    ///
    /// For MCP compatibility (draft-07), we convert:
    /// - Boolean true -> allow any items (no items constraint)
    /// - Boolean false -> not handled here (should be handled by caller with array constraints)
    /// - Object schemas -> recursively convert to JSON Schema
    fn convert_items_schema_to_draft07(
        items_schema: &Schema,
        result: &mut serde_json::Map<String, Value>,
    ) -> Result<(), OpenApiError> {
        match items_schema {
            Schema::Boolean(boolean_schema) => {
                if boolean_schema.0 {
                    // items: true - allow any additional items (draft-07 default behavior)
                    // Don't set items constraint, which allows any items
                } else {
                    // items: false - no additional items allowed
                    // This should typically be handled in combination with prefixItems
                    // but if we see it alone, we set a restrictive constraint
                    result.insert("maxItems".to_string(), json!(0));
                }
            }
            Schema::Object(obj_ref) => {
                match obj_ref.as_ref() {
                    ObjectOrReference::Object(item_schema) => {
                        let mut items_result = serde_json::Map::new();
                        Self::convert_schema_to_json_schema(
                            &Schema::Object(Box::new(ObjectOrReference::Object(
                                item_schema.clone(),
                            ))),
                            &mut items_result,
                        )?;
                        result.insert("items".to_string(), Value::Object(items_result));
                    }
                    ObjectOrReference::Ref { .. } => {
                        // For references, default to string type
                        result.insert("items".to_string(), json!({"type": "string"}));
                    }
                }
            }
        }
        Ok(())
    }

    /// Convert oas3::Schema to JSON Schema properties
    fn convert_schema_to_json_schema(
        schema: &Schema,
        result: &mut serde_json::Map<String, Value>,
    ) -> Result<(), OpenApiError> {
        match schema {
            Schema::Object(obj_schema_ref) => {
                match obj_schema_ref.as_ref() {
                    ObjectOrReference::Object(obj_schema) => {
                        // Handle object schema type
                        if let Some(schema_type) = &obj_schema.schema_type {
                            match schema_type {
                                SchemaTypeSet::Single(SchemaType::String) => {
                                    result.insert("type".to_string(), json!("string"));
                                    if let Some(min_length) = obj_schema.min_length {
                                        result.insert("minLength".to_string(), json!(min_length));
                                    }
                                    if let Some(max_length) = obj_schema.max_length {
                                        result.insert("maxLength".to_string(), json!(max_length));
                                    }
                                    if let Some(pattern) = &obj_schema.pattern {
                                        result.insert("pattern".to_string(), json!(pattern));
                                    }
                                    if let Some(format) = &obj_schema.format {
                                        result.insert("format".to_string(), json!(format));
                                    }
                                }
                                SchemaTypeSet::Single(SchemaType::Number) => {
                                    result.insert("type".to_string(), json!("number"));
                                    if let Some(minimum) = &obj_schema.minimum {
                                        result.insert("minimum".to_string(), json!(minimum));
                                    }
                                    if let Some(maximum) = &obj_schema.maximum {
                                        result.insert("maximum".to_string(), json!(maximum));
                                    }
                                    if let Some(format) = &obj_schema.format {
                                        result.insert("format".to_string(), json!(format));
                                    }
                                }
                                SchemaTypeSet::Single(SchemaType::Integer) => {
                                    result.insert("type".to_string(), json!("integer"));
                                    if let Some(minimum) = &obj_schema.minimum {
                                        result.insert("minimum".to_string(), json!(minimum));
                                    }
                                    if let Some(maximum) = &obj_schema.maximum {
                                        result.insert("maximum".to_string(), json!(maximum));
                                    }
                                    if let Some(format) = &obj_schema.format {
                                        result.insert("format".to_string(), json!(format));
                                    }
                                }
                                SchemaTypeSet::Single(SchemaType::Boolean) => {
                                    result.insert("type".to_string(), json!("boolean"));
                                }
                                SchemaTypeSet::Single(SchemaType::Array) => {
                                    result.insert("type".to_string(), json!("array"));

                                    // Handle modern JSON Schema features (prefixItems + items:false)
                                    // and convert them to JSON Schema draft-07 compatible format for MCP tools.
                                    //
                                    // MCP uses JSON Schema draft-07 which doesn't support:
                                    // - prefixItems (introduced in draft 2020-12)
                                    // - items: false (boolean schemas introduced in draft 2019-09)
                                    //
                                    // We convert these to draft-07 equivalents:
                                    // - Use minItems/maxItems for exact array length constraints
                                    // - Convert prefixItems to regular items schema with oneOf if needed
                                    // - Document tuple nature in description

                                    if !obj_schema.prefix_items.is_empty() {
                                        // Handle prefixItems (tuple-like arrays)
                                        Self::convert_prefix_items_to_draft07(
                                            &obj_schema.prefix_items,
                                            &obj_schema.items,
                                            result,
                                        )?;
                                    } else if let Some(items) = &obj_schema.items {
                                        // Handle regular items field (now a Schema enum)
                                        Self::convert_items_schema_to_draft07(items, result)?;
                                    } else {
                                        // No items specified, default to accepting any items
                                        result
                                            .insert("items".to_string(), json!({"type": "string"}));
                                    }
                                }
                                SchemaTypeSet::Single(SchemaType::Object) => {
                                    result.insert("type".to_string(), json!("object"));
                                    result.insert("additionalProperties".to_string(), json!(true));
                                }
                                _ => {
                                    // Default for other types
                                    result.insert("type".to_string(), json!("string"));
                                }
                            }
                        } else {
                            // Default to object if no type specified
                            result.insert("type".to_string(), json!("object"));
                        }
                    }
                    ObjectOrReference::Ref { .. } => {
                        // For references, default to string
                        result.insert("type".to_string(), json!("string"));
                    }
                }
            }
            Schema::Boolean(_) => {
                // Boolean schema - allow any type
                result.insert("type".to_string(), json!("object"));
            }
        }

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
    use oas3::spec::{
        BooleanSchema, ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn,
        SchemaType, SchemaTypeSet,
    };
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
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        // Create a path parameter
        let param = Parameter {
            name: "petId".to_string(),
            location: ParameterIn::Path,
            description: Some("ID of pet to return".to_string()),
            required: Some(true),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                minimum: Some(serde_json::Number::from(1_i64)),
                format: Some("int64".to_string()),
                ..Default::default()
            })),
            example: None,
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        operation.parameters.push(ObjectOrReference::Object(param));

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

    #[test]
    fn test_convert_prefix_items_to_draft07_mixed_types() {
        // Test prefixItems with mixed types and items:false

        let prefix_items = vec![
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                format: Some("int32".to_string()),
                ..Default::default()
            }),
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
        ];

        // items: false (no additional items allowed)
        let items = Some(Box::new(Schema::Boolean(BooleanSchema(false))));

        let mut result = serde_json::Map::new();
        ToolGenerator::convert_prefix_items_to_draft07(&prefix_items, &items, &mut result).unwrap();

        // Should set exact array length
        assert_eq!(result.get("minItems"), Some(&json!(2)));
        assert_eq!(result.get("maxItems"), Some(&json!(2)));

        // Should use oneOf for mixed types
        let items_schema = result.get("items").unwrap();
        assert!(items_schema.get("oneOf").is_some());
        let one_of = items_schema.get("oneOf").unwrap().as_array().unwrap();
        assert_eq!(one_of.len(), 2);

        // Verify types are present
        let types: Vec<&str> = one_of
            .iter()
            .map(|v| v.get("type").unwrap().as_str().unwrap())
            .collect();
        assert!(types.contains(&"integer"));
        assert!(types.contains(&"string"));
    }

    #[test]
    fn test_convert_prefix_items_to_draft07_uniform_types() {
        // Test prefixItems with uniform types
        let prefix_items = vec![
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
        ];

        // items: false
        let items = Some(Box::new(Schema::Boolean(BooleanSchema(false))));

        let mut result = serde_json::Map::new();
        ToolGenerator::convert_prefix_items_to_draft07(&prefix_items, &items, &mut result).unwrap();

        // Should set exact array length
        assert_eq!(result.get("minItems"), Some(&json!(2)));
        assert_eq!(result.get("maxItems"), Some(&json!(2)));

        // Should use single type for uniform types
        let items_schema = result.get("items").unwrap();
        assert_eq!(items_schema.get("type"), Some(&json!("string")));
        assert!(items_schema.get("oneOf").is_none());
    }

    #[test]
    fn test_convert_items_schema_to_draft07_boolean_true() {
        // Test items: true (allow any additional items)
        let items_schema = Schema::Boolean(BooleanSchema(true));
        let mut result = serde_json::Map::new();

        ToolGenerator::convert_items_schema_to_draft07(&items_schema, &mut result).unwrap();

        // Should not add any constraints (allows any items)
        assert!(result.get("items").is_none());
        assert!(result.get("maxItems").is_none());
    }

    #[test]
    fn test_convert_items_schema_to_draft07_boolean_false() {
        // Test items: false (no additional items allowed)
        let items_schema = Schema::Boolean(BooleanSchema(false));
        let mut result = serde_json::Map::new();

        ToolGenerator::convert_items_schema_to_draft07(&items_schema, &mut result).unwrap();

        // Should set maxItems to 0
        assert_eq!(result.get("maxItems"), Some(&json!(0)));
    }

    #[test]
    fn test_convert_items_schema_to_draft07_object_schema() {
        // Test items with object schema
        let item_schema = ObjectSchema {
            schema_type: Some(SchemaTypeSet::Single(SchemaType::Number)),
            minimum: Some(serde_json::Number::from(0)),
            ..Default::default()
        };

        let items_schema = Schema::Object(Box::new(ObjectOrReference::Object(item_schema)));
        let mut result = serde_json::Map::new();

        ToolGenerator::convert_items_schema_to_draft07(&items_schema, &mut result).unwrap();

        // Should convert to proper items schema
        let items_value = result.get("items").unwrap();
        assert_eq!(items_value.get("type"), Some(&json!("number")));
        assert_eq!(items_value.get("minimum"), Some(&json!(0)));
    }

    #[test]
    fn test_array_with_prefix_items_integration() {
        // Integration test: parameter with prefixItems and items:false
        let param = Parameter {
            name: "coordinates".to_string(),
            location: ParameterIn::Query,
            description: Some("X,Y coordinates as tuple".to_string()),
            required: Some(true),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                prefix_items: vec![
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::Number)),
                        format: Some("double".to_string()),
                        ..Default::default()
                    }),
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::Number)),
                        format: Some("double".to_string()),
                        ..Default::default()
                    }),
                ],
                items: Some(Box::new(Schema::Boolean(BooleanSchema(false)))),
                ..Default::default()
            })),
            example: None,
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let result = ToolGenerator::convert_parameter_schema(&param, "query").unwrap();

        // Verify the result
        assert_eq!(result.get("type"), Some(&json!("array")));
        assert_eq!(result.get("minItems"), Some(&json!(2)));
        assert_eq!(result.get("maxItems"), Some(&json!(2)));
        assert_eq!(
            result.get("items").unwrap().get("type"),
            Some(&json!("number"))
        );
        assert_eq!(
            result.get("description"),
            Some(&json!("X,Y coordinates as tuple"))
        );
    }

    #[test]
    fn test_array_with_regular_items_schema() {
        // Test regular array with object schema items (not boolean)
        let param = Parameter {
            name: "tags".to_string(),
            location: ParameterIn::Query,
            description: Some("List of tags".to_string()),
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                items: Some(Box::new(Schema::Object(Box::new(
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                        min_length: Some(1),
                        max_length: Some(50),
                        ..Default::default()
                    }),
                )))),
                ..Default::default()
            })),
            example: None,
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let result = ToolGenerator::convert_parameter_schema(&param, "query").unwrap();

        // Verify the result
        assert_eq!(result.get("type"), Some(&json!("array")));
        let items = result.get("items").unwrap();
        assert_eq!(items.get("type"), Some(&json!("string")));
        assert_eq!(items.get("minLength"), Some(&json!(1)));
        assert_eq!(items.get("maxLength"), Some(&json!(50)));
    }
}
