/**
 * Shared types for the @nandithebull/latex-to-leaflet SDK.
 */
/** Error raised when the converter addon cannot be found or the conversion fails. */
export class LatexConversionError extends Error {
    cause;
    constructor(message, 
    /** Original error, if any. */
    cause) {
        super(message);
        this.cause = cause;
        this.name = 'LatexConversionError';
    }
}
//# sourceMappingURL=types.js.map