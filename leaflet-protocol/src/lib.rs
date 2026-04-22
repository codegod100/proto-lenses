//! Leaflet.pub document protocol definition.
//!
//! Leaflet is a long-form blogging platform built on ATProto. This module
//! defines the schema protocol for Leaflet documents, mapping the
//! `site.standard.document` ATProto record type to panproto's graph model.
//!
//! ## Vertex kinds (block types)
//!
//! | Kind | Leaflet `$type` | Description |
//! |------|-----------------|-------------|
//! | `document` | `site.standard.document` | Root document |
//! | `page` | `pub.leaflet.pages.linearDocument` | A page of blocks |
//! | `header` | `pub.leaflet.blocks.header` | Heading block |
//! | `text` | `pub.leaflet.blocks.text` | Paragraph / inline text |
//! | `blockquote` | `pub.leaflet.blocks.blockquote` | Quotation block |
//! | `code` | `pub.leaflet.blocks.code` | Code / verbatim block |
//! | `website` | `pub.leaflet.blocks.website` | Link preview / embed |
//! | `image` | `pub.leaflet.blocks.image` | Image block |
//! | `horizontalRule` | `pub.leaflet.blocks.horizontalRule` | Thematic break |
//! | `orderedList` | `pub.leaflet.blocks.orderedList` | Numbered list |
//! | `unorderedList` | `pub.leaflet.blocks.unorderedList` | Bullet list |
//! | `listItem` | `pub.leaflet.blocks.orderedList#listItem` / `#unorderedList#listItem` | List entry |
//! | `bskyPost` | `pub.leaflet.blocks.bskyPost` | Bluesky post embed |
//! | `math` | `pub.leaflet.blocks.math` | LaTeX math block |

use std::collections::HashMap;
use std::hash::BuildHasher;

use panproto_gat::Theory;
use panproto_protocols::emit::{children_by_edge, find_roots, vertex_constraints};
use panproto_protocols::error::ProtocolError;
use panproto_protocols::theories;
use panproto_schema::{EdgeRule, Protocol, Schema, SchemaBuilder};

/// Returns the Leaflet document protocol definition.
#[must_use]
pub fn protocol() -> Protocol {
    Protocol {
        name: "leaflet".into(),
        schema_theory: "ThLeafletSchema".into(),
        instance_theory: "ThLeafletInstance".into(),
        edge_rules: edge_rules(),
        obj_kinds: vec![
            "document".into(),
            "page".into(),
            "header".into(),
            "text".into(),
            "blockquote".into(),
            "code".into(),
            "website".into(),
            "image".into(),
            "horizontalRule".into(),
            "orderedList".into(),
            "unorderedList".into(),
            "listItem".into(),
            "bskyPost".into(),
            "math".into(),
        ],
        constraint_sorts: vec![
            "level".into(),
            "language".into(),
            "plaintext".into(),
            "src".into(),
            "title".into(),
            "description".into(),
            "tex".into(),
            "startIndex".into(),
            "facet".into(),
        ],
        has_order: true,
        nominal_identity: true,
        ..Protocol::default()
    }
}

/// Register the component GATs for Leaflet.
///
/// Uses Group E (constrained multigraph + W-type + metadata), same as
/// DOCX and ODF, because Leaflet is a document format with ordered
/// blocks and metadata.
pub fn register_theories<S: BuildHasher>(registry: &mut HashMap<String, Theory, S>) {
    theories::register_multigraph_wtype_meta(registry, "ThLeafletSchema", "ThLeafletInstance");
}

// ---------------------------------------------------------------------------
// Parser / emitter helpers
// ---------------------------------------------------------------------------

