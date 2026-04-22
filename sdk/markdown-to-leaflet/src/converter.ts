/**
 * Converter implementation.
 *
 * Spawns the Rust `markdown-to-leaflet-cli` binary, feeds Markdown source via
 * stdin, and parses the emitted Leaflet JSON from stdout.
 */

import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  type ConvertOptions,
  type LeafletDocument,
  MarkdownConversionError,
} from './types.js';

/** Resolved paths relative to the package root. */
const PKG_ROOT = resolve(fileURLToPath(import.meta.url), '../..');

const RELEASE_BINARY = resolve(PKG_ROOT, '../../..', 'target/release/markdown-to-leaflet-cli');
const DEBUG_BINARY = resolve(PKG_ROOT, '../../..', 'target/debug/markdown-to-leaflet-cli');

async function discoverBinary(preferred?: string): Promise<string> {
  if (preferred) return preferred;

  for (const candidate of [RELEASE_BINARY, DEBUG_BINARY]) {
    try {
      const { access } = await import('node:fs/promises');
      await access(candidate);
      return candidate;
    } catch {
      // Not found — try next candidate.
    }
  }

  return 'markdown-to-leaflet-cli';
}

/**
 * Convert Markdown source into a Leaflet document.
 *
 * @param source — Raw Markdown source.
 * @param options — Converter options.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link MarkdownConversionError} on spawn or parse failure.
 */
export async function convertMarkdown(
  source: string,
  options?: ConvertOptions,
): Promise<LeafletDocument> {
  const binary = await discoverBinary(options?.binaryPath);

  return new Promise((resolve, reject) => {
    const child = spawn(binary, { stdio: ['pipe', 'pipe', 'pipe'] });

    let stdout = '';
    let stderr = '';

    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');

    child.stdout.on('data', (chunk: string) => {
      stdout += chunk;
    });

    child.stderr.on('data', (chunk: string) => {
      stderr += chunk;
    });

    child.on('error', (err: Error) => {
      reject(
        new MarkdownConversionError(
          `Failed to spawn converter binary (${binary}): ${err.message}`,
          err,
        ),
      );
    });

    child.on('close', (code: number | null) => {
      if (code !== 0) {
        reject(
          new MarkdownConversionError(
            `Converter exited with code ${code}: ${stderr.trim() || '(no stderr)'}`,
          ),
        );
        return;
      }

      if (!stdout.trim()) {
        reject(new MarkdownConversionError('Converter produced empty stdout'));
        return;
      }

      try {
        const parsed = JSON.parse(stdout) as LeafletDocument;
        resolve(parsed);
      } catch (err) {
        reject(
          new MarkdownConversionError(
            `Failed to parse converter output as JSON: ${err instanceof Error ? err.message : String(err)}`,
            err instanceof Error ? err : undefined,
          ),
        );
      }
    });

    child.stdin.write(source, 'utf8', (err?: Error | null) => {
      if (err) {
        reject(
          new MarkdownConversionError(
            `Failed to write to converter stdin: ${err.message}`,
            err,
          ),
        );
        return;
      }
      child.stdin.end();
    });
  });
}
