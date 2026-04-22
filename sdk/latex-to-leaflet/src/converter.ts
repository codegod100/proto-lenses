/**
 * Converter implementation.
 *
 * Spawns the Rust `latex-to-leaflet-cli` binary, feeds LaTeX source via
 * stdin, and parses the emitted Leaflet JSON from stdout.
 */

import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  type ConvertOptions,
  type LeafletDocument,
  LatexConversionError,
} from './types.js';

/** Resolved paths relative to the package root. */
const PKG_ROOT = resolve(fileURLToPath(import.meta.url), '../..');

const RELEASE_BINARY = resolve(PKG_ROOT, '../../..', 'target/release/latex-to-leaflet-cli');
const DEBUG_BINARY = resolve(PKG_ROOT, '../../..', 'target/debug/latex-to-leaflet-cli');

async function discoverBinary(preferred?: string): Promise<string> {
  if (preferred) return preferred;

  // Check release binary first, then debug.
  for (const candidate of [RELEASE_BINARY, DEBUG_BINARY]) {
    try {
      const { access } = await import('node:fs/promises');
      await access(candidate);
      return candidate;
    } catch {
      // Not found — try next candidate.
    }
  }

  // Fall back to PATH.
  return 'latex-to-leaflet-cli';
}

/**
 * Convert LaTeX source into a Leaflet document.
 *
 * @param source — Raw `.tex` source.
 * @param options — Converter options.
 * @returns Parsed Leaflet JSON document.
 * @throws {@link LatexConversionError} on spawn or parse failure.
 */
export async function convertLatex(
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
        new LatexConversionError(
          `Failed to spawn converter binary (${binary}): ${err.message}`,
          err,
        ),
      );
    });

    child.on('close', (code: number | null) => {
      if (code !== 0) {
        reject(
          new LatexConversionError(
            `Converter exited with code ${code}: ${stderr.trim() || '(no stderr)'}`,
          ),
        );
        return;
      }

      if (!stdout.trim()) {
        reject(new LatexConversionError('Converter produced empty stdout'));
        return;
      }

      try {
        const parsed = JSON.parse(stdout) as LeafletDocument;
        resolve(parsed);
      } catch (err) {
        reject(
          new LatexConversionError(
            `Failed to parse converter output as JSON: ${err instanceof Error ? err.message : String(err)}`,
            err instanceof Error ? err : undefined,
          ),
        );
      }
    });

    child.stdin.write(source, 'utf8', (err?: Error | null) => {
      if (err) {
        reject(
          new LatexConversionError(
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
