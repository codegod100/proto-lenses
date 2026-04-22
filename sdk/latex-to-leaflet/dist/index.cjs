Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const require___vite_browser_external$1 = require("./__vite-browser-external-CQXcjybA.cjs");
//#region src/types.ts
var import___vite_browser_external = require___vite_browser_external$1.require___vite_browser_external();
/** Error raised when the converter binary cannot be found or the conversion fails. */
var LatexConversionError = class extends Error {
	constructor(message, cause) {
		super(message);
		this.cause = cause;
		this.name = "LatexConversionError";
	}
};
//#endregion
//#region src/converter.ts
/**
* Converter implementation.
*
* Spawns the Rust `latex-to-leaflet-cli` binary, feeds LaTeX source via
* stdin, and parses the emitted Leaflet JSON from stdout.
*/
/** Resolved paths relative to the package root. */
var PKG_ROOT = (0, import___vite_browser_external.resolve)((0, import___vite_browser_external.fileURLToPath)({}.url), "../..");
var RELEASE_BINARY = (0, import___vite_browser_external.resolve)(PKG_ROOT, "../../..", "target/release/latex-to-leaflet-cli");
var DEBUG_BINARY = (0, import___vite_browser_external.resolve)(PKG_ROOT, "../../..", "target/debug/latex-to-leaflet-cli");
async function discoverBinary(preferred) {
	if (preferred) return preferred;
	for (const candidate of [RELEASE_BINARY, DEBUG_BINARY]) try {
		const { access } = await Promise.resolve().then(() => require("./__vite-browser-external-CQXcjybA.cjs")).then((n) => /* @__PURE__ */ require___vite_browser_external$1.__toESM(n.require___vite_browser_external(), 1));
		await access(candidate);
		return candidate;
	} catch {}
	return "latex-to-leaflet-cli";
}
/**
* Convert LaTeX source into a Leaflet document.
*
* @param source — Raw `.tex` source.
* @param options — Converter options.
* @returns Parsed Leaflet JSON document.
* @throws {@link LatexConversionError} on spawn or parse failure.
*/
async function convertLatex(source, options) {
	const binary = await discoverBinary(options?.binaryPath);
	return new Promise((resolve, reject) => {
		const child = (0, import___vite_browser_external.spawn)(binary, { stdio: [
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
			reject(new LatexConversionError(`Failed to spawn converter binary (${binary}): ${err.message}`, err));
		});
		child.on("close", (code) => {
			if (code !== 0) {
				reject(new LatexConversionError(`Converter exited with code ${code}: ${stderr.trim() || "(no stderr)"}`));
				return;
			}
			if (!stdout.trim()) {
				reject(new LatexConversionError("Converter produced empty stdout"));
				return;
			}
			try {
				resolve(JSON.parse(stdout));
			} catch (err) {
				reject(new LatexConversionError(`Failed to parse converter output as JSON: ${err instanceof Error ? err.message : String(err)}`, err instanceof Error ? err : void 0));
			}
		});
		child.stdin.write(source, "utf8", (err) => {
			if (err) {
				reject(new LatexConversionError(`Failed to write to converter stdin: ${err.message}`, err));
				return;
			}
			child.stdin.end();
		});
	});
}
//#endregion
exports.LatexConversionError = LatexConversionError;
exports.convertLatex = convertLatex;

//# sourceMappingURL=index.cjs.map