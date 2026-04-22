//! Markdown → Leaflet.pub protolens.
//!
//! This crate converts CommonMark / GitHub Flavored Markdown into Leaflet
//! document schemas, mirroring the architecture of `latex-to-leaflet`.

pub mod parse;
pub mod unicode;

pub use parse::parse_markdown_to_leaflet;
pub use unicode::latex_to_unicode;

use panproto_schema::Schema;

/// Errors from Markdown → Leaflet conversion.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownLeafletError {
    /// Schema construction failed.
    #[error("schema construction failed: {reason}")]
    SchemaConstruction {
        /// Human-readable failure reason.
        reason: String,
    },

    /// Markdown parsing failed.
    #[error("markdown parse failed for {path}")]
    MarkdownParse {
        /// Path or identifier for the source file.
        path: String,
    },

    /// A lens-related operation failed.
    #[error("lens error: {0}")]
    Lens(#[from] panproto_lens::LensError),
}

/// Convert Markdown source bytes into a Leaflet document [`Schema`].
pub fn markdown_to_leaflet_schema(
    source: &[u8],
    file_path: &str,
) -> Result<Schema, MarkdownLeafletError> {
    parse_markdown_to_leaflet(source, file_path)
}

/// Return a [`ProtolensChain`] that converts Markdown → Leaflet.
pub fn markdown_to_leaflet_chain() -> Result<panproto_lens::ProtolensChain, MarkdownLeafletError> {
    Ok(panproto_lens::ProtolensChain::new(vec![]))
}

/// Build an identity [`Lens`] from a parsed Markdown schema.
pub fn markdown_to_leaflet_identity_lens(
    schema: &Schema,
) -> Result<panproto_lens::Lens, MarkdownLeafletError> {
    let surviving_verts = schema.vertices.keys().cloned().collect();
    let surviving_edges = schema.edges.keys().cloned().collect();

    let compiled = panproto_inst::CompiledMigration {
        surviving_verts,
        surviving_edges,
        vertex_remap: std::collections::HashMap::new(),
        edge_remap: std::collections::HashMap::new(),
        resolver: std::collections::HashMap::new(),
        hyper_resolver: std::collections::HashMap::new(),
        field_transforms: std::collections::HashMap::new(),
        conditional_survival: std::collections::HashMap::new(),
        expansion_path: std::collections::HashMap::new(),
    };

    Ok(panproto_lens::Lens {
        compiled,
        src_schema: schema.clone(),
        tgt_schema: schema.clone(),
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn convert_minimal_markdown() {
        let src = b"Hello world.";
        let schema = markdown_to_leaflet_schema(src, "test.md").expect("convert");
        assert!(schema.has_vertex("document"));
        assert!(schema.has_vertex("page:0"));
    }

    #[test]
    fn markdown_to_json_roundtrip() {
        let src = b"# Heading\n\nText with **bold** and *italic*.\n\n```rust\ncode\n```\n";
        let schema = markdown_to_leaflet_schema(src, "test.md").expect("convert");
        let json = leaflet_protocol::emit_leaflet_document(&schema).expect("emit");
        assert_eq!(json.get("$type").and_then(|v| v.as_str()), Some("site.standard.document"));
    }
}
