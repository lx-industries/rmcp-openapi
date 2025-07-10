use crate::cli::Cli;
use rmcp_openapi::{OpenApiError, OpenApiSpecLocation};
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub spec_location: OpenApiSpecLocation,
    pub base_url: Option<Url>,
    pub verbose: bool,
    pub port: u16,
    pub bind_address: String,
}

impl Config {
    pub fn from_cli(cli: Cli) -> Result<Self, OpenApiError> {
        // Parse base URL if provided
        let base_url = if let Some(base_url_str) = cli.base_url {
            Some(
                Url::parse(&base_url_str)
                    .map_err(|e| OpenApiError::InvalidUrl(format!("Invalid base URL: {e}")))?,
            )
        } else {
            None
        };

        Ok(Config {
            spec_location: cli.spec,
            base_url,
            verbose: cli.verbose,
            port: cli.port,
            bind_address: cli.bind_address,
        })
    }
}
