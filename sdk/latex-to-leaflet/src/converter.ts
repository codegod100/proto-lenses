/**
 * Converter implementation.
 *
 * Calls the native Rust addon directly (via napi-rs).
 */

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const binding = require('../binding/binding.cjs') as { latexToLeaflet: (source: string) => unknown };
const { latexToLeaflet } = binding;
import {
  type LeafletDocument,
  LatexConversionError,
} from './types.js';

/**
 * Convert LaTeX source into a Leaflet document.
 *
 * @param source — Raw `.tex` source.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link LatexConversionError} on parse or lens failure.
 */
export async function convertLatex(source: string): Promise<LeafletDocument> {
  try {
    const doc = latexToLeaflet(source) as LeafletDocument;
    return doc;
  } catch (err) {
    throw new LatexConversionError(
      err instanceof Error ? err.message : String(err),
      err instanceof Error ? err : undefined,
    );
  }
}
