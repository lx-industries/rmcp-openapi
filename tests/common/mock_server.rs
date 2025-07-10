use mockito::Server;

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
    pub fn base_url(&self) -> String {
        self.server.url()
    }
}

impl Drop for MockPetstoreServer {
    fn drop(&mut self) {
        // Server will be automatically cleaned up when dropped
    }
}
