use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, info_span};
use url::Url;

use crate::error::{
    Error, NetworkErrorCategory, ToolCallError, ToolCallExecutionError, ToolCallValidationError,
};
use crate::tool::ToolMetadata;
use crate::tool_generator::{ExtractedParameters, QueryParameter, ToolGenerator};

/// HTTP client for executing `OpenAPI` requests
#[derive(Clone)]
pub struct HttpClient {
    client: Arc<Client>,
    base_url: Option<Url>,
    default_headers: HeaderMap,
}

impl HttpClient {
    /// Create the user agent string for HTTP requests
    fn create_user_agent() -> String {
        format!("rmcp-openapi-server/{}", env!("CARGO_PKG_VERSION"))
    }
    /// Create a new HTTP client
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be created
    #[must_use]
    pub fn new() -> Self {
        let user_agent = Self::create_user_agent();
        let client = Client::builder()
            .user_agent(&user_agent)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: Arc::new(client),
            base_url: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// Create a new HTTP client with custom timeout
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be created
    #[must_use]
    pub fn with_timeout(timeout_seconds: u64) -> Self {
        let user_agent = Self::create_user_agent();
        let client = Client::builder()
            .user_agent(&user_agent)
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: Arc::new(client),
            base_url: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// Set the base URL for all requests
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid
    pub fn with_base_url(mut self, base_url: Url) -> Result<Self, Error> {
        self.base_url = Some(base_url);
        Ok(self)
    }

    /// Set default headers for all requests
    #[must_use]
    pub fn with_default_headers(mut self, default_headers: HeaderMap) -> Self {
        self.default_headers = default_headers;
        self
    }

    /// Create a new HTTP client with authorization header
    ///
    /// Clones the current client and adds the Authorization header to default headers.
    /// This allows passing authorization through to backend APIs.
    #[must_use]
    pub fn with_authorization(&self, auth_value: &str) -> Self {
        let mut headers = self.default_headers.clone();
        if let Ok(header_value) = HeaderValue::from_str(auth_value) {
            headers.insert(header::AUTHORIZATION, header_value);
        }

        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            default_headers: headers,
        }
    }

    /// Execute an `OpenAPI` tool call
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or parameters are invalid
    pub async fn execute_tool_call(
        &self,
        tool_metadata: &ToolMetadata,
        arguments: &Value,
    ) -> Result<HttpResponse, ToolCallError> {
        let span = info_span!(
            "http_request",
            operation_id = %tool_metadata.name,
            method = %tool_metadata.method,
            path = %tool_metadata.path
        );
        let _enter = span.enter();

        debug!(
            "Executing tool call: {} {} with arguments: {}",
            tool_metadata.method,
            tool_metadata.path,
            serde_json::to_string_pretty(arguments).unwrap_or_else(|_| "invalid json".to_string())
        );

        // Extract parameters from arguments
        let extracted_params = ToolGenerator::extract_parameters(tool_metadata, arguments)?;

        debug!(
            "Extracted parameters: path={:?}, query={:?}, headers={:?}, cookies={:?}",
            extracted_params.path,
            extracted_params.query,
            extracted_params.headers,
            extracted_params.cookies
        );

        // Build the URL with path parameters
        let mut url = self
            .build_url(tool_metadata, &extracted_params)
            .map_err(|e| {
                ToolCallError::Validation(ToolCallValidationError::RequestConstructionError {
                    reason: e.to_string(),
                })
            })?;

        // Add query parameters with proper URL encoding
        if !extracted_params.query.is_empty() {
            Self::add_query_parameters(&mut url, &extracted_params.query);
        }

        info!("Final URL: {}", url);

        // Create the HTTP request
        let mut request = self
            .create_request(&tool_metadata.method, &url)
            .map_err(|e| {
                ToolCallError::Validation(ToolCallValidationError::RequestConstructionError {
                    reason: e.to_string(),
                })
            })?;

        // Add headers: first default headers, then request-specific headers (which take precedence)
        if !self.default_headers.is_empty() {
            // Use the HeaderMap directly with reqwest
            request = Self::add_headers_from_map(request, &self.default_headers);
        }

        // Add request-specific headers (these override default headers)
        if !extracted_params.headers.is_empty() {
            request = Self::add_headers(request, &extracted_params.headers);
        }

        // Add cookies
        if !extracted_params.cookies.is_empty() {
            request = Self::add_cookies(request, &extracted_params.cookies);
        }

        // Add request body if present
        if !extracted_params.body.is_empty() {
            request =
                Self::add_request_body(request, &extracted_params.body, &extracted_params.config)
                    .map_err(|e| {
                    ToolCallError::Execution(ToolCallExecutionError::ResponseParsingError {
                        reason: format!("Failed to serialize request body: {e}"),
                        raw_response: None,
                    })
                })?;
        }

        // Apply custom timeout if specified
        if extracted_params.config.timeout_seconds != 30 {
            request = request.timeout(Duration::from_secs(u64::from(
                extracted_params.config.timeout_seconds,
            )));
        }

        // Capture request details for response formatting
        let request_body_string = if extracted_params.body.is_empty() {
            String::new()
        } else if extracted_params.body.len() == 1
            && extracted_params.body.contains_key("request_body")
        {
            serde_json::to_string(&extracted_params.body["request_body"]).unwrap_or_default()
        } else {
            let body_object = Value::Object(
                extracted_params
                    .body
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            );
            serde_json::to_string(&body_object).unwrap_or_default()
        };

        // Get the final URL for logging
        let final_url = url.to_string();

        // Execute the request
        debug!("Sending HTTP request...");
        let start_time = std::time::Instant::now();
        let response = request.send().await.map_err(|e| {
            error!(
                operation_id = %tool_metadata.name,
                method = %tool_metadata.method,
                url = %final_url,
                error = %e,
                "HTTP request failed"
            );

            // Categorize error based on reqwest's reliable error detection methods
            let (error_msg, category) = if e.is_timeout() {
                (
                    format!(
                        "Request timeout after {} seconds while calling {} {}",
                        extracted_params.config.timeout_seconds,
                        tool_metadata.method.to_uppercase(),
                        final_url
                    ),
                    NetworkErrorCategory::Timeout,
                )
            } else if e.is_connect() {
                (
                    format!(
                        "Connection failed to {final_url} - Error: {e}. Check if the server is running and the URL is correct."
                    ),
                    NetworkErrorCategory::Connect,
                )
            } else if e.is_request() {
                (
                    format!(
                        "Request error while calling {} {} - Error: {}",
                        tool_metadata.method.to_uppercase(),
                        final_url,
                        e
                    ),
                    NetworkErrorCategory::Request,
                )
            } else if e.is_body() {
                (
                    format!(
                        "Body error while calling {} {} - Error: {}",
                        tool_metadata.method.to_uppercase(),
                        final_url,
                        e
                    ),
                    NetworkErrorCategory::Body,
                )
            } else if e.is_decode() {
                (
                    format!(
                        "Response decode error from {} {} - Error: {}",
                        tool_metadata.method.to_uppercase(),
                        final_url,
                        e
                    ),
                    NetworkErrorCategory::Decode,
                )
            } else {
                (
                    format!(
                        "HTTP request failed: {} (URL: {}, Method: {})",
                        e,
                        final_url,
                        tool_metadata.method.to_uppercase()
                    ),
                    NetworkErrorCategory::Other,
                )
            };

            ToolCallError::Execution(ToolCallExecutionError::NetworkError {
                message: error_msg,
                category,
            })
        })?;

        let elapsed = start_time.elapsed();
        info!(
            operation_id = %tool_metadata.name,
            method = %tool_metadata.method,
            url = %final_url,
            status = response.status().as_u16(),
            elapsed_ms = elapsed.as_millis(),
            "HTTP request completed"
        );
        debug!("Response received with status: {}", response.status());

        // Convert response to our format with request details
        self.process_response_with_request(
            response,
            &tool_metadata.method,
            &final_url,
            &request_body_string,
        )
        .await
        .map_err(|e| {
            ToolCallError::Execution(ToolCallExecutionError::HttpError {
                status: 0,
                message: e.to_string(),
                details: None,
            })
        })
    }

    /// Build the complete URL with path parameters substituted
    fn build_url(
        &self,
        tool_metadata: &ToolMetadata,
        extracted_params: &ExtractedParameters,
    ) -> Result<Url, Error> {
        let mut path = tool_metadata.path.clone();

        // Substitute path parameters
        for (param_name, param_value) in &extracted_params.path {
            let placeholder = format!("{{{param_name}}}");
            let value_str = match param_value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => param_value.to_string(),
            };
            path = path.replace(&placeholder, &value_str);
        }

        // Combine with base URL if available
        if let Some(base_url) = &self.base_url {
            base_url.join(&path).map_err(|e| {
                Error::Http(format!(
                    "Failed to join URL '{base_url}' with path '{path}': {e}"
                ))
            })
        } else {
            // Assume the path is already a complete URL
            if path.starts_with("http") {
                Url::parse(&path).map_err(|e| Error::Http(format!("Invalid URL '{path}': {e}")))
            } else {
                Err(Error::Http(
                    "No base URL configured and path is not a complete URL".to_string(),
                ))
            }
        }
    }

