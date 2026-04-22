//! LaTeX math → Unicode converter.
//!
//! Converts simple inline LaTeX math expressions to Unicode equivalents.
//! Designed for inline math (`$...$`) where the result is embedded in
//! plaintext paragraphs, headings, list items, and blockquotes.
//!
//! Display math (`$$...$$`) is **not** passed through this converter — it
//! remains as raw `tex` inside `pub.leaflet.blocks.math` KaTeX blocks.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Convert a LaTeX math expression to Unicode.
///
/// Handles Greek letters, operators, arrows, superscripts, subscripts,
/// fractions, square roots, blackboard-bold, accents, and delimiters.
/// Unknown commands are passed through unchanged. Mismatched braces cause
/// the original expression to be returned unmodified.
pub fn latex_to_unicode(tex: &str) -> String {
    let mut parser = Parser::new(tex);
    match parser.parse_expr_inner() {
        Ok(result) => result,
        Err(_) => tex.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

#[derive(Debug, Clone)]
struct ParseError;

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn parse_expr_inner(&mut self) -> Result<String, ParseError> {
        let mut out = String::new();
        while self.pos < self.input.len() {
            let ch = match self.peek() {
                Some(c) => c,
                None => break,
            };
            match ch {
                '\\' => {
                    self.next();
                    out.push_str(&self.parse_command()?);
                }
                '^' => {
                    self.next();
                    out.push_str(&self.parse_superscript()?);
                }
                '_' => {
                    self.next();
                    out.push_str(&self.parse_subscript()?);
                }
                '{' => {
                    self.next();
                    out.push_str(&self.parse_group()?);
                }
                '}' => {
                    self.next();
                }
                '[' | ']' => {
                    self.next();
                    out.push(ch);
                }
                _ => {
                    self.next();
                    out.push(ch);
                }
            }
        }
        Ok(out)
    }

    fn parse_group(&mut self) -> Result<String, ParseError> {
        let mut out = String::new();
        while self.pos < self.input.len() {
            let ch = match self.peek() {
                Some(c) => c,
                None => return Err(ParseError),
            };
            if ch == '}' {
                self.next();
                return Ok(out);
            }
            match ch {
                '\\' => {
                    self.next();
                    out.push_str(&self.parse_command()?);
                }
                '^' => {
                    self.next();
                    out.push_str(&self.parse_superscript()?);
                }
                '_' => {
                    self.next();
                    out.push_str(&self.parse_subscript()?);
                }
                '{' => {
                    self.next();
                    out.push_str(&self.parse_group()?);
                }
                _ => {
                    self.next();
                    out.push(ch);
                }
            }
        }
        Err(ParseError)
    }

    fn parse_command(&mut self) -> Result<String, ParseError> {
        let name = if let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphabetic() {
                        s.push(c);
                        self.next();
                    } else {
                        break;
                    }
                }
                s
            } else {
                let mut s = String::new();
                s.push(ch);
                self.next();
                s
            }
        } else {
            return Ok("\\".to_string());
        };

        if let Some(replacement) = COMMAND_MAP.get(name.as_str()) {
            return Ok(replacement.to_string());
        }

        match name.as_str() {
            "frac" => self.parse_frac(),
            "sqrt" => self.parse_sqrt(),
            "mathbb" => self.parse_mathbb(),
            "mathfrak" => self.parse_mathfrak(),
            "hat" => self.parse_accent("\u{0302}"),
            "bar" => self.parse_accent("\u{0304}"),
            "tilde" => self.parse_accent("\u{0303}"),
            "vec" => self.parse_accent("\u{20D7}"),
            "dot" => self.parse_accent("\u{0307}"),
            "ddot" => self.parse_accent("\u{0308}"),
            "overline" => self.parse_overline(),
            "underline" => self.parse_underline(),
            _ => {
                let mut result = format!("\\{name}");
                self.skip_ws();
                if self.peek() == Some('{') {
                    self.next();
                    if let Ok(group) = self.parse_group() {
                        result.push('{');
                        result.push_str(&group);
                        result.push('}');
                    }
                }
                Ok(result)
            }
        }
    }

    fn parse_frac(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok("\\frac".to_string());
        }
        self.next();
        let num = self.parse_group()?;
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok(format!("\\frac{{{num}}}"));
        }
        self.next();
        let den = self.parse_group()?;

        if let Some(frac) = unicode_fraction(&num, &den) {
            return Ok(frac.to_string());
        }
        Ok(format!("{num}⁄{den}"))
    }

    fn parse_sqrt(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        let index = if self.peek() == Some('[') {
            self.next();
            let mut idx = String::new();
            while let Some(ch) = self.peek() {
                if ch == ']' {
                    self.next();
                    break;
                }
                if ch == '\\' {
                    self.next();
                    idx.push_str(&self.parse_command()?);
                } else {
                    self.next();
                    idx.push(ch);
                }
            }
            Some(idx)
        } else {
            None
        };

        self.skip_ws();
        if self.peek() != Some('{') {
            if let Some(idx) = index {
                return Ok(format!("√{idx}"));
            }
            return Ok("√".to_string());
        }
        self.next();
        let radicand = self.parse_group()?;

        if let Some(idx) = index {
            let idx_unicode = to_superscripts(&idx);
            Ok(format!("{idx_unicode}√{radicand}"))
        } else {
            Ok(format!("√{radicand}"))
        }
    }

    fn parse_mathbb(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok("\\mathbb".to_string());
        }
        self.next();
        let content = self.parse_group()?;
        let mut out = String::new();
        for ch in content.chars() {
            out.push(mathbb_char(ch));
        }
        Ok(out)
    }

    fn parse_mathfrak(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok("\\mathfrak".to_string());
        }
        self.next();
        let content = self.parse_group()?;
        let mut out = String::new();
        for ch in content.chars() {
            out.push(mathfrak_char(ch));
        }
        Ok(out)
    }

    fn parse_accent(&mut self, combining: &str) -> Result<String, ParseError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok("\\accent".to_string());
        }
        self.next();
        let content = self.parse_group()?;
        let mut out = content.clone();
        out.push_str(combining);
        Ok(out)
    }

    fn parse_overline(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok("\\overline".to_string());
        }
        self.next();
        let content = self.parse_group()?;
        let mut out = content.clone();
        out.push('\u{0304}');
        Ok(out)
    }

    fn parse_underline(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok("\\underline".to_string());
        }
        self.next();
        let content = self.parse_group()?;
        let mut out = content.clone();
        out.push('\u{0332}');
        Ok(out)
    }

    fn parse_superscript(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        let content = if self.peek() == Some('{') {
            self.next();
            self.parse_group()?
        } else if let Some(ch) = self.peek() {
            self.next();
            ch.to_string()
        } else {
            return Ok("^".to_string());
        };
        Ok(to_superscripts(&content))
    }

    fn parse_subscript(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        let content = if self.peek() == Some('{') {
            self.next();
            self.parse_group()?
        } else if let Some(ch) = self.peek() {
            self.next();
            ch.to_string()
        } else {
            return Ok("_".to_string());
        };
        Ok(to_subscripts(&content))
    }
}

