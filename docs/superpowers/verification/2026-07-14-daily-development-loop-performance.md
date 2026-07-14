# Daily Development Loop Performance Verification

## Scope

- Approved design: `docs/superpowers/specs/2026-07-14-daily-development-loop-performance-design.md`
- Implementation plan: `docs/superpowers/plans/2026-07-14-daily-development-loop-performance.md`
- Baseline machine: the same Windows workstation used for the approved design measurements

## Pre-change Baseline

| Operation | Seconds | Inventory |
| --- | ---: | --- |
| Full Vitest, forks | 130.49 | 156 files / 1,253 tests |
| Full Vitest, threads auto probe | 65.09 | 156 files / 1,253 tests |
| No-op cargo check | 1.15 | canonical target |
| No-op cargo test | 22.72 | tests 18.83 s |

## Frontend Checkpoint

The frontend checkpoint is commit `374182b8` (`perf: speed up frontend
feedback loop`). The committed suite contains 157 files and 1,262 tests: the
baseline inventory plus one contract file and nine tests from this slice.

| Check | Exit | Wall seconds | Vitest seconds | Executed set |
| --- | ---: | ---: | ---: | --- |
| `test:changed` on a clean tree | 0 | 3.32 | — | none; `No test files found` |
| `test:changed:last` after the config commit | 0 | 73.05 | 69.70 | full 157 files / 1,262 tests |
| `test:changed` with a temporary `scripts/tauri.mjs` edit | 0 | not separately timed | 0.997 | 1 file / 5 tests |
| `test:related -- src\lib\api\llm.ts` | 0 | 6.55 | 0.993 | 1 file / 5 tests |
| full run 1 | 0 | 68.57 | 65.25 | 157 files / 1,262 tests |
| full run 2 | 0 | 68.72 | 65.39 | 157 files / 1,262 tests |
| full run 3 | 0 | 70.08 | 66.69 | 157 files / 1,262 tests |

The full-run wall-time median is **68.72 seconds**, materially below the
same-machine 130.49-second forks baseline. The 60–70-second range remains
contextual evidence rather than a portable threshold.

The full `test:changed:last` selection is expected: the checkpoint changes
`vite.config.js` and `package.json`, which are Vitest force-rerun triggers. The
temporary leaf-source probe selected only `scripts/tauri.test.ts` and was
removed byte-for-byte before this evidence file was created. The explicit
related command also confirmed Windows backslash-path normalization.

## Cargo Profile and Cache

Status at this checkpoint: the Cargo profile has not been changed, the cleanup
decision has not been requested, and no new-profile Cargo command has run.

## Final Gates

Status at this checkpoint: final mixed frontend/Rust gates have not run because
the Cargo-profile task is still pending.
