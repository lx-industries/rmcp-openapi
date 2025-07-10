#!/bin/bash

# Petstore Demo for rmcp-openapi-server
# This script demonstrates using the OpenAPI MCP server with the Swagger Petstore API

set -e

echo "=== OpenAPI MCP Server - Petstore Demo ==="
echo

# Check if the binary is built
if [ ! -f "target/release/rmcp-openapi-server" ] && [ ! -f "target/debug/rmcp-openapi-server" ]; then
    echo "Building rmcp-openapi-server..."
    cargo build --release
fi

# Determine binary path
if [ -f "target/release/rmcp-openapi-server" ]; then
    BINARY="target/release/rmcp-openapi-server"
else
    BINARY="target/debug/rmcp-openapi-server"
fi

echo "Using binary: $BINARY"
echo

# Default usage with Petstore API
echo "1. Starting OpenAPI MCP Server with default Petstore API..."
echo "   Command: $BINARY"
echo
echo "   This will:"
echo "   - Load the Swagger Petstore OpenAPI spec from: https://petstore.swagger.io/v2/swagger.json"
echo "   - Generate MCP tools for all Petstore operations"
echo "   - Start an MCP server listening on stdio"
echo
echo "   Available tools will include:"
echo "   - addPet: Add a new pet to the store"
echo "   - findPetsByStatus: Finds pets by status"
echo "   - findPetsByTags: Finds pets by tags"
echo "   - getPetById: Find pet by ID"
echo "   - updatePet: Update an existing pet"
echo "   - deletePet: Deletes a pet"
echo "   - getInventory: Returns pet inventories by status"
echo "   - placeOrder: Place an order for a pet"
echo "   - getOrderById: Find purchase order by ID"
echo "   - deleteOrder: Delete purchase order by ID"
echo "   And more..."
echo

# Show help
echo "2. Command line options:"
echo "   $BINARY --help"
echo

# Example with local file
echo "3. Example with local OpenAPI spec file:"
echo "   $BINARY --spec ./tests/assets/petstore-openapi.json"
echo

# Example with custom base URL
echo "4. Example with custom base URL override:"
echo "   $BINARY --spec https://petstore.swagger.io/v2/swagger.json --base-url https://petstore.swagger.io/v2"
echo

# Example with verbose logging
echo "5. Example with verbose logging:"
echo "   $BINARY --verbose"
echo

echo "=== To test the server ==="
echo "Run the server in one terminal:"
echo "  $BINARY"
echo
echo "Then in another terminal, you can use an MCP client to interact with it."
echo "The server will expose all Petstore API operations as MCP tools."
echo

# Uncomment to actually run the server (for testing)
# echo "Starting server now (Ctrl+C to stop)..."
# exec "$BINARY"