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
- **Smart Parameter Handling**: Optional array parameters with empty values are automatically omitted from HTTP requests for OpenAPI compliance
- **Output Schema Support**: Automatic generation of output schemas from OpenAPI response definitions
- **Structured Content**: Returns parsed JSON responses as structured content when output schemas are defined
- **Dual Usage Modes**: Use as a standalone MCP server or integrate as a Rust library
- **Transport Support**: SSE (Server-Sent Events) transport for MCP communication
- **Comprehensive Testing**: Includes integration tests with JavaScript and Python MCP clients
- **Built with Official SDK**: Uses the official Rust MCP SDK for reliable protocol compliance

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

The server exposes an SSE endpoint for MCP clients:

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

### Example with JavaScript MCP Client

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

### Parameter Handling

The server implements intelligent parameter handling to ensure OpenAPI specification compliance:

#### Array Parameters
- **Empty Optional Arrays**: Optional array parameters with empty values (`[]`) are automatically omitted from HTTP requests
- **Non-Empty Optional Arrays**: Optional arrays with values are included normally
- **Required Arrays**: Required array parameters are always processed, even when empty
- **Arrays with Defaults**: Optional arrays with default values are always included, even when empty

#### Examples
```json
// These parameters...
{
  "requiredTags": [],           // Required array - included as "?requiredTags="
  "optionalTags": [],           // Optional array - omitted entirely  
  "optionalWithDefault": [],    // Optional with default - included as "?optionalWithDefault="
  "nonEmptyOptional": ["tag1"]  // Non-empty optional - included as "?nonEmptyOptional=tag1"
}

// ...generate this HTTP request:
// GET /endpoint?requiredTags=&optionalWithDefault=&nonEmptyOptional=tag1
```

This behavior ensures that HTTP requests conform to OpenAPI specifications where optional parameters should be omitted when not needed, while preserving required parameters and those with explicit defaults.

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