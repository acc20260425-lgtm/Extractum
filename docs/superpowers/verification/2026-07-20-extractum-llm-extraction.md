# Extractum LLM Crate Extraction Verification

## Scope and Commits

Phase 5 implements the owner-approved [LLM crate boundary](../specs/2026-07-20-llm-crate-boundary-design.md) through the [execution plan](../plans/2026-07-20-extractum-llm-extraction.md). It extracts the portable LLM execution engine while retaining persistence, credentials, Tauri IPC, events, and diagnostics in the application.

Implementation commits:

- `335c45032e82439ca028c379851f0ab4eaa222af test: characterize LLM IPC and errors`
- `d6164c6b87cda7d9e8804122fe1e830dd6596f02 refactor: prepare portable LLM boundary`
- `8f9f56935d67439066fd5623c6e617fdb9748c9f refactor: make LLM engine portable`
- `3a3833da99682cf1f1c8f4b45992c303ca2b5fc9 test: define LLM crate boundary`
- `e5873cc6d470bad98678124a8c7653cda807b32b refactor: extract portable LLM engine`

## Final Ownership and Dependencies

`src-tauri/crates/extractum-llm/src` owns `types.rs`, `provider.rs`, `gemini.rs`, `openai_compat.rs`, `runner.rs`, `scheduler.rs`, and `streaming.rs`. The application retains `src-tauri/src/llm/app_types.rs`, `profiles.rs`, and `mod.rs`; `mod.rs` is the private compatibility facade and still owns all nine Tauri commands and `llm://response` event emission.

The crate has exactly seven direct production dependency roots: `extractum-core`, `reqwest`, `secrecy`, `serde`, `serde_json`, `tokio`, and `tokio-util`. Production Tokio features are `macros`, `sync`, and `time`; dev-only Tokio features are `io-util`, `net`, `rt`, and `test-util`. `reqwest` and `secrecy` are canonical workspace dependencies inherited by both the app and crate. The app has one path edge to `extractum-llm`. The lock diff added the new package with exactly those seven dependencies and the app edge, with no registry version or checksum churn.

## Safe Profile and Credential Boundary

`LlmProviderAccess::new` and `ResolvedLlmProfile::new` consume `SecretString` credentials. Public getters expose only profile ID, provider, default model, and base URL; neither type serializes or exposes a public secret field/getter. The two external struct literals in the analysis report harness and prompt-pack completion transport were migrated to constructors before extraction.

Both black-box partial-input precedence seams passed: saved API key plus configured base URL, and configured API key plus saved base URL. Profile persistence remains app-owned and the credential remains in `SecretStoreState`/keyring scope.

## Frozen Test Inventory

The baseline contained exactly 51 frozen Rust LLM test names. Final exact ownership is 36 baseline names in `extractum-llm` and 15 in `extractum`, each occurring once. Cargo execution lists 37 crate tests and 19 app LLM tests because five Rust tests were added separately: one safe resolved-profile test, two partial-input profile-access tests, and two IPC/event/error serialization tests.

No disabled `#[cfg(any())]` or `#[cfg(FALSE)]` copy exists in either owner.

## IPC, Event, Error, Profile, and Scheduler Compatibility

The nine command names and frontend payload/result shapes remain unchanged. The event channel remains `llm://response`; event kinds remain `queued`, `started`, `delta`, `completed`, `failed`, and `cancelled`, including nullable field behavior and the exact cancellation message `Request cancelled.`

Characterization proved command errors retain the `AppError` JSON shape while failed stream events retain their distinct event shape. Scheduler cancellation remains distinct from typed provider failure. Profile resolve-once behavior, provider aliases/base-URL policy, timeouts/retries, queue/start/delta/terminal ordering, ignored event-delivery errors, background spawn, and immediate outer `Ok(())` control flow are unchanged.

## Boundary Contract and Mechanical-Move Review

The initial source-boundary RED run executed five tests and failed all five because the workspace member, manifest, root, ownership paths, lock edge, and facade did not yet exist. After extraction, the four-file boundary group passed 20 tests.

Manual `git diff --cached --find-renames --summary` review reported all seven implementation files as 100% renames. No implementation was copied, disabled, or renamed to evade the scanner. Comparing the prepared app facade at `8f9f5693` with the extraction tree showed only module/import/re-export path changes; command and lifecycle control flow remained intact. Existing workspace and Gemini Browser exact allowlists were extended without weakening their prior assertions.

## Package and Workspace Gates

The focused and package gates passed:

```text
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets --locked: PASS
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets: 37 passed
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked: PASS
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets: 998 passed
boundary contract group: 4 files, 20 tests passed
npm.cmd run check:rustfmt: PASS
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets: PASS
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets: PASS
npm.cmd run verify: PASS
```

The retained-path full verify reported 171 Vitest files and 1,383 tests passed, Svelte diagnostics 0 errors/0 warnings, rustfmt success, and successful Cargo workspace check/test.

## Advisory Focused Timing

The baseline and candidate probes were byte-identical:

```text
BASELINE_SHA256=EA6305B4FDBF8A9C7E530EA807A31C5D85A4CFC098F098D6327AC5B8D7DE691A
BASELINE_BLOB=3a7d4086dc785062e76e24f79c1ef853ab0e4ef1
BASELINE_RAW_MS=[]
CANDIDATE_SHA256=EA6305B4FDBF8A9C7E530EA807A31C5D85A4CFC098F098D6327AC5B8D7DE691A
CANDIDATE_BLOB=3a7d4086dc785062e76e24f79c1ef853ab0e4ef1
CANDIDATE_RAW_MS=[]
```

Both one-shot series were incomplete before warm-up because the timing helper could not execute `codex.exe --codex-run-as-apply-patch`: `Access is denied`. Each block nevertheless proved byte restoration and a clean tree. Per the advisory protocol neither series was retried: no median / no performance conclusion. Timing did not decide retention.

## Ordinary Workspace Timing Signal

The single ordinary mandatory workspace check produced the exact line:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.41s
```

The mechanical result is 10,410 ms, below 15,000 ms. Phase 4's 1,620 ms result also falls below the threshold, so Phase 5 cannot form an adjacent above-threshold pair and no investigation is triggered.

## Release and Startup Evidence

`npm.cmd run tauri -- build --no-bundle` passed after the cold compilation cache was completed and reported `Finished release profile [optimized] target(s) in 3m 33s`. It produced `G:\Develop\Extractum\.worktrees\llm-extraction\src-tauri\target\release\extractum.exe`.

The bounded smoke started that exact executable hidden. PID `15172` remained alive after five seconds; exact-PID force cleanup and reap passed, and the post-smoke Extractum process count was zero. No live provider request was made.

## Infrastructure Failures and Exclusions

MSI/WiX packaging was excluded as planned because of the documented pre-existing `light.exe` issue; the required release gate was `--no-bundle`.

The first sandboxed full verify had two unrelated environment-sensitive failures: browser close exceeded its hook timeout once, and process-tree evidence recorded four `taskkill.exe` `Access denied` results. The browser test passed unchanged in isolation. The process-tree test passed unchanged outside the sandbox (13/13), and the full verify then passed outside the sandbox. No correctness failure was excluded.

The first cold no-bundle controller reached its 10-minute tool timeout while Cargo was still compiling normally and produced no executable. A warmed continuation completed successfully and supplied the release evidence above.

## Result and Next Phase

**Result: implemented and retained.**

All non-timing correctness, ownership, package, workspace, release, startup, cleanup, and final repository gates passed. Phase 6 `extractum-prompt-packs` is next and still requires a fresh owner-approved JIT boundary design; this result does not authorize implementation directly.
