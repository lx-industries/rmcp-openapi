use rmcp_openapi::{HttpClient, Server};
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

/// Test path parameter transformation - getPetById
#[tokio::test]
async fn test_get_pet_by_id_path_parameter() -> anyhow::Result<()> {
    let pet_id = 123u64;

    if should_use_live_api() {
        // Test against live API
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        // Find a tool named "getPetById"
        let tool_metadata = server
            .get_tool_metadata("getPetById")
            .expect("getPetById tool should be registered");

        let arguments = json!({
            "petId": pet_id
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // For live API, we expect either success or 404 (pet not found)
        assert!(response.is_success || response.status_code == 404);
        assert_eq!(response.request_method, "GET");
        assert!(response.request_url.contains(&format!("/pet/{pet_id}")));
    } else {
        // Test against mock server
        let mut mock_server = MockPetstoreServer::new_with_port(9101).await;
        let _mock = mock_server.mock_get_pet_by_id(pet_id);

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        // Find a tool named "getPetById"
        let tool_metadata = server
            .get_tool_metadata("getPetById")
            .expect("getPetById tool should be registered");

        let arguments = json!({
            "petId": pet_id
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Assert successful response
        assert!(response.is_success);
        assert_eq!(response.status_code, 200);
        assert_eq!(response.request_method, "GET");
        assert!(response.request_url.contains(&format!("/pet/{pet_id}")));

        // Verify response contains expected pet data
        let pet_data = response.json()?;
        assert_eq!(pet_data["id"], pet_id);
        assert_eq!(pet_data["name"], "doggie");
    }

    Ok(())
}

/// Test path parameter transformation with not found scenario
#[tokio::test]
async fn test_get_pet_by_id_not_found() -> anyhow::Result<()> {
    let pet_id = 99999u64; // Non-existent pet ID

    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .get_tool_metadata("getPetById")
            .expect("getPetById tool should be registered");

        let arguments = json!({
            "petId": pet_id
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should return 404 for non-existent pet
        assert_eq!(response.status_code, 404);
        assert!(!response.is_success);
    } else {
        let mut mock_server = MockPetstoreServer::new_with_port(9102).await;
        let _mock = mock_server.mock_get_pet_by_id_not_found(pet_id);

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .get_tool_metadata("getPetById")
            .expect("getPetById tool should be registered");

        let arguments = json!({
            "petId": pet_id
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Assert 404 response
        assert_eq!(response.status_code, 404);
        assert!(!response.is_success);
        assert!(response.request_url.contains(&format!("/pet/{pet_id}")));

        // Verify error message
        let error_data = response.json()?;
        assert_eq!(error_data["message"], "Pet not found");
    }

    Ok(())
}

/// Test query parameter transformation - findPetsByStatus
#[tokio::test]
async fn test_find_pets_by_status_query_parameter() -> anyhow::Result<()> {
    let status = "available";

    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .get_tool_metadata("findPetsByStatus")
            .expect("findPetsByStatus tool should be registered");

        let arguments = json!({
            "status": [status]
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should return success
        assert!(response.is_success);
        assert_eq!(response.request_method, "GET");
        assert!(response.request_url.contains("/pet/findByStatus"));
        assert!(response.request_url.contains(&format!("status={status}")));
    } else {
        let mut mock_server = MockPetstoreServer::new_with_port(9103).await;
        let _mock = mock_server.mock_find_pets_by_status(status);

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .get_tool_metadata("findPetsByStatus")
            .expect("findPetsByStatus tool should be registered");

        let arguments = json!({
            "status": [status]
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Assert successful response
        assert!(response.is_success);
        assert_eq!(response.status_code, 200);
        assert_eq!(response.request_method, "GET");
        assert!(response.request_url.contains("/pet/findByStatus"));
        assert!(response.request_url.contains(&format!("status={status}")));

        // Verify the mock was actually called by checking response content
        let pets_data = response.json()?;
        assert!(pets_data.is_array());
        let pets = pets_data.as_array().unwrap();
        assert!(!pets.is_empty(), "Mock should return pets data");

        // Verify all pets have the correct status
        for pet in pets {
            assert_eq!(pet["status"], status);
        }
    }

    Ok(())
}

/// Test query parameter transformation with multiple status values
#[tokio::test]
async fn test_find_pets_by_multiple_status_query_parameter() -> anyhow::Result<()> {
    let statuses = vec!["available", "pending"];

    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .get_tool_metadata("findPetsByStatus")
            .expect("findPetsByStatus tool should be registered");

        let arguments = json!({
            "status": statuses
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should return success
        assert!(response.is_success);
        assert_eq!(response.request_method, "GET");
        assert!(response.request_url.contains("/pet/findByStatus"));

        // URL should contain both status values
        for status in &statuses {
            assert!(response.request_url.contains(&format!("status={status}")));
        }
    } else {
        // For mock server, we'll just test with "available" status
        let mut mock_server = MockPetstoreServer::new_with_port(9104).await;
        let _mock = mock_server.mock_find_pets_by_status("available");

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .get_tool_metadata("findPetsByStatus")
            .expect("findPetsByStatus tool should be registered");

        let arguments = json!({
            "status": ["available"]
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Assert successful response
        assert!(response.is_success);
        assert_eq!(response.status_code, 200);
        assert!(response.request_url.contains("status=available"));
    }

    Ok(())
}

/// Test JSON request body transformation - addPet
#[tokio::test]
async fn test_add_pet_json_request_body() -> anyhow::Result<()> {
    let pet_data = json!({
        "name": "new doggie",
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
        "status": "available"
    });

    if should_use_live_api() {
        let server = create_server_with_base_url(Url::parse(LIVE_API_BASE_URL)?).await?;
        let client = HttpClient::new().with_base_url(Url::parse(LIVE_API_BASE_URL)?)?;

        let tool_metadata = server
            .get_tool_metadata("addPet")
            .expect("addPet tool should be registered");

        let arguments = json!({
            "request_body": pet_data
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Live API should return success (200 or 201)
        assert!(response.is_success);
        assert_eq!(response.request_method, "POST");
        assert!(response.request_url.contains("/pet"));
        assert!(!response.request_body.is_empty());

        // Verify the request body contains the pet data
        let request_body: serde_json::Value = serde_json::from_str(&response.request_body)?;
        assert_eq!(request_body["name"], "new doggie");
        assert_eq!(request_body["status"], "available");
    } else {
        let mut mock_server = MockPetstoreServer::new_with_port(9105).await;
        let _mock = mock_server.mock_add_pet();

        let server = create_server_with_base_url(mock_server.base_url()).await?;
        let client = HttpClient::new().with_base_url(mock_server.base_url())?;

        let tool_metadata = server
            .get_tool_metadata("addPet")
            .expect("addPet tool should be registered");

        let arguments = json!({
            "request_body": pet_data
        });

        let response = client.execute_tool_call(tool_metadata, &arguments).await?;

        // Assert successful response
        assert!(response.is_success);
        assert_eq!(response.status_code, 201);
        assert_eq!(response.request_method, "POST");
        assert!(response.request_url.contains("/pet"));
        assert!(!response.request_body.is_empty());

        // Verify the request body contains the pet data
        let request_body: serde_json::Value = serde_json::from_str(&response.request_body)?;
        assert_eq!(request_body["name"], "new doggie");
        assert_eq!(request_body["status"], "available");

        // Verify response contains the created pet
        let created_pet = response.json()?;
        assert_eq!(created_pet["id"], 123);
        assert_eq!(created_pet["name"], "new doggie");
    }

    Ok(())
}

/// Test available tools listing
#[tokio::test]
async fn test_available_tools() -> anyhow::Result<()> {
    let server = create_server_with_base_url(Url::parse("http://example.com")?).await?;

    let tool_names = server.get_tool_names();
    println!("Available tools: {tool_names:?}");

    // Basic assertion that we have some tools
    assert!(!tool_names.is_empty());
    assert!(tool_names.contains(&"getPetById".to_string()));
    assert!(tool_names.contains(&"addPet".to_string()));

    Ok(())
}

async fn create_server_with_base_url(base_url: Url) -> anyhow::Result<Server> {
    // Using petstore-openapi-norefs.json until issue #18 is implemented
    let spec_content = include_str!("assets/petstore-openapi-norefs.json");

    // Parse the embedded spec as JSON value
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;

    let mut server = Server::with_base_url(rmcp_openapi::SpecLocation::Json(json_value), base_url)?;

    // Load the OpenAPI specification using the new API
    server.load_openapi_spec().await?;

    Ok(server)
}

/// Test URL construction with path parameters
#[tokio::test]
async fn test_url_construction_with_path_parameters() -> anyhow::Result<()> {
    let mock_server = MockPetstoreServer::new_with_port(9106).await;
    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    let tool_metadata = server
        .get_tool_metadata("getPetById")
        .expect("getPetById tool should be registered");

    let arguments = json!({
        "petId": 456
    });

    // This will fail because we haven't set up the mock, but we can check the URL construction
    let result = client.execute_tool_call(tool_metadata, &arguments).await;

    // Even if the request fails, we can verify the URL was constructed correctly
    match result {
        Ok(response) => {
            assert!(response.request_url.contains("/pet/456"));
        }
        Err(e) => {
            // Check that the error message contains the expected URL
            let error_message = e.to_string();
            assert!(error_message.contains("/pet/456"));
        }
    }

    Ok(())
}

/// Test content-type header setting for JSON requests
#[tokio::test]
async fn test_content_type_header_for_json() -> anyhow::Result<()> {
    let mut mock_server = MockPetstoreServer::new_with_port(9108).await;
    let _mock = mock_server.mock_add_pet();

    let server = create_server_with_base_url(mock_server.base_url()).await?;
    let client = HttpClient::new().with_base_url(mock_server.base_url())?;

    let tool_metadata = server
        .get_tool_metadata("addPet")
        .expect("addPet tool should be registered");

    let arguments = json!({
        "request_body": {
            "name": "test pet",
            "status": "available",
            "photoUrls": []
        }
    });

    let response = client.execute_tool_call(tool_metadata, &arguments).await?;

    // The mock server expects application/json content-type
    assert!(response.is_success);
    assert_eq!(response.status_code, 201);

    Ok(())
}

// Test-specific mock methods for MockPetstoreServer
impl MockPetstoreServer {
    /// Mock successful getPetById response
    pub fn mock_get_pet_by_id(&mut self, pet_id: u64) -> Mock {
        let pet_response = json!({
            "id": pet_id,
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
            "status": "available"
        });

        self.server
            .mock("GET", format!("/pet/{pet_id}").as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(pet_response.to_string())
            .create()
    }

    /// Mock getPetById with 404 Not Found
    pub fn mock_get_pet_by_id_not_found(&mut self, pet_id: u64) -> Mock {
        self.server
            .mock("GET", format!("/pet/{pet_id}").as_str())
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"message": "Pet not found"}).to_string())
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

    /// Mock successful addPet response
    pub fn mock_add_pet(&mut self) -> Mock {
        let pet_response = json!({
            "id": 123,
            "name": "new doggie",
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
            "status": "available"
        });

        self.server
            .mock("POST", "/pet")
            .match_header("content-type", "application/json")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(pet_response.to_string())
            .create()
    }
}
