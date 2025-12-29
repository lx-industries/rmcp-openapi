use rmcp_openapi::{HttpClient, Tool, ToolMetadata, config::Authorization};
use serde_json::json;

mod common;
use common::mock_server::MockImageServer;
use mockito::Mock;

// ============================================================================
// Helper Functions - Test Image Data
// ============================================================================

/// Create minimal valid PNG image bytes (1x1 transparent pixel)
fn create_test_png_bytes() -> Vec<u8> {
    vec![
        // PNG signature
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // IHDR chunk
        0x00, 0x00, 0x00, 0x0D, // Length: 13 bytes
        0x49, 0x48, 0x44, 0x52, // "IHDR"
        0x00, 0x00, 0x00, 0x01, // Width: 1
        0x00, 0x00, 0x00, 0x01, // Height: 1
        0x08, 0x06, 0x00, 0x00, 0x00, // Bit depth, color type, compression, filter, interlace
        0x1F, 0x15, 0xC4, 0x89, // CRC
        // IDAT chunk (minimal data)
        0x00, 0x00, 0x00, 0x0A, // Length: 10 bytes
        0x49, 0x44, 0x41, 0x54, // "IDAT"
        0x78, 0x9C, 0x62, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
        0xB4, // CRC
        // IEND chunk
        0x00, 0x00, 0x00, 0x00, // Length: 0
        0x49, 0x45, 0x4E, 0x44, // "IEND"
        0xAE, 0x42, 0x60, 0x82, // CRC
    ]
}

/// Create minimal valid JPEG image bytes
fn create_test_jpeg_bytes() -> Vec<u8> {
    vec![
        // SOI marker
        0xFF, 0xD8, // APP0 marker (JFIF)
        0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00,
        0x01, 0x00, 0x00, // SOF0 marker (baseline DCT)
        0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
        // SOS marker (start of scan)
        0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, // Minimal scan data
        0xD2, 0x7F, // EOI marker
        0xFF, 0xD9,
    ]
}

/// Create minimal valid GIF image bytes (1x1 pixel)
fn create_test_gif_bytes() -> Vec<u8> {
    vec![
        // GIF header
        b'G', b'I', b'F', b'8', b'9', b'a', // Logical screen descriptor
        0x01, 0x00, // Width: 1
        0x01, 0x00, // Height: 1
        0x00, // Packed fields (no global color table)
        0x00, // Background color index
        0x00, // Pixel aspect ratio
        // Image descriptor
        0x2C, // Image separator
        0x00, 0x00, // Left
        0x00, 0x00, // Top
        0x01, 0x00, // Width: 1
        0x01, 0x00, // Height: 1
        0x00, // Packed fields
        // Image data
        0x02, // LZW minimum code size
        0x02, // Data sub-block length
        0x4C, 0x01, // Compressed data
        0x00, // Block terminator
        // Trailer
        0x3B, // GIF trailer
    ]
}

// ============================================================================
// Helper Functions - Tool Creation
// ============================================================================

/// Create a Tool instance for testing image endpoints
fn create_image_tool(mock_server: &MockImageServer, path: &str) -> anyhow::Result<Tool> {
    let metadata = ToolMetadata {
        name: "get_image".to_string(),
        title: None,
        description: Some("Get image from endpoint".to_string()),
        parameters: json!({"type": "object", "properties": {}}),
        output_schema: None,
        method: "GET".to_string(),
        path: path.to_string(),
        security: None,
        parameter_mappings: std::collections::HashMap::new(),
    };

    let http_client = HttpClient::new().with_base_url(mock_server.base_url())?;
    Ok(Tool::new(metadata, http_client)?)
}

/// Create a Tool instance for testing text endpoints
fn create_text_tool(mock_server: &MockImageServer, path: &str) -> anyhow::Result<Tool> {
    let metadata = ToolMetadata {
        name: "get_text".to_string(),
        title: None,
        description: Some("Get text from endpoint".to_string()),
        parameters: json!({"type": "object", "properties": {}}),
        output_schema: None,
        method: "GET".to_string(),
        path: path.to_string(),
        security: None,
        parameter_mappings: std::collections::HashMap::new(),
    };

    let http_client = HttpClient::new().with_base_url(mock_server.base_url())?;
    Ok(Tool::new(metadata, http_client)?)
}

/// Create a Tool instance for testing JSON endpoints
fn create_json_tool(mock_server: &MockImageServer, path: &str) -> anyhow::Result<Tool> {
    let metadata = ToolMetadata {
        name: "get_json".to_string(),
        title: None,
        description: Some("Get JSON from endpoint".to_string()),
        parameters: json!({"type": "object", "properties": {}}),
        output_schema: None,
        method: "GET".to_string(),
        path: path.to_string(),
        security: None,
        parameter_mappings: std::collections::HashMap::new(),
    };

    let http_client = HttpClient::new().with_base_url(mock_server.base_url())?;
    Ok(Tool::new(metadata, http_client)?)
}

// ============================================================================
// Mock Server Implementation
// ============================================================================

