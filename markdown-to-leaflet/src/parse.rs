//! Markdown document parser.
//!
//! Parses CommonMark / GFM with `pulldown-cmark` and constructs a
//! Markdown [`Schema`] using the markdown protocol.

use panproto_schema::{Schema, SchemaBuilder};
use pulldown_cmark::{Event, Tag, TagEnd};

use crate::unicode::latex_to_unicode;
use crate::MarkdownLeafletError;

/// Convert Markdown source bytes into a Markdown document [`Schema`].
pub fn parse_markdown(
    source: &[u8],
    _file_path: &str,
) -> Result<Schema, MarkdownLeafletError> {
    let text = std::str::from_utf8(source).unwrap_or("");
    let proto = crate::protocol::protocol();
    let builder = SchemaBuilder::new(&proto);

    let doc_id = "document";
    let builder = builder
        .vertex(doc_id, "document", None)
        .map_err(|e| err_schema(e))?;

    let page_id = "page:0";
    let builder = builder
        .vertex(&page_id, "page", None)
        .map_err(|e| err_schema(e))?;
    let mut builder = builder
        .edge(doc_id, &page_id, "items", None)
        .map_err(|e| err_schema(e))?;

    let mut ctx = Context {
        block_counter: 0,
        pending_text: String::new(),
        blockquote_buf: None,
    };

    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_MATH);
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    opts.insert(pulldown_cmark::Options::ENABLE_WIKILINKS);
    let parser = pulldown_cmark::Parser::new_ext(text, opts);
    let walker = EventWalker {
        page_id: page_id.to_string(),
        active_parent: page_id.to_string(),
        stack: Vec::new(),
        link_url: None,
        pending_code_lang: None,
        table_state: None,
    };
    builder = walker.walk(parser, builder, &mut ctx)?;

    builder
        .build()
        .map_err(|e| MarkdownLeafletError::SchemaConstruction { reason: e.to_string() })
}

// ---------------------------------------------------------------------------
// Context & helpers
// ---------------------------------------------------------------------------

struct Context {
    block_counter: usize,
    pending_text: String,
    /// When `Some`, we are inside a blockquote and text should accumulate here
    /// instead of `pending_text`.
    blockquote_buf: Option<String>,
}

/// Append a text fragment to whichever buffer is currently active.
fn append_text(ctx: &mut Context, fragment: &str) {
    let buf = match ctx.blockquote_buf {
        Some(ref mut b) => b,
        None => &mut ctx.pending_text,
    };
    let fragment = fragment.trim();
    if fragment.is_empty() {
        return;
    }
    if !buf.is_empty() && !buf.ends_with(|c: char| c.is_ascii_whitespace()) {
        let prev_ends_with_opener =
            buf.ends_with(|c: char| c == '(' || c == '[' || c == '{');
        let next_starts_with_closer_or_punct = fragment.starts_with(|c: char| {
            c == ')'
                || c == ']'
                || c == '}'
                || c == ','
                || c == '.'
                || c == ':'
                || c == ';'
                || c == '?'
                || c == '!'
        });
        if !prev_ends_with_opener && !next_starts_with_closer_or_punct {
            buf.push(' ');
        }
    }
    buf.push_str(fragment);
}

fn flush_pending_text(
    builder: SchemaBuilder,
    parent_id: &str,
    ctx: &mut Context,
) -> Result<SchemaBuilder, MarkdownLeafletError> {
    let text = std::mem::take(&mut ctx.pending_text);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(builder);
    }
    if trimmed.len() <= 2 && trimmed.chars().all(|c| c.is_ascii_punctuation()) {
        return Ok(builder);
    }
    add_block(parent_id, "paragraph", builder, ctx, |b, id| {
        Ok(b.constraint(id, "plaintext", trimmed))
    })
}

fn flush_blockquote(
    builder: SchemaBuilder,
    parent_id: &str,
    ctx: &mut Context,
) -> Result<SchemaBuilder, MarkdownLeafletError> {
    let text = ctx.blockquote_buf.take().unwrap_or_default();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(builder);
    }
    add_block(parent_id, "blockquote", builder, ctx, |b, id| {
        Ok(b.constraint(id, "plaintext", trimmed))
    })
}

