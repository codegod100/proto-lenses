#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn h1_heading() {
    let json = convert("# Heading One");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(plaintext(&bs[0]), "Heading One");
    assert_eq!(level(&bs[0]), 1);
}

#[test]
fn h2_heading() {
    let json = convert("## Heading Two");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(level(&bs[0]), 2);
}

#[test]
fn h6_heading() {
    let json = convert("###### Heading Six");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(level(&bs[0]), 6);
}

#[test]
fn heading_followed_by_text() {
    let json = convert("# Title\n\nBody paragraph.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 2);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(block_type(&bs[1]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[1]), "Body paragraph.");
}

#[test]
fn inline_math_in_heading() {
    let json = convert(r#"# Section $\alpha$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(plaintext(&bs[0]), "Section α");
    assert_eq!(level(&bs[0]), 1);
}

#[test]
fn inline_math_superscript_in_heading() {
    let json = convert(r#"## $n^r$ Results"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.header");
    assert_eq!(plaintext(&bs[0]), "nʳ Results");
    assert_eq!(level(&bs[0]), 2);
}
