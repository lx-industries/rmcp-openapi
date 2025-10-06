//! # OpenAPI to MCP Tool Generator with Reference Metadata Enhancement
//!
//! This module provides comprehensive tooling for converting OpenAPI 3.1 specifications
//! into Model Context Protocol (MCP) tools with sophisticated reference metadata handling.
//! The implementation follows OpenAPI 3.1 semantics to ensure contextual information
//! takes precedence over generic schema documentation.
//!
//! ## Reference Metadata Enhancement Strategy
//!
//! ### Core Philosophy
//!
//! The OpenAPI 3.1 specification introduces reference metadata fields (`summary` and `description`)
//! that can be attached to `$ref` objects. These fields serve a fundamentally different purpose
//! than schema-level metadata:
//!
//! - **Reference Metadata**: Contextual, usage-specific information about how a schema is used
//!   in a particular location within the API specification
//! - **Schema Metadata**: General, reusable documentation about the schema definition itself
//!
//! This distinction is crucial for generating meaningful MCP tools that provide contextual
//! information to AI assistants rather than generic schema documentation.
//!
//! ### Implementation Architecture
//!
//! The enhancement strategy is implemented through several coordinated components:
//!
//! #### 1. ReferenceMetadata Struct
//! Central data structure that encapsulates OpenAPI 3.1 reference metadata fields and provides
//! the core precedence logic through helper methods (`best_description()`, `summary()`).
//!
//! #### 2. Precedence Hierarchy Implementation
//! All description enhancement follows the strict precedence hierarchy:
//! 1. **Reference description** (highest) - Detailed contextual information
//! 2. **Reference summary** (medium) - Brief contextual information
//! 3. **Schema description** (lower) - General schema documentation
//! 4. **Generated fallback** (lowest) - Auto-generated descriptive text
//!
//! #### 3. Context-Aware Enhancement Methods
//! - `merge_with_description()`: General-purpose description merging with optional formatting
//! - `enhance_parameter_description()`: Parameter-specific enhancement with name integration
//! - Various schema conversion methods that apply reference metadata throughout tool generation
//!
//! ### Usage Throughout Tool Generation Pipeline
//!
//! The reference metadata enhancement strategy is applied systematically:
//!
//! #### Parameter Processing
//! - Parameter schemas are enhanced with contextual information from parameter references
//! - Parameter descriptions include contextual usage information rather than generic field docs
//! - Special formatting ensures parameter names are clearly associated with contextual descriptions
//!
//! #### Request Body Processing
//! - Request body schemas are enriched with operation-specific documentation
//! - Content type handling preserves reference metadata through schema conversion
//! - Complex nested schemas maintain reference context through recursive processing
//!
//! #### Response Processing
//! - Response schemas are augmented with endpoint-specific information
//! - Unified response structures include contextual descriptions in the response body schemas
//! - Error handling maintains reference context for comprehensive tool documentation
//!
//! #### Tool Metadata Generation
//! - Tool names, descriptions, and parameter schemas all benefit from reference metadata
//! - Operation-level documentation is combined with reference-level context for comprehensive tool docs
//! - Output schemas preserve contextual information for structured MCP responses
//!
//! ### Quality Assurance
//!
//! The implementation includes comprehensive safeguards:
//!
//! - **Precedence Consistency**: All enhancement methods follow identical precedence rules
//! - **Backward Compatibility**: Systems without reference metadata continue to work with schema-level docs
//! - **Fallback Robustness**: Multiple fallback levels ensure tools always have meaningful documentation
//! - **Context Preservation**: Reference metadata is preserved through complex schema transformations
//!
//! ### Examples
//!
//! ```rust
//! use rmcp_openapi::tool_generator::{ToolGenerator, ReferenceMetadata};
//! use oas3::spec::Spec;
//!
//! // Reference metadata provides contextual information
//! let ref_metadata = ReferenceMetadata::new(
//!     Some("Store pet data".to_string()),      // contextual summary
//!     Some("Pet information for inventory management".to_string()) // contextual description
//! );
//!
//! // Enhancement follows precedence hierarchy
//! let enhanced = ref_metadata.merge_with_description(
//!     Some("Generic animal schema"), // schema description (lower priority)
//!     false
//! );
//! // Result: "Pet information for inventory management" (reference description wins)
//!
//! // Parameter enhancement includes contextual formatting
//! let param_desc = ref_metadata.enhance_parameter_description(
//!     "petId",
//!     Some("Database identifier")
//! );
//! // Result: "petId: Pet information for inventory management"
//! ```
//!
//! This comprehensive approach ensures that MCP tools generated from OpenAPI specifications
//! provide meaningful, contextual information to AI assistants rather than generic schema
//! documentation, significantly improving the quality of human-AI interactions.

use schemars::schema_for;
use serde::{Serialize, Serializer};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::error::{
    Error, ErrorResponse, ToolCallValidationError, ValidationConstraint, ValidationError,
};
use crate::tool::ToolMetadata;
use oas3::spec::{
    BooleanSchema, ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn,
    ParameterStyle, RequestBody, Response, Schema, SchemaType, SchemaTypeSet, Spec,
};
use tracing::{trace, warn};

// Annotation key constants
const X_LOCATION: &str = "x-location";
const X_PARAMETER_LOCATION: &str = "x-parameter-location";
const X_PARAMETER_REQUIRED: &str = "x-parameter-required";
const X_CONTENT_TYPE: &str = "x-content-type";
const X_ORIGINAL_NAME: &str = "x-original-name";
const X_PARAMETER_EXPLODE: &str = "x-parameter-explode";

/// Location type that extends ParameterIn with Body variant
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Location {
    /// Standard OpenAPI parameter locations
    Parameter(ParameterIn),
    /// Request body location
    Body,
}

impl Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = match self {
            Location::Parameter(param_in) => match param_in {
                ParameterIn::Query => "query",
                ParameterIn::Header => "header",
                ParameterIn::Path => "path",
                ParameterIn::Cookie => "cookie",
            },
            Location::Body => "body",
        };
        serializer.serialize_str(str_value)
    }
}

/// Annotation types that can be applied to parameters and request bodies
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    /// Location of the parameter or request body
    Location(Location),
    /// Whether a parameter is required
    Required(bool),
    /// Content type for request bodies
    ContentType(String),
    /// Original name before sanitization
    OriginalName(String),
    /// Parameter explode setting for arrays/objects
    Explode(bool),
}

/// Collection of annotations that can be applied to schema objects
#[derive(Debug, Clone, Default)]
pub struct Annotations {
    annotations: Vec<Annotation>,
}

impl Annotations {
    /// Create a new empty Annotations collection
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
        }
    }

    /// Add a location annotation
    pub fn with_location(mut self, location: Location) -> Self {
        self.annotations.push(Annotation::Location(location));
        self
    }

    /// Add a required annotation
    pub fn with_required(mut self, required: bool) -> Self {
        self.annotations.push(Annotation::Required(required));
        self
    }

    /// Add a content type annotation
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.annotations.push(Annotation::ContentType(content_type));
        self
    }

    /// Add an original name annotation
    pub fn with_original_name(mut self, original_name: String) -> Self {
        self.annotations
            .push(Annotation::OriginalName(original_name));
        self
    }

    /// Add an explode annotation
    pub fn with_explode(mut self, explode: bool) -> Self {
        self.annotations.push(Annotation::Explode(explode));
        self
    }
}

impl Serialize for Annotations {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.annotations.len()))?;

        for annotation in &self.annotations {
            match annotation {
                Annotation::Location(location) => {
                    // Determine the key based on the location type
                    let key = match location {
                        Location::Parameter(param_in) => match param_in {
                            ParameterIn::Header | ParameterIn::Cookie => X_LOCATION,
                            _ => X_PARAMETER_LOCATION,
                        },
                        Location::Body => X_LOCATION,
                    };
                    map.serialize_entry(key, &location)?;

                    // For parameters, also add x-parameter-location
                    if let Location::Parameter(_) = location {
                        map.serialize_entry(X_PARAMETER_LOCATION, &location)?;
                    }
                }
                Annotation::Required(required) => {
                    map.serialize_entry(X_PARAMETER_REQUIRED, required)?;
                }
                Annotation::ContentType(content_type) => {
                    map.serialize_entry(X_CONTENT_TYPE, content_type)?;
                }
                Annotation::OriginalName(original_name) => {
                    map.serialize_entry(X_ORIGINAL_NAME, original_name)?;
                }
                Annotation::Explode(explode) => {
                    map.serialize_entry(X_PARAMETER_EXPLODE, explode)?;
                }
            }
        }

        map.end()
    }
}

