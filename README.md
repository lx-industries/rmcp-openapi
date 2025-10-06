# rmcp-openapi

A Rust workspace providing OpenAPI to MCP (Model Context Protocol) conversion tools.

## Overview

This workspace contains two crates that work together to bridge OpenAPI specifications and the Model Context Protocol (MCP):

- **`rmcp-openapi`** (library): Core functionality for converting OpenAPI specifications to MCP tools
- **`rmcp-openapi-server`** (binary): MCP server executable that exposes OpenAPI endpoints as tools

This enables AI assistants to interact with REST APIs through a standardized interface.

## Features

- **Automatic Tool Generation**: Parse OpenAPI specifications and automatically generate MCP tools for all operations
- **Flexible Spec Loading**: Support for both URL-based and local file OpenAPI specifications
- **HTTP Client Integration**: Built-in HTTP client with configurable base URLs and request handling
- **Parameter Mapping**: Intelligent mapping of OpenAPI parameters (path, query, body) to MCP tool parameters
- **Output Schema Support**: Automatic generation of output schemas from OpenAPI response definitions
- **Structured Content**: Returns parsed JSON responses as structured content when output schemas are defined
- **Dual Usage Modes**: Use as a standalone MCP server or integrate as a Rust library
- **Transport Support**: StreamableHttp transport for MCP communication (default), with optional deprecated SSE transport
- **Comprehensive Testing**: Includes integration tests with JavaScript and Python MCP clients
- **Built with Official SDK**: Uses the official Rust MCP SDK for reliable protocol compliance
- **Authorization Header Handling**: Configurable authorization modes to balance MCP compliance with proxy requirements

## Security

`rmcp-openapi` acts as a proxy between MCP clients and OpenAPI services, which creates unique security considerations regarding authorization header handling.

**Important**: Please read the [SECURITY.md](./SECURITY.md) file for detailed information about:
- MCP specification compliance
- Authorization modes (compliant, passthrough-warn, passthrough-silent)
- Security implications and recommendations
- Compile-time feature flags for authorization passthrough

By default, the server operates in MCP-compliant mode and does not forward authorization headers. Non-compliant modes require explicit opt-in via compile-time features and runtime configuration.

## Contributing

We welcome contributions to `rmcp-openapi`! Please follow these guidelines:

### How to Contribute

1. **Fork the repository** on GitLab
2. **Create a feature branch** from `main`: `git checkout -b feature/my-new-feature`
3. **Make your changes** and ensure they follow the project's coding standards
4. **Add tests** for your changes if applicable
5. **Run the test suite** to ensure nothing is broken: `cargo test`
6. **Commit your changes** with clear, descriptive commit messages
7. **Push to your fork** and **create a merge request**

### Development Setup

```bash
# Clone your fork
git clone https://gitlab.com/your-username/rmcp-openapi.git
cd rmcp-openapi

# Build the project
cargo build --workspace

# Run tests
cargo test
```

### Code Standards

- Follow Rust conventions and use `cargo fmt` to format code
- Run `cargo clippy --all-targets` to catch common mistakes
- Add documentation for public APIs
- Include tests for new functionality

### Reporting Issues

