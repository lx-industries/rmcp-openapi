use rmcp::model::{Tool, ToolAnnotations};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

/// Parameter mapping information for converting between MCP and OpenAPI parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ParameterMapping {
    /// The sanitized parameter name used in MCP
    pub sanitized_name: String,
    /// The original parameter name from OpenAPI
    pub original_name: String,
    /// The location of the parameter (query, header, path, cookie, body)
    pub location: String,
    /// Whether the parameter should be exploded (for arrays/objects)
    pub explode: bool,
}

/// Internal metadata for tools generated from OpenAPI operations.
///
/// This struct contains all the information needed to execute HTTP requests
/// and is used internally by the OpenAPI server. It includes fields that are
/// not part of the MCP specification but are necessary for HTTP execution.
///
/// For MCP compliance, this struct is converted to `rmcp::model::Tool` using
/// the `From` trait implementation, which only includes MCP-compliant fields.
///
/// ## MCP Tool Annotations
///
/// When converted to MCP tools, this metadata automatically generates appropriate
/// annotation hints based on HTTP method semantics (see [`ToolMetadata::generate_annotations`]).
/// These annotations help MCP clients understand the nature of each tool operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolMetadata {
    /// Tool name - exposed to MCP clients
    pub name: String,
    /// Tool title - human-readable display name exposed to MCP clients
    pub title: Option<String>,
    /// Tool description - exposed to MCP clients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
    /// Parameter mappings for converting between MCP and OpenAPI parameters - internal only, not exposed to MCP
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub parameter_mappings: HashMap<String, ParameterMapping>,
}

impl ToolMetadata {
    /// Check if this tool requires authentication based on OpenAPI security definitions
    pub fn requires_auth(&self) -> bool {
        self.security.as_ref().is_some_and(|s| !s.is_empty())
    }

