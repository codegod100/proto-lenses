//! Markdown document protocol definition.
//!
//! Defines the schema protocol for Markdown source documents,
//! using the same underlying Group E theory (constrained multigraph + W-type
//! + metadata) as Leaflet, but with Markdown-specific vertex kind names.

use panproto_schema::{EdgeRule, Protocol};

/// Returns the Markdown document protocol definition.
#[must_use]
pub fn protocol() -> Protocol {
    Protocol {
        name: "markdown".into(),
        schema_theory: "ThLeafletSchema".into(),
        instance_theory: "ThLeafletInstance".into(),
        edge_rules: edge_rules(),
        obj_kinds: vec![
            "document".into(),
            "page".into(),
            "heading".into(),
            "paragraph".into(),
            "blockquote".into(),
            "code_block".into(),
            "ordered_list".into(),
            "unordered_list".into(),
            "list_item".into(),
            "thematic_break".into(),
            "image".into(),
            "math_block".into(),
        ],
        constraint_sorts: vec![
            "level".into(),
            "language".into(),
            "plaintext".into(),
            "src".into(),
            "tex".into(),
            "startIndex".into(),
            "facet".into(),
        ],
        has_order: true,
        nominal_identity: true,
        ..Protocol::default()
    }
}

fn edge_rules() -> Vec<EdgeRule> {
    vec![
        EdgeRule {
            edge_kind: "items".into(),
            src_kinds: vec![
                "document".into(),
                "page".into(),
                "ordered_list".into(),
                "unordered_list".into(),
                "list_item".into(),
            ],
            tgt_kinds: vec![],
        },
        EdgeRule {
            edge_kind: "prop".into(),
            src_kinds: vec![
                "document".into(),
                "page".into(),
                "heading".into(),
                "paragraph".into(),
                "blockquote".into(),
                "code_block".into(),
                "image".into(),
                "list_item".into(),
            ],
            tgt_kinds: vec![],
        },
    ]
}
