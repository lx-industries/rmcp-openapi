use crate::error::Error;
use crate::spec::Spec;
use crate::tool::ToolMetadata;
use std::collections::HashMap;

/// Registry for managing dynamically generated MCP tools from `OpenAPI` operations
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// Map of tool name to tool metadata
    tools: HashMap<String, ToolMetadata>,
    /// Map of tool name to `OpenAPI` operation for runtime lookup
    operations: HashMap<String, (oas3::spec::Operation, String, String)>,
    /// Source spec for reference
    spec: Option<Spec>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            operations: HashMap::new(),
            spec: None,
        }
    }

    /// Register tools from a specification
    ///
    /// # Errors
    ///
    /// Returns an error if any tool fails to be generated or registered
    pub fn register_from_spec(
        &mut self,
        spec: Spec,
        tag_filter: Option<&[String]>,
        method_filter: Option<&[reqwest::Method]>,
    ) -> Result<usize, Error> {
        // Clear existing tools
        self.clear();

        // Convert operations to tool metadata
        let tools_metadata = spec.to_tool_metadata(tag_filter, method_filter)?;
        let mut registered_count = 0;

        // Register each tool
        for tool in tools_metadata {
            // Find corresponding operation
            if let Some((operation, method, path)) = spec.get_operation(&tool.name) {
                self.register_tool(tool, (operation.clone(), method, path))?;
                registered_count += 1;
            }
        }

        // Store the spec
        self.spec = Some(spec);

        Ok(registered_count)
    }

    /// Register a single tool with its corresponding operation
    ///
    /// # Errors
    ///
    /// Returns an error if the tool metadata is invalid or the tool name already exists
    pub fn register_tool(
        &mut self,
        tool: ToolMetadata,
        operation: (oas3::spec::Operation, String, String),
    ) -> Result<(), Error> {
        let tool_name = tool.name.clone();

        // Validate tool metadata
        self.validate_tool(&tool)?;

        // Store tool metadata and operation
        self.tools.insert(tool_name.clone(), tool);
        self.operations.insert(tool_name, operation);

        Ok(())
    }

    /// Validate tool metadata
    fn validate_tool(&self, tool: &ToolMetadata) -> Result<(), Error> {
        if tool.name.is_empty() {
            return Err(Error::ToolGeneration(
                "Tool name cannot be empty".to_string(),
            ));
        }

        if tool.method.is_empty() {
            return Err(Error::ToolGeneration(
                "Tool method cannot be empty".to_string(),
            ));
        }

        if tool.path.is_empty() {
            return Err(Error::ToolGeneration(
                "Tool path cannot be empty".to_string(),
            ));
        }

        // Validate that the tool name is unique
        if self.tools.contains_key(&tool.name) {
            return Err(Error::ToolGeneration(format!(
                "Tool '{}' already exists",
                tool.name
            )));
        }

        Ok(())
    }

    /// Get tool metadata by name
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<&ToolMetadata> {
        self.tools.get(name)
    }

    /// Get operation by tool name
    #[must_use]
    pub fn get_operation(
        &self,
        tool_name: &str,
    ) -> Option<&(oas3::spec::Operation, String, String)> {
        self.operations.get(tool_name)
    }

    /// Get all tool names
    #[must_use]
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all tools
    #[must_use]
    pub fn get_all_tools(&self) -> Vec<&ToolMetadata> {
        self.tools.values().collect()
    }

    /// Get number of registered tools
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Check if a tool exists
    #[must_use]
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Remove a tool by name
    pub fn remove_tool(&mut self, name: &str) -> Option<ToolMetadata> {
        self.operations.remove(name);
        self.tools.remove(name)
    }

    /// Clear all tools
    pub fn clear(&mut self) {
        self.tools.clear();
        self.operations.clear();
        self.spec = None;
    }

    /// Get the source `OpenAPI` spec
    #[must_use]
    pub fn get_spec(&self) -> Option<&Spec> {
        self.spec.as_ref()
    }

    /// Get registry statistics
    #[must_use]
    pub fn get_stats(&self) -> ToolRegistryStats {
        let mut method_counts = HashMap::new();
        let mut path_counts = HashMap::new();

        for tool in self.tools.values() {
            *method_counts.entry(tool.method.clone()).or_insert(0) += 1;
            *path_counts.entry(tool.path.clone()).or_insert(0) += 1;
        }

        ToolRegistryStats {
            total_tools: self.tools.len(),
            method_distribution: method_counts,
            unique_paths: path_counts.len(),
            has_spec: self.spec.is_some(),
        }
    }

    /// Validate all tools in the registry
    ///
    /// # Errors
    ///
    /// Returns an error if any tool is missing its operation, has invalid metadata, or there are orphaned operations
    pub fn validate_registry(&self) -> Result<(), Error> {
        for tool in self.tools.values() {
            // Check if corresponding operation exists
            if !self.operations.contains_key(&tool.name) {
                return Err(Error::ToolGeneration(format!(
                    "Missing operation for tool '{}'",
                    tool.name
                )));
            }

            // Validate tool metadata schema
            Self::validate_tool_metadata(&tool.name, tool)?;
        }

        // Check for orphaned operations
        for operation_name in self.operations.keys() {
            if !self.tools.contains_key(operation_name) {
                return Err(Error::ToolGeneration(format!(
                    "Orphaned operation '{operation_name}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate a single tool's metadata
    fn validate_tool_metadata(tool_name: &str, tool_metadata: &ToolMetadata) -> Result<(), Error> {
        // Check that the tool has valid parameters schema
        if !tool_metadata.parameters.is_object() {
            return Err(Error::Validation(format!(
                "Tool '{tool_name}' has invalid parameters schema - must be an object"
            )));
        }

        let schema_obj = tool_metadata.parameters.as_object().unwrap();

        // Check for required properties field
        if let Some(properties) = schema_obj.get("properties") {
            if !properties.is_object() {
                return Err(Error::Validation(format!(
                    "Tool '{tool_name}' properties field must be an object"
                )));
            }
        } else {
            return Err(Error::Validation(format!(
                "Tool '{tool_name}' is missing properties field in parameters schema"
            )));
        }

        // Validate required field if present
        if let Some(required) = schema_obj.get("required")
            && !required.is_array()
        {
            return Err(Error::Validation(format!(
                "Tool '{tool_name}' required field must be an array"
            )));
        }

        // Check HTTP method is valid
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&tool_metadata.method.to_uppercase().as_str()) {
            return Err(Error::Validation(format!(
                "Tool '{}' has invalid HTTP method: {}",
                tool_name, tool_metadata.method
            )));
        }

        // Check path is not empty
        if tool_metadata.path.is_empty() {
            return Err(Error::Validation(format!(
                "Tool '{tool_name}' has empty path"
            )));
        }

        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the tool registry
#[derive(Debug, Clone)]
pub struct ToolRegistryStats {
    pub total_tools: usize,
    pub method_distribution: HashMap<String, usize>,
    pub unique_paths: usize,
    pub has_spec: bool,
}

impl ToolRegistryStats {
    /// Get a summary string of the registry stats
    #[must_use]
    pub fn summary(&self) -> String {
        let methods: Vec<String> = self
            .method_distribution
            .iter()
            .map(|(method, count)| format!("{}: {}", method.to_uppercase(), count))
            .collect();

        format!(
            "Tools: {}, Methods: [{}], Paths: {}, Spec: {}",
            self.total_tools,
            methods.join(", "),
            self.unique_paths,
            if self.has_spec { "loaded" } else { "none" }
        )
    }
}
