# latex-to-leaflet

Convert LaTeX documents to the [Leaflet.pub](https://leaflet.pub) document schema.

## Quick start

```rust
use latex_to_leaflet::parse_latex_to_leaflet;

let src = std::fs::read("examples/sample.tex").unwrap();
let schema = parse_latex_to_leaflet(&src, "sample.tex").expect("parse");
```

Convert the resulting schema back to JSON:

```rust
let json = panproto_protocols::leaflet::emit_leaflet_document(&schema).unwrap();
println!("{}", serde_json::to_string_pretty(&json).unwrap());
```

## Supported LaTeX

| LaTeX construct | Leaflet block |
|---|---|
| `\title{…}` | `site.standard.document` title |
| `\section` / `\subsection` / … | `pub.leaflet.blocks.header` |
| Plain text | `pub.leaflet.blocks.text` |
| `$…$` / `\(…\)` | `pub.leaflet.blocks.math` |
| `$$…$$` / `\[…\]` / `equation` | `pub.leaflet.blocks.math` |
| `itemize` | `pub.leaflet.blocks.unorderedList` |
| `enumerate` | `pub.leaflet.blocks.orderedList` |
| `\item` | `pub.leaflet.blocks.listItem` |
| `verbatim` / `lstlisting` / `minted` | `pub.leaflet.blocks.code` |
| `\includegraphics{…}` | `pub.leaflet.blocks.image` |
| `figure` + `\caption` | `pub.leaflet.blocks.image` (caption stored as `description`) |
| `quote` / `quotation` | `pub.leaflet.blocks.blockquote` |
| Unknown `\begin{…}` | `pub.leaflet.blocks.blockquote` (fallback) |
| Preamble (`\documentclass`, `\usepackage`, etc.) | dropped |

## Running the example

```bash
cargo run -p latex-to-leaflet --example convert examples/sample.tex
```

Or in a small script:

```rust
use latex_to_leaflet::parse_latex_to_leaflet;

fn main() {
    let src = include_bytes!("examples/sample.tex");
    let schema = parse_latex_to_leaflet(src, "sample.tex").unwrap();
    let json = panproto_protocols::leaflet::emit_leaflet_document(&schema).unwrap();
    std::fs::write("sample.json", serde_json::to_string_pretty(&json).unwrap()).unwrap();
}
```
