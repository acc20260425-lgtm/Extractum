# Testing Redesign Slice 3C: Source-Contract Completion Verification

## Checkpoint status

- Status: **COMPLETE**. The original final unsandboxed
  `npm.cmd run verify` exited 1 after 253.645 seconds at one component owner.
  After the bounded fix, the user explicitly amended Task 15 to permit exactly
  one corrective full run; it exited 1 after 460.9 seconds at `check:rustfmt`.
  After the focused rustfmt correction, the user explicitly authorized one
  final full run; it exited 101 after 653.7 seconds during workspace Cargo
  tests in the `extractum-prompt-packs` library test binary.
  After the isolated package passed, the user authorized one last full run;
  it reproduced exit 101 after 492.4 seconds at the same workspace Cargo-test
  package/binary boundary.
- Subsequent exact feature-on diagnosis identified the deterministic Cargo
  feature-unification cause. After the narrow owner correction, the user
  authorized one completion run. A fresh process preflight found zero
  competitors and `npm.cmd run verify` passed every gate with exit 0 after
  818.8 seconds, including Cargo workspace tests and the final Git diff gate.
- Slice 3 is complete and Slice 4 is ready to begin.
- Environment: Windows PowerShell, Node.js 24.13.1, npm 11.12.1, rustc 1.95.0,
  and Cargo 1.95.0.

## Migration inventory

The source-contract transition itself is closed:

- 436 formerly open rows are closed; the ledger contains 671 rows and 0 open.
- All 86 legacy files are absent.
- All 79 connected components are closed. This is the exact forest count in
  the approved design; stale plan references to 84 were corrected during this
  evidence wave without changing the per-batch file or row accounting.
- Final future owners are Node 139 rows / 787 ordinals, jsdom 123 / 864, Cargo
  18 / 147, and Playwright 3 / 21.
- The legacy Vitest project, census owner, npm command, and verification gate
  are absent. The final census is 196 filesystem candidates: 186 Vitest files
  and 10 Playwright files.

The nine cutover batches and their cutover commits are:

| Batch | Commit | Legacy files | Rows closed |
| --- | --- | ---: | ---: |
| Wave 0 | `1d1bc712154739177c3e50d02346285dfb211f49` | 23 | 70 |
| Wave 1 | `6a0bd4f676fee67dbdd07794da73fe19855d9791` | 14 | 29 |
| Browser | `4daa19e71b8f1b4a9b6fb6fbadfaf70e8670086b` | 3 | 11 |
| Wave 2A | `a69579b2298ef98a9cccdd1d2cc9b8e8cf25d8b1` | 7 | 42 |
| Wave 2B | `08686951403c00b120f8ad3f65392641f04521a8` | 7 | 50 |
| Wave 2C | `361a77e171003da05c58f93d8ba870be9ed8d521` | 8 | 58 |
| Wave 2D | `b5bb2db79aedd5ef9421d9aa2f32827c013f48a1` | 7 | 56 |
| Wave 2E | `a577f22c11448b9c74406befb5e8d806732413b3` | 8 | 65 |
| Wave 3 | `ecef50c2645894cf6501beeb263a3dc59f2e8b6b` | 9 | 55 |
| **Total** |  | **86** | **436** |

The browser batch was not deferred: it cut over before Wave 2A. Wave 3 was
therefore the factual-last cutover and removed the empty legacy runner.

## Focused acceptance evidence

The final focused gates ran sequentially:

| Command | Result |
| --- | --- |
| `node scripts/testing/source-contract-redisposition-review.mjs check` | PASS, exit 0; `changed JSON paths` was empty. |
| `node scripts/validate-testing-transition.mjs` | PASS, exit 0; census 196/186/10 and ledger 671/0. Existing Rust unused/dead-code warnings were informational. |
| `npm.cmd run check` | PASS, exit 0; 0 errors and 0 warnings. |
| `git diff --check` | PASS, exit 0 before the evidence wave. |
| `npm.cmd run bootstrap:testing` | PASS, exit 0; rebuilt and confirmed the 45,200,439-byte Windows sidecar prerequisite. |

The first sandboxed CIM observation was invalid because `Get-CimInstance`
returned access denied as a non-terminating PowerShell error. It was discarded
instead of being treated as a zero-process result. The corrected preflight ran
outside the sandbox with `$ErrorActionPreference = 'Stop'`, enumerated
`Win32_Process`, and filtered repository-owned Vitest, Playwright, Cargo,
Rust, Vite, and Chromium processes by executable path or command line. It
exited 0 with `Repository-owned competing process count: 0`.

## Original full-verification observation

The original unsandboxed command was `npm.cmd run verify`. It exited 1 after
253.645 seconds:

