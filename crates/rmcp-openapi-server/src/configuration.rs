use crate::cli::Cli;
use crate::spec_loader::SpecLocation;
use bon::Builder;
use reqwest::header::HeaderMap;
use rmcp_openapi::{AuthorizationMode, CliError, Error, Server};
use url::Url;

#[derive(Debug, Clone, Builder)]
pub struct Configuration {
    pub spec_location: SpecLocation,
    pub base_url: Url,
    pub port: u16,
    pub bind_address: String,
    pub default_headers: HeaderMap,
    pub tags: Option<Vec<String>>,
    pub methods: Option<Vec<reqwest::Method>>,
    pub authorization_mode: AuthorizationMode,
}

impl Configuration {
    pub fn from_cli(cli: Cli) -> Result<Self, Error> {
        // Parse base URL - now required by CLI
        let base_url = Url::parse(&cli.base_url)
            .map_err(|e| Error::InvalidUrl(format!("Invalid base URL: {e}")))?;

        // Parse headers from CLI format "name: value"
        let mut default_headers = HeaderMap::new();
        for header_str in cli.headers {
            if let Some((key, value)) = header_str.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                if key.is_empty() {
                    return Err(Error::Cli(CliError::InvalidHeaderFormat {
                        header: header_str,
                    }));
                }

                // Validate header name using reqwest/http
                let header_name =
                    http::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                        Error::Cli(CliError::InvalidHeaderName {
                            header: header_str.clone(),
                            source: e,
                        })
                    })?;

                // Validate header value using reqwest/http
                let header_value = http::header::HeaderValue::from_str(value).map_err(|e| {
                    Error::Cli(CliError::InvalidHeaderValue {
                        header: header_str.clone(),
                        source: e,
                    })
                })?;

                default_headers.insert(header_name, header_value);
            } else {
                return Err(Error::Cli(CliError::InvalidHeaderFormat {
                    header: header_str,
                }));
            }
        }

        Ok(Configuration {
            spec_location: cli.spec,
            base_url,
            port: cli.port,
            bind_address: cli.bind_address,
            default_headers,
            tags: cli.tags,
            methods: cli.methods,
            authorization_mode: cli.authorization_mode,
        })
    }
}

impl Configuration {
    /// Convert Configuration to Server by loading the OpenAPI spec
    pub async fn try_into_server(self) -> Result<Server, Error> {
        // Load OpenAPI specification from the spec location
        let openapi_spec = self.spec_location.load_json().await?;

        let headers = if self.default_headers.is_empty() {
            None
        } else {
            Some(self.default_headers)
        };

        let mut server = Server::new(
            openapi_spec,
            self.base_url,
            headers,
            self.tags,
            self.methods,
        );

        // Set the authorization mode
        server.set_authorization_mode(self.authorization_mode);

        Ok(server)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::spec_loader::SpecLocation;
    use url::Url;

    #[test]
    fn test_header_parsing_valid_formats() {
        let cli = Cli {
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![
                "Authorization: Bearer token123".to_string(),
                "X-API-Key: key456".to_string(),
                "Content-Type: application/json".to_string(),
                "User-Agent: TestAgent/1.0".to_string(),
            ],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let config = Configuration::from_cli(cli).unwrap();

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
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![
                " Authorization : Bearer token123 ".to_string(),
                "X-Custom  :  value with spaces  ".to_string(),
            ],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let config = Configuration::from_cli(cli).unwrap();

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
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["InvalidHeaderNoEquals".to_string()],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let result = Configuration::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("Invalid header format"));
        assert!(error.contains("expected 'name: value' format"));
    }

    #[test]
    fn test_header_parsing_invalid_format_empty_key() {
        let cli = Cli {
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![": value".to_string()],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let result = Configuration::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("CLI error"));
        assert!(error.contains("Invalid header format"));
    }

    #[test]
    fn test_header_parsing_empty_value_allowed() {
        let cli = Cli {
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["X-Empty-Header:".to_string()],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let config = Configuration::from_cli(cli).unwrap();

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
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec![],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let config = Configuration::from_cli(cli).unwrap();
        assert!(config.default_headers.is_empty());
    }

    #[test]
    fn test_header_validation_invalid_header_name() {
        let cli = Cli {
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["Invalid Header Name: value".to_string()],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let result = Configuration::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("CLI error"));
        assert!(error.contains("Invalid header name"));
    }

    #[test]
    fn test_header_validation_invalid_header_value() {
        let cli = Cli {
            spec: SpecLocation::Url(Url::parse("https://example.com/spec.json").unwrap()),
            base_url: "https://api.example.com".to_string(),
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            headers: vec!["Valid-Header: invalid\x00value".to_string()],
            tags: None,
            methods: None,
            authorization_mode: AuthorizationMode::default(),
        };

        let result = Configuration::from_cli(cli);
        assert!(result.is_err());

        let error = result.unwrap_err().to_string();
        assert!(error.contains("CLI error"));
        assert!(error.contains("Invalid header value"));
    }
}
