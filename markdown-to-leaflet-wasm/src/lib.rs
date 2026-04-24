//! Markdown → Leaflet WASM bindings.

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Convert Markdown source into a Leaflet document JSON object.
///
/// # Errors
/// Returns a JS Error if parsing or lens application fails.
#[wasm_bindgen]
pub fn convert_markdown(source: &str) -> Result<JsValue, JsValue> {
    let schema = markdown_to_leaflet::markdown_to_leaflet_schema(source.as_bytes(), "<wasm>")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let json = leaflet_protocol::emit_leaflet_document(&schema)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    json.serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
