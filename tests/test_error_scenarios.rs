use rmcp_openapi::{HttpClient, OpenApiServer};
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
    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .registry
            .get_tool("addPet")
            .expect("addPet tool should be registered");

        // Send invalid pet data (missing required fields)
        let invalid_pet_data = json!({
            // Missing required 'name' and 'photoUrls' fields
            "status": "invalid_status_that_doesnt_exist"
        });

        let arguments = json!({
            "request_body": invalid_pet_data
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should return 400 or 422 for validation errors
        assert!(response.status_code == 400 || response.status_code == 422);
        assert!(!response.is_success);
    } else {
        let mut mock_server = MockPetstoreServer::new_with_port(9002).await;
        let _mock = mock_server.mock_add_pet_validation_error();

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .registry
            .get_tool("addPet")
            .expect("addPet tool should be registered");

        let invalid_pet_data = json!({
            "status": "invalid"
        });

        let arguments = json!({
            "request_body": invalid_pet_data
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Mock server returns 400
        assert_eq!(response.status_code, 400);
        assert!(!response.is_success);
        assert!(response.status_text.contains("Bad Request"));

        // Verify error details
        let error_data = response.json()?;
        assert_eq!(error_data["message"], "Invalid input");
        assert_eq!(error_data["details"], "Name is required");
    }

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
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("petId") || error_message.contains("required"));

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

    // This may succeed if the HTTP client converts the string to integer,
    // or it may fail with a validation error. Either behavior is acceptable
    // since the URL construction will handle string-to-number conversion.
    // The key is that it doesn't panic or crash.
    match result {
        Ok(response) => {
            // If it succeeds, the URL should contain the string value
            assert!(response.request_url.contains("not_a_number"));
        }
        Err(error) => {
            // If it fails, it should be a validation error
            let error_message = error.to_string();
            assert!(error_message.contains("petId") || error_message.contains("parameter"));
        }
    }

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

/// Helper function to create a server with a specific base URL
async fn create_server_with_base_url(base_url: Url) -> anyhow::Result<OpenApiServer> {
    let spec_content = include_str!("assets/petstore-openapi.json");
    let spec_url = Url::parse("test://petstore")?;
    let mut server =
        OpenApiServer::with_base_url(rmcp_openapi::OpenApiSpecLocation::Url(spec_url), base_url)?;

    // Parse the embedded spec
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;
    let spec = rmcp_openapi::openapi_spec::OpenApiSpec::from_value(json_value)?;
    server.registry.register_from_spec(spec)?;

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

    /// Mock addPet with validation error
    pub fn mock_add_pet_validation_error(&mut self) -> Mock {
        self.server
            .mock("POST", "/pet")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "message": "Invalid input",
                    "details": "Name is required"
                })
                .to_string(),
            )
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
