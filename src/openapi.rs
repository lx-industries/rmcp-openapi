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
    pub fn to_tool_metadata(
        &self,
        tag_filter: Option<&[String]>,
    ) -> Result<Vec<ToolMetadata>, OpenApiError> {
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
                        // Filter by tags if specified
                        if let Some(filter_tags) = tag_filter {
                            if !operation.tags.is_empty() {
                                if !operation.tags.iter().any(|tag| filter_tags.contains(tag)) {
                                    continue; // Skip this operation
                                }
                            } else {
                                continue; // Skip operations without tags when filtering
                            }
                        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_spec_with_tags() -> OpenApiSpec {
        let spec_json = json!({
            "openapi": "3.0.3",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/pets": {
                    "get": {
                        "operationId": "listPets",
                        "tags": ["pet", "list"],
                        "responses": {
                            "200": {
                                "description": "List of pets"
                            }
                        }
                    },
                    "post": {
                        "operationId": "createPet",
                        "tags": ["pet"],
                        "responses": {
                            "201": {
                                "description": "Pet created"
                            }
                        }
                    }
                },
                "/users": {
                    "get": {
                        "operationId": "listUsers",
                        "tags": ["user"],
                        "responses": {
                            "200": {
                                "description": "List of users"
                            }
                        }
                    }
                },
                "/admin": {
                    "get": {
                        "operationId": "adminPanel",
                        "tags": ["admin", "management"],
                        "responses": {
                            "200": {
                                "description": "Admin panel"
                            }
                        }
                    }
                },
                "/public": {
                    "get": {
                        "operationId": "publicEndpoint",
                        "responses": {
                            "200": {
                                "description": "Public endpoint with no tags"
                            }
                        }
                    }
                }
            }
        });

        OpenApiSpec::from_value(spec_json).expect("Failed to create test spec")
    }

    #[test]
    fn test_tag_filtering_no_filter() {
        let spec = create_test_spec_with_tags();
        let tools = spec
            .to_tool_metadata(None)
            .expect("Failed to generate tools");

        // All operations should be included
        assert_eq!(tools.len(), 5);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets"));
        assert!(tool_names.contains(&"createPet"));
        assert!(tool_names.contains(&"listUsers"));
        assert!(tool_names.contains(&"adminPanel"));
        assert!(tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_tag_filtering_single_tag() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["pet".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Only pet operations should be included
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets"));
        assert!(tool_names.contains(&"createPet"));
        assert!(!tool_names.contains(&"listUsers"));
        assert!(!tool_names.contains(&"adminPanel"));
        assert!(!tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_tag_filtering_multiple_tags() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["pet".to_string(), "user".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Pet and user operations should be included
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets"));
        assert!(tool_names.contains(&"createPet"));
        assert!(tool_names.contains(&"listUsers"));
        assert!(!tool_names.contains(&"adminPanel"));
        assert!(!tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_tag_filtering_or_logic() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["list".to_string()]; // listPets has both "pet" and "list" tags
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Only operations with "list" tag should be included
        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets")); // Has both "pet" and "list" tags
        assert!(!tool_names.contains(&"createPet")); // Only has "pet" tag
    }

    #[test]
    fn test_tag_filtering_no_matching_tags() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["nonexistent".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // No operations should be included
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_tag_filtering_excludes_operations_without_tags() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["admin".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Only admin operations should be included, public endpoint (no tags) should be excluded
        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"adminPanel"));
        assert!(!tool_names.contains(&"publicEndpoint")); // No tags, should be excluded
    }

    #[test]
    fn test_tag_filtering_case_sensitive() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["Pet".to_string()]; // Capital P
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Should not match "pet" (lowercase)
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_tag_filtering_empty_filter_list() {
        let spec = create_test_spec_with_tags();
        let filter_tags: Vec<String> = vec![];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Empty filter should exclude all operations
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_tag_filtering_complex_scenario() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["management".to_string(), "list".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags))
            .expect("Failed to generate tools");

        // Should include adminPanel (has "management") and listPets (has "list")
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"adminPanel"));
        assert!(tool_names.contains(&"listPets"));
        assert!(!tool_names.contains(&"createPet"));
        assert!(!tool_names.contains(&"listUsers"));
        assert!(!tool_names.contains(&"publicEndpoint"));
    }
}
