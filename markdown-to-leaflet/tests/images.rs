#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn image_block() {
    let json = convert("![Alt text](/path/to/image.png)");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.image");
    assert_eq!(src(&bs[0]), "/path/to/image.png");
}

#[test]
fn image_inline_in_text() {
    let json = convert("See this ![Alt](/img.png) here.");
    let bs = blocks(&json);
    // Image should be a separate block between text fragments.
    assert!(bs.iter().any(|b| block_type(b) == "pub.leaflet.blocks.image"));
    assert_eq!(src(&bs[1]), "/img.png");
}
