use crate::models::SystemState;
use jsonschema::{Draft, JSONSchema};

pub fn validate_against_contract(state: &SystemState) -> Result<(), String> {
    let schema: serde_json::Value = serde_json::from_str(include_str!("../../scan.schema.json"))
        .map_err(|e| format!("Schema parse error: {e}"))?;
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .map_err(|e| format!("Schema compile error: {e}"))?;
    let instance =
        serde_json::to_value(state).map_err(|e| format!("State serialization failed: {e}"))?;
    if let Err(errors) = compiled.validate(&instance) {
        let mut messages: Vec<String> = errors.map(|e| e.to_string()).collect();
        messages.sort();
        messages.dedup();
        return Err(format!("Schema validation failed: {}", messages.join(", ")));
    }
    Ok(())
}