    /// Generate MCP annotations based on HTTP method semantics.
    ///
    /// This method maps HTTP verbs to appropriate MCP tool annotation hints following
    /// the semantics defined in RFC 9110 (HTTP Semantics) and the Model Context Protocol
    /// specification.
    ///
    /// # HTTP Method to Annotation Mapping
    ///
    /// - **GET, HEAD, OPTIONS**: Safe, idempotent read operations
    ///   - `readOnlyHint: true` - No state modification
    ///   - `destructiveHint: false` - Doesn't alter existing resources
    ///   - `idempotentHint: true` - Multiple requests have same effect
    ///   - `openWorldHint: true` - Interacts with external HTTP API
    ///
    /// - **POST**: Creates resources; not idempotent, not destructive
    ///   - `readOnlyHint: false` - Modifies state
    ///   - `destructiveHint: false` - Creates new resources (doesn't destroy existing)
    ///   - `idempotentHint: false` - Multiple requests may create multiple resources
    ///   - `openWorldHint: true` - Interacts with external HTTP API
    ///
    /// - **PUT**: Replaces/updates resources; idempotent but destructive
    ///   - `readOnlyHint: false` - Modifies state
    ///   - `destructiveHint: true` - Replaces existing resource state
    ///   - `idempotentHint: true` - Multiple identical requests have same effect
    ///   - `openWorldHint: true` - Interacts with external HTTP API
    ///
    /// - **PATCH**: Modifies resources; destructive and typically not idempotent
    ///   - `readOnlyHint: false` - Modifies state
    ///   - `destructiveHint: true` - Alters existing resource state
    ///   - `idempotentHint: false` - Effect may vary based on current state
    ///   - `openWorldHint: true` - Interacts with external HTTP API
    ///
    /// - **DELETE**: Removes resources; idempotent but destructive
    ///   - `readOnlyHint: false` - Modifies state
    ///   - `destructiveHint: true` - Removes resources
    ///   - `idempotentHint: true` - Multiple deletions are no-ops after first
    ///   - `openWorldHint: true` - Interacts with external HTTP API
    ///
    /// # Returns
    ///
    /// - `Some(ToolAnnotations)` for recognized HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
    /// - `None` for unknown or unsupported HTTP methods
    ///
    /// # Notes
    ///
    /// - HTTP method comparison is case-insensitive
    /// - The `title` field in annotations is always `None` (title is handled via `Tool.title`)
    /// - `openWorldHint` is always `true` since all OpenAPI tools interact with external HTTP APIs
    pub fn generate_annotations(&self) -> Option<ToolAnnotations> {
        match self.method.to_uppercase().as_str() {
            "GET" | "HEAD" | "OPTIONS" => Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(true),
                destructive_hint: Some(false),
                idempotent_hint: Some(true),
                open_world_hint: Some(true),
            }),
            "POST" => Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(false),
                destructive_hint: Some(false),
                idempotent_hint: Some(false),
                open_world_hint: Some(true),
            }),
            "PUT" => Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(false),
                destructive_hint: Some(true),
                idempotent_hint: Some(true),
                open_world_hint: Some(true),
            }),
            "PATCH" => Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(false),
                destructive_hint: Some(true),
                idempotent_hint: Some(false),
                open_world_hint: Some(true),
            }),
            "DELETE" => Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(false),
                destructive_hint: Some(true),
                idempotent_hint: Some(true),
                open_world_hint: Some(true),
            }),
            _ => None,
        }
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

        Tool {
            name: metadata.name.clone().into(),
            description: metadata.description.clone().map(|d| d.into()),
            input_schema,
            output_schema,
            annotations: metadata.generate_annotations(),
            title: metadata.title.clone(),
            icons: None,
            meta: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Helper function to create test metadata with a specific HTTP method
    fn create_test_metadata(method: &str) -> ToolMetadata {
        ToolMetadata {
            name: "test_tool".to_string(),
            title: None,
            description: None,
            parameters: json!({}),
            output_schema: None,
            method: method.to_string(),
            path: "/test".to_string(),
            security: None,
            parameter_mappings: HashMap::new(),
        }
    }

    #[test]
    fn test_get_annotations() {
        let metadata = create_test_metadata("GET");
        let annotations = metadata
            .generate_annotations()
            .expect("GET should return annotations");

        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_post_annotations() {
        let metadata = create_test_metadata("POST");
        let annotations = metadata
            .generate_annotations()
            .expect("POST should return annotations");

        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_put_annotations() {
        let metadata = create_test_metadata("PUT");
        let annotations = metadata
            .generate_annotations()
            .expect("PUT should return annotations");

        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_patch_annotations() {
        let metadata = create_test_metadata("PATCH");
        let annotations = metadata
            .generate_annotations()
            .expect("PATCH should return annotations");

        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_delete_annotations() {
        let metadata = create_test_metadata("DELETE");
        let annotations = metadata
            .generate_annotations()
            .expect("DELETE should return annotations");

        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_head_annotations() {
        let metadata = create_test_metadata("HEAD");
        let annotations = metadata
            .generate_annotations()
            .expect("HEAD should return annotations");

        // HEAD should have the same annotations as GET
        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_options_annotations() {
        let metadata = create_test_metadata("OPTIONS");
        let annotations = metadata
            .generate_annotations()
            .expect("OPTIONS should return annotations");

        // OPTIONS should have the same annotations as GET
        assert_eq!(annotations.title, None);
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn test_unknown_method_returns_none() {
        // Test various unknown/unsupported HTTP methods
        let unknown_methods = vec!["TRACE", "CONNECT", "CUSTOM", "INVALID", "UNKNOWN"];

        for method in unknown_methods {
            let metadata = create_test_metadata(method);
            let annotations = metadata.generate_annotations();
            assert_eq!(
                annotations, None,
                "Unknown method '{}' should return None",
                method
            );
        }
    }

    #[test]
    fn test_case_insensitive_method_matching() {
        // Test that method matching is case-insensitive
        let get_variations = vec!["GET", "get", "Get", "gEt", "GeT"];

        for method in get_variations {
            let metadata = create_test_metadata(method);
            let annotations = metadata
                .generate_annotations()
                .unwrap_or_else(|| panic!("'{}' should return annotations", method));

            // All variations should produce GET annotations
            assert_eq!(annotations.read_only_hint, Some(true));
            assert_eq!(annotations.destructive_hint, Some(false));
            assert_eq!(annotations.idempotent_hint, Some(true));
            assert_eq!(annotations.open_world_hint, Some(true));
        }

        // Test POST variations too
        let post_variations = vec!["POST", "post", "Post"];

        for method in post_variations {
            let metadata = create_test_metadata(method);
            let annotations = metadata
                .generate_annotations()
                .unwrap_or_else(|| panic!("'{}' should return annotations", method));

            // All variations should produce POST annotations
            assert_eq!(annotations.read_only_hint, Some(false));
            assert_eq!(annotations.destructive_hint, Some(false));
            assert_eq!(annotations.idempotent_hint, Some(false));
            assert_eq!(annotations.open_world_hint, Some(true));
        }
    }

    #[test]
    fn test_annotations_title_always_none() {
        // Verify that title field in annotations is always None for all methods
        let all_methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

        for method in all_methods {
            let metadata = create_test_metadata(method);
            let annotations = metadata
                .generate_annotations()
                .unwrap_or_else(|| panic!("'{}' should return annotations", method));

            assert_eq!(
                annotations.title, None,
                "Method '{}' should have title=None in annotations",
                method
            );
        }
    }
}
