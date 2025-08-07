mod cli;
mod config;

use cli::Cli;
use config::Config;
use rmcp::transport::SseServer;
use rmcp_openapi::{OpenApiError, OpenApiServer};
use std::process;
use tokio::signal;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

async fn run() -> Result<(), OpenApiError> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    let config = Config::from_cli(cli)?;

    // Set up logging if verbose mode is enabled
    if config.verbose {
        setup_logging();
    }

    // Create OpenApi server
    let mut server = match (config.base_url.as_ref(), !config.default_headers.is_empty()) {
        (Some(base_url), true) => {
            // Both base URL and headers
            OpenApiServer::with_base_url_and_headers(
                config.spec_location.clone(),
                base_url.clone(),
                config.default_headers.clone(),
            )?
        }
        (Some(base_url), false) => {
            // Only base URL
            OpenApiServer::with_base_url(config.spec_location.clone(), base_url.clone())?
        }
        (None, true) => {
            // Only headers
            OpenApiServer::with_default_headers(
                config.spec_location.clone(),
                config.default_headers.clone(),
            )
        }
        (None, false) => {
            // Neither
            OpenApiServer::new(config.spec_location.clone())
        }
    }
    .with_tags(config.tags.clone());

    // Load OpenAPI specification
    eprintln!(
        "Loading OpenAPI specification from: {}",
        config.spec_location
    );
    server.load_openapi_spec().await?;
    eprintln!("Successfully loaded {} tools", server.tool_count());

    if config.verbose {
        eprintln!("Available tools: {}", server.get_tool_names().join(", "));
        eprintln!("Registry stats: {}", server.get_registry_stats().summary());
    }

    // Validate the registry
    server.validate_registry()?;

    let bind_addr = format!("{}:{}", config.bind_address, config.port);
    eprintln!("OpenAPI MCP Server starting on http://{bind_addr}");

    // Set up MCP server with SSE transport
    let cancellation_token = SseServer::serve(
        bind_addr
            .parse()
            .map_err(|e| OpenApiError::InvalidUrl(format!("Invalid bind address: {e}")))?,
    )
    .await
    .map_err(|e| OpenApiError::McpError(format!("Failed to start SSE server: {e}")))?
    .with_service(move || {
        // Clone the already loaded server
        server.clone()
    });

    eprintln!("Server ready! Connect MCP clients to: http://{bind_addr}/sse");

    // Wait for shutdown signal
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    eprintln!("\nShutdown signal received, stopping server...");

    // Cancel the server
    cancellation_token.cancel();
    eprintln!("Server stopped gracefully.");

    Ok(())
}

fn setup_logging() {
    use std::env;

    // Set log level if not already set
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }

    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
