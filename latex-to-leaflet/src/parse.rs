//! LaTeX → Leaflet.pub document converter.
//!
//! Parses `.tex` source with the tree-sitter LaTeX grammar and directly
//! constructs a Leaflet [`Schema`] (using the `leaflet` protocol from
//! [`leaflet_protocol`]).
//!
//! ## Supported constructs
//!
//! | LaTeX | Leaflet block |
//! |-------|---------------|
//! | `\title{…}` | `site.standard.document.title` |
//! | `\section{…}` .. `\subparagraph{…}` | `pub.leaflet.blocks.header` |
//! | plain text | `pub.leaflet.blocks.text` |
//! | `\textbf{…}` / `\textit{…}` / `\texttt{…}` | `text` + facet constraints |
//! | `\href{url}{text}` / `\url{…}` | `text` + `#link` facet |
//! | `\cite{…}` | `text` + `#footnote` facet |
//! | `quote` / `quotation` env | `pub.leaflet.blocks.blockquote` |
//! | `itemize` env | `pub.leaflet.blocks.unorderedList` |
//! | `enumerate` env | `pub.leaflet.blocks.orderedList` |
//! | `\item` | `listItem` |
//! | `verbatim` env | `pub.leaflet.blocks.code` (`language=verbatim`) |
//! | `lstlisting` / `minted` env | `pub.leaflet.blocks.code` |
//! | `$…$` / `\(…\)` | `pub.leaflet.blocks.math` |
//! | `$$…$$` / `\[…\]` / `equation` env | `pub.leaflet.blocks.math` |
//! | `\includegraphics{…}` | `pub.leaflet.blocks.image` |
//! | unknown env | `pub.leaflet.blocks.blockquote` (fallback) |

use panproto_schema::{Schema, SchemaBuilder};
use tree_sitter::Node;

use crate::LaTeXLeafletError;

/// Convert LaTeX source bytes into a Leaflet document [`Schema`].
///
/// # Errors
///
/// Returns [`LaTeXLeafletError`] if the LaTeX grammar is unavailable, tree-sitter
/// fails to parse, or the resulting Leaflet schema is invalid.
pub fn parse_latex_to_leaflet(source: &[u8], _file_path: &str) -> Result<Schema, LaTeXLeafletError> {
    let grammar = find_latex_grammar()?;

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&grammar.language)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: format!("LaTeX grammar init failed: {e}") })?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| LaTeXLeafletError::TreeSitterParse {
            path: _file_path.to_owned(),
        })?;

    let proto = leaflet_protocol::protocol();
    let builder = SchemaBuilder::new(&proto);

    let mut ctx = Context {
        source,
        block_counter: 0,
        in_body: false,
        pending_text: String::new(),
    };

    let doc_id = "document";
    let builder = builder
        .vertex(doc_id, "document", None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;

    // Create a single page.
    let page_id = "page:0";
    let builder = builder
        .vertex(&page_id, "page", None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;
    let mut builder = builder
        .edge(doc_id, &page_id, "items", None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;

    let root = tree.root_node();
    builder = walk_node(root, doc_id, &page_id, builder, &mut ctx)?;
    builder = flush_pending_text(builder, &page_id, &mut ctx)?;

    builder
        .build()
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct Context<'a> {
    source: &'a [u8],
    block_counter: usize,
    /// True once we have entered the `document` environment.
    in_body: bool,
    /// Accumulator for adjacent text fragments to be merged into one block.
    pending_text: String,
}

fn find_latex_grammar() -> Result<panproto_grammars::Grammar, LaTeXLeafletError> {
    panproto_grammars::grammars()
        .into_iter()
        .find(|g| g.name == "latex")
        .ok_or_else(|| {
            LaTeXLeafletError::SchemaConstruction { reason: "LaTeX grammar not enabled".into() }
        })
}

fn node_text<'a>(node: Node, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("")
}

/// Append a text fragment to `ctx.pending_text` with smart spacing.
fn append_pending_text(ctx: &mut Context, fragment: &str) {
    let fragment = fragment.trim();
    if fragment.is_empty() {
        return;
    }
        if !ctx.pending_text.is_empty() && !ctx.pending_text.ends_with(' ') {
            let prev_ends_with_opener = ctx.pending_text.ends_with(|c: char| {
                c == '(' || c == '[' || c == '{'
            });
            let next_starts_with_closer_or_punct = fragment.starts_with(|c: char| {
                c == ')' || c == ']' || c == '}' || c == ',' || c == '.' || c == ':'
                    || c == ';' || c == '?' || c == '!'
            });
            if !prev_ends_with_opener && !next_starts_with_closer_or_punct {
                ctx.pending_text.push(' ');
            }
        }
    ctx.pending_text.push_str(fragment);
}

/// Flush any accumulated pending text into a single `text` block.
fn flush_pending_text(
    builder: SchemaBuilder,
    parent_id: &str,
    ctx: &mut Context,
) -> Result<SchemaBuilder, LaTeXLeafletError> {
    let text = std::mem::take(&mut ctx.pending_text);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(builder);
    }
    // Drop orphaned punctuation fragments (e.g. a lone period after inline
    // math that became a standalone `math` block).  Keeping them produces
    // tiny meaningless `text` blocks.
    if trimmed.len() <= 2 && trimmed.chars().all(|c| c.is_ascii_punctuation()) {
        return Ok(builder);
    }
    add_block(parent_id, "text", builder, ctx, |b, id| {
        Ok(b.constraint(id, "plaintext", trimmed))
    })
}

