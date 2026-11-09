//! Authorization passthrough through the actix-web streamable HTTP transport.
//!
//! These tests serve [`Server`] with `rmcp_actix_web::transport::StreamableHttpService`,
//! send a `tools/call` request that carries an `Authorization` header, and check which
//! `Authorization` header the upstream API receives.

#![cfg(feature = "authorization-token-passthrough")]

use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, HttpServer, web};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp_actix_web::transport::StreamableHttpService;
use rmcp_openapi::{AuthorizationMode, Server};
use serde_json::{Value, json};
use url::Url;

/// Protocol version the test client negotiates.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Token the MCP client sends to the MCP server.
const CLIENT_TOKEN: &str = "Bearer passthrough-token";

/// OpenAPI document with one operation, `getPing` (`GET /ping`).
fn ping_spec() -> Value {
    json!({
        "openapi": "3.0.3",
        "info": { "title": "Ping API", "version": "1.0.0" },
        "paths": {
            "/ping": {
                "get": {
                    "operationId": "getPing",
                    "responses": {
                        "200": {
                            "description": "Pong",
                            "content": {
                                "application/json": {
                                    "schema": { "type": "object" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

/// Posts one JSON-RPC message to the MCP endpoint and returns the response body.
async fn post_json_rpc(
    client: &reqwest::Client,
    url: &str,
    message: Value,
) -> anyhow::Result<String> {
    let response = client
        .post(url)
        .header(AUTHORIZATION, CLIENT_TOKEN)
        .header(ACCEPT, "application/json, text/event-stream")
        .header(CONTENT_TYPE, "application/json")
        .header("MCP-Protocol-Version", PROTOCOL_VERSION)
        .json(&message)
        .send()
        .await?;
    anyhow::ensure!(
        response.status().is_success(),
        "MCP endpoint answered {}",
        response.status()
    );
    Ok(response.text().await?)
}

/// Serves `server` over the actix-web transport and calls `getPing` once.
async fn call_get_ping(server: Server) -> anyhow::Result<String> {
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
    let url = format!("http://{address}/mcp");
    let outcome: anyhow::Result<String> = async {
        post_json_rpc(
            &client,
            &url,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "passthrough-test", "version": "1.0.0" }
                }
            }),
        )
        .await?;
        post_json_rpc(
            &client,
            &url,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": { "name": "getPing", "arguments": {} }
            }),
        )
        .await
    }
    .await;

    drop(client);
    handle.stop(true).await;
    server_task.await??;
    outcome
}

/// Builds a [`Server`] for [`ping_spec`] that targets `base_url` in `mode`.
fn ping_server(base_url: Url, mode: AuthorizationMode) -> anyhow::Result<Server> {
    let mut server = Server::builder()
        .openapi_spec(ping_spec())
        .base_url(base_url)
        .authorization_mode(mode)
        .build();
    server.load_openapi_spec()?;
    Ok(server)
}

/// Calls `getPing` in `mode` and asserts the `Authorization` header the upstream API receives.
async fn assert_upstream_authorization(
    mode: AuthorizationMode,
    expected_authorization: mockito::Matcher,
) -> anyhow::Result<()> {
    let mut upstream = mockito::Server::new_async().await;
    let ping = upstream
        .mock("GET", "/ping")
        .match_header(AUTHORIZATION, expected_authorization)
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("{}")
        .expect(1)
        .create_async()
        .await;
    let base_url = Url::parse(&upstream.url())?;

    let body = call_get_ping(ping_server(base_url, mode)?).await?;

    assert!(body.contains("\"result\""), "tools/call failed: {body}");
    ping.assert_async().await;
    Ok(())
}

#[actix_web::test]
async fn passthrough_silent_forwards_the_client_token_upstream() -> anyhow::Result<()> {
    assert_upstream_authorization(AuthorizationMode::PassthroughSilent, CLIENT_TOKEN.into()).await
}

#[actix_web::test]
async fn compliant_mode_forwards_no_token_upstream() -> anyhow::Result<()> {
    assert_upstream_authorization(AuthorizationMode::Compliant, mockito::Matcher::Missing).await
}
