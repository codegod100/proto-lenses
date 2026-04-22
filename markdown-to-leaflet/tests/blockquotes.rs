#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn simple_blockquote() {
    let json = convert("> Quote text.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    assert_eq!(plaintext(&bs[0]), "Quote text.");
}

#[test]
fn blockquote_with_multiple_paragraphs() {
    let json = convert("> First para.\n>\n> Second para.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    // Multiple paragraphs in a blockquote may be coalesced into one text block.
    let text = plaintext(&bs[0]);
    assert!(text.contains("First para"), "missing first para: {text}");
    assert!(text.contains("Second para"), "missing second para: {text}");
}

#[test]
fn blockquote_followed_by_text() {
    let json = convert("> Quote\n\nAfter.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 2);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    assert_eq!(block_type(&bs[1]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[1]), "After.");
}

#[test]
fn blockquote_with_inline_math() {
    let json = convert(r#"> Quote $n^r$ here"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    assert_eq!(plaintext(&bs[0]), "Quote nʳ here");
}

#[test]
fn blockquote_with_greek_inline_math() {
    let json = convert(r#"> Theorem: $\alpha + \beta = \gamma$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.blockquote");
    assert_eq!(plaintext(&bs[0]), "Theorem: α + β = γ");
}
