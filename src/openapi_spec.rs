use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use crate::tool_generator::ToolGenerator;
use openapiv3::OpenAPI;
use reqwest::Method;
use serde_json::Value;
use url::Url;

/// OpenAPI specification wrapper that provides convenience methods
/// for working with openapiv3::OpenAPI
#[derive(Debug, Clone)]
pub struct OpenApiSpec {
    pub spec: OpenAPI,
}

impl OpenApiSpec {
    /// Load and parse an OpenAPI specification from a URL
    pub async fn from_url(url: &Url) -> Result<Self, OpenApiError> {
        let client = reqwest::Client::new();
        let response = client.get(url.clone()).send().await?;
        let text = response.text().await?;
        let spec: OpenAPI = serde_json::from_str(&text)?;

        Ok(OpenApiSpec { spec })
    }

    /// Load and parse an OpenAPI specification from a file
    pub async fn from_file(path: &str) -> Result<Self, OpenApiError> {
        let content = tokio::fs::read_to_string(path).await?;
        let spec: OpenAPI = serde_json::from_str(&content)?;

        Ok(OpenApiSpec { spec })
    }

    /// Parse an OpenAPI specification from a JSON value
    pub fn from_value(json_value: Value) -> Result<Self, OpenApiError> {
        let spec: OpenAPI = serde_json::from_value(json_value)?;
        Ok(OpenApiSpec { spec })
    }

    /// Convert all operations to MCP tool metadata
    pub fn to_tool_metadata(&self) -> Result<Vec<ToolMetadata>, OpenApiError> {
        let mut tools = Vec::new();

        for (path, path_item_ref) in &self.spec.paths.paths {
            // Handle ReferenceOr<PathItem>
            let path_item = match path_item_ref {
                openapiv3::ReferenceOr::Item(item) => item,
                openapiv3::ReferenceOr::Reference { .. } => continue, // Skip references for now
            };

            // Handle operations in the path item
            let operations = [
                (Method::GET, &path_item.get),
                (Method::POST, &path_item.post),
                (Method::PUT, &path_item.put),
                (Method::DELETE, &path_item.delete),
                (Method::PATCH, &path_item.patch),
                (Method::HEAD, &path_item.head),
                (Method::OPTIONS, &path_item.options),
                (Method::TRACE, &path_item.trace),
            ];

            for (method, operation_ref) in operations {
                if let Some(operation) = operation_ref {
                    let tool_metadata = ToolGenerator::generate_tool_metadata(
                        operation,
                        method.to_string(),
                        path.clone(),
                    )?;
                    tools.push(tool_metadata);
                }
            }
        }

        Ok(tools)
    }

    /// Get operation by operation ID
    pub fn get_operation(
        &self,
        operation_id: &str,
    ) -> Option<(&openapiv3::Operation, String, String)> {
        for (path, path_item_ref) in &self.spec.paths.paths {
            // Handle ReferenceOr<PathItem>
            let path_item = match path_item_ref {
                openapiv3::ReferenceOr::Item(item) => item,
                openapiv3::ReferenceOr::Reference { .. } => continue, // Skip references for now
            };

            let operations = [
                (Method::GET, &path_item.get),
                (Method::POST, &path_item.post),
                (Method::PUT, &path_item.put),
                (Method::DELETE, &path_item.delete),
                (Method::PATCH, &path_item.patch),
                (Method::HEAD, &path_item.head),
                (Method::OPTIONS, &path_item.options),
                (Method::TRACE, &path_item.trace),
            ];

            for (method, operation_ref) in operations {
                if let Some(operation) = operation_ref {
                    let default_id = format!(
                        "{}_{}",
                        method,
                        path.replace('/', "_").replace(['{', '}'], "")
                    );
                    let op_id = operation.operation_id.as_deref().unwrap_or(&default_id);

                    if op_id == operation_id {
                        return Some((operation, method.to_string(), path.clone()));
                    }
                }
            }
        }
        None
    }

    /// Get all operation IDs
    pub fn get_operation_ids(&self) -> Vec<String> {
        let mut operation_ids = Vec::new();

        for (path, path_item_ref) in &self.spec.paths.paths {
            // Handle ReferenceOr<PathItem>
            let path_item = match path_item_ref {
                openapiv3::ReferenceOr::Item(item) => item,
                openapiv3::ReferenceOr::Reference { .. } => continue, // Skip references for now
            };

            let operations = [
                (Method::GET, &path_item.get),
                (Method::POST, &path_item.post),
                (Method::PUT, &path_item.put),
                (Method::DELETE, &path_item.delete),
                (Method::PATCH, &path_item.patch),
                (Method::HEAD, &path_item.head),
                (Method::OPTIONS, &path_item.options),
                (Method::TRACE, &path_item.trace),
            ];

            for (method, operation_ref) in operations {
                if let Some(operation) = operation_ref {
                    let default_id = format!(
                        "{}_{}",
                        method,
                        path.replace('/', "_").replace(['{', '}'], "")
                    );
                    let op_id = operation.operation_id.as_deref().unwrap_or(&default_id);
                    operation_ids.push(op_id.to_string());
                }
            }
        }

        operation_ids
    }
}
