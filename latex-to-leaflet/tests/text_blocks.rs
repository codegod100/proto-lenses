mod common;
use common::*;

#[test]
fn plain_text_paragraph() {
    let json = convert(r#"\begin{document}Hello world.\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "Hello world.");
}

#[test]
fn multiple_sentences_coalesced() {
    let json = convert(r#"\begin{document}First sentence. Second sentence.\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "First sentence. Second sentence.");
}

#[test]
fn comma_punctuation_not_orphaned() {
    let json = convert(r#"\begin{document}Algebraic geometry, theoretical CS, and logic.\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "Algebraic geometry, theoretical CS, and logic.");
}

#[test]
fn parentheses_not_spaced() {
    let json = convert(r#"\begin{document}(arrows) between them.\end{document}"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(plaintext(&bs[0]), "(arrows) between them.");
}

#[test]
fn no_empty_text_blocks() {
    let json = convert(r#"\begin{document}\section{Hello}\end{document}"#);
    let bs = blocks(&json);
    for b in bs {
        if block_type(b) == "pub.leaflet.blocks.text" {
            assert!(!plaintext(b).trim().is_empty(), "empty text block found");
        }
    }
}

#[test]
fn orphaned_period_after_inline_math_absorbed() {
    let json = convert(r#"\begin{document}$E=mc^2$.\section{Next}\end{document}"#);
    let bs = blocks(&json);
    let types: Vec<&str> = bs.iter().map(|b| block_type(b)).collect();
    // There should be no standalone "text" block containing just "."
    assert!(
        !types.windows(2).any(|w| {
            w[0] == "pub.leaflet.blocks.math"
                && w[1] == "pub.leaflet.blocks.text"
                && plaintext(&bs[1]) == "."
        }),
        "orphaned period after math block"
    );
}
