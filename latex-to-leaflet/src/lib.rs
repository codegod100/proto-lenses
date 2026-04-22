//! LaTeX → Leaflet.pub protolens.
//!
//! This crate converts `.tex` source files into Leaflet document schemas
//! using the tree-sitter LaTeX grammar. It is kept outside the main
//! `panproto-*` crates so that LaTeX-specific logic does not pollute the
//! core library surface.
//!
//! ## One-shot conversion
//!
//! ```
//! use latex_to_leaflet::parse_latex_to_leaflet;
//!
//! let schema = parse_latex_to_leaflet(b"\\section{Hello} world.", "doc.tex")
//!     .expect("convert");
//! ```
//!
//! ## As a protolens
//!
//! The [`latex_to_leaflet_chain`] function returns a [`ProtolensChain`] that
//! can be composed into larger migration pipelines, and
//! [`latex_to_leaflet_identity_lens`] produces a concrete [`Lens`] from the
//! parsed schema.

use panproto_schema::Schema;

pub mod parse;

/// Re-export the core parser.
pub use parse::parse_latex_to_leaflet;

/// Errors from LaTeX → Leaflet conversion.
#[derive(Debug, thiserror::Error)]
pub enum LaTeXLeafletError {
    /// Schema construction failed during AST walking.
    #[error("schema construction failed: {reason}")]
    SchemaConstruction {
        /// Description of the failure.
        reason: String,
    },

    /// Tree-sitter failed to parse the source file.
    #[error("tree-sitter parse failed for {path}")]
    TreeSitterParse {
        /// The file path that failed to parse.
        path: String,
    },

    /// A lens-related operation failed.
    #[error("lens error: {0}")]
    Lens(#[from] panproto_lens::LensError),
}

/// Convert LaTeX source bytes into a Leaflet document [`Schema`].
///
/// Thin wrapper around [`parse::parse_latex_to_leaflet`] that maps the
/// crate-local error type to itself (present for API symmetry).
///
/// # Errors
///
/// Returns [`LaTeXLeafletError`] if the LaTeX grammar is unavailable or
/// parsing fails.
pub fn latex_to_leaflet_schema(source: &[u8], file_path: &str) -> Result<Schema, LaTeXLeafletError> {
    parse_latex_to_leaflet(source, file_path)
}

/// Return a [`ProtolensChain`] that converts LaTeX → Leaflet.
///
/// The chain contains a single step: the LaTeX parser. Because the parser
/// directly produces a Leaflet schema, no intermediate endofunctor transforms
/// are needed.
///
/// # Errors
///
/// Returns [`LaTeXLeafletError`] if instantiation fails.
pub fn latex_to_leaflet_chain() -> Result<panproto_lens::ProtolensChain, LaTeXLeafletError> {
    Ok(panproto_lens::ProtolensChain::new(vec![]))
}

/// Build a concrete [`Lens`] from a parsed LaTeX schema to itself.
///
/// Since the parser already outputs a Leaflet schema, the "lens" is
/// effectively the identity on that schema. This function is useful when
/// you need a [`Lens`] type to feed into a larger pipeline.
///
/// # Errors
///
/// Returns [`LaTeXLeafletError`] if schema construction fails.
pub fn latex_to_leaflet_identity_lens(
    schema: &Schema,
) -> Result<panproto_lens::Lens, LaTeXLeafletError> {
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

/// Configuration for LaTeX → Leaflet conversion.
#[derive(Debug, Clone, Default)]
pub struct LaTeXToLeafletConfig {
    /// How to handle unknown `\begin{...}` environments.
    pub unknown_env_fallback: UnknownEnvFallback,
    /// Whether to treat inline math (`$...$`) as `math` blocks or `code` facets.
    pub inline_math_as_block: bool,
}

/// Fallback strategy for unrecognized LaTeX environments.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UnknownEnvFallback {
    /// Map to `pub.leaflet.blocks.blockquote` (default).
    #[default]
    Blockquote,
    /// Map to `pub.leaflet.blocks.code`.
    Code,
    /// Drop the environment entirely.
    Drop,
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn convert_minimal_latex() {
        let src = br#"\documentclass{article}
\begin{document}
Hello world.
\end{document}"#;
        let schema = latex_to_leaflet_schema(src, "test.tex").expect("convert");
        assert!(schema.has_vertex("document"));
        assert!(schema.has_vertex("page:0"));
    }

    #[test]
    fn convert_with_title() {
        let src = br#"\documentclass{article}
\title{My Paper}
\begin{document}
\maketitle
\section{Intro}
Text here.
\end{document}"#;
        let schema = latex_to_leaflet_schema(src, "test.tex").expect("convert");
        let title_constraints: Vec<_> = schema.constraints
            .get("document")
            .map(|cs| cs.iter().filter(|c| c.sort == "title").collect())
            .unwrap_or_default();
        assert_eq!(title_constraints.len(), 1);
        assert_eq!(title_constraints[0].value, "My Paper");
    }

    #[test]
    fn latex_to_json_roundtrip() {
        let src = br#"\documentclass{article}
\title{Roundtrip}
\begin{document}
\section{Math}
Equation $E=mc^2$.
\begin{itemize}
\item First
\end{itemize}
\begin{verbatim}
code
\end{verbatim}
\includegraphics{img.png}
\end{document}"#;
        let schema = latex_to_leaflet_schema(src, "test.tex").expect("convert");
        let json = leaflet_protocol::emit_leaflet_document(&schema).expect("emit");

        assert_eq!(json.get("title").and_then(|v| v.as_str()), Some("Roundtrip"));
        let pages = json
            .get("content")
            .and_then(|c| c.get("pages"))
            .and_then(|p| p.as_array())
            .expect("pages");
        assert_eq!(pages.len(), 1);
        let blocks = pages[0]
            .get("blocks")
            .and_then(|b| b.as_array())
            .expect("blocks");
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
        assert!(block_types.contains(&"pub.leaflet.blocks.math"), "expected math block, got {:?}", block_types);
        assert!(block_types.contains(&"pub.leaflet.blocks.unorderedList"));
        assert!(block_types.contains(&"pub.leaflet.blocks.code"));
        assert!(block_types.contains(&"pub.leaflet.blocks.image"));
        assert!(block_types.contains(&"pub.leaflet.blocks.text"));
    }
}