impl MockImageServer {
    /// Mock an image endpoint with specified content type and bytes
    pub fn mock_image_endpoint(&mut self, path: &str, content_type: &str, bytes: &[u8]) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(200)
            .with_header("content-type", content_type)
            .with_body(bytes)
            .create()
    }

    /// Mock a text endpoint
    pub fn mock_text_endpoint(&mut self, path: &str, text: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(text)
            .create()
    }

    /// Mock a JSON endpoint
    pub fn mock_json_endpoint(&mut self, path: &str, json: &serde_json::Value) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json.to_string())
            .create()
    }

    /// Mock a 404 not found response for an image
    pub fn mock_image_not_found(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"error": "Image not found"}).to_string())
            .create()
    }
}

// ============================================================================
// Core Image Format Tests
// ============================================================================

/// Test PNG image response handling
#[actix_web::test]
async fn test_png_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9201).await;

    let png_bytes = create_test_png_bytes();
    let _mock = mock_server.mock_image_endpoint("/image.png", "image/png", &png_bytes);

    let tool = create_image_tool(&mock_server, "/image.png").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    // Should have exactly one content item
    assert_eq!(call_result.content.len(), 1);

    // Content should be an image
    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            // Verify base64 encoding
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, png_bytes,
                "Decoded data should match original PNG bytes"
            );

            // Verify MIME type
            assert_eq!(&img.mime_type, "image/png");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    // Should not have structured content for images
    assert!(call_result.structured_content.is_none());

    // Should not be an error
    assert_eq!(call_result.is_error, Some(false));
}

/// Test JPEG image response handling
#[actix_web::test]
async fn test_jpeg_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9202).await;

    let jpeg_bytes = create_test_jpeg_bytes();
    let _mock = mock_server.mock_image_endpoint("/image.jpg", "image/jpeg", &jpeg_bytes);

    let tool = create_image_tool(&mock_server, "/image.jpg").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, jpeg_bytes,
                "Decoded data should match original JPEG bytes"
            );
            assert_eq!(&img.mime_type, "image/jpeg");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test GIF image response handling
#[actix_web::test]
async fn test_gif_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9203).await;

    let gif_bytes = create_test_gif_bytes();
    let _mock = mock_server.mock_image_endpoint("/image.gif", "image/gif", &gif_bytes);

    let tool = create_image_tool(&mock_server, "/image.gif").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, gif_bytes,
                "Decoded data should match original GIF bytes"
            );
            assert_eq!(&img.mime_type, "image/gif");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test WebP image response handling
#[actix_web::test]
async fn test_webp_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9204).await;

    // Create minimal WebP RIFF header
    let webp_bytes = vec![
        // RIFF header
        b'R', b'I', b'F', b'F', 0x1A, 0x00, 0x00, 0x00, // File size - 8 (26 bytes)
        b'W', b'E', b'B', b'P', // VP8 chunk
        b'V', b'P', b'8', b' ', 0x0E, 0x00, 0x00, 0x00, // Chunk size (14 bytes)
        // Minimal VP8 bitstream
        0x9D, 0x01, 0x2A, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let _mock = mock_server.mock_image_endpoint("/image.webp", "image/webp", &webp_bytes);

    let tool = create_image_tool(&mock_server, "/image.webp").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, webp_bytes,
                "Decoded data should match original WebP bytes"
            );
            assert_eq!(&img.mime_type, "image/webp");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test SVG+XML image response handling
#[actix_web::test]
async fn test_svg_xml_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9205).await;

    let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><rect width="1" height="1"/></svg>"#;
    let svg_bytes = svg_content.as_bytes();

    let _mock = mock_server.mock_image_endpoint("/image.svg", "image/svg+xml", svg_bytes);

    let tool = create_image_tool(&mock_server, "/image.svg").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, svg_bytes,
                "Decoded data should match original SVG bytes"
            );
            assert_eq!(&img.mime_type, "image/svg+xml");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test BMP image response handling
#[actix_web::test]
async fn test_bmp_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9206).await;

    // Create minimal 1x1 BMP
    let bmp_bytes = vec![
        // BMP Header
        b'B', b'M', // Signature
        0x46, 0x00, 0x00, 0x00, // File size: 70 bytes
        0x00, 0x00, 0x00, 0x00, // Reserved
        0x36, 0x00, 0x00, 0x00, // Pixel data offset: 54 bytes
        // DIB Header (BITMAPINFOHEADER)
        0x28, 0x00, 0x00, 0x00, // Header size: 40 bytes
        0x01, 0x00, 0x00, 0x00, // Width: 1 pixel
        0x01, 0x00, 0x00, 0x00, // Height: 1 pixel
        0x01, 0x00, // Color planes: 1
        0x18, 0x00, // Bits per pixel: 24
        0x00, 0x00, 0x00, 0x00, // Compression: none
        0x10, 0x00, 0x00, 0x00, // Image size: 16 bytes
        0x13, 0x0B, 0x00, 0x00, // Horizontal resolution
        0x13, 0x0B, 0x00, 0x00, // Vertical resolution
        0x00, 0x00, 0x00, 0x00, // Colors in palette: 0
        0x00, 0x00, 0x00, 0x00, // Important colors: 0
        // Pixel data (BGR format, 1x1 white pixel with padding)
        0xFF, 0xFF, 0xFF, 0x00,
    ];

    let _mock = mock_server.mock_image_endpoint("/image.bmp", "image/bmp", &bmp_bytes);

    let tool = create_image_tool(&mock_server, "/image.bmp").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, bmp_bytes,
                "Decoded data should match original BMP bytes"
            );
            assert_eq!(&img.mime_type, "image/bmp");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test image with charset parameter in Content-Type
