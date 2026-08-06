# Testing Redesign Slice 3B: Component Contracts Verification

## Checkpoint Context

- Historical status: **PASS / completed for the pre-integration Slice 3B
  checkpoint only**. The recorded migration was implemented at
  `95ccec08c64174b584c68610b61b3f4a77862409` on branch
  `codex/testing-redesign-slice-3` in
  `G:/Develop/Extractum/.worktrees/codex-slice-3`. The authoritative
  unsandboxed full gate described below passed for that historical checkpoint.
  It does **not** verify commit `b0a88a88` plus the current fix-wave working-tree
  corrections. An earlier sandboxed attempt is retained as non-authoritative
  historical infrastructure evidence.
- Environment: Windows NT 10.0.26100.0, Windows PowerShell 5.1.26100.8875,
  Node.js 24.13.1, npm 11.12.1, rustc 1.95.0, and Cargo 1.95.0.
- The focused matrix and full-gate observations in the historical sections ran
  on 2026-08-05. They must not be reused as current integration evidence.

## Migration Result

Slice 3B closed exactly 46 frozen source-contract rows across both approved
cohorts:

| Removed legacy path | Stable row range | Rows | Replacement evidence |
| --- | --- | ---: | --- |
| `src/lib/research-projects-route-contract.test.ts` | SC-000517--SC-000542 | 26 | `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`: 23 exact replacement declarations plus 14 component/route smokes, 37 declarations total. |
| `src/lib/analysis-report-canvas.test.ts` | SC-000138--SC-000157 | 20 | `src/lib/analysis-report-canvas.behavior.component.test.ts`: 16 exact declarations under `report canvas component contract` plus 2 smokes; `src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts`: 4 exact SC-000151--SC-000154 declarations under the same describe title; 22 declarations total. |

At the historical Slice 3B checkpoint, the ledger had 671 rows and fell from
483 open after Slice 3A to 437 open after Slice 3B, an exact 46-row closure.
The historical runner census was 194 filesystem candidates: 187 Vitest-owned
files and 7 Playwright-owned files. The current fix-wave transition observation
is recorded separately below as 671 rows with 436 open and a 195/188/7 census.
The two deleted legacy identities intentionally remain in durable `row.path`
fields for all 46 rows; replacement ownership is carried only by the three live
`*.behavior.component.test.ts` paths in `replacementIds` or mixed subgroups.

## Ledger Corrections and Intentional Losses

SC-000528 was corrected from the unrelated
`rule:telegram-phase-8b-authority-integrity` identity to `behavior`, owned by
the exact declaration `wires project source Library delete through the main
projects route`. SC-000529 retains its distinct list-route declaration.

The three mixed rows have no top-level disposition, replacement IDs, or
deletion reason. Their two subgroups form exact, non-overlapping partitions:

| Row | Retained behavior ordinals | Deleted ordinals and intentional loss |
| --- | --- | --- |
| SC-000536 (6 assertions) | 2--6: destructive variants and accessible action names across Workspace, Projects, and Runs. | 1: global default-button selector shape is retired because jsdom cannot expose the `+layout.svelte` selector as truthfully applied behavior. |
| SC-000537 (4 assertions) | 1--2: shared navigation-row class and selected state. | 3--4: external `:hover` decoration and applied inset box-shadow are retired because jsdom cannot expose them truthfully. |
| SC-000539 (10 assertions) | 1--9: accessible host names, directly imported column definitions, and labelled sections. | 10: the empty/loading overlay rendered inside the SVAR `Grid` is retired because SVAR internal output is not a truthful jsdom boundary. |

The three existing full-delete rows remain explicit:

- SC-000525 retires exact source order and duplicate shape assertions for the
  two toolbar actions; dedicated add-source and connect-from-Library workflows
  retain the user-visible behavior.
- SC-000538 retires reuse of one exact selected-row CSS recipe; navigation
  selection behavior remains covered.
- SC-000542 retires exact shared density utility-class names because they are
  styling implementation rather than an independent user workflow.

## Source-Reader and SVAR Boundaries

The convention guardrail passed 8/8 tests, including the production-source
reader rule. A direct audit of all three replacement suites found no `?raw`,
`readFileSync`, `readFile(`, `node:fs`, or `sourceReaderExceptions` occurrence.
Neither suite reads production source text.

