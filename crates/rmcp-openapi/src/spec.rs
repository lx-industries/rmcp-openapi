use crate::error::Error;
use crate::normalize_tag;
use crate::tool::ToolMetadata;
use crate::tool_generator::ToolGenerator;
use oas3::Spec as Oas3Spec;
use reqwest::Method;
use serde_json::Value;

/// OpenAPI specification wrapper that provides convenience methods
/// for working with oas3::Spec
#[derive(Debug, Clone)]
pub struct Spec {
    pub spec: Oas3Spec,
}

impl Spec {
    /// Parse an OpenAPI specification from a JSON value
    pub fn from_value(json_value: Value) -> Result<Self, Error> {
        let spec: Oas3Spec = serde_json::from_value(json_value)?;
        Ok(Spec { spec })
    }

    /// Convert all operations to MCP tool metadata
    pub fn to_tool_metadata(
        &self,
        tag_filter: Option<&[String]>,
        method_filter: Option<&[reqwest::Method]>,
        skip_tool_descriptions: bool,
    ) -> Result<Vec<ToolMetadata>, Error> {
        let mut tools = Vec::new();

        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths {
                // Handle operations in the path item
                let operations = [
                    (Method::GET, &path_item.get),
                    (Method::POST, &path_item.post),
                    (Method::PUT, &path_item.put),
                    (Method::DELETE, &path_item.delete),
                    (Method::PATCH, &path_item.patch),
                    (Method::HEAD, &path_item.head),
                    (Method::OPTIONS, &path_item.options),
                    (Method::TRACE, &path_item.trace),
                ];

                for (method, operation_ref) in operations {
                    if let Some(operation) = operation_ref {
                        // Filter by methods if specified
                        if let Some(filter_methods) = method_filter
                            && !filter_methods.contains(&method)
                        {
                            continue; // Skip this operation
                        }

                        // Filter by tags if specified (with kebab-case normalization)
                        if let Some(filter_tags) = tag_filter {
                            if !operation.tags.is_empty() {
                                // Normalize both filter tags and operation tags before comparison
                                let normalized_filter_tags: Vec<String> =
                                    filter_tags.iter().map(|tag| normalize_tag(tag)).collect();

                                let has_matching_tag = operation.tags.iter().any(|operation_tag| {
                                    let normalized_operation_tag = normalize_tag(operation_tag);
                                    normalized_filter_tags.contains(&normalized_operation_tag)
                                });

                                if !has_matching_tag {
                                    continue; // Skip this operation
                                }
                            } else {
                                continue; // Skip operations without tags when filtering
                            }
                        }

                        let tool_metadata = ToolGenerator::generate_tool_metadata(
                            operation,
                            method.to_string(),
                            path.clone(),
                            &self.spec,
                            skip_tool_descriptions,
                        )?;
                        tools.push(tool_metadata);
                    }
                }
            }
        }