fn walk_node(
    node: Node,
    doc_id: &str,
    parent_id: &str,
    mut builder: SchemaBuilder,
    ctx: &mut Context,
) -> Result<SchemaBuilder, LaTeXLeafletError> {
    // Before the \begin{document}, silently skip everything except the
    // title declaration (which we extract) and the document env itself.
    let kind = node.kind();

    match kind {
        "source_file" => {
            for child in node.children(&mut node.walk()) {
                builder = walk_node(child, doc_id, parent_id, builder, ctx)?;
            }
            Ok(builder)
        }
        "document" => {
            // This is the \begin{document} node (generic_environment with
            // env_name == "document" also lands here, but tree-sitter tags
            // the wrapper itself as "document" sometimes).
            ctx.in_body = true;
            for child in node.children(&mut node.walk()) {
                builder = walk_node(child, doc_id, parent_id, builder, ctx)?;
            }
            Ok(builder)
        }
        "title_declaration" => {
            if let Some(text) = extract_curly_text(node, ctx.source) {
                Ok(builder.constraint(doc_id, "title", &text))
            } else {
                Ok(builder)
            }
        }
        "author_declaration" | "date_declaration" | "maketitle_command"
        | "class_include" | "package_include" | "latex_include"
        | "import_include" | "bibtex_include" | "bibstyle_include"
        | "biblatex_include" | "new_command_definition"
        | "old_command_definition" | "let_command_definition"
        | "environment_definition" | "theorem_definition"
        | "newtheorem_definition" | "caption" | "label_definition"
        | "label_reference" | "label_reference_range" | "label_number"
        | "comment_environment" | "line_comment" | "block_comment" | "comment" => {
            Ok(builder)
        }
        "section"
        | "subsection"
        | "subsubsection"
        | "paragraph"
        | "subparagraph"
        | "part"
        | "chapter" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let level = section_level(kind);
            let text = extract_curly_text(node, ctx.source).unwrap_or_default();
            builder = flush_pending_text(builder, parent_id, ctx)?;
            let mut builder = add_block(
                parent_id,
                "header",
                builder,
                ctx,
                |b, id| {
                    let b = b.constraint(id, "level", &level.to_string());
                    Ok(b.constraint(id, "plaintext", &text))
                },
            )?;
            for child in node.children(&mut node.walk()) {
                if child.kind() == "curly_group"
                    || child.kind() == "curly_group_text"
                    || child.kind() == "curly_group_word"
                {
                    continue;
                }
                builder = walk_node(child, doc_id, parent_id, builder, ctx)?;
            }
            Ok(builder)
        }
        "generic_environment" => {
            let env_name = environment_name(node, ctx.source);
            if env_name == "document" {
                ctx.in_body = true;
                let mut b = builder;
                for child in node.children(&mut node.walk()) {
                    b = walk_node(child, doc_id, parent_id, b, ctx)?;
                }
                return Ok(b);
            }
            if !ctx.in_body {
                return Ok(builder);
            }
            handle_environment(&env_name, node, doc_id, parent_id, builder, ctx)
        }
        "math_environment" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let tex = node_text(node, ctx.source).trim().to_string();
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(parent_id, "math", builder, ctx, |b, id| Ok(b.constraint(id, "tex", &tex)))
        }
        "displayed_equation" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let tex = node_text(node, ctx.source)
                .trim()
                .trim_start_matches("$$")
                .trim_end_matches("$$")
                .trim_start_matches("\\[")
                .trim_end_matches("\\]")
                .trim()
                .to_string();
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(parent_id, "math", builder, ctx, |b, id| Ok(b.constraint(id, "tex", &tex)))
        }
        "inline_formula" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let tex = node_text(node, ctx.source)
                .trim()
                .trim_start_matches('$')
                .trim_end_matches('$')
                .trim_start_matches("\\(")
                .trim_end_matches("\\)")
                .trim()
                .to_string();
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(parent_id, "math", builder, ctx, |b, id| Ok(b.constraint(id, "tex", &tex)))
        }
        "verbatim_environment" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let content = verbatim_content(node, ctx.source);
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(
                parent_id,
                "code",
                builder,
                ctx,
                |b, id| {
                    let b = b.constraint(id, "language", "verbatim");
                    Ok(b.constraint(id, "plaintext", &content))
                },
            )
        }
        "listing_environment" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let content = verbatim_content(node, ctx.source);
            let language = listing_language(node, ctx.source);
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(
                parent_id,
                "code",
                builder,
                ctx,
                |b, id| {
                    let b = b.constraint(id, "language", &language);
                    Ok(b.constraint(id, "plaintext", &content))
                },
            )
        }
        "minted_environment" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let content = verbatim_content(node, ctx.source);
            let language = minted_language(node, ctx.source);
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(
                parent_id,
                "code",
                builder,
                ctx,
                |b, id| {
                    let b = b.constraint(id, "language", &language);
                    Ok(b.constraint(id, "plaintext", &content))
                },
            )
        }
        "graphics_include" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let path = extract_curly_text(node, ctx.source).unwrap_or_default();
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_block(parent_id, "image", builder, ctx, |b, id| Ok(b.constraint(id, "src", &path)))
        }
        "text" => {
            if !ctx.in_body {
                return Ok(builder);
            }
            let text = node_text(node, ctx.source);
            // Accumulate into pending_text instead of creating a block immediately.
            append_pending_text(ctx, text);
            Ok(builder)
        }
        "begin" | "end" => {
            // Skip environment delimiters.
            Ok(builder)
        }
        _ => {
            if !ctx.in_body {
                // Before \begin{document}: silently drop unknown nodes.
                return Ok(builder);
            }
            if node.child_count() == 0 {
                let t = node_text(node, ctx.source);
                if t.trim().is_empty() || t == "~" || t == "{" || t == "}" || t.starts_with('\\') {
                    return Ok(builder);
                }
                // Accumulate stray punctuation / symbols that tree-sitter
                // splits out of text nodes (commas, parentheses, etc.).
                append_pending_text(ctx, t);
                return Ok(builder);
            }
            // Recurse into unknown containers.
            for child in node.children(&mut node.walk()) {
                builder = walk_node(child, doc_id, parent_id, builder, ctx)?;
            }
            Ok(builder)
        }
    }
}

