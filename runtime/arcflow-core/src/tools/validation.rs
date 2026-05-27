//! JSON Schema validation for tool inputs.

use jsonschema::{Draft, Validator};
use serde_json::Value;

use super::error::ToolError;

/// Validates `input` against `schema`. Returns Ok on success.
pub fn validate_tool_input(name: &str, schema: &Value, input: &Value) -> Result<(), ToolError> {
    let validator = Validator::options()
        .with_draft(Draft::Draft7)
        .build(schema)
        .map_err(|e| ToolError::ValidationFailed {
            name: name.to_string(),
            reason: format!("invalid schema: {e}"),
        })?;
    if let Err(error) = validator.validate(input) {
        return Err(ToolError::ValidationFailed {
            name: name.to_string(),
            reason: error.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_object_passes() {
        let schema = json!({"type": "object", "properties": {"q": {"type": "string"}}, "required": ["q"]});
        let input = json!({"q": "hello"});
        validate_tool_input("t", &schema, &input).unwrap();
    }

    #[test]
    fn missing_required_field_fails() {
        let schema = json!({"type": "object", "required": ["q"]});
        let err = validate_tool_input("t", &schema, &json!({})).unwrap_err();
        assert!(matches!(err, ToolError::ValidationFailed { .. }));
    }
}
