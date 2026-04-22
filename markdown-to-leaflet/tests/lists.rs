#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn unordered_list_two_items() {
    let json = convert("- First\n- Second");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.unorderedList");
    let items = list_children(&bs[0]);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["$type"], "pub.leaflet.blocks.unorderedList#listItem");
    assert_eq!(items[1]["$type"], "pub.leaflet.blocks.unorderedList#listItem");
}

#[test]
fn ordered_list_three_items() {
    let json = convert("1. One\n2. Two\n3. Three");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.orderedList");
    let items = list_children(&bs[0]);
    assert_eq!(items.len(), 3);
    assert_eq!(bs[0]["block"]["startIndex"], 1);
}

#[test]
fn no_text_bleeding_between_items() {
    let json = convert("- Alpha\n- Beta\n\nText after.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 2);
    let items = list_children(&bs[0]);
    assert_eq!(items.len(), 2);
    let first = items[0]["content"]["block"]["plaintext"]
        .as_str()
        .unwrap_or("");
    let second = items[1]["content"]["block"]["plaintext"]
        .as_str()
        .unwrap_or("");
    assert_eq!(first, "Alpha", "first item wrong");
    assert_eq!(second, "Beta", "second item wrong");
    assert!(!first.contains("Beta"), "text bled from second into first: {first}");
    assert!(!second.contains("Alpha"), "text bled from first into second: {second}");
    assert_eq!(plaintext(&bs[1]), "Text after.");
}

#[test]
fn list_item_with_inline_formatting() {
    let json = convert("- **Bold** item\n- *Italic* item");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    let items = list_children(&bs[0]);
    assert_eq!(items.len(), 2);
    let first = items[0]["content"]["block"]["plaintext"]
        .as_str()
        .unwrap_or("");
    let second = items[1]["content"]["block"]["plaintext"]
        .as_str()
        .unwrap_or("");
    assert_eq!(first, "Bold item");
    assert_eq!(second, "Italic item");
}

#[test]
fn list_item_with_inline_math() {
    let json = convert(r#"- Item $v^i$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.unorderedList");
    let items = list_children(&bs[0]);
    assert_eq!(items.len(), 1);
    let text = items[0]["content"]["block"]["plaintext"]
        .as_str()
        .unwrap_or("");
    assert_eq!(text, "Item vⁱ");
}

#[test]
fn list_item_with_inline_math_subscript() {
    let json = convert(r#"- $n^l$ result"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    let items = list_children(&bs[0]);
    let text = items[0]["content"]["block"]["plaintext"]
        .as_str()
        .unwrap_or("");
    assert_eq!(text, "nˡ result");
}
