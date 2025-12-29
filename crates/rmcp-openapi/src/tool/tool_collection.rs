use super::Tool;
use crate::config::Authorization;
use crate::error::{Error, ToolCallError, ToolCallValidationError};
use crate::transformer::ResponseTransformer;
use rmcp::model::{CallToolResult, Tool as McpTool};
use serde_json::Value;
use std::sync::Arc;
use tracing::debug_span;

/// Collection of tools with built-in validation and lookup capabilities
///
/// This struct encapsulates all tool management logic in the library layer,
/// providing a clean API for the binary to delegate tool operations to.
#[derive(Clone, Default)]
pub struct ToolCollection {
    tools: Vec<Tool>,
}

impl ToolCollection {
    /// Create a new empty tool collection
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Create a tool collection from a vector of tools
    pub fn from_tools(tools: Vec<Tool>) -> Self {
        Self { tools }
    }

    /// Add a tool to the collection
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// Get the number of tools in the collection
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Get all tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools
            .iter()
            .map(|tool| tool.metadata.name.clone())
            .collect()
    }

    /// Check if a specific tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|tool| tool.metadata.name == name)
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.metadata.name == name)
    }

    /// Convert all tools to MCP Tool format for list_tools response
    pub fn to_mcp_tools(&self) -> Vec<McpTool> {
        self.tools.iter().map(McpTool::from).collect()
    }

    /// Set a response transformer for a specific tool, overriding the global one.
    ///
    /// The transformer's `transform_schema` method is immediately applied to the tool's
    /// output schema. The `transform_response` method will be applied to responses
    /// when the tool is called.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool is not found
    pub fn set_tool_transformer(
        &mut self,
        tool_name: &str,
        transformer: Arc<dyn ResponseTransformer>,
    ) -> Result<(), Error> {
        let tool = self
            .tools
            .iter_mut()
            .find(|t| t.metadata.name == tool_name)
            .ok_or_else(|| Error::ToolNotFound(tool_name.to_string()))?;

        // Transform the existing output schema
        if let Some(schema) = tool.metadata.output_schema.take() {
            tool.metadata.output_schema = Some(transformer.transform_schema(schema));
        }

        tool.response_transformer = Some(transformer);
        Ok(())
    }

    /// Call a tool by name with validation
    ///
    /// This method encapsulates all tool validation logic:
    /// - Tool not found errors with suggestions
    /// - Parameter validation
    /// - Tool execution
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to call
    /// * `arguments` - The tool call arguments
    /// * `authorization` - Authorization configuration
    /// * `server_transformer` - Optional server-level response transformer
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: &Value,
        authorization: Authorization,
        server_transformer: Option<&dyn ResponseTransformer>,
    ) -> Result<CallToolResult, ToolCallError> {
        let span = debug_span!(
            "tool_execution",
            tool_name = %tool_name,
            total_tools = self.tools.len()
        );
        let _enter = span.enter();

        // First validate that the tool exists
        if let Some(tool) = self.get_tool(tool_name) {
            // Tool exists, delegate to the tool's call method
            tool.call(arguments, authorization, server_transformer)
                .await
        } else {
            // Tool not found - generate suggestions and return validation error
            let tool_names: Vec<&str> = self
                .tools
                .iter()
                .map(|tool| tool.metadata.name.as_str())
                .collect();

            Err(ToolCallError::Validation(
                ToolCallValidationError::tool_not_found(tool_name.to_string(), &tool_names),
            ))
        }
    }

    /// Get basic statistics about the tool collection
    pub fn get_stats(&self) -> String {
        format!("Total tools: {}", self.tools.len())
    }

    /// Get an iterator over the tools
    pub fn iter(&self) -> impl Iterator<Item = &Tool> {
        self.tools.iter()
    }
}

impl From<Vec<Tool>> for ToolCollection {
    fn from(tools: Vec<Tool>) -> Self {
        Self::from_tools(tools)
    }
}

impl IntoIterator for ToolCollection {
    type Item = Tool;
    type IntoIter = std::vec::IntoIter<Tool>;

    fn into_iter(self) -> Self::IntoIter {
        self.tools.into_iter()
    }
}

impl<'a> IntoIterator for &'a ToolCollection {
    type Item = &'a Tool;
    type IntoIter = std::slice::Iter<'a, Tool>;

    fn into_iter(self) -> Self::IntoIter {
        self.tools.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HttpClient, tool::ToolMetadata};
    use serde_json::json;

