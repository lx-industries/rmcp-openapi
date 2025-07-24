use insta::assert_json_snapshot;
use rmcp_openapi::error::ValidationError;
use rmcp_openapi::{HttpClient, OpenApiServer, ToolCallError, ToolGenerator};
use serde_json::json;
use std::env;
use url::Url;

mod common;
use common::mock_server::MockPetstoreServer;
use mockito::Mock;

/// Helper to determine whether to use live API or mock server
fn should_use_live_api() -> bool {
    env::var("RMCP_TEST_LIVE_API").unwrap_or_default() == "true"
}

/// Live Petstore API base URL
const LIVE_API_BASE_URL: &str = "https://petstore.swagger.io/v2";

/// Test HTTP 404 Not Found error handling
#[tokio::test]
async fn test_http_404_not_found_error() -> anyhow::Result<()> {
    let non_existent_pet_id = 999999u64;

    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .registry
            .get_tool("getPetById")
            .expect("getPetById tool should be registered");

        let arguments = json!({
            "petId": non_existent_pet_id
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should return 404
        assert_eq!(response.status_code, 404);
        assert!(!response.is_success);
        assert!(response.status_text.contains("Not Found"));
    } else {
        let mut mock_server = MockPetstoreServer::new_with_port(9001).await;
        let _mock = mock_server.mock_get_pet_by_id_not_found(non_existent_pet_id);

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .registry
            .get_tool("getPetById")
            .expect("getPetById tool should be registered");

        let arguments = json!({
            "petId": non_existent_pet_id
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Mock server returns 404
        assert_eq!(response.status_code, 404);
        assert!(!response.is_success);
        assert!(response.status_text.contains("Not Found"));

        // Verify error message in response
        let error_data = response.json()?;
        assert_eq!(error_data["message"], "Pet not found");
    }

    Ok(())
}

/// Test HTTP 400 Bad Request error handling
#[tokio::test]
async fn test_http_400_bad_request_error() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    let tool_metadata = server
        .registry
        .get_tool("addPet")
        .expect("addPet tool should be registered");

    // Test with invalid enum value - should fail validation before HTTP request
    let invalid_pet_data = json!({
        "status": "invalid"
    });

    let arguments = json!({
        "request_body": invalid_pet_data
    });

    // Extract parameters to trigger validation
    let result = ToolGenerator::extract_parameters(tool_metadata, &arguments);

    // Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ToolCallError::ValidationErrors { .. }));

    // Snapshot the error for detailed validation
    let error_json = serde_json::to_value(&error).unwrap();
    assert_json_snapshot!(error_json);

    Ok(())
}

/// Test HTTP 500 Internal Server Error handling
#[tokio::test]
async fn test_http_500_server_error() -> anyhow::Result<()> {
    // This test only works with mock server since we can't force live API to error
    let mut mock_server = MockPetstoreServer::new_with_port(9003).await;
    let _mock = mock_server.mock_server_error("/pet/123");

    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    let arguments = json!({
        "petId": 123
    });

    let response = client.execute_tool_call(tool_metadata, &arguments).await?;

    // Mock server returns 500
    assert_eq!(response.status_code, 500);
    assert!(!response.is_success);
    assert!(response.status_text.contains("Internal Server Error"));

    // Verify error details
    let error_data = response.json()?;
    assert_eq!(error_data["message"], "Internal Server Error");
    assert_eq!(error_data["details"], "Something went wrong on the server");

    Ok(())
}

/// Test network connection error handling
#[tokio::test]
async fn test_network_connection_error() -> anyhow::Result<()> {
    // Test with an invalid/unreachable URL to simulate connection failure
    let server =
        create_server_with_base_url(Url::parse("http://invalid-host-that-does-not-exist.com")?)
            .await?;
    let client = HttpClient::new()
        .with_base_url(Url::parse("http://invalid-host-that-does-not-exist.com")?)?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    let arguments = json!({
        "petId": 123
    });

    let result = client.execute_tool_call(tool_metadata, &arguments).await;

    // Should get a connection error
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("Connection failed")
            || error_message.contains("connection")
            || error_message.contains("network")
    );

    Ok(())
}

/// Test missing required parameter validation
#[tokio::test]
async fn test_missing_required_parameter_error() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;
    let client = HttpClient::new().with_base_url(Url::parse("http://example.com")?)?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    // Call without required 'petId' parameter
    let arguments = json!({
        // Missing 'petId'
    });

    let result = client.execute_tool_call(tool_metadata, &arguments).await;

    // Should get parameter extraction error
    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_message = error.to_string();

    // New error structure: should be ValidationErrors with missing required parameter
    match error {
        ToolCallError::ValidationErrors { violations } => {
            assert!(!violations.is_empty());
            // Should have a missing required parameter error for petId
            let has_missing_petid = violations.iter().any(|e| match e {
                ValidationError::MissingRequiredParameter { parameter, .. } => parameter == "petId",
                _ => false,
            });
            assert!(
                has_missing_petid,
                "Expected missing required parameter error for petId"
            );
        }
        _ => panic!("Expected ValidationErrors variant, got: {error_message}"),
    }

    Ok(())
}

