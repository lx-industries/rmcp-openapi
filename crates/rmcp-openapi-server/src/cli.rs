use crate::spec_loader::SpecLocation;
use clap::Parser;
use rmcp_openapi::AuthorizationMode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "rmcp-openapi-server")]
#[command(about = "OpenAPI MCP Server - Expose OpenAPI endpoints as MCP tools")]
pub struct Cli {
    /// `OpenAPI` specification URL or file path
    pub spec: SpecLocation,

    /// Base URL to override the one in the `OpenAPI` spec
    #[arg(long)]
    pub base_url: String,

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

    /// Filter operations by HTTP methods (comma-separated)
    #[arg(
        long,
        num_args(1..),
        value_delimiter = ',',
        help = "Only include operations with these HTTP methods (comma-separated: GET,POST,PUT,DELETE,PATCH,HEAD,OPTIONS,TRACE)"
    )]
    pub methods: Option<Vec<reqwest::Method>>,

    /// Filter operations by OperationId
    #[arg(
        long,
        num_args(1..),
        value_delimiter = ',',
        conflicts_with = "operationids_exclude",
        help = "Only include operations with these OperationIds (comma-separated)"
    )]
    pub operationids_include: Option<Vec<String>>,

    /// Filter operations by OperationId
    #[arg(
        long,
        num_args(1..),
        value_delimiter = ',',
        conflicts_with = "operationids_include",
        help = "Exclude operations with these OperationIds (comma-separated)"
    )]
    pub operationids_exclude: Option<Vec<String>>,

    /// Authorization mode for handling Authorization headers
    #[arg(
        long,
        env = "RMCP_AUTHORIZATION_MODE",
        default_value = "compliant",
        help = "How to handle Authorization headers: compliant (MCP spec, no passthrough), passthrough-warn (pass with warnings), passthrough-silent (pass silently)"
    )]
    pub authorization_mode: AuthorizationMode,

    #[arg(
        long,
        env = "RMCP_SKIP_TOOL_DESCRIPTIONS",
        default_value_t = false,
        help = "Exclude the tool descriptions from the generated MCP schema"
    )]
    pub skip_tool_descriptions: bool,

    #[arg(
        long,
        env = "RMCP_SKIP_PARAMETER_DESCRIPTIONS",
        default_value_t = false,
        help = "Exclude the parameter descriptions from the generated MCP schema"
    )]
    pub skip_parameter_descriptions: bool,

    #[arg(
        long,
        env = "RMCP_STATEFUL",
        default_value_t = false,
        help = "Enable stateful session mode for MCP transport (required for Claude Code)"
    )]
    pub stateful: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