    fn create_test_tool(name: &str, description: &str) -> Tool {
        let metadata = ToolMetadata {
            name: name.to_string(),
            title: Some(name.to_string()),
            description: Some(description.to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer"}
                },
                "required": ["id"]
            }),
            output_schema: None,
            method: "GET".to_string(),
            path: format!("/{}", name),
            security: None,
            parameter_mappings: std::collections::HashMap::new(),
        };
        Tool::new(metadata, HttpClient::new()).unwrap()
    }

    #[test]
    fn test_tool_collection_creation() {
        let collection = ToolCollection::new();
        assert_eq!(collection.len(), 0);
        assert!(collection.is_empty());
    }

    #[test]
    fn test_tool_collection_from_tools() {
        let tool1 = create_test_tool("test1", "Test tool 1");
        let tool2 = create_test_tool("test2", "Test tool 2");
        let tools = vec![tool1, tool2];

        let collection = ToolCollection::from_tools(tools);
        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());
        assert!(collection.has_tool("test1"));
        assert!(collection.has_tool("test2"));
        assert!(!collection.has_tool("test3"));
    }

    #[test]
    fn test_add_tool() {
        let mut collection = ToolCollection::new();
        let tool = create_test_tool("test", "Test tool");

        collection.add_tool(tool);
        assert_eq!(collection.len(), 1);
        assert!(collection.has_tool("test"));
    }

    #[test]
    fn test_get_tool_names() {
        let tool1 = create_test_tool("getPetById", "Get pet by ID");
        let tool2 = create_test_tool("getPetsByStatus", "Get pets by status");
        let collection = ToolCollection::from_tools(vec![tool1, tool2]);

        let names = collection.get_tool_names();
        assert_eq!(names, vec!["getPetById", "getPetsByStatus"]);
    }

    #[test]
    fn test_get_tool() {
        let tool = create_test_tool("test", "Test tool");
        let collection = ToolCollection::from_tools(vec![tool]);

        assert!(collection.get_tool("test").is_some());
        assert!(collection.get_tool("nonexistent").is_none());
    }

    #[test]
    fn test_to_mcp_tools() {
        let tool1 = create_test_tool("test1", "Test tool 1");
        let tool2 = create_test_tool("test2", "Test tool 2");
        let collection = ToolCollection::from_tools(vec![tool1, tool2]);

        let mcp_tools = collection.to_mcp_tools();
        assert_eq!(mcp_tools.len(), 2);
        assert_eq!(mcp_tools[0].name, "test1");
        assert_eq!(mcp_tools[1].name, "test2");
    }

    #[actix_web::test]
    async fn test_call_tool_not_found_with_suggestions() {
        let tool1 = create_test_tool("getPetById", "Get pet by ID");
        let tool2 = create_test_tool("getPetsByStatus", "Get pets by status");
        let collection = ToolCollection::from_tools(vec![tool1, tool2]);

        let result = collection
            .call_tool("getPetByID", &json!({}), Authorization::default(), None)
            .await;
        assert!(result.is_err());

        if let Err(ToolCallError::Validation(ToolCallValidationError::ToolNotFound {
            tool_name,
            suggestions,
        })) = result
        {
            assert_eq!(tool_name, "getPetByID");
            // The algorithm finds multiple similar matches
            assert!(suggestions.contains(&"getPetById".to_string()));
            assert!(!suggestions.is_empty());
        } else {
            panic!("Expected ToolNotFound error with suggestions");
        }
    }

    #[actix_web::test]
    async fn test_call_tool_not_found_no_suggestions() {
        let tool = create_test_tool("getPetById", "Get pet by ID");
        let collection = ToolCollection::from_tools(vec![tool]);

        let result = collection
            .call_tool(
                "completelyDifferentName",
                &json!({}),
                Authorization::default(),
                None,
            )
            .await;
        assert!(result.is_err());

        if let Err(ToolCallError::Validation(ToolCallValidationError::ToolNotFound {
            tool_name,
            suggestions,
        })) = result
        {
            assert_eq!(tool_name, "completelyDifferentName");
            assert!(suggestions.is_empty());
        } else {
            panic!("Expected ToolNotFound error with no suggestions");
        }
    }

    #[test]
    fn test_iterators() {
        let tool1 = create_test_tool("test1", "Test tool 1");
        let tool2 = create_test_tool("test2", "Test tool 2");
        let collection = ToolCollection::from_tools(vec![tool1, tool2]);

        // Test iter()
        let names: Vec<String> = collection
            .iter()
            .map(|tool| tool.metadata.name.clone())
            .collect();
        assert_eq!(names, vec!["test1", "test2"]);

        // Test IntoIterator for &collection
        let names: Vec<String> = (&collection)
            .into_iter()
            .map(|tool| tool.metadata.name.clone())
            .collect();
        assert_eq!(names, vec!["test1", "test2"]);

        // Test IntoIterator for collection (consumes it)
        let names: Vec<String> = collection
            .into_iter()
            .map(|tool| tool.metadata.name.clone())
            .collect();
        assert_eq!(names, vec!["test1", "test2"]);
    }

    #[test]
    fn test_from_vec() {
        let tool1 = create_test_tool("test1", "Test tool 1");
        let tool2 = create_test_tool("test2", "Test tool 2");
        let tools = vec![tool1, tool2];

        let collection: ToolCollection = tools.into();
        assert_eq!(collection.len(), 2);
    }
}
