use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{SseServer, StreamableHttpServerConfig, StreamableHttpService};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod common;
// Calculator removed - using OpenApiServer instead

const SSE_BIND_ADDRESS: &str = "127.0.0.1:8000";
const STREAMABLE_HTTP_BIND_ADDRESS: &str = "127.0.0.1:8001";

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
    Ok(())
}

#[tokio::test]
async fn test_with_js_sse_client() -> anyhow::Result<()> {
    init().await?;

    let ct = SseServer::serve(SSE_BIND_ADDRESS.parse()?)
        .await?
        .with_service(common::create_petstore_mcp_server_sync);

    let output = tokio::process::Command::new("node")
        .arg("client.js")
        .current_dir("tests/test_with_js")
        .output()
        .await?;
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
        if let Some(tools) = response.get_mut("tools").and_then(|t| t.as_array_mut()) {
            tools.sort_by(|a, b| {
                let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                name_a.cmp(name_b)
            });
        }
    }

    insta::assert_json_snapshot!("js_sse_client_responses", responses);
    ct.cancel();
    Ok(())
}

#[tokio::test]
async fn test_with_js_streamable_http_client() -> anyhow::Result<()> {
    init().await?;

    let service = StreamableHttpService::new(
        || Ok(common::create_petstore_mcp_server_sync()),
        std::sync::Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig {
            stateful_mode: true,
            sse_keep_alive: None,
        },
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(STREAMABLE_HTTP_BIND_ADDRESS).await?;
    let ct = tokio_util::sync::CancellationToken::new();

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

    let output = tokio::process::Command::new("node")
        .arg("streamable_client.js")
        .current_dir("tests/test_with_js")
        .output()
        .await?;

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
        if let Some(tools) = response.get_mut("tools").and_then(|t| t.as_array_mut()) {
            tools.sort_by(|a, b| {
                let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                name_a.cmp(name_b)
            });
        }
    }

    insta::assert_json_snapshot!("js_streamable_http_client_responses", responses);

    ct.cancel();
    server_handle.await?;
    Ok(())
}