/// Parse a `site.standard.document` JSON record into a [`Schema`].
///
/// # Errors
///
/// Returns [`ProtocolError`] if parsing fails.
pub fn parse_leaflet_document(json: &serde_json::Value) -> Result<Schema, ProtocolError> {
    let proto = protocol();
    let mut builder = SchemaBuilder::new(&proto);

    // Create the root document vertex.
    builder = builder.vertex("document", "document", None)?;

    // Extract top-level fields.
    if let Some(title) = json.get("title").and_then(serde_json::Value::as_str) {
        builder = builder.constraint("document", "title", title);
    }
    if let Some(desc) = json.get("description").and_then(serde_json::Value::as_str) {
        builder = builder.constraint("document", "description", desc);
    }

    // Parse content.pages.
    if let Some(content) = json.get("content").and_then(serde_json::Value::as_object) {
        if let Some(pages) = content.get("pages").and_then(serde_json::Value::as_array) {
            for (page_idx, page) in pages.iter().enumerate() {
                let page_id = format!("page:{page_idx}");
                builder = builder.vertex(&page_id, "page", None)?;
                builder = builder.edge("document", &page_id, "items", None)?;

                if let Some(blocks) = page
                    .get("blocks")
                    .and_then(serde_json::Value::as_array)
                {
                for (block_idx, block) in blocks.iter().enumerate() {
                    builder = parse_block(builder, &page_id, block_idx, block)?;
                }
                }
            }
        }
    }

    Ok(builder.build()?)
}

