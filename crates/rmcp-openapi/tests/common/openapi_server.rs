use rmcp_openapi::{Server, Spec};
use url::Url;

/// Helper to create an OpenAPI server for testing
#[allow(dead_code)]
pub fn create_petstore_server(base_url: Url) -> anyhow::Result<Server> {
    // Using petstore-openapi-norefs.json until issue #18 is implemented
    let spec_content = include_str!("../assets/petstore-openapi-norefs.json");

    // Parse the embedded spec as JSON value
    let json_value: serde_json::Value = serde_json::from_str(spec_content)?;

    let mut server = Server::builder()
        .openapi_spec(json_value)
        .base_url(base_url)
        .build();

    // Load the OpenAPI specification
    server.load_openapi_spec()?;

    Ok(server)
}

/// SSE transport functionality (deprecated)
#[cfg(feature = "transport-sse")]
#[allow(deprecated)]
pub mod sse_server {
    use rmcp::transport::SseServer;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;
    use url::Url;

    /// Start a test MCP server with OpenAPI tools using RMCP + SSE
    #[allow(dead_code)]
    pub async fn start_sse_server_with_petstore(
        bind_addr: &str,
        base_url: Url,
    ) -> anyhow::Result<(Arc<super::Server>, CancellationToken)> {
        let server = Arc::new(super::create_petstore_server(base_url.clone())?);

        let ct = SseServer::serve(bind_addr.parse()?)
            .await?
            .with_service(move || {
                // Using petstore-openapi-norefs.json until issue #18 is implemented
                let spec_content = include_str!("../assets/petstore-openapi-norefs.json");

                // Parse the embedded spec as JSON value
                let json_value: serde_json::Value = serde_json::from_str(spec_content).unwrap();

                let mut server = super::Server::builder()
                    .openapi_spec(json_value)
                    .base_url(base_url.clone())
                    .build();

                // Load the OpenAPI specification
                server.load_openapi_spec().unwrap();

                server
            });

        Ok((server, ct))
    }
}
