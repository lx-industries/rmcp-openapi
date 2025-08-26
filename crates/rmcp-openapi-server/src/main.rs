mod cli;
mod config;
mod spec_loader;

use cli::Cli;
use config::Config;
use rmcp::transport::SseServer;
use rmcp_openapi::{Error, Server};
use std::process;
use tokio::signal;
use tracing::{debug, error, info, info_span};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    let config = Config::from_cli(cli)?;

    // Set up structured logging
    setup_logging();

    let span = info_span!(
        "server_initialization",
        bind_address = %config.bind_address,
        port = config.port,
    );
    let _enter = span.enter();

    // Load OpenAPI specification JSON first
    info!(
        spec_location = %config.spec_location,
        "Loading OpenAPI specification"
    );
    let openapi_json = config.spec_location.load_json().await?;

    // Create server using the builder pattern
    let mut server = Server::builder()
        .openapi_spec(openapi_json)
        .maybe_tag_filter(config.tags.clone())
        .maybe_method_filter(config.methods.clone())
        .maybe_base_url(config.base_url.clone())
        .maybe_default_headers(
            (!config.default_headers.is_empty()).then_some(config.default_headers.clone()),
        )
        .build();

    // Parse OpenAPI specification and generate tools
    server.load_openapi_spec()?;
    info!(
        tool_count = server.tool_count(),
        "Successfully loaded tools from OpenAPI specification"
    );

    debug!(
        tools = %server.get_tool_names().join(", "),
        "Available tools"
    );
    debug!(
        stats = %server.get_tool_stats(),
        "Tool statistics"
    );

    // Validate the registry
    server.validate_registry()?;

    let bind_addr = format!("{}:{}", config.bind_address, config.port);
    info!(
        bind_address = %bind_addr,
        "OpenAPI MCP Server starting"
    );

    // Set up MCP server with SSE transport
    let cancellation_token = SseServer::serve(
        bind_addr
            .parse()
            .map_err(|e| Error::InvalidUrl(format!("Invalid bind address: {e}")))?,
    )
    .await
    .map_err(|e| Error::McpError(format!("Failed to start SSE server: {e}")))?
    .with_service(move || {
        // Clone the already loaded server
        server.clone()
    });

    info!(
        connection_url = %format!("http://{bind_addr}/sse"),
        "Server ready for MCP client connections"
    );

    // Wait for shutdown signal
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    info!("Shutdown signal received, stopping server");

    // Cancel the server
    cancellation_token.cancel();
    info!("Server stopped gracefully");

    Ok(())
}

fn setup_logging() {
    // Initialize tracing subscriber for structured logging using RMCP_OPENAPI_LOG
    let env_filter = tracing_subscriber::EnvFilter::try_from_env("RMCP_OPENAPI_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true) // Include the target (module path) in logs
        .init();
}
