use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities, Tool,
        ToolsCapability,
    },
    service::RequestContext,
};
use serde_json::{Value, json};
use std::sync::Arc;
use url::Url;

use crate::error::OpenApiError;
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub output_schema: Option<Value>,
    pub method: String,
    pub path: String,
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
            // Convert parameters to the expected Arc<Map> format
            let input_schema = if let Value::Object(obj) = &tool_metadata.parameters {
                Arc::new(obj.clone())
            } else {
                Arc::new(serde_json::Map::new())
            };

            // Convert output_schema to the expected Arc<Map> format if present
            let output_schema = tool_metadata.output_schema.as_ref().and_then(|schema| {
                if let Value::Object(obj) = schema {
                    Some(Arc::new(obj.clone()))
                } else {
                    None
                }
            });

            let tool = Tool {
                name: tool_metadata.name.clone().into(),
                description: Some(tool_metadata.description.clone().into()),
                input_schema,
                output_schema,
                annotations: None,
            };
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

                    // Return successful response
                    Ok(CallToolResult {
                        content: Some(vec![Content::text(response.to_mcp_content())]),
                        structured_content,
                        is_error: Some(!response.is_success),
                    })
                }
                Err(e) => {
                    // Return error response with details
                    Ok(CallToolResult {
                        content: Some(vec![Content::text(format!(
                            "❌ Error executing tool '{}'\n\nError: {}\n\nTool details:\n- Method: {}\n- Path: {}\n- Arguments: {}",
                            request.name,
                            e,
                            tool_metadata.method.to_uppercase(),
                            tool_metadata.path,
                            serde_json::to_string_pretty(&arguments_value)
                                .unwrap_or_else(|_| "Invalid JSON".to_string())
                        ))]),
                        structured_content: None,
                        is_error: Some(true),
                    })
                }
            }
        } else {
            Err(OpenApiError::ToolNotFound(request.name.to_string()).into())
        }
    }
}
