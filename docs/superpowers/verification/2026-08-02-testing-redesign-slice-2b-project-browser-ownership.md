# Testing Redesign Slice 2B: Project and Browser Ownership Verification

## Checkpoint Context

- Final verified source checkpoint:
  `3ddb3a18` (`test: harden component cleanup wiring guard`).

This record is documentation for that source checkpoint. It does not add a
runner, scheduler, retry, timing collector, dependency, CI configuration, Rust
behavior, or production behavior. The separate documentation commit that
contains this record is evidence-only and is not the verified source
checkpoint.

## Ownership and Complete Verification

`test` remains the manual all-project Vitest compatibility command. The
sequential fail-closed `verify` command instead runs these owner commands once,
in this order, after the sidecar binary and live transition gates:

```text
test:unit
test:component
test:architecture
test:legacy-contract
test:integration:os
test:e2e
```

`test:e2e` is the compatibility alias `npm.cmd run
test:gemini-browser-adapter:e2e`; that existing adapter command remains the
single Playwright definition and census owner. `verify` then preserves `check`,
`check:rustfmt`, Cargo workspace check/test, and `git diff HEAD --check` in the
same sequential fail-fast chain. It contains no `test` step, so no Vitest file
is run twice by the complete gate.

## Fresh Commands and Results

The following commands were run for this checkpoint:

```powershell
npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20
npm.cmd run verify
```

The owner results were non-empty and disjoint:

| Owner | Vitest files | Vitest tests |
| --- | ---: | ---: |
| `test:unit` | 81 | 764 |
| `test:component` | 19 | 121 |
| `test:architecture` | 1 | 2 |
| `test:legacy-contract` | 78 | 787 |
| `test:integration:os` | 6 | 87 |
| Total | 185 | 1,761 |

`test:e2e` passed 79 Playwright tests. The live census independently reported
192 filesystem candidates: the same 185 Vitest files and 7 Playwright files,
with every candidate owned exactly once. The live source-contract ledger
reported 671 rows, 670 open.

The full `npm.cmd run verify` exited `0` on its first run. Sidecar binary
validation, live transition validation, all six owners, Svelte check,
`check:rustfmt`, Cargo workspace check/test, and the git diff gate completed in
order.

## Chromium Lifecycle Acceptance Evidence

```powershell
npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20
```

The exact command exited `0`. Runs 1 through 20 each reported `pass`, and the
final CIM snapshot produced no leak diagnostic.

## Frozen Ledger Reconciliation

The full gate exposed a stale root-Vite assertion left by the approved project
split. `SC-000508` now hashes the corrected assertion for root Vite plugin
ownership; `SC-000509` and `SC-000510` only update their shifted manual source
ranges. All three preserve their stable IDs, invariant/disposition, lineage,
and replacement metadata. No row, path, or source-reader obligation was added.