fn parse_block(
    mut builder: SchemaBuilder,
    parent_id: &str,
    idx: usize,
    block: &serde_json::Value,
) -> Result<SchemaBuilder, ProtocolError> {
    let block_obj = block
        .get("block")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| ProtocolError::MissingField("block".into()))?;

    let block_type = block_obj
        .get("$type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ProtocolError::MissingField("$type".into()))?;

    let block_id = format!("{parent_id}:block:{idx}");

    match block_type {
        "pub.leaflet.blocks.header" => {
            let level = block_obj
                .get("level")
                .and_then(|v| v.as_u64())
                .unwrap_or(2);
            let plaintext = block_obj
                .get("plaintext")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "header", None)?;
            builder = builder.constraint(&block_id, "level", &level.to_string());
            builder = builder.constraint(&block_id, "plaintext", plaintext);
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.text" => {
            let plaintext = block_obj
                .get("plaintext")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "text", None)?;
            builder = builder.constraint(&block_id, "plaintext", plaintext);
            // Facets are stored as a JSON blob on the `facet` constraint.
            if let Some(facets) = block_obj.get("facets") {
                builder = builder.constraint(
                    &block_id,
                    "facet",
                    &serde_json::to_string(facets).unwrap_or_default(),
                );
            }
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.blockquote" => {
            let plaintext = block_obj
                .get("plaintext")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "blockquote", None)?;
            builder = builder.constraint(&block_id, "plaintext", plaintext);
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.code" => {
            let language = block_obj
                .get("language")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let plaintext = block_obj
                .get("plaintext")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "code", None)?;
            builder = builder.constraint(&block_id, "language", language);
            builder = builder.constraint(&block_id, "plaintext", plaintext);
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.website" => {
            let src = block_obj
                .get("src")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let title = block_obj
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let description = block_obj
                .get("description")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "website", None)?;
            builder = builder.constraint(&block_id, "src", src);
            builder = builder.constraint(&block_id, "title", title);
            builder = builder.constraint(&block_id, "description", description);
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.image" => {
            // Image blocks use `blob` references in practice, but for schema
            // modelling we store a `src` constraint for any URL or blob ref.
            let src = block_obj
                .get("src")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "image", None)?;
            builder = builder.constraint(&block_id, "src", src);
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.horizontalRule" => {
            builder = builder.vertex(&block_id, "horizontalRule", None)?;
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.orderedList" => {
            let start_index = block_obj
                .get("startIndex")
                .and_then(|v| v.as_u64())
                .unwrap_or(1);
            builder = builder.vertex(&block_id, "orderedList", None)?;
            builder = builder.constraint(&block_id, "startIndex", &start_index.to_string());
            builder = builder.edge(parent_id, &block_id, "items", None)?;
            if let Some(children) = block_obj.get("children").and_then(|v| v.as_array()) {
                for (child_idx, child) in children.iter().enumerate() {
                    let child_id = format!("{block_id}:item:{child_idx}");
                    let child_block = child
                        .get("$type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("pub.leaflet.blocks.orderedList#listItem");
                    if child_block.ends_with("listItem") {
                        builder = builder.vertex(&child_id, "listItem", None)?;
                        builder = builder.edge(&block_id, &child_id, "items", None)?;
                        // Recurse into nested content if present.
                        if let Some(content) = child.get("content").and_then(|v| v.as_object()) {
                            builder = parse_block(
                                builder,
                                &child_id,
                                0,
                                &serde_json::json!({ "block": content }),
                            )?;
                        }
                    }
                }
            }
        }
        "pub.leaflet.blocks.unorderedList" => {
            builder = builder.vertex(&block_id, "unorderedList", None)?;
            builder = builder.edge(parent_id, &block_id, "items", None)?;
            if let Some(children) = block_obj.get("children").and_then(|v| v.as_array()) {
                for (child_idx, child) in children.iter().enumerate() {
                    let child_id = format!("{block_id}:item:{child_idx}");
                    let child_block = child
                        .get("$type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("pub.leaflet.blocks.unorderedList#listItem");
                    if child_block.ends_with("listItem") {
                        builder = builder.vertex(&child_id, "listItem", None)?;
                        builder = builder.edge(&block_id, &child_id, "items", None)?;
                        if let Some(content) = child.get("content").and_then(|v| v.as_object()) {
                            builder = parse_block(
                                builder,
                                &child_id,
                                0,
                                &serde_json::json!({ "block": content }),
                            )?;
                        }
                    }
                }
            }
        }
        "pub.leaflet.blocks.bskyPost" => {
            builder = builder.vertex(&block_id, "bskyPost", None)?;
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        "pub.leaflet.blocks.math" => {
            let tex = block_obj
                .get("tex")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            builder = builder.vertex(&block_id, "math", None)?;
            builder = builder.constraint(&block_id, "tex", tex);
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
        _ => {
            // Unknown block type: ignore or map to generic text.
            // For robustness, treat as a text block with raw JSON.
            builder = builder.vertex(&block_id, "text", None)?;
            builder = builder.constraint(
                &block_id,
                "plaintext",
                &serde_json::to_string(block_obj).unwrap_or_default(),
            );
            builder = builder.edge(parent_id, &block_id, "items", None)?;
        }
    }

    Ok(builder)
}

/// Emit a [`Schema`] as a `site.standard.document` JSON record.
///
/// # Errors
///
/// Returns [`ProtocolError`] if emission fails.
pub fn emit_leaflet_document(schema: &Schema) -> Result<serde_json::Value, ProtocolError> {
    let structural = &["items"];
    let roots = find_roots(schema, structural);

    let root = roots.first().ok_or_else(|| {
        ProtocolError::Emit("no document root found".into())
    })?;

    let mut document = serde_json::Map::new();
    document.insert("$type".into(), serde_json::json!("site.standard.document"));

    // Root constraints become top-level fields.
    for c in vertex_constraints(schema, &root.id) {
        match c.sort.as_ref() {
            "title" => {
                document.insert("title".into(), serde_json::json!(c.value));
            }
            "description" => {
                document.insert("description".into(), serde_json::json!(c.value));
            }
            _ => {}
        }
    }

    let mut pages = Vec::new();
    let mut page_children = children_by_edge(schema, &root.id, "items");
    page_children.sort_by(|a, b| vertex_order_key(&a.1.id).cmp(&vertex_order_key(&b.1.id)));

    for (_edge, page) in &page_children {
        if page.kind != "page" {
            continue;
        }

        let mut blocks = Vec::new();
        let mut block_children = children_by_edge(schema, &page.id, "items");
        block_children.sort_by(|a, b| vertex_order_key(&a.1.id).cmp(&vertex_order_key(&b.1.id)));

        for (_edge, block) in &block_children {
            if let Ok(block_json) = emit_block(schema, block) {
                blocks.push(block_json);
            }
        }

        let mut page_obj = serde_json::Map::new();
        page_obj.insert("id".into(), serde_json::json!(page.id));
        page_obj.insert(
            "$type".into(),
            serde_json::json!("pub.leaflet.pages.linearDocument"),
        );
        page_obj.insert("blocks".into(), serde_json::Value::Array(blocks));
        pages.push(serde_json::Value::Object(page_obj));
    }

    let mut content = serde_json::Map::new();
    content.insert("$type".into(), serde_json::json!("pub.leaflet.content"));
    content.insert("pages".into(), serde_json::Value::Array(pages));
    document.insert("content".into(), serde_json::Value::Object(content));

    Ok(serde_json::Value::Object(document))
}

fn emit_block(schema: &Schema, block: &panproto_schema::Vertex) -> Result<serde_json::Value, ProtocolError> {
    let constraints = vertex_constraints(schema, &block.id);
    let mut block_map = serde_json::Map::new();

    let block_type = match block.kind.as_ref() {
        "header" => "pub.leaflet.blocks.header",
        "text" => "pub.leaflet.blocks.text",
        "blockquote" => "pub.leaflet.blocks.blockquote",
        "code" => "pub.leaflet.blocks.code",
        "website" => "pub.leaflet.blocks.website",
        "image" => "pub.leaflet.blocks.image",
        "horizontalRule" => "pub.leaflet.blocks.horizontalRule",
        "orderedList" => "pub.leaflet.blocks.orderedList",
        "unorderedList" => "pub.leaflet.blocks.unorderedList",
        "listItem" => "pub.leaflet.blocks.orderedList#listItem",
        "bskyPost" => "pub.leaflet.blocks.bskyPost",
        "math" => "pub.leaflet.blocks.math",
        _ => return Err(ProtocolError::Emit(
            format!("unknown block kind '{}'", block.kind)
        )),
    };

    block_map.insert("$type".into(), serde_json::json!(block_type));

    for c in constraints {
        match c.sort.as_ref() {
            "level" => {
                if let Ok(v) = c.value.parse::<u64>() {
                    block_map.insert("level".into(), serde_json::json!(v));
                }
            }
            "language" => {
                block_map.insert("language".into(), serde_json::json!(c.value));
            }
            "plaintext" => {
                block_map.insert("plaintext".into(), serde_json::json!(c.value));
            }
            "src" => {
                block_map.insert("src".into(), serde_json::json!(c.value));
            }
            "title" => {
                block_map.insert("title".into(), serde_json::json!(c.value));
            }
            "description" => {
                block_map.insert("description".into(), serde_json::json!(c.value));
            }
            "tex" => {
                block_map.insert("tex".into(), serde_json::json!(c.value));
            }
            "startIndex" => {
                if let Ok(v) = c.value.parse::<u64>() {
                    block_map.insert("startIndex".into(), serde_json::json!(v));
                }
            }
            "facet" => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c.value) {
                    block_map.insert("facets".into(), v);
                }
            }
            _ => {}
        }
    }

    // Emit children for list types.
    if block.kind == "orderedList" || block.kind == "unorderedList" {
        let mut children = Vec::new();
        let mut items = children_by_edge(schema, &block.id, "items");
        items.sort_by(|a, b| vertex_order_key(&a.1.id).cmp(&vertex_order_key(&b.1.id)));
        for (_edge, item) in &items {
            if item.kind != "listItem" {
                continue;
            }
            let mut item_children = children_by_edge(schema, &item.id, "items");
            item_children.sort_by(|a, b| vertex_order_key(&a.1.id).cmp(&vertex_order_key(&b.1.id)));
            if let Some((_edge, nested)) = item_children.first() {
                if let Ok(nested_json) = emit_block(schema, nested) {
                    let suffix = if block.kind == "orderedList" {
                        "orderedList#listItem"
                    } else {
                        "unorderedList#listItem"
                    };
                    let mut item_map = serde_json::Map::new();
                    item_map.insert("$type".into(), serde_json::json!(format!("pub.leaflet.blocks.{suffix}")));
                    item_map.insert("content".into(), nested_json);
                    children.push(serde_json::Value::Object(item_map));
                }
            }
        }
        if !children.is_empty() {
            block_map.insert("children".into(), serde_json::Value::Array(children));
        }
    }

    let mut wrapper = serde_json::Map::new();
    wrapper.insert("$type".into(), serde_json::json!("pub.leaflet.pages.linearDocument#block"));
    wrapper.insert("block".into(), serde_json::Value::Object(block_map));

    Ok(serde_json::Value::Object(wrapper))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn edge_rules() -> Vec<EdgeRule> {
    vec![
        EdgeRule {
            edge_kind: "items".into(),
            src_kinds: vec![
                "document".into(),
                "page".into(),
                "orderedList".into(),
                "unorderedList".into(),
                "listItem".into(),
            ],
            tgt_kinds: vec![],
        },
        EdgeRule {
            edge_kind: "prop".into(),
            src_kinds: vec![
                "document".into(),
                "page".into(),
                "header".into(),
                "text".into(),
                "blockquote".into(),
                "code".into(),
                "website".into(),
                "image".into(),
                "listItem".into(),
            ],
            tgt_kinds: vec![],
        },
    ]
}

