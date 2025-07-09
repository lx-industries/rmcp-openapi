pub mod openapi_server;

use rmcp_openapi::OpenApiServer;

/// Create a test OpenApiServer with Petstore spec
#[allow(dead_code)]
pub async fn create_test_server() -> anyhow::Result<OpenApiServer> {
    let spec_content = include_str!("../assets/petstore-openapi.json");
    let mut server = OpenApiServer::new("test://petstore".to_string());

    // Parse the embedded spec
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;
    let spec = rmcp_openapi::openapi_spec::OpenApiSpec::from_value(json_value)?;
    server.registry.register_from_spec(spec)?;

    Ok(server)
}

/// Create a test OpenApiServer with Petstore spec for MCP tests
#[allow(dead_code)]
pub async fn create_petstore_mcp_server() -> anyhow::Result<OpenApiServer> {
    let spec_content = include_str!("../assets/petstore-openapi.json");
    let mut server = OpenApiServer::new("test://petstore".to_string());

    // Parse the embedded spec
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;
    let spec = rmcp_openapi::openapi_spec::OpenApiSpec::from_value(json_value)?;
    server.registry.register_from_spec(spec)?;

    Ok(server)
}

/// Create a petstore server synchronously for use with MCP service providers
pub fn create_petstore_mcp_server_sync() -> OpenApiServer {
    let spec_content = include_str!("../assets/petstore-openapi.json");
    let mut server = OpenApiServer::new("test://petstore".to_string());

    // Parse the embedded spec
    let json_value: serde_json::Value = serde_json::from_str(spec_content).unwrap();
    let spec = rmcp_openapi::openapi_spec::OpenApiSpec::from_value(json_value).unwrap();
    server.registry.register_from_spec(spec).unwrap();

    server
}

/// Wait for server to be ready
#[allow(dead_code)]
pub async fn wait_for_server_ready(port: u16) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/health");

    for _ in 0..30 {
        if let Ok(response) = client.get(&url).send().await {
            if response.status().is_success() {
                return Ok(());
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Err(anyhow::anyhow!("Server failed to become ready"))
}