Found a bug or have a feature request? Please report it on our [GitLab issue tracker](https://gitlab.com/lx-industries/rmcp-openapi/-/issues).

## Installation

### Install Server Binary
```bash
cargo install rmcp-openapi-server
```

### Build from Source
```bash
# Build entire workspace
cargo build --workspace --release

# Build specific crates
cargo build --package rmcp-openapi --release       # Library only
cargo build --package rmcp-openapi-server --release # Server only
```

### Using as a Library
Add to your `Cargo.toml`:
```toml
[dependencies]
rmcp-openapi = "0.8.2"
```

## Cargo Features

### Transport Features

- **`rustls-tls`** (default): Use rustls for TLS support
- **`native-tls`**: Use native TLS implementation
- **`transport-sse`**: Enable deprecated SSE (Server-Sent Events) transport for backward compatibility

### Security Features

- **`authorization-token-passthrough`**: Enable non-compliant authorization header forwarding (see SECURITY.md)

### Usage Examples

```toml
# Default configuration (StreamableHttp transport only)
[dependencies]
rmcp-openapi = "0.13.0"

# Enable deprecated SSE transport for backward compatibility
[dependencies]
rmcp-openapi = { version = "0.13.0", features = ["transport-sse"] }

# Server with SSE transport
[dependencies]
rmcp-openapi-server = { version = "0.13.0", features = ["transport-sse"] }
```

## Usage as a Library

### Basic Example
```rust
use rmcp_openapi::Server;
use serde_json::Value;
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load OpenAPI specification JSON (from URL, file, or embedded)
    let openapi_json: Value = {
        // Example: Load from embedded JSON string
        let spec_content = r#"{
            "openapi": "3.0.3",
            "info": {"title": "Pet Store", "version": "1.0.0"},
            "paths": {
                "/pets": {
                    "get": {
                        "operationId": "listPets",
                        "responses": {"200": {"description": "List of pets"}}
                    }
                }
            }
        }"#;
        serde_json::from_str(spec_content)?
        // In practice, you'd load from file or URL using your preferred method
    };

    // Create server with OpenAPI specification and base URL
    let mut server = Server::builder()
        .openapi_spec(openapi_json)
        .base_url(Url::parse("https://api.example.com")?)
        .build();

    // Parse the OpenAPI specification and generate tools
    server.load_openapi_spec()?;

    // Get information about generated tools
    println!("Generated {} tools", server.tool_count());
    println!("Available tools: {}", server.get_tool_names().join(", "));

    Ok(())
}
```

### Advanced Example with Custom Configuration
```rust
use rmcp_openapi::Server;
use reqwest::header::HeaderMap;
use serde_json::Value;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load OpenAPI specification from file (async I/O handled by your code)
    let openapi_json: Value = {
        let content = tokio::fs::read_to_string("./api-spec.json").await?;
        serde_json::from_str(&content)?
    };

    let base_url = Url::parse("https://api.example.com")?;

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Bearer token123".parse()?);

    // Create server with custom configuration using builder pattern
    let mut server = Server::builder()
        .openapi_spec(openapi_json)
        .base_url(base_url)
        .default_headers(headers)
        .tag_filter(Some(vec!["user".to_string(), "pets".to_string()]))
        .build();

    // Parse specification and generate tools
    server.load_openapi_spec()?;

    // Get tool information
    println!("Generated {} tools", server.tool_count());
    println!("Tool stats: {}", server.get_tool_stats());

    Ok(())
}
```

## Usage as an MCP Server

### Basic Usage

```bash
# Basic usage with Petstore API
rmcp-openapi-server https://petstore.swagger.io/v2/swagger.json

# See all available options
rmcp-openapi-server --help
```

### MCP Client Connection

The server exposes a StreamableHttp endpoint for MCP clients by default.

If you enable the deprecated `transport-sse` feature, an SSE endpoint is also available:

```
http://localhost:8080/sse
```

### Example with Claude Desktop

Add to your Claude Desktop MCP configuration:

```json
{
  "servers": {
    "petstore-api": {
      "command": "rmcp-openapi-server",
      "args": ["https://petstore.swagger.io/v2/swagger.json", "--port", "8080"]
    }
  }
}
```

### Example with JavaScript MCP Client (Deprecated SSE Transport)

**Note**: This example uses the deprecated SSE transport. Enable the `transport-sse` feature to use this.

```javascript
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { SseClientTransport } from '@modelcontextprotocol/sdk/client/sse.js';

const client = new Client(
  {
    name: "my-client",
    version: "1.0.0"
  },
  {
    capabilities: {}
  }
);

const transport = new SseClientTransport(
  new URL("http://localhost:8080/sse")
);

await client.connect(transport);

// List available tools
const tools = await client.listTools();
console.log("Available tools:", tools.tools.map(t => t.name));

// Call a tool
const result = await client.callTool({
  name: "getPetById",
  arguments: { petId: 123 }
});
```

### Generated Tools

The server automatically generates MCP tools for each OpenAPI operation:

- **Tool Names**: Uses `operationId` or generates from HTTP method + path
- **Parameters**: Maps OpenAPI parameters (path, query, body) to tool parameters
- **Descriptions**: Combines OpenAPI `summary` and `description` fields
- **Validation**: Includes parameter schemas for validation
- **Output Schemas**: Automatically generated from OpenAPI response definitions

Example generated tools for Petstore API:
- `addPet`: Add a new pet to the store
- `findPetsByStatus`: Find pets by status
- `getPetById`: Find pet by ID
- `updatePet`: Update an existing pet
- `deletePet`: Delete a pet

### Output Schema Support

The server now generates output schemas for all tools based on OpenAPI response definitions. This enables:

1. **Type-Safe Responses**: MCP clients can validate response data against the schema
2. **Structured Content**: When an output schema is defined, the server returns parsed JSON as `structured_content`
3. **Consistent Format**: All responses are wrapped in a standard structure:

```json
{
  "status": 200,           // HTTP status code
  "body": {                // Actual response data
    "id": 123,
    "name": "Fluffy",
    "status": "available"
  }
}
```

This wrapper ensures:
- All output schemas are objects (required by MCP)
- HTTP status codes are preserved
- Both success and error responses follow the same structure
- Clients can uniformly handle all responses

Example output schema for `getPetById`:
```json
{
  "type": "object",
  "properties": {
    "status": {
      "type": "integer",
      "description": "HTTP status code"
    },
    "body": {
      "type": "object",
      "properties": {
        "id": { "type": "integer", "format": "int64" },
        "name": { "type": "string" },
        "status": { "type": "string", "enum": ["available", "pending", "sold"] }
      }
    }
  },
  "required": ["status", "body"]
}
```

## Error Handling

The library distinguishes between two types of errors:

### Validation Errors (MCP Protocol Errors)
These occur before tool execution and are returned as MCP protocol errors:
- **ToolNotFound**: Requested tool doesn't exist (includes suggestions for similar tool names)
- **InvalidParameters**: Parameter validation failed (unknown names, missing required, constraint violations)
- **RequestConstructionError**: Failed to construct the HTTP request

### Execution Errors (Tool Output Errors)
These occur during tool execution and are returned as structured content in the tool response:
- **HttpError**: HTTP error response from the API (4xx, 5xx status codes)
- **NetworkError**: Network/connection failures (timeout, DNS, connection refused)
- **ResponseParsingError**: Failed to parse the response

### Error Response Format
For tools with output schemas, execution errors are wrapped in the standard response structure:
```json
{
  "status": 404,
  "body": {
    "error": {
      "type": "http-error",
      "status": 404,
      "message": "Pet not found"
    }
  }
}
```

Validation errors are returned as MCP protocol errors:
```json
{
  "code": -32602,
  "message": "Validation failed with 1 error",
  "data": {
    "type": "validation-errors",
    "violations": [
      {
        "type": "invalid-parameter",
        "parameter": "pet_id",
        "suggestions": ["petId"],
        "valid_parameters": ["petId", "status"]
      }
    ]
  }
}
```

The library provides clear, context-aware error messages for null values that help distinguish between nullable, optional, and required parameters:

**For required parameters with null values:**
```json
{
  "type": "constraint-violation",
  "parameter": "request_body",
  "message": "Parameter 'name' is required and must be non-null (expected: string)",
  "field_path": "request_body/name",
  "expected_type": "string"
}
```

**For optional parameters with null values:**
```json
{
  "type": "constraint-violation",
  "parameter": "request_body",
  "message": "Parameter 'status' must be string when provided (null not allowed, omit if not needed)",
  "field_path": "request_body/status",
  "expected_type": "string"
}
```

## Logging Configuration

The server uses structured logging with the `tracing` crate for comprehensive observability and debugging.

### Log Levels

Set the log level using the `RMCP_OPENAPI_LOG` environment variable:

```bash
# Info level (default for normal operation)
RMCP_OPENAPI_LOG=info rmcp-openapi-server https://petstore.swagger.io/v2/swagger.json

# Debug level (detailed operation info)
RMCP_OPENAPI_LOG=debug rmcp-openapi-server https://petstore.swagger.io/v2/swagger.json

# Trace level (very detailed debugging)
RMCP_OPENAPI_LOG=trace rmcp-openapi-server https://petstore.swagger.io/v2/swagger.json

# Or use the --verbose flag for debug level
rmcp-openapi-server --verbose https://petstore.swagger.io/v2/swagger.json
```

### Log Level Details

- **`error`**: Critical errors that need attention
- **`warn`**: Potential issues or warnings
- **`info`**: Important operational events (server startup, tool registration, HTTP request completion)
- **`debug`**: General debugging information (parameter extraction, tool lookup)
- **`trace`**: Very detailed debugging (detailed parameter parsing)

**Note**: Request and response bodies are never logged for security reasons.

### Structured Logging Format

Logs include structured fields for easy parsing and filtering:

```
2025-08-19T10:30:45.123Z INFO rmcp_openapi_server::main: OpenAPI MCP Server starting bind_address="127.0.0.1:8080"
2025-08-19T10:30:45.125Z INFO rmcp_openapi::server: Loaded tools from OpenAPI spec tool_count=12
2025-08-19T10:30:45.130Z INFO http_request{tool_name="getPetById" method="GET" path="/pet/{petId}"}: rmcp_openapi::http_client: HTTP request completed status=200 elapsed_ms=45
```

### Module-Specific Logging

You can control logging for specific modules:

```bash
# Only HTTP client debug logs
RMCP_OPENAPI_LOG=rmcp_openapi::http_client=debug rmcp-openapi-server spec.json

# Only server info logs, everything else warn
RMCP_OPENAPI_LOG=warn,rmcp_openapi::server=info rmcp-openapi-server spec.json

# Debug parameter extraction and tool generation
RMCP_OPENAPI_LOG=info,rmcp_openapi::tool_generator=debug rmcp-openapi-server spec.json
```

## Examples

See the `examples/` directory for usage examples:

- `petstore.sh`: Demonstrates server usage with the Swagger Petstore API

## Testing

```bash
# Run all tests
cargo test

# Run with live API testing
RMCP_TEST_LIVE_API=true cargo test

# Run specific integration tests
cargo test test_http_integration
```

## License

MIT License - see LICENSE file for details.
