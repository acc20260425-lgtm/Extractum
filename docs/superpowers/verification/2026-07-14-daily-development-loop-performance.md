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

The user explicitly approved the one-time cleanup. Immediately before it, no
`cargo`, `rustc`, `rust-analyzer`, Extractum, or Tauri-dev process was running.
`cargo clean` removed 166,392 files / 359.0 GiB. This was the only cleanup; the
new profile was not warmed and then deleted.

| Observation | Value |
| --- | ---: |
| Target before cleanup | 358.95 GiB |
| Historical `codex-*` targets before cleanup | 18 |
| First `cargo check --timings` after cleanup/profile change | 234.84 s |
| First full `cargo test` after cleanup/profile change | 203.00 s |
| First full Rust test harness | 1,125 passed in 18.10 s |
| Transitional post-test `cargo check` | 5.15 s |
| Steady no-op `cargo check` | 1.08 s |
| Repeated full `cargo test` | 21.25 s |
| Repeated Rust test harness | 1,125 passed in 17.70 s |
| Target after canonical warm-up | 4.36 GiB |
| `codex-*` targets after warm-up | 0 |

The focused wrapper selected and passed three
`prompt_packs::runtime::tests::load_run_runtime_config*` tests, so its green
result was not a zero-test false positive. Ordinary commands created no new
slice-specific target.

Cargo timing report:
`src-tauri/target/cargo-timings/cargo-timing-20260714T180231751Z-a54253738dfaee23.html`.
The three longest units in that cold report were:

| Unit | Duration |
| --- | ---: |
| `windows 0.61.3` | 42.82 s |
| `tauri-utils 2.8.3` | 27.35 s |
| `extractum 0.2.0` | 26.22 s |

These cold/profile-triggered measurements are recorded as operational evidence
and are not compared directly with the old partially warmed build baseline.

## Final Gates

Status at this checkpoint: final mixed frontend/Rust gates have not run because
the Cargo-profile task is still pending.
