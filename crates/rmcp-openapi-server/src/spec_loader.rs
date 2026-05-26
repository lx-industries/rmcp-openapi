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
    /// Load the OpenAPI specification from the source and return as JSON.
    ///
    /// When loading from a URL, `insecure` is forwarded to
    /// [`load_from_url`] and disables TLS verification for that fetch.
    /// The flag is ignored for file-based specs.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the URL cannot be
    /// fetched, or if the resulting body is not valid JSON.
    pub async fn load_json(&self, insecure: bool) -> Result<Value, Error> {
        match self {
            SpecLocation::File(path) => {
                load_from_file(
                    path.to_str().ok_or_else(|| {
                        Error::InvalidPath("Invalid file path encoding".to_string())
                    })?,
                )
                .await
            }
            SpecLocation::Url(url) => load_from_url(url, insecure).await,
        }
    }
}

/// Load and parse an OpenAPI specification from a file
pub async fn load_from_file(path: &str) -> Result<Value, Error> {
    let content = tokio::fs::read_to_string(path).await?;
    let spec: Value = serde_json::from_str(&content)?;
    Ok(spec)
}

/// Load and parse an OpenAPI specification from a URL.
///
/// When `insecure` is `true`, the underlying `reqwest::Client` accepts
/// invalid and hostname-mismatched TLS certificates, mirroring
/// `curl --insecure`. Otherwise, default TLS verification applies.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the response cannot be
/// read, or the body is not valid JSON.
pub async fn load_from_url(url: &Url, insecure: bool) -> Result<Value, Error> {
    let mut builder = reqwest::Client::builder();
    if insecure {
        builder = builder
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true);
    }
    let client = builder
        .build()
        .map_err(|e| Error::Http(format!("Failed to build HTTP client: {e}")))?;
    let response = client.get(url.clone()).send().await?;
    let text = response.text().await?;
    let spec: Value = serde_json::from_str(&text)?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_from_url_with_insecure_works_on_plain_http() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/spec.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"openapi":"3.0.0","info":{"title":"t","version":"1"},"paths":{}}"#)
            .create_async()
            .await;

        let url: Url = format!("{}/spec.json", server.url()).parse().unwrap();
        let spec = load_from_url(&url, true).await.unwrap();
        assert!(spec.is_object());
        assert_eq!(spec["openapi"], "3.0.0");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn load_from_url_without_insecure_works_on_plain_http() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/spec.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"openapi":"3.0.0","info":{"title":"t","version":"1"},"paths":{}}"#)
            .create_async()
            .await;

        let url: Url = format!("{}/spec.json", server.url()).parse().unwrap();
        let spec = load_from_url(&url, false).await.unwrap();
        assert!(spec.is_object());

        mock.assert_async().await;
    }
}
