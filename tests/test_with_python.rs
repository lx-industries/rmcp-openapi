use rmcp::transport::SseServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod common;

async fn init() -> anyhow::Result<()> {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init();
    tokio::process::Command::new("uv")
        .args(["sync"])
        .current_dir("tests/test_with_python")
        .spawn()?
        .wait()
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_with_python_client() -> anyhow::Result<()> {
    init().await?;

    const BIND_ADDRESS: &str = "127.0.0.1:8000";

    let ct = SseServer::serve(BIND_ADDRESS.parse()?)
        .await?
        .with_service(common::create_petstore_mcp_server_sync);

    let output = tokio::process::Command::new("uv")
        .arg("run")
        .arg("client.py")
        .arg(format!("http://{BIND_ADDRESS}/sse"))
        .current_dir("tests/test_with_python")
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

    insta::assert_json_snapshot!("python_sse_client_responses", responses);
    ct.cancel();
    Ok(())
}

// TODO: Add test_nested_with_python_client once nested routing support is implemented
// See https://gitlab.com/lx-industries/rmcp-actix-web/-/issues/2
