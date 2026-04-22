#![allow(missing_docs)]
mod common;
use common::*;

#[test]
fn minimal_markdown_snapshot() {
    let json = convert("Hello world.");
    insta::assert_json_snapshot!(json);
}

#[test]
fn rich_document_snapshot() {
    let md = r#"# Introduction

This is a **rich** Markdown document with *formatting*, `inline code`, and math: $E=mc^2$.

## Lists

- First unordered item
- Second unordered item

1. First ordered item
2. Second ordered item

## Code

```python
def hello():
    return "world"
```

## Blockquote

> A famous quote.
> Spread across lines.

---

![Diagram](/images/diagram.png)
"#;
    let json = convert(md);
    insta::assert_json_snapshot!(json);
}
