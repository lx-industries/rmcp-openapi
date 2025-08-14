use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, ErrorData, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities, Tool,
        ToolsCapability,
    },
    service::RequestContext,
};
use serde_json::Value;

use reqwest::header::HeaderMap;
use url::Url;

use crate::error::{OpenApiError, ToolCallValidationError};
use crate::openapi::OpenApiSpecLocation;
use crate::tool::OpenApiTool;

#[derive(Clone)]
pub struct OpenApiServer {
    pub spec_location: OpenApiSpecLocation,
    pub tools: Vec<OpenApiTool>,
    pub base_url: Option<Url>,
    pub default_headers: Option<HeaderMap>,
    pub tag_filter: Option<Vec<String>>,
    pub method_filter: Option<Vec<reqwest::Method>>,
}

impl OpenApiServer {
    #[must_use]
    pub fn new(spec_location: OpenApiSpecLocation) -> Self {
        Self {
            spec_location,
            tools: Vec::new(),
            base_url: None,
            default_headers: None,
            tag_filter: None,
            method_filter: None,
        }
    }

    /// Create a new server with a base URL for API calls
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid
    pub fn with_base_url(
        spec_location: OpenApiSpecLocation,
        base_url: Url,
    ) -> Result<Self, OpenApiError> {
        Ok(Self {
            spec_location,
            tools: Vec::new(),
            base_url: Some(base_url),
            default_headers: None,
            tag_filter: None,
            method_filter: None,
        })
    }

    /// Create a new server with both base URL and default headers
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid
    pub fn with_base_url_and_headers(
        spec_location: OpenApiSpecLocation,
        base_url: Url,
        default_headers: HeaderMap,
    ) -> Result<Self, OpenApiError> {
        Ok(Self {
            spec_location,
            tools: Vec::new(),
            base_url: Some(base_url),
            default_headers: Some(default_headers),
            tag_filter: None,
            method_filter: None,
        })
    }

    /// Create a new server with default headers but no base URL
    #[must_use]
    pub fn with_default_headers(
        spec_location: OpenApiSpecLocation,
        default_headers: HeaderMap,
    ) -> Self {
        Self {
            spec_location,
            tools: Vec::new(),
            base_url: None,
            default_headers: Some(default_headers),
            tag_filter: None,
            method_filter: None,
        }
    }

    /// Load the `OpenAPI` specification and convert to OpenApiTool instances
    ///
    /// # Errors
    ///
    /// Returns an error if the spec cannot be loaded or tools cannot be generated
    pub async fn load_openapi_spec(&mut self) -> Result<(), OpenApiError> {
        // Load the OpenAPI specification
        let spec = self.spec_location.load_spec().await?;

        // Generate OpenApiTool instances directly
        let tools = spec.to_openapi_tools(
            self.tag_filter.as_deref(),
            self.method_filter.as_deref(),
            self.base_url.clone(),
            self.default_headers.clone(),
        )?;

        self.tools = tools;

        println!("Loaded {} tools from OpenAPI spec", self.tools.len());

        Ok(())
    }

    /// Get the number of loaded tools
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Get all tool names
    #[must_use]
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools
            .iter()
            .map(|tool| tool.metadata.name.clone())
            .collect()
    }

    /// Check if a specific tool exists
    #[must_use]
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|tool| tool.metadata.name == name)
    }

    /// Get a tool by name
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<&crate::tool::OpenApiTool> {
        self.tools.iter().find(|tool| tool.metadata.name == name)
    }

    /// Get tool metadata by name
    #[must_use]
    pub fn get_tool_metadata(&self, name: &str) -> Option<&crate::ToolMetadata> {
        self.get_tool(name).map(|tool| &tool.metadata)
    }

    /// Get basic tool statistics
    #[must_use]
    pub fn get_tool_stats(&self) -> String {
        format!("Total tools: {}", self.tools.len())
    }

    /// Set tag filter for this server instance
    #[must_use]
    pub fn with_tags(mut self, tags: Option<Vec<String>>) -> Self {
        self.tag_filter = tags;
        self
    }

    /// Set method filter for this server instance
    #[must_use]
    pub fn with_methods(mut self, methods: Option<Vec<reqwest::Method>>) -> Self {
        self.method_filter = methods;
        self
    }

    /// Simple validation - check that tools are loaded
    ///
    /// # Errors
    ///
    /// Returns an error if no tools are loaded
    pub fn validate_registry(&self) -> Result<(), OpenApiError> {
        if self.tools.is_empty() {
            return Err(OpenApiError::McpError("No tools loaded".to_string()));
        }
        Ok(())
    }
}

