use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, ErrorData, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities,
        ToolsCapability,
    },
    service::RequestContext,
};
use serde_json::Value;

use reqwest::header::HeaderMap;
use url::Url;

use crate::error::Error;
use crate::tool::{Tool, ToolCollection, ToolMetadata};
use tracing::{debug, info, info_span, warn};

#[derive(Clone)]
pub struct Server {
    pub openapi_spec: serde_json::Value,
    pub tool_collection: ToolCollection,
    pub base_url: Url,
    pub default_headers: Option<HeaderMap>,
    pub tag_filter: Option<Vec<String>>,
    pub method_filter: Option<Vec<reqwest::Method>>,
}

impl Server {
    /// Create a new Server instance with required parameters
    pub fn new(
        openapi_spec: serde_json::Value,
        base_url: Url,
        default_headers: Option<HeaderMap>,
        tag_filter: Option<Vec<String>>,
        method_filter: Option<Vec<reqwest::Method>>,
    ) -> Self {
        Self {
            openapi_spec,
            tool_collection: ToolCollection::new(),
            base_url,
            default_headers,
            tag_filter,
            method_filter,
        }
    }

    /// Parse the `OpenAPI` specification and convert to OpenApiTool instances
    ///
    /// # Errors
    ///
    /// Returns an error if the spec cannot be parsed or tools cannot be generated
    pub fn load_openapi_spec(&mut self) -> Result<(), Error> {
        let span = info_span!("tool_registration");
        let _enter = span.enter();

        // Parse the OpenAPI specification
        let spec = crate::spec::Spec::from_value(self.openapi_spec.clone())?;

        // Generate OpenApiTool instances directly
        let tools = spec.to_openapi_tools(
            self.tag_filter.as_deref(),
            self.method_filter.as_deref(),
            Some(self.base_url.clone()),
            self.default_headers.clone(),
        )?;

        self.tool_collection = ToolCollection::from_tools(tools);

        info!(
            tool_count = self.tool_collection.len(),
            "Loaded tools from OpenAPI spec"
        );

        Ok(())
    }

    /// Get the number of loaded tools
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.tool_collection.len()
    }

    /// Get all tool names
    #[must_use]
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tool_collection.get_tool_names()
    }

    /// Check if a specific tool exists
    #[must_use]
    pub fn has_tool(&self, name: &str) -> bool {
        self.tool_collection.has_tool(name)
    }

    /// Get a tool by name
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tool_collection.get_tool(name)
    }

    /// Get tool metadata by name
    #[must_use]
    pub fn get_tool_metadata(&self, name: &str) -> Option<&ToolMetadata> {
        self.get_tool(name).map(|tool| &tool.metadata)
    }

    /// Get basic tool statistics
    #[must_use]
    pub fn get_tool_stats(&self) -> String {
        self.tool_collection.get_stats()
    }

    /// Simple validation - check that tools are loaded
    ///
    /// # Errors
    ///
    /// Returns an error if no tools are loaded
    pub fn validate_registry(&self) -> Result<(), Error> {
        if self.tool_collection.is_empty() {
            return Err(Error::McpError("No tools loaded".to_string()));
        }
        Ok(())
    }
}

impl ServerHandler for Server {
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
        let span = info_span!("list_tools", tool_count = self.tool_collection.len());
        let _enter = span.enter();

        debug!("Processing MCP list_tools request");

        // Delegate to tool collection for MCP tool conversion
        let tools = self.tool_collection.to_mcp_tools();

        info!(
            returned_tools = tools.len(),
            "MCP list_tools request completed successfully"
        );

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
        let span = info_span!(
            "call_tool",
            tool_name = %request.name
        );
        let _enter = span.enter();

        debug!(
            tool_name = %request.name,
            has_arguments = !request.arguments.as_ref().unwrap_or(&serde_json::Map::new()).is_empty(),
            "Processing MCP call_tool request"
        );

        let arguments = request.arguments.unwrap_or_default();
        let arguments_value = Value::Object(arguments);

        // Delegate all tool validation and execution to the tool collection
        match self
            .tool_collection
            .call_tool(&request.name, &arguments_value)
            .await
        {
            Ok(result) => {
                info!(
                    tool_name = %request.name,
                    success = true,
                    "MCP call_tool request completed successfully"
                );
                Ok(result)
            }
            Err(e) => {
                warn!(
                    tool_name = %request.name,
                    success = false,
                    error = %e,
                    "MCP call_tool request failed"
                );
                // Convert ToolCallError to ErrorData and return as error
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ToolCallValidationError;
    use crate::{ToolCallError, ToolMetadata};
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
        let tool1 = Tool::new(tool1_metadata, None, None).unwrap();
        let tool2 = Tool::new(tool2_metadata, None, None).unwrap();

        // Create server with tools
        let mut server = Server::new(
            serde_json::Value::Null,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            None,
        );
        server.tool_collection = ToolCollection::from_tools(vec![tool1, tool2]);

        // Test: Create ToolNotFound error with a typo
        let tool_names = server.get_tool_names();
        let tool_name_refs: Vec<&str> = tool_names.iter().map(|s| s.as_str()).collect();

        let error = ToolCallError::Validation(ToolCallValidationError::tool_not_found(
            "getPetByID".to_string(),
            &tool_name_refs,
        ));
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
        let tool = Tool::new(tool_metadata, None, None).unwrap();

        // Create server with tool
        let mut server = Server::new(
            serde_json::Value::Null,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            None,
        );
        server.tool_collection = ToolCollection::from_tools(vec![tool]);

        // Test: Create ToolNotFound error with unrelated name
        let tool_names = server.get_tool_names();
        let tool_name_refs: Vec<&str> = tool_names.iter().map(|s| s.as_str()).collect();

        let error = ToolCallError::Validation(ToolCallValidationError::tool_not_found(
            "completelyUnrelatedToolName".to_string(),
            &tool_name_refs,
        ));
        let error_data: ErrorData = error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        // Snapshot the error to verify no suggestions
        insta::assert_json_snapshot!(error_json);
    }

    #[test]
    fn test_validation_error_converted_to_error_data() {
        // Test that validation errors are properly converted to ErrorData
        let error = ToolCallError::Validation(ToolCallValidationError::InvalidParameters {
            violations: vec![crate::error::ValidationError::invalid_parameter(
                "page".to_string(),
                &["page_number".to_string(), "page_size".to_string()],
            )],
        });

        let error_data: ErrorData = error.into();
        let error_json = serde_json::to_value(&error_data).unwrap();

        // Verify the basic structure
        assert_eq!(error_json["code"], -32602); // Invalid params error code

        // Snapshot the full error to verify the new error message format
        insta::assert_json_snapshot!(error_json);
    }
}
