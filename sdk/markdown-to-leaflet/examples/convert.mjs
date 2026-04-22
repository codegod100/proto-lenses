#!/usr/bin/env node
/**
 * Example: Convert a Markdown fixture to Leaflet JSON.
 *
 * Usage (from sdk/markdown-to-leaflet):
 *   node examples/convert.mjs
 *
 * Prerequisites:
 *   cargo build -p markdown-to-leaflet
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { convertMarkdown } from '../dist/index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));

const FIXTURE = resolve(__dirname, 'fixtures', 'sample.md');
const OUTPUT = resolve(__dirname, 'fixtures', 'sample.leaflet.json');

const BINARY = resolve(__dirname, '../../../target/debug/markdown-to-leaflet-cli');

async function main() {
  const source = readFileSync(FIXTURE, 'utf-8');
  console.error(`Converting ${FIXTURE} ...`);

  const doc = await convertMarkdown(source, { binaryPath: BINARY });

  const json = JSON.stringify(doc, null, 2);
  writeFileSync(OUTPUT, json + '\n', 'utf-8');

  console.error(`Wrote Leaflet JSON to ${OUTPUT}`);
  console.log(json);
}

main().catch((err) => {
  console.error('Conversion failed:', err.message);
  process.exit(1);
});
