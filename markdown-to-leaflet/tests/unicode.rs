//! Unicode converter unit tests.

use markdown_to_leaflet::latex_to_unicode;

#[test]
fn greek_alpha() {
    assert_eq!(latex_to_unicode(r"\alpha"), "α");
}

#[test]
fn greek_beta() {
    assert_eq!(latex_to_unicode(r"\beta"), "β");
}

#[test]
fn greek_uppercase_gamma() {
    assert_eq!(latex_to_unicode(r"\Gamma"), "Γ");
}

#[test]
fn superscript_digit() {
    assert_eq!(latex_to_unicode("x^2"), "x²");
}

#[test]
fn superscript_braced() {
    assert_eq!(latex_to_unicode("x^{2+3}"), "x²⁺³");
}

#[test]
fn subscript_digit() {
    assert_eq!(latex_to_unicode("x_i"), "xᵢ");
}

#[test]
fn subscript_braced() {
    assert_eq!(latex_to_unicode("x_{ij}"), "xᵢⱼ");
}

#[test]
fn superscript_nr() {
    assert_eq!(latex_to_unicode("n^r"), "nʳ");
}

#[test]
fn superscript_nl() {
    assert_eq!(latex_to_unicode("n^l"), "nˡ");
}

#[test]
fn superscript_vi() {
    assert_eq!(latex_to_unicode("v^i"), "vⁱ");
}

#[test]
fn operator_otimes() {
    assert_eq!(latex_to_unicode(r"\otimes"), "⊗");
}

#[test]
fn operator_oplus() {
    assert_eq!(latex_to_unicode(r"\oplus"), "⊕");
}

#[test]
fn arrow_rightarrow() {
    assert_eq!(latex_to_unicode(r"\rightarrow"), "→");
}

#[test]
fn arrow_longrightarrow() {
    assert_eq!(latex_to_unicode(r"\longrightarrow"), "⟶");
}

#[test]
fn relation_leq() {
    assert_eq!(latex_to_unicode(r"\leq"), "≤");
}

#[test]
fn fraction_simple() {
    assert_eq!(latex_to_unicode(r"\frac{1}{2}"), "½");
}

#[test]
fn fraction_fallback() {
    assert_eq!(latex_to_unicode(r"\frac{a}{b}"), "a⁄b");
}

#[test]
fn sqrt_simple() {
    assert_eq!(latex_to_unicode(r"\sqrt{x}"), "√x");
}

#[test]
fn sqrt_indexed() {
    assert_eq!(latex_to_unicode(r"\sqrt[n]{x}"), "ⁿ√x");
}

#[test]
fn mathbb_real() {
    assert_eq!(latex_to_unicode(r"\mathbb{R}"), "ℝ");
}

#[test]
fn mathbb_natural() {
    assert_eq!(latex_to_unicode(r"\mathbb{N}"), "ℕ");
}

#[test]
fn accent_hat() {
    assert_eq!(latex_to_unicode(r"\hat{x}"), "x̂");
}

#[test]
fn accent_bar() {
    assert_eq!(latex_to_unicode(r"\bar{x}"), "x̄");
}

#[test]
fn accent_tilde() {
    assert_eq!(latex_to_unicode(r"\tilde{x}"), "x̃");
}

#[test]
fn accent_vec() {
    assert_eq!(latex_to_unicode(r"\vec{x}"), "x⃗");
}

#[test]
fn overline() {
    assert_eq!(latex_to_unicode(r"\overline{AB}"), "AB̄");
}

#[test]
fn underline() {
    assert_eq!(latex_to_unicode(r"\underline{AB}"), "AB̲");
}

#[test]
fn delimiter_angle() {
    assert_eq!(latex_to_unicode(r"\langle a \rangle"), "⟨ a ⟩");
}

#[test]
fn unknown_command_passes_through() {
    assert_eq!(latex_to_unicode(r"\foo"), r"\foo");
}

#[test]
fn mixed_expression() {
    assert_eq!(
        latex_to_unicode(r"\alpha + \beta = \gamma"),
        "α + β = γ"
    );
}

#[test]
fn special_aleph() {
    assert_eq!(latex_to_unicode(r"\aleph"), "ℵ");
}

#[test]
fn special_nabla() {
    assert_eq!(latex_to_unicode(r"\nabla"), "∇");
}

#[test]
fn fraction_unicode_many() {
    assert_eq!(latex_to_unicode(r"\frac{3}{4}"), "¾");
    assert_eq!(latex_to_unicode(r"\frac{1}{3}"), "⅓");
    assert_eq!(latex_to_unicode(r"\frac{2}{3}"), "⅔");
}
