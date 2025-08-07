use crate::cli::Cli;
use reqwest::header::HeaderMap;
use rmcp_openapi::{CliError, OpenApiError, OpenApiSpecLocation};

use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub spec_location: OpenApiSpecLocation,
    pub base_url: Option<Url>,
    pub verbose: bool,
    pub port: u16,
    pub bind_address: String,
    pub default_headers: HeaderMap,
    pub tags: Option<Vec<String>>,
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

        // Parse headers from CLI format "name: value"
        let mut default_headers = HeaderMap::new();
        for header_str in cli.headers {
            if let Some((key, value)) = header_str.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                if key.is_empty() {
                    return Err(OpenApiError::Cli(CliError::InvalidHeaderFormat {
                        header: header_str,
                    }));
                }

                // Validate header name using reqwest/http
                let header_name =
                    http::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                        OpenApiError::Cli(CliError::InvalidHeaderName {
                            header: header_str.clone(),
                            source: e,
                        })
                    })?;

                // Validate header value using reqwest/http
                let header_value = http::header::HeaderValue::from_str(value).map_err(|e| {
                    OpenApiError::Cli(CliError::InvalidHeaderValue {
                        header: header_str.clone(),
                        source: e,
                    })
                })?;

                default_headers.insert(header_name, header_value);
            } else {
                return Err(OpenApiError::Cli(CliError::InvalidHeaderFormat {
                    header: header_str,
                }));
            }
        }

        Ok(Config {
            spec_location: cli.spec,
            base_url,
            verbose: cli.verbose,
            port: cli.port,
            bind_address: cli.bind_address,
            default_headers,
            tags: cli.tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use rmcp_openapi::OpenApiSpecLocation;
    use url::Url;

    #[test]
    fn test_header_parsing_valid_formats() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![
                "Authorization: Bearer token123".to_string(),
                "X-API-Key: key456".to_string(),
                "Content-Type: application/json".to_string(),
                "User-Agent: TestAgent/1.0".to_string(),
            ],
            tags: None,
        };

        let config = Config::from_cli(cli).unwrap();

        assert_eq!(config.default_headers.len(), 4);
        assert_eq!(
            config
                .default_headers
                .get("Authorization")
                .map(|v| v.to_str().unwrap()),
            Some("Bearer token123")
        );
        assert_eq!(
            config
                .default_headers
                .get("X-API-Key")
                .map(|v| v.to_str().unwrap()),
            Some("key456")
        );
        assert_eq!(
            config
                .default_headers
                .get("Content-Type")
                .map(|v| v.to_str().unwrap()),
            Some("application/json")
        );
        assert_eq!(
            config
                .default_headers
                .get("User-Agent")
                .map(|v| v.to_str().unwrap()),
            Some("TestAgent/1.0")
        );
    }

    #[test]
    fn test_header_parsing_with_spaces() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![
                " Authorization : Bearer token123 ".to_string(),
                "X-Custom  :  value with spaces  ".to_string(),
            ],
            tags: None,
        };

        let config = Config::from_cli(cli).unwrap();

        assert_eq!(config.default_headers.len(), 2);
        assert_eq!(
            config
                .default_headers
                .get("Authorization")
                .map(|v| v.to_str().unwrap()),
            Some("Bearer token123")
        );
        assert_eq!(
            config
                .default_headers
                .get("X-Custom")
                .map(|v| v.to_str().unwrap()),
            Some("value with spaces")
        );
    }

    #[test]
    fn test_header_parsing_invalid_format_no_equals() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["InvalidHeaderNoEquals".to_string()],
            tags: None,
        };

        let result = Config::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("Invalid header format"));
        assert!(error.contains("expected 'name: value' format"));
    }

    #[test]
    fn test_header_parsing_invalid_format_empty_key() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![": value".to_string()],
            tags: None,
        };

        let result = Config::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("CLI error"));
        assert!(error.contains("Invalid header format"));
    }

    #[test]
    fn test_header_parsing_empty_value_allowed() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["X-Empty-Header:".to_string()],
            tags: None,
        };

        let config = Config::from_cli(cli).unwrap();

        assert_eq!(config.default_headers.len(), 1);
        assert_eq!(
            config
                .default_headers
                .get("X-Empty-Header")
                .map(|v| v.to_str().unwrap()),
            Some("")
        );
    }

    #[test]
    fn test_no_headers() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![],
            tags: None,
        };

        let config = Config::from_cli(cli).unwrap();
        assert!(config.default_headers.is_empty());
    }

    #[test]
    fn test_header_validation_invalid_header_name() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["Invalid Header Name: value".to_string()],
            tags: None,
        };

        let result = Config::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("CLI error"));
        assert!(error.contains("Invalid header name"));
    }

    #[test]
    fn test_header_validation_invalid_header_value() {
        let cli = Cli {
            spec: OpenApiSpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: None,
            verbose: false,
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["Valid-Header: invalid\x00value".to_string()],
            tags: None,
        };

        let result = Config::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("CLI error"));
        assert!(error.contains("Invalid header value"));
    }
}
