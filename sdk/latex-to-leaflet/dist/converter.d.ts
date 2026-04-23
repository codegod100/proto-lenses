/**
 * Converter implementation.
 *
 * Calls the native Rust addon directly (via napi-rs).
 */
import { type LeafletDocument } from './types.js';
/**
 * Convert LaTeX source into a Leaflet document.
 *
 * @param source — Raw `.tex` source.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link LatexConversionError} on parse or lens failure.
 */
export declare function convertLatex(source: string): Promise<LeafletDocument>;
//# sourceMappingURL=converter.d.ts.map