// ---------------------------------------------------------------------------
// Unicode lookup helpers
// ---------------------------------------------------------------------------

fn to_superscripts(s: &str) -> String {
    s.chars()
        .map(|c| SUPERSCRIPT_MAP.get(&c).copied().unwrap_or(c))
        .collect()
}

fn to_subscripts(s: &str) -> String {
    s.chars()
        .map(|c| SUBSCRIPT_MAP.get(&c).copied().unwrap_or(c))
        .collect()
}

fn mathbb_char(c: char) -> char {
    match c {
        'A' => '𝔸', 'B' => '𝔹', 'C' => 'ℂ', 'D' => '𝔻', 'E' => '𝔼',
        'F' => '𝔽', 'G' => '𝔾', 'H' => 'ℍ', 'I' => '𝕀', 'J' => '𝕁',
        'K' => '𝕂', 'L' => '𝕃', 'M' => '𝕄', 'N' => 'ℕ', 'O' => '𝕆',
        'P' => 'ℙ', 'Q' => 'ℚ', 'R' => 'ℝ', 'S' => '𝕊', 'T' => '𝕋',
        'U' => '𝕌', 'V' => '𝕍', 'W' => '𝕎', 'X' => '𝕏', 'Y' => '𝕐',
        'Z' => 'ℤ',
        'a' => '𝕒', 'b' => '𝕓', 'c' => '𝕔', 'd' => '𝕕', 'e' => '𝕖',
        'f' => '𝕗', 'g' => '𝕘', 'h' => '𝕙', 'i' => '𝕚', 'j' => '𝕛',
        'k' => '𝕜', 'l' => '𝕝', 'm' => '𝕞', 'n' => '𝕟', 'o' => '𝕠',
        'p' => '𝕡', 'q' => '𝕢', 'r' => '𝕣', 's' => '𝕤', 't' => '𝕥',
        'u' => '𝕦', 'v' => '𝕧', 'w' => '𝕨', 'x' => '𝕩', 'y' => '𝕪',
        'z' => '𝕫',
        '0' => '𝟘', '1' => '𝟙', '2' => '𝟚', '3' => '𝟛', '4' => '𝟜',
        '5' => '𝟝', '6' => '𝟞', '7' => '𝟟', '8' => '𝟠', '9' => '𝟡',
        _ => c,
    }
}

