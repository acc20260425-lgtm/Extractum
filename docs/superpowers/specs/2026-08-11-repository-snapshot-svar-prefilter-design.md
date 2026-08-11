# Repository Snapshot SVAR Prefilter Design

## Goal

Reduce the cost of the repository snapshot test by avoiding TypeScript and Svelte AST construction for production files that cannot contain a direct `@svar-ui/` import, without weakening the existing Extractum grid wrapper boundary.

## Scope

The permanent change is limited to one text prefilter in `scripts/testing/repository-rules.mjs` and one regression test in `scripts/testing/repository-rules.test.ts`. Timing instrumentation is used only for local before-and-after measurements and is not committed.

## Current Behavior

`evaluateExtractumGridWrapperBoundary()` enumerates every non-test TypeScript, JavaScript, JSX, TSX, and Svelte file below `src/`. It builds a full TypeScript or Svelte fact graph for each file and then inspects only its import facts for sources beginning with `@svar-ui/`.

The current repository contains 379 matching production files. Only five contain the literal text `@svar-ui/`: the three approved grid implementation files, one ambient module declaration file, and one source comment. The two Svelte wrappers are also subject to separate strict AST validation later in the same evaluator.

## Design

Immediately before the existing TypeScript-or-Svelte parse, the general forbidden-import scan will read the file through `index.getText(file)` and continue when the immutable cached source does not include the exact literal `@svar-ui/`. The remaining parse, import filtering, and `APPROVED_SVAR_GRID_PATHS` check stay in their current order.

This is a conservative prefilter. Every matching import must contain the literal prefix in its source text, so the prefilter cannot hide a forbidden import. Comments, string literals, or ambient module declarations may cause an unnecessary parse, but the AST import check prevents false violations.

The existing strict wrapper checks remain unchanged. `DataGrid.svelte` and `TreeDataGrid.svelte` continue to be parsed and checked for required component imports, component composition, tree mode, and scoped SVAR cell styling. Their later reads reuse the repository index cache. Approved files still pass through the general scan, preserving its current semantics.

## Tests

Add one evaluator regression test whose otherwise valid fixture contains a syntactically invalid production source without `@svar-ui/`. Before the change, the general scan tries to parse that source and returns an `INFRA_ERROR`; after the change, the source is skipped and the rule returns no violations. This tests the public rule behavior without spying on repository-index internals.

The existing forbidden Svelte import mutation continues to prove that a file containing a real marker is parsed and rejected. Existing wrapper fixtures continue to prove strict validation of approved files.

Direct TypeScript-import coverage is a separate existing coverage gap and is not part of this optimization slice.

## Validation

After the RED/GREEN cycle, run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.mjs
npm.cmd run check
```

Measure the focused test locally before and after with the same command, but do not commit timing instrumentation or enforce an environment-dependent threshold. Because this change affects only frontend test infrastructure and no Rust code, Rust verification loops and `npm.cmd run verify` are outside this slice.

## Non-Goals

- Adding `getImports()` or `hasImportPrefix()` to the repository index.
- Introducing a second imports-only AST representation.
