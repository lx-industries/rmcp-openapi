use mockito::Server;
use url::Url;

/// Mock Petstore server for controlled testing
///
/// Core infrastructure only - test-specific mock methods are defined
/// directly in the test files that use them to avoid false positive
/// dead code warnings from cross-compilation unit dependencies.
pub struct MockPetstoreServer {
    pub server: mockito::Server,
}

impl MockPetstoreServer {
    /// Create a new mock server instance with a specific port
    pub async fn new_with_port(port: u16) -> Self {
        let opts = mockito::ServerOpts {
            port,
            ..Default::default()
        };
        let server = Server::new_with_opts_async(opts).await;
        Self { server }
    }

    /// Get the base URL for the mock server
    pub fn base_url(&self) -> Url {
        Url::parse(&self.server.url()).expect("Mock server URL should be valid")
    }
}

impl Drop for MockPetstoreServer {
    fn drop(&mut self) {
        // Server will be automatically cleaned up when dropped
    }
}

/// Mock Image server for testing image response handling
///
/// Core infrastructure only - test-specific mock methods are defined
/// directly in the test files that use them to avoid false positive
/// dead code warnings from cross-compilation unit dependencies.
pub struct MockImageServer {
    pub server: mockito::Server,
}

impl MockImageServer {
    /// Create a new mock server instance with a specific port
    pub async fn new_with_port(port: u16) -> Self {
        let opts = mockito::ServerOpts {
            port,
            ..Default::default()
        };
        let server = Server::new_with_opts_async(opts).await;
        Self { server }
    }

    /// Get the base URL for the mock server
    pub fn base_url(&self) -> Url {
        Url::parse(&self.server.url()).expect("Mock server URL should be valid")
    }
}

impl Drop for MockImageServer {
    fn drop(&mut self) {
        // Server will be automatically cleaned up when dropped
    }
}