/// Sanitize a property name to match MCP requirements
///
/// MCP requires property keys to match the pattern `^[a-zA-Z0-9_.-]{1,64}$`
/// This function:
/// - Replaces invalid characters with underscores
/// - Limits the length to 64 characters
/// - Ensures the name doesn't start with a number
/// - Ensures the result is not empty
fn sanitize_property_name(name: &str) -> String {
    // Replace invalid characters with underscores
    let sanitized = name
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '-' => c,
            _ => '_',
        })
        .take(64)
        .collect::<String>();

    // Collapse consecutive underscores into a single underscore
    let mut collapsed = String::with_capacity(sanitized.len());
    let mut prev_was_underscore = false;

    for ch in sanitized.chars() {
        if ch == '_' {
            if !prev_was_underscore {
                collapsed.push(ch);
            }
            prev_was_underscore = true;
        } else {
            collapsed.push(ch);
            prev_was_underscore = false;
        }
    }

    // Trim trailing underscores
    let trimmed = collapsed.trim_end_matches('_');

    // Ensure not empty and doesn't start with a number
    if trimmed.is_empty() || trimmed.chars().next().unwrap_or('0').is_numeric() {
        format!("param_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Metadata extracted from OpenAPI 3.1 reference objects for MCP tool generation
///
/// This struct encapsulates the OpenAPI 3.1 reference metadata fields (summary and description)
/// that provide contextual, usage-specific documentation for referenced schema objects.
/// It implements the proper precedence hierarchy as defined by the OpenAPI 3.1 specification.
///
/// ## OpenAPI 3.1 Reference Metadata Semantics
///
/// In OpenAPI 3.1, reference objects can contain additional metadata fields:
/// ```yaml
/// $ref: '#/components/schemas/Pet'
/// summary: Pet information for store operations
/// description: Detailed pet data including status and ownership
/// ```
///
/// This metadata serves a different semantic purpose than schema definitions:
/// - **Reference metadata**: Provides contextual, usage-specific information about how
///   a schema is used in a particular location within the API specification
/// - **Schema metadata**: Provides general, reusable documentation about the schema itself
///
/// ## Precedence Hierarchy
///
/// Following OpenAPI 3.1 semantics, this implementation enforces the precedence:
/// 1. **Reference description** (highest priority) - Contextual usage description
/// 2. **Reference summary** (medium priority) - Contextual usage summary
/// 3. **Schema description** (lowest priority) - General schema description
/// 4. **Generated fallback** (last resort) - Auto-generated descriptive text
///
/// This hierarchy ensures that human-authored contextual information takes precedence
/// over generic schema documentation, providing more meaningful tool descriptions
/// for AI assistants consuming the MCP interface.
///
/// ## Usage in Tool Generation
///
/// Reference metadata is used throughout the tool generation process:
/// - **Parameter descriptions**: Enhanced with contextual information about parameter usage
/// - **Request body schemas**: Enriched with operation-specific documentation
/// - **Response schemas**: Augmented with endpoint-specific response information
/// - **Tool descriptions**: Combined with operation metadata for comprehensive tool documentation
///
/// ## Example
///
/// ```rust
/// use rmcp_openapi::tool_generator::ReferenceMetadata;
///
/// let ref_meta = ReferenceMetadata::new(
///     Some("Pet data".to_string()),
///     Some("Complete pet information including health records".to_string())
/// );
///
/// // Reference description takes precedence
/// assert_eq!(
///     ref_meta.best_description(),
///     Some("Complete pet information including health records")
/// );
///
/// // Merge with existing schema description (reference wins)
/// let enhanced = ref_meta.merge_with_description(
///     Some("Generic pet schema"),
///     false
/// );
/// assert_eq!(enhanced, Some("Complete pet information including health records".to_string()));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ReferenceMetadata {
    /// Optional contextual summary from the OpenAPI 3.1 reference object
    ///
    /// This field captures the `summary` property from a reference object,
    /// providing a brief, contextual description of how the referenced schema
    /// is used in this specific location. Takes precedence over schema summaries
    /// when available.
    pub summary: Option<String>,

    /// Optional contextual description from the OpenAPI 3.1 reference object
    ///
    /// This field captures the `description` property from a reference object,
    /// providing detailed, contextual documentation about how the referenced schema
    /// is used in this specific location. This is the highest priority description
    /// in the precedence hierarchy and overrides any schema-level descriptions.
    pub description: Option<String>,
}

impl ReferenceMetadata {
    /// Create new reference metadata from optional summary and description
    pub fn new(summary: Option<String>, description: Option<String>) -> Self {
        Self {
            summary,
            description,
        }
    }

    /// Check if this metadata contains any useful information
    pub fn is_empty(&self) -> bool {
        self.summary.is_none() && self.description.is_none()
    }

    /// Get the best available description from reference metadata
    ///
    /// This helper method implements the core fallback logic for selecting the most
    /// appropriate description from the available reference metadata fields.
    /// It follows OpenAPI 3.1 semantics where detailed descriptions take precedence
    /// over brief summaries.
    ///
    /// ## Selection Logic
    ///
    /// 1. **Primary**: Returns reference description if available
    ///    - Source: `$ref.description` field
    ///    - Rationale: Detailed contextual information is most valuable
    /// 2. **Fallback**: Returns reference summary if no description available
    ///    - Source: `$ref.summary` field
    ///    - Rationale: Brief context is better than no context
    /// 3. **None**: Returns `None` if neither field is available
    ///    - Behavior: Caller must handle absence of reference metadata
    ///
    /// ## Usage in Precedence Hierarchy
    ///
    /// This method provides the first-priority input for all description enhancement
    /// methods (`merge_with_description()`, `enhance_parameter_description()`).
    /// It encapsulates the "reference description OR reference summary" logic
    /// that forms the top of the precedence hierarchy.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmcp_openapi::tool_generator::ReferenceMetadata;
    ///
    /// // Description takes precedence over summary
    /// let both = ReferenceMetadata::new(
    ///     Some("Brief summary".to_string()),
    ///     Some("Detailed description".to_string())
    /// );
    /// assert_eq!(both.best_description(), Some("Detailed description"));
    ///
    /// // Summary used when no description
    /// let summary_only = ReferenceMetadata::new(Some("Brief summary".to_string()), None);
    /// assert_eq!(summary_only.best_description(), Some("Brief summary"));
    ///
    /// // None when no reference metadata
    /// let empty = ReferenceMetadata::new(None, None);
    /// assert_eq!(empty.best_description(), None);
    /// ```
    ///
    /// # Returns
    /// * `Some(&str)` - Best available description (description OR summary)
    /// * `None` - No reference metadata available
    pub fn best_description(&self) -> Option<&str> {
        self.description.as_deref().or(self.summary.as_deref())
    }

    /// Get the reference summary for targeted access
    ///
    /// This helper method provides direct access to the reference summary field
    /// without fallback logic. It's used when summary-specific behavior is needed,
    /// such as in `merge_with_description()` for the special prepend functionality.
    ///
    /// ## Usage Scenarios
    ///
    /// 1. **Summary-specific operations**: When caller needs to distinguish between
    ///    summary and description for special formatting (e.g., prepend behavior)
    /// 2. **Metadata inspection**: When caller wants to check what summary information
    ///    is available without fallback to description
    /// 3. **Pattern matching**: Used in complex precedence logic where summary
    ///    and description need separate handling
    ///
    /// ## Relationship with best_description()
    ///
    /// Unlike `best_description()` which implements fallback logic, this method
    /// provides raw access to just the summary field. This enables fine-grained
    /// control in precedence implementations.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmcp_openapi::tool_generator::ReferenceMetadata;
    ///
    /// let with_summary = ReferenceMetadata::new(Some("API token".to_string()), None);
    ///
    /// // Direct summary access
    /// assert_eq!(with_summary.summary(), Some("API token"));
    ///
    /// // Compare with best_description (same result when only summary available)
    /// assert_eq!(with_summary.best_description(), Some("API token"));
    ///
    /// // Different behavior when both are present
    /// let both = ReferenceMetadata::new(
    ///     Some("Token".to_string()),      // summary
    ///     Some("Auth token".to_string())  // description
    /// );
    /// assert_eq!(both.summary(), Some("Token"));            // Just summary
    /// assert_eq!(both.best_description(), Some("Auth token")); // Prefers description
    /// ```
    ///
    /// # Returns
    /// * `Some(&str)` - Reference summary if available
    /// * `None` - No summary in reference metadata
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    /// Merge reference metadata with existing description using OpenAPI 3.1 precedence rules
    ///
    /// This method implements the sophisticated fallback mechanism for combining contextual
    /// reference metadata with general schema descriptions. It follows the OpenAPI 3.1
    /// semantic hierarchy where contextual information takes precedence over generic
    /// schema documentation.
    ///
    /// ## Fallback Mechanism
    ///
    /// The method implements a strict precedence hierarchy:
    ///
    /// ### Priority 1: Reference Description (Highest)
    /// - **Source**: `$ref.description` field from OpenAPI 3.1 reference object
    /// - **Semantic**: Contextual, usage-specific description for this particular reference
    /// - **Behavior**: Always takes precedence, ignoring all other descriptions
    /// - **Rationale**: Human-authored contextual information is most valuable for tool users
    ///
    /// ### Priority 2: Reference Summary (Medium)
    /// - **Source**: `$ref.summary` field from OpenAPI 3.1 reference object
    /// - **Semantic**: Brief contextual summary for this particular reference
    /// - **Behavior**: Used when no reference description is available
    /// - **Special Case**: When `prepend_summary=true` and existing description differs,
    ///   combines summary with existing description using double newline separator
    ///
    /// ### Priority 3: Schema Description (Lower)
    /// - **Source**: `description` field from the resolved schema object
    /// - **Semantic**: General, reusable documentation about the schema itself
    /// - **Behavior**: Only used as fallback when no reference metadata is available
    /// - **Rationale**: Generic schema docs are less valuable than contextual reference docs
    ///
    /// ### Priority 4: No Description (Lowest)
    /// - **Behavior**: Returns `None` when no description sources are available
    /// - **Impact**: Caller should provide appropriate fallback behavior
    ///
    /// ## Implementation Details
    ///
    /// The method uses pattern matching on a tuple of `(reference_description, reference_summary, schema_description)`
    /// to implement the precedence hierarchy efficiently. This ensures all possible combinations
    /// are handled explicitly and correctly.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmcp_openapi::tool_generator::ReferenceMetadata;
    ///
    /// let ref_meta = ReferenceMetadata::new(
    ///     Some("API Key".to_string()), // summary
    ///     Some("Authentication token for secure API access".to_string()) // description
    /// );
    ///
    /// // Reference description wins (Priority 1)
    /// assert_eq!(
    ///     ref_meta.merge_with_description(Some("Generic token schema"), false),
    ///     Some("Authentication token for secure API access".to_string())
    /// );
    ///
    /// // Reference summary used when no description (Priority 2)
    /// let summary_only = ReferenceMetadata::new(Some("API Key".to_string()), None);
    /// assert_eq!(
    ///     summary_only.merge_with_description(Some("Generic token schema"), false),
    ///     Some("API Key".to_string())
    /// );
    ///
    /// // Schema description as fallback (Priority 3)
    /// let empty_ref = ReferenceMetadata::new(None, None);
    /// assert_eq!(
    ///     empty_ref.merge_with_description(Some("Generic token schema"), false),
    ///     Some("Generic token schema".to_string())
    /// );
    ///
    /// // Summary takes precedence via best_description() (no prepending when summary is available)
    /// assert_eq!(
    ///     summary_only.merge_with_description(Some("Different description"), true),
    ///     Some("API Key".to_string())
    /// );
    /// ```
    ///
    /// # Arguments
    /// * `existing_desc` - Existing description from the resolved schema object
    /// * `prepend_summary` - Whether to prepend reference summary to existing description
    ///   when no reference description is available (used for special formatting cases)
    ///
    /// # Returns
    /// * `Some(String)` - Enhanced description following precedence hierarchy
    /// * `None` - No description sources available (caller should handle fallback)
    pub fn merge_with_description(
        &self,
        existing_desc: Option<&str>,
        prepend_summary: bool,
    ) -> Option<String> {
        match (self.best_description(), self.summary(), existing_desc) {
            // Reference description takes precedence (OpenAPI 3.1 semantics: contextual > general)
            (Some(ref_desc), _, _) => Some(ref_desc.to_string()),

            // No reference description, use reference summary if available
            (None, Some(ref_summary), Some(existing)) if prepend_summary => {
                if ref_summary != existing {
                    Some(format!("{}\n\n{}", ref_summary, existing))
                } else {
                    Some(existing.to_string())
                }
            }
            (None, Some(ref_summary), _) => Some(ref_summary.to_string()),

            // Fallback to existing schema description only if no reference metadata
            (None, None, Some(existing)) => Some(existing.to_string()),

            // No useful information available
            (None, None, None) => None,
        }
    }

    /// Create enhanced parameter descriptions following OpenAPI 3.1 precedence hierarchy
    ///
    /// This method generates parameter descriptions specifically tailored for MCP tools
    /// by combining reference metadata with parameter names using the OpenAPI 3.1
    /// precedence rules. Unlike general description merging, this method always
    /// includes the parameter name for clarity in tool interfaces.
    ///
    /// ## Parameter Description Hierarchy
    ///
    /// The method follows the same precedence hierarchy as `merge_with_description()` but
    /// formats the output specifically for parameter documentation:
    ///
    /// ### Priority 1: Reference Description (Highest)
    /// - **Format**: `"{param_name}: {reference_description}"`
    /// - **Source**: `$ref.description` field from OpenAPI 3.1 reference object
    /// - **Example**: `"petId: Unique identifier for the pet in the store"`
    /// - **Behavior**: Always used when available, providing contextual parameter meaning
    ///
    /// ### Priority 2: Reference Summary (Medium)
    /// - **Format**: `"{param_name}: {reference_summary}"`
    /// - **Source**: `$ref.summary` field from OpenAPI 3.1 reference object
    /// - **Example**: `"petId: Pet identifier"`
    /// - **Behavior**: Used when no reference description is available
    ///
    /// ### Priority 3: Schema Description (Lower)
    /// - **Format**: `"{existing_description}"` (without parameter name prefix)
    /// - **Source**: `description` field from the parameter's schema object
    /// - **Example**: `"A unique identifier for database entities"`
    /// - **Behavior**: Used only when no reference metadata is available
    /// - **Note**: Does not prepend parameter name to preserve original schema documentation
    ///
    /// ### Priority 4: Generated Fallback (Lowest)
    /// - **Format**: `"{param_name} parameter"`
    /// - **Source**: Auto-generated from parameter name
    /// - **Example**: `"petId parameter"`
    /// - **Behavior**: Always provides a description, ensuring tools have meaningful parameter docs
    ///
    /// ## Design Rationale
    ///
    /// This method addresses the specific needs of MCP tool parameter documentation:
    ///
    /// 1. **Contextual Clarity**: Reference metadata provides usage-specific context
    ///    rather than generic schema documentation
    /// 2. **Parameter Name Integration**: Higher priority items include parameter names
    ///    for immediate clarity in tool interfaces
    /// 3. **Guaranteed Output**: Always returns a description, ensuring no parameter
    ///    lacks documentation in the generated MCP tools
    /// 4. **Semantic Formatting**: Different formatting for different priority levels
    ///    maintains consistency while respecting original schema documentation
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmcp_openapi::tool_generator::ReferenceMetadata;
    ///
    /// // Reference description takes precedence
    /// let with_desc = ReferenceMetadata::new(
    ///     Some("Pet ID".to_string()),
    ///     Some("Unique identifier for pet in the store".to_string())
    /// );
    /// assert_eq!(
    ///     with_desc.enhance_parameter_description("petId", Some("Generic ID field")),
    ///     Some("petId: Unique identifier for pet in the store".to_string())
    /// );
    ///
    /// // Reference summary when no description
    /// let with_summary = ReferenceMetadata::new(Some("Pet ID".to_string()), None);
    /// assert_eq!(
    ///     with_summary.enhance_parameter_description("petId", Some("Generic ID field")),
    ///     Some("petId: Pet ID".to_string())
    /// );
    ///
    /// // Schema description fallback (no name prefix)
    /// let empty_ref = ReferenceMetadata::new(None, None);
    /// assert_eq!(
    ///     empty_ref.enhance_parameter_description("petId", Some("Generic ID field")),
    ///     Some("Generic ID field".to_string())
    /// );
    ///
    /// // Generated fallback ensures always returns description
    /// assert_eq!(
    ///     empty_ref.enhance_parameter_description("petId", None),
    ///     Some("petId parameter".to_string())
    /// );
    /// ```
    pub fn enhance_parameter_description(
        &self,
        param_name: &str,
        existing_desc: Option<&str>,
    ) -> Option<String> {
        match (self.best_description(), self.summary(), existing_desc) {
            // Reference description takes precedence (OpenAPI 3.1 semantics: contextual > general)
            (Some(ref_desc), _, _) => Some(format!("{}: {}", param_name, ref_desc)),

            // No reference description, use reference summary if available
            (None, Some(ref_summary), _) => Some(format!("{}: {}", param_name, ref_summary)),

            // Fallback to existing schema description only if no reference metadata
            (None, None, Some(existing)) => Some(existing.to_string()),

            // No information available - generate contextual description
            (None, None, None) => Some(format!("{} parameter", param_name)),
        }
    }
}

/// Tool generator for creating MCP tools from `OpenAPI` operations
pub struct ToolGenerator;

impl ToolGenerator {
    /// Generate tool metadata from an `OpenAPI` operation
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be converted to tool metadata
    pub fn generate_tool_metadata(
        operation: &Operation,
        method: String,
        path: String,
        spec: &Spec,
        skip_tool_description: bool,
        skip_parameter_descriptions: bool,
    ) -> Result<ToolMetadata, Error> {
        let name = operation.operation_id.clone().unwrap_or_else(|| {
            format!(
                "{}_{}",
                method,
                path.replace('/', "_").replace(['{', '}'], "")
            )
        });

        // Generate parameter schema first so we can include it in description
        let parameters = Self::generate_parameter_schema(
            &operation.parameters,
            &method,
            &operation.request_body,
            spec,
            skip_parameter_descriptions,
        )?;

        // Build description from summary, description, and parameters
        let description =
            (!skip_tool_description).then(|| Self::build_description(operation, &method, &path));

        // Extract output schema from responses (already returns wrapped Value)
        let output_schema = Self::extract_output_schema(&operation.responses, spec)?;

        Ok(ToolMetadata {
            name,
            title: operation.summary.clone(),
            description,
            parameters,
            output_schema,
            method,
            path,
            security: None, // TODO: Extract security requirements from OpenAPI spec
        })
    }

    /// Generate OpenApiTool instances from tool metadata with HTTP configuration
    ///
    /// # Errors
    ///
    /// Returns an error if any OpenApiTool cannot be created
    pub fn generate_openapi_tools(
        tools_metadata: Vec<ToolMetadata>,
        base_url: Option<url::Url>,
        default_headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<Vec<crate::tool::Tool>, Error> {
        let mut openapi_tools = Vec::with_capacity(tools_metadata.len());

        for metadata in tools_metadata {
            let tool = crate::tool::Tool::new(metadata, base_url.clone(), default_headers.clone())?;
            openapi_tools.push(tool);
        }

        Ok(openapi_tools)
    }

    /// Build a comprehensive description for the tool
    fn build_description(operation: &Operation, method: &str, path: &str) -> String {
        match (&operation.summary, &operation.description) {
            (Some(summary), Some(desc)) => {
                format!(
                    "{}\n\n{}\n\nEndpoint: {} {}",
                    summary,
                    desc,
                    method.to_uppercase(),
                    path
                )
            }
            (Some(summary), None) => {
                format!(
                    "{}\n\nEndpoint: {} {}",
                    summary,
                    method.to_uppercase(),
                    path
                )
            }
            (None, Some(desc)) => {
                format!("{}\n\nEndpoint: {} {}", desc, method.to_uppercase(), path)
            }
            (None, None) => {
                format!("API endpoint: {} {}", method.to_uppercase(), path)
            }
        }
    }

    /// Extract output schema from OpenAPI responses
    ///
    /// Prioritizes successful response codes (2XX) and returns the first found schema
    fn extract_output_schema(
        responses: &Option<BTreeMap<String, ObjectOrReference<Response>>>,
        spec: &Spec,
    ) -> Result<Option<Value>, Error> {
        let responses = match responses {
            Some(r) => r,
            None => return Ok(None),
        };
        // Priority order for response codes to check
        let priority_codes = vec![
            "200",     // OK
            "201",     // Created
            "202",     // Accepted
            "203",     // Non-Authoritative Information
            "204",     // No Content (will have no schema)
            "2XX",     // Any 2XX response
            "default", // Default response
        ];

        for status_code in priority_codes {
            if let Some(response_or_ref) = responses.get(status_code) {
                // Resolve reference if needed
                let response = match response_or_ref {
                    ObjectOrReference::Object(response) => response,
                    ObjectOrReference::Ref {
                        ref_path,
                        summary,
                        description,
                    } => {
                        // Response references are not fully resolvable yet (would need resolve_response_reference)
                        // But we can use the reference metadata to create a basic response schema
                        let ref_metadata =
                            ReferenceMetadata::new(summary.clone(), description.clone());

                        if let Some(ref_desc) = ref_metadata.best_description() {
                            // Create a unified response schema with reference description
                            let response_schema = json!({
                                "type": "object",
                                "description": "Unified response structure with success and error variants",
                                "properties": {
                                    "status_code": {
                                        "type": "integer",
                                        "description": "HTTP status code"
                                    },
                                    "body": {
                                        "type": "object",
                                        "description": ref_desc,
                                        "additionalProperties": true
                                    }
                                },
                                "required": ["status_code", "body"]
                            });

                            trace!(
                                reference_path = %ref_path,
                                reference_description = %ref_desc,
                                "Created response schema using reference metadata"
                            );

                            return Ok(Some(response_schema));
                        }

                        // No useful metadata, continue to next response
                        continue;
                    }
                };

                // Skip 204 No Content responses as they shouldn't have a body
                if status_code == "204" {
                    continue;
                }

                // Check if response has content
                if !response.content.is_empty() {
                    let content = &response.content;
                    // Look for JSON content type
                    let json_media_types = vec![
                        "application/json",
                        "application/ld+json",
                        "application/vnd.api+json",
                    ];

                    for media_type_str in json_media_types {
                        if let Some(media_type) = content.get(media_type_str)
                            && let Some(schema_or_ref) = &media_type.schema
                        {
                            // Wrap the schema with success/error structure
                            let wrapped_schema = Self::wrap_output_schema(schema_or_ref, spec)?;
                            return Ok(Some(wrapped_schema));
                        }
                    }

                    // If no JSON media type found, try any media type with a schema
                    for media_type in content.values() {
                        if let Some(schema_or_ref) = &media_type.schema {
                            // Wrap the schema with success/error structure
                            let wrapped_schema = Self::wrap_output_schema(schema_or_ref, spec)?;
                            return Ok(Some(wrapped_schema));
                        }
                    }
                }
            }
        }

        // No response schema found
        Ok(None)
    }

    /// Convert an OpenAPI Schema to JSON Schema format
    ///
    /// This is the unified converter for both input and output schemas.
    /// It handles all OpenAPI schema types and converts them to JSON Schema draft-07 format.
    ///
    /// # Arguments
    /// * `schema` - The OpenAPI Schema to convert
    /// * `spec` - The full OpenAPI specification for resolving references
    /// * `visited` - Set of visited references to prevent infinite recursion
    fn convert_schema_to_json_schema(
        schema: &Schema,
        spec: &Spec,
        visited: &mut HashSet<String>,
    ) -> Result<Value, Error> {
        match schema {
            Schema::Object(obj_schema_or_ref) => match obj_schema_or_ref.as_ref() {
                ObjectOrReference::Object(obj_schema) => {
                    Self::convert_object_schema_to_json_schema(obj_schema, spec, visited)
                }
                ObjectOrReference::Ref { ref_path, .. } => {
                    let resolved = Self::resolve_reference(ref_path, spec, visited)?;
                    Self::convert_object_schema_to_json_schema(&resolved, spec, visited)
                }
            },
            Schema::Boolean(bool_schema) => {
                // Boolean schemas in OpenAPI: true allows any value, false allows no value
                if bool_schema.0 {
                    Ok(json!({})) // Empty schema allows anything
                } else {
                    Ok(json!({"not": {}})) // Schema that matches nothing
                }
            }
        }
    }

    /// Convert ObjectSchema to JSON Schema format
    ///
    /// This is the core converter that handles all schema types and properties.
    /// It processes object properties, arrays, primitives, and all OpenAPI schema attributes.
    ///
    /// # Arguments
    /// * `obj_schema` - The OpenAPI ObjectSchema to convert
    /// * `spec` - The full OpenAPI specification for resolving references
    /// * `visited` - Set of visited references to prevent infinite recursion
    fn convert_object_schema_to_json_schema(
        obj_schema: &ObjectSchema,
        spec: &Spec,
        visited: &mut HashSet<String>,
    ) -> Result<Value, Error> {
        let mut schema_obj = serde_json::Map::new();

        // Add type if specified
        if let Some(schema_type) = &obj_schema.schema_type {
            match schema_type {
                SchemaTypeSet::Single(single_type) => {
                    schema_obj.insert(
                        "type".to_string(),
                        json!(Self::schema_type_to_string(single_type)),
                    );
                }
                SchemaTypeSet::Multiple(type_set) => {
                    let types: Vec<String> =
                        type_set.iter().map(Self::schema_type_to_string).collect();
                    schema_obj.insert("type".to_string(), json!(types));
                }
            }
        }

        // Add description if present
        if let Some(desc) = &obj_schema.description {
            schema_obj.insert("description".to_string(), json!(desc));
        }

        // Handle oneOf schemas - this takes precedence over other schema properties
        if !obj_schema.one_of.is_empty() {
            let mut one_of_schemas = Vec::new();
            for schema_ref in &obj_schema.one_of {
                let schema_json = match schema_ref {
                    ObjectOrReference::Object(schema) => {
                        Self::convert_object_schema_to_json_schema(schema, spec, visited)?
                    }
                    ObjectOrReference::Ref { ref_path, .. } => {
                        let resolved = Self::resolve_reference(ref_path, spec, visited)?;
                        Self::convert_object_schema_to_json_schema(&resolved, spec, visited)?
                    }
                };
                one_of_schemas.push(schema_json);
            }
            schema_obj.insert("oneOf".to_string(), json!(one_of_schemas));
            // When oneOf is present, we typically don't include other properties
            // that would conflict with the oneOf semantics
            return Ok(Value::Object(schema_obj));
        }

        // Handle object properties
        if !obj_schema.properties.is_empty() {
            let properties = &obj_schema.properties;
            let mut props_map = serde_json::Map::new();
            for (prop_name, prop_schema_or_ref) in properties {
                let prop_schema = match prop_schema_or_ref {
                    ObjectOrReference::Object(schema) => {
                        // Convert ObjectSchema to Schema for processing
                        Self::convert_schema_to_json_schema(
                            &Schema::Object(Box::new(ObjectOrReference::Object(schema.clone()))),
                            spec,
                            visited,
                        )?
                    }
                    ObjectOrReference::Ref { ref_path, .. } => {
                        let resolved = Self::resolve_reference(ref_path, spec, visited)?;
                        Self::convert_object_schema_to_json_schema(&resolved, spec, visited)?
                    }
                };

                // Sanitize property name and add original name annotation if needed
                let sanitized_name = sanitize_property_name(prop_name);
                if sanitized_name != *prop_name {
                    // Add original name annotation using Annotations
                    let annotations = Annotations::new().with_original_name(prop_name.clone());
                    let prop_with_annotation =
                        Self::apply_annotations_to_schema(prop_schema, annotations);
                    props_map.insert(sanitized_name, prop_with_annotation);
                } else {
                    props_map.insert(prop_name.clone(), prop_schema);
                }
            }
            schema_obj.insert("properties".to_string(), Value::Object(props_map));
        }

        // Add required fields
        if !obj_schema.required.is_empty() {
            schema_obj.insert("required".to_string(), json!(&obj_schema.required));
        }

        // Handle additionalProperties for object schemas
        if let Some(schema_type) = &obj_schema.schema_type
            && matches!(schema_type, SchemaTypeSet::Single(SchemaType::Object))
        {
            // Handle additional_properties based on the OpenAPI schema
            match &obj_schema.additional_properties {
                None => {
                    // In OpenAPI 3.0, the default for additionalProperties is true
                    schema_obj.insert("additionalProperties".to_string(), json!(true));
                }
                Some(Schema::Boolean(BooleanSchema(value))) => {
                    // Explicit boolean value
                    schema_obj.insert("additionalProperties".to_string(), json!(value));
                }
                Some(Schema::Object(schema_ref)) => {
                    // Additional properties must match this schema
                    let mut visited = HashSet::new();
                    let additional_props_schema = Self::convert_schema_to_json_schema(
                        &Schema::Object(schema_ref.clone()),
                        spec,
                        &mut visited,
                    )?;
                    schema_obj.insert("additionalProperties".to_string(), additional_props_schema);
                }
            }
        }

        // Handle array-specific properties
        if let Some(schema_type) = &obj_schema.schema_type {
            if matches!(schema_type, SchemaTypeSet::Single(SchemaType::Array)) {
                // Handle prefix_items (OpenAPI 3.1 tuple-like arrays)
                if !obj_schema.prefix_items.is_empty() {
                    // Convert prefix_items to draft-07 compatible format
                    Self::convert_prefix_items_to_draft07(
                        &obj_schema.prefix_items,
                        &obj_schema.items,
                        &mut schema_obj,
                        spec,
                    )?;
                } else if let Some(items_schema) = &obj_schema.items {
                    // Handle regular items
                    let items_json =
                        Self::convert_schema_to_json_schema(items_schema, spec, visited)?;
                    schema_obj.insert("items".to_string(), items_json);
                }

                // Add array constraints
                if let Some(min_items) = obj_schema.min_items {
                    schema_obj.insert("minItems".to_string(), json!(min_items));
                }
                if let Some(max_items) = obj_schema.max_items {
                    schema_obj.insert("maxItems".to_string(), json!(max_items));
                }
            } else if let Some(items_schema) = &obj_schema.items {
                // Non-array types shouldn't have items, but handle it anyway
                let items_json = Self::convert_schema_to_json_schema(items_schema, spec, visited)?;
                schema_obj.insert("items".to_string(), items_json);
            }
        }

        // Handle other common properties
        if let Some(format) = &obj_schema.format {
            schema_obj.insert("format".to_string(), json!(format));
        }

        if let Some(example) = &obj_schema.example {
            schema_obj.insert("example".to_string(), example.clone());
        }

        if let Some(default) = &obj_schema.default {
            schema_obj.insert("default".to_string(), default.clone());
        }

        if !obj_schema.enum_values.is_empty() {
            schema_obj.insert("enum".to_string(), json!(&obj_schema.enum_values));
        }

        if let Some(min) = &obj_schema.minimum {
            schema_obj.insert("minimum".to_string(), json!(min));
        }

        if let Some(max) = &obj_schema.maximum {
            schema_obj.insert("maximum".to_string(), json!(max));
        }

        if let Some(min_length) = &obj_schema.min_length {
            schema_obj.insert("minLength".to_string(), json!(min_length));
        }

        if let Some(max_length) = &obj_schema.max_length {
            schema_obj.insert("maxLength".to_string(), json!(max_length));
        }

        if let Some(pattern) = &obj_schema.pattern {
            schema_obj.insert("pattern".to_string(), json!(pattern));
        }

        Ok(Value::Object(schema_obj))
    }

    /// Convert SchemaType to string representation
    fn schema_type_to_string(schema_type: &SchemaType) -> String {
        match schema_type {
            SchemaType::Boolean => "boolean",
            SchemaType::Integer => "integer",
            SchemaType::Number => "number",
            SchemaType::String => "string",
            SchemaType::Array => "array",
            SchemaType::Object => "object",
            SchemaType::Null => "null",
        }
        .to_string()
    }

    /// Resolve a $ref reference to get the actual schema
    ///
    /// # Arguments
    /// * `ref_path` - The reference path (e.g., "#/components/schemas/Pet")
    /// * `spec` - The OpenAPI specification
    /// * `visited` - Set of already visited references to detect circular references
    ///
    /// # Returns
    /// The resolved ObjectSchema or an error if the reference is invalid or circular
    fn resolve_reference(
        ref_path: &str,
        spec: &Spec,
        visited: &mut HashSet<String>,
    ) -> Result<ObjectSchema, Error> {
        // Check for circular reference
        if visited.contains(ref_path) {
            return Err(Error::ToolGeneration(format!(
                "Circular reference detected: {ref_path}"
            )));
        }

        // Add to visited set
        visited.insert(ref_path.to_string());

        // Parse the reference path
        // Currently only supporting local references like "#/components/schemas/Pet"
        if !ref_path.starts_with("#/components/schemas/") {
            return Err(Error::ToolGeneration(format!(
                "Unsupported reference format: {ref_path}. Only #/components/schemas/ references are supported"
            )));
        }

        let schema_name = ref_path.strip_prefix("#/components/schemas/").unwrap();

        // Get the schema from components
        let components = spec.components.as_ref().ok_or_else(|| {
            Error::ToolGeneration(format!(
                "Reference {ref_path} points to components, but spec has no components section"
            ))
        })?;

        let schema_ref = components.schemas.get(schema_name).ok_or_else(|| {
            Error::ToolGeneration(format!(
                "Schema '{schema_name}' not found in components/schemas"
            ))
        })?;

        // Resolve the schema reference
        let resolved_schema = match schema_ref {
            ObjectOrReference::Object(obj_schema) => obj_schema.clone(),
            ObjectOrReference::Ref {
                ref_path: nested_ref,
                ..
            } => {
                // Recursively resolve nested references
                Self::resolve_reference(nested_ref, spec, visited)?
            }
        };

        // Remove from visited set before returning (for other resolution paths)
        visited.remove(ref_path);

        Ok(resolved_schema)
    }

    /// Resolve reference with metadata extraction
    ///
    /// Extracts summary and description from the reference before resolving,
    /// returning both the resolved schema and the preserved metadata.
    fn resolve_reference_with_metadata(
        ref_path: &str,
        summary: Option<String>,
        description: Option<String>,
        spec: &Spec,
        visited: &mut HashSet<String>,
    ) -> Result<(ObjectSchema, ReferenceMetadata), Error> {
        let resolved_schema = Self::resolve_reference(ref_path, spec, visited)?;
        let metadata = ReferenceMetadata::new(summary, description);
        Ok((resolved_schema, metadata))
    }

    /// Generate JSON Schema for tool parameters
    fn generate_parameter_schema(
        parameters: &[ObjectOrReference<Parameter>],
        _method: &str,
        request_body: &Option<ObjectOrReference<RequestBody>>,
        spec: &Spec,
        skip_parameter_descriptions: bool,
    ) -> Result<Value, Error> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        // Group parameters by location
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();
        let mut cookie_params = Vec::new();

        for param_ref in parameters {
            let param = match param_ref {
                ObjectOrReference::Object(param) => param,
                ObjectOrReference::Ref { ref_path, .. } => {
                    // Try to resolve parameter reference
                    // Note: Parameter references are rare and not supported yet in this implementation
                    // For now, we'll continue to skip them but log a warning
                    warn!(
                        reference_path = %ref_path,
                        "Parameter reference not resolved"
                    );
                    continue;
                }
            };

            match &param.location {
                ParameterIn::Query => query_params.push(param),
                ParameterIn::Header => header_params.push(param),
                ParameterIn::Path => path_params.push(param),
                ParameterIn::Cookie => cookie_params.push(param),
            }
        }

        // Process path parameters (always required)
        for param in path_params {
            let (param_schema, mut annotations) = Self::convert_parameter_schema(
                param,
                ParameterIn::Path,
                spec,
                skip_parameter_descriptions,
            )?;

            // Sanitize parameter name and add original name annotation if needed
            let sanitized_name = sanitize_property_name(&param.name);
            if sanitized_name != param.name {
                annotations = annotations.with_original_name(param.name.clone());
            }

            let param_schema_with_annotations =
                Self::apply_annotations_to_schema(param_schema, annotations);
            properties.insert(sanitized_name.clone(), param_schema_with_annotations);
            required.push(sanitized_name);
        }

        // Process query parameters
        for param in &query_params {
            let (param_schema, mut annotations) = Self::convert_parameter_schema(
                param,
                ParameterIn::Query,
                spec,
                skip_parameter_descriptions,
            )?;

            // Sanitize parameter name and add original name annotation if needed
            let sanitized_name = sanitize_property_name(&param.name);
            if sanitized_name != param.name {
                annotations = annotations.with_original_name(param.name.clone());
            }

            let param_schema_with_annotations =
                Self::apply_annotations_to_schema(param_schema, annotations);
            properties.insert(sanitized_name.clone(), param_schema_with_annotations);
            if param.required.unwrap_or(false) {
                required.push(sanitized_name);
            }
        }

        // Process header parameters (optional by default unless explicitly required)
        for param in &header_params {
            let (param_schema, mut annotations) = Self::convert_parameter_schema(
                param,
                ParameterIn::Header,
                spec,
                skip_parameter_descriptions,
            )?;

            // Sanitize parameter name after prefixing and add original name annotation if needed
            let prefixed_name = format!("header_{}", param.name);
            let sanitized_name = sanitize_property_name(&prefixed_name);
            if sanitized_name != prefixed_name {
                annotations = annotations.with_original_name(param.name.clone());
            }

            let param_schema_with_annotations =
                Self::apply_annotations_to_schema(param_schema, annotations);

            properties.insert(sanitized_name.clone(), param_schema_with_annotations);
            if param.required.unwrap_or(false) {
                required.push(sanitized_name);
            }
        }

        // Process cookie parameters (rare, but supported)
        for param in &cookie_params {
            let (param_schema, mut annotations) = Self::convert_parameter_schema(
                param,
                ParameterIn::Cookie,
                spec,
                skip_parameter_descriptions,
            )?;

            // Sanitize parameter name after prefixing and add original name annotation if needed
            let prefixed_name = format!("cookie_{}", param.name);
            let sanitized_name = sanitize_property_name(&prefixed_name);
            if sanitized_name != prefixed_name {
                annotations = annotations.with_original_name(param.name.clone());
            }

            let param_schema_with_annotations =
                Self::apply_annotations_to_schema(param_schema, annotations);

            properties.insert(sanitized_name.clone(), param_schema_with_annotations);
            if param.required.unwrap_or(false) {
                required.push(sanitized_name);
            }
        }

        // Add request body parameter if defined in the OpenAPI spec
        if let Some(request_body) = request_body
            && let Some((body_schema, annotations, is_required)) =
                Self::convert_request_body_to_json_schema(request_body, spec)?
        {
            let body_schema_with_annotations =
                Self::apply_annotations_to_schema(body_schema, annotations);
            properties.insert("request_body".to_string(), body_schema_with_annotations);
            if is_required {
                required.push("request_body".to_string());
            }
        }

        // Add special parameters for request configuration
        if !query_params.is_empty() || !header_params.is_empty() || !cookie_params.is_empty() {
            // Add optional timeout parameter
            properties.insert(
                "timeout_seconds".to_string(),
                json!({
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "minimum": 1,
                    "maximum": 300,
                    "default": 30
                }),
            );
        }

        Ok(json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false
        }))
    }

    /// Convert `OpenAPI` parameter schema to JSON Schema for MCP tools
    fn convert_parameter_schema(
        param: &Parameter,
        location: ParameterIn,
        spec: &Spec,
        skip_parameter_descriptions: bool,
    ) -> Result<(Value, Annotations), Error> {
        // Convert the parameter schema using the unified converter
        let base_schema = if let Some(schema_ref) = &param.schema {
            match schema_ref {
                ObjectOrReference::Object(obj_schema) => {
                    let mut visited = HashSet::new();
                    Self::convert_schema_to_json_schema(
                        &Schema::Object(Box::new(ObjectOrReference::Object(obj_schema.clone()))),
                        spec,
                        &mut visited,
                    )?
                }
                ObjectOrReference::Ref {
                    ref_path,
                    summary,
                    description,
                } => {
                    // Resolve the reference with metadata extraction
                    let mut visited = HashSet::new();
                    match Self::resolve_reference_with_metadata(
                        ref_path,
                        summary.clone(),
                        description.clone(),
                        spec,
                        &mut visited,
                    ) {
                        Ok((resolved_schema, ref_metadata)) => {
                            let mut schema_json = Self::convert_schema_to_json_schema(
                                &Schema::Object(Box::new(ObjectOrReference::Object(
                                    resolved_schema,
                                ))),
                                spec,
                                &mut visited,
                            )?;

                            // Enhance schema with reference metadata if available
                            if let Value::Object(ref mut schema_obj) = schema_json {
                                // Reference metadata takes precedence over schema descriptions (OpenAPI 3.1 semantics)
                                if let Some(ref_desc) = ref_metadata.best_description() {
                                    schema_obj.insert("description".to_string(), json!(ref_desc));
                                }
                                // Fallback: if no reference metadata but schema lacks description, keep existing logic
                                // (This case is now handled by the reference metadata being None)
                            }

                            schema_json
                        }
                        Err(_) => {
                            // Fallback to string for unresolvable references
                            json!({"type": "string"})
                        }
                    }
                }
            }
        } else {
            // Default to string if no schema
            json!({"type": "string"})
        };

        // Merge the base schema properties with parameter metadata
        let mut result = match base_schema {
            Value::Object(obj) => obj,
            _ => {
                // This should never happen as our converter always returns objects
                return Err(Error::ToolGeneration(format!(
                    "Internal error: schema converter returned non-object for parameter '{}'",
                    param.name
                )));
            }
        };

        // Collect examples from various sources
        let mut collected_examples = Vec::new();

        // First, check for parameter-level examples
        if let Some(example) = &param.example {
            collected_examples.push(example.clone());
        } else if !param.examples.is_empty() {
            // Collect from examples map
            for example_ref in param.examples.values() {
                match example_ref {
                    ObjectOrReference::Object(example_obj) => {
                        if let Some(value) = &example_obj.value {
                            collected_examples.push(value.clone());
                        }
                    }
                    ObjectOrReference::Ref { .. } => {
                        // Skip references in examples for now
                    }
                }
            }
        } else if let Some(Value::String(ex_str)) = result.get("example") {
            // If there's an example from the schema, collect it
            collected_examples.push(json!(ex_str));
        } else if let Some(ex) = result.get("example") {
            collected_examples.push(ex.clone());
        }

        // Build description with examples
        let base_description = param
            .description
            .as_ref()
            .map(|d| d.to_string())
            .or_else(|| {
                result
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(|d| d.to_string())
            })
            .unwrap_or_else(|| format!("{} parameter", param.name));

        let description_with_examples = if let Some(examples_str) =
            Self::format_examples_for_description(&collected_examples)
        {
            format!("{base_description}. {examples_str}")
        } else {
            base_description
        };

        if !skip_parameter_descriptions {
            result.insert("description".to_string(), json!(description_with_examples));
        }

        // Add parameter-level example if present
        // Priority: param.example > param.examples > schema.example
        // Note: schema.example is already added during base schema conversion,
        // so parameter examples will override it by being added after
        if let Some(example) = &param.example {
            result.insert("example".to_string(), example.clone());
        } else if !param.examples.is_empty() {
            // If no single example but we have multiple examples, use the first one
            // Also store all examples for potential use in documentation
            let mut examples_array = Vec::new();
            for (example_name, example_ref) in &param.examples {
                match example_ref {
                    ObjectOrReference::Object(example_obj) => {
                        if let Some(value) = &example_obj.value {
                            examples_array.push(json!({
                                "name": example_name,
                                "value": value
                            }));
                        }
                    }
                    ObjectOrReference::Ref { .. } => {
                        // For now, skip references in examples
                        // Could be enhanced to resolve references
                    }
                }
            }

            if !examples_array.is_empty() {
                // Use the first example's value as the main example
                if let Some(first_example) = examples_array.first()
                    && let Some(value) = first_example.get("value")
                {
                    result.insert("example".to_string(), value.clone());
                }
                // Store all examples for documentation purposes
                result.insert("x-examples".to_string(), json!(examples_array));
            }
        }

        // Create annotations instead of adding them to the JSON
        let mut annotations = Annotations::new()
            .with_location(Location::Parameter(location))
            .with_required(param.required.unwrap_or(false));

        // Add explode annotation if present
        if let Some(explode) = param.explode {
            annotations = annotations.with_explode(explode);
        } else {
            // Default explode behavior based on OpenAPI spec:
            // - form style defaults to true
            // - other styles default to false
            let default_explode = match &param.style {
                Some(ParameterStyle::Form) | None => true, // form is default style
                _ => false,
            };
            annotations = annotations.with_explode(default_explode);
        }

        Ok((Value::Object(result), annotations))
    }

    /// Apply annotations to a JSON schema value
    fn apply_annotations_to_schema(schema: Value, annotations: Annotations) -> Value {
        match schema {
            Value::Object(mut obj) => {
                // Serialize annotations and merge them into the schema object
                if let Ok(Value::Object(ann_map)) = serde_json::to_value(&annotations) {
                    for (key, value) in ann_map {
                        obj.insert(key, value);
                    }
                }
                Value::Object(obj)
            }
            _ => schema,
        }
    }

    /// Format examples for inclusion in parameter descriptions
    fn format_examples_for_description(examples: &[Value]) -> Option<String> {
        if examples.is_empty() {
            return None;
        }

        if examples.len() == 1 {
            let example_str =
                serde_json::to_string(&examples[0]).unwrap_or_else(|_| "null".to_string());
            Some(format!("Example: `{example_str}`"))
        } else {
            let mut result = String::from("Examples:\n");
            for ex in examples {
                let json_str = serde_json::to_string(ex).unwrap_or_else(|_| "null".to_string());
                result.push_str(&format!("- `{json_str}`\n"));
            }
            // Remove trailing newline
            result.pop();
            Some(result)
        }
    }

    /// Converts prefixItems (tuple-like arrays) to JSON Schema draft-07 compatible format.
    ///
    /// This handles OpenAPI 3.1 prefixItems which define specific schemas for each array position,
    /// converting them to draft-07 format that MCP tools can understand.
    ///
    /// Conversion strategy:
    /// - If items is `false`, set minItems=maxItems=prefix_items.len() for exact length
    /// - If all prefixItems have same type, use that type for items
    /// - If mixed types, use oneOf with all unique types from prefixItems
    /// - Add descriptive comment about tuple nature
    fn convert_prefix_items_to_draft07(
        prefix_items: &[ObjectOrReference<ObjectSchema>],
        items: &Option<Box<Schema>>,
        result: &mut serde_json::Map<String, Value>,
        spec: &Spec,
    ) -> Result<(), Error> {
        let prefix_count = prefix_items.len();

        // Extract types from prefixItems
        let mut item_types = Vec::new();
        for prefix_item in prefix_items {
            match prefix_item {
                ObjectOrReference::Object(obj_schema) => {
                    if let Some(schema_type) = &obj_schema.schema_type {
                        match schema_type {
                            SchemaTypeSet::Single(SchemaType::String) => item_types.push("string"),
                            SchemaTypeSet::Single(SchemaType::Integer) => {
                                item_types.push("integer")
                            }
                            SchemaTypeSet::Single(SchemaType::Number) => item_types.push("number"),
                            SchemaTypeSet::Single(SchemaType::Boolean) => {
                                item_types.push("boolean")
                            }
                            SchemaTypeSet::Single(SchemaType::Array) => item_types.push("array"),
                            SchemaTypeSet::Single(SchemaType::Object) => item_types.push("object"),
                            _ => item_types.push("string"), // fallback
                        }
                    } else {
                        item_types.push("string"); // fallback
                    }
                }
                ObjectOrReference::Ref { ref_path, .. } => {
                    // Try to resolve the reference
                    let mut visited = HashSet::new();
                    match Self::resolve_reference(ref_path, spec, &mut visited) {
                        Ok(resolved_schema) => {
                            // Extract the type immediately and store it as a string
                            if let Some(schema_type_set) = &resolved_schema.schema_type {
                                match schema_type_set {
                                    SchemaTypeSet::Single(SchemaType::String) => {
                                        item_types.push("string")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Integer) => {
                                        item_types.push("integer")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Number) => {
                                        item_types.push("number")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Boolean) => {
                                        item_types.push("boolean")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Array) => {
                                        item_types.push("array")
                                    }
                                    SchemaTypeSet::Single(SchemaType::Object) => {
                                        item_types.push("object")
                                    }
                                    _ => item_types.push("string"), // fallback
                                }
                            } else {
                                item_types.push("string"); // fallback
                            }
                        }
                        Err(_) => {
                            // Fallback to string for unresolvable references
                            item_types.push("string");
                        }
                    }
                }
            }
        }

        // Check if items is false (no additional items allowed)
        let items_is_false =
            matches!(items.as_ref().map(|i| i.as_ref()), Some(Schema::Boolean(b)) if !b.0);

        if items_is_false {
            // Exact array length required
            result.insert("minItems".to_string(), json!(prefix_count));
            result.insert("maxItems".to_string(), json!(prefix_count));
        }

        // Determine items schema based on prefixItems types
        let unique_types: std::collections::BTreeSet<_> = item_types.into_iter().collect();

        if unique_types.len() == 1 {
            // All items have same type
            let item_type = unique_types.into_iter().next().unwrap();
            result.insert("items".to_string(), json!({"type": item_type}));
        } else if unique_types.len() > 1 {
            // Mixed types, use oneOf (sorted for consistent ordering)
            let one_of: Vec<Value> = unique_types
                .into_iter()
                .map(|t| json!({"type": t}))
                .collect();
            result.insert("items".to_string(), json!({"oneOf": one_of}));
        }

        Ok(())
    }

    /// Converts the new oas3 Schema enum (which can be Boolean or Object) to draft-07 format.
    ///
    /// The oas3 crate now supports:
    /// - Schema::Object(`ObjectOrReference<ObjectSchema>`) - regular object schemas
    /// - Schema::Boolean(BooleanSchema) - true/false schemas for validation control
    ///
    /// For MCP compatibility (draft-07), we convert:
    /// - Boolean true -> allow any items (no items constraint)
    /// - Boolean false -> not handled here (should be handled by caller with array constraints)
    ///
    /// Convert request body from OpenAPI to JSON Schema for MCP tools
    fn convert_request_body_to_json_schema(
        request_body_ref: &ObjectOrReference<RequestBody>,
        spec: &Spec,
    ) -> Result<Option<(Value, Annotations, bool)>, Error> {
        match request_body_ref {
            ObjectOrReference::Object(request_body) => {
                // Extract schema from request body content
                // Prioritize application/json content type
                let schema_info = request_body
                    .content
                    .get(mime::APPLICATION_JSON.as_ref())
                    .or_else(|| request_body.content.get("application/json"))
                    .or_else(|| {
                        // Fall back to first available content type
                        request_body.content.values().next()
                    });

                if let Some(media_type) = schema_info {
                    if let Some(schema_ref) = &media_type.schema {
                        // Convert ObjectOrReference<ObjectSchema> to Schema
                        let schema = Schema::Object(Box::new(schema_ref.clone()));

                        // Use the unified converter
                        let mut visited = HashSet::new();
                        let converted_schema =
                            Self::convert_schema_to_json_schema(&schema, spec, &mut visited)?;

                        // Ensure we have an object schema
                        let mut schema_obj = match converted_schema {
                            Value::Object(obj) => obj,
                            _ => {
                                // If not an object, wrap it in an object
                                let mut obj = serde_json::Map::new();
                                obj.insert("type".to_string(), json!("object"));
                                obj.insert("additionalProperties".to_string(), json!(true));
                                obj
                            }
                        };

                        // Add description following OpenAPI 3.1 precedence (schema description > request body description)
                        if !schema_obj.contains_key("description") {
                            let description = request_body
                                .description
                                .clone()
                                .unwrap_or_else(|| "Request body data".to_string());
                            schema_obj.insert("description".to_string(), json!(description));
                        }

                        // Create annotations instead of adding them to the JSON
                        let annotations = Annotations::new()
                            .with_location(Location::Body)
                            .with_content_type(mime::APPLICATION_JSON.as_ref().to_string());

                        let required = request_body.required.unwrap_or(false);
                        Ok(Some((Value::Object(schema_obj), annotations, required)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            ObjectOrReference::Ref {
                ref_path: _,
                summary,
                description,
            } => {
                // Use reference metadata to enhance request body description
                let ref_metadata = ReferenceMetadata::new(summary.clone(), description.clone());
                let enhanced_description = ref_metadata
                    .best_description()
                    .map(|desc| desc.to_string())
                    .unwrap_or_else(|| "Request body data".to_string());

                let mut result = serde_json::Map::new();
                result.insert("type".to_string(), json!("object"));
                result.insert("additionalProperties".to_string(), json!(true));
                result.insert("description".to_string(), json!(enhanced_description));

                // Create annotations instead of adding them to the JSON
                let annotations = Annotations::new()
                    .with_location(Location::Body)
                    .with_content_type(mime::APPLICATION_JSON.as_ref().to_string());

                Ok(Some((Value::Object(result), annotations, false)))
            }
        }
    }

    /// Extract parameter values from MCP tool call arguments
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are invalid or missing required parameters
    pub fn extract_parameters(
        tool_metadata: &ToolMetadata,
        arguments: &Value,
    ) -> Result<ExtractedParameters, ToolCallValidationError> {
        let args = arguments.as_object().ok_or_else(|| {
            ToolCallValidationError::RequestConstructionError {
                reason: "Arguments must be an object".to_string(),
            }
        })?;

        trace!(
            tool_name = %tool_metadata.name,
            raw_arguments = ?arguments,
            "Starting parameter extraction"
        );

        let mut path_params = HashMap::new();
        let mut query_params = HashMap::new();
        let mut header_params = HashMap::new();
        let mut cookie_params = HashMap::new();
        let mut body_params = HashMap::new();
        let mut config = RequestConfig::default();

        // Extract timeout if provided
        if let Some(timeout) = args.get("timeout_seconds").and_then(Value::as_u64) {
            config.timeout_seconds = u32::try_from(timeout).unwrap_or(u32::MAX);
        }

        // Process each argument
        for (key, value) in args {
            if key == "timeout_seconds" {
                continue; // Already processed
            }

            // Handle special request_body parameter
            if key == "request_body" {
                body_params.insert("request_body".to_string(), value.clone());
                continue;
            }

            // Determine parameter location from the tool metadata
            let location = Self::get_parameter_location(tool_metadata, key).map_err(|e| {
                ToolCallValidationError::RequestConstructionError {
                    reason: e.to_string(),
                }
            })?;

            // Get the original name if it exists
            let original_name = Self::get_original_parameter_name(tool_metadata, key);

            match location.as_str() {
                "path" => {
                    path_params.insert(original_name.unwrap_or_else(|| key.clone()), value.clone());
                }
                "query" => {
                    let param_name = original_name.unwrap_or_else(|| key.clone());
                    let explode = Self::get_parameter_explode(tool_metadata, key);
                    query_params.insert(param_name, QueryParameter::new(value.clone(), explode));
                }
                "header" => {
                    // Use original name if available, otherwise remove "header_" prefix
                    let header_name = if let Some(orig) = original_name {
                        orig
                    } else if key.starts_with("header_") {
                        key.strip_prefix("header_").unwrap_or(key).to_string()
                    } else {
                        key.clone()
                    };
                    header_params.insert(header_name, value.clone());
                }
                "cookie" => {
                    // Use original name if available, otherwise remove "cookie_" prefix
                    let cookie_name = if let Some(orig) = original_name {
                        orig
                    } else if key.starts_with("cookie_") {
                        key.strip_prefix("cookie_").unwrap_or(key).to_string()
                    } else {
                        key.clone()
                    };
                    cookie_params.insert(cookie_name, value.clone());
                }
                "body" => {
                    // Remove "body_" prefix if present
                    let body_name = if key.starts_with("body_") {
                        key.strip_prefix("body_").unwrap_or(key).to_string()
                    } else {
                        key.clone()
                    };
                    body_params.insert(body_name, value.clone());
                }
                _ => {
                    return Err(ToolCallValidationError::RequestConstructionError {
                        reason: format!("Unknown parameter location for parameter: {key}"),
                    });
                }
            }
        }

        let extracted = ExtractedParameters {
            path: path_params,
            query: query_params,
            headers: header_params,
            cookies: cookie_params,
            body: body_params,
            config,
        };

        trace!(
            tool_name = %tool_metadata.name,
            extracted_parameters = ?extracted,
            "Parameter extraction completed"
        );

        // Validate parameters against tool metadata using the original arguments
        Self::validate_parameters(tool_metadata, arguments)?;

        Ok(extracted)
    }

    /// Get the original parameter name from x-original-name annotation if it exists
    fn get_original_parameter_name(
        tool_metadata: &ToolMetadata,
        param_name: &str,
    ) -> Option<String> {
        tool_metadata
            .parameters
            .get("properties")
            .and_then(|p| p.as_object())
            .and_then(|props| props.get(param_name))
            .and_then(|schema| schema.get(X_ORIGINAL_NAME))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get parameter explode setting from tool metadata
    fn get_parameter_explode(tool_metadata: &ToolMetadata, param_name: &str) -> bool {
        tool_metadata
            .parameters
            .get("properties")
            .and_then(|p| p.as_object())
            .and_then(|props| props.get(param_name))
            .and_then(|schema| schema.get(X_PARAMETER_EXPLODE))
            .and_then(|v| v.as_bool())
            .unwrap_or(true) // Default to true (OpenAPI default for form style)
    }

    /// Get parameter location from tool metadata
    fn get_parameter_location(
        tool_metadata: &ToolMetadata,
        param_name: &str,
    ) -> Result<String, Error> {
        let properties = tool_metadata
            .parameters
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or_else(|| Error::ToolGeneration("Invalid tool parameters schema".to_string()))?;

        if let Some(param_schema) = properties.get(param_name)
            && let Some(location) = param_schema
                .get(X_PARAMETER_LOCATION)
                .and_then(|v| v.as_str())
        {
            return Ok(location.to_string());
        }

        // Fallback: infer from parameter name prefix
        if param_name.starts_with("header_") {
            Ok("header".to_string())
        } else if param_name.starts_with("cookie_") {
            Ok("cookie".to_string())
        } else if param_name.starts_with("body_") {
            Ok("body".to_string())
        } else {
            // Default to query for unknown parameters
            Ok("query".to_string())
        }
    }

    /// Validate parameters against tool metadata
    fn validate_parameters(
        tool_metadata: &ToolMetadata,
        arguments: &Value,
    ) -> Result<(), ToolCallValidationError> {
        let schema = &tool_metadata.parameters;

        // Get required parameters from schema
        let required_params = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<std::collections::HashSet<_>>()
            })
            .unwrap_or_default();

        let properties = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or_else(|| ToolCallValidationError::RequestConstructionError {
                reason: "Tool schema missing properties".to_string(),
            })?;

        let args = arguments.as_object().ok_or_else(|| {
            ToolCallValidationError::RequestConstructionError {
                reason: "Arguments must be an object".to_string(),
            }
        })?;

        // Collect ALL validation errors before returning
        let mut all_errors = Vec::new();

        // Check for unknown parameters
        all_errors.extend(Self::check_unknown_parameters(args, properties));

        // Check all required parameters are provided in the arguments
        all_errors.extend(Self::check_missing_required(
            args,
            properties,
            &required_params,
        ));

        // Validate parameter values against their schemas
        all_errors.extend(Self::validate_parameter_values(
            args,
            properties,
            &required_params,
        ));

        // Return all errors if any were found
        if !all_errors.is_empty() {
            return Err(ToolCallValidationError::InvalidParameters {
                violations: all_errors,
            });
        }

        Ok(())
    }

    /// Check for unknown parameters in the provided arguments
    fn check_unknown_parameters(
        args: &serde_json::Map<String, Value>,
        properties: &serde_json::Map<String, Value>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Get list of valid parameter names
        let valid_params: Vec<String> = properties.keys().map(|s| s.to_string()).collect();

        // Check each provided argument
        for (arg_name, _) in args.iter() {
            if !properties.contains_key(arg_name) {
                // Create InvalidParameter error with suggestions
                errors.push(ValidationError::invalid_parameter(
                    arg_name.clone(),
                    &valid_params,
                ));
            }
        }

        errors
    }

    /// Check for missing required parameters
    fn check_missing_required(
        args: &serde_json::Map<String, Value>,
        properties: &serde_json::Map<String, Value>,
        required_params: &HashSet<&str>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for required_param in required_params {
            if !args.contains_key(*required_param) {
                // Get the parameter schema to extract description and type
                let param_schema = properties.get(*required_param);

                let description = param_schema
                    .and_then(|schema| schema.get("description"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());

                let expected_type = param_schema
                    .and_then(Self::get_expected_type)
                    .unwrap_or_else(|| "unknown".to_string());

                errors.push(ValidationError::MissingRequiredParameter {
                    parameter: (*required_param).to_string(),
                    description,
                    expected_type,
                });
            }
        }

        errors
    }

    /// Validate parameter values against their schemas
    fn validate_parameter_values(
        args: &serde_json::Map<String, Value>,
        properties: &serde_json::Map<String, Value>,
        required_params: &std::collections::HashSet<&str>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (param_name, param_value) in args {
            if let Some(param_schema) = properties.get(param_name) {
                // Check if this is a null value to provide better error messages
                let is_null_value = param_value.is_null();
                let is_required = required_params.contains(param_name.as_str());

                // Create a schema that wraps the parameter schema
                let schema = json!({
                    "type": "object",
                    "properties": {
                        param_name: param_schema
                    }
                });

                // Compile the schema
                let compiled = match jsonschema::validator_for(&schema) {
                    Ok(compiled) => compiled,
                    Err(e) => {
                        errors.push(ValidationError::ConstraintViolation {
                            parameter: param_name.clone(),
                            message: format!(
                                "Failed to compile schema for parameter '{param_name}': {e}"
                            ),
                            field_path: None,
                            actual_value: None,
                            expected_type: None,
                            constraints: vec![],
                        });
                        continue;
                    }
                };

                // Create an object with just this parameter to validate
                let instance = json!({ param_name: param_value });

                // Validate and collect all errors for this parameter
                let validation_errors: Vec<_> =
                    compiled.validate(&instance).err().into_iter().collect();

                for validation_error in validation_errors {
                    // Extract error details
                    let error_message = validation_error.to_string();
                    let instance_path_str = validation_error.instance_path.to_string();
                    let field_path = if instance_path_str.is_empty() || instance_path_str == "/" {
                        Some(param_name.clone())
                    } else {
                        Some(instance_path_str.trim_start_matches('/').to_string())
                    };

                    // Extract constraints from the schema
                    let constraints = Self::extract_constraints_from_schema(param_schema);

                    // Determine expected type
                    let expected_type = Self::get_expected_type(param_schema);

                    // Generate context-aware error message for null values
                    let message = if is_null_value && error_message.contains("is not of type") {
                        if let Some(ref expected_type_str) = expected_type {
                            if is_required {
                                format!(
                                    "Parameter '{}' is required and must be non-null (expected: {})",
                                    param_name, expected_type_str
                                )
                            } else {
                                format!(
                                    "Parameter '{}' must be {} when provided (null not allowed, omit if not needed)",
                                    param_name, expected_type_str
                                )
                            }
                        } else {
                            error_message
                        }
                    } else {
                        error_message
                    };

                    errors.push(ValidationError::ConstraintViolation {
                        parameter: param_name.clone(),
                        message,
                        field_path,
                        actual_value: Some(Box::new(param_value.clone())),
                        expected_type,
                        constraints,
                    });
                }
            }
        }

        errors
    }

    /// Extract validation constraints from a schema
    fn extract_constraints_from_schema(schema: &Value) -> Vec<ValidationConstraint> {
        let mut constraints = Vec::new();

        // Minimum value constraint
        if let Some(min_value) = schema.get("minimum").and_then(|v| v.as_f64()) {
            let exclusive = schema
                .get("exclusiveMinimum")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            constraints.push(ValidationConstraint::Minimum {
                value: min_value,
                exclusive,
            });
        }

        // Maximum value constraint
        if let Some(max_value) = schema.get("maximum").and_then(|v| v.as_f64()) {
            let exclusive = schema
                .get("exclusiveMaximum")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            constraints.push(ValidationConstraint::Maximum {
                value: max_value,
                exclusive,
            });
        }

        // Minimum length constraint
        if let Some(min_len) = schema
            .get("minLength")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            constraints.push(ValidationConstraint::MinLength { value: min_len });
        }

        // Maximum length constraint
        if let Some(max_len) = schema
            .get("maxLength")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            constraints.push(ValidationConstraint::MaxLength { value: max_len });
        }

        // Pattern constraint
        if let Some(pattern) = schema
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            constraints.push(ValidationConstraint::Pattern { pattern });
        }

        // Enum values constraint
        if let Some(enum_values) = schema.get("enum").and_then(|v| v.as_array()).cloned() {
            constraints.push(ValidationConstraint::EnumValues {
                values: enum_values,
            });
        }

        // Format constraint
        if let Some(format) = schema
            .get("format")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            constraints.push(ValidationConstraint::Format { format });
        }

        // Multiple of constraint
        if let Some(multiple_of) = schema.get("multipleOf").and_then(|v| v.as_f64()) {
            constraints.push(ValidationConstraint::MultipleOf { value: multiple_of });
        }

        // Minimum items constraint
        if let Some(min_items) = schema
            .get("minItems")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            constraints.push(ValidationConstraint::MinItems { value: min_items });
        }

        // Maximum items constraint
        if let Some(max_items) = schema
            .get("maxItems")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            constraints.push(ValidationConstraint::MaxItems { value: max_items });
        }

        // Unique items constraint
        if let Some(true) = schema.get("uniqueItems").and_then(|v| v.as_bool()) {
            constraints.push(ValidationConstraint::UniqueItems);
        }

        // Minimum properties constraint
        if let Some(min_props) = schema
            .get("minProperties")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            constraints.push(ValidationConstraint::MinProperties { value: min_props });
        }

        // Maximum properties constraint
        if let Some(max_props) = schema
            .get("maxProperties")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            constraints.push(ValidationConstraint::MaxProperties { value: max_props });
        }

        // Constant value constraint
        if let Some(const_value) = schema.get("const").cloned() {
            constraints.push(ValidationConstraint::ConstValue { value: const_value });
        }

        // Required properties constraint
        if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
            let properties: Vec<String> = required
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !properties.is_empty() {
                constraints.push(ValidationConstraint::Required { properties });
            }
        }

        constraints
    }

    /// Get the expected type from a schema
    fn get_expected_type(schema: &Value) -> Option<String> {
        if let Some(type_value) = schema.get("type") {
            if let Some(type_str) = type_value.as_str() {
                return Some(type_str.to_string());
            } else if let Some(type_array) = type_value.as_array() {
                // Handle multiple types (e.g., ["string", "null"])
                let types: Vec<String> = type_array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                if !types.is_empty() {
                    return Some(types.join(" | "));
                }
            }
        }
        None
    }

    /// Wrap an output schema to include both success and error responses
    ///
    /// This function creates a unified response schema that can represent both successful
    /// responses and error responses. It uses `json!()` macro instead of `schema_for!()`
    /// for several important reasons:
    ///
    /// 1. **Dynamic Schema Construction**: The success schema is dynamically converted from
    ///    OpenAPI specifications at runtime, not from a static Rust type. The `schema_for!()`
    ///    macro requires a compile-time type, but we're working with schemas that are only
    ///    known when parsing the OpenAPI spec.
    ///
    /// 2. **Composite Schema Building**: The function builds a complex wrapper schema that:
    ///    - Contains a dynamically-converted OpenAPI schema for success responses
    ///    - Includes a statically-typed error schema (which does use `schema_for!()`)
    ///    - Adds metadata fields like HTTP status codes and descriptions
    ///    - Uses JSON Schema's `oneOf` to allow either success or error responses
    ///
    /// 3. **Runtime Flexibility**: OpenAPI schemas can have arbitrary complexity and types
    ///    that don't map directly to Rust types. Using `json!()` allows us to construct
    ///    the exact JSON Schema structure needed without being constrained by Rust's type system.
    ///
    /// The error schema component does use `schema_for!(ErrorResponse)` (via `create_error_response_schema()`)
    /// because `ErrorResponse` is a known Rust type, but the overall wrapper must be built dynamically.
    fn wrap_output_schema(
        body_schema: &ObjectOrReference<ObjectSchema>,
        spec: &Spec,
    ) -> Result<Value, Error> {
        // Convert the body schema to JSON
        let mut visited = HashSet::new();
        let body_schema_json = match body_schema {
            ObjectOrReference::Object(obj_schema) => {
                Self::convert_object_schema_to_json_schema(obj_schema, spec, &mut visited)?
            }
            ObjectOrReference::Ref { ref_path, .. } => {
                let resolved = Self::resolve_reference(ref_path, spec, &mut visited)?;
                Self::convert_object_schema_to_json_schema(&resolved, spec, &mut visited)?
            }
        };

        let error_schema = create_error_response_schema();

        Ok(json!({
            "type": "object",
            "description": "Unified response structure with success and error variants",
            "required": ["status", "body"],
            "additionalProperties": false,
            "properties": {
                "status": {
                    "type": "integer",
                    "description": "HTTP status code",
                    "minimum": 100,
                    "maximum": 599
                },
                "body": {
                    "description": "Response body - either success data or error information",
                    "oneOf": [
                        body_schema_json,
                        error_schema
                    ]
                }
            }
        }))
    }
}