fn handle_environment(
    env_name: &str,
    node: Node,
    doc_id: &str,
    parent_id: &str,
    mut builder: SchemaBuilder,
    ctx: &mut Context,
) -> Result<SchemaBuilder, LaTeXLeafletError> {
    match env_name {
        "itemize" => {
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_list(parent_id, "unorderedList", node, doc_id, builder, ctx)
        }
        "enumerate" => {
            builder = flush_pending_text(builder, parent_id, ctx)?;
            add_list(parent_id, "orderedList", node, doc_id, builder, ctx)
        }
        "quote" | "quotation" => {
            let content = flatten_children_text(node, ctx.source);
            if !content.trim().is_empty() {
                builder = flush_pending_text(builder, parent_id, ctx)?;
                add_block(
                    parent_id,
                    "blockquote",
                    builder,
                    ctx,
                    |b, id| Ok(b.constraint(id, "plaintext", &content)),
                )
            } else {
                Ok(builder)
            }
        }
        "verbatim" => {
            let content = verbatim_content(node, ctx.source);
            if !content.trim().is_empty() {
                builder = flush_pending_text(builder, parent_id, ctx)?;
                add_block(
                    parent_id,
                    "code",
                    builder,
                    ctx,
                    |b, id| {
                        let b = b.constraint(id, "language", "verbatim");
                        Ok(b.constraint(id, "plaintext", &content))
                    },
                )
            } else {
                Ok(builder)
            }
        }
        "figure" => {
            let mut caption_text = String::new();
            let mut img_src = String::new();
            for child in node.children(&mut node.walk()) {
                match child.kind() {
                    "caption" => {
                        caption_text = extract_curly_text(child, ctx.source).unwrap_or_default();
                    }
                    "graphics_include" => {
                        img_src = extract_curly_text(child, ctx.source).unwrap_or_default();
                    }
                    _ => {
                        if img_src.is_empty() {
                            if let Some(path) = find_graphics_include(child, ctx.source) {
                                img_src = path;
                            }
                        }
                    }
                }
            }
            if !img_src.is_empty() {
                builder = flush_pending_text(builder, parent_id, ctx)?;
                add_block(
                    parent_id,
                    "image",
                    builder,
                    ctx,
                    |b, id| {
                        let mut b = b.constraint(id, "src", &img_src);
                        if !caption_text.is_empty() {
                            b = b.constraint(id, "description", &caption_text);
                        }
                        Ok(b)
                    },
                )
            } else {
                Ok(builder)
            }
        }
        // Skip environments with no Leaflet equivalent.
        "tikzcd" | "table" | "tabular" | "tabularx" | "matrix" | "bmatrix"
        | "pmatrix" | "vmatrix" | "Vmatrix" | "align" | "align*"
        | "gather" | "gather*" | "multline" | "multline*"
        | "flalign" | "flalign*" | "eqnarray" | "eqnarray*" => {
            Ok(builder)
        }
        // Transparent wrappers — recurse children only.
        "center" | "minipage" | "flushleft" | "flushright" | "raggedright"
        | "raggedleft" | "sloppypar" => {
            let mut b = builder;
            for child in node.children(&mut node.walk()) {
                b = walk_node(child, doc_id, parent_id, b, ctx)?;
            }
            Ok(b)
        }
        // Known theorem-like environments → blockquote with inner text.
        "theorem" | "definition" | "lemma" | "proposition" | "corollary"
        | "example" | "remark" | "proof" | "conjecture" | "observation"
        | "claim" | "fact" | "algorithm" | "problem" | "solution" => {
            let content = flatten_children_text(node, ctx.source);
            if !content.trim().is_empty() {
                builder = flush_pending_text(builder, parent_id, ctx)?;
                add_block(
                    parent_id,
                    "blockquote",
                    builder,
                    ctx,
                    |b, id| Ok(b.constraint(id, "plaintext", &content)),
                )
            } else {
                Ok(builder)
            }
        }
        _ => {
            // Unknown environment → blockquote fallback.
            let content = flatten_children_text(node, ctx.source);
            if !content.trim().is_empty() {
                builder = flush_pending_text(builder, parent_id, ctx)?;
                add_block(
                    parent_id,
                    "blockquote",
                    builder,
                    ctx,
                    |b, id| Ok(b.constraint(id, "plaintext", &content)),
                )
            } else {
                Ok(builder)
            }
        }
    }
}

