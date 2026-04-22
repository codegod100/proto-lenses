mod common;
use common::*;

#[test]
fn inline_math_dollar() {
    let json = convert(r#"\begin{document}$\mathcal{C}$\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    assert_eq!(tex(&bs[0]), r"\mathcal{C}");
}

#[test]
fn inline_math_parens() {
    let json = convert(r#"\begin{document}\(\alpha \to \beta\)\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    assert_eq!(tex(&bs[0]), r"\alpha \to \beta");
}

#[test]
fn display_math_bracket() {
    let json = convert(r#"\begin{document}\[E = mc^2\]\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    assert_eq!(tex(&bs[0]), "E = mc^2");
}

#[test]
fn display_math_double_dollar() {
    let json = convert(r#"\begin{document}$$\int_0^1 x dx$$\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    assert_eq!(tex(&bs[0]), r"\int_0^1 x dx");
}

#[test]
fn math_inside_list_item() {
    let json = convert(r#"
\begin{document}
\begin{itemize}
\item $a + b = c$.
\end{itemize}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.unorderedList");
    let items = bs[0]["block"]["children"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    let item_blocks = items[0]["content"]["block"].clone();
    // The single item should contain a math block (period absorbed)
    assert_eq!(item_blocks["$type"], "pub.leaflet.blocks.math");
    assert_eq!(item_blocks["tex"], "a + b = c");
}

#[test]
fn math_in_theorem_blockquote() {
    let json = convert(r#"
\begin{document}
\begin{theorem}
For any $A \in \mathcal{C}$.
\end{theorem}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    let text = plaintext(&bs[0]);
    assert!(text.contains("$A \\in \\mathcal{C}$"), "math source not preserved in blockquote: {text}");
}
