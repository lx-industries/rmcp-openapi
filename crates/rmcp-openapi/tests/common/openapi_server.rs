use rmcp::transport::SseServer;
use rmcp_openapi::{Server, Spec, SpecLocation};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use url::Url;

/// Helper to create an OpenAPI server for testing
#[allow(dead_code)]
pub async fn create_petstore_server(base_url: Option<Url>) -> anyhow::Result<Server> {
    // Using petstore-openapi-norefs.json until issue #18 is implemented
    let spec_content = include_str!("../assets/petstore-openapi-norefs.json");

    // Parse the embedded spec as JSON value
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;
    
    let mut server = if let Some(url) = base_url {
        Server::with_base_url(SpecLocation::Json(json_value.clone()), url)?
    } else {
        Server::new(SpecLocation::Json(json_value))
    };

    // Load the OpenAPI specification using the new API
    server.load_openapi_spec().await?;

    Ok(server)
}

/// Start a test MCP server with OpenAPI tools using RMCP + SSE
#[allow(dead_code)]
pub async fn start_sse_server_with_petstore(
    bind_addr: &str,
    base_url: Option<Url>,
) -> anyhow::Result<(Arc<Server>, CancellationToken)> {
    let server = Arc::new(create_petstore_server(base_url.clone()).await?);

    let ct = SseServer::serve(bind_addr.parse()?)
        .await?
        .with_service(move || {
            // Using petstore-openapi-norefs.json until issue #18 is implemented
            let spec_content = include_str!("../assets/petstore-openapi-norefs.json");
            
            // Parse the embedded spec as JSON value
            let json_value: serde_json::Value = serde_json::from_str(spec_content).unwrap();
            
            let mut server = if let Some(ref url) = base_url {
                Server::with_base_url(SpecLocation::Json(json_value.clone()), url.clone()).unwrap()
            } else {
                Server::new(SpecLocation::Json(json_value))
            };

            // Load the OpenAPI specification using the new API
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    server.load_openapi_spec().await.unwrap();
                })
            });

            server
        });

    Ok((server, ct))
}
