mod common;
use common::*;

#[test]
fn title_declaration() {
    let json = convert(r#"
\documentclass{article}
\title{My Paper}
\begin{document}
\maketitle
\end{document}
"#);
    assert_eq!(title(&json), Some("My Paper"));
}

#[test]
fn author_dropped() {
    let json = convert(r#"
\documentclass{article}
\author{John Doe}
\begin{document}
Hello.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(plaintext(&bs[0]), "Hello.");
}

#[test]
fn date_dropped() {
    let json = convert(r#"
\documentclass{article}
\date{2024-01-01}
\begin{document}
Hello.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(plaintext(&bs[0]), "Hello.");
}

#[test]
fn maketitle_dropped() {
    let json = convert(r#"
\documentclass{article}
\title{T}
\begin{document}
\maketitle
Hello.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "Hello.");
}

#[test]
fn includegraphics() {
    let json = convert(r#"
\begin{document}
\includegraphics{img.png}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.image");
    assert_eq!(bs[0]["block"]["src"], "img.png");
}

#[test]
fn textbf_produces_facet() {
    let json = convert(r#"
\begin{document}
\textbf{bold} text.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "bold text.");
    // Facet tests will be enabled once the parser emits facets.
    // assert!(has_facet(&bs[0], "#strong"));
}

#[test]
fn textit_produces_facet() {
    let json = convert(r#"
\begin{document}
\textit{italic} text.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "italic text.");
    // assert!(has_facet(&bs[0], "#em"));
}