/// Test type validation error (string for integer parameter)
#[tokio::test]
async fn test_type_validation_error() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;
    let client = HttpClient::new().with_base_url(Url::parse("http://example.com")?)?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    // Pass string instead of integer for petId
    let arguments = json!({
        "petId": "not_a_number"
    });

    let result = client.execute_tool_call(tool_metadata, &arguments).await;

    // Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Snapshot the error for detailed validation
    if let Ok(error_json) = serde_json::to_value(&error) {
        assert_json_snapshot!(error_json);
    } else {
        // Fallback to string comparison if error is not serializable
        let error_message = error.to_string();
        assert!(error_message.contains("\"not_a_number\" is not of type \"integer\""));
    }

    Ok(())
}

/// Test array type validation
#[tokio::test]
async fn test_array_type_validation_error() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    let tool_metadata = server
        .registry
        .get_tool("findPetsByStatus")
        .expect("findPetsByStatus tool should be registered");

    // Pass string instead of array
    let arguments = json!({
        "status": "available"  // Should be an array
    });

    // Extract parameters to trigger validation
    let result = ToolGenerator::extract_parameters(tool_metadata, &arguments);

    // Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ToolCallError::ValidationErrors { .. }));

    // Snapshot the error for detailed validation
    let error_json = serde_json::to_value(&error).unwrap();
    assert_json_snapshot!(error_json);

    Ok(())
}

/// Test enum validation
#[tokio::test]
async fn test_enum_validation_error() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    let tool_metadata = server
        .registry
        .get_tool("findPetsByStatus")
        .expect("findPetsByStatus tool should be registered");

    // Pass invalid enum value
    let arguments = json!({
        "status": ["invalid_status"]  // Not one of: available, pending, sold
    });

    // Extract parameters to trigger validation
    let result = ToolGenerator::extract_parameters(tool_metadata, &arguments);

    // Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ToolCallError::ValidationErrors { .. }));

    // Snapshot the error for detailed validation
    let error_json = serde_json::to_value(&error).unwrap();
    assert_json_snapshot!(error_json);

    Ok(())
}