| Gate reached | Observation |
| --- | --- |
| sidecar prerequisite | PASS. |
| transition validator | PASS; census 196/186/10, ledger 671/0. |
| `test:unit` | PASS; 118 files and 989/989 tests. |
| `test:component` | FAIL; 61 files passed and 1 failed, 280 tests passed and 1 failed. |

The failed declaration was
`library prototype contract > coordinates filter selection, row selection, and Inspector resizing in the screen component`
in `src/lib/components/research-projects/LibraryScreen.behavior.test.ts`.
It still queried the old synthetic `Select grid row ...` buttons after the
shared SVAR receiver had moved to its public `role="row"` interaction surface.

The bounded fix changed only the three stale row-selection queries to their
accessible row roles. The directly affected command
`npm.cmd run test:component -- src/lib/components/research-projects/LibraryScreen.behavior.test.ts`
then passed 1 file / 1 test (exit 0). The focused result was not by itself
promoted to full workspace evidence.

## Authorized corrective full-verification observation

The user explicitly amended the single-run policy to authorize exactly one
corrective `npm.cmd run verify` after the selector fix. A fresh unsandboxed CIM
preflight again found zero competing repository-owned processes; bootstrap was
not repeated. The corrective command exited 1 after 460.9 seconds:

| Gate reached | Observation |
| --- | --- |
| sidecar prerequisite | PASS. |
| transition validator | PASS; census 196/186/10, ledger 671/0. |
| `test:unit` | PASS; 118 files and 989/989 tests. |
| `test:component` | PASS; 62 files and 281/281 tests. |
| `test:architecture` | PASS; 1 file and 2/2 tests. |
| `test:integration:os` | PASS; 5 files and 44/44 tests. |
| Gemini browser adapter Playwright | PASS; 79/79 tests. |
| application Playwright | PASS; completed before the later rustfmt failure. |
| `npm.cmd run check` | PASS; completed before the later rustfmt failure. |
| `npm.cmd run check:rustfmt` | **FAIL**, exit 1; rustfmt required changes in `src-tauri/src/prompt_packs/runtime_commands.rs`. |

The fail-fast scheduler did not run Cargo workspace check/test or the final
Git diff gate. The nearby `Not implemented: navigation to another Document`
line was not the rustfmt diagnostic; the focused gate below exposed the actual
formatting diff.

The one focused diagnostic `npm.cmd run check:rustfmt` reproduced exit 1 and
reported only formatting changes in `runtime_commands.rs`. A mechanical
`cargo fmt --manifest-path src-tauri/Cargo.toml --all` changed only that file,
and the one permitted focused `npm.cmd run check:rustfmt` repeat passed with
exit 0. At that checkpoint the retained corrective full-gate observation
remained failed; the subsequently authorized final run is recorded below.

## Final authorized full-verification observation

After both the selector and rustfmt corrections, the user explicitly
authorized exactly one final `npm.cmd run verify`. A fresh unsandboxed CIM
preflight again found zero competing repository-owned processes. The command
exited 101 after 653.7 seconds:

| Gate reached | Observation |
| --- | --- |
| sidecar prerequisite | PASS. |
| transition validator | PASS; census 196/186/10, ledger 671/0. |
| `test:unit` | PASS; 118 files and 989/989 tests. |
| `test:component` | PASS; 62 files and 281/281 tests. |
| `test:architecture` | PASS; 1 file and 2/2 tests. |
| `test:integration:os` | PASS; 5 files and 44/44 tests. |
| Gemini browser adapter Playwright | PASS; 79/79 tests. |
| application Playwright | PASS. |
| `npm.cmd run check` | PASS. |
| `npm.cmd run check:rustfmt` | PASS. |
| Cargo workspace check | PASS. |
| Cargo workspace test | **FAIL**, exit 101; Cargo identified the failing binary as `-p extractum-prompt-packs --lib`. |

The final Git diff gate was not reached. The retained tool output was
truncated around the individual failure body, so this record does not invent
an exact declaration name beyond Cargo's retained package/binary diagnostic.
Per the user's instruction, there is no retry or post-failure fix.

One subsequently authorized diagnostic ran the smallest exact package gate,
`cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib`,
once. It exited 0: all 252 tests passed in 143.09 seconds. The only test
reported as running for more than 60 seconds was
`public_api_tests::cancellation_smoke_services_remain_test_only`; it eventually
passed. That test launches two nested offline Cargo checks with an isolated
temporary target directory. This is a plausible workspace/resource-sensitive
point, but the truncated full-verify failure body does not prove it was the
failed declaration. The evidence therefore classifies the exit-101 observation
as a non-reproducing workspace-context discrepancy, not as a confirmed
deterministic package defect. No source change or second package run followed.

