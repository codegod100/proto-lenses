#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn fenced_code_block_with_lang() {
    let json = convert("```rust\nfn main() {}\n```");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.code");
    assert_eq!(language(&bs[0]), "rust");
    assert_eq!(plaintext(&bs[0]), "fn main() {}");
}

#[test]
fn fenced_code_block_without_lang() {
    let json = convert("```\nsome code\n```");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.code");
    assert_eq!(language(&bs[0]), "plaintext");
}

#[test]
fn indented_code_block() {
    let json = convert("    line 1\n    line 2");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.code");
    assert_eq!(language(&bs[0]), "plaintext");
}