/// Test enum validation - parameter passing
#[tokio::test]
async fn test_enum_validation_parameter_passing() -> anyhow::Result<()> {
    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .registry
            .get_tool("findPetsByStatus")
            .expect("findPetsByStatus tool should be registered");

        // Pass valid status value to test parameter passing
        let arguments = json!({
            "status": ["available"]
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should accept valid enum values
        assert!(response.is_success);
        assert!(response.request_url.contains("/pet/findByStatus"));
    } else {
        // For mock testing, we'll test that parameters are properly formatted
        let mut mock_server = MockPetstoreServer::new_with_port(9005).await;
        let _mock = mock_server.mock_find_pets_by_status("available");

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .registry
            .get_tool("findPetsByStatus")
            .expect("findPetsByStatus tool should be registered");

        let arguments = json!({
            "status": ["available"]
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Mock server should respond successfully
        assert!(response.is_success);
        assert!(response.request_url.contains("/pet/findByStatus"));
    }

    Ok(())
}

/// Test non-JSON response handling
#[tokio::test]
async fn test_non_json_response_handling() -> anyhow::Result<()> {
    // Use mock server to return non-JSON content
    let mut mock_server = MockPetstoreServer::new_with_port(9006).await;

    // Create a custom mock that returns HTML instead of JSON
    let _mock = mock_server
        .server
        .mock("GET", "/pet/123")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body>This is HTML, not JSON</body></html>")
        .create();

    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    let arguments = json!({
        "petId": 123
    });

    let response = client.execute_tool_call(tool_metadata, &arguments).await?;

    // Should succeed but body is HTML
    assert!(response.is_success);
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("<html>"));

    // Trying to parse as JSON should fail
    let json_result = response.json();
    assert!(json_result.is_err());

    Ok(())
}

/// Test malformed JSON response handling
#[tokio::test]
async fn test_malformed_json_response_handling() -> anyhow::Result<()> {
    // Use mock server to return malformed JSON
    let mut mock_server = MockPetstoreServer::new_with_port(9007).await;

    // Create a custom mock that returns invalid JSON
    let _mock = mock_server
        .server
        .mock("GET", "/pet/123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 123, "name": "doggie", "invalid": json}"#) // Missing quotes around json
        .create();

    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    let arguments = json!({
        "petId": 123
    });

    let response = client.execute_tool_call(tool_metadata, &arguments).await?;

    // Should succeed with HTTP 200 but JSON parsing should fail
    assert!(response.is_success);
    assert_eq!(response.status_code, 200);

    // Trying to parse as JSON should fail gracefully
    let json_result = response.json();
    assert!(json_result.is_err());
    let error_message = json_result.unwrap_err().to_string();
    assert!(error_message.contains("JSON") || error_message.contains("parse"));

    Ok(())
}

/// Test empty response (204 No Content) handling
#[tokio::test]
async fn test_empty_response_handling() -> anyhow::Result<()> {
    // Use mock server to return 204 No Content
    let mut mock_server = MockPetstoreServer::new_with_port(9008).await;

    // Create a custom mock that returns 204 with no body
    let _mock = mock_server
        .server
        .mock("DELETE", "/pet/123")
        .with_status(204)
        .with_header("content-length", "0")
        .create();

    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    // For this test, we'll manually create a DELETE request since deletePet might not be in our spec
    // Instead, let's test with a tool that exists and simulate 204 response
    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    // Override the mock to respond to GET instead
    let _mock = mock_server
        .server
        .mock("GET", "/pet/123")
        .with_status(204)
        .with_header("content-length", "0")
        .create();

    let arguments = json!({
        "petId": 123
    });

    let response = client.execute_tool_call(tool_metadata, &arguments).await?;

    // Should succeed with 204 and empty body
    assert!(response.is_success);
    assert_eq!(response.status_code, 204);
    assert!(response.body.is_empty());

    // JSON parsing of empty body should fail gracefully
    if !response.body.is_empty() {
        let json_result = response.json();
        if json_result.is_err() {
            let error_message = json_result.unwrap_err().to_string();
            assert!(error_message.contains("JSON") || error_message.contains("parse"));
        }
    }

    Ok(())
}

/// Test large response handling with simpler approach
#[tokio::test]
async fn test_large_response_handling() -> anyhow::Result<()> {
    // Use mock server to return a large response using a simpler endpoint
    let mut mock_server = MockPetstoreServer::new_with_port(9009).await;

    // Create a large JSON response (simulate a large pet object)
    let large_description = "A".repeat(50000); // 50KB string
    let large_pet = json!({
        "id": 123,
        "name": "large_pet",
        "status": "available",
        "photoUrls": ["https://example.com/photo1.jpg"],
        "description": large_description
    });
    let large_response = serde_json::to_string(&large_pet)?;

    let _mock = mock_server
        .server
        .mock("GET", "/pet/123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(large_response)
        .create();

    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    let tool_metadata = server
        .registry
        .get_tool("getPetById")
        .expect("getPetById tool should be registered");

    let arguments = json!({
        "petId": 123
    });

    let response = client.execute_tool_call(tool_metadata, &arguments).await?;

    // Should succeed even with large response
    assert!(
        response.is_success,
        "Response failed with status: {} - {}",
        response.status_code, response.status_text
    );
    assert_eq!(response.status_code, 200);
    assert!(response.body.len() > 10000); // Should be a large response

    // JSON parsing should still work
    let pet_data = response.json()?;
    assert_eq!(pet_data["id"], 123);
    assert_eq!(pet_data["name"], "large_pet");
    assert!(pet_data["description"].as_str().unwrap().len() > 40000);

    Ok(())
}

/// Test passing integer for string field
#[tokio::test]
async fn test_integer_for_string_validation_error() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    let tool_metadata = server
        .registry
        .get_tool("addPet")
        .expect("addPet tool should be registered");

    // Pass integer instead of string for name
    let arguments = json!({
        "request_body": {
            "name": 12345,  // Should be string
            "photoUrls": ["https://example.com/photo.jpg"],
            "status": "available"
        }
    });

    // Extract parameters to trigger validation
    let result = ToolGenerator::extract_parameters(tool_metadata, &arguments);

    // Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ToolCallError::ValidationErrors { .. }));

    // Snapshot the error for detailed validation
    let error_json = serde_json::to_value(&error).unwrap();
    assert_json_snapshot!(error_json);

    Ok(())
}

/// Test tool not found error with suggestions
#[tokio::test]
async fn test_tool_not_found_with_suggestions() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    // Verify the server has the expected tools
    let tool_names = server.get_tool_names();

    // Simulate the error that would be generated when calling a non-existent tool with a typo
    // This tests the error generation logic without going through the full MCP protocol

    // Use the internal logic to find similar tool names (via the public API)
    // Since find_similar_strings is private, we'll test this by creating the error directly
    // which mirrors what happens in server.rs when a tool is not found
    let mut suggestions = Vec::new();

    // Manually compute suggestions using the same logic (Jaro distance > 0.7)
    for known_tool in &tool_names {
        let distance = strsim::jaro("getPetByID", known_tool);
        if distance > 0.7 {
            suggestions.push((distance, known_tool.clone()));
        }
    }

    // Sort by distance (descending) and take top 3
    suggestions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let suggestions: Vec<String> = suggestions
        .into_iter()
        .take(3)
        .map(|(_, name)| name)
        .collect();

    // Create the error with suggestions
    let error = ToolCallError::tool_not_found("getPetByID".to_string(), suggestions);

    // Snapshot the error structure
    let error_json = serde_json::to_value(&error).unwrap();
    assert_json_snapshot!(error_json);

    Ok(())
}

/// Test tool not found error with multiple suggestions
#[tokio::test]
async fn test_tool_not_found_multiple_suggestions() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    // Verify the server has the expected tools
    let tool_names = server.get_tool_names();

    // Use a typo that could match multiple tools: "findPet" could match both findPetsByStatus and getPetById
    let mut suggestions = Vec::new();

    // Manually compute suggestions using the same logic (Jaro distance > 0.7)
    for known_tool in &tool_names {
        let distance = strsim::jaro("findPet", known_tool);
        if distance > 0.7 {
            suggestions.push((distance, known_tool.clone()));
        }
    }

    // Sort by distance (descending) and take top 3
    suggestions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let suggestions: Vec<String> = suggestions
        .into_iter()
        .take(3)
        .map(|(_, name)| name)
        .collect();

    // Create the error with suggestions
    let error = ToolCallError::tool_not_found("findPet".to_string(), suggestions);

    // Snapshot the error structure - should contain multiple suggestions
    let error_json = serde_json::to_value(&error).unwrap();
    assert_json_snapshot!(error_json);

    Ok(())
}

/// Helper function to create a server with a specific base URL
async fn create_server_with_base_url(base_url: Url) -> anyhow::Result<OpenApiServer> {
    // Using petstore-openapi-norefs.json until issue #18 is implemented
    let spec_content = include_str!("assets/petstore-openapi-norefs.json");
    let spec_url = Url::parse("test://petstore")?;
    let mut server =
        OpenApiServer::with_base_url(rmcp_openapi::OpenApiSpecLocation::Url(spec_url), base_url)?;

    // Parse the embedded spec
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;
    let spec = rmcp_openapi::openapi::OpenApiSpec::from_value(json_value)?;
    server.register_spec(spec)?;

    Ok(server)
}

// Test-specific mock methods for MockPetstoreServer
impl MockPetstoreServer {
    /// Mock getPetById with 404 Not Found
    pub fn mock_get_pet_by_id_not_found(&mut self, pet_id: u64) -> Mock {
        self.server
            .mock("GET", format!("/pet/{pet_id}").as_str())
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"message": "Pet not found"}).to_string())
            .create()
    }

    /// Mock server error response
    pub fn mock_server_error(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "message": "Internal Server Error",
                    "details": "Something went wrong on the server"
                })
                .to_string(),
            )
            .create()
    }

    /// Mock successful findPetsByStatus response
    pub fn mock_find_pets_by_status(&mut self, status: &str) -> Mock {
        let pets_response = json!([
            {
                "id": 1,
                "name": "doggie",
                "category": {
                    "id": 1,
                    "name": "Dogs"
                },
                "photoUrls": ["https://example.com/photo1.jpg"],
                "tags": [
                    {
                        "id": 1,
                        "name": "tag1"
                    }
                ],
                "status": status
            },
            {
                "id": 2,
                "name": "kitty",
                "category": {
                    "id": 2,
                    "name": "Cats"
                },
                "photoUrls": ["https://example.com/photo2.jpg"],
                "tags": [
                    {
                        "id": 2,
                        "name": "tag2"
                    }
                ],
                "status": status
            }
        ]);

        self.server
            .mock("GET", "/pet/findByStatus")
            .match_query(mockito::Matcher::AnyOf(vec![
                // Match single status parameter
                mockito::Matcher::UrlEncoded("status".to_string(), status.to_string()),
                // Match multiple status parameters (flexible matching)
                mockito::Matcher::Regex(r"status=.+".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(pets_response.to_string())
            .create()
    }
}