## Last authorized full-verification observation

After the exact package gate passed, the user authorized one last complete
`npm.cmd run verify`. A fresh unsandboxed CIM preflight again found zero
competing repository-owned processes; bootstrap was not repeated. The run
exited 101 after 492.4 seconds. Transition `196/186/10` and ledger `671/0`,
unit `118/989`, component `62/281`, architecture `1/2`, OS `5/44`, Gemini
Playwright `79/79`, application Playwright, Svelte check, rustfmt, and Cargo
workspace check all passed. Cargo workspace tests again failed in the
`extractum-prompt-packs` library binary, and the final Git diff gate was not
reached.

The repeated workspace failure, contrasted with the green exact package run,
establishes a reproducible workspace-context discrepancy rather than an
isolated random package failure. The truncated workspace output still does not
identify the individual declaration, so this record does not claim a narrower
root cause. Per authorization there is no retry or post-failure fix.

Those statements describe the evidence available at that blocked checkpoint.
The later exact feature-on reproduction below established the narrower cause
before any correction was made.

## Authorized blocker diagnosis and correction

The workspace uses Cargo resolver v2. Its full test gate selects the root
application and all workspace targets; the application's dev-dependency on
`extractum-prompt-packs` enables the non-default `dev-fixtures` feature. Cargo
therefore unified `dev-fixtures` into the prompt-pack library test binary. The
isolated package command did not select that consumer edge and compiled the
same binary feature-off, which explains the apparently contradictory results.

The exact owner
`public_api_tests::cancellation_smoke_services_remain_test_only` began with an
unconditional assertion that `dev-fixtures` was disabled. It failed before
either of its external Cargo probes whenever the workspace feature graph
enabled the feature. An exact run of that owner with `--features dev-fixtures`
selected one test and reproduced the panic at that assertion.

The bounded correction removed only that invalid execution-context assertion.
It did not change production exports or feature declarations. The same owner
continues to prove all parts of SC-000474 through real external consumers and
behavior:

- a feature-off dependent crate cannot import the cancellation smoke services;
- a `dev-fixtures` dependent crate can import them;
- the real fixture seed/clear path tracks, cancels, and removes the run.

Focused GREEN evidence was sequential:

| Command | Result |
| --- | --- |
| Exact owner with `--features dev-fixtures` | PASS, 1/1. |
| Exact owner with default/feature-off package selection | PASS, 1/1. |
| `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib --no-default-features` | PASS. |
| `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets` | PASS. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets` | PASS, 252/252. |

## Successful completion verification

A fresh unsandboxed PowerShell/CIM preflight enumerated `Win32_Process` with
terminating error handling, filtered repository-owned Vitest, Playwright,
Cargo, Rust, Vite, and Chromium processes by executable path or command line,
and returned `Repository-owned competing process count: 0`. Bootstrap was not
repeated because its prerequisite remained present and no sidecar-packaging
input changed.

The user then authorized exactly one complete `npm.cmd run verify`. It exited
0 after 818.8 seconds:

| Gate reached | Observation |
| --- | --- |
| sidecar prerequisite | PASS; 45,200,439-byte Windows binary present. |
| transition validator | PASS; census 196/186/10, ledger 671/0. |
| `test:unit` | PASS; 118 files and 989/989 tests. |
| `test:component` | PASS; 62 files and 281/281 tests. |
| `test:architecture` | PASS; 1 file and 2/2 tests. |
| `test:integration:os` | PASS; 5 files and 44/44 tests. |
| Gemini browser adapter Playwright | PASS; 79/79 tests. |
| application Playwright | PASS; 3/3 tests. |
| `npm.cmd run check` | PASS; 0 errors and 0 warnings. |
| `npm.cmd run check:rustfmt` | PASS. |
| Cargo workspace check | PASS. |
| Cargo workspace test | PASS, including the feature-unified `extractum-prompt-packs` library binary. |
| final Git diff gate | PASS. |

This successful run is completion evidence, not a benchmark. No warmup,
threshold, median, paired series, Nx decision, or performance conclusion was
introduced.

## Handoff

Nx remains unselected. This slice records no threshold, warmup, median,
benchmark series, or performance conclusion. The retained wall-clock
observations are the original 253.645-second failure and the explicitly
authorized 460.9-second corrective failure, plus the explicitly authorized
653.7-second and 492.4-second failures, followed by the authorized 818.8-second
successful completion run. These observations are historical correctness
evidence and are not a timing series.

Slice 4 is the next planned slice and is ready to begin without Nx. The blocked
checkpoint remains in history at `61c435f9`; the completion handoff subject is
`docs: record slice 3c source-contract completion`.
