# Extractum Process Extraction Verification

**Date:** 2026-07-17
**Baseline commit:** `bfc86d6b73301cd6e9fe9f64d900d37f1442d79e`
**Candidate commit:** `b364756c7b5768d644321afeaeb81ec04e2481a4`
**Outcome:** `not_retained`

## Environment

- Cargo/Rust: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`; `rustc 1.95.0 (59807616e 2026-04-14)`.
- Valid session: `20260718T003457504-eb40c172356b41cb85ca1459dfd4f374`; baseline attempt `baseline-20260718T003544477-c76d4fc208ef422a9d40c9225a8b65f1`; post attempt `post-20260718T010203799-0ce42526f7c04f8397203b5b61d059f9`.
- Invalidated sessions, excluded from medians: six pre-baseline/infrastructure attempts covering environment capture, locator replacement, executor transcription, Windows path length, `Start-Process -Wait`, and leaked `ErrorActionPreference`; their `invalid-session.json` files remain under `%TEMP%/extractum-process-*`.
- Power: Balanced, GUID `381b4222-f694-41f0-9685-ff5bb260df2e`.
- Defender: unavailable (`Access denied`).
- Canonical target: `G:\Develop\Extractum\src-tauri\target`.
- Platform/host: Windows / `x86_64-pc-windows-msvc`.
- Cross-target scope: not an acceptance gate for Windows-only Phase 3.

## Boundary Evidence

- The candidate moved shared process lifetime, hidden-child, and Windows job-object infrastructure below the application while preserving all app consumer paths through private facades.
- Consumer hash comparison: 12 unchanged / 12.
- Reviewed public API: process lifecycle types/functions, hidden-child flag/helper, and `ProcessTreeGuard` construction/assignment/termination only.
- Direct dependency roots: `anyhow`, `parking_lot`, `tokio`, `windows-sys`.

## Test Inventory

- Baseline total: 1126.
- Candidate total: 1126.
- Missing baseline tests: 0.
- Process tests before/after: 20 / 20.

## Measurements

| Series | Samples (ms) | Median (ms) |
| --- | --- | ---: |
| Baseline app-domain | 10112, 9130, 9171, 10184, 9121 | 9171 |
| Candidate focused process | 2101, 1042, 2052, 1070, 2049 | 2049 |
| Baseline app-shell | 9158, 9127, 10138, 9135, 9128 | 9135 |
| Candidate app-shell | 10188, 10177, 10137, 11167, 10160 | 10177 |
| Reserved/repeat shell | 9144, 9156, 9134, 9144, 9127 / repeat not used | 9144 / n/a |

- Shell delta: +1042 ms / +11.406677613574166%.
- Primary cap pass: false (limit +500 ms and +5%).
- Repeat used/pass: false / false; the primary result was outside the 8%/800 ms marginal window.
- Focused process timing role: diagnostic only; median improved by 7122 ms (77.66%).

## Verification

- Boundary RED: 4 of 5 assertions failed because the workspace member, moved modules, and facades did not exist; the contract loaded without TypeScript/import errors.
- Boundary GREEN: 3 contract files, 14 tests passed.
- Focused process tests/check: exact narrow test 1/1; full process crate 20/20; package check passed.
- Windows process-crate check: passed on `x86_64-pc-windows-msvc`.
- App dependent checkpoint: passed (`cargo check -p extractum --all-targets`).
- Completion outcome: skipped after the predeclared performance gate rejected the candidate.
- Restored workspace check/test: 2 source-contract files / 9 tests passed;
  workspace `cargo check --workspace --all-targets` passed; 12 coordinator
  tests passed.
- `npm.cmd run verify`: skipped on the negative path.
- Release no-bundle/startup smoke: skipped on the negative path.
- MSI/WiX: excluded due to pre-existing baseline failure.

## Decision

`decision.json` recorded `reason=protocol_completed`, `primary_shell_pass=false`, `repeat_used=false`, and `retain_candidate=false`. The candidate is not retained because the app-shell median regressed by 1042 ms / 11.41%, beyond both retention limits. Rollback commit: `c47372dc`.
