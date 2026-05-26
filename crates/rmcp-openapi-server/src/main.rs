mod cli;
mod configuration;
mod spec_loader;

use std::{process, sync::Arc};

use actix_web::{App, HttpServer, web};
use cli::Cli;
use configuration::Configuration;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp_actix_web::transport::StreamableHttpService;
use rmcp_openapi::Error;
use tracing::{debug, error, info, info_span, warn};

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

    // Surface the TLS bypass before any outbound request can fire.
    log_insecure_warning(config.insecure);

    // Extract values needed after server creation
    let bind_address = config.bind_address.clone();
    let port = config.port;
    let stateful = config.stateful;

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

    // Log the authorization mode on startup
    info!(
        authorization_mode = ?server.authorization_mode(),
        "Authorization mode configured"
    );

    // Log security warning if in passthrough mode
    #[cfg(feature = "authorization-token-passthrough")]
    match server.authorization_mode() {
        rmcp_openapi::AuthorizationMode::PassthroughWarn => {
            tracing::warn!(
                "⚠️  Authorization header passthrough is enabled with warnings. \
                This violates MCP specification but may be necessary for proxy scenarios."
            );
        }
        rmcp_openapi::AuthorizationMode::PassthroughSilent => {
            info!(
                "Authorization header passthrough is enabled (silent mode). \
                Headers will be forwarded without per-request warnings."
            );
        }
        _ => {}
    }

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
        .stateful_mode(stateful)
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

/// Emit a single `WARN` log line when TLS verification has been disabled
/// via `--insecure` / `RMCP_INSECURE`, so operators see the bypass at
/// startup. No-op when `insecure` is false.
fn log_insecure_warning(insecure: bool) {
    if insecure {
        warn!(
            "⚠️  TLS certificate verification is DISABLED (--insecure / RMCP_INSECURE). \
             All outbound HTTPS requests will accept invalid, self-signed, or \
             hostname-mismatched certificates. DO NOT USE IN PRODUCTION."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::log_insecure_warning;
    use std::io;
    use std::sync::{Arc, Mutex};
    use tracing::subscriber;
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for CapturedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for CapturedWriter {
        type Writer = CapturedWriter;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn captured_warn(insecure: bool) -> String {
        let writer = CapturedWriter::default();
        let buffer = writer.0.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::WARN)
            .with_ansi(false)
            .without_time()
            .finish();

        subscriber::with_default(subscriber, || log_insecure_warning(insecure));

        String::from_utf8(buffer.lock().unwrap().clone()).unwrap()
    }

    #[test]
    fn log_insecure_warning_emits_warn_when_enabled() {
        let output = captured_warn(true);
        assert!(
            output.contains("WARN"),
            "expected WARN level, got: {output}"
        );
        assert!(
            output.contains("TLS certificate verification is DISABLED"),
            "expected disabled-TLS message, got: {output}"
        );
        assert!(
            output.contains("--insecure"),
            "expected --insecure mention, got: {output}"
        );
        assert_eq!(
            output.matches("WARN").count(),
            1,
            "expected exactly one WARN line, got: {output}"
        );
    }

    #[test]
    fn log_insecure_warning_silent_when_disabled() {
        let output = captured_warn(false);
        assert!(output.is_empty(), "expected no output, got: {output}");
    }
}
