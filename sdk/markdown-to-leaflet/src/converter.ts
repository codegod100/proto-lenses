/**
 * Converter implementation.
 *
 * Calls the native Rust addon directly (via napi-rs).
 */

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const binding = require('../binding/binding.cjs') as { markdownToLeaflet: (source: string) => unknown };
const { markdownToLeaflet } = binding;
import {
  type LeafletDocument,
  MarkdownConversionError,
} from './types.js';

/**
 * Convert Markdown source into a Leaflet document.
 *
 * @param source — Raw Markdown source.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link MarkdownConversionError} on parse or lens failure.
 */
export async function convertMarkdown(source: string): Promise<LeafletDocument> {
  try {
    const doc = markdownToLeaflet(source) as LeafletDocument;
    return doc;
  } catch (err) {
    throw new MarkdownConversionError(
      err instanceof Error ? err.message : String(err),
      err instanceof Error ? err : undefined,
    );
  }
}
