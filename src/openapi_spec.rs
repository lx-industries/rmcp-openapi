use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use crate::tool_generator::ToolGenerator;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct OpenApiSpec {
    pub raw: Value,
    pub info: OpenApiInfo,
    pub operations: Vec<OpenApiOperation>,
}

#[derive(Debug, Clone)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OpenApiOperation {
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<OpenApiParameter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterLocation {
    Query,
    Path,
    Header,
    Cookie,
    FormData,
}

impl std::fmt::Display for ParameterLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterLocation::Query => write!(f, "query"),
            ParameterLocation::Path => write!(f, "path"),
            ParameterLocation::Header => write!(f, "header"),
            ParameterLocation::Cookie => write!(f, "cookie"),
            ParameterLocation::FormData => write!(f, "formData"),
        }
    }
}

impl std::convert::TryFrom<&str> for ParameterLocation {
    type Error = crate::error::OpenApiError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "query" => Ok(ParameterLocation::Query),
            "path" => Ok(ParameterLocation::Path),
            "header" => Ok(ParameterLocation::Header),
            "cookie" => Ok(ParameterLocation::Cookie),
            "formData" => Ok(ParameterLocation::FormData),
            _ => Err(crate::error::OpenApiError::InvalidParameterLocation(
                s.to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenApiParameter {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub param_type: String,
    pub description: Option<String>,
    pub schema: Value,
}

impl OpenApiSpec {
    /// Load and parse an OpenAPI specification from a URL
    pub async fn from_url(url: &str) -> Result<Self, OpenApiError> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let text = response.text().await?;
        let json_value: Value = serde_json::from_str(&text)?;

        Self::from_value(json_value)
    }

    /// Load and parse an OpenAPI specification from a file
    pub async fn from_file(path: &str) -> Result<Self, OpenApiError> {
        let content = tokio::fs::read_to_string(path).await?;
        let json_value: Value = serde_json::from_str(&content)?;

        Self::from_value(json_value)
    }

    /// Parse an OpenAPI specification from a JSON value
    pub fn from_value(json_value: Value) -> Result<Self, OpenApiError> {
        // Extract info section
        let info_obj = json_value
            .get("info")
            .ok_or_else(|| OpenApiError::Spec("Missing 'info' section".to_string()))?;

        let info = OpenApiInfo {
            title: info_obj
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown API")
                .to_string(),
            version: info_obj
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            description: info_obj
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Extract and parse all operations
        let mut operations = Vec::new();

        let paths = json_value
            .get("paths")
            .ok_or_else(|| OpenApiError::Spec("Missing 'paths' section".to_string()))?
            .as_object()
            .ok_or_else(|| OpenApiError::Spec("'paths' is not an object".to_string()))?;

        for (path, path_obj) in paths {
            let path_obj = path_obj
                .as_object()
                .ok_or_else(|| OpenApiError::Spec(format!("Path '{path}' is not an object")))?;

            for (method, operation_obj) in path_obj {
                // Skip non-HTTP methods (like parameters, summary, etc.)
                if ![
                    "get", "post", "put", "delete", "patch", "head", "options", "trace",
                ]
                .contains(&method.as_str())
                {
                    continue;
                }

                let operation =
                    Self::parse_operation(method.clone(), path.clone(), operation_obj.clone())?;
                operations.push(operation);
            }
        }

        Ok(OpenApiSpec {
            raw: json_value,
            info,
            operations,
        })
    }

    /// Parse a single operation from OpenAPI spec
    fn parse_operation(
        method: String,
        path: String,
        operation_value: Value,
    ) -> Result<OpenApiOperation, OpenApiError> {
        let operation_id = operation_value
            .get("operationId")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!(
                "{}_{}",
                method,
                path.replace('/', "_").replace(['{', '}'], "")
            ))
            .to_string();

        let summary = operation_value
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = operation_value
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut parameters = Vec::new();

        if let Some(params_array) = operation_value.get("parameters").and_then(|v| v.as_array()) {
            for param in params_array {
                if let Ok(parameter) = Self::parse_parameter(param.clone()) {
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

    /// Parse a parameter from OpenAPI spec
    fn parse_parameter(param_value: Value) -> Result<OpenApiParameter, OpenApiError> {
        let name = param_value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenApiError::Spec("Parameter missing 'name'".to_string()))?
            .to_string();

        let location_str = param_value
            .get("in")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenApiError::Spec("Parameter missing 'in'".to_string()))?;
        let location = ParameterLocation::try_from(location_str)?;

        let required = param_value
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let description = param_value
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let schema = param_value
            .get("schema")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"type": "string"}));

        let param_type = schema
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("string")
            .to_string();

        Ok(OpenApiParameter {
            name,
            location,
            required,
            param_type,
            description,
            schema,
        })
    }

    /// Convert all operations to MCP tool metadata
    pub fn to_tool_metadata(&self) -> Result<Vec<ToolMetadata>, OpenApiError> {
        let mut tools = Vec::new();

        for operation in &self.operations {
            let tool_metadata = ToolGenerator::generate_tool_metadata(operation)?;
            tools.push(tool_metadata);
        }

        Ok(tools)
    }

    /// Get operation by operation ID
    pub fn get_operation(&self, operation_id: &str) -> Option<&OpenApiOperation> {
        self.operations
            .iter()
            .find(|op| op.operation_id == operation_id)
    }

    /// Get all operation IDs
    pub fn get_operation_ids(&self) -> Vec<String> {
        self.operations
            .iter()
            .map(|op| op.operation_id.clone())
            .collect()
    }
}