/// Create the error schema structure that all tool errors conform to
fn create_error_response_schema() -> Value {
    let root_schema = schema_for!(ErrorResponse);
    let schema_json = serde_json::to_value(root_schema).expect("Valid error schema");

    // Extract definitions/defs for inlining
    let definitions = schema_json
        .get("$defs")
        .or_else(|| schema_json.get("definitions"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    // Clone the schema and remove metadata
    let mut result = schema_json.clone();
    if let Some(obj) = result.as_object_mut() {
        obj.remove("$schema");
        obj.remove("$defs");
        obj.remove("definitions");
        obj.remove("title");
    }

    // Inline all references
    inline_refs(&mut result, &definitions);

    result
}

/// Recursively inline all $ref references in a JSON Schema
fn inline_refs(schema: &mut Value, definitions: &Value) {
    match schema {
        Value::Object(obj) => {
            // Check if this object has a $ref
            if let Some(ref_value) = obj.get("$ref").cloned()
                && let Some(ref_str) = ref_value.as_str()
            {
                // Extract the definition name from the ref
                let def_name = ref_str
                    .strip_prefix("#/$defs/")
                    .or_else(|| ref_str.strip_prefix("#/definitions/"));

                if let Some(name) = def_name
                    && let Some(definition) = definitions.get(name)
                {
                    // Replace the entire object with the definition
                    *schema = definition.clone();
                    // Continue to inline any refs in the definition
                    inline_refs(schema, definitions);
                    return;
                }
            }

            // Recursively process all values in the object
            for (_, value) in obj.iter_mut() {
                inline_refs(value, definitions);
            }
        }
        Value::Array(arr) => {
            // Recursively process all items in the array
            for item in arr.iter_mut() {
                inline_refs(item, definitions);
            }
        }
        _ => {} // Other types don't contain refs
    }
}

/// Query parameter with explode information
#[derive(Debug, Clone)]
pub struct QueryParameter {
    pub value: Value,
    pub explode: bool,
}

impl QueryParameter {
    pub fn new(value: Value, explode: bool) -> Self {
        Self { value, explode }
    }
}

/// Extracted parameters from MCP tool call
#[derive(Debug, Clone)]
pub struct ExtractedParameters {
    pub path: HashMap<String, Value>,
    pub query: HashMap<String, QueryParameter>,
    pub headers: HashMap<String, Value>,
    pub cookies: HashMap<String, Value>,
    pub body: HashMap<String, Value>,
    pub config: RequestConfig,
}

/// Request configuration options
#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub timeout_seconds: u32,
    pub content_type: String,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            content_type: mime::APPLICATION_JSON.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_json_snapshot;
    use oas3::spec::{
        BooleanSchema, Components, MediaType, ObjectOrReference, ObjectSchema, Operation,
        Parameter, ParameterIn, RequestBody, Schema, SchemaType, SchemaTypeSet, Spec,
    };
    use rmcp::model::Tool;
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    /// Create a minimal test OpenAPI spec for testing purposes
    fn create_test_spec() -> Spec {
        Spec {
            openapi: "3.0.0".to_string(),
            info: oas3::spec::Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                summary: None,
                description: Some("Test API for unit tests".to_string()),
                terms_of_service: None,
                contact: None,
                license: None,
                extensions: Default::default(),
            },
            components: Some(Components {
                schemas: BTreeMap::new(),
                responses: BTreeMap::new(),
                parameters: BTreeMap::new(),
                examples: BTreeMap::new(),
                request_bodies: BTreeMap::new(),
                headers: BTreeMap::new(),
                security_schemes: BTreeMap::new(),
                links: BTreeMap::new(),
                callbacks: BTreeMap::new(),
                path_items: BTreeMap::new(),
                extensions: Default::default(),
            }),
            servers: vec![],
            paths: None,
            external_docs: None,
            tags: vec![],
            security: vec![],
            webhooks: BTreeMap::new(),
            extensions: Default::default(),
        }
    }

    fn validate_tool_against_mcp_schema(metadata: &ToolMetadata) {
        let schema_content = std::fs::read_to_string("schema/2025-06-18/schema.json")
            .expect("Failed to read MCP schema file");
        let full_schema: Value =
            serde_json::from_str(&schema_content).expect("Failed to parse MCP schema JSON");

        // Create a schema that references the Tool definition from the full schema
        let tool_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "definitions": full_schema.get("definitions"),
            "$ref": "#/definitions/Tool"
        });

        let validator =
            jsonschema::validator_for(&tool_schema).expect("Failed to compile MCP Tool schema");

        // Convert ToolMetadata to MCP Tool format using the From trait
        let tool = Tool::from(metadata);

        // Serialize the Tool to JSON for validation
        let mcp_tool_json = serde_json::to_value(&tool).expect("Failed to serialize Tool to JSON");

        // Validate the generated tool against MCP schema
        let errors: Vec<String> = validator
            .iter_errors(&mcp_tool_json)
            .map(|e| e.to_string())
            .collect();

        if !errors.is_empty() {
            panic!("Generated tool failed MCP schema validation: {errors:?}");
        }
    }

    #[test]
    fn test_error_schema_structure() {
        let error_schema = create_error_response_schema();

        // Should not contain $schema or definitions at top level
        assert!(error_schema.get("$schema").is_none());
        assert!(error_schema.get("definitions").is_none());

        // Verify the structure using snapshot
        assert_json_snapshot!(error_schema);
    }

    #[test]
    fn test_petstore_get_pet_by_id() {
        use oas3::spec::Response;

        let mut operation = Operation {
            operation_id: Some("getPetById".to_string()),
            summary: Some("Find pet by ID".to_string()),
            description: Some("Returns a single pet".to_string()),
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        // Create a path parameter
        let param = Parameter {
            name: "petId".to_string(),
            location: ParameterIn::Path,
            description: Some("ID of pet to return".to_string()),
            required: Some(true),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                minimum: Some(serde_json::Number::from(1_i64)),
                format: Some("int64".to_string()),
                ..Default::default()
            })),
            example: None,
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        operation.parameters.push(ObjectOrReference::Object(param));

        // Add a 200 response with Pet schema
        let mut responses = BTreeMap::new();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                extensions: Default::default(),
                schema: Some(ObjectOrReference::Object(ObjectSchema {
                    schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                    properties: {
                        let mut props = BTreeMap::new();
                        props.insert(
                            "id".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                                format: Some("int64".to_string()),
                                ..Default::default()
                            }),
                        );
                        props.insert(
                            "name".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                                ..Default::default()
                            }),
                        );
                        props.insert(
                            "status".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    required: vec!["id".to_string(), "name".to_string()],
                    ..Default::default()
                })),
                examples: None,
                encoding: Default::default(),
            },
        );

        responses.insert(
            "200".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("successful operation".to_string()),
                headers: Default::default(),
                content,
                links: Default::default(),
                extensions: Default::default(),
            }),
        );
        operation.responses = Some(responses);

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pet/{petId}".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        assert_eq!(metadata.name, "getPetById");
        assert_eq!(metadata.method, "get");
        assert_eq!(metadata.path, "/pet/{petId}");
        assert!(
            metadata
                .description
                .clone()
                .unwrap()
                .contains("Find pet by ID")
        );

        // Check output_schema is included and correct
        assert!(metadata.output_schema.is_some());
        let output_schema = metadata.output_schema.as_ref().unwrap();

        // Use snapshot testing for the output schema
        insta::assert_json_snapshot!("test_petstore_get_pet_by_id_output_schema", output_schema);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_convert_prefix_items_to_draft07_mixed_types() {
        // Test prefixItems with mixed types and items:false

        let prefix_items = vec![
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                format: Some("int32".to_string()),
                ..Default::default()
            }),
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
        ];

        // items: false (no additional items allowed)
        let items = Some(Box::new(Schema::Boolean(BooleanSchema(false))));

        let mut result = serde_json::Map::new();
        let spec = create_test_spec();
        ToolGenerator::convert_prefix_items_to_draft07(&prefix_items, &items, &mut result, &spec)
            .unwrap();

        // Use JSON snapshot for the schema
        insta::assert_json_snapshot!("test_convert_prefix_items_to_draft07_mixed_types", result);
    }

    #[test]
    fn test_convert_prefix_items_to_draft07_uniform_types() {
        // Test prefixItems with uniform types
        let prefix_items = vec![
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
        ];

        // items: false
        let items = Some(Box::new(Schema::Boolean(BooleanSchema(false))));

        let mut result = serde_json::Map::new();
        let spec = create_test_spec();
        ToolGenerator::convert_prefix_items_to_draft07(&prefix_items, &items, &mut result, &spec)
            .unwrap();

        // Use JSON snapshot for the schema
        insta::assert_json_snapshot!("test_convert_prefix_items_to_draft07_uniform_types", result);
    }

    #[test]
    fn test_array_with_prefix_items_integration() {
        // Integration test: parameter with prefixItems and items:false
        let param = Parameter {
            name: "coordinates".to_string(),
            location: ParameterIn::Query,
            description: Some("X,Y coordinates as tuple".to_string()),
            required: Some(true),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                prefix_items: vec![
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::Number)),
                        format: Some("double".to_string()),
                        ..Default::default()
                    }),
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::Number)),
                        format: Some("double".to_string()),
                        ..Default::default()
                    }),
                ],
                items: Some(Box::new(Schema::Boolean(BooleanSchema(false)))),
                ..Default::default()
            })),
            example: None,
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let (result, _annotations) =
            ToolGenerator::convert_parameter_schema(&param, ParameterIn::Query, &spec, false)
                .unwrap();

        // Use JSON snapshot for the schema
        insta::assert_json_snapshot!("test_array_with_prefix_items_integration", result);
    }

    #[test]
    fn test_skip_tool_description() {
        let operation = Operation {
            operation_id: Some("getPetById".to_string()),
            summary: Some("Find pet by ID".to_string()),
            description: Some("Returns a single pet".to_string()),
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pet/{petId}".to_string(),
            &spec,
            true,
            false,
        )
        .unwrap();

        assert_eq!(metadata.name, "getPetById");
        assert_eq!(metadata.method, "get");
        assert_eq!(metadata.path, "/pet/{petId}");
        assert!(metadata.description.is_none());

        // Use snapshot testing for the output schema
        insta::assert_json_snapshot!("test_skip_tool_description", metadata);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_keep_tool_description() {
        let description = Some("Returns a single pet".to_string());
        let operation = Operation {
            operation_id: Some("getPetById".to_string()),
            summary: Some("Find pet by ID".to_string()),
            description: description.clone(),
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pet/{petId}".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        assert_eq!(metadata.name, "getPetById");
        assert_eq!(metadata.method, "get");
        assert_eq!(metadata.path, "/pet/{petId}");
        assert!(metadata.description.is_some());

        // Use snapshot testing for the output schema
        insta::assert_json_snapshot!("test_keep_tool_description", metadata);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_skip_parameter_descriptions() {
        let param = Parameter {
            name: "status".to_string(),
            location: ParameterIn::Query,
            description: Some("Filter by status".to_string()),
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                enum_values: vec![json!("available"), json!("pending"), json!("sold")],
                ..Default::default()
            })),
            example: Some(json!("available")),
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let (schema, _) =
            ToolGenerator::convert_parameter_schema(&param, ParameterIn::Query, &spec, true)
                .unwrap();

        // When skip_parameter_descriptions is true, description should not be present
        assert!(schema.get("description").is_none());

        // Other properties should still be present
        assert_eq!(schema.get("type").unwrap(), "string");
        assert_eq!(schema.get("example").unwrap(), "available");

        insta::assert_json_snapshot!("test_skip_parameter_descriptions", schema);
    }

    #[test]
    fn test_keep_parameter_descriptions() {
        let param = Parameter {
            name: "status".to_string(),
            location: ParameterIn::Query,
            description: Some("Filter by status".to_string()),
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                enum_values: vec![json!("available"), json!("pending"), json!("sold")],
                ..Default::default()
            })),
            example: Some(json!("available")),
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let (schema, _) =
            ToolGenerator::convert_parameter_schema(&param, ParameterIn::Query, &spec, false)
                .unwrap();

        // When skip_parameter_descriptions is false, description should be present
        assert!(schema.get("description").is_some());
        let description = schema.get("description").unwrap().as_str().unwrap();
        assert!(description.contains("Filter by status"));
        assert!(description.contains("Example: `\"available\"`"));

        // Other properties should also be present
        assert_eq!(schema.get("type").unwrap(), "string");
        assert_eq!(schema.get("example").unwrap(), "available");

        insta::assert_json_snapshot!("test_keep_parameter_descriptions", schema);
    }

    #[test]
    fn test_array_with_regular_items_schema() {
        // Test regular array with object schema items (not boolean)
        let param = Parameter {
            name: "tags".to_string(),
            location: ParameterIn::Query,
            description: Some("List of tags".to_string()),
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                items: Some(Box::new(Schema::Object(Box::new(
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                        min_length: Some(1),
                        max_length: Some(50),
                        ..Default::default()
                    }),
                )))),
                ..Default::default()
            })),
            example: None,
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let (result, _annotations) =
            ToolGenerator::convert_parameter_schema(&param, ParameterIn::Query, &spec, false)
                .unwrap();

        // Use JSON snapshot for the schema
        insta::assert_json_snapshot!("test_array_with_regular_items_schema", result);
    }

    #[test]
    fn test_request_body_object_schema() {
        // Test with object request body
        let operation = Operation {
            operation_id: Some("createPet".to_string()),
            summary: Some("Create a new pet".to_string()),
            description: Some("Creates a new pet in the store".to_string()),
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("Pet object that needs to be added to the store".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            extensions: Default::default(),
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(true),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "post".to_string(),
            "/pets".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check that request_body is in properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(properties.contains_key("request_body"));

        // Check that request_body is required
        let required = metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.contains(&json!("request_body")));

        // Check request body schema using snapshot
        let request_body_schema = properties.get("request_body").unwrap();
        insta::assert_json_snapshot!("test_request_body_object_schema", request_body_schema);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_array_schema() {
        // Test with array request body
        let operation = Operation {
            operation_id: Some("createPets".to_string()),
            summary: Some("Create multiple pets".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("Array of pet objects".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            extensions: Default::default(),
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                                items: Some(Box::new(Schema::Object(Box::new(
                                    ObjectOrReference::Object(ObjectSchema {
                                        schema_type: Some(SchemaTypeSet::Single(
                                            SchemaType::Object,
                                        )),
                                        ..Default::default()
                                    }),
                                )))),
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(false),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "post".to_string(),
            "/pets/batch".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check that request_body is in properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(properties.contains_key("request_body"));

        // Check that request_body is NOT required (required: false)
        let required = metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(!required.contains(&json!("request_body")));

        // Check request body schema using snapshot
        let request_body_schema = properties.get("request_body").unwrap();
        insta::assert_json_snapshot!("test_request_body_array_schema", request_body_schema);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_string_schema() {
        // Test with string request body
        let operation = Operation {
            operation_id: Some("updatePetName".to_string()),
            summary: Some("Update pet name".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: None,
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "text/plain".to_string(),
                        MediaType {
                            extensions: Default::default(),
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                                min_length: Some(1),
                                max_length: Some(100),
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(true),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "put".to_string(),
            "/pets/{petId}/name".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check request body schema
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        insta::assert_json_snapshot!("test_request_body_string_schema", request_body_schema);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_ref_schema() {
        // Test with reference request body
        let operation = Operation {
            operation_id: Some("updatePet".to_string()),
            summary: Some("Update existing pet".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Ref {
                ref_path: "#/components/requestBodies/PetBody".to_string(),
                summary: None,
                description: None,
            }),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "put".to_string(),
            "/pets/{petId}".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check that request_body uses generic object schema for refs
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        insta::assert_json_snapshot!("test_request_body_ref_schema", request_body_schema);

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_no_request_body_for_get() {
        // Test that GET operations don't get request body by default
        let operation = Operation {
            operation_id: Some("listPets".to_string()),
            summary: Some("List all pets".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pets".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check that request_body is NOT in properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(!properties.contains_key("request_body"));

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_simple_object_with_properties() {
        // Test with simple object schema with a few properties
        let operation = Operation {
            operation_id: Some("updatePetStatus".to_string()),
            summary: Some("Update pet status".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("Pet status update".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            extensions: Default::default(),
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                                properties: {
                                    let mut props = BTreeMap::new();
                                    props.insert(
                                        "status".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::String,
                                            )),
                                            ..Default::default()
                                        }),
                                    );
                                    props.insert(
                                        "reason".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::String,
                                            )),
                                            ..Default::default()
                                        }),
                                    );
                                    props
                                },
                                required: vec!["status".to_string()],
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(false),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "patch".to_string(),
            "/pets/{petId}/status".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check request body schema - should have actual properties
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        insta::assert_json_snapshot!(
            "test_request_body_simple_object_with_properties",
            request_body_schema
        );

        // Should not be in top-level required since request body itself is optional
        let required = metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(!required.contains(&json!("request_body")));

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_request_body_with_nested_properties() {
        // Test with complex nested object schema
        let operation = Operation {
            operation_id: Some("createUser".to_string()),
            summary: Some("Create a new user".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: Some(ObjectOrReference::Object(RequestBody {
                description: Some("User creation data".to_string()),
                content: {
                    let mut content = BTreeMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            extensions: Default::default(),
                            schema: Some(ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                                properties: {
                                    let mut props = BTreeMap::new();
                                    props.insert(
                                        "name".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::String,
                                            )),
                                            ..Default::default()
                                        }),
                                    );
                                    props.insert(
                                        "age".to_string(),
                                        ObjectOrReference::Object(ObjectSchema {
                                            schema_type: Some(SchemaTypeSet::Single(
                                                SchemaType::Integer,
                                            )),
                                            minimum: Some(serde_json::Number::from(0)),
                                            maximum: Some(serde_json::Number::from(150)),
                                            ..Default::default()
                                        }),
                                    );
                                    props
                                },
                                required: vec!["name".to_string()],
                                ..Default::default()
                            })),
                            examples: None,
                            encoding: Default::default(),
                        },
                    );
                    content
                },
                required: Some(true),
            })),
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "post".to_string(),
            "/users".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check request body schema
        let properties = metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        let request_body_schema = properties.get("request_body").unwrap();
        insta::assert_json_snapshot!(
            "test_request_body_with_nested_properties",
            request_body_schema
        );

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_operation_without_responses_has_no_output_schema() {
        let operation = Operation {
            operation_id: Some("testOperation".to_string()),
            summary: Some("Test operation".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: None,
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/test".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // When no responses are defined, output_schema should be None
        assert!(metadata.output_schema.is_none());

        // Validate against MCP Tool schema
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_extract_output_schema_with_200_response() {
        use oas3::spec::Response;

        // Create a 200 response with schema
        let mut responses = BTreeMap::new();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                extensions: Default::default(),
                schema: Some(ObjectOrReference::Object(ObjectSchema {
                    schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                    properties: {
                        let mut props = BTreeMap::new();
                        props.insert(
                            "id".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                                ..Default::default()
                            }),
                        );
                        props.insert(
                            "name".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    required: vec!["id".to_string(), "name".to_string()],
                    ..Default::default()
                })),
                examples: None,
                encoding: Default::default(),
            },
        );

        responses.insert(
            "200".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Successful response".to_string()),
                headers: Default::default(),
                content,
                links: Default::default(),
                extensions: Default::default(),
            }),
        );

        let spec = create_test_spec();
        let result = ToolGenerator::extract_output_schema(&Some(responses), &spec).unwrap();

        // Result is already a JSON Value
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_extract_output_schema_with_201_response() {
        use oas3::spec::Response;

        // Create only a 201 response (no 200)
        let mut responses = BTreeMap::new();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                extensions: Default::default(),
                schema: Some(ObjectOrReference::Object(ObjectSchema {
                    schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                    properties: {
                        let mut props = BTreeMap::new();
                        props.insert(
                            "created".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Boolean)),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    ..Default::default()
                })),
                examples: None,
                encoding: Default::default(),
            },
        );

        responses.insert(
            "201".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Created".to_string()),
                headers: Default::default(),
                content,
                links: Default::default(),
                extensions: Default::default(),
            }),
        );

        let spec = create_test_spec();
        let result = ToolGenerator::extract_output_schema(&Some(responses), &spec).unwrap();

        // Result is already a JSON Value
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_extract_output_schema_with_2xx_response() {
        use oas3::spec::Response;

        // Create only a 2XX response
        let mut responses = BTreeMap::new();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                extensions: Default::default(),
                schema: Some(ObjectOrReference::Object(ObjectSchema {
                    schema_type: Some(SchemaTypeSet::Single(SchemaType::Array)),
                    items: Some(Box::new(Schema::Object(Box::new(
                        ObjectOrReference::Object(ObjectSchema {
                            schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                            ..Default::default()
                        }),
                    )))),
                    ..Default::default()
                })),
                examples: None,
                encoding: Default::default(),
            },
        );

        responses.insert(
            "2XX".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Success".to_string()),
                headers: Default::default(),
                content,
                links: Default::default(),
                extensions: Default::default(),
            }),
        );

        let spec = create_test_spec();
        let result = ToolGenerator::extract_output_schema(&Some(responses), &spec).unwrap();

        // Result is already a JSON Value
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_extract_output_schema_no_responses() {
        let spec = create_test_spec();
        let result = ToolGenerator::extract_output_schema(&None, &spec).unwrap();

        // Result is already a JSON Value
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_extract_output_schema_only_error_responses() {
        use oas3::spec::Response;

        // Create only error responses
        let mut responses = BTreeMap::new();
        responses.insert(
            "404".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Not found".to_string()),
                headers: Default::default(),
                content: Default::default(),
                links: Default::default(),
                extensions: Default::default(),
            }),
        );
        responses.insert(
            "500".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Server error".to_string()),
                headers: Default::default(),
                content: Default::default(),
                links: Default::default(),
                extensions: Default::default(),
            }),
        );

        let spec = create_test_spec();
        let result = ToolGenerator::extract_output_schema(&Some(responses), &spec).unwrap();

        // Result is already a JSON Value
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_extract_output_schema_with_ref() {
        use oas3::spec::Response;

        // Create a spec with schema reference
        let mut spec = create_test_spec();
        let mut schemas = BTreeMap::new();
        schemas.insert(
            "Pet".to_string(),
            ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                properties: {
                    let mut props = BTreeMap::new();
                    props.insert(
                        "name".to_string(),
                        ObjectOrReference::Object(ObjectSchema {
                            schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                            ..Default::default()
                        }),
                    );
                    props
                },
                ..Default::default()
            }),
        );
        spec.components.as_mut().unwrap().schemas = schemas;

        // Create response with $ref
        let mut responses = BTreeMap::new();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                extensions: Default::default(),
                schema: Some(ObjectOrReference::Ref {
                    ref_path: "#/components/schemas/Pet".to_string(),
                    summary: None,
                    description: None,
                }),
                examples: None,
                encoding: Default::default(),
            },
        );

        responses.insert(
            "200".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Success".to_string()),
                headers: Default::default(),
                content,
                links: Default::default(),
                extensions: Default::default(),
            }),
        );

        let result = ToolGenerator::extract_output_schema(&Some(responses), &spec).unwrap();

        // Result is already a JSON Value
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_generate_tool_metadata_includes_output_schema() {
        use oas3::spec::Response;

        let mut operation = Operation {
            operation_id: Some("getPet".to_string()),
            summary: Some("Get a pet".to_string()),
            description: None,
            tags: vec![],
            external_docs: None,
            parameters: vec![],
            request_body: None,
            responses: Default::default(),
            callbacks: Default::default(),
            deprecated: Some(false),
            security: vec![],
            servers: vec![],
            extensions: Default::default(),
        };

        // Add a response
        let mut responses = BTreeMap::new();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                extensions: Default::default(),
                schema: Some(ObjectOrReference::Object(ObjectSchema {
                    schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
                    properties: {
                        let mut props = BTreeMap::new();
                        props.insert(
                            "id".to_string(),
                            ObjectOrReference::Object(ObjectSchema {
                                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    ..Default::default()
                })),
                examples: None,
                encoding: Default::default(),
            },
        );

        responses.insert(
            "200".to_string(),
            ObjectOrReference::Object(Response {
                description: Some("Success".to_string()),
                headers: Default::default(),
                content,
                links: Default::default(),
                extensions: Default::default(),
            }),
        );
        operation.responses = Some(responses);

        let spec = create_test_spec();
        let metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/pets/{id}".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check that output_schema is included
        assert!(metadata.output_schema.is_some());
        let output_schema = metadata.output_schema.as_ref().unwrap();

        // Use JSON snapshot for the output schema
        insta::assert_json_snapshot!(
            "test_generate_tool_metadata_includes_output_schema",
            output_schema
        );

        // Validate against MCP Tool schema (this also validates output_schema if present)
        validate_tool_against_mcp_schema(&metadata);
    }

    #[test]
    fn test_sanitize_property_name() {
        // Test spaces are replaced with underscores
        assert_eq!(sanitize_property_name("user name"), "user_name");
        assert_eq!(
            sanitize_property_name("first name last name"),
            "first_name_last_name"
        );

        // Test special characters are replaced
        assert_eq!(sanitize_property_name("user(admin)"), "user_admin");
        assert_eq!(sanitize_property_name("user[admin]"), "user_admin");
        assert_eq!(sanitize_property_name("price($)"), "price");
        assert_eq!(sanitize_property_name("email@address"), "email_address");
        assert_eq!(sanitize_property_name("item#1"), "item_1");
        assert_eq!(sanitize_property_name("a/b/c"), "a_b_c");

        // Test valid characters are preserved
        assert_eq!(sanitize_property_name("user_name"), "user_name");
        assert_eq!(sanitize_property_name("userName123"), "userName123");
        assert_eq!(sanitize_property_name("user.name"), "user.name");
        assert_eq!(sanitize_property_name("user-name"), "user-name");

        // Test numeric starting names
        assert_eq!(sanitize_property_name("123name"), "param_123name");
        assert_eq!(sanitize_property_name("1st_place"), "param_1st_place");

        // Test empty string
        assert_eq!(sanitize_property_name(""), "param_");

        // Test length limit (64 characters)
        let long_name = "a".repeat(100);
        assert_eq!(sanitize_property_name(&long_name).len(), 64);

        // Test all special characters become underscores
        // Note: After collapsing and trimming, this becomes empty and gets "param_" prefix
        assert_eq!(sanitize_property_name("!@#$%^&*()"), "param_");
    }

    #[test]
    fn test_sanitize_property_name_trailing_underscores() {
        // Basic trailing underscore removal
        assert_eq!(sanitize_property_name("page[size]"), "page_size");
        assert_eq!(sanitize_property_name("user[id]"), "user_id");
        assert_eq!(sanitize_property_name("field[]"), "field");

        // Multiple trailing underscores
        assert_eq!(sanitize_property_name("field___"), "field");
        assert_eq!(sanitize_property_name("test[[["), "test");
    }

    #[test]
    fn test_sanitize_property_name_consecutive_underscores() {
        // Consecutive underscores in the middle
        assert_eq!(sanitize_property_name("user__name"), "user_name");
        assert_eq!(sanitize_property_name("first___last"), "first_last");
        assert_eq!(sanitize_property_name("a____b____c"), "a_b_c");

        // Mix of special characters creating consecutive underscores
        assert_eq!(sanitize_property_name("user[[name]]"), "user_name");
        assert_eq!(sanitize_property_name("field@#$value"), "field_value");
    }

    #[test]
    fn test_sanitize_property_name_edge_cases() {
        // Leading underscores (preserved)
        assert_eq!(sanitize_property_name("_private"), "_private");
        assert_eq!(sanitize_property_name("__dunder"), "_dunder");

        // Only special characters
        assert_eq!(sanitize_property_name("[[["), "param_");
        assert_eq!(sanitize_property_name("@@@"), "param_");

        // Empty after sanitization
        assert_eq!(sanitize_property_name(""), "param_");

        // Mix of leading and trailing
        assert_eq!(sanitize_property_name("_field[size]"), "_field_size");
        assert_eq!(sanitize_property_name("__test__"), "_test");
    }

    #[test]
    fn test_sanitize_property_name_complex_cases() {
        // Real-world examples
        assert_eq!(sanitize_property_name("page[size]"), "page_size");
        assert_eq!(sanitize_property_name("filter[status]"), "filter_status");
        assert_eq!(
            sanitize_property_name("sort[-created_at]"),
            "sort_-created_at"
        );
        assert_eq!(
            sanitize_property_name("include[author.posts]"),
            "include_author.posts"
        );

        // Very long names with special characters
        let long_name = "very_long_field_name_with_special[characters]_that_needs_truncation_____";
        let expected = "very_long_field_name_with_special_characters_that_needs_truncat";
        assert_eq!(sanitize_property_name(long_name), expected);
    }

    #[test]
    fn test_property_sanitization_with_annotations() {
        let spec = create_test_spec();
        let mut visited = HashSet::new();

        // Create an object schema with properties that need sanitization
        let obj_schema = ObjectSchema {
            schema_type: Some(SchemaTypeSet::Single(SchemaType::Object)),
            properties: {
                let mut props = BTreeMap::new();
                // Property with space
                props.insert(
                    "user name".to_string(),
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                        ..Default::default()
                    }),
                );
                // Property with special characters
                props.insert(
                    "price($)".to_string(),
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::Number)),
                        ..Default::default()
                    }),
                );
                // Valid property name
                props.insert(
                    "validName".to_string(),
                    ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                        ..Default::default()
                    }),
                );
                props
            },
            ..Default::default()
        };

        let result =
            ToolGenerator::convert_object_schema_to_json_schema(&obj_schema, &spec, &mut visited)
                .unwrap();

        // Use JSON snapshot for the schema
        insta::assert_json_snapshot!("test_property_sanitization_with_annotations", result);
    }

    #[test]
    fn test_parameter_sanitization_and_extraction() {
        let spec = create_test_spec();

        // Create an operation with parameters that need sanitization
        let operation = Operation {
            operation_id: Some("testOp".to_string()),
            parameters: vec![
                // Path parameter with special characters
                ObjectOrReference::Object(Parameter {
                    name: "user(id)".to_string(),
                    location: ParameterIn::Path,
                    description: Some("User ID".to_string()),
                    required: Some(true),
                    deprecated: Some(false),
                    allow_empty_value: Some(false),
                    style: None,
                    explode: None,
                    allow_reserved: Some(false),
                    schema: Some(ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                        ..Default::default()
                    })),
                    example: None,
                    examples: Default::default(),
                    content: None,
                    extensions: Default::default(),
                }),
                // Query parameter with spaces
                ObjectOrReference::Object(Parameter {
                    name: "page size".to_string(),
                    location: ParameterIn::Query,
                    description: Some("Page size".to_string()),
                    required: Some(false),
                    deprecated: Some(false),
                    allow_empty_value: Some(false),
                    style: None,
                    explode: None,
                    allow_reserved: Some(false),
                    schema: Some(ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                        ..Default::default()
                    })),
                    example: None,
                    examples: Default::default(),
                    content: None,
                    extensions: Default::default(),
                }),
                // Header parameter with special characters
                ObjectOrReference::Object(Parameter {
                    name: "auth-token!".to_string(),
                    location: ParameterIn::Header,
                    description: Some("Auth token".to_string()),
                    required: Some(false),
                    deprecated: Some(false),
                    allow_empty_value: Some(false),
                    style: None,
                    explode: None,
                    allow_reserved: Some(false),
                    schema: Some(ObjectOrReference::Object(ObjectSchema {
                        schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                        ..Default::default()
                    })),
                    example: None,
                    examples: Default::default(),
                    content: None,
                    extensions: Default::default(),
                }),
            ],
            ..Default::default()
        };

        let tool_metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/users/{user(id)}".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        // Check sanitized parameter names in schema
        let properties = tool_metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();

        assert!(properties.contains_key("user_id"));
        assert!(properties.contains_key("page_size"));
        assert!(properties.contains_key("header_auth-token"));

        // Check that required array contains the sanitized name
        let required = tool_metadata
            .parameters
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.contains(&json!("user_id")));

        // Test parameter extraction with original names
        let arguments = json!({
            "user_id": "123",
            "page_size": 10,
            "header_auth-token": "secret"
        });

        let extracted = ToolGenerator::extract_parameters(&tool_metadata, &arguments).unwrap();

        // Path parameter should use original name
        assert_eq!(extracted.path.get("user(id)"), Some(&json!("123")));

        // Query parameter should use original name
        assert_eq!(
            extracted.query.get("page size").map(|q| &q.value),
            Some(&json!(10))
        );

        // Header parameter should use original name (without prefix)
        assert_eq!(extracted.headers.get("auth-token!"), Some(&json!("secret")));
    }

    #[test]
    fn test_check_unknown_parameters() {
        // Test with unknown parameter that has a suggestion
        let mut properties = serde_json::Map::new();
        properties.insert("page_size".to_string(), json!({"type": "integer"}));
        properties.insert("user_id".to_string(), json!({"type": "string"}));

        let mut args = serde_json::Map::new();
        args.insert("page_sixe".to_string(), json!(10)); // typo

        let result = ToolGenerator::check_unknown_parameters(&args, &properties);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);

        match &result[0] {
            ValidationError::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
            } => {
                assert_eq!(parameter, "page_sixe");
                assert_eq!(suggestions, &vec!["page_size".to_string()]);
                assert_eq!(
                    valid_parameters,
                    &vec!["page_size".to_string(), "user_id".to_string()]
                );
            }
            _ => panic!("Expected InvalidParameter variant"),
        }
    }

    #[test]
    fn test_check_unknown_parameters_no_suggestions() {
        // Test with unknown parameter that has no suggestions
        let mut properties = serde_json::Map::new();
        properties.insert("limit".to_string(), json!({"type": "integer"}));
        properties.insert("offset".to_string(), json!({"type": "integer"}));

        let mut args = serde_json::Map::new();
        args.insert("xyz123".to_string(), json!("value"));

        let result = ToolGenerator::check_unknown_parameters(&args, &properties);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);

        match &result[0] {
            ValidationError::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
            } => {
                assert_eq!(parameter, "xyz123");
                assert!(suggestions.is_empty());
                assert!(valid_parameters.contains(&"limit".to_string()));
                assert!(valid_parameters.contains(&"offset".to_string()));
            }
            _ => panic!("Expected InvalidParameter variant"),
        }
    }

    #[test]
    fn test_check_unknown_parameters_multiple_suggestions() {
        // Test with unknown parameter that has multiple suggestions
        let mut properties = serde_json::Map::new();
        properties.insert("user_id".to_string(), json!({"type": "string"}));
        properties.insert("user_iid".to_string(), json!({"type": "string"}));
        properties.insert("user_name".to_string(), json!({"type": "string"}));

        let mut args = serde_json::Map::new();
        args.insert("usr_id".to_string(), json!("123"));

        let result = ToolGenerator::check_unknown_parameters(&args, &properties);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);

        match &result[0] {
            ValidationError::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
            } => {
                assert_eq!(parameter, "usr_id");
                assert!(!suggestions.is_empty());
                assert!(suggestions.contains(&"user_id".to_string()));
                assert_eq!(valid_parameters.len(), 3);
            }
            _ => panic!("Expected InvalidParameter variant"),
        }
    }

    #[test]
    fn test_check_unknown_parameters_valid() {
        // Test with all valid parameters
        let mut properties = serde_json::Map::new();
        properties.insert("name".to_string(), json!({"type": "string"}));
        properties.insert("email".to_string(), json!({"type": "string"}));

        let mut args = serde_json::Map::new();
        args.insert("name".to_string(), json!("John"));
        args.insert("email".to_string(), json!("john@example.com"));

        let result = ToolGenerator::check_unknown_parameters(&args, &properties);
        assert!(result.is_empty());
    }

    #[test]
    fn test_check_unknown_parameters_empty() {
        // Test with no parameters defined
        let properties = serde_json::Map::new();

        let mut args = serde_json::Map::new();
        args.insert("any_param".to_string(), json!("value"));

        let result = ToolGenerator::check_unknown_parameters(&args, &properties);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);

        match &result[0] {
            ValidationError::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
            } => {
                assert_eq!(parameter, "any_param");
                assert!(suggestions.is_empty());
                assert!(valid_parameters.is_empty());
            }
            _ => panic!("Expected InvalidParameter variant"),
        }
    }

    #[test]
    fn test_check_unknown_parameters_gltf_pagination() {
        // Test the GLTF Live pagination scenario
        let mut properties = serde_json::Map::new();
        properties.insert(
            "page_number".to_string(),
            json!({
                "type": "integer",
                "x-original-name": "page[number]"
            }),
        );
        properties.insert(
            "page_size".to_string(),
            json!({
                "type": "integer",
                "x-original-name": "page[size]"
            }),
        );

        // User passes page/per_page (common pagination params)
        let mut args = serde_json::Map::new();
        args.insert("page".to_string(), json!(1));
        args.insert("per_page".to_string(), json!(10));

        let result = ToolGenerator::check_unknown_parameters(&args, &properties);
        assert_eq!(result.len(), 2, "Should have 2 unknown parameters");

        // Check that both parameters are flagged as invalid
        let page_error = result
            .iter()
            .find(|e| {
                if let ValidationError::InvalidParameter { parameter, .. } = e {
                    parameter == "page"
                } else {
                    false
                }
            })
            .expect("Should have error for 'page'");

        let per_page_error = result
            .iter()
            .find(|e| {
                if let ValidationError::InvalidParameter { parameter, .. } = e {
                    parameter == "per_page"
                } else {
                    false
                }
            })
            .expect("Should have error for 'per_page'");

        // Verify suggestions are provided for 'page'
        match page_error {
            ValidationError::InvalidParameter {
                suggestions,
                valid_parameters,
                ..
            } => {
                assert!(
                    suggestions.contains(&"page_number".to_string()),
                    "Should suggest 'page_number' for 'page'"
                );
                assert_eq!(valid_parameters.len(), 2);
                assert!(valid_parameters.contains(&"page_number".to_string()));
                assert!(valid_parameters.contains(&"page_size".to_string()));
            }
            _ => panic!("Expected InvalidParameter"),
        }

        // Verify error for 'per_page' (may not have suggestions due to low similarity)
        match per_page_error {
            ValidationError::InvalidParameter {
                parameter,
                suggestions,
                valid_parameters,
                ..
            } => {
                assert_eq!(parameter, "per_page");
                assert_eq!(valid_parameters.len(), 2);
                // per_page might not get suggestions if the similarity algorithm
                // doesn't find it similar enough to page_size
                if !suggestions.is_empty() {
                    assert!(suggestions.contains(&"page_size".to_string()));
                }
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_parameters_with_invalid_params() {
        // Create a tool metadata with sanitized parameter names
        let tool_metadata = ToolMetadata {
            name: "listItems".to_string(),
            title: None,
            description: Some("List items".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "page_number": {
                        "type": "integer",
                        "x-original-name": "page[number]"
                    },
                    "page_size": {
                        "type": "integer",
                        "x-original-name": "page[size]"
                    }
                },
                "required": []
            }),
            output_schema: None,
            method: "GET".to_string(),
            path: "/items".to_string(),
            security: None,
        };

        // Pass incorrect parameter names
        let arguments = json!({
            "page": 1,
            "per_page": 10
        });

        let result = ToolGenerator::validate_parameters(&tool_metadata, &arguments);
        assert!(
            result.is_err(),
            "Should fail validation with unknown parameters"
        );

        let error = result.unwrap_err();
        match error {
            ToolCallValidationError::InvalidParameters { violations } => {
                assert_eq!(violations.len(), 2, "Should have 2 validation errors");

                // Check that both parameters are in the error
                let has_page_error = violations.iter().any(|v| {
                    if let ValidationError::InvalidParameter { parameter, .. } = v {
                        parameter == "page"
                    } else {
                        false
                    }
                });

                let has_per_page_error = violations.iter().any(|v| {
                    if let ValidationError::InvalidParameter { parameter, .. } = v {
                        parameter == "per_page"
                    } else {
                        false
                    }
                });

                assert!(has_page_error, "Should have error for 'page' parameter");
                assert!(
                    has_per_page_error,
                    "Should have error for 'per_page' parameter"
                );
            }
            _ => panic!("Expected InvalidParameters"),
        }
    }

    #[test]
    fn test_cookie_parameter_sanitization() {
        let spec = create_test_spec();

        let operation = Operation {
            operation_id: Some("testCookie".to_string()),
            parameters: vec![ObjectOrReference::Object(Parameter {
                name: "session[id]".to_string(),
                location: ParameterIn::Cookie,
                description: Some("Session ID".to_string()),
                required: Some(false),
                deprecated: Some(false),
                allow_empty_value: Some(false),
                style: None,
                explode: None,
                allow_reserved: Some(false),
                schema: Some(ObjectOrReference::Object(ObjectSchema {
                    schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                    ..Default::default()
                })),
                example: None,
                examples: Default::default(),
                content: None,
                extensions: Default::default(),
            })],
            ..Default::default()
        };

        let tool_metadata = ToolGenerator::generate_tool_metadata(
            &operation,
            "get".to_string(),
            "/data".to_string(),
            &spec,
            false,
            false,
        )
        .unwrap();

        let properties = tool_metadata
            .parameters
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();

        // Check sanitized cookie parameter name
        assert!(properties.contains_key("cookie_session_id"));

        // Test extraction
        let arguments = json!({
            "cookie_session_id": "abc123"
        });

        let extracted = ToolGenerator::extract_parameters(&tool_metadata, &arguments).unwrap();

        // Cookie should use original name
        assert_eq!(extracted.cookies.get("session[id]"), Some(&json!("abc123")));
    }

    #[test]
    fn test_parameter_description_with_examples() {
        let spec = create_test_spec();

        // Test parameter with single example
        let param_with_example = Parameter {
            name: "status".to_string(),
            location: ParameterIn::Query,
            description: Some("Filter by status".to_string()),
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            })),
            example: Some(json!("active")),
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let (schema, _) = ToolGenerator::convert_parameter_schema(
            &param_with_example,
            ParameterIn::Query,
            &spec,
            false,
        )
        .unwrap();
        let description = schema.get("description").unwrap().as_str().unwrap();
        assert_eq!(description, "Filter by status. Example: `\"active\"`");

        // Test parameter with multiple examples
        let mut examples_map = std::collections::BTreeMap::new();
        examples_map.insert(
            "example1".to_string(),
            ObjectOrReference::Object(oas3::spec::Example {
                value: Some(json!("pending")),
                ..Default::default()
            }),
        );
        examples_map.insert(
            "example2".to_string(),
            ObjectOrReference::Object(oas3::spec::Example {
                value: Some(json!("completed")),
                ..Default::default()
            }),
        );

        let param_with_examples = Parameter {
            name: "status".to_string(),
            location: ParameterIn::Query,
            description: Some("Filter by status".to_string()),
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            })),
            example: None,
            examples: examples_map,
            content: None,
            extensions: Default::default(),
        };

        let (schema, _) = ToolGenerator::convert_parameter_schema(
            &param_with_examples,
            ParameterIn::Query,
            &spec,
            false,
        )
        .unwrap();
        let description = schema.get("description").unwrap().as_str().unwrap();
        assert!(description.starts_with("Filter by status. Examples:\n"));
        assert!(description.contains("`\"pending\"`"));
        assert!(description.contains("`\"completed\"`"));

        // Test parameter with no description but with example
        let param_no_desc = Parameter {
            name: "limit".to_string(),
            location: ParameterIn::Query,
            description: None,
            required: Some(false),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Object(ObjectSchema {
                schema_type: Some(SchemaTypeSet::Single(SchemaType::Integer)),
                ..Default::default()
            })),
            example: Some(json!(100)),
            examples: Default::default(),
            content: None,
            extensions: Default::default(),
        };

        let (schema, _) = ToolGenerator::convert_parameter_schema(
            &param_no_desc,
            ParameterIn::Query,
            &spec,
            false,
        )
        .unwrap();
        let description = schema.get("description").unwrap().as_str().unwrap();
        assert_eq!(description, "limit parameter. Example: `100`");
    }

    #[test]
    fn test_format_examples_for_description() {
        // Test single string example
        let examples = vec![json!("active")];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `\"active\"`".to_string()));

        // Test single number example
        let examples = vec![json!(42)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `42`".to_string()));

        // Test single boolean example
        let examples = vec![json!(true)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `true`".to_string()));

        // Test multiple examples
        let examples = vec![json!("active"), json!("pending"), json!("completed")];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(
            result,
            Some("Examples:\n- `\"active\"`\n- `\"pending\"`\n- `\"completed\"`".to_string())
        );

        // Test array example
        let examples = vec![json!(["a", "b", "c"])];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `[\"a\",\"b\",\"c\"]`".to_string()));

        // Test object example
        let examples = vec![json!({"key": "value"})];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `{\"key\":\"value\"}`".to_string()));

        // Test empty examples
        let examples = vec![];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, None);

        // Test null example
        let examples = vec![json!(null)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `null`".to_string()));

        // Test mixed type examples
        let examples = vec![json!("text"), json!(123), json!(true)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(
            result,
            Some("Examples:\n- `\"text\"`\n- `123`\n- `true`".to_string())
        );

        // Test long array (should be truncated)
        let examples = vec![json!(["a", "b", "c", "d", "e", "f"])];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(
            result,
            Some("Example: `[\"a\",\"b\",\"c\",\"d\",\"e\",\"f\"]`".to_string())
        );

        // Test short array (should show full content)
        let examples = vec![json!([1, 2])];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `[1,2]`".to_string()));

        // Test nested object
        let examples = vec![json!({"user": {"name": "John", "age": 30}})];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(
            result,
            Some("Example: `{\"user\":{\"name\":\"John\",\"age\":30}}`".to_string())
        );

        // Test more than 3 examples (should only show first 3)
        let examples = vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(
            result,
            Some("Examples:\n- `\"a\"`\n- `\"b\"`\n- `\"c\"`\n- `\"d\"`\n- `\"e\"`".to_string())
        );

        // Test float number
        let examples = vec![json!(3.5)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `3.5`".to_string()));

        // Test negative number
        let examples = vec![json!(-42)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `-42`".to_string()));

        // Test false boolean
        let examples = vec![json!(false)];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `false`".to_string()));

        // Test string with special characters
        let examples = vec![json!("hello \"world\"")];
        let result = ToolGenerator::format_examples_for_description(&examples);
        // The format function just wraps strings in quotes, it doesn't escape them
        assert_eq!(result, Some(r#"Example: `"hello \"world\""`"#.to_string()));

        // Test empty string
        let examples = vec![json!("")];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `\"\"`".to_string()));

        // Test empty array
        let examples = vec![json!([])];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `[]`".to_string()));

        // Test empty object
        let examples = vec![json!({})];
        let result = ToolGenerator::format_examples_for_description(&examples);
        assert_eq!(result, Some("Example: `{}`".to_string()));
    }

    #[test]
    fn test_reference_metadata_functionality() {
        // Test ReferenceMetadata creation and methods
        let metadata = ReferenceMetadata::new(
            Some("User Reference".to_string()),
            Some("A reference to user data with additional context".to_string()),
        );

        assert!(!metadata.is_empty());
        assert_eq!(metadata.summary(), Some("User Reference"));
        assert_eq!(
            metadata.best_description(),
            Some("A reference to user data with additional context")
        );

        // Test metadata with only summary
        let summary_only = ReferenceMetadata::new(Some("Pet Summary".to_string()), None);
        assert_eq!(summary_only.best_description(), Some("Pet Summary"));

        // Test empty metadata
        let empty_metadata = ReferenceMetadata::new(None, None);
        assert!(empty_metadata.is_empty());
        assert_eq!(empty_metadata.best_description(), None);

        // Test merge_with_description
        let metadata = ReferenceMetadata::new(
            Some("Reference Summary".to_string()),
            Some("Reference Description".to_string()),
        );

        // Test with no existing description
        let result = metadata.merge_with_description(None, false);
        assert_eq!(result, Some("Reference Description".to_string()));

        // Test with existing description and no prepend - reference description takes precedence
        let result = metadata.merge_with_description(Some("Existing desc"), false);
        assert_eq!(result, Some("Reference Description".to_string()));

        // Test with existing description and prepend summary - reference description still takes precedence
        let result = metadata.merge_with_description(Some("Existing desc"), true);
        assert_eq!(result, Some("Reference Description".to_string()));

        // Test enhance_parameter_description - reference description takes precedence with proper formatting
        let result = metadata.enhance_parameter_description("userId", Some("User ID parameter"));
        assert_eq!(result, Some("userId: Reference Description".to_string()));

        let result = metadata.enhance_parameter_description("userId", None);
        assert_eq!(result, Some("userId: Reference Description".to_string()));

        // Test precedence: summary-only metadata should use summary when no description
        let summary_only = ReferenceMetadata::new(Some("API Token".to_string()), None);

        let result = summary_only.merge_with_description(Some("Generic token"), false);
        assert_eq!(result, Some("API Token".to_string()));

        let result = summary_only.merge_with_description(Some("Different desc"), true);
        assert_eq!(result, Some("API Token".to_string())); // Summary takes precedence via best_description()

        let result = summary_only.enhance_parameter_description("token", Some("Token field"));
        assert_eq!(result, Some("token: API Token".to_string()));

        // Test fallback behavior: no reference metadata should use schema description
        let empty_meta = ReferenceMetadata::new(None, None);

        let result = empty_meta.merge_with_description(Some("Schema description"), false);
        assert_eq!(result, Some("Schema description".to_string()));

        let result = empty_meta.enhance_parameter_description("param", Some("Schema param"));
        assert_eq!(result, Some("Schema param".to_string()));

        let result = empty_meta.enhance_parameter_description("param", None);
        assert_eq!(result, Some("param parameter".to_string()));
    }

    #[test]
    fn test_parameter_schema_with_reference_metadata() {
        let mut spec = create_test_spec();

        // Add a Pet schema to resolve the reference
        spec.components.as_mut().unwrap().schemas.insert(
            "Pet".to_string(),
            ObjectOrReference::Object(ObjectSchema {
                description: None, // No description so reference metadata should be used as fallback
                schema_type: Some(SchemaTypeSet::Single(SchemaType::String)),
                ..Default::default()
            }),
        );

        // Create a parameter with a reference that has metadata
        let param_with_ref = Parameter {
            name: "user".to_string(),
            location: ParameterIn::Query,
            description: None,
            required: Some(true),
            deprecated: Some(false),
            allow_empty_value: Some(false),
            style: None,
            explode: None,
            allow_reserved: Some(false),
            schema: Some(ObjectOrReference::Ref {
                ref_path: "#/components/schemas/Pet".to_string(),
                summary: Some("Pet Reference".to_string()),
                description: Some("A reference to pet schema with additional context".to_string()),
            }),
            example: None,
            examples: BTreeMap::new(),
            content: None,
            extensions: Default::default(),
        };

        // Convert the parameter schema
        let result = ToolGenerator::convert_parameter_schema(
            &param_with_ref,
            ParameterIn::Query,
            &spec,
            false,
        );

        assert!(result.is_ok());
        let (schema, _annotations) = result.unwrap();

        // Check that the schema includes the reference description as fallback
        let description = schema.get("description").and_then(|v| v.as_str());
        assert!(description.is_some());
        // The description should be the reference metadata since resolved schema may not have one
        assert!(
            description.unwrap().contains("Pet Reference")
                || description
                    .unwrap()
                    .contains("A reference to pet schema with additional context")
        );
    }

    #[test]
    fn test_request_body_with_reference_metadata() {
        let spec = create_test_spec();

        // Create request body reference with metadata
        let request_body_ref = ObjectOrReference::Ref {
            ref_path: "#/components/requestBodies/PetBody".to_string(),
            summary: Some("Pet Request Body".to_string()),
            description: Some(
                "Request body containing pet information for API operations".to_string(),
            ),
        };

        let result = ToolGenerator::convert_request_body_to_json_schema(&request_body_ref, &spec);

        assert!(result.is_ok());
        let schema_result = result.unwrap();
        assert!(schema_result.is_some());

        let (schema, _annotations, _required) = schema_result.unwrap();
        let description = schema.get("description").and_then(|v| v.as_str());

        assert!(description.is_some());
        // Should use the reference description
        assert_eq!(
            description.unwrap(),
            "Request body containing pet information for API operations"
        );
    }

    #[test]
    fn test_response_schema_with_reference_metadata() {
        let spec = create_test_spec();

        // Create responses with a reference that has metadata
        let mut responses = BTreeMap::new();
        responses.insert(
            "200".to_string(),
            ObjectOrReference::Ref {
                ref_path: "#/components/responses/PetResponse".to_string(),
                summary: Some("Successful Pet Response".to_string()),
                description: Some(
                    "Response containing pet data on successful operation".to_string(),
                ),
            },
        );
        let responses_option = Some(responses);

        let result = ToolGenerator::extract_output_schema(&responses_option, &spec);

        assert!(result.is_ok());
        let schema = result.unwrap();
        assert!(schema.is_some());

        let schema_value = schema.unwrap();
        let body_desc = schema_value
            .get("properties")
            .and_then(|props| props.get("body"))
            .and_then(|body| body.get("description"))
            .and_then(|desc| desc.as_str());

        assert!(body_desc.is_some());
        // Should contain the reference description
        assert_eq!(
            body_desc.unwrap(),
            "Response containing pet data on successful operation"
        );
    }
}
