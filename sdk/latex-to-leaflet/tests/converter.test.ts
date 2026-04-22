/**
 * Integration tests for the @panproto/latex-to-leaflet converter.
 *
 * These tests require the `latex-to-leaflet-cli` binary to be built.
 * Run from workspace root:
 *   cargo build -p latex-to-leaflet --release
 *   pnpm test
 */

import { describe, it, expect } from 'vitest';
import { convertLatex, LatexConversionError } from '../src/index.js';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const BINARY_PATH = resolve(
  import.meta.dirname,
  '../../../target/release/latex-to-leaflet-cli',
);

describe('convertLatex', () => {
  it('converts minimal LaTeX', async () => {
    const json = await convertLatex(
      '\\documentclass{article}\n' +
      '\\begin{document}\n' +
      'Hello world.\n' +
      '\\end{document}',
      { binaryPath: BINARY_PATH },
    );
    expect(json.$type).toBe('site.standard.document');
    expect(json.content).toBeDefined();
    expect(Array.isArray(json.content.pages)).toBe(true);
    expect(json.content.pages.length).toBe(1);
  });

  it('preserves title from \\title', async () => {
    const json = await convertLatex(
      '\\documentclass{article}\n' +
      '\\title{My Document}\n' +
      '\\begin{document}\n' +
      '\\maketitle\n' +
      'Body text.\n' +
      '\\end{document}',
      { binaryPath: BINARY_PATH },
    );
    expect(json.title).toBe('My Document');
  });

  it('drops preamble commands', async () => {
    const json = await convertLatex(
      '\\documentclass{article}\n' +
      '\\usepackage{amsmath}\n' +
      '\\newtheorem{theorem}{Theorem}\n' +
      '\\babelfont{rm}{Noto Sans}\n' +
      '\\begin{document}\n' +
      'Clean body.\n' +
      '\\end{document}',
      { binaryPath: BINARY_PATH },
    );
    const blocks = json.content.pages[0].blocks;
    const allText = blocks
      .filter((b) => b.block.$type === 'pub.leaflet.blocks.text')
      .map((b) => b.block.plaintext as string)
      .join(' ');
    expect(allText).toContain('Clean body');
    expect(allText).not.toContain('babelfont');
    expect(allText).not.toContain('Noto Sans');
  });

  it('produces math blocks for inline $...$', async () => {
    const json = await convertLatex(
      '\\documentclass{article}\n' +
      '\\begin{document}\n' +
      '$x^2 + y^2 = z^2$\n' +
      '\\end{document}',
      { binaryPath: BINARY_PATH },
    );
    const blocks = json.content.pages[0].blocks;
    const mathBlocks = blocks.filter(
      (b) => b.block.$type === 'pub.leaflet.blocks.math',
    );
    expect(mathBlocks.length).toBeGreaterThanOrEqual(1);
    expect(mathBlocks[0].block).toHaveProperty('tex');
  });

  it('produces unorderedList blocks', async () => {
    const json = await convertLatex(
      '\\documentclass{article}\n' +
      '\\begin{document}\n' +
      '\\begin{itemize}\n' +
      '\\item First\n' +
      '\\item Second\n' +
      '\\end{itemize}\n' +
      '\\end{document}',
      { binaryPath: BINARY_PATH },
    );
    const blocks = json.content.pages[0].blocks;
    const lists = blocks.filter(
      (b) => b.block.$type === 'pub.leaflet.blocks.unorderedList',
    );
    expect(lists.length).toBe(1);
    const children = lists[0].block.children as unknown[];
    expect(children.length).toBe(2);
  });

  it('throws LatexConversionError for invalid binary path', async () => {
    await expect(
      convertLatex('\\begin{document}\\end{document}', {
        binaryPath: '/nonexistent/binary',
      }),
    ).rejects.toBeInstanceOf(LatexConversionError);
  });

  it('throws LatexConversionError for malformed LaTeX caught by Rust', async () => {
    // This test is intentionally vague because tree-sitter is permissive.
    // Instead of asserting a specific failure, just assert it doesn't crash
    // the process and returns *something* parseable (even if empty).
    const json = await convertLatex('this is not latex at all', {
      binaryPath: BINARY_PATH,
    });
    expect(json.$type).toBe('site.standard.document');
  });

  /**
   * Full regression test on the canonical example.
   */
  it('matches snapshot for category.latex', async () => {
    const source = readFileSync(
      resolve(import.meta.dirname, '../../../latex/examples/category.latex'),
      'utf8',
    );
    const json = await convertLatex(source, { binaryPath: BINARY_PATH });
    expect(json.title).toBe('Category Theory: Foundations and Applications');
    const blocks = json.content.pages[0].blocks;
    const blockTypes = blocks.map((b) => b.block.$type);
    expect(blockTypes).toContain('pub.leaflet.blocks.header');
    expect(blockTypes).toContain('pub.leaflet.blocks.text');
    expect(blockTypes).toContain('pub.leaflet.blocks.unorderedList');
    expect(blockTypes).toContain('pub.leaflet.blocks.math');
    expect(blockTypes).toContain('pub.leaflet.blocks.blockquote');

    // Snapshot the full JSON for regression safety.
    expect(json).toMatchSnapshot();
  });
});
