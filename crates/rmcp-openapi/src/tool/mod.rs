pub mod metadata;
pub mod tool_collection;

pub use metadata::ToolMetadata;
pub use tool_collection::ToolCollection;

use crate::config::Authorization;
use crate::error::Error;
use crate::http_client::HttpClient;
use crate::security::SecurityObserver;
use reqwest::header::HeaderMap;
use rmcp::model::{CallToolResult, Tool as McpTool};
use serde_json::Value;
use url::Url;

/// Self-contained tool with embedded HTTP client
#[derive(Clone)]
pub struct Tool {
    pub metadata: ToolMetadata,
    http_client: HttpClient,
}

impl Tool {
    /// Create tool with HTTP configuration
    pub fn new(
        metadata: ToolMetadata,
        base_url: Option<Url>,
        default_headers: Option<HeaderMap>,
    ) -> Result<Self, Error> {
        let mut http_client = HttpClient::new();

        if let Some(url) = base_url {
            http_client = http_client.with_base_url(url)?;
        }

        if let Some(headers) = default_headers {
            http_client = http_client.with_default_headers(headers);
        }

        Ok(Self {
            metadata,
            http_client,
        })
    }

    /// Execute tool and return MCP-compliant result
    pub async fn call(
        &self,
        arguments: &Value,
        authorization: Authorization,
    ) -> Result<CallToolResult, crate::error::ToolCallError> {
        use rmcp::model::Content;
        use serde_json::json;

        // Create security observer for logging
        let observer = SecurityObserver::new(&authorization);

        // Log the authorization decision
        let has_auth = match &authorization {
            Authorization::None => false,
            #[cfg(feature = "authorization-token-passthrough")]
            Authorization::PassthroughWarn(header) | Authorization::PassthroughSilent(header) => {
                header.is_some()
            }
        };

        observer.observe_request(&self.metadata.name, has_auth, self.metadata.requires_auth());

        // Extract authorization header if present
        let auth_header: Option<&rmcp_actix_web::transport::AuthorizationHeader> =
            match &authorization {
                Authorization::None => None,
                #[cfg(feature = "authorization-token-passthrough")]
                Authorization::PassthroughWarn(header)
                | Authorization::PassthroughSilent(header) => header.as_ref(),
            };

        // Create HTTP client with authorization if provided
        let client = if let Some(auth) = auth_header {
            self.http_client.with_authorization(&auth.0)
        } else {
            self.http_client.clone()
        };

        // Execute the HTTP request using the (potentially auth-enhanced) HTTP client
        match client.execute_tool_call(&self.metadata, arguments).await {
            Ok(response) => {
                // Check if the tool has an output schema
                let structured_content = if self.metadata.output_schema.is_some() {
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
                        Ok(json_string) => vec![Content::text(json_string)],
                        Err(e) => {
                            // Return error if we can't serialize the structured content
                            let error = crate::error::ToolCallError::Execution(
                                crate::error::ToolCallExecutionError::ResponseParsingError {
                                    reason: format!("Failed to serialize structured content: {e}"),
                                    raw_response: None,
                                },
                            );
                            return Err(error);
                        }
                    }
                } else {
                    vec![Content::text(response.to_mcp_content())]
                };

                // Return successful response
                Ok(CallToolResult {
                    content,
                    structured_content,
                    is_error: Some(!response.is_success),
                })
            }
            Err(e) => {
                // Return ToolCallError directly
                Err(e)
            }
        }
    }

    /// Execute tool and return raw HTTP response
    pub async fn execute(
        &self,
        arguments: &Value,
        authorization: Authorization,
    ) -> Result<crate::http_client::HttpResponse, crate::error::ToolCallError> {
        // Extract authorization header if present
        let auth_header: Option<&rmcp_actix_web::transport::AuthorizationHeader> =
            match &authorization {
                Authorization::None => None,
                #[cfg(feature = "authorization-token-passthrough")]
                Authorization::PassthroughWarn(header)
                | Authorization::PassthroughSilent(header) => header.as_ref(),
            };

        // Create HTTP client with authorization if provided
        let client = if let Some(auth) = auth_header {
            self.http_client.with_authorization(&auth.0)
        } else {
            self.http_client.clone()
        };

        // Execute the HTTP request using the (potentially auth-enhanced) HTTP client
        // Return the raw HttpResponse without MCP formatting
        client.execute_tool_call(&self.metadata, arguments).await
    }
}

/// MCP compliance - Convert Tool to rmcp::model::Tool
impl From<&Tool> for McpTool {
    fn from(tool: &Tool) -> Self {
        (&tool.metadata).into()
    }
}