    /// Create a new HTTP request with the specified method and URL
    fn create_request(&self, method: &str, url: &Url) -> Result<RequestBuilder, Error> {
        let http_method = method.to_uppercase();
        let method = match http_method.as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            _ => {
                return Err(Error::Http(format!(
                    "Unsupported HTTP method: {http_method}"
                )));
            }
        };

        Ok(self.client.request(method, url.clone()))
    }

    /// Add query parameters to the request using proper URL encoding
    fn add_query_parameters(url: &mut Url, query_params: &HashMap<String, QueryParameter>) {
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, query_param) in query_params {
                if let Value::Array(arr) = &query_param.value {
                    if query_param.explode {
                        // explode=true: Handle array parameters - add each value as a separate query parameter
                        for item in arr {
                            let item_str = match item {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                _ => item.to_string(),
                            };
                            query_pairs.append_pair(key, &item_str);
                        }
                    } else {
                        // explode=false: Join array values with commas
                        let array_values: Vec<String> = arr
                            .iter()
                            .map(|item| match item {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                _ => item.to_string(),
                            })
                            .collect();
                        let comma_separated = array_values.join(",");
                        query_pairs.append_pair(key, &comma_separated);
                    }
                } else {
                    let value_str = match &query_param.value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => query_param.value.to_string(),
                    };
                    query_pairs.append_pair(key, &value_str);
                }
            }
        }
    }

    /// Add headers to the request from HeaderMap
    fn add_headers_from_map(mut request: RequestBuilder, headers: &HeaderMap) -> RequestBuilder {
        for (key, value) in headers {
            // HeaderName and HeaderValue are already validated, pass them directly to reqwest
            request = request.header(key, value);
        }
        request
    }

    /// Add headers to the request
    fn add_headers(
        mut request: RequestBuilder,
        headers: &HashMap<String, Value>,
    ) -> RequestBuilder {
        for (key, value) in headers {
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            request = request.header(key, value_str);
        }
        request
    }

    /// Add cookies to the request
    fn add_cookies(
        mut request: RequestBuilder,
        cookies: &HashMap<String, Value>,
    ) -> RequestBuilder {
        if !cookies.is_empty() {
            let cookie_header = cookies
                .iter()
                .map(|(key, value)| {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => value.to_string(),
                    };
                    format!("{key}={value_str}")
                })
                .collect::<Vec<_>>()
                .join("; ");

            request = request.header(header::COOKIE, cookie_header);
        }
        request
    }

    /// Add request body to the request
    fn add_request_body(
        mut request: RequestBuilder,
        body: &HashMap<String, Value>,
        config: &crate::tool_generator::RequestConfig,
    ) -> Result<RequestBuilder, Error> {
        if body.is_empty() {
            return Ok(request);
        }

        // Set content type header
        request = request.header(header::CONTENT_TYPE, &config.content_type);

        // Handle different content types
        match config.content_type.as_str() {
            s if s == mime::APPLICATION_JSON.as_ref() => {
                // For JSON content type, serialize the body
                if body.len() == 1 && body.contains_key("request_body") {
                    // Use the request_body directly if it's the only parameter
                    let body_value = &body["request_body"];
                    let json_string = serde_json::to_string(body_value).map_err(|e| {
                        Error::Http(format!("Failed to serialize request body: {e}"))
                    })?;
                    request = request.body(json_string);
                } else {
                    // Create JSON object from all body parameters
                    let body_object =
                        Value::Object(body.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                    let json_string = serde_json::to_string(&body_object).map_err(|e| {
                        Error::Http(format!("Failed to serialize request body: {e}"))
                    })?;
                    request = request.body(json_string);
                }
            }
            s if s == mime::APPLICATION_WWW_FORM_URLENCODED.as_ref() => {
                // Handle form data
                let form_data: Vec<(String, String)> = body
                    .iter()
                    .map(|(key, value)| {
                        let value_str = match value {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => value.to_string(),
                        };
                        (key.clone(), value_str)
                    })
                    .collect();
                request = request.form(&form_data);
            }
            _ => {
                // For other content types, try to serialize as JSON
                let body_object =
                    Value::Object(body.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                let json_string = serde_json::to_string(&body_object)
                    .map_err(|e| Error::Http(format!("Failed to serialize request body: {e}")))?;
                request = request.body(json_string);
            }
        }

        Ok(request)
    }

    /// Process the HTTP response with request details for better formatting
    async fn process_response_with_request(
        &self,
        response: reqwest::Response,
        method: &str,
        url: &str,
        request_body: &str,
    ) -> Result<HttpResponse, Error> {
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().unwrap_or("<invalid>").to_string(),
                )
            })
            .collect();

        let body = response
            .text()
            .await
            .map_err(|e| Error::Http(format!("Failed to read response body: {e}")))?;

        let is_success = status.is_success();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        // Add additional context for common error status codes
        let enhanced_status_text = match status {
            StatusCode::BAD_REQUEST => {
                format!("{status_text} - Bad Request: Check request parameters")
            }
            StatusCode::UNAUTHORIZED => {
                format!("{status_text} - Unauthorized: Authentication required")
            }
            StatusCode::FORBIDDEN => format!("{status_text} - Forbidden: Access denied"),
            StatusCode::NOT_FOUND => {
                format!("{status_text} - Not Found: Endpoint or resource does not exist")
            }
            StatusCode::METHOD_NOT_ALLOWED => format!(
                "{} - Method Not Allowed: {} method not supported",
                status_text,
                method.to_uppercase()
            ),
            StatusCode::UNPROCESSABLE_ENTITY => {
                format!("{status_text} - Unprocessable Entity: Request validation failed")
            }
            StatusCode::TOO_MANY_REQUESTS => {
                format!("{status_text} - Too Many Requests: Rate limit exceeded")
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                format!("{status_text} - Internal Server Error: Server encountered an error")
            }
            StatusCode::BAD_GATEWAY => {
                format!("{status_text} - Bad Gateway: Upstream server error")
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                format!("{status_text} - Service Unavailable: Server temporarily unavailable")
            }
            StatusCode::GATEWAY_TIMEOUT => {
                format!("{status_text} - Gateway Timeout: Upstream server timeout")
            }
            _ => status_text,
        };

        Ok(HttpResponse {
            status_code,
            status_text: enhanced_status_text,
            headers,
            body,
            is_success,
            request_method: method.to_string(),
            request_url: url.to_string(),
            request_body: request_body.to_string(),
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP response from an API call
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub is_success: bool,
    pub request_method: String,
    pub request_url: String,
    pub request_body: String,
}

impl HttpResponse {
    /// Try to parse the response body as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if the body is not valid JSON
    pub fn json(&self) -> Result<Value, Error> {
        serde_json::from_str(&self.body)
            .map_err(|e| Error::Http(format!("Failed to parse response as JSON: {e}")))
    }

    /// Get a formatted response summary for MCP
    #[must_use]
    pub fn to_mcp_content(&self) -> String {
        let method = if self.request_method.is_empty() {
            None
        } else {
            Some(self.request_method.as_str())
        };
        let url = if self.request_url.is_empty() {
            None
        } else {
            Some(self.request_url.as_str())
        };
        let body = if self.request_body.is_empty() {
            None
        } else {
            Some(self.request_body.as_str())
        };
        self.to_mcp_content_with_request(method, url, body)
    }

    /// Get a formatted response summary for MCP with request details
    pub fn to_mcp_content_with_request(
        &self,
        method: Option<&str>,
        url: Option<&str>,
        request_body: Option<&str>,
    ) -> String {
        let mut result = format!(
            "HTTP {} {}\n\nStatus: {} {}\n",
            if self.is_success { "✅" } else { "❌" },
            if self.is_success { "Success" } else { "Error" },
            self.status_code,
            self.status_text
        );

        // Add request details if provided
        if let (Some(method), Some(url)) = (method, url) {
            result.push_str("\nRequest: ");
            result.push_str(&method.to_uppercase());
            result.push(' ');
            result.push_str(url);
            result.push('\n');

            if let Some(body) = request_body
                && !body.is_empty()
                && body != "{}"
            {
                result.push_str("\nRequest Body:\n");
                if let Ok(parsed) = serde_json::from_str::<Value>(body) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                        result.push_str(&pretty);
                    } else {
                        result.push_str(body);
                    }
                } else {
                    result.push_str(body);
                }
                result.push('\n');
            }
        }

        // Add important headers
        if !self.headers.is_empty() {
            result.push_str("\nHeaders:\n");
            for (key, value) in &self.headers {
                // Only show commonly useful headers
                if [
                    header::CONTENT_TYPE.as_str(),
                    header::CONTENT_LENGTH.as_str(),
                    header::LOCATION.as_str(),
                    header::SET_COOKIE.as_str(),
                ]
                .iter()
                .any(|&h| key.to_lowercase().contains(h))
                {
                    result.push_str("  ");
                    result.push_str(key);
                    result.push_str(": ");
                    result.push_str(value);
                    result.push('\n');
                }
            }
        }

        // Add body content
        result.push_str("\nResponse Body:\n");
        if self.body.is_empty() {
            result.push_str("(empty)");
        } else if let Ok(json_value) = self.json() {
            // Pretty print JSON if possible
            match serde_json::to_string_pretty(&json_value) {
                Ok(pretty) => result.push_str(&pretty),
                Err(_) => result.push_str(&self.body),
            }
        } else {
            // Truncate very long responses
            if self.body.len() > 2000 {
                result.push_str(&self.body[..2000]);
                result.push_str("\n... (");
                result.push_str(&(self.body.len() - 2000).to_string());
                result.push_str(" more characters)");
            } else {
                result.push_str(&self.body);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_generator::ExtractedParameters;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_with_base_url_validation() {
        // Test valid URLs
        let url = Url::parse("https://api.example.com").unwrap();
        let client = HttpClient::new().with_base_url(url);
        assert!(client.is_ok());

        let url = Url::parse("http://localhost:8080").unwrap();
        let client = HttpClient::new().with_base_url(url);
        assert!(client.is_ok());

        // Test invalid URLs - these will fail at parse time now
        assert!(Url::parse("not-a-url").is_err());
        assert!(Url::parse("").is_err());

        // Test schemes that parse successfully
        let url = Url::parse("ftp://invalid-scheme.com").unwrap();
        let client = HttpClient::new().with_base_url(url);
        assert!(client.is_ok()); // url crate accepts ftp, our HttpClient should too
    }

    #[test]
    fn test_build_url_with_base_url() {
        let base_url = Url::parse("https://api.example.com").unwrap();
        let client = HttpClient::new().with_base_url(base_url).unwrap();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/pets/{id}".to_string(),
            security: None,
        };

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), json!(123));

        let extracted_params = ExtractedParameters {
            path: path_params,
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let url = client.build_url(&tool_metadata, &extracted_params).unwrap();
        assert_eq!(url.to_string(), "https://api.example.com/pets/123");
    }

    #[test]
    fn test_build_url_without_base_url() {
        let client = HttpClient::new();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "https://api.example.com/pets/123".to_string(),
            security: None,
        };

        let extracted_params = ExtractedParameters {
            path: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let url = client.build_url(&tool_metadata, &extracted_params).unwrap();
        assert_eq!(url.to_string(), "https://api.example.com/pets/123");

        // Test error case: relative path without base URL
        let tool_metadata_relative = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/pets/123".to_string(),
            security: None,
        };

        let result = client.build_url(&tool_metadata_relative, &extracted_params);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No base URL configured")
        );
    }

    #[test]
    fn test_query_parameter_encoding_integration() {
        let base_url = Url::parse("https://api.example.com").unwrap();
        let client = HttpClient::new().with_base_url(base_url).unwrap();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/search".to_string(),
            security: None,
        };

        // Test various query parameter values that need encoding
        let mut query_params = HashMap::new();
        query_params.insert(
            "q".to_string(),
            QueryParameter::new(json!("hello world"), true),
        ); // space
        query_params.insert(
            "category".to_string(),
            QueryParameter::new(json!("pets&dogs"), true),
        ); // ampersand
        query_params.insert(
            "special".to_string(),
            QueryParameter::new(json!("foo=bar"), true),
        ); // equals
        query_params.insert(
            "unicode".to_string(),
            QueryParameter::new(json!("café"), true),
        ); // unicode
        query_params.insert(
            "percent".to_string(),
            QueryParameter::new(json!("100%"), true),
        ); // percent

        let extracted_params = ExtractedParameters {
            path: HashMap::new(),
            query: query_params,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let mut url = client.build_url(&tool_metadata, &extracted_params).unwrap();
        HttpClient::add_query_parameters(&mut url, &extracted_params.query);

        let url_string = url.to_string();

        // Verify the URL contains properly encoded parameters
        // Note: url crate encodes spaces as + in query parameters (which is valid)
        assert!(url_string.contains("q=hello+world")); // space encoded as +
        assert!(url_string.contains("category=pets%26dogs")); // & encoded as %26
        assert!(url_string.contains("special=foo%3Dbar")); // = encoded as %3D
        assert!(url_string.contains("unicode=caf%C3%A9")); // é encoded as %C3%A9
        assert!(url_string.contains("percent=100%25")); // % encoded as %25
    }

    #[test]
    fn test_array_query_parameters() {
        let base_url = Url::parse("https://api.example.com").unwrap();
        let client = HttpClient::new().with_base_url(base_url).unwrap();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/search".to_string(),
            security: None,
        };

        let mut query_params = HashMap::new();
        query_params.insert(
            "status".to_string(),
            QueryParameter::new(json!(["available", "pending"]), true),
        );
        query_params.insert(
            "tags".to_string(),
            QueryParameter::new(json!(["red & blue", "fast=car"]), true),
        );

        let extracted_params = ExtractedParameters {
            path: HashMap::new(),
            query: query_params,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let mut url = client.build_url(&tool_metadata, &extracted_params).unwrap();
        HttpClient::add_query_parameters(&mut url, &extracted_params.query);

        let url_string = url.to_string();

        // Verify array parameters are added multiple times with proper encoding
        assert!(url_string.contains("status=available"));
        assert!(url_string.contains("status=pending"));
        assert!(url_string.contains("tags=red+%26+blue")); // "red & blue" encoded (spaces as +)
        assert!(url_string.contains("tags=fast%3Dcar")); // "fast=car" encoded
    }

    #[test]
    fn test_path_parameter_substitution() {
        let base_url = Url::parse("https://api.example.com").unwrap();
        let client = HttpClient::new().with_base_url(base_url).unwrap();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/users/{userId}/pets/{petId}".to_string(),
            security: None,
        };

        let mut path_params = HashMap::new();
        path_params.insert("userId".to_string(), json!(42));
        path_params.insert("petId".to_string(), json!("special-pet-123"));

        let extracted_params = ExtractedParameters {
            path: path_params,
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let url = client.build_url(&tool_metadata, &extracted_params).unwrap();
        assert_eq!(
            url.to_string(),
            "https://api.example.com/users/42/pets/special-pet-123"
        );
    }

    #[test]
    fn test_url_join_edge_cases() {
        // Test trailing slash handling
        let base_url1 = Url::parse("https://api.example.com/").unwrap();
        let client1 = HttpClient::new().with_base_url(base_url1).unwrap();

        let base_url2 = Url::parse("https://api.example.com").unwrap();
        let client2 = HttpClient::new().with_base_url(base_url2).unwrap();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/pets".to_string(),
            security: None,
        };

        let extracted_params = ExtractedParameters {
            path: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let url1 = client1
            .build_url(&tool_metadata, &extracted_params)
            .unwrap();
        let url2 = client2
            .build_url(&tool_metadata, &extracted_params)
            .unwrap();

        // Both should produce the same normalized URL
        assert_eq!(url1.to_string(), "https://api.example.com/pets");
        assert_eq!(url2.to_string(), "https://api.example.com/pets");
    }

    #[test]
    fn test_explode_array_parameters() {
        let base_url = Url::parse("https://api.example.com").unwrap();
        let client = HttpClient::new().with_base_url(base_url).unwrap();

        let tool_metadata = crate::ToolMetadata {
            name: "test".to_string(),
            title: None,
            description: Some("test".to_string()),
            parameters: json!({}),
            output_schema: None,
            method: "GET".to_string(),
            path: "/search".to_string(),
            security: None,
        };

        // Test explode=true (should generate separate parameters)
        let mut query_params_exploded = HashMap::new();
        query_params_exploded.insert(
            "include".to_string(),
            QueryParameter::new(json!(["asset", "scenes"]), true),
        );

        let extracted_params_exploded = ExtractedParameters {
            path: HashMap::new(),
            query: query_params_exploded,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let mut url_exploded = client
            .build_url(&tool_metadata, &extracted_params_exploded)
            .unwrap();
        HttpClient::add_query_parameters(&mut url_exploded, &extracted_params_exploded.query);
        let url_exploded_string = url_exploded.to_string();

        // Test explode=false (should generate comma-separated values)
        let mut query_params_not_exploded = HashMap::new();
        query_params_not_exploded.insert(
            "include".to_string(),
            QueryParameter::new(json!(["asset", "scenes"]), false),
        );

        let extracted_params_not_exploded = ExtractedParameters {
            path: HashMap::new(),
            query: query_params_not_exploded,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: HashMap::new(),
            config: crate::tool_generator::RequestConfig::default(),
        };

        let mut url_not_exploded = client
            .build_url(&tool_metadata, &extracted_params_not_exploded)
            .unwrap();
        HttpClient::add_query_parameters(
            &mut url_not_exploded,
            &extracted_params_not_exploded.query,
        );
        let url_not_exploded_string = url_not_exploded.to_string();

        // Verify explode=true generates separate parameters
        assert!(url_exploded_string.contains("include=asset"));
        assert!(url_exploded_string.contains("include=scenes"));

        // Verify explode=false generates comma-separated values
        assert!(url_not_exploded_string.contains("include=asset%2Cscenes")); // comma is URL-encoded as %2C

        // Make sure they're different
        assert_ne!(url_exploded_string, url_not_exploded_string);

        println!("Exploded URL: {url_exploded_string}");
        println!("Non-exploded URL: {url_not_exploded_string}");
    }
}
