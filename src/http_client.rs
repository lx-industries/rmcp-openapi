use reqwest::header;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::error::OpenApiError;
use crate::server::ToolMetadata;
use crate::tool_generator::{ExtractedParameters, ToolGenerator};

/// HTTP client for executing OpenAPI requests
pub struct HttpClient {
    client: Client,
    base_url: Option<String>,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: None,
        }
    }

    /// Create a new HTTP client with custom timeout
    pub fn with_timeout(timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: None,
        }
    }

    /// Set the base URL for all requests
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Execute an OpenAPI tool call
    pub async fn execute_tool_call(
        &self,
        tool_metadata: &ToolMetadata,
        arguments: &Value,
    ) -> Result<HttpResponse, OpenApiError> {
        // Extract parameters from arguments
        let extracted_params = ToolGenerator::extract_parameters(tool_metadata, arguments)?;

        // Build the URL with path parameters
        let url = self.build_url(tool_metadata, &extracted_params)?;

        // Create the HTTP request
        let mut request = self.create_request(&tool_metadata.method, &url)?;

        // Add query parameters
        if !extracted_params.query.is_empty() {
            request = self.add_query_parameters(request, &extracted_params.query)?;
        }

        // Add headers
        if !extracted_params.headers.is_empty() {
            request = self.add_headers(request, &extracted_params.headers)?;
        }

        // Add cookies
        if !extracted_params.cookies.is_empty() {
            request = self.add_cookies(request, &extracted_params.cookies)?;
        }

        // Add request body if present
        if !extracted_params.body.is_empty() {
            request =
                self.add_request_body(request, &extracted_params.body, &extracted_params.config)?;
        }

        // Apply custom timeout if specified
        if extracted_params.config.timeout_seconds != 30 {
            request = request.timeout(Duration::from_secs(
                extracted_params.config.timeout_seconds as u64,
            ));
        }

        // Capture request details for response formatting
        let request_body_string = if !extracted_params.body.is_empty() {
            if extracted_params.body.len() == 1
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
            }
        } else {
            String::new()
        };

        // Execute the request
        let response = request.send().await.map_err(|e| {
            // Provide more specific error information
            if e.is_timeout() {
                OpenApiError::Http(format!(
                    "Request timeout after {} seconds while calling {} {}",
                    extracted_params.config.timeout_seconds,
                    tool_metadata.method.to_uppercase(),
                    url
                ))
            } else if e.is_connect() {
                OpenApiError::Http(format!(
                    "Connection failed to {url} - check if the server is running and the URL is correct"
                ))
            } else if e.is_request() {
                OpenApiError::Http(format!(
                    "Request error: {} (URL: {}, Method: {})",
                    e,
                    url,
                    tool_metadata.method.to_uppercase()
                ))
            } else {
                OpenApiError::Http(format!(
                    "HTTP request failed: {} (URL: {}, Method: {})",
                    e,
                    url,
                    tool_metadata.method.to_uppercase()
                ))
            }
        })?;

        // Convert response to our format with request details
        self.process_response_with_request(
            response,
            &tool_metadata.method,
            &url,
            &request_body_string,
        )
        .await
    }

    /// Build the complete URL with path parameters substituted
    fn build_url(
        &self,
        tool_metadata: &ToolMetadata,
        extracted_params: &ExtractedParameters,
    ) -> Result<String, OpenApiError> {
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
            let base = base_url.trim_end_matches('/');
            let path = path.trim_start_matches('/');
            Ok(format!("{base}/{path}"))
        } else {
            // Assume the path is already a complete URL
            if path.starts_with("http") {
                Ok(path)
            } else {
                Err(OpenApiError::Http(
                    "No base URL configured and path is not a complete URL".to_string(),
                ))
            }
        }
    }

    /// Create a new HTTP request with the specified method and URL
    fn create_request(&self, method: &str, url: &str) -> Result<RequestBuilder, OpenApiError> {
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
                return Err(OpenApiError::Http(format!(
                    "Unsupported HTTP method: {method}"
                )));
            }
        };

        Ok(self.client.request(method, url))
    }

    /// Add query parameters to the request
    fn add_query_parameters(
        &self,
        mut request: RequestBuilder,
        query_params: &HashMap<String, Value>,
    ) -> Result<RequestBuilder, OpenApiError> {
        for (key, value) in query_params {
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Array(arr) => {
                    // Handle array parameters (comma-separated for now)
                    arr.iter()
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => v.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                }
                _ => value.to_string(),
            };
            request = request.query(&[(key, value_str)]);
        }
        Ok(request)
    }

    /// Add headers to the request
    fn add_headers(
        &self,
        mut request: RequestBuilder,
        headers: &HashMap<String, Value>,
    ) -> Result<RequestBuilder, OpenApiError> {
        for (key, value) in headers {
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            request = request.header(key, value_str);
        }
        Ok(request)
    }

    /// Add cookies to the request
    fn add_cookies(
        &self,
        mut request: RequestBuilder,
        cookies: &HashMap<String, Value>,
    ) -> Result<RequestBuilder, OpenApiError> {
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
        Ok(request)
    }

    /// Add request body to the request
    fn add_request_body(
        &self,
        mut request: RequestBuilder,
        body: &HashMap<String, Value>,
        config: &crate::tool_generator::RequestConfig,
    ) -> Result<RequestBuilder, OpenApiError> {
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
                        OpenApiError::Http(format!("Failed to serialize request body: {e}"))
                    })?;
                    request = request.body(json_string);
                } else {
                    // Create JSON object from all body parameters
                    let body_object =
                        Value::Object(body.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                    let json_string = serde_json::to_string(&body_object).map_err(|e| {
                        OpenApiError::Http(format!("Failed to serialize request body: {e}"))
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
                let json_string = serde_json::to_string(&body_object).map_err(|e| {
                    OpenApiError::Http(format!("Failed to serialize request body: {e}"))
                })?;
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
    ) -> Result<HttpResponse, OpenApiError> {
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
            .map_err(|e| OpenApiError::Http(format!("Failed to read response body: {e}")))?;

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
    pub fn json(&self) -> Result<Value, OpenApiError> {
        serde_json::from_str(&self.body)
            .map_err(|e| OpenApiError::Http(format!("Failed to parse response as JSON: {e}")))
    }

    /// Get a formatted response summary for MCP
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
            result.push_str(&format!("\nRequest: {} {}\n", method.to_uppercase(), url));

            if let Some(body) = request_body {
                if !body.is_empty() && body != "{}" {
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
                    result.push_str(&format!("  {key}: {value}\n"));
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
                result.push_str(&format!(
                    "\n... ({} more characters)",
                    self.body.len() - 2000
                ));
            } else {
                result.push_str(&self.body);
            }
        }

        result
    }
}
