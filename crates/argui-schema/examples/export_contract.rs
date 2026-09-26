//! Exports the canonical native schema for JavaScript code generation.

use argui_schema::builtin;

/// Writes the schema contract to stdout for the TypeScript generator.
///
/// # Errors
///
/// Returns a schema or JSON serialization error when export fails.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&builtin::registry()?.contract())?
    );
    Ok(())
}
