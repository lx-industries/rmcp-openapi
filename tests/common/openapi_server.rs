use rmcp::transport::SseServer;
use rmcp_openapi::{OpenApiServer, OpenApiSpec};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Helper to create an OpenAPI server for testing
#[allow(dead_code)]
pub async fn create_petstore_server(base_url: Option<String>) -> anyhow::Result<OpenApiServer> {
    let spec_content = include_str!("../assets/petstore-openapi.json");

    let server = if let Some(url) = base_url {
        OpenApiServer::with_base_url("test://petstore".to_string(), url)
    } else {
        OpenApiServer::new("test://petstore".to_string())
    };

    // Parse and register the spec
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;
    let spec = OpenApiSpec::from_value(json_value)?;
    let mut server = server;
    server.registry.register_from_spec(spec)?;

    Ok(server)
}

/// Start a test MCP server with OpenAPI tools using RMCP + SSE
#[allow(dead_code)]
pub async fn start_sse_server_with_petstore(
    bind_addr: &str,
    base_url: Option<String>,
) -> anyhow::Result<(Arc<OpenApiServer>, CancellationToken)> {
    let server = Arc::new(create_petstore_server(base_url.clone()).await?);

    let ct = SseServer::serve(bind_addr.parse()?)
        .await?
        .with_service(move || {
            let spec_content = include_str!("../assets/petstore-openapi.json");
            let mut server = if let Some(ref url) = base_url {
                OpenApiServer::with_base_url("test://petstore".to_string(), url.clone())
            } else {
                OpenApiServer::new("test://petstore".to_string())
            };

            // Parse and register the spec
            let json_value: serde_json::Value = serde_json::from_str(spec_content).unwrap();
            let spec = OpenApiSpec::from_value(json_value).unwrap();
            server.registry.register_from_spec(spec).unwrap();

            server
        });

    Ok((server, ct))
}
