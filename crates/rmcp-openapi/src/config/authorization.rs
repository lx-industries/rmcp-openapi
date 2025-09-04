use rmcp_actix_web::transport::AuthorizationHeader;
use std::str::FromStr;

/// Authorization handling for MCP server operations
///
/// This enum combines the authorization mode with the actual header value,
/// ensuring type safety and a cleaner API.
#[derive(Debug, Clone, Default)]
pub enum Authorization {
    /// No authorization header will be forwarded (MCP-compliant)
    #[default]
    None,

    /// Forward authorization with debug logging (requires feature flag)
    #[cfg(feature = "authorization-token-passthrough")]
    PassthroughWarn(Option<AuthorizationHeader>),

    /// Forward authorization silently (requires feature flag)
    #[cfg(feature = "authorization-token-passthrough")]
    PassthroughSilent(Option<AuthorizationHeader>),
}

/// Simple mode enum for conversion (matches CLI AuthorizationMode)
#[derive(Debug, Clone, Copy, Default)]
pub enum AuthorizationMode {
    #[default]
    Compliant,
    #[cfg(feature = "authorization-token-passthrough")]
    PassthroughWarn,
    #[cfg(feature = "authorization-token-passthrough")]
    PassthroughSilent,
}

impl FromStr for AuthorizationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compliant" => Ok(AuthorizationMode::Compliant),
            #[cfg(feature = "authorization-token-passthrough")]
            "passthrough-warn" => Ok(AuthorizationMode::PassthroughWarn),
            #[cfg(feature = "authorization-token-passthrough")]
            "passthrough-silent" => Ok(AuthorizationMode::PassthroughSilent),
            _ => {
                #[cfg(feature = "authorization-token-passthrough")]
                let valid = "compliant, passthrough-warn, passthrough-silent";
                #[cfg(not(feature = "authorization-token-passthrough"))]
                let valid = "compliant";
                Err(format!(
                    "Invalid authorization mode: '{}'. Valid values: {}",
                    s, valid
                ))
            }
        }
    }
}

impl Authorization {
    /// Create Authorization from a mode and optional header
    pub fn from_mode(
        mode: AuthorizationMode,
        #[cfg_attr(
            not(feature = "authorization-token-passthrough"),
            allow(unused_variables)
        )]
        header: Option<AuthorizationHeader>,
    ) -> Self {
        match mode {
            AuthorizationMode::Compliant => Authorization::None,
            #[cfg(feature = "authorization-token-passthrough")]
            AuthorizationMode::PassthroughWarn => Authorization::PassthroughWarn(header),
            #[cfg(feature = "authorization-token-passthrough")]
            AuthorizationMode::PassthroughSilent => Authorization::PassthroughSilent(header),
        }
    }
}
