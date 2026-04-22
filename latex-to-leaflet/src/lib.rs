//! LaTeX → Leaflet.pub protolens.
//!
//! This crate converts `.tex` source files into Leaflet document schemas
//! using the tree-sitter LaTeX grammar and a declarative lens DSL spec.
//!
//! ## One-shot conversion
//!
//! ```
//! use latex_to_leaflet::latex_to_leaflet_schema;
//!
//! let schema = latex_to_leaflet_schema(br#"\section{Hello} world."#, "doc.tex")
//!     .expect("convert");
//! ```

use panproto_schema::Schema;
use std::sync::LazyLock;

pub mod parse;
pub mod protocol;

/// Re-export the core parser.
pub use parse::parse_latex;

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

    /// A lens DSL operation failed.
    #[error("lens DSL error: {0}")]
    LensDsl(#[from] panproto_lens_dsl::LensDslError),

    /// A lens-related operation failed.
    #[error("lens error: {0}")]
    Lens(#[from] panproto_lens::LensError),
}

/// Convert LaTeX source bytes into a Leaflet document [`Schema`].
///
/// # Errors
///
/// Returns [`LaTeXLeafletError`] if the LaTeX grammar is unavailable or
/// parsing fails.
pub fn latex_to_leaflet_schema(source: &[u8], file_path: &str) -> Result<Schema, LaTeXLeafletError> {
    let latex_schema = parse_latex(source, file_path)?;
    let chain = latex_to_leaflet_chain()?;
    let proto = protocol::protocol();
    let lens = chain.instantiate(&latex_schema, &proto)?;
    Ok(lens.tgt_schema)
}

/// Load and compile the LaTeX → Leaflet lens spec.
///
/// The spec is embedded at compile time from `theories/latex_to_leaflet.ncl`.
fn latex_to_leaflet_chain()
    -> Result<&'static panproto_lens::ProtolensChain, LaTeXLeafletError>
{
    static CHAIN: LazyLock<Result<panproto_lens::ProtolensChain, String>> = LazyLock::new(|| {
        let source = include_str!("../theories/latex_to_leaflet.ncl");
        panproto_lens_dsl::eval::eval_nickel(source, &[])
            .and_then(|doc| panproto_lens_dsl::compile(&doc, "document", &|_| None))
            .map(|c| c.chain)
            .map_err(|e| e.to_string())
    });

    match &*CHAIN {
        Ok(chain) => Ok(chain),
        Err(e) => Err(LaTeXLeafletError::SchemaConstruction {
            reason: e.clone(),
        }),
    }
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
