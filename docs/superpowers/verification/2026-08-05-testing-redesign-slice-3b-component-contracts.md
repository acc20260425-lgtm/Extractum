# Testing Redesign Slice 3B: Component Contracts Verification

## Checkpoint Context

- Status: **PASS / completed**. The Slice 3B migration is implemented at
  `95ccec08c64174b584c68610b61b3f4a77862409` on branch
  `codex/testing-redesign-slice-3` in
  `G:/Develop/Extractum/.worktrees/codex-slice-3`. The authoritative
  unsandboxed full gate passed. An earlier sandboxed attempt is retained below
  as non-authoritative infrastructure evidence rather than substituted for or
  concealed by the authoritative result.
- Environment: Windows NT 10.0.26100.0, Windows PowerShell 5.1.26100.8875,
  Node.js 24.13.1, npm 11.12.1, rustc 1.95.0, and Cargo 1.95.0.
- The focused matrix and full-gate observations ran on 2026-08-05. No
  Cargo, browser, or OS command overlapped another command.

## Migration Result

Slice 3B closed exactly 46 frozen source-contract rows across both approved
cohorts:

| Removed legacy path | Stable row range | Rows | Replacement evidence |
| --- | --- | ---: | --- |
| `src/lib/research-projects-route-contract.test.ts` | SC-000517--SC-000542 | 26 | `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`: 23 exact replacement declarations plus 14 component/route smokes, 37 declarations total. |
| `src/lib/analysis-report-canvas.test.ts` | SC-000138--SC-000157 | 20 | `src/lib/analysis-report-canvas.behavior.component.test.ts`: 20 exact declarations under `report canvas component contract` plus 2 smokes, 22 declarations total. |

The live ledger remains 671 rows and fell from 483 open after Slice 3A to 437
open after Slice 3B, an exact 46-row closure. The runner census remains 194
filesystem candidates: 187 Vitest-owned files and 7 Playwright-owned files.
The two deleted legacy identities intentionally remain in durable `row.path`
fields for all 46 rows; replacement ownership is carried only by the two live
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
reader rule. A direct audit of both replacement suites found no `?raw`,
`readFileSync`, `readFile(`, `node:fs`, or `sourceReaderExceptions` occurrence.
Neither suite reads production source text.

The Projects replacement imports ordinary TypeScript column data and queries
the outer `DataGrid` regions by accessible name (`Project sources`, `Library
sources available to connect`, and `Prompt Pack runs grid`). It contains no
SVAR import or internal overlay assertion. SC-000539 ordinal 10 records that
boundary as an intentional loss instead of manufacturing nominal jsdom
evidence.

## Focused Verification Evidence

| Command | Result | Exit | Observed wall time |
| --- | --- | ---: | ---: |
| `node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts` | PASS; 1 file, 8/8 tests. Vitest reported 4.08 s. | 0 | 6.841 s |
| `node scripts/validate-testing-transition.mjs` | PASS; 194 candidates = 187 Vitest + 7 Playwright; 671 rows, 437 open. It printed existing Rust dead-code/unused warnings. | 0 | 17.833 s |
| `npm.cmd run test:component` | PASS; 22 files, 227/227 tests. Vitest reported 38.19 s. | 0 | 41.152 s |
| `npm.cmd run check` | PASS; 0 Svelte errors and 0 warnings. | 0 | 19.030 s |

## Authoritative Full-Gate Observation

The required unsandboxed `npm.cmd run verify` observation exited 0 after 208.1
seconds. Its complete non-empty owner and gate results were:

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

The Slice 3B commit range from the completed Slice 3A documentation checkpoint
`384c1b2e` through implementation checkpoint `95ccec08` changes no
`package.json`, `vitest.config.ts`, or file under `scripts/`. Public npm
commands, Vitest project ownership, the sequential fail-fast scheduler, Cargo
workspace gates, and timing architecture are unchanged. Nx remains unselected
and deferred to the disposable Slice 2C decision gate after Slice 4.

The migration inventory is closed at 437 open rows, and the authoritative full
gate is green. Slice 3C planning must begin by re-reading that live ledger.

## Final Repository Audit

The Task 7 final audit is recorded after the documentation update:

| Audit | Result |
| --- | --- |
| Obsolete pre-component replacement-path search across `testing`, `docs`, `scripts`, `src`, `package.json`, and `vitest.config.ts` | PASS; no matches (`rg` exit 1, the expected no-match status). |
| Structural durable-identity assertion | PASS; exactly 46 matching legacy `row.path` rows and zero rows with either legacy identity in top-level or subgroup `replacementIds` (exit 0). |
| Legacy filename search across `vitest.config.ts`, `package.json`, and `scripts` | PASS; exactly two matches: `scripts/testing/test-conventions.test.ts` lines 239 and 288, the frozen `LEGACY_TEST_FILES` expectations (exit 0). |
| `git diff --check` | PASS, exit 0. |
| `git status --short` | Exactly the modified program index and this new verification record are tracked Task 7 changes. |
