# markdown-to-leaflet

Markdown → Leaflet.pub document converter.

This crate converts CommonMark / GitHub Flavored Markdown into [`Leaflet`](https://leaflet.pub) document schemas, mirroring the architecture of `latex-to-leaflet`.

## Features

- **Headings** (`#` → `##` → `######`) → `pub.leaflet.blocks.header`
- **Paragraphs** → `pub.leaflet.blocks.text`
- **Inline formatting** (`**bold**`, `*italic*`, `` `code` ``) → stripped to plain text (facet support TBD)
- **Code blocks** (fenced + indented) → `pub.leaflet.blocks.code` with language detection
- **Math** (`$...$` and `$$...$$`) → `pub.leaflet.blocks.math`
- **Lists** (ordered + unordered) → `pub.leaflet.blocks.orderedList` / `unorderedList`
- **Blockquotes** (`>`) → `pub.leaflet.blocks.blockquote`
- **Horizontal rules** (`---`) → `pub.leaflet.blocks.horizontalRule`
- **Images** (`![alt](url)`) → `pub.leaflet.blocks.image`

## CLI

```bash
cargo build -p markdown-to-leaflet --release
echo "# Hello" | ./target/release/markdown-to-leaflet-cli
```

## Library

```rust
use markdown_to_leaflet::parse_markdown_to_leaflet;

let schema = parse_markdown_to_leaflet(b"# Hello\n\nWorld.", "test.md")?;
```

## Tests

```bash
cargo test -p markdown-to-leaflet
```
