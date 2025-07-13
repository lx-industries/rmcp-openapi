use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use crate::tool_generator::ToolGenerator;
use oas3::Spec;
use reqwest::Method;
use serde_json::Value;
use url::Url;

#[derive(Debug, Clone)]
pub enum OpenApiSpecLocation {
    File(PathBuf),
    Url(Url),
}

impl FromStr for OpenApiSpecLocation {
    type Err = OpenApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            let url =
                Url::parse(s).map_err(|e| OpenApiError::InvalidUrl(format!("Invalid URL: {e}")))?;
            Ok(OpenApiSpecLocation::Url(url))
        } else {
            let path = PathBuf::from(s);
            Ok(OpenApiSpecLocation::File(path))
        }
    }
}

impl OpenApiSpecLocation {
    pub async fn load_spec(&self) -> Result<OpenApiSpec, OpenApiError> {
        match self {
            OpenApiSpecLocation::File(path) => {
                OpenApiSpec::from_file(path.to_str().ok_or_else(|| {
                    OpenApiError::InvalidPath("Invalid file path encoding".to_string())
                })?)
                .await
            }
            OpenApiSpecLocation::Url(url) => OpenApiSpec::from_url(url).await,
        }
    }
}

impl fmt::Display for OpenApiSpecLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenApiSpecLocation::File(path) => write!(f, "{}", path.display()),
            OpenApiSpecLocation::Url(url) => write!(f, "{url}"),
        }
    }
}

/// OpenAPI specification wrapper that provides convenience methods
/// for working with oas3::Spec
#[derive(Debug, Clone)]
pub struct OpenApiSpec {
    pub spec: Spec,
}

impl OpenApiSpec {
    /// Load and parse an OpenAPI specification from a URL
    pub async fn from_url(url: &Url) -> Result<Self, OpenApiError> {
        let client = reqwest::Client::new();
        let response = client.get(url.clone()).send().await?;
        let text = response.text().await?;
        let spec: Spec = serde_json::from_str(&text)?;

        Ok(OpenApiSpec { spec })
    }

    /// Load and parse an OpenAPI specification from a file
    pub async fn from_file(path: &str) -> Result<Self, OpenApiError> {
        let content = tokio::fs::read_to_string(path).await?;
        let spec: Spec = serde_json::from_str(&content)?;

        Ok(OpenApiSpec { spec })
    }

    /// Parse an OpenAPI specification from a JSON value
    pub fn from_value(json_value: Value) -> Result<Self, OpenApiError> {
        let spec: Spec = serde_json::from_value(json_value)?;
        Ok(OpenApiSpec { spec })
    }

    /// Convert all operations to MCP tool metadata
    pub fn to_tool_metadata(&self) -> Result<Vec<ToolMetadata>, OpenApiError> {
        let mut tools = Vec::new();

        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths {
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
                            &self.spec,
                        )?;
                        tools.push(tool_metadata);
                    }
                }
            }
        }

        Ok(tools)
    }

    /// Get operation by operation ID
    pub fn get_operation(
        &self,
        operation_id: &str,
    ) -> Option<(&oas3::spec::Operation, String, String)> {
        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths {
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
        }
        None
    }

    /// Get all operation IDs
    pub fn get_operation_ids(&self) -> Vec<String> {
        let mut operation_ids = Vec::new();

        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths {
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
        }

        operation_ids
    }
}
