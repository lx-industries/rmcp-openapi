use std::path::PathBuf;
use std::str::FromStr;

use rmcp_openapi::Error;
use serde_json::Value;
use url::Url;

/// Represents different sources for loading OpenAPI specifications
#[derive(Debug, Clone)]
pub enum SpecLocation {
    File(PathBuf),
    Url(Url),
}

impl FromStr for SpecLocation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            let url = Url::parse(s).map_err(|e| Error::InvalidUrl(format!("Invalid URL: {e}")))?;
            Ok(SpecLocation::Url(url))
        } else {
            let path = PathBuf::from(s);
            Ok(SpecLocation::File(path))
        }
    }
}

impl std::fmt::Display for SpecLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecLocation::File(path) => write!(f, "{}", path.display()),
            SpecLocation::Url(url) => write!(f, "{url}"),
        }
    }
}

impl SpecLocation {
    /// Load the OpenAPI specification from the source and return as JSON
    pub async fn load_json(&self) -> Result<Value, Error> {
        match self {
            SpecLocation::File(path) => {
                load_from_file(
                    path.to_str().ok_or_else(|| {
                        Error::InvalidPath("Invalid file path encoding".to_string())
                    })?,
                )
                .await
            }
            SpecLocation::Url(url) => load_from_url(url).await,
        }
    }
}

/// Load and parse an OpenAPI specification from a file
pub async fn load_from_file(path: &str) -> Result<Value, Error> {
    let content = tokio::fs::read_to_string(path).await?;
    let spec: Value = serde_json::from_str(&content)?;
    Ok(spec)
}

/// Load and parse an OpenAPI specification from a URL
pub async fn load_from_url(url: &Url) -> Result<Value, Error> {
    let client = reqwest::Client::new();
    let response = client.get(url.clone()).send().await?;
    let text = response.text().await?;
    let spec: Value = serde_json::from_str(&text)?;
    Ok(spec)
}
