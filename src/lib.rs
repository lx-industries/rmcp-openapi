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

/// Normalize tag strings to kebab-case for consistent filtering
/// Converts any case format (camelCase, PascalCase, snake_case, etc.) to kebab-case
pub(crate) fn normalize_tag(tag: &str) -> String {
    use heck::ToKebabCase;
    tag.to_kebab_case()
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

    #[test]
    fn test_normalize_tag() {
        // Test camelCase conversion
        assert_eq!(normalize_tag("userManagement"), "user-management");
        assert_eq!(normalize_tag("getUsers"), "get-users");

        // Test PascalCase conversion
        assert_eq!(normalize_tag("UserManagement"), "user-management");
        assert_eq!(normalize_tag("APIKey"), "api-key");

        // Test snake_case conversion
        assert_eq!(normalize_tag("user_management"), "user-management");
        assert_eq!(normalize_tag("get_users"), "get-users");

        // Test SCREAMING_SNAKE_CASE conversion
        assert_eq!(normalize_tag("USER_MANAGEMENT"), "user-management");
        assert_eq!(normalize_tag("API_KEY"), "api-key");

        // Test already kebab-case
        assert_eq!(normalize_tag("user-management"), "user-management");
        assert_eq!(normalize_tag("api-key"), "api-key");

        // Test single words
        assert_eq!(normalize_tag("users"), "users");
        assert_eq!(normalize_tag("API"), "api");

        // Test empty string
        assert_eq!(normalize_tag(""), "");

        // Test edge cases
        assert_eq!(normalize_tag("XMLHttpRequest"), "xml-http-request");
        assert_eq!(normalize_tag("HTTPSConnection"), "https-connection");

        // Test whitespace (heck handles spaces and trimming automatically)
        assert_eq!(normalize_tag("user management"), "user-management");
        assert_eq!(normalize_tag(" user "), "user"); // Leading/trailing spaces are trimmed

        // Test multiple separators - heck handles these well
        assert_eq!(normalize_tag("user__management"), "user-management");
        assert_eq!(normalize_tag("user---management"), "user-management");

        // Test special characters - heck handles these
        assert_eq!(normalize_tag("user123Management"), "user123-management");
        assert_eq!(normalize_tag("user@management"), "user-management"); // @ gets removed

        // Test numbers and mixed content
        assert_eq!(normalize_tag("v2ApiEndpoint"), "v2-api-endpoint");
        assert_eq!(normalize_tag("HTML5Parser"), "html5-parser");
    }
}
