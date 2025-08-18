mod cli;
mod config;

use cli::Cli;
use config::Config;
use rmcp::transport::SseServer;
use rmcp_openapi::{Error, Server};
use std::process;
use tokio::signal;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    let config = Config::from_cli(cli)?;

    // Set up logging if verbose mode is enabled
    if config.verbose {
        setup_logging();
    }

    // Create server using the builder pattern
    let mut server = Server::builder()
        .spec_location(config.spec_location.clone())
        .maybe_tag_filter(config.tags.clone())
        .maybe_method_filter(config.methods.clone())
        .maybe_base_url(config.base_url.clone())
        .maybe_default_headers(
            (!config.default_headers.is_empty()).then_some(config.default_headers.clone()),
        )
        .build();

    // Load OpenAPI specification
    eprintln!(
        "Loading OpenAPI specification from: {}",
        config.spec_location
    );
    server.load_openapi_spec().await?;
    eprintln!("Successfully loaded {} tools", server.tool_count());

    if config.verbose {
        eprintln!("Available tools: {}", server.get_tool_names().join(", "));
        eprintln!("Tool stats: {}", server.get_tool_stats());
    }

    // Validate the registry
    server.validate_registry()?;

    let bind_addr = format!("{}:{}", config.bind_address, config.port);
    eprintln!("OpenAPI MCP Server starting on http://{bind_addr}");

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
