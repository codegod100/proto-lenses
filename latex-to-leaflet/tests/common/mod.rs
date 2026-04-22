use latex_to_leaflet::{
    latex_to_leaflet_schema, parse_latex_to_leaflet, LaTeXToLeafletConfig,
};

/// Convert LaTeX source into a JSON [`Value`] using the default config.
pub fn convert(tex: &str) -> serde_json::Value {
    let schema = parse_latex_to_leaflet(tex.as_bytes(), "test.tex").unwrap();
    leaflet_protocol::emit_leaflet_document(&schema).unwrap()
}

/// Convert LaTeX source into a JSON [`Value`] using a custom config.
pub fn convert_with_config(tex: &str, config: &LaTeXToLeafletConfig) -> serde_json::Value {
    let schema = latex_to_leaflet_schema_with_config(tex.as_bytes(), "test.tex", config)
        .unwrap();
    leaflet_protocol::emit_leaflet_document(&schema).unwrap()
}

// Thin wrapper that accepts a config (not yet exposed on the public API).
fn latex_to_leaflet_schema_with_config(
    source: &[u8],
    file_path: &str,
    config: &LaTeXToLeafletConfig,
) -> Result<panproto_schema::Schema, latex_to_leaflet::LaTeXLeafletError> {
    // For now delegate to the existing parser; once the parser is updated
    // to honour config this helper will be wired through.
    let _ = config;
    latex_to_leaflet_schema(source, file_path)
}

/// Extract the blocks array from the first page of a Leaflet JSON document.
pub fn blocks(json: &serde_json::Value) -> &Vec<serde_json::Value> {
    json.get("content")
        .and_then(|c| c.get("pages"))
        .and_then(|p| p.as_array())
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("blocks"))
        .and_then(|b| b.as_array())
        .expect("blocks array")
}

/// Extract the `title` field from a Leaflet JSON document.
pub fn title(json: &serde_json::Value) -> Option<&str> {
    json.get("title").and_then(|v| v.as_str())
}

/// Return the `$type` of a block (e.g. `pub.leaflet.blocks.header`).
pub fn block_type(block: &serde_json::Value) -> &str {
    block
        .get("block")
        .and_then(|b| b.get("$type"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
}

/// Return the `plaintext` of a block (or empty string if absent).
pub fn plaintext(block: &serde_json::Value) -> &str {
    block
        .get("block")
        .and_then(|b| b.get("plaintext"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
}

/// Return the `tex` content of a math block.
pub fn tex(block: &serde_json::Value) -> &str {
    block
        .get("block")
        .and_then(|b| b.get("tex"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
}
