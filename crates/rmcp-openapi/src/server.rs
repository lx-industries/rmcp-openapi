use bon::Builder;
use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, ErrorData, Implementation, InitializeResult,
        ListToolsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities,
        ToolsCapability,
    },
    service::{RequestContext, RoleServer},
};
use rmcp_actix_web::transport::AuthorizationHeader;
use serde_json::Value;

use reqwest::header::HeaderMap;
use url::Url;

use crate::error::Error;
use crate::tool::{Tool, ToolCollection, ToolMetadata};
use crate::{
    config::{Authorization, AuthorizationMode},
    spec::Filters,
};
use tracing::{debug, info, info_span, warn};

#[derive(Clone, Builder)]
pub struct Server {
    pub openapi_spec: serde_json::Value,
    #[builder(default)]
    pub tool_collection: ToolCollection,
    pub base_url: Url,
    pub default_headers: Option<HeaderMap>,
    pub filters: Option<Filters>,
    #[builder(default)]
    pub authorization_mode: AuthorizationMode,
    pub name: Option<String>,
    pub version: Option<String>,
    pub title: Option<String>,
    pub instructions: Option<String>,
    #[builder(default)]
    pub skip_tool_descriptions: bool,
    #[builder(default)]
    pub skip_parameter_descriptions: bool,
}