The Projects replacement imports ordinary TypeScript column data and queries
the outer `DataGrid` regions by accessible name (`Project sources`, `Library
sources available to connect`, and `Prompt Pack runs grid`). It contains no
SVAR import or internal overlay assertion. SC-000539 ordinal 10 records that
boundary as an intentional loss instead of manufacturing nominal jsdom
evidence.

## Historical Slice 3B Focused Evidence

The following results belong to the pre-integration checkpoint and do not
verify `b0a88a88` or the current fix wave.

| Command | Result | Exit | Observed wall time |
| --- | --- | ---: | ---: |
| `node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts` | PASS; 1 file, 8/8 tests. Vitest reported 4.08 s. | 0 | 6.841 s |
| `node scripts/validate-testing-transition.mjs` | PASS; 194 candidates = 187 Vitest + 7 Playwright; 671 rows, 437 open. It printed existing Rust dead-code/unused warnings. | 0 | 17.833 s |
| `npm.cmd run test:component` | PASS; 22 files, 227/227 tests. Vitest reported 38.19 s. | 0 | 41.152 s |
| `npm.cmd run check` | PASS; 0 Svelte errors and 0 warnings. | 0 | 19.030 s |

## Historical Slice 3B Full-Gate Observation

The historical unsandboxed `npm.cmd run verify` observation exited 0 after
208.1 seconds. Its complete non-empty owner and gate results were:

| Gate | Result |
| --- | --- |
| sidecar prerequisite | PASS. |
| transition validator | PASS; 194 candidates = 187 Vitest + 7 Playwright; 671 rows, 437 open. |
| `test:unit` | PASS; 85 files, 808/808 tests. |
| `test:component` | PASS; 22 files, 227/227 tests. |
| `test:architecture` | PASS; 1 file, 2/2 tests. |
| `test:legacy-contract` | PASS; 73 files, 476/476 tests. |
| `test:integration:os` | PASS; 6 files, 87/87 tests. |
| `test:e2e` | PASS; 79/79 Playwright tests in 1.0 minute. |
| Svelte check | PASS; 0 errors and 0 warnings. |
| rustfmt | PASS. |
| Cargo workspace check/test | PASS; the output included the 674-test application suite plus the other workspace packages. |
| final Git diff gate | PASS. |

No automated 90-second timing warning was printed. The 208.1-second duration
is an ordinary observation; this slice adds no timing budget, warning, or
scheduler state.

## Final Integration Verify Observation

The integration was retained as one amended checkpoint. Its authoritative
unsandboxed `npm.cmd run verify` exited 0 after 383.4 seconds. Two earlier
unsandboxed attempts and one pre-format attempt are retained below as real
fail-closed observations; none is relabelled as infrastructure noise or a
successful gate.

| Current command or gate | Current observation |
| --- | --- |
| Focused Canvas component replacements | PASS: 22/22 tests across the main Canvas and route-receiver suites. |
| Focused Projects component replacement | PASS: 37/37 tests. |
| `npm.cmd run check` | PASS: 0 errors and 0 warnings. |
| `node scripts/validate-testing-transition.mjs` | PASS: 195 candidates = 188 Vitest + 7 Playwright; 671 rows, 436 open. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::tests::restore_emits_failure_event_for_each_failed_account -- --exact` | PASS: 1/1 test; 673 filtered out. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib tests::analysis_command_registration_inventory_is_exact -- --exact` | PASS: 1/1 test; 673 filtered out. |
| `node scripts/run-vitest.mjs run --project unit-node scripts/testing/repository-rules.test.ts -t "rule:telegram-crate-dependency-ownership rejects a reordered generated baseline"` | PASS: 1/1 selected test; 11 skipped. |
| First unsandboxed integration `npm.cmd run verify` | **FAIL after 46.6 s:** transition passed at 195/188/7 and 671/436; `test:unit` stopped at 811/812 because `test-conventions.test.ts` expected the retired rule ID. |
| Focused guardrail repair | PASS: `test-conventions.test.ts` 8/8 in 4.78 s. |
| Second unsandboxed integration `npm.cmd run verify` | **FAIL after 127.9 s:** unit 812/812, component 228/228, and architecture 2/2 passed; legacy stopped at 473/475 because a nested inventory macro hid flat command registrations from two frozen source-contract parsers. |
| Focused inventory repair | PASS: the two affected legacy files 19/19; `analysis_command_registration_inventory_is_exact` 1/1. Read-only Rust review: CLEAN. |
| Third unsandboxed integration `npm.cmd run verify` | **FAIL after 251.0 s at `check:rustfmt`:** transition, all Vitest owners, 79/79 Playwright tests, and Svelte check had passed. The reported diff was indentation-only in the exact-inventory test arrays. |
| `npm.cmd run check:rustfmt` after `cargo fmt` | PASS. |
| Final authoritative unsandboxed `npm.cmd run verify` | **PASS, exit 0 after 383.4 s:** transition 195/188/7 and 671/436; unit 85 files / 812 tests; component 23 / 228; architecture 1 / 2; legacy 73 / 475; OS 6 / 87; Playwright 79/79; Svelte check 0/0; rustfmt, Cargo workspace check/test, and final diff gate passed. |

