# Repository Snapshot SVAR Prefilter Design

## Goal

Reduce the cost of the repository snapshot test by avoiding TypeScript and Svelte AST construction for production files that cannot contain a direct `@svar-ui/` import, without weakening the existing Extractum grid wrapper boundary.

## Scope

The permanent change is limited to one text prefilter in `scripts/testing/repository-rules.mjs` and two regression cases in `scripts/testing/repository-rules.test.ts`: the no-marker invalid `.ts` skip test and an escaped forbidden Svelte-import mutation.

## Current Behavior

`evaluateExtractumGridWrapperBoundary()` enumerates every non-test TypeScript, JavaScript, JSX, TSX, and Svelte file below `src/`. It builds a full TypeScript or Svelte fact graph for each file and then inspects only its import facts for sources beginning with `@svar-ui/`.

The current repository contains 379 matching production files. The conservative candidate set contains 22 files with either the literal text `@svar-ui/` or any backslash. The two Svelte wrappers are also subject to separate strict AST validation later in the same evaluator.

## Design

Immediately before the existing TypeScript-or-Svelte parse, the general forbidden-import scan will read the file through `index.getText(file)` and continue only when the immutable cached source contains neither the exact literal `@svar-ui/` nor a backslash. Files containing either the literal marker or any backslash reach the existing parser. The remaining parse, import filtering, and `APPROVED_SVAR_GRID_PATHS` check stay in their current order.

This adds no filesystem reads: `getText()` and the later `getTypeScript()` or `getSvelte()` call share the repository index's `rawSourceCache`, and the general scan already caused every matching file to be read before this change. The prefilter removes AST construction only.

The prefilter must not narrow the coverage of the existing rule. A backslash is the only channel for spelling module-specifier characters non-literally, including line continuations, so parsing every file that contains one preserves escaped static-import coverage. Comments, string literals, ambient module declarations, or unrelated escapes may cause an unnecessary parse, but the AST import check prevents false violations.

The current rule recognizes static `ImportDeclaration` facts only. Export-from declarations and dynamic imports are existing coverage limitations outside this optimization.

The existing strict wrapper checks remain unchanged. `DataGrid.svelte` and `TreeDataGrid.svelte` continue to be parsed and checked for required component imports, component composition, tree mode, and scoped SVAR cell styling. Their later reads reuse the repository index cache. Approved files still pass through the general scan, preserving its current semantics.

## Tests

Add two regression cases. The evaluator test named `skips production files without the @svar-ui/ marker` contains a syntactically invalid `.ts` production source without the marker. A fixture comment states that the invalid syntax detects whether parsing occurred and is not a supported repository state. Before the change, the general scan tries to parse that source through the TypeScript `parseDiagnostics` path and returns an `INFRA_ERROR`; after the change, the source is skipped and the rule returns no violations. This tests the public rule behavior without spying on repository-index internals.

The escaped forbidden Svelte-import mutation uses `String.raw` to place a real backslash in a static import source. It proves that a non-literal spelling of `@svar-ui/` still reaches the parser and produces the existing semantic direct-import violation. The existing literal forbidden Svelte import mutation continues to prove that a file containing a literal marker is parsed and rejected. Existing wrapper fixtures continue to prove strict validation of approved files.

Export-from and dynamic-import coverage remain outside this optimization slice.

## Validation

After the RED/GREEN cycle, run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.mjs
```

After the related test selection passes, run:

```powershell
npm.cmd run check
```

Because this change affects only frontend test infrastructure and no Rust code, Rust verification loops and `npm.cmd run verify` are outside this slice.

## Non-Goals

- Adding `getImports()` or `hasImportPrefix()` to the repository index.
- Introducing a second imports-only AST representation.
