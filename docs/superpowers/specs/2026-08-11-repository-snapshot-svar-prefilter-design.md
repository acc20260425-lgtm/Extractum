# Repository Snapshot SVAR Prefilter Design

## Goal

Reduce the cost of the repository snapshot test by avoiding TypeScript and Svelte AST construction for production files that cannot contain a direct `@svar-ui/` import, without weakening the existing Extractum grid wrapper boundary.

## Scope

The permanent change is limited to:

- `scripts/testing/repository-rules.mjs`;
- regression coverage in `scripts/testing/repository-rules.test.ts`.

Timing instrumentation is used only for local before-and-after measurements and is not committed. The repository index API, Cargo metadata loading, recursive freezing, worker threads, and cross-run caches remain unchanged.

## Current Behavior

`evaluateExtractumGridWrapperBoundary()` enumerates every non-test TypeScript, JavaScript, JSX, TSX, and Svelte file below `src/`. It builds a full TypeScript or Svelte fact graph for each file and then inspects only its import facts for sources beginning with `@svar-ui/`.

The current repository contains 379 matching production files. Only five contain the literal text `@svar-ui/`: the three approved grid implementation files, one ambient module declaration file, and one source comment. The two Svelte wrappers are also subject to separate strict AST validation later in the same evaluator.

## Design

The general forbidden-import scan will apply checks in this order:

1. Skip files in `APPROVED_SVAR_GRID_PATHS`, because their permitted imports and wrapper composition are validated separately.
2. Read the file through `index.getText(file)`, preserving the repository index's immutable raw-source snapshot and cache.
3. Skip AST construction when the source does not include the exact literal `@svar-ui/`.
4. Parse remaining candidates through the existing `index.getSvelte()` or `index.getTypeScript()` path.
5. Report the existing violation when an actual import source begins with `@svar-ui/`.

This is a conservative prefilter. Every matching import must contain the literal prefix in its source text, so the prefilter cannot hide a forbidden import. Comments, string literals, or ambient module declarations may cause an unnecessary parse, but the AST import check prevents false violations.

The existing strict wrapper checks remain unchanged. `DataGrid.svelte` and `TreeDataGrid.svelte` continue to be parsed and checked for required component imports, component composition, tree mode, and scoped SVAR cell styling. `data-grid-date-format.ts` remains an approved implementation file and is excluded from the forbidden-import scan.

## Tests

Regression coverage will prove both performance behavior and boundary semantics:

- a focused evaluator fixture will spy on TypeScript and Svelte parser entry points and verify that unrelated production files without the marker are not parsed;
- the existing forbidden Svelte import mutation will continue to produce a semantic violation;
- a forbidden TypeScript import mutation will be added so both parser branches remain covered;
- the existing positive wrapper fixture and wrapper mutations will continue to prove strict validation of approved files.

The parser-count assertion will describe only the stable contract: unrelated files without `@svar-ui/` are not parsed. It will not assert elapsed wall-clock time, which would be environment-dependent.

## Local Measurement

Run the focused repository rule test before and after the implementation using the same checkout and command. Record Vitest's test duration and the outer command duration locally. No timing logs, environment flags, or instrumentation hooks will remain in committed files.

The expected structural improvement is more important than a brittle timing threshold: the general scan should reduce AST candidates from all 379 matching production files to only non-approved files whose text contains `@svar-ui/`, while the two approved Svelte wrappers retain their mandatory strict parses.

## Validation

After the RED/GREEN cycle, run:

```powershell
npm.cmd run test:related -- scripts/testing/repository-rules.mjs
npm.cmd run check
```

Because this change affects only frontend test infrastructure and no Rust code, Rust verification loops and `npm.cmd run verify` are outside this slice.

## Non-Goals

- Adding `getImports()` or `hasImportPrefix()` to the repository index.
- Introducing a second imports-only AST representation.
- Optimizing or removing Cargo metadata freezing.
- Persisting benchmarks or enforcing a timing threshold in CI.
- Increasing the repository snapshot timeout as a substitute for reducing work.
