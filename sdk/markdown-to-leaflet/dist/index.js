import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
//#region src/types.ts
/** Error raised when the converter binary cannot be found or the conversion fails. */
var MarkdownConversionError = class extends Error {
	constructor(message, cause) {
		super(message);
		this.cause = cause;
		this.name = "MarkdownConversionError";
	}
};
//#endregion
//#region src/converter.ts
/**
* Converter implementation.
*
* Spawns the Rust `markdown-to-leaflet-cli` binary, feeds Markdown source via
* stdin, and parses the emitted Leaflet JSON from stdout.
*/
/** Resolved paths relative to the package root. */
var PKG_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
var RELEASE_BINARY = resolve(PKG_ROOT, "../../..", "target/release/markdown-to-leaflet-cli");
var DEBUG_BINARY = resolve(PKG_ROOT, "../../..", "target/debug/markdown-to-leaflet-cli");
async function discoverBinary(preferred) {
	if (preferred) return preferred;
	for (const candidate of [RELEASE_BINARY, DEBUG_BINARY]) try {
		const { access } = await import("node:fs/promises");
		await access(candidate);
		return candidate;
	} catch {}
	return "markdown-to-leaflet-cli";
}
/**
* Convert Markdown source into a Leaflet document.
*
* @param source — Raw Markdown source.
* @param options — Converter options.
* @returns Parsed Leaflet JSON document.
* @throws {@link MarkdownConversionError} on spawn or parse failure.
*/
async function convertMarkdown(source, options) {
	const binary = await discoverBinary(options?.binaryPath);
	return new Promise((resolve, reject) => {
		const child = spawn(binary, { stdio: [
			"pipe",
			"pipe",
			"pipe"
		] });
		let stdout = "";
		let stderr = "";
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk) => {
			stdout += chunk;
		});
		child.stderr.on("data", (chunk) => {
			stderr += chunk;
		});
		child.on("error", (err) => {
			reject(new MarkdownConversionError(`Failed to spawn converter binary (${binary}): ${err.message}`, err));
		});
		child.on("close", (code) => {
			if (code !== 0) {
				reject(new MarkdownConversionError(`Converter exited with code ${code}: ${stderr.trim() || "(no stderr)"}`));
				return;
			}
			if (!stdout.trim()) {
				reject(new MarkdownConversionError("Converter produced empty stdout"));
				return;
			}
			try {
				resolve(JSON.parse(stdout));
			} catch (err) {
				reject(new MarkdownConversionError(`Failed to parse converter output as JSON: ${err instanceof Error ? err.message : String(err)}`, err instanceof Error ? err : void 0));
			}
		});
		child.stdin.write(source, "utf8", (err) => {
			if (err) {
				reject(new MarkdownConversionError(`Failed to write to converter stdin: ${err.message}`, err));
				return;
			}
			child.stdin.end();
		});
	});
}
//#endregion
export { MarkdownConversionError, convertMarkdown };

//# sourceMappingURL=index.js.map