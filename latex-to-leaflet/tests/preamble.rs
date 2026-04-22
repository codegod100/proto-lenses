mod common;
use common::*;

#[test]
fn preamble_fully_dropped() {
    let json = convert(r#"
\documentclass[11pt]{article}
\usepackage{amsmath}
\usepackage{hyperref}
\newtheorem{theorem}{Theorem}
\babelfont{rm}{Noto Sans}
\title{The Title}
\author{The Author}
\date{The Date}
\begin{document}
Body here.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "Body here.");
    assert_eq!(title(&json), Some("The Title"));
}

#[test]
fn documentclass_only_before_begin() {
    let json = convert(r#"
\documentclass{article}
\begin{document}
Text.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(plaintext(&bs[0]), "Text.");
}

#[test]
fn no_babelfont_leakage() {
    let json = convert(r#"
\documentclass{article}
\usepackage{babel}
\babelfont{rm}{Noto Sans}
\begin{document}
Clean.
\end{document}
"#);
    let bs = blocks(&json);
    let all_text: String = bs.iter().map(|b| plaintext(b)).collect();
    assert!(
        !all_text.contains("babelfont"),
        "babelfont leaked into output: {all_text}"
    );
    assert!(
        !all_text.contains("Noto"),
        "Noto leaked into output: {all_text}"
    );
}
