use crate::openapi_spec::{OpenApiOperation, OpenApiParameter};
use crate::server::ToolMetadata;
use crate::tool_generator::ToolGenerator;
use serde_json::{Value, json};

/// Legacy function for backward compatibility
/// Use ToolGenerator::generate_tool_metadata for new code
pub fn convert_path_to_tool_metadata(
    path_info: Value,
    method: String,
    path: String,
) -> Result<ToolMetadata, serde_json::Error> {
    // Convert to OpenApiOperation and use the new generator
    let operation = convert_to_operation(path_info, method, path)?;
    ToolGenerator::generate_tool_metadata(&operation).map_err(|e| {
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })
}

/// Convert legacy path_info format to OpenApiOperation
fn convert_to_operation(
    path_info: Value,
    method: String,
    path: String,
) -> Result<OpenApiOperation, serde_json::Error> {
    let operation_id = path_info["operationId"]
        .as_str()
        .unwrap_or(&format!(
            "{}_{}",
            method,
            path.replace('/', "_").replace(['{', '}'], "")
        ))
        .to_string();

    let summary = path_info["summary"].as_str().map(|s| s.to_string());
    let description = path_info["description"].as_str().map(|s| s.to_string());

    let mut parameters = Vec::new();
    if let Some(params_array) = path_info["parameters"].as_array() {
        for param in params_array {
            if let Ok(parameter) = convert_to_parameter(param.clone()) {
                parameters.push(parameter);
            }
        }
    }

    Ok(OpenApiOperation {
        operation_id,
        method,
        path,
        summary,
        description,
        parameters,
    })
}

/// Convert legacy parameter format to OpenApiParameter
fn convert_to_parameter(param_value: Value) -> Result<OpenApiParameter, serde_json::Error> {
    let name = param_value["name"]
        .as_str()
        .ok_or_else(|| {
            serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Parameter missing 'name'",
            ))
        })?
        .to_string();

    let location_str = param_value["in"].as_str().unwrap_or("query"); // Default to query if not specified
    let location = crate::openapi_spec::ParameterLocation::try_from(location_str)
        .unwrap_or(crate::openapi_spec::ParameterLocation::Query);

    let required = param_value["required"].as_bool().unwrap_or(false);
    let description = param_value["description"].as_str().map(|s| s.to_string());

    let schema = param_value["schema"].clone();
    let schema = if schema.is_null() {
        json!({"type": "string"})
    } else {
        schema
    };

    let param_type = schema["type"].as_str().unwrap_or("string").to_string();

    Ok(OpenApiParameter {
        name,
        location,
        required,
        param_type,
        description,
        schema,
    })
}
