use rmcp::model::{Tool, ToolAnnotations};
use serde_json::Value;
use std::sync::Arc;

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
    /// Security requirements from OpenAPI spec - internal only, not exposed to MCP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<String>>,
}

impl ToolMetadata {
    /// Check if this tool requires authentication based on OpenAPI security definitions
    pub fn requires_auth(&self) -> bool {
        self.security.as_ref().is_some_and(|s| !s.is_empty())
    }
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
