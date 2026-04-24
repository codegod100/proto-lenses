//! LaTeX document protocol definition.
//!
//! Defines the schema protocol for LaTeX source documents,
//! using the same underlying Group E theory (constrained multigraph + W-type
//! + metadata) as Leaflet, but with LaTeX-specific vertex kind names.

use panproto_schema::{EdgeRule, Protocol};

/// Returns the LaTeX document protocol definition.
#[must_use]
pub fn protocol() -> Protocol {
    Protocol {
        name: "latex".into(),
        schema_theory: "ThLeafletSchema".into(),
        instance_theory: "ThLeafletInstance".into(),
        edge_rules: edge_rules(),
        obj_kinds: vec![
            "document".into(),
            "page".into(),
            "section".into(),
            "paragraph".into(),
            "blockquote".into(),
            "quote".into(),
            "verbatim".into(),
            "code_listing".into(),
            "itemize".into(),
            "enumerate".into(),
            "item".into(),
            "equation".into(),
            "figure".into(),
            "image".into(),
        ],
        constraint_sorts: vec![
            "level".into(),
            "language".into(),
            "plaintext".into(),
            "src".into(),
            "tex".into(),
            "startIndex".into(),
            "facet".into(),
            "title".into(),
            "description".into(),
            "display".into(),
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
                "itemize".into(),
                "enumerate".into(),
                "item".into(),
            ],
            tgt_kinds: vec![],
        },
        EdgeRule {
            edge_kind: "prop".into(),
            src_kinds: vec![
                "document".into(),
                "page".into(),
                "section".into(),
                "paragraph".into(),
                "blockquote".into(),
                "quote".into(),
                "verbatim".into(),
                "code_listing".into(),
                "image".into(),
                "item".into(),
            ],
            tgt_kinds: vec![],
        },
    ]
}
