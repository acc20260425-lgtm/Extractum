# Gemini Browser Crate Extraction Verification

## 1. Scope and commits

Phase 4 extracts the portable Gemini Browser domain engine into
`extractum-gemini-browser` while retaining concrete process, Tauri, Apalis, and
application-error adapters in `extractum`.

Implementation commits:

- `b829492c test: characterize Gemini browser terminal payloads`
- `614ed94f refactor: introduce Gemini browser executor port`
- `0e58f25a refactor: prepare Gemini browser portable modules`
- `bee0a812 refactor: prepare Gemini browser job engine`
- `9b52a9de refactor: extract Gemini browser domain crate`
- `d599d376 test: update lifecycle contract for Gemini crate`

The first completion attempt was recorded as incomplete. The focused
remediation commit updated the lifecycle source contract to the moved portable
launch module; a new retained-path attempt then passed every mandatory gate.

## 2. Final ownership and dependency roots

The workspace contains `.`, `crates/extractum-core`, and
`crates/extractum-gemini-browser`. The application has one path dependency on
the new crate. The new crate's production dependency roots are exactly
`parking_lot`, `serde`, `serde_json`, `time`, `tokio` with `macros`, `sync`, and
`time`, `tokio-util`, and `url`. Its development dependencies are `tempfile`
and `tokio` with `rt` and `test-util`. It has no `extractum-core` dependency.

Portable error, executor port, DTO, run-id, run-log, launch specification,
protocol codec, state, runtime, status, submission, reconciliation, and
execution logic moved to the crate. Concrete sidecar/CDP processes, Tauri
commands/state, Apalis adapters, and `AppError` mapping remain app-side.

## 3. Pre-manifest core-use audit

The pre-seam command was:

```powershell
rg -n "extractum_core|crate::(?:error|time|media_metadata|compression)|AppError|AppResult|encode_media_metadata|decode_media_metadata" src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/run_log.rs src-tauri/src/gemini_browser/paths.rs src-tauri/src/gemini_browser/sidecar_launch.rs src-tauri/src/gemini_browser/cdp_chrome.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/jobs.rs src-tauri/src/gemini_browser/sidecar.rs src-tauri/src/gemini_browser/state.rs
```

It found no match in `types.rs` or `sidecar_launch.rs`; prepared portable
fragments did not use `extractum_core`, media metadata, compression, or the
core-time API. Application facade code retained `AppError`/`AppResult` use.

## 4. Frozen 94-name inventory and ownership

The baseline inventory contained exactly 94
`gemini_browser::...: test` names. The source-boundary GREEN check proved every
frozen name occurred exactly once after extraction, with 75 names owned by
`extractum-gemini-browser` and 19 names owned by the application. The extracted
crate currently runs 77 total unit tests because it also contains two added
lifecycle characterization tests.

## 5. Characterization and exact outward serialization

The characterization commit froze queued, running, cancelled, timeout, and
failed run-log serialization, including value-only timestamp normalization,
and froze the legacy `AppError` JSON mapping at the executor adapter.

Post-move exact commands each ran one non-empty test and passed:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib runtime::tests::wait_for_result_removes_waiter_on_timeout -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json -- --exact
```

Result: four commands, one passed test each, zero failures.

## 6. Source-contract RED/GREEN

The initial extraction-boundary RED run failed 6 of 7 assertions because the
crate, workspace member, ownership, and root contract did not yet exist. After
the move, the combined boundary/workspace run passed 2 files and 12 tests:

```powershell
npm.cmd run test -- src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts
```

The GREEN contract confirmed the curated roots, exact Cargo/lock dependency
edges, portable-type exclusions, no app-side `CancellationToken`, and the
75/19 frozen ownership split.

## 7. Package, app, workspace, and Linux results

The following package and immediate-dependent checkpoints passed:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

The crate reported 77 passed tests. The application reported 1030 passed tests.
The three exact app process sentinels for owned Chrome shutdown, concurrent
sidecar stderr draining, and tainted cancelled transport each passed one test.

The target `x86_64-unknown-linux-gnu` was installed during verification, and
this portability command passed:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --target x86_64-unknown-linux-gnu
```

The final direct workspace check and `check:rustfmt` passed. The final workspace
test command passed all targets; its visible package summaries included 1030
application tests and 77 Gemini Browser crate tests with zero failures.

## 8. Advisory timing and restoration hashes

Pre-move handoff object:

```text
hash=263EC9AAB16EE59C9D94F9952726553B17E9DA43D2EB393D996102E495F14338
warmup_ms=9563
samples_ms=[8492, 8593, 8866]
median_ms=8593
restored_hash=38586744870A8A83271028A65646F7018FE590D2B1F2B10A3B83C825FD6F5963
clean_status=true after targeted git restoration
```

The inverse `apply_patch` normalized raw line endings, so the restored raw hash
did not equal the original raw hash. Per protocol, the baseline is incomplete.

