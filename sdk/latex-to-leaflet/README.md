# @nandithebull/latex-to-leaflet

TypeScript SDK for converting LaTeX documents to Leaflet.pub JSON.

Powered by a native Node.js addon (napi-rs) — no external binaries required.

## Quick start

```typescript
import { convertLatex } from '@nandithebull/latex-to-leaflet';

const json = await convertLatex('\\documentclass{article}\n\\begin{document}\nHello\n\\end{document}');
console.log(json.title);
```

## License

MIT