fn mathfrak_char(c: char) -> char {
    match c {
        'A' => '𝔄', 'B' => '𝔅', 'C' => 'ℭ', 'D' => '𝔇', 'E' => '𝔈',
        'F' => '𝔉', 'G' => '𝔊', 'H' => 'ℌ', 'I' => 'ℑ', 'J' => '𝔍',
        'K' => '𝔎', 'L' => '𝔏', 'M' => '𝔐', 'N' => '𝔑', 'O' => '𝔒',
        'P' => '𝔓', 'Q' => '𝔔', 'R' => 'ℜ', 'S' => '𝔖', 'T' => '𝔗',
        'U' => '𝔘', 'V' => '𝔙', 'W' => '𝔚', 'X' => '𝔛', 'Y' => '𝔜',
        'Z' => 'ℨ',
        'a' => '𝔞', 'b' => '𝔟', 'c' => '𝔠', 'd' => '𝔡', 'e' => '𝔢',
        'f' => '𝔣', 'g' => '𝔤', 'h' => '𝔥', 'i' => '𝔦', 'j' => '𝔧',
        'k' => '𝔨', 'l' => '𝔩', 'm' => '𝔪', 'n' => '𝔫', 'o' => '𝔬',
        'p' => '𝔭', 'q' => '𝔮', 'r' => '𝔯', 's' => '𝔰', 't' => '𝔱',
        'u' => '𝔲', 'v' => '𝔳', 'w' => '𝔴', 'x' => '𝔵', 'y' => '𝔶',
        'z' => '𝔷',
        _ => c,
    }
}

