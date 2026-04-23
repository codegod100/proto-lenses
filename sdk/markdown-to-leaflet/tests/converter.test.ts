/**
 * Integration tests for the @nandithebull/markdown-to-leaflet converter.
 *
 * These tests use the native napi-rs addon, not a CLI binary.
 */

import { describe, it, expect } from 'vitest';
import { convertMarkdown } from '../src/index.js';

describe('convertMarkdown', () => {
  it('converts minimal Markdown', async () => {
    const json = await convertMarkdown('Hello world.');
    expect(json.$type).toBe('site.standard.document');
    expect(json.content).toBeDefined();
    expect(Array.isArray(json.content.pages)).toBe(true);
    expect(json.content.pages.length).toBe(1);
    const blocks = json.content.pages[0].blocks;
    expect(blocks.length).toBe(1);
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.text');
    expect(blocks[0].block.plaintext).toBe('Hello world.');
  });

  it('preserves headings', async () => {
    const json = await convertMarkdown('# Title\n\nBody.');
    const blocks = json.content.pages[0].blocks;
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.header');
    expect(blocks[0].block.plaintext).toBe('Title');
    expect(blocks[0].block.level).toBe(1);
    expect(blocks[1].block.$type).toBe('pub.leaflet.blocks.text');
    expect(blocks[1].block.plaintext).toBe('Body.');
  });

  it('converts inline $...$ math to Unicode in text blocks', async () => {
    const json = await convertMarkdown('$E=mc^2$');
    const blocks = json.content.pages[0].blocks;
    expect(blocks.length).toBe(1);
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.text');
    expect(blocks[0].block.plaintext).toBe('E=mc²');
    // No math block should exist for inline math.
    const mathBlocks = blocks.filter(
      (b) => b.block.$type === 'pub.leaflet.blocks.math',
    );
    expect(mathBlocks.length).toBe(0);
  });

  it('converts inline Greek math to Unicode', async () => {
    const json = await convertMarkdown('$\\alpha + \\beta$');
    const blocks = json.content.pages[0].blocks;
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.text');
    expect(blocks[0].block.plaintext).toBe('α + β');
  });

  it('produces math blocks for $$...$$', async () => {
    const json = await convertMarkdown('$$\\int_0^1 x dx$$');
    const blocks = json.content.pages[0].blocks;
    const mathBlocks = blocks.filter(
      (b) => b.block.$type === 'pub.leaflet.blocks.math',
    );
    expect(mathBlocks.length).toBe(1);
    expect(mathBlocks[0].block.tex).toBe('\\int_0^1 x dx');
  });

  it('coexists inline and display math correctly', async () => {
    const json = await convertMarkdown(
      'Text $x^2$ more text.\n\n$$E=mc^2$$\n\nAfter.',
    );
    const blocks = json.content.pages[0].blocks;
    expect(blocks.length).toBe(3);
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.text');
    expect(blocks[0].block.plaintext).toBe('Text x² more text.');
    expect(blocks[1].block.$type).toBe('pub.leaflet.blocks.math');
    expect(blocks[1].block.tex).toBe('E=mc^2');
    expect(blocks[2].block.$type).toBe('pub.leaflet.blocks.text');
    expect(blocks[2].block.plaintext).toBe('After.');
  });

  it('converts inline math in headings', async () => {
    const json = await convertMarkdown('# Section $\\alpha$');
    const blocks = json.content.pages[0].blocks;
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.header');
    expect(blocks[0].block.plaintext).toBe('Section α');
    expect(blocks[0].block.level).toBe(1);
  });

  it('produces unorderedList blocks', async () => {
    const json = await convertMarkdown('- First\n- Second');
    const blocks = json.content.pages[0].blocks;
    const lists = blocks.filter(
      (b) => b.block.$type === 'pub.leaflet.blocks.unorderedList',
    );
    expect(lists.length).toBe(1);
    const children = lists[0].block.children as unknown[];
    expect(children.length).toBe(2);
  });

  it('produces orderedList blocks', async () => {
    const json = await convertMarkdown('1. One\n2. Two');
    const blocks = json.content.pages[0].blocks;
    const lists = blocks.filter(
      (b) => b.block.$type === 'pub.leaflet.blocks.orderedList',
    );
    expect(lists.length).toBe(1);
    expect(lists[0].block.startIndex).toBe(1);
  });

  it('produces code blocks with language', async () => {
    const json = await convertMarkdown('```rust\nfn main() {}\n```');
    const blocks = json.content.pages[0].blocks;
    expect(blocks[0].block.$type).toBe('pub.leaflet.blocks.code');
    expect(blocks[0].block.language).toBe('rust');
    expect(blocks[0].block.plaintext).toBe('fn main() {}');
  });

  it('matches snapshot for rich document', async () => {
    const source = `# Introduction

This is a **rich** Markdown document with *formatting*, \`inline code\`, and math: $E=mc^2$.

## Lists

- First unordered item
- Second unordered item

1. First ordered item
2. Second ordered item

## Code

\`\`\`python
def hello():
    return "world"
\`\`\`

## Blockquote

> A famous quote.
> Spread across lines.

---

![Diagram](/images/diagram.png)
`;
    const json = await convertMarkdown(source);
    expect(json.$type).toBe('site.standard.document');
    const blocks = json.content.pages[0].blocks;
    const blockTypes = blocks.map((b) => b.block.$type);
    expect(blockTypes).toContain('pub.leaflet.blocks.header');
    expect(blockTypes).toContain('pub.leaflet.blocks.text');
    expect(blockTypes).toContain('pub.leaflet.blocks.unorderedList');
    expect(blockTypes).toContain('pub.leaflet.blocks.orderedList');
    expect(blockTypes).toContain('pub.leaflet.blocks.code');
    expect(blockTypes).toContain('pub.leaflet.blocks.blockquote');
    expect(blockTypes).toContain('pub.leaflet.blocks.horizontalRule');
    expect(blockTypes).toContain('pub.leaflet.blocks.image');
    // Inline math is now Unicode in text blocks, so math blocks should only
    // appear for display math (which this doc doesn't have).

    expect(json).toMatchSnapshot();
  });
});
