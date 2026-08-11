# Repository Snapshot SVAR Prefilter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Avoid constructing TypeScript and Svelte ASTs for production files that cannot contain a direct `@svar-ui/` import.

**Architecture:** Keep the existing repository index and SVAR boundary evaluator intact. Add one conservative raw-text prefilter immediately before the evaluator's existing AST branch; `getText()` and the parser views share `rawSourceCache`, so this removes parsing without adding filesystem reads.

**Tech Stack:** JavaScript, TypeScript, Vitest, Svelte compiler, TypeScript compiler.

## Global Constraints

- Preserve the existing order of import filtering and `APPROVED_SVAR_GRID_PATHS` validation after parsing.
- Do not add repository-index APIs or a second imports-only AST representation.
- Add exactly one regression test for the prefilter; direct TypeScript-import coverage remains outside this slice.

---

### Task 1: Prefilter the SVAR boundary scan

**Files:**
- Modify: `scripts/testing/repository-rules.mjs:42-57`
- Test: `scripts/testing/repository-rules.test.ts:305-345`
- Reference: `docs/superpowers/specs/2026-08-11-repository-snapshot-svar-prefilter-design.md`

**Interfaces:**
- Consumes: `index.getText(inputPath)`, `index.getTypeScript(inputPath)`, `index.getSvelte(inputPath)`, and `evaluateRule({ id, index })`.
- Produces: unchanged `evaluateRule()` results; files without the literal `@svar-ui/` no longer reach either AST parser from the general SVAR scan.

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

- [ ] **Step 4: Run the complete validation**

Run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.mjs
npm.cmd run check
```

Expected: the non-empty related Vitest selection passes, including the new regression and the live repository snapshot test; then `svelte-check` reports zero errors and zero warnings. This is the GREEN proof for the new test and the regression gate for the full affected surface.

- [ ] **Step 5: Inspect and commit the implementation**

Run:

```powershell
git diff -- scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts
git add scripts/testing/repository-rules.mjs scripts/testing/repository-rules.test.ts
git commit -m "test: prefilter repository SVAR boundary scan"
```

Expected: the diff contains one production prefilter and one regression test, with no repository-index changes or unrelated files; the commit succeeds. This plan document is tracked and committed separately, so it is intentionally absent from the implementation staging command.