## Preserved Sandboxed Infrastructure Observation

Before the authoritative run, a sandboxed `npm.cmd run verify` attempt exited
1 after 99.460 seconds in
`scripts/testing/run-observation.test.ts > runObservedCommand > does not
resolve until a real child and grandchild are dead`. The failing assertion
expected exit code 130 and `cancellationConfirmed: true`, but the sandboxed
process tree returned exit code 3 and `cancellationConfirmed: false`; both
sides reported `SIGTERM` and `termination: "signal"`.

That attempt passed the sidecar prerequisite, transition validator, unit
(85 files, 808/808 tests, 9.98 s), component (22 files, 227/227 tests, 38.65
s), and architecture (1 file, 2/2 tests, 244 ms) steps. Its legacy owner
reported 72 passed and 1 failed file, 475 passed and 1 failed test, in 17.69 s,
then the fail-fast scheduler stopped before later gates.

Diagnosis established that this was sandbox-only process-tree behavior: the
specific test failed after 15.170 seconds in the sandbox and passed in 206 ms
under the unsandboxed authoritative environment. No code, configuration, or
test changed between those observations. This result is infrastructure
evidence, not a failed authoritative gate and not a browser-flake retry.

## Architecture and Handoff

The historical Slice 3B commit range from completed Slice 3A documentation
checkpoint `384c1b2e` through implementation checkpoint `95ccec08` did not
change `package.json`, `vitest.config.ts`, or files under `scripts/`. That
statement is historical and does not describe the current integration:
The amended integration checkpoint changes Rust application code and
`scripts/testing/repository-rules.mjs` / `.test.ts` to correct SC-000355 and
SC-000649 evidence. These corrections preserve approved product behavior and
do not expand Slice 3B architecture or product scope.

Public npm commands, Vitest project ownership, the sequential fail-fast
scheduler, Cargo workspace gates, and timing architecture remain unchanged.
Nx remains unselected and deferred to the disposable Slice 2C decision gate
after Slice 4. The current ledger observation is 436 open rows. The integration
handoff is complete with the final fresh unsandboxed full gate at exit 0.

## Historical Pre-Integration Repository Audit

The Task 7 audit below was recorded for the pre-integration Slice 3B checkpoint.
It is retained as historical evidence only and must not be read as an audit of
`b0a88a88` plus the fix wave:

| Audit | Result |
| --- | --- |
| Obsolete pre-component replacement-path search across `testing`, `docs`, `scripts`, `src`, `package.json`, and `vitest.config.ts` | PASS; no matches (`rg` exit 1, the expected no-match status). |
| Structural durable-identity assertion | PASS; exactly 46 matching legacy `row.path` rows and zero rows with either legacy identity in top-level or subgroup `replacementIds` (exit 0). |
| Legacy filename search across `vitest.config.ts`, `package.json`, and `scripts` | PASS; exactly two matches: `scripts/testing/test-conventions.test.ts` lines 239 and 288, the frozen `LEGACY_TEST_FILES` expectations (exit 0). |
| `git diff --check` | Historical PASS, exit 0; current fix-wave result is reported separately at handoff. |
| `git status --short` | Historical Task 7 state only; stale for the current integration and intentionally not asserted here. |
