//! Dynamic tool filtering based on request context.
//!
//! This module provides the [`ToolFilter`] trait for controlling which tools are
//! visible and callable based on runtime context such as user permissions, scopes,
//! or other request-specific criteria.
//!
//! # Example
//!
//! ```rust,ignore
//! use rmcp_openapi::{ToolFilter, Tool};
//! use rmcp::service::{RequestContext, RoleServer};
//! use async_trait::async_trait;
//!
//! /// Filter that only allows read-only (GET) tools
//! struct ReadOnlyFilter;
//!
//! #[async_trait]
//! impl ToolFilter for ReadOnlyFilter {
//!     async fn allow(&self, tool: &Tool, _context: &RequestContext<RoleServer>) -> bool {
//!         tool.metadata.method == "GET"
//!     }
//! }
//! ```

use async_trait::async_trait;
use rmcp::service::{RequestContext, RoleServer};

use crate::tool::Tool;

/// Trait for dynamically filtering tools based on request context.
///
/// Implement this to control which tools are visible and callable
/// based on user permissions, scopes, or other runtime context.
///
/// # Usage
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use rmcp_openapi::{Server, ToolFilter};
///
/// let server = Server::builder()
///     .openapi_spec(spec)
///     .base_url(url)
///     .tool_filter(Arc::new(MyFilter::new()))
///     .build();
/// ```
///
/// # Behavior
///
/// - `list_tools`: Only returns tools where `allow` returns `true`
/// - `call_tool`: Returns "tool not found" error if filter rejects the tool
#[async_trait]
pub trait ToolFilter: Send + Sync {
    /// Returns true if the tool should be accessible in this context.
    ///
    /// Called for both `list_tools` (to filter visible tools) and
    /// `call_tool` (to enforce access control).
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to check access for
    /// * `context` - The request context containing extensions (e.g., user scopes)
    ///
    /// # Returns
    ///
    /// `true` if the tool should be accessible, `false` to hide/block it
    async fn allow(&self, tool: &Tool, context: &RequestContext<RoleServer>) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Filter that allows all tools
    struct AllowAll;

    #[async_trait]
    impl ToolFilter for AllowAll {
        async fn allow(&self, _tool: &Tool, _context: &RequestContext<RoleServer>) -> bool {
            true
        }
    }

    /// Filter that blocks all tools
    struct BlockAll;

    #[async_trait]
    impl ToolFilter for BlockAll {
        async fn allow(&self, _tool: &Tool, _context: &RequestContext<RoleServer>) -> bool {
            false
        }
    }

    /// Filter based on tool name prefix
    struct PrefixFilter {
        allowed_prefix: String,
    }

    #[async_trait]
    impl ToolFilter for PrefixFilter {
        async fn allow(&self, tool: &Tool, _context: &RequestContext<RoleServer>) -> bool {
            tool.metadata.name.starts_with(&self.allowed_prefix)
        }
    }

    #[test]
    fn test_trait_is_object_safe() {
        // Verify the trait can be used with dynamic dispatch
        fn accepts_filter(_filter: &dyn ToolFilter) {}
        fn accepts_arc_filter(_filter: Arc<dyn ToolFilter>) {}

        let allow_all = AllowAll;
        let block_all = BlockAll;

        accepts_filter(&allow_all);
        accepts_filter(&block_all);
        accepts_arc_filter(Arc::new(AllowAll));
        accepts_arc_filter(Arc::new(BlockAll));
    }

    #[test]
    fn test_filter_can_be_cloned_via_arc() {
        // Verify Arc<dyn ToolFilter> can be cloned (important for Server which derives Clone)
        let filter: Arc<dyn ToolFilter> = Arc::new(AllowAll);
        let _cloned = filter.clone();
    }

    #[test]
    fn test_prefix_filter_can_be_constructed() {
        // Verify PrefixFilter can be used as a ToolFilter
        let filter: Arc<dyn ToolFilter> = Arc::new(PrefixFilter {
            allowed_prefix: "get".to_string(),
        });
        let _cloned = filter.clone();
    }
}