impl Server {
    /// Create a new Server instance with required parameters
    pub fn new(
        openapi_spec: serde_json::Value,
        base_url: Url,
        default_headers: Option<HeaderMap>,
        filters: Option<Filters>,
        skip_tool_descriptions: bool,
        skip_parameter_descriptions: bool,
    ) -> Self {
        Self {
            openapi_spec,
            tool_collection: ToolCollection::new(),
            base_url,
            default_headers,
            filters,
            authorization_mode: AuthorizationMode::default(),
            name: None,
            version: None,
            title: None,
            instructions: None,
            skip_tool_descriptions,
            skip_parameter_descriptions,
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
            self.filters.as_ref(),
            Some(self.base_url.clone()),
            self.default_headers.clone(),
            self.skip_tool_descriptions,
            self.skip_parameter_descriptions,
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

    /// Set the authorization mode for the server
    pub fn set_authorization_mode(&mut self, mode: AuthorizationMode) {
        self.authorization_mode = mode;
    }

    /// Get the current authorization mode
    pub fn authorization_mode(&self) -> AuthorizationMode {
        self.authorization_mode
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

    /// Extract title from OpenAPI spec info section
    fn extract_openapi_title(&self) -> Option<String> {
        self.openapi_spec
            .get("info")?
            .get("title")?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Extract version from OpenAPI spec info section
    fn extract_openapi_version(&self) -> Option<String> {
        self.openapi_spec
            .get("info")?
            .get("version")?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Extract description from OpenAPI spec info section
    fn extract_openapi_description(&self) -> Option<String> {
        self.openapi_spec
            .get("info")?
            .get("description")?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Extract display title from OpenAPI spec info section
    /// First checks for x-display-title extension, then derives from title
    fn extract_openapi_display_title(&self) -> Option<String> {
        // First check for x-display-title extension
        if let Some(display_title) = self
            .openapi_spec
            .get("info")
            .and_then(|info| info.get("x-display-title"))
            .and_then(|t| t.as_str())
        {
            return Some(display_title.to_string());
        }

        // Fallback: enhance the title with "Server" suffix if not already present
        self.extract_openapi_title().map(|title| {
            if title.to_lowercase().contains("server") {
                title
            } else {
                format!("{} Server", title)
            }
        })
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> InitializeResult {
        // 3-level fallback for server name: custom -> OpenAPI spec -> default
        let server_name = self
            .name
            .clone()
            .or_else(|| self.extract_openapi_title())
            .unwrap_or_else(|| "OpenAPI MCP Server".to_string());

        // 3-level fallback for server version: custom -> OpenAPI spec -> crate version
        let server_version = self
            .version
            .clone()
            .or_else(|| self.extract_openapi_version())
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

        // 3-level fallback for title: custom -> OpenAPI-derived -> None
        let server_title = self
            .title
            .clone()
            .or_else(|| self.extract_openapi_display_title());

        // 3-level fallback for instructions: custom -> OpenAPI spec -> default
        let instructions = self
            .instructions
            .clone()
            .or_else(|| self.extract_openapi_description())
            .or_else(|| Some("Exposes OpenAPI endpoints as MCP tools".to_string()));

        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation {
                name: server_name,
                version: server_version,
                title: server_title,
                icons: None,
                website_url: None,
            },
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            instructions,
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
            meta: None,
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
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

        // Extract authorization header from context extensions
        let auth_header = context.extensions.get::<AuthorizationHeader>().cloned();

        if auth_header.is_some() {
            debug!("Authorization header is present");
        }

        // Create Authorization enum from mode and header
        let authorization = Authorization::from_mode(self.authorization_mode, auth_header);

        // Delegate all tool validation and execution to the tool collection
        match self
            .tool_collection
            .call_tool(&request.name, &arguments_value, authorization)
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
    use crate::{HttpClient, ToolCallError, ToolMetadata};
    use serde_json::json;

    #[test]
    fn test_tool_not_found_error_with_suggestions() {
        // Create test tool metadata
        let tool1_metadata = ToolMetadata {
            name: "getPetById".to_string(),
            title: Some("Get Pet by ID".to_string()),
            description: Some("Find pet by ID".to_string()),
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
            security: None,
            parameter_mappings: std::collections::HashMap::new(),
        };

        let tool2_metadata = ToolMetadata {
            name: "getPetsByStatus".to_string(),
            title: Some("Find Pets by Status".to_string()),
            description: Some("Find pets by status".to_string()),
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
            security: None,
            parameter_mappings: std::collections::HashMap::new(),
        };

        // Create OpenApiTool instances
        let http_client = HttpClient::new();
        let tool1 = Tool::new(tool1_metadata, http_client.clone()).unwrap();
        let tool2 = Tool::new(tool2_metadata, http_client.clone()).unwrap();

        // Create server with tools
        let mut server = Server::new(
            serde_json::Value::Null,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
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
            description: Some("Find pet by ID".to_string()),
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
            security: None,
            parameter_mappings: std::collections::HashMap::new(),
        };

        // Create OpenApiTool instance
        let tool = Tool::new(tool_metadata, HttpClient::new()).unwrap();

        // Create server with tool
        let mut server = Server::new(
            serde_json::Value::Null,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
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

    #[test]
    fn test_extract_openapi_info_with_full_spec() {
        let openapi_spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Pet Store API",
                "version": "2.1.0",
                "description": "A sample API for managing pets"
            },
            "paths": {}
        });

        let server = Server::new(
            openapi_spec,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        assert_eq!(
            server.extract_openapi_title(),
            Some("Pet Store API".to_string())
        );
        assert_eq!(server.extract_openapi_version(), Some("2.1.0".to_string()));
        assert_eq!(
            server.extract_openapi_description(),
            Some("A sample API for managing pets".to_string())
        );
    }

    #[test]
    fn test_extract_openapi_info_with_minimal_spec() {
        let openapi_spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "My API",
                "version": "1.0.0"
            },
            "paths": {}
        });

        let server = Server::new(
            openapi_spec,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        assert_eq!(server.extract_openapi_title(), Some("My API".to_string()));
        assert_eq!(server.extract_openapi_version(), Some("1.0.0".to_string()));
        assert_eq!(server.extract_openapi_description(), None);
    }

    #[test]
    fn test_extract_openapi_info_with_invalid_spec() {
        let openapi_spec = json!({
            "invalid": "spec"
        });

        let server = Server::new(
            openapi_spec,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        assert_eq!(server.extract_openapi_title(), None);
        assert_eq!(server.extract_openapi_version(), None);
        assert_eq!(server.extract_openapi_description(), None);
    }

    #[test]
    fn test_get_info_fallback_hierarchy_custom_metadata() {
        let server = Server::new(
            serde_json::Value::Null,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        // Set custom metadata directly
        let mut server = server;
        server.name = Some("Custom Server".to_string());
        server.version = Some("3.0.0".to_string());
        server.instructions = Some("Custom instructions".to_string());

        let result = server.get_info();

        assert_eq!(result.server_info.name, "Custom Server");
        assert_eq!(result.server_info.version, "3.0.0");
        assert_eq!(result.instructions, Some("Custom instructions".to_string()));
    }

    #[test]
    fn test_get_info_fallback_hierarchy_openapi_spec() {
        let openapi_spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "OpenAPI Server",
                "version": "1.5.0",
                "description": "Server from OpenAPI spec"
            },
            "paths": {}
        });

        let server = Server::new(
            openapi_spec,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        let result = server.get_info();

        assert_eq!(result.server_info.name, "OpenAPI Server");
        assert_eq!(result.server_info.version, "1.5.0");
        assert_eq!(
            result.instructions,
            Some("Server from OpenAPI spec".to_string())
        );
    }

    #[test]
    fn test_get_info_fallback_hierarchy_defaults() {
        let server = Server::new(
            serde_json::Value::Null,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        let result = server.get_info();

        assert_eq!(result.server_info.name, "OpenAPI MCP Server");
        assert_eq!(result.server_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            result.instructions,
            Some("Exposes OpenAPI endpoints as MCP tools".to_string())
        );
    }

    #[test]
    fn test_get_info_fallback_hierarchy_mixed() {
        let openapi_spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "OpenAPI Server",
                "version": "2.5.0",
                "description": "Server from OpenAPI spec"
            },
            "paths": {}
        });

        let mut server = Server::new(
            openapi_spec,
            url::Url::parse("http://example.com").unwrap(),
            None,
            None,
            false,
            false,
        );

        // Set custom name and instructions, leave version to fallback to OpenAPI
        server.name = Some("Custom Server".to_string());
        server.instructions = Some("Custom instructions".to_string());

        let result = server.get_info();

        // Custom name takes precedence
        assert_eq!(result.server_info.name, "Custom Server");
        // OpenAPI version is used
        assert_eq!(result.server_info.version, "2.5.0");
        // Custom instructions take precedence
        assert_eq!(result.instructions, Some("Custom instructions".to_string()));
    }
}
