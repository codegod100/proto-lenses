# @nandithebull/markdown-to-leaflet

TypeScript SDK for the Markdown → Leaflet.pub converter.

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

The SDK discovers the `markdown-to-leaflet-cli` binary automatically by searching:
1. `../../target/release/markdown-to-leaflet-cli`
2. `../../target/debug/markdown-to-leaflet-cli`
3. `markdown-to-leaflet-cli` on `$PATH`

You can also pass an explicit path:

```typescript
const doc = await convertMarkdown(source, {
  binaryPath: '/path/to/markdown-to-leaflet-cli',
});
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