fn add_block<F>(
    parent_id: &str,
    kind: &str,
    builder: SchemaBuilder,
    ctx: &mut Context,
    f: F,
) -> Result<SchemaBuilder, MarkdownLeafletError>
where
    F: FnOnce(SchemaBuilder, &str) -> Result<SchemaBuilder, MarkdownLeafletError>,
{
    let id = format!("{parent_id}:block:{:04}", ctx.block_counter);
    ctx.block_counter += 1;

    let builder = builder
        .vertex(&id, kind, None)
        .map_err(|e| err_schema(e))?;
    let builder = f(builder, &id)?;
    let builder = builder
        .edge(parent_id, &id, "items", None)
        .map_err(|e| err_schema(e))?;
    Ok(builder)
}

fn add_list(
    parent_id: &str,
    list_kind: &str,
    builder: SchemaBuilder,
    ctx: &mut Context,
) -> Result<(SchemaBuilder, String), MarkdownLeafletError> {
    let list_id = format!("{parent_id}:list:{:04}", ctx.block_counter);
    ctx.block_counter += 1;

    let mut builder = builder
        .vertex(&list_id, list_kind, None)
        .map_err(|e| err_schema(e))?;
    if list_kind == "ordered_list" {
        builder = builder.constraint(&list_id, "startIndex", "1");
    }
    builder = builder
        .edge(parent_id, &list_id, "items", None)
        .map_err(|e| err_schema(e))?;
    Ok((builder, list_id))
}

fn add_list_item(
    list_id: &str,
    builder: SchemaBuilder,
    ctx: &mut Context,
) -> Result<(SchemaBuilder, String), MarkdownLeafletError> {
    let item_id = format!("{list_id}:item:{:04}", ctx.block_counter);
    ctx.block_counter += 1;

        let builder = builder
            .vertex(&item_id, "list_item", None)
            .map_err(|e| err_schema(e))?;
    let builder = builder
        .edge(list_id, &item_id, "items", None)
        .map_err(|e| err_schema(e))?;
    Ok((builder, item_id))
}

fn err_schema(e: panproto_schema::SchemaError) -> MarkdownLeafletError {
    MarkdownLeafletError::SchemaConstruction {
        reason: e.to_string(),
    }
}

/// Escape plain-text characters that are special in LaTeX math mode.
fn escape_latex(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '{' => "\\{".to_string(),
            '}' => "\\}".to_string(),
            '$' => "\\$".to_string(),
            '&' => "\\&".to_string(),
            '#' => "\\#".to_string(),
            '%' => "\\%".to_string(),
            '_' => "\\_".to_string(),
            '^' => "\\^".to_string(),
            '~' => "\\textasciitilde{}".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

/// One segment of a table cell's LaTeX representation.
enum CellSegment {
    Text(String),
    Math(String),
}

/// Accumulates inline content for a single table cell.
struct CellAccumulator {
    segments: Vec<CellSegment>,
    pending_text: String,
}

impl CellAccumulator {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
            pending_text: String::new(),
        }
    }

    fn push_text(&mut self, text: &str) {
        self.pending_text.push_str(text);
    }

    fn push_space(&mut self) {
        if !self.pending_text.ends_with(' ') {
            self.pending_text.push(' ');
        }
    }

    fn push_math(&mut self, tex: &str) {
        self.flush_text();
        self.segments.push(CellSegment::Math(tex.to_string()));
    }

    fn flush_text(&mut self) {
        if !self.pending_text.is_empty() {
            self.segments
                .push(CellSegment::Text(std::mem::take(&mut self.pending_text)));
        }
    }

    fn build(mut self) -> String {
        self.flush_text();
        self.segments
            .iter()
            .map(|s| match s {
                CellSegment::Text(t) => format!(r"\text{{{}}}", escape_latex(t)),
                CellSegment::Math(m) => m.clone(),
            })
            .collect()
    }
}

/// State collected while walking a Markdown table.
struct TableState {
    alignments: Vec<pulldown_cmark::Alignment>,
    rows: Vec<Vec<String>>,
    cur_row: Vec<String>,
    cur_cell: CellAccumulator,
}

