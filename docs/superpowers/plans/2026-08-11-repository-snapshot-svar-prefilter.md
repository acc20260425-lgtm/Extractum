# Repository Snapshot SVAR Prefilter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Avoid constructing TypeScript and Svelte ASTs for production files that cannot contain a direct `@svar-ui/` import.

**Architecture:** Keep the existing repository index and SVAR boundary evaluator intact. Add one conservative raw-text prefilter immediately before the evaluator's existing AST branch: files containing either the literal `@svar-ui/` marker or any backslash reach the existing parser. `getText()` and the parser views share `rawSourceCache`, so this removes parsing without adding filesystem reads or narrowing existing static-import coverage.

**Tech Stack:** JavaScript, TypeScript, Vitest, Svelte compiler, TypeScript compiler.

## Global Constraints

- Preserve the existing order of import filtering and `APPROVED_SVAR_GRID_PATHS` validation after parsing.
- Do not add repository-index APIs or a second imports-only AST representation.
- Add exactly two regression cases for the prefilter: the no-marker skip test and an escaped forbidden Svelte-import mutation that must produce a semantic violation.

---

### Task 1: Prefilter the SVAR boundary scan

**Files:**
- Modify: `scripts/testing/repository-rules.mjs:48-49`
- Test: `scripts/testing/repository-rules.test.ts:305-345`
- Reference: `docs/superpowers/specs/2026-08-11-repository-snapshot-svar-prefilter-design.md`

**Interfaces:**
- Consumes: `index.getText(inputPath)`, `index.getTypeScript(inputPath)`, `index.getSvelte(inputPath)`, and `evaluateRule({ id, index })`.
- Produces: unchanged `evaluateRule()` results; files containing either the literal `@svar-ui/` marker or any backslash reach the existing parser, while files containing neither skip AST construction in the general SVAR scan.

- [ ] **Step 1: Write the failing prefilter regression test**

Add this test inside `describe("repository rule registry", ...)` in `scripts/testing/repository-rules.test.ts`, after the current-snapshot test:

```ts
  it("skips production files without the @svar-ui/ marker", () => {
    const index = indexFor({
      ...extractumGridBoundarySources,
      // Invalid syntax is a parse detector, not a supported repository state.
      "src/lib/unrelated-without-marker.ts": "export const = ;",
    });

    expect(evaluateRule({
      id: "rule:extractum-grid-wrapper-boundary",
      index,
    })).toEqual({
      id: "rule:extractum-grid-wrapper-boundary",
      violations: [],
    });
  });
```

- [ ] **Step 2: Run the new test and verify RED**

Run:

```powershell
npm.cmd exec vitest -- run scripts/testing/repository-rules.test.ts -t "skips production files without the @svar-ui/ marker" --reporter=dot
```

Expected: FAIL because the actual result contains an `INFRA_ERROR` qualified with `src/lib/unrelated-without-marker.ts`. This confirms that the invalid syntax detects the current unwanted parse.

- [ ] **Step 3: Add the minimal text prefilter**

In `evaluateExtractumGridWrapperBoundary()` within `scripts/testing/repository-rules.mjs`, change only the start of the production-file loop to:

```js
  for (const file of productionFiles) {
    if (!index.getText(file).includes("@svar-ui/")) continue;
    const facts = file.endsWith(".svelte") ? index.getSvelte(file) : index.getTypeScript(file);
```

Leave the subsequent import filtering, approved-path check, strict wrapper validation, and repository-index implementation unchanged.

Run the focused no-marker test from Step 2 again. Expected: PASS; the invalid source is skipped without reaching the TypeScript parser.

- [ ] **Step 4: Add the escaped-import mutation and verify the second RED**

Add this mutation to the existing `rule:extractum-grid-wrapper-boundary` fixture:

```ts
      "imports escaped SVAR from a feature component": {
        ...extractumGridBoundarySources,
        "src/lib/components/research-projects/FeatureGrid.svelte": String.raw`
          <script lang="ts">import { Grid } from "\x40svar-ui/svelte-grid";</script>
          <Grid />
        `,
      },
```

Run:

```powershell
npm.cmd exec vitest -- run scripts/testing/repository-rules.test.ts -t "gives every registered evaluator its own positive fixture and violating mutation" --reporter=dot
```

Expected: FAIL because the escaped mutation returns no violations. This confirms that the literal-only prefilter narrows the existing static-import coverage.

- [ ] **Step 5: Add the conservative backslash fallback and verify GREEN**

Replace the literal-only prefilter with:

```js
  for (const file of productionFiles) {
    const source = index.getText(file);
    // A backslash is the only way to spell module-specifier characters non-literally,
    // including line continuations, so always parse those files.
    if (!source.includes("@svar-ui/") && !source.includes("\\")) continue;
    const facts = file.endsWith(".svelte") ? index.getSvelte(file) : index.getTypeScript(file);
```

Keep import filtering and `APPROVED_SVAR_GRID_PATHS` validation in their existing order after parsing. Do not change `repository-index.mjs` or expand import-fact coverage.

Run the focused mutation command from Step 4 again. Expected: PASS; the escaped static import produces the existing semantic direct-import violation.

- [ ] **Step 6: Run the complete validation**

Run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.mjs
```

Expected: the non-empty related Vitest selection passes, including the no-marker regression, the live repository snapshot test, and both the literal and escaped forbidden Svelte import mutations. The mutations prove that files containing a marker or a backslash are parsed and rejected semantically.

Only after the related test selection passes, run:

```powershell
npm.cmd run check
```

Expected: `svelte-check` reports zero errors and zero warnings.

- [ ] **Step 7: Inspect and commit the implementation**

Run:

```powershell
git diff -- scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts docs/superpowers/specs/2026-08-11-repository-snapshot-svar-prefilter-design.md docs/superpowers/plans/2026-08-11-repository-snapshot-svar-prefilter.md
git add scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts docs/superpowers/specs/2026-08-11-repository-snapshot-svar-prefilter-design.md docs/superpowers/plans/2026-08-11-repository-snapshot-svar-prefilter.md
git commit -m "fix: preserve escaped SVAR import coverage"
```

Expected: the four-file diff contains the conservative production prefilter, both regression cases, and corrected design and implementation documentation, with no repository-index changes or unrelated files; the commit succeeds.
