#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn horizontal_rule() {
    let json = convert("---");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.horizontalRule");
}

#[test]
fn horizontal_rule_between_text() {
    let json = convert("Before\n\n---\n\nAfter");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 3);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(block_type(&bs[1]), "pub.leaflet.blocks.horizontalRule");
    assert_eq!(block_type(&bs[2]), "pub.leaflet.blocks.text");
}
