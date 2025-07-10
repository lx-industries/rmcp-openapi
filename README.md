# rmcp-openapi

Expose OpenAPI definition endpoints as MCP tools using the official Rust SDK for the Model Context Protocol.

## Overview

This project provides a bridge between OpenAPI specifications and the Model Context Protocol (MCP), allowing you to automatically generate MCP tools from OpenAPI definitions. This enables AI assistants to interact with REST APIs through a standardized interface.

## Features

- **Automatic Tool Generation**: Parse OpenAPI specifications and automatically generate MCP tools for all operations
- **Flexible Spec Loading**: Support for both URL-based and local file OpenAPI specifications
- **HTTP Client Integration**: Built-in HTTP client with configurable base URLs and request handling
- **Parameter Mapping**: Intelligent mapping of OpenAPI parameters (path, query, body) to MCP tool parameters
- **Dual Usage Modes**: Use as a standalone MCP server or integrate as a Rust library
- **Transport Support**: SSE (Server-Sent Events) transport for MCP communication
- **Comprehensive Testing**: Includes integration tests with JavaScript and Python MCP clients
- **Built with Official SDK**: Uses the official Rust MCP SDK for reliable protocol compliance

## Installation

### Install Binary
```bash
cargo install rmcp-openapi
```

### Build from Source
```bash
cargo build --release
```

### As a Library
Add to your `Cargo.toml`:
```toml
[dependencies]
rmcp-openapi = "0.1.0"
```

## Usage as a Library

### Basic Example
```rust
use rmcp_openapi::{OpenApiServer, OpenApiSpecLocation};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server from OpenAPI spec URL
    let spec_location = OpenApiSpecLocation::from("https://petstore.swagger.io/v2/swagger.json");
    let mut server = OpenApiServer::new(spec_location);
    
    // Load the OpenAPI specification
    server.load_openapi_spec().await?;
    
    // Get information about generated tools
    println!("Generated {} tools", server.tool_count());
    println!("Available tools: {}", server.get_tool_names().join(", "));
    
    Ok(())
}
```

### Advanced Example with Custom Base URL
```rust
use rmcp_openapi::{OpenApiServer, OpenApiSpecLocation, HttpClient};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec_location = OpenApiSpecLocation::from("./api-spec.json");
    let base_url = Url::parse("https://api.example.com")?;
    
    // Create server with custom base URL
    let mut server = OpenApiServer::with_base_url(spec_location, base_url)?;
    server.load_openapi_spec().await?;
    
    // Validate the registry
    server.validate_registry()?;
    
    // Get tool metadata
    if let Some(tool) = server.registry.get_tool("getUserById") {
        println!("Tool: {} - {}", tool.name, tool.description);
    }
    
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

Example generated tools for Petstore API:
- `addPet`: Add a new pet to the store
- `findPetsByStatus`: Find pets by status
- `getPetById`: Find pet by ID
- `updatePet`: Update an existing pet
- `deletePet`: Delete a pet

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