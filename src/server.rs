use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities, Tool,
        ToolAnnotations, ToolsCapability,
    },
    service::RequestContext,
};
use serde_json::{Value, json};

use reqwest::header::HeaderMap;
use std::sync::Arc;
use url::Url;

use crate::error::{OpenApiError, ToolCallError, ToolCallExecutionError, ToolCallValidationError};
use crate::http_client::HttpClient;
use crate::openapi::OpenApiSpecLocation;
use crate::tool_registry::ToolRegistry;

#[derive(Clone)]
pub struct OpenApiServer {
    pub spec_location: OpenApiSpecLocation,
    pub registry: Arc<ToolRegistry>,
    pub http_client: HttpClient,
    pub base_url: Option<Url>,
}

/// Internal metadata for tools generated from OpenAPI operations.
///
/// This struct contains all the information needed to execute HTTP requests
/// and is used internally by the OpenAPI server. It includes fields that are
/// not part of the MCP specification but are necessary for HTTP execution.
///
/// For MCP compliance, this struct is converted to `rmcp::model::Tool` using
/// the `From` trait implementation, which only includes MCP-compliant fields.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolMetadata {
    /// Tool name - exposed to MCP clients
    pub name: String,
    /// Tool title - human-readable display name exposed to MCP clients
    pub title: Option<String>,
    /// Tool description - exposed to MCP clients  
    pub description: String,
    /// Input parameters schema - exposed to MCP clients as `inputSchema`
    pub parameters: Value,
    /// Output schema - exposed to MCP clients as `outputSchema`
    pub output_schema: Option<Value>,
    /// HTTP method (GET, POST, etc.) - internal only, not exposed to MCP
    pub method: String,
    /// URL path for the API endpoint - internal only, not exposed to MCP
    pub path: String,
}

/// Converts internal `ToolMetadata` to MCP-compliant `Tool`.
///
/// This implementation ensures that only MCP-compliant fields are exposed to clients.
/// Internal fields like `method` and `path` are not included in the conversion.
impl From<&ToolMetadata> for Tool {
    fn from(metadata: &ToolMetadata) -> Self {
        // Convert parameters to the expected Arc<Map> format
        let input_schema = if let Value::Object(obj) = &metadata.parameters {
            Arc::new(obj.clone())
        } else {
            Arc::new(serde_json::Map::new())
        };

        // Convert output_schema to the expected Arc<Map> format if present
        let output_schema = metadata.output_schema.as_ref().and_then(|schema| {
            if let Value::Object(obj) = schema {
                Some(Arc::new(obj.clone()))
            } else {
                None
            }
        });

        // Create annotations with title if present
        let annotations = metadata.title.as_ref().map(|title| ToolAnnotations {
            title: Some(title.clone()),
            ..Default::default()
        });

        Tool {
            name: metadata.name.clone().into(),
            description: Some(metadata.description.clone().into()),
            input_schema,
            output_schema,
            annotations,
            // TODO: Consider migration to Tool.title when rmcp supports MCP 2025-06-18 (see issue #26)
        }
    }
}

