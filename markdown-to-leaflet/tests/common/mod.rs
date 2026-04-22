#![allow(dead_code)]

use markdown_to_leaflet::parse_markdown_to_leaflet;

/// Convert Markdown source into a JSON [`Value`].
pub fn convert(md: &str) -> serde_json::Value {
    let schema = parse_markdown_to_leaflet(md.as_bytes(), "test.md").unwrap();
    leaflet_protocol::emit_leaflet_document(&schema).unwrap()
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

/// Return the `level` of a header block.
pub fn level(block: &serde_json::Value) -> u64 {
    block
        .get("block")
        .and_then(|b| b.get("level"))
        .and_then(|l| l.as_u64())
        .unwrap_or(0)
}

/// Return the `language` of a code block.
pub fn language(block: &serde_json::Value) -> &str {
    block
        .get("block")
        .and_then(|b| b.get("language"))
        .and_then(|l| l.as_str())
        .unwrap_or("")
}

/// Return the `src` of an image block.
pub fn src(block: &serde_json::Value) -> &str {
    block
        .get("block")
        .and_then(|b| b.get("src"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
}

/// Return children array of a list block.
pub fn list_children(block: &serde_json::Value) -> &Vec<serde_json::Value> {
    block
        .get("block")
        .and_then(|b| b.get("children"))
        .and_then(|c| c.as_array())
        .expect("list children")
}
