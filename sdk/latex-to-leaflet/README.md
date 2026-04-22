# @panproto/latex-to-leaflet

TypeScript SDK for converting LaTeX documents to Leaflet.pub JSON.

## Quick start

```typescript
import { convertLatex } from '@panproto/latex-to-leaflet';

const json = await convertLatex('\\documentclass{article}\n\\begin{document}\nHello\\n\\end{document}');
console.log(json.title);
```

## Requirements

You must have the `latex-to-leaflet-cli` binary available. By default the SDK
looks in the default Cargo target directories relative to the package root.
You can also point it to a custom binary via `binaryPath`.

```typescript
const json = await convertLatex(source, { binaryPath: '/path/to/latex-to-leaflet-cli' });
```

## Building the Rust binary

From the workspace root:

```bash
cargo build -p latex-to-leaflet --release
```

## License

MIT
