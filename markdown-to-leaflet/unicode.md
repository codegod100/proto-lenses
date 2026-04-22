# Agent Instructions: Make `mathblog` Unicode-First for Inline Math

## Context

We examined `@nandithebull/markdown-to-leaflet@0.30.0` (the Rust CLI that powers the canonical converter). It puts **ALL** math — inline `$...$` and display `$$...$$` — into `pub.leaflet.blocks.math` blocks with raw `tex`. We are deliberately diverging from that behavior: **inline `$...$` math stays in text blocks but is converted to Unicode** (so it renders legibly in Leaflet's plaintext paragraphs), while **display `$$...$$` math stays as `pub.leaflet.blocks.math`** KaTeX blocks.

Your task is to make this Unicode conversion **thorough, well-tested, and robust** across the entire pipeline.

---

## Scope of Changes

### 1. `src/latex-to-unicode.ts` — Expand & Harden

- **Add more symbol mappings** — coverage gaps:
  - **More letters**: add missing uppercase superscripts (C, F, S, Y, Z) if they exist; document clearly when a character has NO Unicode superscript/subscript (e.g. `q`).
  - **Additional commands**: `\aleph`, `\hbar`, `\ell`, `\Re`, `\Im`, `\wp`, `\mho`, `\nabla` (already there?), `\partial` (already there?), `\varphi`, `\varepsilon`, `\vartheta`, `\varsigma`, `\varkappa`.
  - **More arrows**: `\longrightarrow`, `\longleftarrow`, `\Longrightarrow`, `\Longleftarrow`, `\hookrightarrow`, `\hookleftarrow`, `\rightharpoonup`, `\rightharpoondown`.
  - **More operators**: `\circ`, `\bullet`, `\star`, `\dagger`, `\ddagger`, `\amalg`, `\setminus`, `\wr`, `\cdotp`, `\times` (already there).
  - **Delimiters**: `\langle` → `⟨`, `\rangle` → `⟩`, `\mid` → `|`, `\nmid` → `∤`.
  - **Accents**: `\hat{x}` → `x̂`, `\bar{x}` → `x̄`, `\tilde{x}` → `x̃`, `\vec{x}` → `x⃗`, `\dot{x}` → `ẋ`, `\ddot{x}` → `ẍ`. Implement a simple accent map that applies the combining character to the argument.
  - **Stretchy ops**: `\overline{AB}` can just become `AB̄` (accent on last char), `\underline{AB}` → `AB̲`.
  - **Better `\frac` handling**: when numerator/denominator are single digits or simple expressions, try Unicode fraction characters if available (e.g. `½`, `⅓`, `¼`, `⅕`, `⅙`, `⅛`, `⅔`, `¾`, `⅖`, `⅗`, `⅘`, `⅚`, `⅜`, `⅝`, `⅞`). Fallback to `num⁄den` (already done).
  - **Better `\sqrt`**: handle `\sqrt[n]{x}` → `ⁿ√x`.
- **Add defensive parsing**:
  - When `\frac` argument parsing fails (mismatched braces), return the original expression instead of corrupting it.
  - When a command has no mapping, keep the original `\command` text (current behavior) but log a warning somehow if possible (or at least document this behavior).
- **Header comment** explaining the design decision: why Unicode instead of `math` blocks for inline math.

### 2. `src/latex-to-unicode.test.ts` — NEW Test File

Create comprehensive tests:

```ts
import { describe, it, expect } from 'vitest'
import { texToUnicode } from './latex-to-unicode'

describe('texToUnicode', () => {
  it('converts Greek commands', () => { ... })
  it('converts superscripts', () => { expect(texToUnicode('$x^2$')).toBe('x²') })
  it('converts subscripts', () => { expect(texToUnicode('$x_i$')).toBe('xᵢ') })
  it('converts operators', () => { expect(texToUnicode('$\otimes$')).toBe('⊗') })
  it('converts arrows', () => { expect(texToUnicode('$\rightarrow$')).toBe('→') })
  it('converts relations', () => { expect(texToUnicode('$\leq$')).toBe('≤') })
  it('converts fractions', () => { expect(texToUnicode('$\frac{a}{b}$')).toContain('⁄') })
  it('converts sqrt', () => { expect(texToUnicode('$\sqrt{x}$')).toBe('√x') })
  it('converts mathbb', () => { expect(texToUnicode('$\mathbb{R}$')).toBe('ℝ') })
  it('converts accents', () => { expect(texToUnicode('$\hat{x}$')).toBe('x̂') })
  it('handles nested braces', () => { expect(texToUnicode('$x^{2+3}$')).toBe('x²⁺³') })
  it('gracefully handles unknown commands', () => { expect(texToUnicode('$\foo$')).toBe('\foo') })
  it('strips $ wrappers', () => { expect(texToUnicode('$...$')).toBe('...') })
  it('strips $$ wrappers', () => { expect(texToUnicode('$$...$$')).toBe('...') })
})
```

Aim for **≥20 tests** covering every major category.

### 3. `src/panproto/markdown-instance.ts` — Harden Forward Path

- The inline-math regex is:

  ```ts
  const INLINE_MATH_RE = /(?<!\$)\$([^$\n]+?)\$(?!\$)/g
  ```

  This is mostly fine but add a comment explaining it **intentionally does NOT match `$$...$$`** (that is handled by `DISPLAY_MATH_RE`).
- Ensure `convertInlineMath()` is called on **all** text-bearing blocks:
  - `heading` segments ✅
  - `paragraph` segments ✅
  - `blockquote` segments ✅
  - `listItem` segments ✅ (verify it's happening via `listItemContent` → `parseInlineSegments`)
- Verify `table` cell plaintext does NOT try to convert math (table cells should remain raw markdown for now, since table→KaTeX grid conversion is separate).
- Add one more test in `markdown-instance.test.ts`: inline math inside a **heading** should convert to Unicode. E.g.:

  ```ts
  it('should convert inline math in headings', () => {
    const doc = parseMarkdownToMdDocument(draft('# Section $\\alpha$'))
    expect(doc.blocks[0].$type).toBe('heading')
    if (doc.blocks[0].$type === 'heading') {
      const text = doc.blocks[0].segments.map(s => s.text).join('')
      expect(text).toContain('α')
      expect(text).not.toContain('$')
    }
  })
  ```

### 4. `src/panproto/lens-b.ts` — Harden Reverse Path

- `convertInlineMathInSegments()` should use the **exact same regex** as `markdown-instance.ts`. Consider extracting `INLINE_MATH_RE` to a shared constant file (e.g. `src/panproto/constants.ts`) so both paths use the same pattern. If you create `src/panproto/constants.ts`, put this in it:

  ```ts
  export const DISPLAY_MATH_RE = /^\s*\$\$\s*([\s\S]+?)\s*\$\$\s*$/
  export const INLINE_MATH_RE = /(?<!\$)\$([^$\n]+?)\$(?!\$)/g
  ```

  Then import it in both `markdown-instance.ts` and `lens-b.ts`.
- Ensure `splitTextBlockIntoArticleBlocks()` runs **before** `convertInlineMathInSegments()` so that display `$$...$$` math is extracted into separate `math` blocks and does NOT get passed to the Unicode converter.

### 5. `src/panproto/lens-b.test.ts` — Expand Unicode Tests

- Add a test specifically verifying that after re-import from the `RECORD` fixture, text blocks with `$n^r$` become `nʳ` (Unicode) and `$n^l$` become `nˡ` (Unicode).
- Add a test verifying that `$v^i$` in the `RECORD` becomes `vⁱ`.
- Add a test verifying that a text block containing ONLY `$Server$` (from the `RECORD`) becomes `Server` — no `$` wrappers, no math block splitting.

### 6. `src/panproto/lens-a.ts` — (read to verify, likely no changes needed)

Verify that `mdDocumentToArticle()` doesn't interfere with math. It should:
- Pass `math` blocks through unchanged.
- Copy `paragraph` segments as-is (the Unicode conversion happened upstream in `markdown-instance.ts`).

### 7. `src/commands/publish.ts` — Verify Integration

- No structural changes needed here, but trace through the pipeline mentally to confirm:
  1. `parseMarkdownToMdDocument(draft)` converts `$...$` to Unicode in segments.
  2. `mdDocumentToArticle()` preserves those Unicode segments.
  3. `articleToLexicon()` emits `pub.leaflet.blocks.text` with the Unicode segments as `plaintext`.
- If there are any issues, fix them.

---

## Acceptance Criteria

- `npm test` passes with zero failures.
- At least **20 new tests** added across `latex-to-unicode.test.ts`, `markdown-instance.test.ts`, and `lens-b.test.ts`.
- Any inline `$...$` in test input should NEVER appear in the final `plaintext` output — only Unicode.
- Display `$$...$$` should NEVER be passed through `texToUnicode` — it should always become a separate `pub.leaflet.blocks.math` block.
- The codebase should have consistent regex patterns for math delimiters (extracted to shared constants if duplicated).

## What NOT to Change

- Do NOT switch inline `$...$` to `pub.leaflet.blocks.math` blocks. We are deliberately keeping the Unicode divergence.
- Do NOT modify the Obsidian preview (`src/views/preview-view.ts`) or auth code.
- Do NOT modify the manifest version.

Reply with a summary of the changes you made and the final test count.
