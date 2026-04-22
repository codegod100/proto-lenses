mod common;
use common::*;

#[test]
fn quote_environment() {
    let json = convert(r#"
\begin{document}
\begin{quote}
A famous quote.
\end{quote}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    assert_eq!(plaintext(&bs[0]), "A famous quote.");
}

#[test]
fn verbatim_environment() {
    let json = convert(r#"
\begin{document}
\begin{verbatim}
fn main() {}
\end{verbatim}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.code");
    assert_eq!(bs[0]["block"]["language"], "verbatim");
    assert!(plaintext(&bs[0]).contains("fn main()"));
}

#[test]
fn theorem_environment() {
    let json = convert(r#"
\begin{document}
\begin{theorem}[Yoneda]
For any $A$ there is an isomorphism.
\end{theorem}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    let text = plaintext(&bs[0]);
    assert!(
        !text.contains("begin"),
        "environment name leaked: {text}"
    );
    assert!(
        !text.contains("end"),
        "environment name leaked: {text}"
    );
    assert!(text.contains("$A$"), "math missing from theorem: {text}");
}

#[test]
fn definition_environment() {
    let json = convert(r#"
\begin{document}
\begin{definition}
A \textbf{category} is ...
\end{definition}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    assert!(plaintext(&bs[0]).contains("category"));
}

#[test]
fn skipped_tikzcd() {
    let json = convert(r#"
\begin{document}
\begin{tikzcd}
A \arrow{r} & B
\end{tikzcd}
After diagram.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "After diagram.");
}

#[test]
fn skipped_table() {
    let json = convert(r#"
\begin{document}
\begin{table}
\begin{tabular}{ll}
a & b \\
\end{tabular}
\end{table}
After table.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "After table.");
}

#[test]
fn center_is_transparent() {
    let json = convert(r#"
\begin{document}
\begin{center}
Centered text.
\end{center}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "Centered text.");
}
