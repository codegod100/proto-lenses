/**
 * Core types for the LaTeX → Leaflet converter.
 */
/** A single block inside a Leaflet page. */
export interface LeafletBlock {
    $type: string;
    block: Record<string, unknown> & {
        $type: string;
    };
}
/** A Leaflet page. */
export interface LeafletPage {
    $type: string;
    blocks: LeafletBlock[];
    id: string;
}
/** The emitted Leaflet document. */
export interface LeafletDocument {
    $type: 'site.standard.document';
    title?: string;
    content: {
        $type: 'pub.leaflet.content';
        pages: LeafletPage[];
    };
}
/** Options for {@link convertLatex}. */
export interface ConvertOptions {
    /**
     * Path to the `latex-to-leaflet-cli` binary.
     *
     * If omitted the SDK searches:
     * 1. `../../target/release/latex-to-leaflet-cli`
     * 2. `../../target/debug/latex-to-leaflet-cli`
     * 3. `latex-to-leaflet-cli` on `$PATH`
     */
    binaryPath?: string;
}
/** Error raised when the converter binary cannot be found or the conversion fails. */
export declare class LatexConversionError extends Error {
    readonly cause?: Error | undefined;
    constructor(message: string, cause?: Error | undefined);
}
//# sourceMappingURL=types.d.ts.map