impl ServerHandler for OpenApiServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation {
                name: "OpenAPI MCP Server".to_string(),
                version: "0.1.0".to_string(),
            },
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            instructions: Some("Exposes OpenAPI endpoints as MCP tools".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut tools = Vec::new();

        // Convert all OpenApiTool instances to MCP Tool format
        for openapi_tool in &self.tools {
            let tool = Tool::from(openapi_tool);
            tools.push(tool);
        }

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Find the tool by name
        if let Some(openapi_tool) = self
            .tools
            .iter()
            .find(|tool| tool.metadata.name == request.name)
        {
            let arguments = request.arguments.unwrap_or_default();
            let arguments_value = Value::Object(arguments.clone());

            // Call the tool directly
            match openapi_tool.call(&arguments_value).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    // Convert ToolCallError to ErrorData and return as error
                    Err(e.into())
                }
            }
        } else {
            // Generate tool name suggestions when tool not found
            let tool_names: Vec<&str> = self
                .tools
                .iter()
                .map(|tool| tool.metadata.name.as_str())
                .collect();
            let suggestions = crate::find_similar_strings(&request.name, &tool_names);

            // Create ToolCallValidationError with suggestions
            let error = ToolCallValidationError::ToolNotFound {
                tool_name: request.name.to_string(),
                suggestions,
            };
            Err(error.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolMetadata;
    use crate::error::ToolCallError;
    use serde_json::json;

    #[test]
    fn test_tool_not_found_error_with_suggestions() {
        // Create test tool metadata
        let tool1_metadata = ToolMetadata {
            name: "getPetById".to_string(),
            title: Some("Get Pet by ID".to_string()),
            description: "Find pet by ID".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "petId": {
                        "type": "integer"
                    }
                },
                "required": ["petId"]
            }),
            output_schema: None,
            method: "GET".to_string(),
            path: "/pet/{petId}".to_string(),
        };

        let tool2_metadata = ToolMetadata {
            name: "getPetsByStatus".to_string(),
            title: Some("Find Pets by Status".to_string()),
            description: "Find pets by status".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["status"]
            }),
            output_schema: None,
            method: "GET".to_string(),
            path: "/pet/findByStatus".to_string(),
        };

        // Create OpenApiTool instances
        let tool1 = crate::tool::OpenApiTool::new(tool1_metadata, None, None).unwrap();
        let tool2 = crate::tool::OpenApiTool::new(tool2_metadata, None, None).unwrap();

        // Create server with tools
        let mut server = OpenApiServer::new(OpenApiSpecLocation::Url(
            Url::parse("test://example").unwrap(),
        ));
        server.tools = vec![tool1, tool2];

        // Test: Create ToolNotFound error with a typo
        let tool_names = server.get_tool_names();
        let tool_name_refs: Vec<&str> = tool_names.iter().map(|s| s.as_str()).collect();
        let suggestions = crate::find_similar_strings("getPetByID", &tool_name_refs);

        let error = ToolCallError::Validation(ToolCallValidationError::ToolNotFound {
            tool_name: "getPetByID".to_string(),
            suggestions,
        });
        let error_data: ErrorData = error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        // Snapshot the error to verify suggestions
        insta::assert_json_snapshot!(error_json);
    }

    #[test]
    fn test_tool_not_found_error_no_suggestions() {
        // Create test tool metadata
        let tool_metadata = ToolMetadata {
            name: "getPetById".to_string(),
            title: Some("Get Pet by ID".to_string()),
            description: "Find pet by ID".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "petId": {
                        "type": "integer"
                    }
                },
                "required": ["petId"]
            }),
            output_schema: None,
            method: "GET".to_string(),
            path: "/pet/{petId}".to_string(),
        };

        // Create OpenApiTool instance
        let tool = crate::tool::OpenApiTool::new(tool_metadata, None, None).unwrap();

        // Create server with tool
        let mut server = OpenApiServer::new(OpenApiSpecLocation::Url(
            Url::parse("test://example").unwrap(),
        ));
        server.tools = vec![tool];

        // Test: Create ToolNotFound error with unrelated name
        let tool_names = server.get_tool_names();
        let tool_name_refs: Vec<&str> = tool_names.iter().map(|s| s.as_str()).collect();
        let suggestions =
            crate::find_similar_strings("completelyUnrelatedToolName", &tool_name_refs);

        let error = ToolCallError::Validation(ToolCallValidationError::ToolNotFound {
            tool_name: "completelyUnrelatedToolName".to_string(),
            suggestions,
        });
        let error_data: ErrorData = error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        // Snapshot the error to verify no suggestions
        insta::assert_json_snapshot!(error_json);
    }

    #[test]
    fn test_validation_error_converted_to_error_data() {
        // Test that validation errors are properly converted to ErrorData
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![crate::error::ValidationError::InvalidParameter {
                parameter: "page".to_string(),
                suggestions: vec!["page_number".to_string()],
                valid_parameters: vec!["page_number".to_string(), "page_size".to_string()],
            }],
        });

        let error_data: ErrorData = error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        // Verify the basic structure
        assert_eq!(error_json["code"], -32602); // Invalid params error code

        // Snapshot the full error to verify the new error message format
        insta::assert_json_snapshot!(error_json);
    }
}
