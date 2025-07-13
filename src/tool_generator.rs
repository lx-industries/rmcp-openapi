use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use oas3::spec::{
    ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn, RequestBody, Schema,
    SchemaType, SchemaTypeSet, Spec,
};

/// Tool generator for creating MCP tools from `OpenAPI` operations
pub struct ToolGenerator;

impl ToolGenerator {
    /// Generate tool metadata from an `OpenAPI` operation
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be converted to tool metadata
    pub fn generate_tool_metadata(
        operation: &Operation,
        method: String,
        path: String,
        spec: &Spec,
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
        let parameters = Self::generate_parameter_schema(
            &operation.parameters,
            &method,
            &operation.request_body,
            spec,
        )?;

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

    /// Resolve a $ref reference to get the actual schema
    ///
    /// # Arguments
    /// * `ref_path` - The reference path (e.g., "#/components/schemas/Pet")
    /// * `spec` - The OpenAPI specification
    /// * `visited` - Set of already visited references to detect circular references
    ///
    /// # Returns
    /// The resolved ObjectSchema or an error if the reference is invalid or circular
    fn resolve_reference(
        ref_path: &str,
        spec: &Spec,
        visited: &mut HashSet<String>,
    ) -> Result<ObjectSchema, OpenApiError> {
        // Check for circular reference
        if visited.contains(ref_path) {
            return Err(OpenApiError::ToolGeneration(format!(
                "Circular reference detected: {ref_path}"
            )));
        }

        // Add to visited set
        visited.insert(ref_path.to_string());

        // Parse the reference path
        // Currently only supporting local references like "#/components/schemas/Pet"
        if !ref_path.starts_with("#/components/schemas/") {
            return Err(OpenApiError::ToolGeneration(format!(
                "Unsupported reference format: {ref_path}. Only #/components/schemas/ references are supported"
            )));
        }

        let schema_name = ref_path.strip_prefix("#/components/schemas/").unwrap();

        // Get the schema from components
        let components = spec.components.as_ref().ok_or_else(|| {
            OpenApiError::ToolGeneration(format!(
                "Reference {ref_path} points to components, but spec has no components section"
            ))
        })?;

        let schema_ref = components.schemas.get(schema_name).ok_or_else(|| {
            OpenApiError::ToolGeneration(format!(
                "Schema '{schema_name}' not found in components/schemas"
            ))
        })?;

        // Resolve the schema reference
        let resolved_schema = match schema_ref {
            ObjectOrReference::Object(obj_schema) => obj_schema.clone(),
            ObjectOrReference::Ref {
                ref_path: nested_ref,
            } => {
                // Recursively resolve nested references
                Self::resolve_reference(nested_ref, spec, visited)?
            }
        };

        // Remove from visited set before returning (for other resolution paths)
        visited.remove(ref_path);

        Ok(resolved_schema)
    }

    /// Generate JSON Schema for tool parameters
    fn generate_parameter_schema(
        parameters: &[ObjectOrReference<Parameter>],
        _method: &str,
        request_body: &Option<ObjectOrReference<RequestBody>>,
        spec: &Spec,
    ) -> Result<Value, OpenApiError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        // Group parameters by location
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();
        let mut cookie_params = Vec::new();

        for param_ref in parameters {
            let param = match param_ref {
                ObjectOrReference::Object(param) => param,
                ObjectOrReference::Ref { ref_path } => {
                    // Try to resolve parameter reference
                    // Note: Parameter references are rare and not supported yet in this implementation
                    // For now, we'll continue to skip them but log a warning
                    eprintln!("Warning: Parameter reference not resolved: {ref_path}");
                    continue;
                }
            };

            match &param.location {
                ParameterIn::Query => query_params.push(param),
                ParameterIn::Header => header_params.push(param),
                ParameterIn::Path => path_params.push(param),
                ParameterIn::Cookie => cookie_params.push(param),
            }
        }

        // Process path parameters (always required)
        for param in path_params {
            let param_schema = Self::convert_parameter_schema(param, "path", spec)?;
            properties.insert(param.name.clone(), param_schema);
            required.push(param.name.clone());
        }

        // Process query parameters
        for param in &query_params {
            let param_schema = Self::convert_parameter_schema(param, "query", spec)?;
            properties.insert(param.name.clone(), param_schema);
            if param.required.unwrap_or(false) {
                required.push(param.name.clone());
            }
        }

        // Process header parameters (optional by default unless explicitly required)
        for param in &header_params {
            let mut param_schema = Self::convert_parameter_schema(param, "header", spec)?;

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
            let mut param_schema = Self::convert_parameter_schema(param, "cookie", spec)?;

            // Add location metadata for cookies
            if let Value::Object(ref mut obj) = param_schema {
                obj.insert("x-location".to_string(), json!("cookie"));
            }

            properties.insert(format!("cookie_{}", param.name), param_schema);
            if param.required.unwrap_or(false) {
                required.push(format!("cookie_{}", param.name));
            }
        }

        // Add request body parameter if defined in the OpenAPI spec
        if let Some(request_body) = request_body {
            if let Some((body_schema, is_required)) =
                Self::convert_request_body_to_json_schema(request_body, spec)?
            {
                properties.insert("request_body".to_string(), body_schema);
                if is_required {
                    required.push("request_body".to_string());
                }
            }
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

    /// Convert `OpenAPI` parameter schema to JSON Schema for MCP tools
    fn convert_parameter_schema(
        param: &Parameter,
        location: &str,
        spec: &Spec,
    ) -> Result<Value, OpenApiError> {
        let mut result = serde_json::Map::new();

        // Handle the parameter schema
        if let Some(schema_ref) = &param.schema {
            match schema_ref {
                ObjectOrReference::Object(obj_schema) => {
                    Self::convert_schema_to_json_schema(
                        &Schema::Object(Box::new(ObjectOrReference::Object(obj_schema.clone()))),
                        &mut result,
                        spec,
                    )?;
                }
                ObjectOrReference::Ref { ref_path } => {
                    // Resolve the reference and convert to JSON schema
                    let mut visited = HashSet::new();
                    match Self::resolve_reference(ref_path, spec, &mut visited) {
                        Ok(resolved_schema) => {
                            Self::convert_schema_to_json_schema(
                                &Schema::Object(Box::new(ObjectOrReference::Object(
                                    resolved_schema,
                                ))),
                                &mut result,
                                spec,
                            )?;
                        }
                        Err(_) => {
                            // Fallback to string for unresolvable references
                            result.insert("type".to_string(), json!("string"));
                        }
                    }
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
        spec: &Spec,
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
                ObjectOrReference::Ref { ref_path } => {
                    // Try to resolve the reference
                    let mut visited = HashSet::new();
                    match Self::resolve_reference(ref_path, spec, &mut visited) {
                        Ok(resolved_schema) => {
                            // Extract the type immediately and store it as a string
                            if let Some(schema_type_set) = &resolved_schema.schema_type {
                                match schema_type_set {
                                    SchemaTypeSet::Single(SchemaType::String) => {
                                        item_types.push("string")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Integer) => {
                                        item_types.push("integer")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Number) => {
                                        item_types.push("number")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Boolean) => {
                                        item_types.push("boolean")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Array) => {
                                        item_types.push("array")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Object) => {
                                        item_types.push("object")
                                    }
                                    _ => item_types.push("string"), // fallback
                                }
                            } else {
                                item_types.push("string"); // fallback
                            }
                        }
                        Err(_) => {
                            // Fallback to string for unresolvable references
                            item_types.push("string");
                        }
                    }
                }
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
        spec: &Spec,
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
            Schema::Object(obj_ref) => match obj_ref.as_ref() {
                ObjectOrReference::Object(item_schema) => {
                    let mut items_result = serde_json::Map::new();
                    Self::convert_schema_to_json_schema(
                        &Schema::Object(Box::new(ObjectOrReference::Object(item_schema.clone()))),
                        &mut items_result,
                        spec,
                    )?;
                    result.insert("items".to_string(), Value::Object(items_result));
                }
                ObjectOrReference::Ref { ref_path } => {
                    // Try to resolve reference and convert to JSON schema
                    let mut visited = HashSet::new();
                    match Self::resolve_reference(ref_path, spec, &mut visited) {
                        Ok(resolved_schema) => {
                            let mut items_result = serde_json::Map::new();
                            Self::convert_schema_to_json_schema(
                                &Schema::Object(Box::new(ObjectOrReference::Object(
                                    resolved_schema,
                                ))),
                                &mut items_result,
                                spec,
                            )?;
                            result.insert("items".to_string(), Value::Object(items_result));
                        }
                        Err(_) => {
                            // Fallback to string type for unresolvable references
                            result.insert("items".to_string(), json!({"type": "string"}));
                        }
                    }
                }
            },
        }
        Ok(())
    }

    /// Convert request body from OpenAPI to JSON Schema for MCP tools
    fn convert_request_body_to_json_schema(
        request_body_ref: &ObjectOrReference<RequestBody>,
        spec: &Spec,
    ) -> Result<Option<(Value, bool)>, OpenApiError> {
        match request_body_ref {
            ObjectOrReference::Object(request_body) => {
                // Extract schema from request body content
                // Prioritize application/json content type
                let schema_info = request_body
                    .content
                    .get(mime::APPLICATION_JSON.as_ref())
                    .or_else(|| request_body.content.get("application/json"))
                    .or_else(|| {
                        // Fall back to first available content type
                        request_body.content.values().next()
                    });

                if let Some(media_type) = schema_info {
                    if let Some(schema_ref) = &media_type.schema {
                        let mut result = serde_json::Map::new();

                        // Convert the schema to JSON Schema
                        match schema_ref {
                            ObjectOrReference::Object(obj_schema) => {
                                Self::convert_schema_to_json_schema(
                                    &Schema::Object(Box::new(ObjectOrReference::Object(
                                        obj_schema.clone(),
                                    ))),
                                    &mut result,
                                    spec,
                                )?;
                            }
                            ObjectOrReference::Ref { ref_path } => {
                                // Resolve the reference and convert to JSON schema
                                let mut visited = HashSet::new();
                                match Self::resolve_reference(ref_path, spec, &mut visited) {
                                    Ok(resolved_schema) => {
                                        Self::convert_schema_to_json_schema(
                                            &Schema::Object(Box::new(ObjectOrReference::Object(
                                                resolved_schema,
                                            ))),
                                            &mut result,
                                            spec,
                                        )?;
                                    }
                                    Err(_) => {
                                        // Fallback to generic object for unresolvable references
                                        result.insert("type".to_string(), json!("object"));
                                        result.insert(
                                            "additionalProperties".to_string(),
                                            json!(true),
                                        );
                                    }
                                }
                            }
                        }

                        // Add description if available
                        if let Some(desc) = &request_body.description {
                            result.insert("description".to_string(), json!(desc));
                        } else {
                            result.insert("description".to_string(), json!("Request body data"));
                        }

                        // Add metadata
                        result.insert("x-location".to_string(), json!("body"));
                        result.insert(
                            "x-content-type".to_string(),
                            json!(mime::APPLICATION_JSON.as_ref()),
                        );

                        let required = request_body.required.unwrap_or(false);
                        Ok(Some((Value::Object(result), required)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            ObjectOrReference::Ref { .. } => {
                // For references, return a generic object schema
                let mut result = serde_json::Map::new();
                result.insert("type".to_string(), json!("object"));
                result.insert("additionalProperties".to_string(), json!(true));
                result.insert("description".to_string(), json!("Request body data"));
                result.insert("x-location".to_string(), json!("body"));
                result.insert(
                    "x-content-type".to_string(),
                    json!(mime::APPLICATION_JSON.as_ref()),
                );

                Ok(Some((Value::Object(result), false)))
            }
        }
    }

    /// Convert `oas3::Schema` to JSON Schema properties
    fn convert_schema_to_json_schema(
        schema: &Schema,
        result: &mut serde_json::Map<String, Value>,
        spec: &Spec,
    ) -> Result<(), OpenApiError> {
        match schema {
            Schema::Object(obj_schema_ref) => match obj_schema_ref.as_ref() {
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
                                        spec,
                                    )?;
                                } else if let Some(items) = &obj_schema.items {
                                    // Handle regular items field (now a Schema enum)
                                    Self::convert_items_schema_to_draft07(items, result, spec)?;
                                } else {
                                    // No items specified, default to accepting any items
                                    result.insert("items".to_string(), json!({"type": "string"}));
                                }
                            }
                            SchemaTypeSet::Single(SchemaType::Object) => {
                                result.insert("type".to_string(), json!("object"));

                                // Convert properties if present
                                if !obj_schema.properties.is_empty() {
                                    let mut properties_map = serde_json::Map::new();
                                    for (prop_name, prop_schema) in &obj_schema.properties {
                                        let mut prop_result = serde_json::Map::new();
                                        match prop_schema {
                                            ObjectOrReference::Object(prop_obj_schema) => {
                                                Self::convert_schema_to_json_schema(
                                                    &Schema::Object(Box::new(
                                                        ObjectOrReference::Object(
                                                            prop_obj_schema.clone(),
                                                        ),
                                                    )),
                                                    &mut prop_result,
                                                    spec,
                                                )?;
                                            }
                                            ObjectOrReference::Ref { ref_path } => {
                                                // Try to resolve reference and convert to JSON schema
                                                let mut visited = HashSet::new();
                                                match Self::resolve_reference(
                                                    ref_path,
                                                    spec,
                                                    &mut visited,
                                                ) {
                                                    Ok(resolved_schema) => {
                                                        Self::convert_schema_to_json_schema(
                                                            &Schema::Object(Box::new(
                                                                ObjectOrReference::Object(
                                                                    resolved_schema,
                                                                ),
                                                            )),
                                                            &mut prop_result,
                                                            spec,
                                                        )?;
                                                    }
                                                    Err(_) => {
                                                        // Fallback to string for unresolvable references
                                                        prop_result.insert(
                                                            "type".to_string(),
                                                            json!("string"),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        properties_map
                                            .insert(prop_name.clone(), Value::Object(prop_result));
                                    }
                                    result.insert(
                                        "properties".to_string(),
                                        Value::Object(properties_map),
                                    );

                                    // Only set additionalProperties to false if we have explicit properties
                                    result.insert("additionalProperties".to_string(), json!(false));
                                } else {
                                    // No properties defined, allow additional properties
                                    result.insert("additionalProperties".to_string(), json!(true));
                                }

                                // Add required array if present
                                if !obj_schema.required.is_empty() {
                                    result
                                        .insert("required".to_string(), json!(obj_schema.required));
                                }
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
                ObjectOrReference::Ref { ref_path } => {
                    // Try to resolve reference and convert to JSON schema
                    let mut visited = HashSet::new();
                    match Self::resolve_reference(ref_path, spec, &mut visited) {
                        Ok(resolved_schema) => {
                            Self::convert_schema_to_json_schema(
                                &Schema::Object(Box::new(ObjectOrReference::Object(
                                    resolved_schema,
                                ))),
                                result,
                                spec,
                            )?;
                        }
                        Err(_) => {
                            // Fallback to string for unresolvable references
                            result.insert("type".to_string(), json!("string"));
                        }
                    }
                }
            },
            Schema::Boolean(_) => {
                // Boolean schema - allow any type
                result.insert("type".to_string(), json!("object"));
            }
        }

        Ok(())
    }

    /// Extract parameter values from MCP tool call arguments
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are invalid or missing required parameters
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
        if let Some(timeout) = args.get("timeout_seconds").and_then(Value::as_u64) {
            config.timeout_seconds = u32::try_from(timeout).unwrap_or(u32::MAX);
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
                    parameter: (*required_param).to_string(),
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
        BooleanSchema, Components, MediaType, ObjectOrReference, ObjectSchema, Operation,
        Parameter, ParameterIn, RequestBody, SchemaType, SchemaTypeSet, Spec,
    };
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    /// Create a minimal test OpenAPI spec for testing purposes
    fn create_test_spec() -> Spec {
        Spec {
            openapi: "3.0.0".to_string(),
            info: oas3::spec::Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                summary: None,
                description: Some("Test API for unit tests".to_string()),
                terms_of_service: None,
                contact: None,
                license: None,
                extensions: Default::default(),
            },
            components: Some(Components {
                schemas: BTreeMap::new(),
                responses: BTreeMap::new(),
                parameters: BTreeMap::new(),
                examples: BTreeMap::new(),
                request_bodies: BTreeMap::new(),
                headers: BTreeMap::new(),
                security_schemes: BTreeMap::new(),
                links: BTreeMap::new(),
                callbacks: BTreeMap::new(),
                path_items: BTreeMap::new(),
                extensions: Default::default(),
            }),
            servers: vec![],
            paths: None,
            external_docs: None,
            tags: vec![],
            security: vec![],
            webhooks: BTreeMap::new(),
            extensions: Default::default(),
        }
    }

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

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pet/{petId}".to_string(),
            &spec,
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
        let spec = create_test_spec();
        ToolGenerator::convert_prefix_items_to_draft07(&prefix_items, &items, &mut result, &spec)
            .unwrap();

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
        let spec = create_test_spec();
        ToolGenerator::convert_prefix_items_to_draft07(&prefix_items, &items, &mut result, &spec)
            .unwrap();

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
        let spec = create_test_spec();

        ToolGenerator::convert_items_schema_to_draft07(&items_schema, &mut result, &spec).unwrap();

        // Should not add any constraints (allows any items)
        assert!(result.get("items").is_none());
        assert!(result.get("maxItems").is_none());
    }

    #[test]
    fn test_convert_items_schema_to_draft07_boolean_false() {
        // Test items: false (no additional items allowed)
        let items_schema = Schema::Boolean(BooleanSchema(false));
        let mut result = serde_json::Map::new();
        let spec = create_test_spec();

        ToolGenerator::convert_items_schema_to_draft07(&items_schema, &mut result, &spec).unwrap();

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
        let spec = create_test_spec();

        ToolGenerator::convert_items_schema_to_draft07(&items_schema, &mut result, &spec).unwrap();

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

        let spec = create_test_spec();
        let result = ToolGenerator::convert_parameter_schema(&param, "query", &spec).unwrap();

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

        let spec = create_test_spec();
        let result = ToolGenerator::convert_parameter_schema(&param, "query", &spec).unwrap();

        // Verify the result
        assert_eq!(result.get("type"), Some(&json!("array")));
        let items = result.get("items").unwrap();
        assert_eq!(items.get("type"), Some(&json!("string")));
        assert_eq!(items.get("minLength"), Some(&json!(1)));
        assert_eq!(items.get("maxLength"), Some(&json!(50)));
    }

    #[test]
    fn test_request_body_object_schema() {
        // Test with object request body
        let operation = Operation {
            operation_id: Some("createPet".to_string()),
            summary: Some("Create a new pet".to_string()),
            description: Some("Creates a new pet in the store".to_string()),
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("Pet object that needs to be added to the store".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(true),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "post".to_string(),
            "/pets".to_string(),
            &spec,
        )
        .unwrap();

        // Check that request_body is in properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(properties.contains_key("request_body"));

        // Check that request_body is required
        let required = metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.contains(&json!("request_body")));

        // Check request body schema
        let request_body_schema = properties.get("request_body").unwrap();
        assert_eq!(request_body_schema.get("type"), Some(&json!("object")));
        assert_eq!(
            request_body_schema.get("description"),
            Some(&json!("Pet object that needs to be added to the store"))
        );
        assert_eq!(request_body_schema.get("x-location"), Some(&json!("body")));

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_array_schema() {
        // Test with array request body
        let operation = Operation {
            operation_id: Some("createPets".to_string()),
            summary: Some("Create multiple pets".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("Array of pet objects".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                                items: Some(Box::new(Schema::Object(Box::new(
                                    ObjectOrReference::Object(ObjectSchema {
                                        schema_type: Some(SchemaTypeSet::Single(
                                            SchemaType::Object,
                                        )),
                                        ..Default::default()
                                    }),
                                )))),
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(false),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "post".to_string(),
            "/pets/batch".to_string(),
            &spec,
        )
        .unwrap();

        // Check that request_body is in properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(properties.contains_key("request_body"));

        // Check that request_body is NOT required (required: false)
        let required = metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(!required.contains(&json!("request_body")));

        // Check request body schema
        let request_body_schema = properties.get("request_body").unwrap();
        assert_eq!(request_body_schema.get("type"), Some(&json!("array")));
        assert_eq!(
            request_body_schema.get("description"),
            Some(&json!("Array of pet objects"))
        );

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_string_schema() {
        // Test with string request body
        let operation = Operation {
            operation_id: Some("updatePetName".to_string()),
            summary: Some("Update pet name".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: None,
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "text/plain".to_string(),
                        MediaType {
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                                min_length: Some(1),
                                max_length: Some(100),
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(true),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "put".to_string(),
            "/pets/{petId}/name".to_string(),
            &spec,
        )
        .unwrap();

        // Check request body schema
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        assert_eq!(request_body_schema.get("type"), Some(&json!("string")));
        assert_eq!(request_body_schema.get("minLength"), Some(&json!(1)));
        assert_eq!(request_body_schema.get("maxLength"), Some(&json!(100)));
        assert_eq!(
            request_body_schema.get("description"),
            Some(&json!("Request body data"))
        );

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_ref_schema() {
        // Test with reference request body
        let operation = Operation {
            operation_id: Some("updatePet".to_string()),
            summary: Some("Update existing pet".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Ref {
                ref_path: "#/components/requestBodies/PetBody".to_string(),
            }),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "put".to_string(),
            "/pets/{petId}".to_string(),
            &spec,
        )
        .unwrap();

        // Check that request_body uses generic object schema for refs
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        assert_eq!(request_body_schema.get("type"), Some(&json!("object")));
        assert_eq!(
            request_body_schema.get("additionalProperties"),
            Some(&json!(true))
        );

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_no_request_body_for_get() {
        // Test that GET operations don't get request body by default
        let operation = Operation {
            operation_id: Some("listPets".to_string()),
            summary: Some("List all pets".to_string()),
            description: None,
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

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pets".to_string(),
            &spec,
        )
        .unwrap();

        // Check that request_body is NOT in properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(!properties.contains_key("request_body"));

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_simple_object_with_properties() {
        // Test with simple object schema with a few properties
        let operation = Operation {
            operation_id: Some("updatePetStatus".to_string()),
            summary: Some("Update pet status".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("Pet status update".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                                properties: {
                                    let mut props = BTreeMap::new();
                                    props.insert(
                                        "status".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::String,
                                            )),
                                            ..Default::default()
                                        }),
                                    );
                                    props.insert(
                                        "reason".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::String,
                                            )),
                                            ..Default::default()
                                        }),
                                    );
                                    props
                                },
                                required: vec!["status".to_string()],
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(false),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "patch".to_string(),
            "/pets/{petId}/status".to_string(),
            &spec,
        )
        .unwrap();

        // Check request body schema - should have actual properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();

        // Check basic structure
        assert_eq!(request_body_schema.get("type"), Some(&json!("object")));
        assert_eq!(
            request_body_schema.get("description"),
            Some(&json!("Pet status update"))
        );

        // Check extracted properties
        let body_props = request_body_schema
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(body_props.len(), 2);
        assert!(body_props.contains_key("status"));
        assert!(body_props.contains_key("reason"));

        // Check required array from schema
        assert_eq!(
            request_body_schema.get("required"),
            Some(&json!(["status"]))
        );

        // Should not be in top-level required since request body itself is optional
        let required = metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(!required.contains(&json!("request_body")));

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_with_nested_properties() {
        // Test with complex nested object schema
        let operation = Operation {
            operation_id: Some("createUser".to_string()),
            summary: Some("Create a new user".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("User creation data".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                                properties: {
                                    let mut props = BTreeMap::new();
                                    props.insert(
                                        "name".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::String,
                                            )),
                                            ..Default::default()
                                        }),
                                    );
                                    props.insert(
                                        "age".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::Integer,
                                            )),
                                            minimum: Some(serde_json::Number::from(0)),
                                            maximum: Some(serde_json::Number::from(150)),
                                            ..Default::default()
                                        }),
                                    );
                                    props
                                },
                                required: vec!["name".to_string()],
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(true),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "post".to_string(),
            "/users".to_string(),
            &spec,
        )
        .unwrap();

        // Check request body schema
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        assert_eq!(request_body_schema.get("type"), Some(&json!("object")));

        // Check that properties were extracted
        assert!(request_body_schema.get("properties").is_some());
        let props = request_body_schema
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(props.contains_key("name"));
        assert!(props.contains_key("age"));

        // Check name property
        let name_prop = props.get("name").unwrap();
        assert_eq!(name_prop.get("type"), Some(&json!("string")));

        // Check age property
        let age_prop = props.get("age").unwrap();
        assert_eq!(age_prop.get("type"), Some(&json!("integer")));
        assert_eq!(age_prop.get("minimum"), Some(&json!(0)));
        assert_eq!(age_prop.get("maximum"), Some(&json!(150)));

        // Check required array
        assert_eq!(request_body_schema.get("required"), Some(&json!(["name"])));

        // With properties defined, additionalProperties should be false
        assert_eq!(
            request_body_schema.get("additionalProperties"),
            Some(&json!(false))
        );

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }
}
