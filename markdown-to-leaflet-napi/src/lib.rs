//! Markdown → Leaflet N-API bindings.

use napi_derive::napi;

/// Convert Markdown source into a Leaflet document JSON object.
///
/// # Errors
/// Returns a JS Error if parsing or lens application fails.
#[napi]
pub fn markdown_to_leaflet(source: String) -> napi::Result<serde_json::Value> {
    let schema = markdown_to_leaflet::markdown_to_leaflet_schema(source.as_bytes(), "<input>")
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let json = leaflet_protocol::emit_leaflet_document(&schema)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(json)
}