/// Extract a numeric ordering key from a vertex ID.
///
/// Vertex IDs are generated sequentially by the parsers (e.g.
/// `page:0:block:0003`, `page:0:list:0004`).  Sorting by the last
/// colon-separated numeric component restores source insertion order
/// regardless of the type prefix (`block:`, `list:`, `item:`), which
/// is necessary because upstream schema transforms may rebuild the
/// edge HashMap and randomise iteration order.
fn vertex_order_key(id: &str) -> u32 {
    id.rsplit(':').next().and_then(|s| s.parse().ok()).unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn protocol_def() {
        let p = protocol();
        assert_eq!(p.name, "leaflet");
    }

    #[test]
    fn register_theories_works() {
        let mut registry = HashMap::new();
        register_theories(&mut registry);
        assert!(registry.contains_key("ThLeafletSchema"));
        assert!(registry.contains_key("ThLeafletInstance"));
    }

    #[test]
    fn parse_emit_roundtrip() {
        let json = serde_json::json!({
            "$type": "site.standard.document",
            "title": "Test",
            "description": "A doc",
            "content": {
                "$type": "pub.leaflet.content",
                "pages": [
                    {
                        "$type": "pub.leaflet.pages.linearDocument",
                        "id": "page-1",
                        "blocks": [
                            {
                                "$type": "pub.leaflet.pages.linearDocument#block",
                                "block": {
                                    "$type": "pub.leaflet.blocks.header",
                                    "level": 1,
                                    "plaintext": "Hello"
                                }
                            },
                            {
                                "$type": "pub.leaflet.pages.linearDocument#block",
                                "block": {
                                    "$type": "pub.leaflet.blocks.text",
                                    "plaintext": "World"
                                }
                            },
                            {
                                "$type": "pub.leaflet.pages.linearDocument#block",
                                "block": {
                                    "$type": "pub.leaflet.blocks.math",
                                    "tex": "E=mc^2"
                                }
                            },
                            {
                                "$type": "pub.leaflet.pages.linearDocument#block",
                                "block": {
                                    "$type": "pub.leaflet.blocks.image",
                                    "src": "pic.png"
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let schema = parse_leaflet_document(&json).expect("parse");
        assert!(schema.has_vertex("document"));

        let emitted = emit_leaflet_document(&schema).expect("emit");
        assert_eq!(emitted.get("title").and_then(|v| v.as_str()), Some("Test"));
        assert_eq!(emitted.get("description").and_then(|v| v.as_str()), Some("A doc"));

        let pages = emitted
            .get("content")
            .and_then(|c| c.get("pages"))
            .and_then(|p| p.as_array())
            .expect("pages");
        assert_eq!(pages.len(), 1);

        let blocks = pages[0]
            .get("blocks")
            .and_then(|b| b.as_array())
            .expect("blocks");
        assert_eq!(blocks.len(), 4);

        let block_types: Vec<_> = blocks
            .iter()
            .map(|b| {
                b.get("block")
                    .and_then(|blk| blk.get("$type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
            })
            .collect();
        assert!(block_types.contains(&"pub.leaflet.blocks.header"));
        assert!(block_types.contains(&"pub.leaflet.blocks.text"));
        assert!(block_types.contains(&"pub.leaflet.blocks.math"));
        assert!(block_types.contains(&"pub.leaflet.blocks.image"));
    }
}
