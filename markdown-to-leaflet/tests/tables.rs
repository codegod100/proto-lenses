#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn simple_table_to_math_block() {
    let md = r#"| A | B |
|---|---|
| 1 | 2 |"#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    let tex_str = tex(&bs[0]);
    assert!(tex_str.contains(r"\begin{array}"), "missing \\begin{{array}}: {tex_str}");
    assert!(tex_str.contains(r"\end{array}"), "missing \\end{{array}}: {tex_str}");
    assert!(tex_str.contains(r"\hline"), "missing \\hline: {tex_str}");
    // Grid tables should have vertical borders.
    assert!(tex_str.contains("|"), "expected vertical bars for grid: {tex_str}");
}

#[test]
fn table_alignment_columns() {
    let md = r#"| Left | Center | Right |
| :--- | :----: | ----: |
| 1    |   2    |     3 |"#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    let tex_str = tex(&bs[0]);
    assert!(tex_str.contains("{|l|c|r|}"), "expected lcr alignment in: {tex_str}");
}

#[test]
fn table_cell_inline_math() {
    let md = r#"| Expr | Value |
|------|-------|
| $x^2$ | 4 |"#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    let tex_str = tex(&bs[0]);
    assert!(tex_str.contains("x^2"), "inline math should be raw LaTeX in table cell: {tex_str}");
}

#[test]
fn table_cell_wikilink() {
    let md = r#"| Ref |
|-----|
| [[Category Theory]] |"#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    let tex_str = tex(&bs[0]);
    assert!(tex_str.contains("Category Theory"), "wikilink text should appear in cell: {tex_str}");
}

#[test]
fn table_empty_cells() {
    let md = r#"| A | B |
|---|---|
|   | 2 |"#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.math");
    let tex_str = tex(&bs[0]);
    // Should still compile without panic.
    assert!(tex_str.contains(r"\begin{array}"));
}

#[test]
fn table_adjacent_text() {
    let md = r#"Before.

| A | B |
|---|---|
| 1 | 2 |

After."#;
    let json = convert(md);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 3);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[0]), "Before.");
    assert_eq!(block_type(&bs[1]), "pub.leaflet.blocks.math");
    assert_eq!(block_type(&bs[2]), "pub.leaflet.blocks.text");
    assert_eq!(plaintext(&bs[2]), "After.");
}
