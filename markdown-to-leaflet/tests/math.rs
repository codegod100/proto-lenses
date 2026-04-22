#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn display_math_double_dollar() {
    let json = convert(r#"$$\int_0^1 x dx$$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    assert_eq!(tex(&bs[0]), r"\int_0^1 x dx");
}

#[test]
fn inline_math_converted_to_unicode() {
    let json = convert(r#"$E=mc^2$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "E=mc²");
}

#[test]
fn inline_greek_converted_to_unicode() {
    let json = convert(r#"$\alpha + \beta$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "α + β");
}

#[test]
fn inline_math_between_text_paragraphs() {
    let json = convert(r"Before.

$\alpha + \beta$.

After.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 3);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "Before.");
    assert_eq!(block_type(&bs[1]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[1]), "α + β.");
    assert_eq!(block_type(&bs[2]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[2]), "After.");
}

#[test]
fn inline_math_multiple_in_same_paragraph() {
    let json = convert("$a$ and then $b$");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "a and then b");
}

#[test]
fn inline_math_with_superscripts() {
    let json = convert("$x^2 + y^2 = z^2$");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "x² + y² = z²");
}

#[test]
fn inline_math_with_subscripts() {
    let json = convert("$x_i + y_j$");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "xᵢ + yⱼ");
}

#[test]
fn inline_and_display_math_coexist() {
    let json = convert("Text $x^2$ more text.\n\n$$E=mc^2$$\n\nAfter.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 3);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "Text x² more text.");
    assert_eq!(block_type(&bs[1]), "pub.leaflet.blocks.math");
    assert_eq!(tex(&bs[1]), "E=mc^2");
    assert_eq!(block_type(&bs[2]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[2]), "After.");
}
