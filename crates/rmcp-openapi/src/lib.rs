pub mod config;
pub mod error;
pub mod http_client;
pub mod security;
pub mod server;
pub mod spec;
pub mod tool;
pub mod tool_generator;
pub mod tool_registry;
pub mod transformer;

pub use config::{Authorization, AuthorizationMode};
pub use error::{CliError, Error, ToolCallError};
pub use http_client::{HttpClient, HttpResponse};
pub use security::SecurityObserver;
pub use server::Server;
pub use spec::Spec;
pub use tool::{Tool, ToolCollection, ToolMetadata};
pub use tool_generator::{ExtractedParameters, RequestConfig, ToolGenerator};
pub use tool_registry::{ToolRegistry, ToolRegistryStats};
pub use transformer::ResponseTransformer;

/// Normalize tag strings to kebab-case for consistent filtering
/// Converts any case format (camelCase, PascalCase, snake_case, etc.) to kebab-case
pub fn normalize_tag(tag: &str) -> String {
    use heck::ToKebabCase;
    tag.to_kebab_case()
}

#[cfg(test)]
mod tests {
    use super::*;

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