impl TableState {
    fn new(alignments: Vec<pulldown_cmark::Alignment>) -> Self {
        Self {
            alignments,
            rows: Vec::new(),
            cur_row: Vec::new(),
            cur_cell: CellAccumulator::new(),
        }
    }

    fn build_latex(self) -> String {
        let cols = self
            .alignments
            .iter()
            .map(|a| match a {
                pulldown_cmark::Alignment::Left | pulldown_cmark::Alignment::None => "l",
                pulldown_cmark::Alignment::Center => "c",
                pulldown_cmark::Alignment::Right => "r",
            })
            .collect::<String>();

        let grid = cols.chars().map(|c| format!("|{}", c)).collect::<String>() + "|";

        let mut lines = vec![format!(r"\begin{{array}}{{{}}}", grid)];
        lines.push(r"  \hline".to_string());

        for row in self.rows {
            let row_content = row.join(" & ");
            lines.push(format!("  {} \\\\", row_content));
            lines.push(r"  \hline".to_string());
        }

        lines.push(r"\end{array}".to_string());
        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Event walker
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct StackFrame {
    container_kind: ContainerKind,
    list_id: Option<String>,
    item_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerKind {
    BlockQuote,
    List,
    Item,
}

struct EventWalker {
    page_id: String,
    active_parent: String,
    stack: Vec<StackFrame>,
    link_url: Option<String>,
    pending_code_lang: Option<String>,
    table_state: Option<TableState>,
}

impl EventWalker {
    fn walk<'a>(
        mut self,
        parser: pulldown_cmark::Parser<'a>,
        mut builder: SchemaBuilder,
        ctx: &mut Context,
    ) -> Result<SchemaBuilder, MarkdownLeafletError> {
        for event in parser {
            match event {
                // =====================================================================
                // Start tags
                // =====================================================================
                Event::Start(tag) => {
                    match tag {
                        // --- Block containers ------------------------------------------------
                        Tag::BlockQuote(_) => {
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            ctx.blockquote_buf = Some(String::new());
                            self.stack.push(StackFrame {
                                container_kind: ContainerKind::BlockQuote,
                                list_id: None,
                                item_id: None,
                            });
                        }
                        Tag::List(start_num) => {
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            // If inside a blockquote, end it first (blockquote is terminal).
                            if ctx.blockquote_buf.is_some() {
                                builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                                self.stack.pop();
                            }
                            let list_kind = if start_num.is_some() {
                                "ordered_list"
                            } else {
                                "unordered_list"
                            };
                            let (b, list_id) =
                                add_list(&self.active_parent, list_kind, builder, ctx)?;
                            builder = b;
                            self.stack.push(StackFrame {
                                container_kind: ContainerKind::List,
                                list_id: Some(list_id.clone()),
                                item_id: None,
                            });
                            self.active_parent = list_id;
                        }
                        Tag::Item => {
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            if let Some(list_id) =
                                self.stack.last().and_then(|f| f.list_id.clone())
                            {
                                let (b, item_id) =
                                    add_list_item(&list_id, builder, ctx)?;
                                builder = b;
                                if let Some(last) = self.stack.last_mut() {
                                    last.item_id = Some(item_id.clone());
                                }
                                self.stack.push(StackFrame {
                                    container_kind: ContainerKind::Item,
                                    list_id: Some(list_id),
                                    item_id: Some(item_id.clone()),
                                });
                                self.active_parent = item_id;
                            }
                        }

                        // --- Paragraph boundary -----------------------------------------------
                        Tag::Paragraph => {
                            // Paragraphs inside blockquotes are normal text containers.
                            // Outside blockquotes they just start a new implicit text block.
                            if ctx.blockquote_buf.is_none() {
                                builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            }
                        }

                        // --- Terminal block boundaries -----------------------------------------
                        Tag::Heading { .. } | Tag::CodeBlock(_) => {
                            // If inside a blockquote, end it first.
                            if ctx.blockquote_buf.is_some() {
                                builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                                self.stack.pop();
                            }
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            if let Tag::CodeBlock(kind) = tag {
                                self.pending_code_lang = match kind {
                                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                                        let s = lang.to_string();
                                        if s.is_empty() { None } else { Some(s) }
                                    }
                                    _ => None,
                                };
                            }
                        }

                        // --- Inline formatting ------------------------------------------------
                        Tag::Emphasis | Tag::Strong | Tag::Strikethrough => {}
                        Tag::Link { dest_url, .. } => {
                            self.link_url = Some(dest_url.to_string());
                        }
                        Tag::Image { dest_url, .. } => {
                            if self.table_state.is_some() {
                                // Suppress image blocks inside table cells.
                                self.link_url = Some(dest_url.to_string());
                            } else {
                                if ctx.blockquote_buf.is_some() {
                                    builder =
                                        flush_blockquote(builder, &self.active_parent, ctx)?;
                                    self.stack.pop();
                                }
                                builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                                self.link_url = Some(dest_url.to_string());
                            }
                        }
                        Tag::Table(alignments) => {
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            if ctx.blockquote_buf.is_some() {
                                builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                                self.stack.pop();
                            }
                            self.table_state = Some(TableState::new(alignments.to_vec()));
                        }
                        Tag::TableHead | Tag::TableRow => {}
                        Tag::TableCell => {
                            if let Some(state) = self.table_state.as_mut() {
                                state.cur_cell = CellAccumulator::new();
                            }
                        }

                        _ => {}
                    }
                }

                // =====================================================================
                // End tags
                // =====================================================================
                Event::End(tag_end) => {
                    match tag_end {
                        // --- Container ends --------------------------------------------------
                        TagEnd::BlockQuote(_) => {
                            builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                            self.stack.pop();
                            // Blockquote does not change active_parent.
                        }
                        TagEnd::List(_) => {
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            self.stack.pop();
                            if self.stack.last().map(|f| f.container_kind)
                                == Some(ContainerKind::Item)
                            {
                                self.stack.pop();
                            }
                            self.active_parent = self.restore_parent();
                        }
                        TagEnd::Item => {
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            self.stack.pop();
                            self.active_parent = if let Some(frame) = self.stack.last() {
                                frame
                                    .list_id
                                    .clone()
                                    .unwrap_or_else(|| self.page_id.clone())
                            } else {
                                self.page_id.clone()
                            };
                        }

                        // --- Terminal block ends ---------------------------------------------
                        TagEnd::Heading(level) => {
                            let content = std::mem::take(&mut ctx.pending_text);
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                let level_str = (level as u64).to_string();
                                builder = add_block(
                                    &self.active_parent,
                                    "heading",
                                    builder,
                                    ctx,
                                    |b, id| {
                                        let b = b.constraint(id, "level", &level_str);
                                        Ok(b.constraint(id, "plaintext", trimmed))
                                    },
                                )?;
                            }
                        }
                        TagEnd::CodeBlock => {
                            let content = std::mem::take(&mut ctx.pending_text);
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                let lang = self
                                    .pending_code_lang
                                    .take()
                                    .unwrap_or_else(|| "plaintext".to_string());
                                builder = add_block(
                                    &self.active_parent,
                                    "code_block",
                                    builder,
                                    ctx,
                                    |b, id| {
                                        let b = b.constraint(id, "language", &lang);
                                        Ok(b.constraint(id, "plaintext", trimmed))
                                    },
                                )?;
                            }
                        }
                        TagEnd::Paragraph => {
                            // If we're inside a blockquote, add a blank-line separator
                            // between paragraphs.
                            if ctx.blockquote_buf.is_some() {
                                if let Some(ref mut b) = ctx.blockquote_buf {
                                    if !b.is_empty() && !b.ends_with('\n') {
                                        b.push('\n');
                                        b.push('\n');
                                    }
                                }
                            } else {
                                builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            }
                        }

                        // --- Inline ends ------------------------------------------------------
                        TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {}
                        TagEnd::Link => {
                            self.link_url = None;
                        }
                        TagEnd::Image => {
                            if self.table_state.is_some() {
                                self.link_url = None;
                            } else if let Some(url) = self.link_url.take() {
                                let alt = std::mem::take(&mut ctx.pending_text).trim().to_string();
                                let alt_cloned = alt.clone();
                                builder = add_block(
                                    &self.active_parent,
                                    "image",
                                    builder,
                                    ctx,
                                    |b, id| {
                                        let b = b.constraint(id, "src", &url);
                                        Ok(b.constraint(id, "title", &alt_cloned))
                                    },
                                )?;
                            }
                        }
                        TagEnd::TableCell => {
                            if let Some(state) = self.table_state.as_mut() {
                                let cell_tex = std::mem::replace(&mut state.cur_cell, CellAccumulator::new()).build();
                                state.cur_row.push(cell_tex);
                            }
                        }
                        TagEnd::TableRow | TagEnd::TableHead => {
                            if let Some(state) = self.table_state.as_mut() {
                                if !state.cur_row.is_empty() {
                                    state.rows.push(std::mem::take(&mut state.cur_row));
                                }
                            }
                        }
                        TagEnd::Table => {
                            if let Some(state) = self.table_state.take() {
                                let latex = state.build_latex();
                                builder = add_block(
                                    &self.active_parent,
                                    "math_block",
                                    builder,
                                    ctx,
                                    |b, id| Ok(b.constraint(id, "tex", &latex)),
                                )?;
                            }
                        }
                        _ => {}
                    }
                }

                // =====================================================================
                // Content events
                // =====================================================================
                Event::Text(text) => {
                    if let Some(state) = self.table_state.as_mut() {
                        state.cur_cell.push_text(&text);
                    } else {
                        append_text(ctx, &text);
                    }
                }
                Event::Code(code) => {
                    if let Some(state) = self.table_state.as_mut() {
                        state.cur_cell.push_text(&code);
                    } else {
                        append_text(ctx, &code);
                    }
                }
                Event::Html(_) | Event::InlineHtml(_) => {}
                Event::SoftBreak | Event::HardBreak => {
                    if let Some(state) = self.table_state.as_mut() {
                        state.cur_cell.push_space();
                    } else if ctx.blockquote_buf.is_some() {
                        if let Some(ref mut b) = ctx.blockquote_buf {
                            if !b.is_empty() && !b.ends_with(|c: char| c.is_ascii_whitespace()) {
                                b.push('\n');
                            }
                        }
                    } else {
                        ctx.pending_text.push('\n');
                    }
                }
                Event::Rule => {
                    if self.table_state.is_some() {
                        // Suppress horizontal rules inside table cells.
                    } else {
                        if ctx.blockquote_buf.is_some() {
                            builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                            self.stack.pop();
                        }
                        builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                        builder = add_block(
                            &self.active_parent,
                            "thematic_break",
                            builder,
                            ctx,
                            |b, _id| Ok(b),
                        )?;
                    }
                }
                Event::InlineMath(tex) => {
                    if let Some(state) = self.table_state.as_mut() {
                        state.cur_cell.push_math(&tex);
                    } else {
                        let unicode = latex_to_unicode(&tex);
                        append_text(ctx, &unicode);
                    }
                }
                Event::DisplayMath(tex) => {
                    if let Some(state) = self.table_state.as_mut() {
                        state.cur_cell.push_math(&tex);
                    } else {
                        if ctx.blockquote_buf.is_some() {
                            builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                            self.stack.pop();
                        }
                        builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                        builder = add_block(&self.active_parent, "math_block", builder, ctx, |b, id| {
                            Ok(b.constraint(id, "tex", &tex))
                        })?;
                    }
                }
                Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
            }
        }

        // Final flush of any trailing text.
        if ctx.blockquote_buf.is_some() {
            builder = flush_blockquote(builder, &self.active_parent, ctx)?;
        }
        builder = flush_pending_text(builder, &self.active_parent, ctx)?;
        Ok(builder)
    }

    fn restore_parent(&self) -> String {
        self.stack
            .last()
            .map(|f| match f.container_kind {
                ContainerKind::BlockQuote => f
                    .item_id
                    .clone()
                    .or_else(|| f.list_id.clone())
                    .unwrap_or_else(|| self.page_id.clone()),
                ContainerKind::List => f
                    .item_id
                    .clone()
                    .unwrap_or_else(|| self.page_id.clone()),
                ContainerKind::Item => f
                    .list_id
                    .clone()
                    .unwrap_or_else(|| self.page_id.clone()),
            })
            .unwrap_or_else(|| self.page_id.clone())
    }
}
