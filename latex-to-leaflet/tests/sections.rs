mod common;
use common::*;

#[test]
fn section_level_2() {
    let json = convert(r#"\begin{document}\section{Intro}\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(bs[0]["block"]["level"], 2);
    assert_eq!(plaintext(&bs[0]), "Intro");
}

#[test]
fn subsection_level_3() {
    let json = convert(r#"\begin{document}\subsection{Sub}\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(bs[0]["block"]["level"], 3);
}

#[test]
fn subsubsection_level_4() {
    let json = convert(r#"\begin{document}\subsubsection{SubSub}\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(bs[0]["block"]["level"], 4);
}

#[test]
fn paragraph_level_5() {
    let json = convert(r#"\begin{document}\paragraph{Para}\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(bs[0]["block"]["level"], 5);
}

#[test]
fn starred_section() {
    let json = convert(r#"\begin{document}\section*{Bib}\text{cite me}\end{document}"#);
    let bs = blocks(&json);
    let headers: Vec<_> = bs.iter().filter(|b| block_type(b) == "pub.leaflet.blocks.header").collect();
    assert_eq!(headers.len(), 1);
    assert_eq!(plaintext(headers[0]), "Bib");
}