impl OpenApiServer {
    #[must_use]
    pub fn new(spec_location: OpenApiSpecLocation) -> Self {
        Self {
            spec_location,
            registry: Arc::new(ToolRegistry::new()),
            http_client: HttpClient::new(),
            base_url: None,
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
        let http_client = HttpClient::new().with_base_url(base_url.clone())?;
        Ok(Self {
            spec_location,
            registry: Arc::new(ToolRegistry::new()),
            http_client,
            base_url: Some(base_url),
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
        let http_client = HttpClient::new()
            .with_base_url(base_url.clone())?
            .with_default_headers(default_headers);
        Ok(Self {
            spec_location,
            registry: Arc::new(ToolRegistry::new()),
            http_client,
            base_url: Some(base_url),
        })
    }

    /// Create a new server with default headers but no base URL
    #[must_use]
    pub fn with_default_headers(
        spec_location: OpenApiSpecLocation,
        default_headers: HeaderMap,
    ) -> Self {
        let http_client = HttpClient::new().with_default_headers(default_headers);
        Self {
            spec_location,
            registry: Arc::new(ToolRegistry::new()),
            http_client,
            base_url: None,
        }
    }

    /// Load the `OpenAPI` specification from the configured location
    ///
    /// # Errors
    ///
    /// Returns an error if the spec cannot be loaded or registered
    pub async fn load_openapi_spec(&mut self) -> Result<(), OpenApiError> {
        // Load the OpenAPI specification using the new simplified approach
        let spec = self.spec_location.load_spec().await?;
        self.register_spec(spec)
    }

    /// Register a spec into the registry. This requires exclusive access to the server.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry is already shared or if spec registration fails
    pub fn register_spec(&mut self, spec: crate::openapi::OpenApiSpec) -> Result<(), OpenApiError> {
        // During initialization, we should have exclusive access to the Arc
        let registry = Arc::get_mut(&mut self.registry)
            .ok_or_else(|| OpenApiError::McpError("Registry is already shared".to_string()))?;
        let registered_count = registry.register_from_spec(spec)?;

        println!("Loaded {registered_count} tools from OpenAPI spec");
        println!("Registry stats: {}", self.registry.get_stats().summary());

        Ok(())
    }

    /// Get the number of registered tools
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.registry.tool_count()
    }

    /// Get all tool names
    #[must_use]
    pub fn get_tool_names(&self) -> Vec<String> {
        self.registry.get_tool_names()
    }

    /// Check if a specific tool exists
    #[must_use]
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_tool(name)
    }

    /// Get registry statistics
    #[must_use]
    pub fn get_registry_stats(&self) -> crate::tool_registry::ToolRegistryStats {
        self.registry.get_stats()
    }

    /// Validate the registry integrity
    ///
    /// # Errors
    ///
    /// Returns an error if the registry validation fails
    pub fn validate_registry(&self) -> Result<(), OpenApiError> {
        self.registry.validate_registry()
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

        // Convert all registered tools to MCP Tool format
        for tool_metadata in self.registry.get_all_tools() {
            let tool = Tool::from(tool_metadata);
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
        // Check if tool exists in registry
        if let Some(tool_metadata) = self.registry.get_tool(&request.name) {
            let arguments = request.arguments.unwrap_or_default();
            let arguments_value = Value::Object(arguments.clone());

            // Execute the HTTP request
            match self
                .http_client
                .execute_tool_call(tool_metadata, &arguments_value)
                .await
            {
                Ok(response) => {
                    // Check if the tool has an output schema
                    let structured_content = if tool_metadata.output_schema.is_some() {
                        // Try to parse the response body as JSON
                        match response.json() {
                            Ok(json_value) => {
                                // Wrap the response in our standard HTTP response structure
                                Some(json!({
                                    "status": response.status_code,
                                    "body": json_value
                                }))
                            }
                            Err(_) => None, // If parsing fails, fall back to text content
                        }
                    } else {
                        None
                    };

                    // For structured content, serialize to JSON for backwards compatibility
                    let content = if let Some(ref structured) = structured_content {
                        // MCP Specification: https://modelcontextprotocol.io/specification/2025-06-18/server/tools#structured-content
                        // "For backwards compatibility, a tool that returns structured content SHOULD also
                        // return the serialized JSON in a TextContent block."
                        match serde_json::to_string(structured) {
                            Ok(json_string) => Some(vec![Content::text(json_string)]),
                            Err(e) => {
                                // Return error if we can't serialize the structured content
                                let error = ToolCallError::Execution(
                                    ToolCallExecutionError::ResponseParsingError {
                                        reason: format!(
                                            "Failed to serialize structured content: {e}"
                                        ),
                                        raw_response: None,
                                    },
                                );
                                return Err(error.into());
                            }
                        }
                    } else {
                        Some(vec![Content::text(response.to_mcp_content())])
                    };

                    // Return successful response
                    Ok(CallToolResult {
                        content,
                        structured_content,
                        is_error: Some(!response.is_success),
                    })
                }
                Err(e) => {
                    // Convert ToolCallError to ErrorData and return as error
                    Err(e.into())
                }
            }
        } else {
            // Generate tool name suggestions when tool not found
            let tool_names = self.registry.get_tool_names();
            let tool_name_refs: Vec<&str> = tool_names.iter().map(|s| s.as_str()).collect();
            let suggestions = crate::find_similar_strings(&request.name, &tool_name_refs);

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
    use crate::error::ToolCallError;

    #[test]
    fn test_tool_not_found_error_with_suggestions() {
        // Create a server with test tools
        let mut server = OpenApiServer::new(OpenApiSpecLocation::Url(
            Url::parse("test://example").unwrap(),
        ));

        // Create test tool metadata
        let tool1 = ToolMetadata {
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

        let tool2 = ToolMetadata {
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

        // Get mutable access to registry and register tools
        let registry = Arc::get_mut(&mut server.registry).unwrap();

        // Create a mock operation for testing
        let mock_operation = oas3::spec::Operation::default();

        // Register tools with mock operations
        registry
            .register_tool(
                tool1,
                (
                    mock_operation.clone(),
                    "GET".to_string(),
                    "/pet/{petId}".to_string(),
                ),
            )
            .unwrap();
        registry
            .register_tool(
                tool2,
                (
                    mock_operation,
                    "GET".to_string(),
                    "/pet/findByStatus".to_string(),
                ),
            )
            .unwrap();

        // Test: Create ToolNotFound error with a typo
        let tool_names = server.registry.get_tool_names();
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
        // Create a server with test tools
        let mut server = OpenApiServer::new(OpenApiSpecLocation::Url(
            Url::parse("test://example").unwrap(),
        ));

        // Create test tool metadata
        let tool = ToolMetadata {
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

        // Get mutable access to registry and register tool
        let registry = Arc::get_mut(&mut server.registry).unwrap();

        // Create a mock operation for testing
        let mock_operation = oas3::spec::Operation::default();

        // Register tool with mock operation
        registry
            .register_tool(
                tool,
                (
                    mock_operation,
                    "GET".to_string(),
                    "/pet/{petId}".to_string(),
                ),
            )
            .unwrap();

        // Test: Create ToolNotFound error with unrelated name
        let tool_names = server.registry.get_tool_names();
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