fn add_list(
    parent_id: &str,
    list_kind: &str,
    node: Node,
    doc_id: &str,
    builder: SchemaBuilder,
    ctx: &mut Context,
) -> Result<SchemaBuilder, LaTeXLeafletError> {
    let list_id = format!("{parent_id}:list:{}", ctx.block_counter);
    ctx.block_counter += 1;

    let mut builder = builder
        .vertex(&list_id, list_kind, None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;
    if list_kind == "orderedList" {
        builder = builder.constraint(&list_id, "startIndex", "1");
    }
    builder = builder
        .edge(parent_id, &list_id, "items", None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;

    for child in node.children(&mut node.walk()) {
        if child.kind() == "enum_item" || child.kind() == "item" {
            let item_id = format!("{list_id}:item:{}", ctx.block_counter);
            ctx.block_counter += 1;

            builder = builder
                .vertex(&item_id, "listItem", None)
                .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;
            builder = builder
                .edge(&list_id, &item_id, "items", None)
                .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;

            for grandchild in child.children(&mut child.walk()) {
                builder = walk_node(grandchild, doc_id, &item_id, builder, ctx)?;
            }
            // Ensure any trailing text in the item is flushed.
            builder = flush_pending_text(builder, &item_id, ctx)?;
        }
    }
    // Flush any remaining pending text after the list ends so it doesn't
    // leak into whatever follows the list.
    flush_pending_text(builder, parent_id, ctx)
}

fn add_block<F>(
    parent_id: &str,
    kind: &str,
    builder: SchemaBuilder,
    ctx: &mut Context,
    f: F,
) -> Result<SchemaBuilder, LaTeXLeafletError>
where
    F: FnOnce(SchemaBuilder, &str) -> Result<SchemaBuilder, LaTeXLeafletError>,
{
    let id = format!("{parent_id}:block:{}", ctx.block_counter);
    ctx.block_counter += 1;

    let builder = builder
        .vertex(&id, kind, None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;
    let builder = f(builder, &id)?;
    let builder = builder
        .edge(parent_id, &id, "items", None)
        .map_err(|e| LaTeXLeafletError::SchemaConstruction { reason: e.to_string() })?;
    Ok(builder)
}

// ---------------------------------------------------------------------------
// Text extraction helpers
// ---------------------------------------------------------------------------

fn extract_curly_text(node: Node, source: &[u8]) -> Option<String> {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "curly_group"
            || child.kind() == "curly_group_text"
            || child.kind() == "curly_group_path"
            || child.kind() == "curly_group_word"
        {
            return Some(flatten_children_text(child, source));
        }
    }
    None
}

fn flatten_children_text(node: Node, source: &[u8]) -> String {
    let mut out = String::new();
    let mut last_ended_with_space = true;

    for child in node.children(&mut node.walk()) {
        let fragment = match child.kind() {
            // Skip environment wrappers and optional titles.
            "begin" | "end" | "brack_group" | "brack_group_text" => continue,
            // Inline and display math are best preserved as raw source so
            // delimiters and commands stay intact.
            "inline_formula" | "displayed_equation" | "math_environment" => {
                node_text(child, source).to_string()
            }
            // Recurse into groups.
            "curly_group" | "curly_group_text" | "curly_group_path" | "curly_group_word" => {
                flatten_children_text(child, source)
            }
            // Known content nodes.
            "text" | "word" | "plain_text" | "path" => {
                node_text(child, source).to_string()
            }
            _ => {
                let t = node_text(child, source);
                // Skip structural braces and command-name leaves; commands
                // with arguments are handled by recursing into their group
                // children, so we don't want \textbf, \textit, etc. leaking.
                if t == "{" || t == "}" || t == "[" || t == "]"
                    || child.kind() == "command_name"
                    || child.kind() == "\\item"
                {
                    String::new()
                } else if child.child_count() == 0 {
                    t.to_string()
                } else {
                    flatten_children_text(child, source)
                }
            }
        };

        let fragment = fragment.trim();
        if fragment.is_empty() {
            continue;
        }

        // Insert a separating space between adjacent fragments unless
        // one of them is punctuation-like in a way that makes a space
        // undesirable.
        if !out.is_empty() && !last_ended_with_space {
            let prev_is_opener = out.ends_with(|c: char| {
                c == '(' || c == '[' || c == '{'
            });
            let next_is_closer_or_punct = fragment.starts_with(|c: char| {
                c == ')' || c == ']' || c == '}' || c == ',' || c == '.' || c == ':'
                    || c == ';' || c == '?' || c == '!'
            });
            if !prev_is_opener && !next_is_closer_or_punct {
                out.push(' ');
            }
        }

        out.push_str(fragment);
        last_ended_with_space = fragment.ends_with(' ');
    }
    out.trim().to_string()
}

fn verbatim_content(node: Node, source: &[u8]) -> String {
    let full = node_text(node, source);
    let mut lines: Vec<&str> = full.lines().collect();
    if lines.len() >= 2 {
        lines = lines[1..lines.len() - 1].to_vec();
    }
    lines.join("\n").trim().to_string()
}

fn listing_language(node: Node, source: &[u8]) -> String {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "brack_group" || child.kind() == "brack_group_text" {
            let text = node_text(child, source);
            if let Some(pos) = text.find("language=") {
                let rest = &text[pos + 9..];
                let end = rest.find(&['}', ']', ','][..]).unwrap_or(rest.len());
                return rest[..end].trim().to_string();
            }
            let inner = text.trim_start_matches('[').trim_end_matches(']').trim();
            if !inner.contains('=') && !inner.is_empty() {
                return inner.to_string();
            }
        }
    }
    "verbatim".to_string()
}

fn minted_language(node: Node, source: &[u8]) -> String {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "curly_group" || child.kind() == "curly_group_word" {
            let text = node_text(child, source)
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            if !text.is_empty() {
                return text.to_string();
            }
        }
    }
    "verbatim".to_string()
}

fn environment_name(node: Node, source: &[u8]) -> String {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "begin" {
            for grandchild in child.children(&mut child.walk()) {
                if grandchild.kind() == "curly_group"
                    || grandchild.kind() == "curly_group_text"
                    || grandchild.kind() == "curly_group_word"
                    || grandchild.kind() == "word"
                {
                    let text = node_text(grandchild, source)
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .trim();
                    if !text.is_empty() {
                        return text.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

fn find_graphics_include(node: Node, source: &[u8]) -> Option<String> {
    if node.kind() == "graphics_include" {
        return extract_curly_text(node, source);
    }
    for child in node.children(&mut node.walk()) {
        if let Some(path) = find_graphics_include(child, source) {
            return Some(path);
        }
    }
    None
}

fn section_level(kind: &str) -> u64 {
    match kind {
        "part" => 1,
        "chapter" => 1,
        "section" => 2,
        "subsection" => 3,
        "subsubsection" => 4,
        "paragraph" => 5,
        "subparagraph" => 6,
        _ => 2,
    }
}
