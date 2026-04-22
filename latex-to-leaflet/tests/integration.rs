mod common;
use common::*;
use std::fs;

#[test]
fn category_latex_snapshot() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/category.latex"))
        .expect("read category.latex");
    let json = convert(&src);
    insta::assert_json_snapshot!(json);
}

#[test]
fn minimal_latex_snapshot() {
    let json = convert(r#"\documentclass{article}\begin{document}Hello.\end{document}"#);
    insta::assert_json_snapshot!(json);
}

#[test]
fn rich_document_snapshot() {
    let json = convert(r#"
\documentclass{article}
\title{Rich Doc}
\begin{document}
\maketitle
\section{Intro}
Some \textbf{bold} and \textit{italic} text with $x^2$ math.
\begin{itemize}
\item First item
\item Second item
\end{itemize}
\begin{theorem}[Important]
$A = B$.
\end{theorem}
\end{document}
"#);
    insta::assert_json_snapshot!(json);
}
