//! Protocol version negotiation through the actix-web streamable HTTP transport.
//!
//! These tests serve [`Server`] with `rmcp_actix_web::transport::StreamableHttpService`
//! in stateless mode, send `initialize` with a given `protocolVersion`, and check the
//! `protocolVersion` the server answers.

use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, HttpServer, web};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp_actix_web::transport::StreamableHttpService;
use rmcp_openapi::Server;
use serde_json::{Value, json};
use url::Url;

/// Newest protocol version with an `initialize` handshake that `rmcp` 3.9.0 knows.
const NEWEST_VERSION_WITH_INITIALIZE: &str = "2025-11-25";

/// Builds a [`Server`] with no tools; negotiation does not depend on the tool set.
fn empty_server() -> anyhow::Result<Server> {
    let mut server = Server::builder()
        .openapi_spec(json!({
            "openapi": "3.0.3",
            "info": { "title": "Empty API", "version": "1.0.0" },
            "paths": {}
        }))
        .base_url(Url::parse("http://127.0.0.1:1")?)
        .build();
    server.load_openapi_spec()?;
    Ok(server)
}

/// Returns the JSON-RPC message carried by a JSON body or by the first non-empty SSE `data` line.
fn json_rpc_message(body: &str) -> anyhow::Result<Value> {
    let payload = body
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .find(|data| !data.is_empty())
        .unwrap_or(body);
    Ok(serde_json::from_str(payload)?)
}

/// Sends `initialize` with `requested` and returns the `protocolVersion` the server answers.
async fn negotiated_protocol_version(requested: &str) -> anyhow::Result<String> {
    let server = empty_server()?;
    let service = StreamableHttpService::builder()
        .service_factory(Arc::new(move || Ok(server.clone())))
        .session_manager(Arc::new(LocalSessionManager::default()))
        .stateful_mode(false)
        .build();
    let http_server = HttpServer::new(move || {
        App::new().service(web::scope("/mcp").service(service.clone().scope()))
    })
    .workers(1)
    .bind("127.0.0.1:0")?;
    let address = *http_server
        .addrs()
        .first()
        .expect("a bound server has at least one address");
    let running = http_server.run();
    let handle = running.handle();
    let server_task = tokio::spawn(running);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let outcome: anyhow::Result<String> = async {
        let response = client
            .post(format!("http://{address}/mcp"))
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": requested,
                    "capabilities": {},
                    "clientInfo": { "name": "protocol-version-test", "version": "1.0.0" }
                }
            }))
            .send()
            .await?;
        anyhow::ensure!(
            response.status().is_success(),
            "MCP endpoint answered {}",
            response.status()
        );
        let message = json_rpc_message(&response.text().await?)?;
        message["result"]["protocolVersion"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow::anyhow!("initialize returned no protocolVersion: {message}"))
    }
    .await;

    drop(client);
    handle.stop(true).await;
    server_task.await??;
    outcome
}

#[actix_web::test]
async fn unknown_version_falls_back_to_newest_version_with_initialize() -> anyhow::Result<()> {
    assert_eq!(
        negotiated_protocol_version("1999-01-01").await?,
        NEWEST_VERSION_WITH_INITIALIZE
    );
    Ok(())
}

#[actix_web::test]
async fn initialize_less_version_falls_back_to_newest_version_with_initialize() -> anyhow::Result<()>
{
    assert_eq!(
        negotiated_protocol_version("2026-07-28").await?,
        NEWEST_VERSION_WITH_INITIALIZE
    );
    Ok(())
}

#[actix_web::test]
async fn known_version_with_initialize_is_kept() -> anyhow::Result<()> {
    assert_eq!(
        negotiated_protocol_version("2025-06-18").await?,
        "2025-06-18"
    );
    Ok(())
}
