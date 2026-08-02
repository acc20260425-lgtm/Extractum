# Testing Redesign Slice 2B: Project and Browser Ownership Verification

## Checkpoint Context

- Slice 2B prerequisite checkpoint: `df59b053fc3a9045978b1cfd9e39de0a7e03b67f`
  (`test: add Chromium lifecycle audit`).
- Final verified source checkpoint:
  `4352e0000dacb90495b92392fd17f55e2c4c152f`
  (`test: preserve Slice 2B verification coverage`).

This record is documentation for that source checkpoint. It does not add a
runner, scheduler, retry, timing collector, dependency, CI configuration, Rust
behavior, or production behavior.

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
npm.cmd run test -- scripts/verify.test.ts
npm.cmd run test:unit
npm.cmd run test:component
npm.cmd run test:architecture
npm.cmd run test:legacy-contract
npm.cmd run test:integration:os
npm.cmd run test:e2e
npm.cmd run test -- src/lib/research-projects-foundation-contract.test.ts scripts/testing/validate-testing-transition.test.ts
node scripts/validate-testing-transition.mjs
npm.cmd run verify
```

The owner results were non-empty and disjoint:

| Owner | Vitest files | Vitest tests |
| --- | ---: | ---: |
| `test:unit` | 81 | 758 |
| `test:component` | 19 | 121 |
| `test:architecture` | 1 | 2 |
| `test:legacy-contract` | 78 | 787 |
| `test:integration:os` | 6 | 87 |
| Total | 185 | 1,755 |

`test:e2e` passed 79 Playwright tests. The live census independently reported
192 filesystem candidates: the same 185 Vitest files and 7 Playwright files,
with every candidate owned exactly once. The focused contract plus transition
unit command passed 2 files and 57 tests.

The full `npm.cmd run verify` exited `0`: sidecar binary validation, live
transition validation, all six owners, Svelte check, rustfmt, Cargo workspace
check/test, and `git diff HEAD --check` completed in order.

## Chromium Lifecycle Acceptance Evidence

Task 4's approved acceptance command was retained as the lifecycle evidence;
it was not rerun for this documentation checkpoint:

```powershell
npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20
```

Its approved controller observation recorded all 20 outcomes as `pass`, with
no retry and no newly leaked Playwright headless-Chromium process after the
before/after process audit. The initial managed-sandbox observation failed
before run 1 because its required CIM process query was denied; that was an
infrastructure limitation, not a lifecycle-run outcome. The accepted
outside-sandbox observation supplied the 20-pass, no-leak result.

## Frozen Ledger Reconciliation

The full gate exposed a stale root-Vite assertion left by the approved project
split. `SC-000508` now hashes the corrected assertion for root Vite plugin
ownership; `SC-000509` and `SC-000510` only update their shifted manual source
ranges. All three preserve their stable IDs, invariant/disposition, lineage,
and replacement metadata. No row, path, or source-reader obligation was added.
