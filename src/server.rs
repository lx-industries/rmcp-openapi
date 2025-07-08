use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities, Tool,
        ToolsCapability,
    },
    service::RequestContext,
};
use serde_json::Value;
use std::sync::Arc;

use crate::error::OpenApiError;
use crate::http_client::HttpClient;
use crate::openapi_spec::OpenApiSpec;
use crate::tool_registry::ToolRegistry;

pub struct OpenApiServer {
    pub spec_url: String,
    pub registry: ToolRegistry,
    pub http_client: HttpClient,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub method: String,
    pub path: String,
}

impl OpenApiServer {
    pub fn new(spec_url: String) -> Self {
        Self {
            spec_url,
            registry: ToolRegistry::new(),
            http_client: HttpClient::new(),
            base_url: None,
        }
    }

    /// Create a new server with a base URL for API calls
    pub fn with_base_url(spec_url: String, base_url: String) -> Self {
        let http_client = HttpClient::new().with_base_url(base_url.clone());
        Self {
            spec_url,
            registry: ToolRegistry::new(),
            http_client,
            base_url: Some(base_url),
        }
    }

    pub async fn load_openapi_spec(&mut self) -> Result<(), OpenApiError> {
        // Load the OpenAPI specification
        let spec = if self.spec_url.starts_with("http") {
            OpenApiSpec::from_url(&self.spec_url).await?
        } else {
            OpenApiSpec::from_file(&self.spec_url).await?
        };

        // Register tools from the spec
        let registered_count = self.registry.register_from_spec(spec)?;

        println!("Loaded {registered_count} tools from OpenAPI spec");
        println!("Registry stats: {}", self.registry.get_stats().summary());

        Ok(())
    }

    /// Get the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.registry.tool_count()
    }

    /// Get all tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.registry.get_tool_names()
    }

    /// Check if a specific tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_tool(name)
    }

    /// Get registry statistics
    pub fn get_registry_stats(&self) -> crate::tool_registry::ToolRegistryStats {
        self.registry.get_stats()
    }

    /// Validate the registry integrity
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

            let tool = Tool {
                name: tool_metadata.name.clone().into(),
                description: Some(tool_metadata.description.clone().into()),
                input_schema,
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
                    // Return successful response
                    Ok(CallToolResult {
                        content: vec![Content::text(response.to_mcp_content())],
                        is_error: Some(!response.is_success),
                    })
                }
                Err(e) => {
                    // Return error response with details
                    Ok(CallToolResult {
                        content: vec![Content::text(format!(
                            "❌ Error executing tool '{}'\n\nError: {}\n\nTool details:\n- Method: {}\n- Path: {}\n- Arguments: {}",
                            request.name,
                            e,
                            tool_metadata.method.to_uppercase(),
                            tool_metadata.path,
                            serde_json::to_string_pretty(&arguments_value)
                                .unwrap_or_else(|_| "Invalid JSON".to_string())
                        ))],
                        is_error: Some(true),
                    })
                }
            }
        } else {
            Err(OpenApiError::ToolNotFound(request.name.to_string()).into())
        }
    }
}
