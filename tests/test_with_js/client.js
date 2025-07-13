import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { SSEClientTransport } from "@modelcontextprotocol/sdk/client/sse.js";

// Helper function to clean tool response text by removing headers section
function cleanToolResponseText(responseData) {
  if (responseData && responseData.content && responseData.content[0] && responseData.content[0].text) {
    let text = responseData.content[0].text;
    // Remove the Headers section from the response text
    text = text.replace(/\nHeaders:\n(?:.*\n)*?\nResponse Body:\n/, "\nResponse Body:\n");
    return {
      ...responseData,
      content: [{
        ...responseData.content[0],
        text: text
      }]
    };
  }
  return responseData;
}

const transport = new SSEClientTransport(new URL(process.env.MCP_SSE_URL || `http://127.0.0.1:8000/sse`));

const client = new Client(
  {
    name: "example-client",
    version: "1.0.0"
  },
  {
    capabilities: {
      prompts: {},
      resources: {},
      tools: {}
    }
  }
);

try {
  await client.connect(transport);
  
  // Step 1: List available tools
  const tools = await client.listTools();
  console.log(JSON.stringify({
    type: "tools_list",
    data: tools
  }));
  
  // Step 2: List other MCP resources for completeness
  const resources = await client.listResources();
  console.log(JSON.stringify({
    type: "resources_list", 
    data: resources
  }));
  
  const templates = await client.listResourceTemplates();
  console.log(JSON.stringify({
    type: "resource_templates_list",
    data: templates
  }));
  
  const prompts = await client.listPrompts();
  console.log(JSON.stringify({
    type: "prompts_list",
    data: prompts
  }));
  
  // Step 3: Test MCP Tool Calls - Path Parameter Test
  try {
    const getPetResult = await client.callTool({
      name: "getPetById",
      arguments: {
        petId: 123
      }
    });
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "getPetById",
      arguments: { petId: 123 },
      success: true,
      data: cleanToolResponseText(getPetResult)
    }));
  } catch (error) {
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "getPetById", 
      arguments: { petId: 123 },
      success: false,
      error: {
        message: error.message,
        code: error.code || "unknown"
      }
    }));
  }
  
  // Step 4: Test MCP Tool Calls - Query Parameter Test
  try {
    const findPetsResult = await client.callTool({
      name: "findPetsByStatus",
      arguments: {
        status: ["available", "pending"]
      }
    });
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "findPetsByStatus",
      arguments: { status: ["available", "pending"] },
      success: true,
      data: cleanToolResponseText(findPetsResult)
    }));
  } catch (error) {
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "findPetsByStatus",
      arguments: { status: ["available", "pending"] },
      success: false,
      error: {
        message: error.message,
        code: error.code || "unknown"
      }
    }));
  }
  
  // Step 5: Test MCP Tool Calls - Request Body Test
  try {
    const addPetResult = await client.callTool({
      name: "addPet",
      arguments: {
        request_body: {
          name: "MCP Test Dog",
          category: {
            id: 1,
            name: "Dogs"
          },
          photoUrls: ["https://example.com/mcp-test-dog.jpg"],
          tags: [
            {
              id: 1,
              name: "mcp-test"
            }
          ],
          status: "available"
        }
      }
    });
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "addPet",
      arguments: {
        request_body: {
          name: "MCP Test Dog",
          status: "available"
        }
      },
      success: true,
      data: cleanToolResponseText(addPetResult)
    }));
  } catch (error) {
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "addPet",
      arguments: {
        request_body: {
          name: "MCP Test Dog",
          status: "available"
        }
      },
      success: false,
      error: {
        message: error.message,
        code: error.code || "unknown"
      }
    }));
  }
  
  // Step 6: Test Error Scenarios - Invalid Pet ID (404)
  try {
    const errorResult = await client.callTool({
      name: "getPetById",
      arguments: {
        petId: 999999
      }
    });
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "getPetById",
      arguments: { petId: 999999 },
      success: true,
      data: cleanToolResponseText(errorResult)
    }));
  } catch (error) {
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "getPetById",
      arguments: { petId: 999999 },
      success: false,
      error: {
        message: error.message,
        code: error.code || "unknown"
      }
    }));
  }
  
  // Step 7: Test Error Scenarios - Invalid Request Body (400)
  try {
    const invalidPetResult = await client.callTool({
      name: "addPet",
      arguments: {
        request_body: {
          // Missing required fields like 'name' and 'photoUrls'
          status: "invalid_status_value"
        }
      }
    });
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "addPet",
      arguments: { request_body: { status: "invalid_status_value" } },
      success: true,
      data: cleanToolResponseText(invalidPetResult)
    }));
  } catch (error) {
    console.log(JSON.stringify({
      type: "tool_call_result",
      tool: "addPet",
      arguments: { request_body: { status: "invalid_status_value" } },
      success: false,
      error: {
        message: error.message,
        code: error.code || "unknown"
      }
    }));
  }

} catch (connectionError) {
  console.log(JSON.stringify({
    type: "connection_error",
    error: {
      message: connectionError.message,
      code: connectionError.code || "connection_failed"
    }
  }));
} finally {
  try {
    await client.close();
    await transport.close();
  } catch (closeError) {
    // Ignore close errors
  }
}