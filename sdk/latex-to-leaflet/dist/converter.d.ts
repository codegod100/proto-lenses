import { ConvertOptions, LeafletDocument } from './types.js';
/**
 * Convert LaTeX source into a Leaflet document.
 *
 * @param source — Raw `.tex` source.
 * @param options — Converter options.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link LatexConversionError} on spawn or parse failure.
 */
export declare function convertLatex(source: string, options?: ConvertOptions): Promise<LeafletDocument>;
//# sourceMappingURL=converter.d.ts.map