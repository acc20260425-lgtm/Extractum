# Rust Workspace Core Extraction Verification

## Scope and Commits

- Baseline code commit: `c49cd9ef370a8cfdbfef9f437a34ff4b2bb83217`
- Measurement-protocol update: `cb41f58e`
- Implementation commit: `a448512b3f0d3d0660535c65c220d2f35b0ade4d`
- Workspace members: `extractum` and `extractum-core`
- Canonical target: `src-tauri/target`

This slice moved only `error`, `time`, and `compression` into
`extractum-core`. The application keeps its existing `crate::error`,
`crate::time`, and `crate::compression` paths through explicit private wrapper
modules. The core manifest depends only on `serde`, `time`, and `zstd`; member
manifests contain no profile sections, and `Cargo.lock` contains no external
version update.

## Environment and Method

- OS: Microsoft Windows NT `10.0.26100.0`
- CPU: Intel64 Family 6 Model 158 Stepping 11, 4 logical cores
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Cargo target: `G:\Develop\Extractum\src-tauri\target`
- Process inventory: native Cargo/rustc/rust-analyzer/Extractum and ports
  1420/1421 were clear; CIM command-line inspection was denied, so the
  documented `Get-Process` fallback was used and 13 unrelated Node processes
  were left alone.

The domain probe alternated the inert comment
`// cargo-measurement-probe: a|b` immediately before the test module in
`src-tauri/src/notebooklm_export/chunker.rs`. The shell probe used the same
comment immediately before `run()` in `src-tauri/src/lib.rs`. Every recorded
probe caused Cargo to rebuild `extractum`, and both files were restored with a
clean worktree after their series.

`lib.rs` had mixed line endings before measurement. Focused patching and Git
checkout normalized its working-copy representation to CRLF while producing
no content or staged diff. The canonical post-checkout SHA-256 is
`EC21466993001F6D3C76BFC56EDB3959478DDCA0A3F86A42D278617F4AC89312`;
the domain probe restored its original SHA-256
`164ECD9A59515DC5E9B327E3A71341CBA8D1FFE8FA47BE8D7975EDCA57059E7B`.

## Raw Measurements

All values are wall milliseconds. Medians use the predeclared rule; the two
production values use their midpoint.

| Metric | Baseline samples | Post samples | Baseline median | Post median | Delta |
| --- | --- | --- | ---: | ---: | ---: |
| no-op check | 1147, 1057, 1010, 982, 1011 | 1072, 1141, 994, 1001, 1004 | 1011 | 1004 | -7 ms (-0.69%) |
| domain probe | 8350, 7613, 7572, 7608, 7562 | 8397, 7631, 7632, 7595, 7817 | 7608 | 7632 | +24 ms (+0.32%) |
| shell probe | 18411, 7592, 7561, 7609, 7589 | 19036, 7430, 7732, 7616, 7634 | 7592 | 7634 | +42 ms (+0.55%) |
| test compile (`--no-run`) | 22990, 1405, 1191 | 19560, 1645, 1219 | 1405 | 1645 | +240 ms (+17.08%) |
| test execution | 18888, 18468, 18723 | 20081, 19866, 19174 | 18723 | 19866 | +1143 ms (+6.10%) |
| production `--no-bundle` | 293071, 288779 | 291249, 285118 | 290925 | 288183.5 | -2741.5 ms (-0.94%) |

The first shell and test-compilation samples were cold/noisy in both series.
They remain visible rather than being removed. No-op did not regress by both
5% and 0.5 seconds, and shell did not regress by both 5% and 1 second, so
neither predeclared regression gate fired. The minimal core extraction is
enabling infrastructure; the later 25%/two-second domain stop/go gate does not
apply to this slice.

## Cargo Timing Reports

- Baseline: `src-tauri/target/cargo-timings/cargo-timing-20260716T170048700Z-a54253738dfaee23.html`
  (`33257 ms`, SHA-256
  `F055C5BFD8E0857FC8217985888BBDE8A4E3B0DDED52968F2F7B507F85A6B04C`)
- Post: `src-tauri/target/cargo-timings/cargo-timing-20260716T174648696Z-a54253738dfaee23.html`
  (`29085 ms`, SHA-256
  `7BEFF0800B30553C3FD9D03E9144F70813EAD88B6C60A4B72B6748EB722097BD`)

The post report contains both `extractum` and `extractum-core` library/test
units. The total timing difference is diagnostic local-machine evidence, not a
standalone performance claim.

## Test Inventory and Correctness

- Baseline Rust inventory: 1125 unique tests
- Post-workspace Rust inventory: 1125 tests
- Missing baseline tests: 0
- Application package: 1107 tests
- Core package: 18 tests
- Moved-module sentinels for error, time, and compression: all present
- Workspace source contracts: 15 tests passed across 2 files
- Focused core check/test: passed, 18/18
- Root library tests: passed, 1107/1107
- Full workspace tests: passed, 1125/1125
- Full `npm.cmd run verify`: 160 Vitest files / 1272 tests, Svelte 0 errors
  and 0 warnings, workspace rustfmt/check/test and diff check all passed
- `npm.cmd run tauri -- build --no-bundle`: passed and produced
  `src-tauri/target/release/extractum.exe`

Release compilation retained two pre-existing warnings: unused `Manager` in
`takeout_import/state.rs` and unused `PromptPackRunState::track`. The workspace
slice introduced no warning in owned code.

## Live Smoke Evidence

The canonical `npm.cmd run tauri dev` workflow compiled from the workspace and
opened a main window titled `extractum`. The first automated host command timed
out during a cold compile; a warmed retry opened the window. A sandboxed
`CloseMainWindow` request could not close that dev instance, so its owned
processes were cleaned up and ports 1420/1421 were confirmed free.

The freshly built release executable opened a main window titled `extractum`
and then closed normally with `CloseMainWindow=True`. Desktop navigation could
not be automated by the available browser-only tooling, so the automated live
evidence is startup, main-window creation, and normal release shutdown.

## Pre-existing WiX Limitation

Full MSI bundling was already unavailable on the baseline commit: one
`tauri build` reached `light.exe` and failed, while a second reached WiX and
hung. The first attempt also included a cold release compile, so neither full
build is a valid timing sample. This slice therefore uses
`npm.cmd run tauri -- build --no-bundle`, which builds the same release exe
needed by the startup smoke while excluding the unavailable packaging step.
WiX/interactive-session diagnosis remains a separate follow-up and is not
classified as a workspace regression.

## Deferred Work

No deferred extraction was implemented. Backlog remains:

- move `sql_helpers` and `tx` when their boundary is ready;
- split and extract `media_metadata` without pulling grammers into core;
- place the shared YouTube DTO below the future sources/youtube crate boundary;
- audit `test_support` for every later domain extraction;
- optionally consolidate direct `analysis/trace.rs` zstd calls behind the
  shared compression API.

## Result

The first workspace/core slice is verified and retained. It preserves the
complete test inventory and application startup while making full Cargo gates
workspace-aware. The measured daily-loop impact is neutral at this extraction
size; meaningful compilation gains remain a question for later domain slices.