        Ok(tools)
    }

    /// Convert all operations to OpenApiTool instances with HTTP configuration
    ///
    /// # Errors
    ///
    /// Returns an error if any operations cannot be converted or OpenApiTool instances cannot be created
    pub fn to_openapi_tools(
        &self,
        tag_filter: Option<&[String]>,
        method_filter: Option<&[reqwest::Method]>,
        base_url: Option<url::Url>,
        default_headers: Option<reqwest::header::HeaderMap>,
        skip_tool_descriptions: bool,
    ) -> Result<Vec<crate::tool::Tool>, Error> {
        // First generate the tool metadata using existing method
        let tools_metadata =
            self.to_tool_metadata(tag_filter, method_filter, skip_tool_descriptions)?;

        // Then convert to Tool instances
        crate::tool_generator::ToolGenerator::generate_openapi_tools(
            tools_metadata,
            base_url,
            default_headers,
        )
    }

    /// Get operation by operation ID
    pub fn get_operation(
        &self,
        operation_id: &str,
    ) -> Option<(&oas3::spec::Operation, String, String)> {
        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths {
                let operations = [
                    (Method::GET, &path_item.get),
                    (Method::POST, &path_item.post),
                    (Method::PUT, &path_item.put),
                    (Method::DELETE, &path_item.delete),
                    (Method::PATCH, &path_item.patch),
                    (Method::HEAD, &path_item.head),
                    (Method::OPTIONS, &path_item.options),
                    (Method::TRACE, &path_item.trace),
                ];

                for (method, operation_ref) in operations {
                    if let Some(operation) = operation_ref {
                        let default_id = format!(
                            "{}_{}",
                            method,
                            path.replace('/', "_").replace(['{', '}'], "")
                        );
                        let op_id = operation.operation_id.as_deref().unwrap_or(&default_id);

                        if op_id == operation_id {
                            return Some((operation, method.to_string(), path.clone()));
                        }
                    }
                }
            }
        }
        None
    }

    /// Get all operation IDs
    pub fn get_operation_ids(&self) -> Vec<String> {
        let mut operation_ids = Vec::new();

        if let Some(paths) = &self.spec.paths {
            for (path, path_item) in paths {
                let operations = [
                    (Method::GET, &path_item.get),
                    (Method::POST, &path_item.post),
                    (Method::PUT, &path_item.put),
                    (Method::DELETE, &path_item.delete),
                    (Method::PATCH, &path_item.patch),
                    (Method::HEAD, &path_item.head),
                    (Method::OPTIONS, &path_item.options),
                    (Method::TRACE, &path_item.trace),
                ];

                for (method, operation_ref) in operations {
                    if let Some(operation) = operation_ref {
                        let default_id = format!(
                            "{}_{}",
                            method,
                            path.replace('/', "_").replace(['{', '}'], "")
                        );
                        let op_id = operation.operation_id.as_deref().unwrap_or(&default_id);
                        operation_ids.push(op_id.to_string());
                    }
                }
            }
        }

        operation_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_spec_with_tags() -> Spec {
        let spec_json = json!({
            "openapi": "3.0.3",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/pets": {
                    "get": {
                        "operationId": "listPets",
                        "tags": ["pet", "list"],
                        "responses": {
                            "200": {
                                "description": "List of pets"
                            }
                        }
                    },
                    "post": {
                        "operationId": "createPet",
                        "tags": ["pet"],
                        "responses": {
                            "201": {
                                "description": "Pet created"
                            }
                        }
                    }
                },
                "/users": {
                    "get": {
                        "operationId": "listUsers",
                        "tags": ["user"],
                        "responses": {
                            "200": {
                                "description": "List of users"
                            }
                        }
                    }
                },
                "/admin": {
                    "get": {
                        "operationId": "adminPanel",
                        "tags": ["admin", "management"],
                        "responses": {
                            "200": {
                                "description": "Admin panel"
                            }
                        }
                    }
                },
                "/public": {
                    "get": {
                        "operationId": "publicEndpoint",
                        "responses": {
                            "200": {
                                "description": "Public endpoint with no tags"
                            }
                        }
                    }
                }
            }
        });

        Spec::from_value(spec_json).expect("Failed to create test spec")
    }

    fn create_test_spec_with_mixed_case_tags() -> Spec {
        let spec_json = json!({
            "openapi": "3.0.3",
            "info": {
                "title": "Test API with Mixed Case Tags",
                "version": "1.0.0"
            },
            "paths": {
                "/camel": {
                    "get": {
                        "operationId": "camelCaseOperation",
                        "tags": ["userManagement"],
                        "responses": {
                            "200": {
                                "description": "camelCase tag"
                            }
                        }
                    }
                },
                "/pascal": {
                    "get": {
                        "operationId": "pascalCaseOperation",
                        "tags": ["UserManagement"],
                        "responses": {
                            "200": {
                                "description": "PascalCase tag"
                            }
                        }
                    }
                },
                "/snake": {
                    "get": {
                        "operationId": "snakeCaseOperation",
                        "tags": ["user_management"],
                        "responses": {
                            "200": {
                                "description": "snake_case tag"
                            }
                        }
                    }
                },
                "/screaming": {
                    "get": {
                        "operationId": "screamingCaseOperation",
                        "tags": ["USER_MANAGEMENT"],
                        "responses": {
                            "200": {
                                "description": "SCREAMING_SNAKE_CASE tag"
                            }
                        }
                    }
                },
                "/kebab": {
                    "get": {
                        "operationId": "kebabCaseOperation",
                        "tags": ["user-management"],
                        "responses": {
                            "200": {
                                "description": "kebab-case tag"
                            }
                        }
                    }
                },
                "/mixed": {
                    "get": {
                        "operationId": "mixedCaseOperation",
                        "tags": ["XMLHttpRequest", "HTTPSConnection", "APIKey"],
                        "responses": {
                            "200": {
                                "description": "Mixed case with acronyms"
                            }
                        }
                    }
                }
            }
        });

        Spec::from_value(spec_json).expect("Failed to create test spec")
    }

    fn create_test_spec_with_methods() -> Spec {
        let spec_json = json!({
            "openapi": "3.0.3",
            "info": {
                "title": "Test API with Multiple Methods",
                "version": "1.0.0"
            },
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "listUsers",
                        "tags": ["user"],
                        "responses": {
                            "200": {
                                "description": "List of users"
                            }
                        }
                    },
                    "post": {
                        "operationId": "createUser",
                        "tags": ["user"],
                        "responses": {
                            "201": {
                                "description": "User created"
                            }
                        }
                    },
                    "put": {
                        "operationId": "updateUser",
                        "tags": ["user"],
                        "responses": {
                            "200": {
                                "description": "User updated"
                            }
                        }
                    },
                    "delete": {
                        "operationId": "deleteUser",
                        "tags": ["user"],
                        "responses": {
                            "204": {
                                "description": "User deleted"
                            }
                        }
                    }
                },
                "/pets": {
                    "get": {
                        "operationId": "listPets",
                        "tags": ["pet"],
                        "responses": {
                            "200": {
                                "description": "List of pets"
                            }
                        }
                    },
                    "post": {
                        "operationId": "createPet",
                        "tags": ["pet"],
                        "responses": {
                            "201": {
                                "description": "Pet created"
                            }
                        }
                    },
                    "patch": {
                        "operationId": "patchPet",
                        "tags": ["pet"],
                        "responses": {
                            "200": {
                                "description": "Pet patched"
                            }
                        }
                    }
                },
                "/health": {
                    "head": {
                        "operationId": "healthCheck",
                        "tags": ["health"],
                        "responses": {
                            "200": {
                                "description": "Health check"
                            }
                        }
                    },
                    "options": {
                        "operationId": "healthOptions",
                        "tags": ["health"],
                        "responses": {
                            "200": {
                                "description": "Health options"
                            }
                        }
                    }
                }
            }
        });

        Spec::from_value(spec_json).expect("Failed to create test spec")
    }

    #[test]
    fn test_tag_filtering_no_filter() {
        let spec = create_test_spec_with_tags();
        let tools = spec
            .to_tool_metadata(None, None, false)
            .expect("Failed to generate tools");

        // All operations should be included
        assert_eq!(tools.len(), 5);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets"));
        assert!(tool_names.contains(&"createPet"));
        assert!(tool_names.contains(&"listUsers"));
        assert!(tool_names.contains(&"adminPanel"));
        assert!(tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_tag_filtering_single_tag() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["pet".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Only pet operations should be included
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets"));
        assert!(tool_names.contains(&"createPet"));
        assert!(!tool_names.contains(&"listUsers"));
        assert!(!tool_names.contains(&"adminPanel"));
        assert!(!tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_tag_filtering_multiple_tags() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["pet".to_string(), "user".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Pet and user operations should be included
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets"));
        assert!(tool_names.contains(&"createPet"));
        assert!(tool_names.contains(&"listUsers"));
        assert!(!tool_names.contains(&"adminPanel"));
        assert!(!tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_tag_filtering_or_logic() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["list".to_string()]; // listPets has both "pet" and "list" tags
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Only operations with "list" tag should be included
        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listPets")); // Has both "pet" and "list" tags
        assert!(!tool_names.contains(&"createPet")); // Only has "pet" tag
    }

    #[test]
    fn test_tag_filtering_no_matching_tags() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["nonexistent".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // No operations should be included
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_tag_filtering_excludes_operations_without_tags() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["admin".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Only admin operations should be included, public endpoint (no tags) should be excluded
        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"adminPanel"));
        assert!(!tool_names.contains(&"publicEndpoint")); // No tags, should be excluded
    }

    #[test]
    fn test_tag_normalization_all_cases_match() {
        let spec = create_test_spec_with_mixed_case_tags();
        let filter_tags = vec!["user-management".to_string()]; // kebab-case filter
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // All userManagement variants should match user-management filter
        assert_eq!(tools.len(), 5);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"camelCaseOperation")); // userManagement
        assert!(tool_names.contains(&"pascalCaseOperation")); // UserManagement
        assert!(tool_names.contains(&"snakeCaseOperation")); // user_management
        assert!(tool_names.contains(&"screamingCaseOperation")); // USER_MANAGEMENT
        assert!(tool_names.contains(&"kebabCaseOperation")); // user-management
        assert!(!tool_names.contains(&"mixedCaseOperation")); // Different tags
    }

    #[test]
    fn test_tag_normalization_camel_case_filter() {
        let spec = create_test_spec_with_mixed_case_tags();
        let filter_tags = vec!["userManagement".to_string()]; // camelCase filter
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // All userManagement variants should match camelCase filter
        assert_eq!(tools.len(), 5);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"camelCaseOperation"));
        assert!(tool_names.contains(&"pascalCaseOperation"));
        assert!(tool_names.contains(&"snakeCaseOperation"));
        assert!(tool_names.contains(&"screamingCaseOperation"));
        assert!(tool_names.contains(&"kebabCaseOperation"));
    }

    #[test]
    fn test_tag_normalization_snake_case_filter() {
        let spec = create_test_spec_with_mixed_case_tags();
        let filter_tags = vec!["user_management".to_string()]; // snake_case filter
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // All userManagement variants should match snake_case filter
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_tag_normalization_acronyms() {
        let spec = create_test_spec_with_mixed_case_tags();
        let filter_tags = vec!["xml-http-request".to_string()]; // kebab-case filter for acronym
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Should match XMLHttpRequest tag
        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"mixedCaseOperation"));
    }

    #[test]
    fn test_tag_normalization_multiple_mixed_filters() {
        let spec = create_test_spec_with_mixed_case_tags();
        let filter_tags = vec![
            "user-management".to_string(), // kebab-case
            "HTTPSConnection".to_string(), // PascalCase with acronym
        ];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Should match all userManagement variants + mixedCaseOperation (for HTTPSConnection)
        assert_eq!(tools.len(), 6);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"camelCaseOperation"));
        assert!(tool_names.contains(&"pascalCaseOperation"));
        assert!(tool_names.contains(&"snakeCaseOperation"));
        assert!(tool_names.contains(&"screamingCaseOperation"));
        assert!(tool_names.contains(&"kebabCaseOperation"));
        assert!(tool_names.contains(&"mixedCaseOperation"));
    }

    #[test]
    fn test_tag_filtering_empty_filter_list() {
        let spec = create_test_spec_with_tags();
        let filter_tags: Vec<String> = vec![];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Empty filter should exclude all operations
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_tag_filtering_complex_scenario() {
        let spec = create_test_spec_with_tags();
        let filter_tags = vec!["management".to_string(), "list".to_string()];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), None, false)
            .expect("Failed to generate tools");

        // Should include adminPanel (has "management") and listPets (has "list")
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"adminPanel"));
        assert!(tool_names.contains(&"listPets"));
        assert!(!tool_names.contains(&"createPet"));
        assert!(!tool_names.contains(&"listUsers"));
        assert!(!tool_names.contains(&"publicEndpoint"));
    }

    #[test]
    fn test_method_filtering_no_filter() {
        let spec = create_test_spec_with_methods();
        let tools = spec
            .to_tool_metadata(None, None, false)
            .expect("Failed to generate tools");

        // All operations should be included (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
        assert_eq!(tools.len(), 9);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listUsers")); // GET /users
        assert!(tool_names.contains(&"createUser")); // POST /users
        assert!(tool_names.contains(&"updateUser")); // PUT /users
        assert!(tool_names.contains(&"deleteUser")); // DELETE /users
        assert!(tool_names.contains(&"listPets")); // GET /pets
        assert!(tool_names.contains(&"createPet")); // POST /pets
        assert!(tool_names.contains(&"patchPet")); // PATCH /pets
        assert!(tool_names.contains(&"healthCheck")); // HEAD /health
        assert!(tool_names.contains(&"healthOptions")); // OPTIONS /health
    }

    #[test]
    fn test_method_filtering_single_method() {
        use reqwest::Method;

        let spec = create_test_spec_with_methods();
        let filter_methods = vec![Method::GET];
        let tools = spec
            .to_tool_metadata(None, Some(&filter_methods), false)
            .expect("Failed to generate tools");

        // Only GET operations should be included
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listUsers")); // GET /users
        assert!(tool_names.contains(&"listPets")); // GET /pets
        assert!(!tool_names.contains(&"createUser")); // POST /users
        assert!(!tool_names.contains(&"updateUser")); // PUT /users
        assert!(!tool_names.contains(&"deleteUser")); // DELETE /users
        assert!(!tool_names.contains(&"createPet")); // POST /pets
        assert!(!tool_names.contains(&"patchPet")); // PATCH /pets
        assert!(!tool_names.contains(&"healthCheck")); // HEAD /health
        assert!(!tool_names.contains(&"healthOptions")); // OPTIONS /health
    }

    #[test]
    fn test_method_filtering_multiple_methods() {
        use reqwest::Method;

        let spec = create_test_spec_with_methods();
        let filter_methods = vec![Method::GET, Method::POST];
        let tools = spec
            .to_tool_metadata(None, Some(&filter_methods), false)
            .expect("Failed to generate tools");

        // Only GET and POST operations should be included
        assert_eq!(tools.len(), 4);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listUsers")); // GET /users
        assert!(tool_names.contains(&"createUser")); // POST /users
        assert!(tool_names.contains(&"listPets")); // GET /pets
        assert!(tool_names.contains(&"createPet")); // POST /pets
        assert!(!tool_names.contains(&"updateUser")); // PUT /users
        assert!(!tool_names.contains(&"deleteUser")); // DELETE /users
        assert!(!tool_names.contains(&"patchPet")); // PATCH /pets
        assert!(!tool_names.contains(&"healthCheck")); // HEAD /health
        assert!(!tool_names.contains(&"healthOptions")); // OPTIONS /health
    }

    #[test]
    fn test_method_filtering_uncommon_methods() {
        use reqwest::Method;

        let spec = create_test_spec_with_methods();
        let filter_methods = vec![Method::HEAD, Method::OPTIONS, Method::PATCH];
        let tools = spec
            .to_tool_metadata(None, Some(&filter_methods), false)
            .expect("Failed to generate tools");

        // Only HEAD, OPTIONS, and PATCH operations should be included
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"patchPet")); // PATCH /pets
        assert!(tool_names.contains(&"healthCheck")); // HEAD /health
        assert!(tool_names.contains(&"healthOptions")); // OPTIONS /health
        assert!(!tool_names.contains(&"listUsers")); // GET /users
        assert!(!tool_names.contains(&"createUser")); // POST /users
        assert!(!tool_names.contains(&"updateUser")); // PUT /users
        assert!(!tool_names.contains(&"deleteUser")); // DELETE /users
        assert!(!tool_names.contains(&"listPets")); // GET /pets
        assert!(!tool_names.contains(&"createPet")); // POST /pets
    }

    #[test]
    fn test_method_and_tag_filtering_combined() {
        use reqwest::Method;

        let spec = create_test_spec_with_methods();
        let filter_tags = vec!["user".to_string()];
        let filter_methods = vec![Method::GET, Method::POST];
        let tools = spec
            .to_tool_metadata(Some(&filter_tags), Some(&filter_methods), false)
            .expect("Failed to generate tools");

        // Only user operations with GET and POST methods should be included
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"listUsers")); // GET /users (has user tag)
        assert!(tool_names.contains(&"createUser")); // POST /users (has user tag)
        assert!(!tool_names.contains(&"updateUser")); // PUT /users (user tag but not GET/POST)
        assert!(!tool_names.contains(&"deleteUser")); // DELETE /users (user tag but not GET/POST)
        assert!(!tool_names.contains(&"listPets")); // GET /pets (GET method but not user tag)
        assert!(!tool_names.contains(&"createPet")); // POST /pets (POST method but not user tag)
        assert!(!tool_names.contains(&"patchPet")); // PATCH /pets (neither user tag nor GET/POST)
        assert!(!tool_names.contains(&"healthCheck")); // HEAD /health (neither user tag nor GET/POST)
        assert!(!tool_names.contains(&"healthOptions")); // OPTIONS /health (neither user tag nor GET/POST)
    }

    #[test]
    fn test_method_filtering_no_matching_methods() {
        use reqwest::Method;

        let spec = create_test_spec_with_methods();
        let filter_methods = vec![Method::TRACE]; // No TRACE operations in the spec
        let tools = spec
            .to_tool_metadata(None, Some(&filter_methods), false)
            .expect("Failed to generate tools");

        // No operations should be included
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn test_method_filtering_empty_filter_list() {
        let spec = create_test_spec_with_methods();
        let filter_methods: Vec<reqwest::Method> = vec![];
        let tools = spec
            .to_tool_metadata(None, Some(&filter_methods), false)
            .expect("Failed to generate tools");

        // Empty filter should exclude all operations
        assert_eq!(tools.len(), 0);
    }
}
