use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp_openapi::Server;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

mod common;
use common::mock_server::MockPetstoreServer;
use mockito::Mock;
use serde_json::json;

/// Create a petstore server with base URL for HTTP requests
fn create_petstore_mcp_server_with_spec(base_url: Url, spec_path: &str) -> anyhow::Result<Server> {
    let spec_content = match spec_path {
        "assets/petstore-openapi-norefs.json" => {
            include_str!("assets/petstore-openapi-norefs.json")
        }
        "assets/petstore-openapi.json" => include_str!("assets/petstore-openapi.json"),
        _ => panic!("Unsupported spec path: {spec_path}"),
    };

    // Parse the embedded spec as JSON value and create tools directly
    let json_value: serde_json::Value = serde_json::from_str(spec_content).unwrap();
    let spec = rmcp_openapi::Spec::from_value(json_value)?;

    // Generate OpenApiTool instances directly (synchronously)
    let tools = spec.to_openapi_tools(
        None, // filters
        Some(base_url.clone()),
        None,  // default_headers
        false, // skip_tool_descriptions
        false, // skip_parameter_descriptions
    )?;

    let mut server = Server::builder()
        .openapi_spec(serde_json::Value::Null) // Dummy value since we set tools directly
        .base_url(base_url)
        .build();

    // Set tools directly
    server.tool_collection = rmcp_openapi::ToolCollection::from_tools(tools);

    Ok(server)
}

async fn init() -> anyhow::Result<()> {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init();
    tokio::process::Command::new("npm")
        .arg("install")
        .current_dir("tests/test_with_js")
        .spawn()?
        .wait()
        .await?;

    // Add small delay to ensure Node.js export maps are fully processed
    // This prevents intermittent module resolution failures in CI
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    Ok(())
}

async fn run_js_streamable_http_client_test(
    spec_path: &str,
    mock_port: u16,
    streamable_port: u16,
    snapshot_name: &str,
) -> anyhow::Result<()> {
    init().await?;

    // Start mock server for HTTP requests
    let mut mock_server = MockPetstoreServer::new_with_port(mock_port).await;

    // Set up mock responses for all tool calls
    let _get_pet_mock = mock_server.mock_get_pet_by_id(123);
    let _find_pets_mock = mock_server.mock_find_pets_by_multiple_status();
    let _add_pet_mock = mock_server.mock_add_pet();
    let _error_mock = mock_server.mock_get_pet_by_id_not_found(999999);
    let _validation_error_mock = mock_server.mock_add_pet_validation_error();

    let base_url = mock_server.base_url();
    let spec_path = spec_path.to_string(); // Convert to owned string
    let ct = tokio_util::sync::CancellationToken::new();
    let service = StreamableHttpService::new(
        move || {
            create_petstore_mcp_server_with_spec(base_url.clone(), &spec_path)
                .map_err(std::io::Error::other)
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig {
            stateful_mode: true,
            sse_keep_alive: None,
            cancellation_token: ct.clone(),
        },
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let streamable_bind_address = format!("127.0.0.1:{streamable_port}");
    let tcp_listener = tokio::net::TcpListener::bind(&streamable_bind_address).await?;

    let server_handle = tokio::spawn({
        let ct = ct.clone();
        async move {
            let _ = axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await;
        }
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let mut cmd = tokio::process::Command::new("node");
    cmd.arg("streamable_client.js")
        .current_dir("tests/test_with_js");

    // Set environment variable for Streamable URL if not using default port
    if streamable_port != 8001 {
        cmd.env(
            "MCP_STREAMABLE_URL",
            format!("http://{streamable_bind_address}/mcp/"),
        );
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        eprintln!("StreamableHttp client failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // Capture and validate the actual MCP responses
    let stdout = String::from_utf8(output.stdout)?;
    let mut responses: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()?;

    // Sort arrays for deterministic snapshots (preserve_order handles object properties)
    for response in &mut responses {
        if let Some(tools) = response
            .get_mut("data")
            .and_then(|d| d.get_mut("tools"))
            .and_then(|t| t.as_array_mut())
        {
            tools.sort_by(|a, b| {
                let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                name_a.cmp(name_b)
            });
        }
    }

    insta::assert_json_snapshot!(snapshot_name, responses);

    ct.cancel();
    server_handle.await?;
    Ok(())
}

#[actix_web::test]
async fn test_with_js_streamable_http_client() -> anyhow::Result<()> {
    run_js_streamable_http_client_test(
        "assets/petstore-openapi-norefs.json",
        8085,
        8001,
        "js_streamable_http_client_responses",
    )
    .await
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

    /// Mock findPetsByStatus response for multiple status values
    pub fn mock_find_pets_by_multiple_status(&mut self) -> Mock {
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
                "status": "available"
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
                "status": "pending"
            }
        ]);

        self.server
            .mock("GET", "/pet/findByStatus")
            .match_query(mockito::Matcher::AnyOf(vec![
                // Match multiple status values in query
                mockito::Matcher::Regex(r"status=available.*status=pending".to_string()),
                mockito::Matcher::Regex(r"status=pending.*status=available".to_string()),
                // Fallback to any status query
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
}

// =============================================================================
// Tests using original petstore spec WITH $refs (to test $ref resolution)
// =============================================================================

#[actix_web::test]
async fn test_with_js_streamable_http_client_with_refs() -> anyhow::Result<()> {
    run_js_streamable_http_client_test(
        "assets/petstore-openapi.json",
        8087,
        8003,
        "js_streamable_http_client_responses_with_refs",
    )
    .await
}
