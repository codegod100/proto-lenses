/**
 * Converter implementation.
 *
 * Calls the native Rust addon directly (via napi-rs).
 */
import { type LeafletDocument } from './types.js';
/**
 * Convert Markdown source into a Leaflet document.
 *
 * @param source — Raw Markdown source.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link MarkdownConversionError} on parse or lens failure.
 */
export declare function convertMarkdown(source: string): Promise<LeafletDocument>;
//# sourceMappingURL=converter.d.ts.map