#[actix_web::test]
async fn test_image_with_charset_parameter() {
    let mut mock_server = MockImageServer::new_with_port(9207).await;

    let png_bytes = create_test_png_bytes();
    // Mock with content-type including charset parameter
    let _mock =
        mock_server.mock_image_endpoint("/image.png", "image/png; charset=utf-8", &png_bytes);

    let tool = create_image_tool(&mock_server, "/image.png").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    // Should still be recognized as an image despite charset parameter
    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(decoded, png_bytes);
            // MIME type should include the full content-type as received
            assert_eq!(&img.mime_type, "image/png; charset=utf-8");
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test that text/plain responses are not converted to images
#[actix_web::test]
async fn test_text_response_not_converted() {
    let mut mock_server = MockImageServer::new_with_port(9208).await;

    let text_content = "This is plain text, not an image";
    let _mock = mock_server.mock_text_endpoint("/text.txt", text_content);

    let tool = create_text_tool(&mock_server, "/text.txt").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    // Should be Text content, not Image
    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Text(txt) => {
            assert!(
                txt.text.contains(text_content),
                "Should contain the text content"
            );
        }
        _ => panic!("Expected Text content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test that JSON responses are not converted to images
#[actix_web::test]
async fn test_json_response_not_converted() {
    let mut mock_server = MockImageServer::new_with_port(9209).await;

    let json_data = json!({"message": "This is JSON data", "type": "test"});
    let _mock = mock_server.mock_json_endpoint("/data.json", &json_data);

    let tool = create_json_tool(&mock_server, "/data.json").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    // Should be Text content (JSON serialized), not Image
    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Text(txt) => {
            assert!(
                txt.text.contains("This is JSON data"),
                "Should contain the JSON content"
            );
        }
        _ => panic!("Expected Text content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test error response (404) for image endpoint
#[actix_web::test]
async fn test_error_image_response_404() {
    let mut mock_server = MockImageServer::new_with_port(9210).await;

    let _mock = mock_server.mock_image_not_found("/missing.png");

    let tool = create_image_tool(&mock_server, "/missing.png").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    // Error responses should be marked as errors
    assert_eq!(call_result.is_error, Some(true));

    // Should have text content with error information
    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Text(txt) => {
            assert!(
                txt.text.contains("404") || txt.text.contains("Image not found"),
                "Error message should mention 404 or error details"
            );
        }
        _ => panic!(
            "Expected Text content for error, got: {:?}",
            call_result.content[0]
        ),
    }
}

/// Test base64 encoding correctness with known data
#[actix_web::test]
async fn test_base64_encoding_correctness() {
    let mut mock_server = MockImageServer::new_with_port(9211).await;

    // Use a simple known byte pattern
    let test_bytes = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    let _mock = mock_server.mock_image_endpoint("/test.png", "image/png", &test_bytes);

    let tool = create_image_tool(&mock_server, "/test.png").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            // Verify MIME type
            assert_eq!(&img.mime_type, "image/png");

            // Decode and verify byte-for-byte correctness
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(
                decoded, test_bytes,
                "Decoded bytes should exactly match original"
            );

            // Also verify the base64 string is valid RFC 4648
            let expected_base64 = STANDARD.encode(&test_bytes);
            assert_eq!(
                &img.data, &expected_base64,
                "Base64 encoding should match expected"
            );
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}

/// Test handling of empty image response
#[actix_web::test]
async fn test_empty_image_response() {
    let mut mock_server = MockImageServer::new_with_port(9212).await;

    let empty_bytes: Vec<u8> = vec![];
    let _mock = mock_server.mock_image_endpoint("/empty.png", "image/png", &empty_bytes);

    let tool = create_image_tool(&mock_server, "/empty.png").expect("Failed to create tool");
    let result = tool.call(&json!({}), Authorization::None, None).await;

    assert!(result.is_ok(), "Tool call should succeed");
    let call_result = result.unwrap();

    assert_eq!(call_result.content.len(), 1);

    // Even empty images should be returned as Image content
    use rmcp::model::RawContent;
    match &call_result.content[0].raw {
        RawContent::Image(img) => {
            assert_eq!(&img.mime_type, "image/png");

            // Empty data should produce empty base64 string
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let decoded = STANDARD.decode(&img.data).expect("Should be valid base64");
            assert_eq!(decoded.len(), 0, "Decoded empty image should have 0 bytes");
            assert_eq!(
                &img.data, "",
                "Base64 encoding of empty bytes should be empty string"
            );
        }
        _ => panic!("Expected Image content, got: {:?}", call_result.content[0]),
    }

    assert!(call_result.structured_content.is_none());
    assert_eq!(call_result.is_error, Some(false));
}
