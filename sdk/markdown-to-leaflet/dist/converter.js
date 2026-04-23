/**
 * Converter implementation.
 *
 * Calls the native Rust addon directly (via napi-rs).
 */
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const binding = require('../binding/binding.cjs');
const { markdownToLeaflet } = binding;
import { MarkdownConversionError, } from './types.js';
/**
 * Convert Markdown source into a Leaflet document.
 *
 * @param source — Raw Markdown source.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link MarkdownConversionError} on parse or lens failure.
 */
export async function convertMarkdown(source) {
    try {
        const doc = markdownToLeaflet(source);
        return doc;
    }
    catch (err) {
        throw new MarkdownConversionError(err instanceof Error ? err.message : String(err), err instanceof Error ? err : undefined);
    }
}
//# sourceMappingURL=converter.js.map