fn unicode_fraction(num: &str, den: &str) -> Option<char> {
    match (num, den) {
        ("1", "2") => Some('½'),
        ("1", "3") => Some('⅓'),
        ("2", "3") => Some('⅔'),
        ("1", "4") => Some('¼'),
        ("3", "4") => Some('¾'),
        ("1", "5") => Some('⅕'),
        ("2", "5") => Some('⅖'),
        ("3", "5") => Some('⅗'),
        ("4", "5") => Some('⅘'),
        ("1", "6") => Some('⅙'),
        ("5", "6") => Some('⅚'),
        ("1", "8") => Some('⅛'),
        ("3", "8") => Some('⅜'),
        ("5", "8") => Some('⅝'),
        ("7", "8") => Some('⅞'),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Static maps
// ---------------------------------------------------------------------------

static COMMAND_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Greek lowercase
    m.insert("alpha", "α");
    m.insert("beta", "β");
    m.insert("gamma", "γ");
    m.insert("delta", "δ");
    m.insert("epsilon", "ε");
    m.insert("varepsilon", "ε");
    m.insert("zeta", "ζ");
    m.insert("eta", "η");
    m.insert("theta", "θ");
    m.insert("vartheta", "ϑ");
    m.insert("iota", "ι");
    m.insert("kappa", "κ");
    m.insert("varkappa", "ϰ");
    m.insert("lambda", "λ");
    m.insert("mu", "μ");
    m.insert("nu", "ν");
    m.insert("xi", "ξ");
    m.insert("omicron", "ο");
    m.insert("pi", "π");
    m.insert("rho", "ρ");
    m.insert("sigma", "σ");
    m.insert("varsigma", "ς");
    m.insert("tau", "τ");
    m.insert("upsilon", "υ");
    m.insert("phi", "ϕ");
    m.insert("varphi", "φ");
    m.insert("chi", "χ");
    m.insert("psi", "ψ");
    m.insert("omega", "ω");
    // Greek uppercase
    m.insert("Alpha", "Α");
    m.insert("Beta", "Β");
    m.insert("Gamma", "Γ");
    m.insert("Delta", "Δ");
    m.insert("Epsilon", "Ε");
    m.insert("Zeta", "Ζ");
    m.insert("Eta", "Η");
    m.insert("Theta", "Θ");
    m.insert("Iota", "Ι");
    m.insert("Kappa", "Κ");
    m.insert("Lambda", "Λ");
    m.insert("Mu", "Μ");
    m.insert("Nu", "Ν");
    m.insert("Xi", "Ξ");
    m.insert("Omicron", "Ο");
    m.insert("Pi", "Π");
    m.insert("Rho", "Ρ");
    m.insert("Sigma", "Σ");
    m.insert("Tau", "Τ");
    m.insert("Upsilon", "Υ");
    m.insert("Phi", "Φ");
    m.insert("Chi", "Χ");
    m.insert("Psi", "Ψ");
    m.insert("Omega", "Ω");
    // Special letters / symbols
    m.insert("aleph", "ℵ");
    m.insert("hbar", "ℏ");
    m.insert("ell", "ℓ");
    m.insert("Re", "ℜ");
    m.insert("Im", "ℑ");
    m.insert("wp", "℘");
    m.insert("mho", "℧");
    m.insert("nabla", "∇");
    m.insert("partial", "∂");
    m.insert("infty", "∞");
    m.insert("emptyset", "∅");
    m.insert("varnothing", "∅");
    // Operators
    m.insert("times", "×");
    m.insert("div", "÷");
    m.insert("pm", "±");
    m.insert("mp", "∓");
    m.insert("oplus", "⊕");
    m.insert("ominus", "⊖");
    m.insert("otimes", "⊗");
    m.insert("oslash", "⊘");
    m.insert("odot", "⊙");
    m.insert("circ", "∘");
    m.insert("bullet", "•");
    m.insert("star", "⋆");
    m.insert("dagger", "†");
    m.insert("ddagger", "‡");
    m.insert("amalg", "⨿");
    m.insert("setminus", "⧵");
    m.insert("wr", "≀");
    m.insert("cdotp", "·");
    m.insert("cdot", "·");
    m.insert("cdots", "⋯");
    m.insert("ldots", "…");
    m.insert("vdots", "⋮");
    m.insert("ddots", "⋱");
    // Relations
    m.insert("leq", "≤");
    m.insert("le", "≤");
    m.insert("geq", "≥");
    m.insert("ge", "≥");
    m.insert("neq", "≠");
    m.insert("ne", "≠");
    m.insert("approx", "≈");
    m.insert("equiv", "≡");
    m.insert("sim", "∼");
    m.insert("simeq", "≃");
    m.insert("cong", "≅");
    m.insert("subset", "⊂");
    m.insert("supset", "⊃");
    m.insert("subseteq", "⊆");
    m.insert("supseteq", "⊇");
    m.insert("in", "∈");
    m.insert("notin", "∉");
    m.insert("ni", "∋");
    m.insert("forall", "∀");
    m.insert("exists", "∃");
    m.insert("nexists", "∄");
    m.insert("land", "∧");
    m.insert("lor", "∨");
    m.insert("neg", "¬");
    m.insert("lnot", "¬");
    m.insert("top", "⊤");
    m.insert("bot", "⊥");
    m.insert("vdash", "⊢");
    m.insert("models", "⊨");
    m.insert("perp", "⊥");
    m.insert("parallel", "∥");
    m.insert("nparallel", "∦");
    // Arrows
    m.insert("to", "→");
    m.insert("rightarrow", "→");
    m.insert("leftarrow", "←");
    m.insert("Rightarrow", "⇒");
    m.insert("Leftarrow", "⇐");
    m.insert("leftrightarrow", "↔");
    m.insert("Leftrightarrow", "⇔");
    m.insert("mapsto", "↦");
    m.insert("longrightarrow", "⟶");
    m.insert("longleftarrow", "⟵");
    m.insert("Longrightarrow", "⟹");
    m.insert("Longleftarrow", "⟸");
    m.insert("hookrightarrow", "↪");
    m.insert("hookleftarrow", "↩");
    m.insert("rightharpoonup", "⇀");
    m.insert("rightharpoondown", "⇁");
    m.insert("uparrow", "↑");
    m.insert("downarrow", "↓");
    m.insert("Uparrow", "⇑");
    m.insert("Downarrow", "⇓");
    // Delimiters
    m.insert("langle", "⟨");
    m.insert("rangle", "⟩");
    m.insert("mid", "∣");
    m.insert("nmid", "∤");
    m.insert("backslash", "\\");
    // Integrals / sums / products
    m.insert("int", "∫");
    m.insert("iint", "∬");
    m.insert("iiint", "∭");
    m.insert("oint", "∮");
    m.insert("sum", "∑");
    m.insert("prod", "∏");
    m.insert("coprod", "∐");
    // Misc
    m.insert("angle", "∠");
    m.insert("triangle", "△");
    m.insert("square", "□");
    m.insert("diamond", "◇");
    m.insert("clubsuit", "♣");
    m.insert("diamondsuit", "♢");
    m.insert("heartsuit", "♡");
    m.insert("spadesuit", "♠");
    m.insert("S", "§");
    m.insert("P", "¶");
    m
});

