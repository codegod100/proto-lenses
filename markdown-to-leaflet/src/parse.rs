//! Markdown → Leaflet.pub document converter.
//!
//! Parses CommonMark / GFM with `pulldown-cmark` and directly constructs a
//! Leaflet [`Schema`].

use panproto_schema::{Schema, SchemaBuilder};
use pulldown_cmark::{Event, Tag, TagEnd};

use crate::unicode::latex_to_unicode;
use crate::MarkdownLeafletError;

/// Convert Markdown source bytes into a Leaflet document [`Schema`].
pub fn parse_markdown_to_leaflet(
    source: &[u8],
    _file_path: &str,
) -> Result<Schema, MarkdownLeafletError> {
    let text = std::str::from_utf8(source).unwrap_or("");
    let proto = leaflet_protocol::protocol();
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
    let parser = pulldown_cmark::Parser::new_ext(text, opts);
    let walker = EventWalker {
        page_id: page_id.to_string(),
        active_parent: page_id.to_string(),
        stack: Vec::new(),
        link_url: None,
        pending_code_lang: None,
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
    if !buf.is_empty() && !buf.ends_with(' ') {
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
    add_block(parent_id, "text", builder, ctx, |b, id| {
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
    let id = format!("{parent_id}:block:{}", ctx.block_counter);
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
    let list_id = format!("{parent_id}:list:{}", ctx.block_counter);
    ctx.block_counter += 1;

    let mut builder = builder
        .vertex(&list_id, list_kind, None)
        .map_err(|e| err_schema(e))?;
    if list_kind == "orderedList" {
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
    let item_id = format!("{list_id}:item:{}", ctx.block_counter);
    ctx.block_counter += 1;

    let builder = builder
        .vertex(&item_id, "listItem", None)
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
                                "orderedList"
                            } else {
                                "unorderedList"
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
                            if ctx.blockquote_buf.is_some() {
                                builder =
                                    flush_blockquote(builder, &self.active_parent, ctx)?;
                                self.stack.pop();
                            }
                            builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                            self.link_url = Some(dest_url.to_string());
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
                                    "header",
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
                                    "code",
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
                            if let Some(url) = self.link_url.take() {
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
                        _ => {}
                    }
                }

                // =====================================================================
                // Content events
                // =====================================================================
                Event::Text(text) => {
                    append_text(ctx, &text);
                }
                Event::Code(code) => {
                    append_text(ctx, &code);
                }
                Event::Html(_) | Event::InlineHtml(_) => {}
                Event::SoftBreak | Event::HardBreak => {
                    if ctx.blockquote_buf.is_some() {
                        if let Some(ref mut b) = ctx.blockquote_buf {
                            if !b.is_empty() && !b.ends_with(' ') && !b.ends_with('\n') {
                                b.push(' ');
                            }
                        }
                    } else {
                        ctx.pending_text.push(' ');
                    }
                }
                Event::Rule => {
                    if ctx.blockquote_buf.is_some() {
                        builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                        self.stack.pop();
                    }
                    builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                    builder = add_block(
                        &self.active_parent,
                        "horizontalRule",
                        builder,
                        ctx,
                        |b, _id| Ok(b),
                    )?;
                }
                Event::InlineMath(tex) => {
                    // Inline math is converted to Unicode and merged into
                    // the surrounding text block rather than emitted as a
                    // standalone `math` block.
                    let unicode = latex_to_unicode(&tex);
                    append_text(ctx, &unicode);
                }
                Event::DisplayMath(tex) => {
                    if ctx.blockquote_buf.is_some() {
                        builder = flush_blockquote(builder, &self.active_parent, ctx)?;
                        self.stack.pop();
                    }
                    builder = flush_pending_text(builder, &self.active_parent, ctx)?;
                    builder = add_block(&self.active_parent, "math", builder, ctx, |b, id| {
                        Ok(b.constraint(id, "tex", &tex))
                    })?;
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
