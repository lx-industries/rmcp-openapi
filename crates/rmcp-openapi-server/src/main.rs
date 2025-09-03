mod cli;
mod configuration;
mod spec_loader;

use std::{process, sync::Arc};

use actix_web::{App, HttpServer, web};
use cli::Cli;
use configuration::Configuration;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp_actix_web::StreamableHttpService;
use rmcp_openapi::Error;
use tracing::{debug, error, info, info_span};

#[actix_web::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    let config = Configuration::from_cli(cli)?;

    // Set up structured logging
    setup_logging();

    // Extract values needed after server creation
    let bind_address = config.bind_address.clone();
    let port = config.port;

    let span = info_span!(
        "server_initialization",
        bind_address = %bind_address,
        port = port,
    );
    let _enter = span.enter();

    // Create server from configuration by loading OpenAPI spec
    let mut server = config.try_into_server().await?;

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

    let bind_addr = format!("{}:{}", bind_address, port);
    info!(
        bind_address = %bind_addr,
        "OpenAPI MCP Server starting"
    );

    let service = StreamableHttpService::builder()
        .service_factory(Arc::new(move || Ok(server.clone())))
        .session_manager(LocalSessionManager::default().into())
        .stateful_mode(false)
        .build();

    let http_server = HttpServer::new(move || {
        App::new()
            // Mount MCP services at custom paths
            .service(web::scope("/mcp").service(service.clone().scope()))
    })
    .bind(bind_addr.clone())?
    .run();

    info!(
        connection_url = %format!("http://{bind_addr}/mcp"),
        "Server ready for MCP client connections"
    );

    http_server.await?;

    info!("Shutdown signal received, stopping server");

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
