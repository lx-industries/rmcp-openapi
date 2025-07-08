use crate::error::OpenApiError;
use std::env;
use std::io;

/// Legacy OpenAPI functions for backward compatibility
/// These functions use environment variables and custom TLS setup
pub async fn get_openapi_definition() -> Result<serde_json::Value, OpenApiError> {
    let ca_bundle = env::var("REQUESTS_CA_BUNDLE")?;
    let api_host = env::var("OPENAI_API_DEFINITION")?;

    let client = reqwest::Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(reqwest::Certificate::from_pem(&std::fs::read(&ca_bundle)?)?)
        .build()?;

    let response = client.get(&api_host).send().await?;
    let text = response.text().await?;
    let json_value: serde_json::Value = serde_json::from_str(&text)?;

    Ok(json_value)
}

pub async fn get_paths_definition(
    definition: serde_json::Value,
    method: reqwest::Method,
    path: &str,
) -> Result<serde_json::Value, OpenApiError> {
    let paths = definition
        .get("paths")
        .ok_or_else(|| {
            serde_json::Error::io(io::Error::new(
                io::ErrorKind::NotFound,
                "paths not found in OpenAPI definition",
            ))
        })
        .map_err(OpenApiError::Json)?;

    let path_obj = paths
        .get(path)
        .ok_or_else(|| {
            serde_json::Error::io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("path {path} not found"),
            ))
        })
        .map_err(OpenApiError::Json)?;

    let method_str = method.as_str().to_lowercase();
    let method_info = path_obj
        .get(&method_str)
        .ok_or_else(|| {
            serde_json::Error::io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("method {method_str} not found for path {path}"),
            ))
        })
        .map_err(OpenApiError::Json)?;

    Ok(method_info.clone())
}
