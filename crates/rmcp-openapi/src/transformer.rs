//! Response transformers for modifying tool call responses.
//!
//! This module provides the [`ResponseTransformer`] trait that allows users to modify/filter
//! tool call responses before they are returned to the MCP client (LLM). Common use cases include:
//!
//! - Removing irrelevant fields to reduce token count
//! - Removing null value fields
//! - Making responses more suitable for LLM consumption
//!
//! # Example
//!
//! ```rust
//! use rmcp_openapi::ResponseTransformer;
//! use serde_json::Value;
//!
//! /// Transformer that removes null fields from responses
//! struct RemoveNulls;
//!
//! impl ResponseTransformer for RemoveNulls {
//!     fn transform_response(&self, response: Value) -> Value {
//!         remove_nulls(response)
//!     }
//!
//!     fn transform_schema(&self, schema: Value) -> Value {
//!         // Mark all fields as optional since they may be removed
//!         mark_fields_optional(schema)
//!     }
//! }
//!
//! fn remove_nulls(value: Value) -> Value {
//!     match value {
//!         Value::Object(map) => {
//!             let filtered: serde_json::Map<String, Value> = map
//!                 .into_iter()
//!                 .filter(|(_, v)| !v.is_null())
//!                 .map(|(k, v)| (k, remove_nulls(v)))
//!                 .collect();
//!             Value::Object(filtered)
//!         }
//!         Value::Array(arr) => {
//!             Value::Array(arr.into_iter().map(remove_nulls).collect())
//!         }
//!         other => other,
//!     }
//! }
//!
//! fn mark_fields_optional(schema: Value) -> Value {
//!     // Implementation would remove required fields from schema
//!     schema
//! }
//! ```

use serde_json::Value;

/// Transforms tool responses and their corresponding schemas.
///
/// Implementors must ensure [`transform_response`](Self::transform_response) and
/// [`transform_schema`](Self::transform_schema) are consistent - if a field is removed
/// from responses, it should also be removed from the schema.
///
/// # Usage
///
/// Response transformers can be applied globally to all tools or per-tool:
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use rmcp_openapi::{Server, ResponseTransformer};
///
/// // Global transformer
/// let mut server = Server::builder()
///     .openapi_spec(spec)
///     .base_url(url)
///     .response_transformer(Arc::new(RemoveNulls))
///     .build();
///
/// server.load_openapi_spec()?;
///
/// // Per-tool override
/// server.set_tool_transformer("verbose-endpoint", Arc::new(AggressiveFilter))?;
/// ```
///
/// # Resolution Order
///
/// 1. Per-tool transformer (if set) takes precedence
/// 2. Else global server transformer
/// 3. Else no transformation
pub trait ResponseTransformer: Send + Sync {
    /// Transform the response body before returning to MCP client.
    ///
    /// This method is called after each successful tool call to transform
    /// the JSON response body. The transformation should be consistent with
    /// [`transform_schema`](Self::transform_schema) - any fields removed here
    /// should also be removed from the schema.
    ///
    /// # Arguments
    ///
    /// * `response` - The JSON response body from the HTTP call
    ///
    /// # Returns
    ///
    /// The transformed response body
    fn transform_response(&self, response: Value) -> Value;

    /// Transform the output schema to match response transformations.
    ///
    /// This method is called when:
    /// - Loading the OpenAPI spec (for global transformers)
    /// - Setting a per-tool transformer
    ///
    /// The schema transformation should reflect what [`transform_response`](Self::transform_response)
    /// will do to the actual responses.
    ///
    /// # Arguments
    ///
    /// * `schema` - The JSON Schema for the tool's output
    ///
    /// # Returns
    ///
    /// The transformed schema
    fn transform_schema(&self, schema: Value) -> Value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Simple transformer that removes null fields
    struct RemoveNulls;

    impl ResponseTransformer for RemoveNulls {
        fn transform_response(&self, response: Value) -> Value {
            remove_nulls(response)
        }

        fn transform_schema(&self, schema: Value) -> Value {
            // For simplicity, just return the schema as-is in tests
            schema
        }
    }

    fn remove_nulls(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let filtered: serde_json::Map<String, Value> = map
                    .into_iter()
                    .filter(|(_, v)| !v.is_null())
                    .map(|(k, v)| (k, remove_nulls(v)))
                    .collect();
                Value::Object(filtered)
            }
            Value::Array(arr) => Value::Array(arr.into_iter().map(remove_nulls).collect()),
            other => other,
        }
    }

    #[test]
    fn test_remove_nulls_transformer() {
        let transformer = RemoveNulls;

        let response = json!({
            "id": 1,
            "name": "Test",
            "description": null,
            "nested": {
                "value": 42,
                "optional": null
            }
        });

        let transformed = transformer.transform_response(response);

        assert_eq!(
            transformed,
            json!({
                "id": 1,
                "name": "Test",
                "nested": {
                    "value": 42
                }
            })
        );
    }

    #[test]
    fn test_transformer_with_arrays() {
        let transformer = RemoveNulls;

        let response = json!({
            "items": [
                {"id": 1, "value": null},
                {"id": 2, "value": "test"}
            ]
        });

        let transformed = transformer.transform_response(response);

        assert_eq!(
            transformed,
            json!({
                "items": [
                    {"id": 1},
                    {"id": 2, "value": "test"}
                ]
            })
        );
    }

    #[test]
    fn test_transformer_preserves_non_null_values() {
        let transformer = RemoveNulls;

        let response = json!({
            "string": "hello",
            "number": 42,
            "boolean": true,
            "array": [1, 2, 3],
            "object": {"key": "value"}
        });

        let transformed = transformer.transform_response(response.clone());

        assert_eq!(transformed, response);
    }
}
