import { ConvertOptions, LeafletDocument } from './types.js';
/**
 * Convert Markdown source into a Leaflet document.
 *
 * @param source — Raw Markdown source.
 * @param options — Converter options.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link MarkdownConversionError} on spawn or parse failure.
 */
export declare function convertMarkdown(source: string, options?: ConvertOptions): Promise<LeafletDocument>;
//# sourceMappingURL=converter.d.ts.map