Post-move handoff object:

```text
hash=38586744870A8A83271028A65646F7018FE590D2B1F2B10A3B83C825FD6F5963
warmup_ms=1339
samples_ms=[960, 980, 965]
median_ms=965
restored_hash=1F411645749A5DA181AAD2E080D6DDFBFADC740516E82900171F772F03009E29
clean_status=false before targeted git restoration
```

The post-move inverse patch had the same raw-line-ending problem. A targeted
`git restore --source=HEAD --worktree` then restored SHA-256
`38586744870A8A83271028A65646F7018FE590D2B1F2B10A3B83C825FD6F5963`
and clean status. The timing sequence was not retried.

Advisory result: `incomplete / no conclusion`. Delta and percentage are not
computed because both handoff series failed their raw restoration proof.

## 9. Ordinary workspace-check duration

The single direct mandatory command was:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

The retained-path attempt's exact Cargo line was:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.62s
```

The mechanical conversion was `$workspaceCheckMs = 1620`. This completed
result is below 15,000 ms, so it breaks rather than advances the roadmap's
adjacent-completed-slice investigation sequence. The earlier failed attempt's
38.14-second check is not a roadmap signal.

## 10. Sidecar, CDP, no-bundle, startup, and shutdown evidence

- Node smoke returned `id: "smoke-1"` and `response.type: "status"`.
- Sidecar TypeScript/package build completed and wrote
  `src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe`.
- Binary smoke returned `id: "smoke-1"` and `response.type: "status"`.
- CDP negative smoke returned `needs_manual_action` with
  `start_chrome_cdp` for `http://127.0.0.1:65530`.
- `npm.cmd run tauri -- build --no-bundle` exited 0 with
  `Finished release profile [optimized] target(s) in 3m 39s` and produced
  fresh `extractum.exe` and `gemini-browser-sidecar.exe` release artifacts.
- Visible startup used application PID `20908`. After Managed status refresh,
  exactly one owned sidecar existed, PID `25192`.
- Closing the main window normally removed both PID `20908` and PID `25192`;
  neither process required force-stop.

## 11. Infrastructure, exclusions, and remediation

MSI/WiX packaging was intentionally excluded; the required release gate used
`--no-bundle`.

The first sidecar build attempt inside the sandbox hit `EPERM` while writing
`sidecars/gemini-browser/dist`; the required rerun outside the sandbox passed.
The first no-bundle controller timed out after 124 seconds while its owned Cargo
compile continued. Its exit status was therefore discarded; a subsequent
single controlled rerun exited 0 and supplied the build evidence above.

The first completion attempt stopped at this command:

```powershell
npm.cmd run verify
```

Literal failure summary:

```text
Test Files  3 failed | 167 passed (170)
Tests  3 failed | 1368 passed (1371)
Duration  142.14s

FAIL src/lib/external-process-lifecycle-contract.test.ts
Error: ENOENT: no such file or directory, open 'G:\src-tauri\src\gemini_browser\sidecar_launch.rs'

FAIL scripts/process-shell-diagnostic/runtime.test.ts
Expected: "timeout"
Received: "termination_unconfirmed"

FAIL sidecars/gemini-browser/src/adapter.test.ts
writes a reduced extraction artifact for ok timeout_latest without changing status
TypeError: .toMatch() expects to receive a string, but got object

FAIL sidecars/gemini-browser/src/adapter.test.ts
writes a reduced extraction artifact for missing answer timeout failures
TypeError: .toMatch() expects to receive a string, but got object

EPERM: operation not permitted, open 'G:\Develop\Extractum\.svelte-kit\generated\client\nodes\9.js'
Verification failed during: npm run test
```

The missing `sidecar_launch.rs` import was a repository contract regression
caused by the extraction path move. Focused RED/GREEN updated that contract to
read `crates/extractum-gemini-browser/src/sidecar_launch.rs`, require
`std::env::current_exe` app-side, and require `bundled_sidecar_path` portable.

The two artifact assertions received `null` because sandbox execution could
not overwrite existing generated artifact files. The process-tree diagnostic
captured `taskkill` exit code 1 with `ERROR: Access denied`, so its conservative
`termination_unconfirmed` classification was correct. Both unchanged test
groups passed outside the sandbox. The combined focused confirmation passed 3
files and 42 tests.

The retained-path `npm.cmd run verify` then exited 0 outside the sandbox. A
compact complete Vitest confirmation reported 170 files and 1,376 tests passed
with zero failures. `check:rustfmt` and the full Cargo workspace test also
exited 0.

## 12. Result and next roadmap action

**Result: implemented and retained.**

The specification links this verification with status `Implemented and
retained`. The roadmap records Phase 4 as `done: retained`, the 75/19 ownership
split, final dependency roots, and the 1,620 ms ordinary workspace result.

Next roadmap action: start the Phase 5 `extractum-llm` JIT boundary design. It
has not started.
