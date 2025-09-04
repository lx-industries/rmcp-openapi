# Security Considerations

## Authorization Header Handling

The `rmcp-openapi` server acts as a bridge between MCP clients and OpenAPI services. This creates an architectural challenge regarding Authorization header handling.

### MCP Specification Compliance

According to the [MCP specification](https://modelcontextprotocol.io/docs/concepts/authentication):

> "Servers MUST NOT pass authentication tokens received in MCP requests through to subsequent HTTP requests when making external API calls"

This is designed to prevent the "confused deputy" vulnerability where a server might inadvertently use its own credentials on behalf of a client.

### Proxy Architecture Requirements

However, `rmcp-openapi` operates as a **proxy** rather than a traditional server:
- It doesn't have its own API credentials
- It relies on the client to provide appropriate authentication for the target OpenAPI service
- Without token passthrough, authenticated OpenAPI operations become inaccessible

## Authorization Modes

To balance security compliance with practical needs, `rmcp-openapi` provides three authorization modes:

### 1. Compliant Mode (Default)
- **Behavior**: No Authorization headers are passed through
- **MCP Spec**: ✅ Fully compliant
- **Use Case**: When accessing public APIs or using other authentication methods
- **Configuration**: `--authorization-mode compliant` or `RMCP_AUTHORIZATION_MODE=compliant`

### 2. Passthrough with Warnings
- **Behavior**: Authorization headers are passed through with debug/trace logging
- **MCP Spec**: ⚠️ Non-compliant (with awareness)
- **Use Case**: Development/debugging of authenticated APIs
- **Configuration**: `--authorization-mode passthrough-warn`
- **Requires**: Compile with `authorization-token-passthrough` feature

### 3. Silent Passthrough
- **Behavior**: Authorization headers are passed through silently
- **MCP Spec**: ❌ Non-compliant
- **Use Case**: Production proxy scenarios where passthrough is required
- **Configuration**: `--authorization-mode passthrough-silent`
- **Requires**: Compile with `authorization-token-passthrough` feature

## Compile-Time Feature Flag

The `authorization-token-passthrough` feature must be explicitly enabled at compile time to allow non-compliant modes:

```toml
# In Cargo.toml
rmcp-openapi = { version = "0.11.0", features = ["authorization-token-passthrough"] }
```

```bash
# Or via cargo command
cargo build --features authorization-token-passthrough
```

Without this feature, only `Compliant` mode is available, ensuring MCP specification compliance by default.

## Security Observability

The server provides comprehensive logging for authorization decisions:

1. **Startup Logging**: The configured authorization mode is logged when the server starts
2. **Request Logging**: Each request logs whether authorization was present and if the operation requires authentication (based on OpenAPI security definitions)
3. **Audit Trail**: All authorization decisions are traceable through structured logging

## Recommendations

1. **Default to Compliant Mode**: Use the default compliant mode unless you specifically need token passthrough
2. **Understand the Risks**: If using passthrough modes, ensure you understand the security implications
3. **Use Appropriate Mode**: 
   - Use `passthrough-warn` during development to maintain awareness
   - Use `passthrough-silent` in production only when necessary
4. **Monitor Logs**: Regularly review authorization logs for unexpected patterns
5. **Principle of Least Privilege**: Only enable passthrough for specific deployments that require it

## OpenAPI Security Awareness

The server extracts security requirements from the OpenAPI specification:
- Operations with security definitions are identified
- The `requires_auth()` check helps log which operations expect authentication
- This provides visibility into potential authentication failures

## Future Considerations

Potential future enhancements could include:
- Selective passthrough based on operation security requirements
- Token transformation or validation
- Integration with external authentication services
- Per-operation authorization policies

## Reporting Security Issues

If you discover a security vulnerability, please report it to the maintainers privately before public disclosure.