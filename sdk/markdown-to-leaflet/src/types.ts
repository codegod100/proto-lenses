/**
 * Shared types for the @nandithebull/markdown-to-leaflet SDK.
 */

/**
 * A Leaflet document returned by the converter.
 */
export interface LeafletDocument {
  $type: 'site.standard.document';
  title?: string;
  description?: string;
  content: {
    $type: 'pub.leaflet.content';
    pages: Array<{
      $type: 'pub.leaflet.pages.linearDocument';
      id: string;
      blocks: Array<{
        $type: 'pub.leaflet.pages.linearDocument#block';
        block: Record<string, unknown>;
      }>;
    }>;
  };
}

/** Error raised when the converter binary cannot be found or the conversion fails. */
export class MarkdownConversionError extends Error {
  constructor(
    message: string,
    /** Original error, if any. */
    public readonly cause?: Error,
  ) {
    super(message);
    this.name = 'MarkdownConversionError';
  }
}