static SUPERSCRIPT_MAP: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert('0', '⁰');
    m.insert('1', '¹');
    m.insert('2', '²');
    m.insert('3', '³');
    m.insert('4', '⁴');
    m.insert('5', '⁵');
    m.insert('6', '⁶');
    m.insert('7', '⁷');
    m.insert('8', '⁸');
    m.insert('9', '⁹');
    m.insert('a', 'ᵃ');
    m.insert('b', 'ᵇ');
    m.insert('c', 'ᶜ');
    m.insert('d', 'ᵈ');
    m.insert('e', 'ᵉ');
    m.insert('f', 'ᶠ');
    m.insert('g', 'ᵍ');
    m.insert('h', 'ʰ');
    m.insert('i', 'ⁱ');
    m.insert('j', 'ʲ');
    m.insert('k', 'ᵏ');
    m.insert('l', 'ˡ');
    m.insert('m', 'ᵐ');
    m.insert('n', 'ⁿ');
    m.insert('o', 'ᵒ');
    m.insert('p', 'ᵖ');
    m.insert('r', 'ʳ');
    m.insert('s', 'ˢ');
    m.insert('t', 'ᵗ');
    m.insert('u', 'ᵘ');
    m.insert('v', 'ᵛ');
    m.insert('w', 'ʷ');
    m.insert('x', 'ˣ');
    m.insert('y', 'ʸ');
    m.insert('z', 'ᶻ');
    m.insert('A', 'ᴬ');
    m.insert('B', 'ᴮ');
    m.insert('F', 'ᶠ');
    m.insert('S', 'ˢ');
    m.insert('Z', 'ᶻ');
    m.insert('+', '⁺');
    m.insert('-', '⁻');
    m.insert('=', '⁼');
    m.insert('(', '⁽');
    m.insert(')', '⁾');
    m
});

static SUBSCRIPT_MAP: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert('0', '₀');
    m.insert('1', '₁');
    m.insert('2', '₂');
    m.insert('3', '₃');
    m.insert('4', '₄');
    m.insert('5', '₅');
    m.insert('6', '₆');
    m.insert('7', '₇');
    m.insert('8', '₈');
    m.insert('9', '₉');
    m.insert('a', 'ₐ');
    m.insert('e', 'ₑ');
    m.insert('h', 'ₕ');
    m.insert('i', 'ᵢ');
    m.insert('j', 'ⱼ');
    m.insert('k', 'ₖ');
    m.insert('l', 'ₗ');
    m.insert('m', 'ₘ');
    m.insert('n', 'ₙ');
    m.insert('o', 'ₒ');
    m.insert('p', 'ₚ');
    m.insert('r', 'ᵣ');
    m.insert('s', 'ₛ');
    m.insert('t', 'ₜ');
    m.insert('u', 'ᵤ');
    m.insert('v', 'ᵥ');
    m.insert('x', 'ₓ');
    m.insert('+', '₊');
    m.insert('-', '₋');
    m.insert('=', '₌');
    m.insert('(', '₍');
    m.insert(')', '₎');
    m
});
