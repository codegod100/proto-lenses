/**
 * Shared types for the @nandithebull/markdown-to-leaflet SDK.
 */
/** Error raised when the converter binary cannot be found or the conversion fails. */
export class MarkdownConversionError extends Error {
    cause;
    constructor(message, 
    /** Original error, if any. */
    cause) {
        super(message);
        this.cause = cause;
        this.name = 'MarkdownConversionError';
    }
}
//# sourceMappingURL=types.js.map