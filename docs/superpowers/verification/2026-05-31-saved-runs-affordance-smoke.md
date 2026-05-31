# Saved Runs Affordance Smoke Verification

> Date: 2026-05-31
> Branch: `saved-runs-affordance-smoke-coverage`

## Commands

```powershell
npm.cmd run smoke:analysis
npm.cmd run test -- src/lib/analysis-run-snapshot-affordance.test.ts src/lib/analysis-run-companion-state.test.ts
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures
npm.cmd run verify
```

## Result

The analysis UI smoke passed with the saved-run affordance step group included.

The accepted smoke run includes:

- `PASS saved-runs-affordance.rows`
- `PASS saved-runs-affordance.missing-legacy`
- `PASS saved-runs-affordance.capture-failed`
- `Analysis UI smoke passed.`

## Fixture Summary

The seeded analysis redesign fixture set includes at least:

- `runs: 7`
- `snapshotMessages: 4`
- `chatMessages: 2`
- `sourceGroups: 1`
- `sources: 4`

The degraded saved-run labels covered by smoke are:

- `__analysis_redesign_fixture__ Missing Snapshot Run`
- `__analysis_redesign_fixture__ Capture Failed Snapshot Run`
- `__analysis_redesign_fixture__ Failed Run`
- `__analysis_redesign_fixture__ Cancelled Run`

## PASS Table

| Area | Evidence |
| --- | --- |
| Runs rows | Missing legacy and capture-failed rows show degraded badges, with error details omitted from row-scoped text. |
| Missing legacy | Opened-run details, Source, Evidence, and Chat expose helper-derived missing-snapshot affordances. |
| Capture failed | Opened report remains readable and details/Source show sanitized snapshot error text. |
| Live source explicitness | Degraded Source view explains that View live source opens live data, not the saved run snapshot. |
| Existing Source Browser smoke | Telegram, YouTube video, YouTube playlist, live source group, and captured run snapshot tab checks still pass. |

## Startup Caveat

A cold Rust/Tauri build can exceed the smoke harness 90-second MCP bridge discovery window before the app finishes compiling. A single warmed rerun after a compile-only bridge timeout is acceptable; a second bridge-discovery failure should be treated as a real smoke harness or app startup failure.

## Cleanup

The smoke harness cleaned analysis redesign fixtures and stopped the Tauri dev process after the accepted run.
