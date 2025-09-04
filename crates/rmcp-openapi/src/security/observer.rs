use crate::config::Authorization;
use tracing::{debug, trace, warn};

/// Observes and logs security-related decisions
pub struct SecurityObserver<'a> {
    authorization: &'a Authorization,
}

impl<'a> SecurityObserver<'a> {
    /// Create a new security observer with the given authorization
    pub fn new(authorization: &'a Authorization) -> Self {
        Self { authorization }
    }

    /// Observe and log an authorization decision for a request
    pub fn observe_request(&self, operation_id: &str, has_auth: bool, requires_auth: bool) {
        match self.authorization {
            Authorization::None if has_auth => {
                debug!(
                    operation_id,
                    "Authorization header stripped (MCP-compliant mode)"
                );
            }
            #[cfg(feature = "authorization-token-passthrough")]
            Authorization::PassthroughWarn(_) if has_auth => {
                debug!(
                    operation_id,
                    "Forwarding Authorization header (passthrough mode)"
                );
            }
            #[cfg(feature = "authorization-token-passthrough")]
            Authorization::PassthroughSilent(_) => {
                trace!(operation_id, has_auth, "Processing request");
            }
            _ => {
                trace!(operation_id, has_auth, requires_auth, "Processing request");
            }
        }

        // Only warn when there's a potential issue
        if requires_auth && !has_auth {
            warn!(
                operation_id,
                "OpenAPI spec requires auth but no Authorization header present"
            );
        }
    }

    /// Log the authorization mode at startup
    pub fn log_startup(&self) {
        match self.authorization {
            Authorization::None => {
                tracing::info!("Authorization mode: compliant (headers will not be forwarded)");
            }
            #[cfg(feature = "authorization-token-passthrough")]
            Authorization::PassthroughWarn(_) => {
                tracing::warn!(
                    "Authorization mode: passthrough (non-MCP-compliant) - \
                     Authorization headers WILL be forwarded to backend APIs. See SECURITY.md"
                );
            }
            #[cfg(feature = "authorization-token-passthrough")]
            Authorization::PassthroughSilent(_) => {
                tracing::info!("Authorization mode: passthrough-silent");
            }
        }
    }
}
