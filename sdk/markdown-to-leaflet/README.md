# @nandithebull/markdown-to-leaflet

TypeScript SDK for the Markdown → Leaflet.pub converter.

Powered by a native Node.js addon (napi-rs) — no external binaries required.

## Install

```bash
npm install @nandithebull/markdown-to-leaflet
```

## Usage

```typescript
import { convertMarkdown } from '@nandithebull/markdown-to-leaflet';

const doc = await convertMarkdown(`
# Hello World

This is **bold** and *italic*.

- Item 1
- Item 2
`);

console.log(doc.content.pages[0].blocks);
```

## Error handling

```typescript
import { convertMarkdown, MarkdownConversionError } from '@nandithebull/markdown-to-leaflet';

try {
  const doc = await convertMarkdown(source);
} catch (err) {
  if (err instanceof MarkdownConversionError) {
    console.error('Conversion failed:', err.message);
  }
}
```
