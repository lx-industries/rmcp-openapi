use clap::Parser;
use rmcp_openapi::OpenApiSpecLocation;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "rmcp-openapi-server")]
#[command(about = "OpenAPI MCP Server - Expose OpenAPI endpoints as MCP tools")]
pub struct Cli {
    /// `OpenAPI` specification URL or file path
    pub spec: OpenApiSpecLocation,

    /// Base URL to override the one in the `OpenAPI` spec
    #[arg(long)]
    pub base_url: Option<String>,

    /// Enable verbose logging
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Port to bind the MCP server to
    #[arg(long, short = 'p', default_value = "8080")]
    pub port: u16,

    /// Address to bind the MCP server to
    #[arg(long, default_value = "127.0.0.1")]
    pub bind_address: String,

    /// HTTP headers to add to all requests (format: "name: value")
    #[arg(long = "header", action = clap::ArgAction::Append, help = "HTTP headers to add to all requests in 'name: value' format (can be used multiple times)")]
    pub headers: Vec<String>,

    /// Filter operations by tags (comma-separated)
    #[arg(
        long,
        num_args(1..),
        value_delimiter = ',',
        help = "Only include operations with these tags (comma-separated, normalized to kebab-case)"
    )]
    pub tags: Option<Vec<String>>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
