#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn plain_text_paragraph() {
    let json = convert("Hello world.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "Hello world.");
}

#[test]
fn multiple_sentences_coalesced() {
    let json = convert("First sentence. Second sentence.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(
        plaintext(&bs[0]),
        "First sentence. Second sentence."
    );
}

#[test]
fn inline_bold_not_fragmented() {
    let json = convert("This is **bold** text.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "This is bold text.");
}

#[test]
fn inline_italic_not_fragmented() {
    let json = convert("This is *italic* text.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "This is italic text.");
}

#[test]
fn inline_code_not_fragmented() {
    let json = convert("Use `print()` here.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "Use print() here.");
}

#[test]
fn mixed_inline_formatting() {
    let json = convert("**Bold** and *italic* and ``code``.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(
        plaintext(&bs[0]),
        "Bold and italic and code."
    );
}

#[test]
fn no_empty_text_blocks() {
    let json = convert("# Heading\n\nSome text.");
    let bs = blocks(&json);
    for b in bs {
        if block_type(b) == "pub.leaflet.blocks.text" {
            assert!(!plaintext(b).trim().is_empty(), "empty text block found");
        }
    }
}

#[test]
fn soft_break_preserves_newline() {
    let json = convert("Line one\nLine two.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "Line one\nLine two.");
}

#[test]
fn hard_break_preserves_newline() {
    let json = convert("Line one  \nLine two.");
    let bs = blocks(&json);
    assert_eq!(plaintext(&bs[0]), "Line one\nLine two.");
}

#[test]
fn multiline_text_block() {
    let md = r#"$Server$: $n$ $\leftarrow$ Server is a **Noun**
$Needs$: $n^r \otimes s \otimes n^l$ $\leftarrow$ **Noun** needs a **Noun**
$Database$: $n \leftarrow$ Database is also a **Noun**
$Data$: $n$"#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    let text = plaintext(&bs[0]);
    println!("multiline_text_block: {:?}", text);
    assert!(text.contains("Server"), "missing Server: {}", text);
    assert!(text.contains("Noun"), "missing Noun: {}", text);
    assert!(text.contains("Needs"), "missing Needs: {}", text);
    assert!(text.contains("Database"), "missing Database: {}", text);
    assert!(text.contains("Data"), "missing Data: {}", text);
    // Should contain newlines, not be one long line
    assert!(text.contains('\n'), "expected newlines preserved: {}", text);
}

#[test]
fn parentheses_not_spaced() {
    let json = convert("(arrows) between them.");
    let bs = blocks(&json);
    assert_eq!(plaintext(&bs[0]), "(arrows) between them.");
}

#[test]
fn comma_punctuation_not_orphaned() {
    let json = convert("Algebraic geometry, theoretical CS, and logic.");
    let bs = blocks(&json);
    assert_eq!(plaintext(&bs[0]), "Algebraic geometry, theoretical CS, and logic.");
}

#[test]
fn inline_math_superscript_in_text() {
    let json = convert("$x^2$");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "x²");
}

#[test]
fn inline_math_greek_in_text() {
    let json = convert(r#"$\alpha$ and $\beta$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "α and β");
}

#[test]
fn inline_math_operator_in_text() {
    let json = convert(r#"$a \otimes b$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "a ⊗ b");
}

#[test]
fn inline_math_fraction_in_text() {
    let json = convert(r#"$\frac{1}{2}$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "½");
}

#[test]
fn inline_math_sqrt_in_text() {
    let json = convert(r#"$\sqrt{x}$"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "√x");
}

#[test]
fn inline_math_mixed_with_formatting() {
    let json = convert(r#"This is **bold** and $x^2$ math."#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "This is bold and x² math.");
}

#[test]
fn wikilink_plain_text() {
    let json = convert("See [[Category Theory]] for more.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "See Category Theory for more.");
}

#[test]
fn wikilink_with_display_text() {
    let json = convert("Read [[math#algebra|Algebra]] now.");
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "Read Algebra now.");
}
