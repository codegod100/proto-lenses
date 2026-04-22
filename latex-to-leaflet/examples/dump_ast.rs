//! AST dumper for debugging the LaTeX parser.
use std::fs;

fn main() {
    let path = "latex/examples/category.latex";
    let source = fs::read(path).expect("read");

    let grammars = panproto_grammars::grammars();
    let grammar = grammars.into_iter().find(|g| g.name == "latex").expect("latex grammar");

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar.language).unwrap();
    let tree = parser.parse(&source, None).expect("parse");

    fn print_tree(node: tree_sitter::Node, source: &[u8], depth: usize) {
        let indent = "  ".repeat(depth);
        let text = std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("");
        let preview = if text.len() > 60 { &text[..60] } else { text };
        let preview = preview.replace('\n', "\\n");
        println!("{}{} [{}..{}] {:?}", indent, node.kind(), node.start_byte(), node.end_byte(), preview);
        for child in node.children(&mut node.walk()) {
            print_tree(child, source, depth + 1);
        }
    }

    print_tree(tree.root_node(), &source, 0);
}
