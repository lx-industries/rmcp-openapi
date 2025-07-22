from mcp import ClientSession, StdioServerParameters, types
from mcp.client.sse import sse_client
import sys
import json
import re

def clean_tool_response_text(response_data):
    """Remove headers from HTTP response text for deterministic testing"""
    if (isinstance(response_data, dict) and 
        'content' in response_data and 
        isinstance(response_data['content'], list) and 
        len(response_data['content']) > 0 and 
        'text' in response_data['content'][0]):
        
        text = response_data['content'][0]['text']
        # Remove the Headers section from the response text
        text = re.sub(r'\nHeaders:\n(?:.*\n)*?\nResponse Body:\n', '\nResponse Body:\n', text)
        
        response_data = response_data.copy()
        response_data['content'] = response_data['content'].copy()
        response_data['content'][0] = response_data['content'][0].copy()
        response_data['content'][0]['text'] = text
    
    return response_data

async def run():
    url = sys.argv[1]
    
    try:
        async with sse_client(url) as (read, write):
            async with ClientSession(read, write) as session:
                # Initialize the connection
                await session.initialize()

                # Step 1: List available tools
                tools = await session.list_tools()
                print(json.dumps({
                    "type": "tools_list",
                    "data": tools.model_dump()
                }))
                
                # Step 2: List other MCP resources for completeness
                resources = await session.list_resources()
                print(json.dumps({
                    "type": "resources_list",
                    "data": resources.model_dump()
                }))

                prompts = await session.list_prompts()
                print(json.dumps({
                    "type": "prompts_list", 
                    "data": prompts.model_dump()
                }))

                # Step 3: Test MCP Tool Calls - Path Parameter Test
                try:
                    get_pet_result = await session.call_tool(
                        name="getPetById",
                        arguments={
                            "petId": 123
                        }
                    )
                    result_data = get_pet_result.model_dump() if hasattr(get_pet_result, 'model_dump') else str(get_pet_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "getPetById",
                        "arguments": {"petId": 123},
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "getPetById",
                        "arguments": {"petId": 123},
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

                # Step 4: Test MCP Tool Calls - Query Parameter Test
                try:
                    find_pets_result = await session.call_tool(
                        name="findPetsByStatus",
                        arguments={
                            "status": ["available", "pending"]
                        }
                    )
                    result_data = find_pets_result.model_dump() if hasattr(find_pets_result, 'model_dump') else str(find_pets_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "findPetsByStatus",
                        "arguments": {"status": ["available", "pending"]},
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "findPetsByStatus",
                        "arguments": {"status": ["available", "pending"]},
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

                # Step 5: Test MCP Tool Calls - Request Body Test
                try:
                    add_pet_result = await session.call_tool(
                        name="addPet",
                        arguments={
                            "request_body": {
                                "name": "MCP Test Dog",
                                "category": {
                                    "id": 1,
                                    "name": "Dogs"
                                },
                                "photoUrls": ["https://example.com/mcp-test-dog.jpg"],
                                "tags": [
                                    {
                                        "id": 1,
                                        "name": "mcp-test"
                                    }
                                ],
                                "status": "available"
                            }
                        }
                    )
                    result_data = add_pet_result.model_dump() if hasattr(add_pet_result, 'model_dump') else str(add_pet_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "addPet",
                        "arguments": {
                            "request_body": {
                                "name": "MCP Test Dog",
                                "status": "available"
                            }
                        },
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "addPet",
                        "arguments": {
                            "request_body": {
                                "name": "MCP Test Dog",
                                "status": "available"
                            }
                        },
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

                # Step 6: Test Error Scenarios - Invalid Pet ID (404)
                try:
                    error_result = await session.call_tool(
                        name="getPetById",
                        arguments={
                            "petId": 999999
                        }
                    )
                    result_data = error_result.model_dump() if hasattr(error_result, 'model_dump') else str(error_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "getPetById",
                        "arguments": {"petId": 999999},
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "getPetById",
                        "arguments": {"petId": 999999},
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

                # Step 7: Test Error Scenarios - Invalid Request Body (400)
                try:
                    invalid_pet_result = await session.call_tool(
                        name="addPet",
                        arguments={
                            "request_body": {
                                # Missing required fields like 'name' and 'photoUrls'
                                "status": "invalid_status_value"
                            }
                        }
                    )
                    result_data = invalid_pet_result.model_dump() if hasattr(invalid_pet_result, 'model_dump') else str(invalid_pet_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "addPet",
                        "arguments": {"request_body": {"status": "invalid_status_value"}},
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "addPet",
                        "arguments": {"request_body": {"status": "invalid_status_value"}},
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

                # Step 8: Test Invalid Parameter Validation
                # Test with typo in parameter name (pet_id instead of petId)
                try:
                    invalid_param_result = await session.call_tool(
                        name="getPetById",
                        arguments={
                            "pet_id": 123  # Typo: should be petId
                        }
                    )
                    result_data = invalid_param_result.model_dump() if hasattr(invalid_param_result, 'model_dump') else str(invalid_param_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "getPetById",
                        "arguments": {"pet_id": 123},
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "getPetById",
                        "arguments": {"pet_id": 123},
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

                # Test with completely unknown parameter
                try:
                    unknown_param_result = await session.call_tool(
                        name="findPetsByStatus",
                        arguments={
                            "statuses": ["available"],  # Wrong parameter name
                            "limit": 10  # Extra unknown parameter
                        }
                    )
                    result_data = unknown_param_result.model_dump() if hasattr(unknown_param_result, 'model_dump') else str(unknown_param_result)
                    cleaned_data = clean_tool_response_text(result_data)
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "findPetsByStatus",
                        "arguments": {"statuses": ["available"], "limit": 10},
                        "success": True,
                        "data": cleaned_data
                    }))
                except Exception as error:
                    print(json.dumps({
                        "type": "tool_call_result",
                        "tool": "findPetsByStatus",
                        "arguments": {"statuses": ["available"], "limit": 10},
                        "success": False,
                        "error": {
                            "message": str(error),
                            "code": getattr(error, 'code', 'unknown')
                        }
                    }))

    except Exception as connection_error:
        print(json.dumps({
            "type": "connection_error",
            "error": {
                "message": str(connection_error),
                "code": getattr(connection_error, 'code', 'connection_failed')
            }
        }))

if __name__ == "__main__":
    import asyncio

    asyncio.run(run())
