pub mod error;
pub mod http_client;
pub mod openapi;
pub mod server;
pub mod tool_generator;
pub mod tool_registry;

pub use error::{CliError, OpenApiError, ToolCallError};
pub use http_client::{HttpClient, HttpResponse};
pub use openapi::OpenApiSpec;
pub use openapi::OpenApiSpecLocation;
pub use server::{OpenApiServer, ToolMetadata};
pub use tool_generator::{ExtractedParameters, RequestConfig, ToolGenerator};
pub use tool_registry::{ToolRegistry, ToolRegistryStats};

/// Find similar strings using Jaro distance algorithm
/// Used for parameter and tool name suggestions in errors
pub(crate) fn find_similar_strings(unknown: &str, known_strings: &[&str]) -> Vec<String> {
    use strsim::jaro;

    let mut candidates = Vec::new();
    for string in known_strings {
        let confidence = jaro(unknown, string);
        if confidence > 0.7 {
            candidates.push((confidence, string.to_string()));
        }
    }

    // Sort by confidence (highest first)
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    candidates.into_iter().map(|(_, name)| name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_similar_strings() {
        // Test basic similarity
        let known = vec!["page_size", "user_id", "status"];
        let suggestions = find_similar_strings("page_sixe", &known);
        assert_eq!(suggestions, vec!["page_size"]);

        // Test no suggestions for very different string
        let suggestions = find_similar_strings("xyz123", &known);
        assert!(suggestions.is_empty());

        // Test transposed characters
        let known = vec!["limit", "offset"];
        let suggestions = find_similar_strings("lmiit", &known);
        assert_eq!(suggestions, vec!["limit"]);

        // Test missing character
        let known = vec!["project_id", "merge_request_id"];
        let suggestions = find_similar_strings("projct_id", &known);
        assert_eq!(suggestions, vec!["project_id"]);

        // Test extra character
        let known = vec!["name", "email"];
        let suggestions = find_similar_strings("namee", &known);
        assert_eq!(suggestions, vec!["name"]);
    }
}
