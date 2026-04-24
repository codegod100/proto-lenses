//! Markdown → Leaflet.pub protolens.
//!
//! This crate converts CommonMark / GitHub Flavored Markdown into Leaflet
//! document schemas via a declarative lens DSL spec.

pub mod parse;
pub mod protocol;
pub mod unicode;

pub use parse::parse_markdown;
pub use unicode::latex_to_unicode;

use panproto_schema::Schema;
use std::sync::LazyLock;

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

    /// A lens DSL operation failed.
    #[error("lens DSL error: {0}")]
    LensDsl(#[from] panproto_lens_dsl::LensDslError),

    /// A lens-related operation failed.
    #[error("lens error: {0}")]
    Lens(#[from] panproto_lens::LensError),
}

/// Convert Markdown source bytes into a Leaflet document [`Schema`].
pub fn markdown_to_leaflet_schema(
    source: &[u8],
    file_path: &str,
) -> Result<Schema, MarkdownLeafletError> {
    let md_schema = parse_markdown(source, file_path)?;
    let chain = markdown_to_leaflet_chain()?;
    let proto = protocol::protocol();
    let lens = chain.instantiate(&md_schema, &proto)?;
    Ok(lens.tgt_schema)
}

/// Load and compile the Markdown → Leaflet lens spec.
///
/// The spec is embedded at compile time from `theories/markdown_to_leaflet.json`
/// and deserialized via `serde_json::from_str`. This avoids Nickel evaluation
/// at runtime (which panics in WASM due to `temp_dir` / filesystem usage).
fn markdown_to_leaflet_chain()
    -> Result<&'static panproto_lens::ProtolensChain, MarkdownLeafletError>
{
    static CHAIN: LazyLock<Result<panproto_lens::ProtolensChain, String>> = LazyLock::new(|| {
        let source = include_str!("../theories/markdown_to_leaflet.json");
        let doc: panproto_lens_dsl::LensDocument = serde_json::from_str(source)
            .map_err(|e| format!("JSON parse failed: {e}"))?;
        panproto_lens_dsl::compile(&doc, "document", &|_| None)
            .map(|c| c.chain)
            .map_err(|e| e.to_string())
    });

    match &*CHAIN {
        Ok(chain) => Ok(chain),
        Err(e) => Err(MarkdownLeafletError::SchemaConstruction {
            reason: e.clone(),
        }),
    }
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
