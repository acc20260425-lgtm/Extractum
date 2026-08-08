# Testing Redesign Slice 3B: Component Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the 46 frozen source-contract rows owned by the Projects workspace and analysis report canvas through rendered Svelte behavior, truthful direct module facts, three explicit mixed-row retirements, and no new production-source reader.

**Architecture:** The three replacement files belong to the existing Vitest `component` project by their `*.behavior.component.test.ts` suffix. Each cohort first proves that its component and route graph can mount under jsdom with bounded API fixtures. Characterization tests then preserve the exact frozen replacement titles: 23 top-level Projects declarations, 16 component-owned Canvas declarations, and four route-receiver declarations for SC-000151--SC-000154, each under `describe("report canvas component contract")`. Route-owned claims use rendered pages plus API spies; component-owned claims use rendered DOM, callbacks, or direct imports of ordinary TypeScript data. The ledger is changed and each legacy file is deleted only after all replacement declarations for that cohort pass.

**Tech Stack:** Svelte 5, Svelte Testing Library, Vitest 4.1.5, jsdom, existing SvelteKit Vitest plugin, existing source-contract ledger and transition validator.

## Global Constraints

- Implement the approved design at `docs/superpowers/specs/2026-08-05-testing-redesign-slice-3b-design.md`; do not renegotiate unrelated ledger rows.
- Nx remains deferred until after Slice 4. Do not add Nx, a scheduler, a cache, timing eligibility, performance thresholds, GitHub Actions, or branch protection.
- Do not add `?raw`, `readFileSync`, filesystem scans, regular-expression source assertions, or a `sourceReaderExceptions` entry to either replacement suite.
- Do not assert SVAR grid internals. Assert the outer `DataGrid` host, direct TypeScript column data, and ordinary Svelte behavior only.
- Keep the 23 required Projects declarations at top level. Keep the 16 main Canvas declarations and the four SC-000151--SC-000154 route-receiver declarations under the exact `report canvas component contract` describe title.
- API/listener fixtures return deterministic empty or minimal data. Every listener mock resolves to an explicit unlisten spy so teardown is observable and cannot leak.
- Do not change production behavior to make a characterization test pass. If rendered behavior cannot truthfully establish a retained ordinal, stop for design review rather than replacing it with source text.
- Keep `legacy-contract` ownership while other ledger files remain. Do not change public npm commands, Vitest project definitions, census ownership, or verify scheduling in this slice.
- On Windows use `npm.cmd`. Record durations only as ordinary observations; no command in this slice gains a timing acceptance criterion.

---

## File Map

| Path | Responsibility |
| --- | --- |
| `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts` | Component and route smokes plus the exact 23 Projects replacement declarations for the behavior-bearing rows in SC-000517--SC-000541; SC-000525 and SC-000538 remain delete-only. |
| `src/lib/analysis-report-canvas.behavior.component.test.ts` | Report-canvas/analysis-route smokes plus the 16 component-owned replacement declarations for SC-000138--SC-000150 and SC-000155--SC-000157. |
| `src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts` | The four route-owned replacement declarations for SC-000151--SC-000154. |
| `testing/source-contract-ledger.json` | Change replacement paths, correct SC-000528, encode SC-000536/537/539 as mixed rows, and preserve every other frozen field. |
| `src/lib/research-projects-route-contract.test.ts` | Delete only after all 26 Projects rows resolve or have an approved deletion resolution. |
| `src/lib/analysis-report-canvas.test.ts` | Delete only after all 20 Canvas rows resolve. |
| `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md` | Record Slice 3B completion and identify Slice 3C as the next ledger-planning step. |
| `docs/superpowers/verification/2026-08-05-testing-redesign-slice-3b-component-contracts.md` | Exact commands, outcomes, final census/ledger counts, intentional losses, and timing observations. |

## Rust Verification Loops

The approved Slice 3B product scope remains frontend component-contract
migration. The b0a88a88 integration corrections additionally touch the
`extractum` application package to preserve SC-000649 Telegram restore-failure
emission and to make the analysis command-inventory evidence inspect the real
application adapter. SC-000355 also strengthens the repository evaluator for
the generated Grammers baseline. These are integration corrections to frozen
contract evidence, not product-scope expansion.

- Affected Rust package: `extractum`.
- Exact SC-000649 RED/GREEN test:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::tests::restore_emits_failure_event_for_each_failed_account -- --exact`.
- Exact command-inventory RED/GREEN test:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib tests::analysis_command_registration_inventory_is_exact -- --exact`.
- Focused package check:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
  This proves the affected package compiles across its targets; it does not
  replace a non-empty exact behavior test.
- Package checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
  This is the package-wide integration checkpoint after the two exact tests;
  filtered or unexpectedly empty selections are not checkpoint evidence.
- End-of-slice workspace gate: `npm.cmd run verify`. It remains the required
  final integration evidence and includes the workspace Cargo gates. Use only
  canonical `src-tauri/target`; do not introduce another target directory.

## Frozen Replacement Map

The Projects file must contain these exact top-level declarations. The task split is an implementation boundary only; it does not alter replacement identity.

| Task | Rows | Exact top-level `it` titles |
| --- | --- | --- |
| Component behavior | SC-000518--SC-000524 | `renders three-zone projects workspace`; `exposes create/edit/delete and run eligibility UI`; `defaults project run dates to all synced history instead of today only`; `shows project runs in the central Runs tab`; `keeps prompt-pack run details in the Runs tab instead of duplicating them in the inspector`; `matches the Library type column in Workspace project sources`; `shows full source type labels when connecting sources from Library` |
| Component behavior | SC-000526, SC-000535--SC-000537, SC-000539--SC-000541 | `wires the project Add source dialog through the current ProjectsShell`; `keeps top command actions honest while project export is out of scope`; `keeps project action hierarchy consistent across Workspace, Projects, and Runs`; `keeps project navigation rows visually neutral until selected`; `labels project data grids for assistive technology`; `scopes repeated project refresh controls`; `clarifies the Workspace Runs taxonomy` |
| Route behavior | SC-000517, SC-000527--SC-000534 | `uses real project APIs instead of analysis source group APIs`; `passes project add-source workflow callbacks from both current project routes`; `wires project source Library delete through the main projects route`; `wires project source Library delete through the list projects route`; `keeps Remove membership-only and adds a separate Delete from Library action`; `wires the project Add source dialog through the next Projects route`; `wires Delete from Library in the next projects bulk bar`; `wires selected Workspace source syncs to the YouTube source job command`; `refreshes Workspace source content when source sync jobs finish` |

The two Canvas files contain these exact titles inside `describe("report canvas component contract")`; the main suite owns every row except SC-000151--SC-000154, which the route-receiver suite owns:

| Rows | Exact `it` titles |
| --- | --- |
| SC-000138--SC-000142 | `owns the central Report and Source modes`; `shows setup only when no run is open and report mode is selected`; `renders required opened-run header metadata`; `keeps report setup copy aware of existing saved runs`; `keeps snapshot and live source basis explicit` |
| SC-000143--SC-000147 | `passes YouTube comments and source activity callbacks through the report canvas`; `passes Telegram topic state into the source surface`; `labels source surfaces without repeating the selected workspace title`; `keeps run snapshot reading bounded and snapshot-only`; `keeps source-group run snapshots pageable through the snapshot browser` |
| Main: SC-000148--SC-000150, SC-000155--SC-000157 | `uses real chat availability in the report toolbar`; `keeps workspace tools reachable before setup report and source bodies`; `derives NotebookLM export availability from live canvas source or Telegram group`; `renders the scoped evidence return affordance above source reader headers`; `passes evidence return context and callback through the report canvas`; `passes bounded source browser mode only for live source canvas review` |
| Route receiver: SC-000151--SC-000154 | `submits NotebookLM export for either current source or current source group`; `opens NotebookLM export for Telegram source groups without the old single-source guard`; `passes transient evidence highlight tokens from route to source surfaces`; `passes focused group transcript segments from route to source surfaces` |

## Rollback Boundary

Each task commit is an independent rollback boundary. Tasks 1, 2, 4, and 5 only add passing replacement evidence while the corresponding legacy file and ledger resolution remain unchanged, so reverting one of those commits restores the preceding green checkpoint. Tasks 3 and 6 are atomic cohort cutovers: replacement completion, ledger edits, legacy-file deletion, transition validation, and owner/type checkpoints land together and are reverted together. Task 7 changes evidence and program documentation only. Do not carry a partial ledger cutover across task commits.

---

## Task 1: Prove the Projects render harness

**Files:**

- Create: `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`
- Reference: `src/lib/components/research-projects/SourcesTab.component.test.ts`
- Reference: `src/lib/projects-shared.svelte.ts`
- Reference: `src/routes/projects/+page.svelte`
- Reference: `src/routes/projects/list/+page.svelte`
- Reference: `src/routes/projects/next/+page.svelte`

**Fixture interfaces:**

- Import `type ComponentProps` from `svelte` and derive each fixture type from its component instead of copying production prop interfaces.
- `emptyProjectsState(overrides)` returns a complete `ResearchProjectsWorkflowState`: empty raw/project/source/run/catalog/job/template arrays, empty derived arrays, `selectedProjectId: null`, `selectedLibrarySourceIds: new Set()`, `loading: false`, `saving: false`, and empty status.
- Hoisted API mocks cover the direct `$lib/api/*` imports of the three pages and `ProjectsShell`: projects, library sources, source jobs, analysis runs, analysis prompt templates, LLM profiles, and opener. List/get calls resolve to empty or one-item fixtures; mutation calls are spies; event listeners resolve to named unlisten spies.
- Import production hosts statically for their real-component smokes, but import route modules dynamically after API mocks are installed. If a route-to-child assertion needs a receiving-component fixture, use a test-local `vi.doMock` followed by a fresh dynamic route import, then `vi.resetModules`/`vi.doUnmock` before any later real-component render. Do not let a route fixture replace the host used by the component smoke or component-owned replacement tests.
- Reset mocks and DOM after every test. Do not globally mock `$app/environment`; first exercise the real SvelteKit plugin resolution.

- [ ] **Step 1: Establish the focused RED boundary**

Run before creating the file:

`node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`

Expected: FAIL/no test files because the approved replacement does not exist.

- [ ] **Step 2: Add the typed fixture and API-mock foundation**

Create the file with hoisted mocks, representative Project/Source/Run data builders, `emptyProjectsState`, listener cleanup spies, and a shared `afterEach` that runs Testing Library cleanup and restores mocks. Prefer semantic queries and callback spies; do not create a custom source scanner.

- [ ] **Step 3: Add 11 component smoke declarations**

Render each previously unproved host with the smallest truthful props: `ProjectsShell`, `ProjectRail`, `ProjectInspector`, `ProjectRunsTab`, `ProjectRunsScreen`, `ProjectRunDialog`, `ConnectFromLibrary`, `ProjectSourceSummary`, `TopCommandBar`, `ProjectWorkspace`, and `YoutubeSummaryRunsPanel`. Use the exact distinct titles `smoke renders ProjectsShell`, `smoke renders ProjectRail`, `smoke renders ProjectInspector`, `smoke renders ProjectRunsTab`, `smoke renders ProjectRunsScreen`, `smoke renders ProjectRunDialog`, `smoke renders ConnectFromLibrary`, `smoke renders ProjectSourceSummary`, `smoke renders TopCommandBar`, `smoke renders ProjectWorkspace`, and `smoke renders YoutubeSummaryRunsPanel`. Keep them at top level so none gains a `describe` prefix or collides with a frozen replacement identity.

For `ProjectsShell`, await its LLM-profile on-mount work and prove the route graph resolves `$app/environment`. If that one smoke fails specifically on module resolution, add a file-local `vi.mock("$app/environment", () => ({ browser: false }))`, record why in the test, and rerun; do not add a repository-wide alias.

- [ ] **Step 4: Add three route smoke declarations**

Render the main, list, and next Projects page components in exact tests `smoke renders main Projects route`, `smoke renders list Projects route`, and `smoke renders next Projects route`. Await initial API calls, assert one stable page landmark, unmount, and assert each registered event-listener unlisten callback. The route smokes prove harness viability only; they are not replacement declarations.

- [ ] **Step 5: Run focused GREEN and guardrails**

Run:

`node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`

Expected: PASS with 14 non-empty smoke tests.

Run:

`node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts`

Expected: PASS; the new file owns no raw production-source read.

- [ ] **Step 6: Commit the harness checkpoint**

Run `git diff --check`, inspect `git status --short`, then commit:

Run: `git add src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`

Run: `git commit -m "test: establish projects component harness"`

---

## Task 2: Replace Projects component-owned contracts

**Files:**

- Modify: `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`
- Reference: `src/lib/research-projects-route-contract.test.ts`
- Reference: `src/lib/ui/research-projects-project-source-grid.ts`
- Reference: `src/lib/components/extractum-ui/DataGrid.svelte`

- [ ] **Step 1: Add the 14 exact component-owned declarations**

Add the Task 2 titles from the Frozen Replacement Map as top-level `it` declarations. Translate each frozen invariant into observable behavior:

- workspace zones, project navigation, action buttons, dialog flows, run ranges/taxonomy, repeated refresh scopes, and accessible action names use rendered components and user interactions;
- callback ownership uses spies with exact payload assertions;
- source-grid column identity and eligibility use direct imports from `research-projects-project-source-grid.ts`;
- accessible grid names query the outer `role="region"` host and never the SVAR child grid;
- SC-000536 retains only ordinals 2--6, SC-000537 only ordinals 1--2, and SC-000539 only ordinals 1--9. Do not recreate their deleted CSS/SVAR assertions.

- [ ] **Step 2: Run the focused characterization suite**

Run:

`node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`

Expected: PASS with all 14 component replacement titles plus the smoke tests. A production defect is fixed only after reporting the mismatch against the approved invariant; do not weaken the test to mirror implementation.

- [ ] **Step 3: Demonstrate that the cohort is not yet closable**

Run:

`node scripts/validate-testing-transition.mjs`

Expected: PASS for the repository, but SC-000517--SC-000542 remain open because the legacy file still exists and nine route replacements are not yet present. This is a migration checkpoint, not completion evidence.

- [ ] **Step 4: Commit the component-behavior checkpoint**

Run `git diff --check`, inspect the diff for exact top-level titles, then commit:

Run: `git add src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`

Run: `git commit -m "test: preserve projects component behavior"`

---

## Task 3: Replace Projects route contracts and cut over the cohort

**Files:**

- Modify: `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`
- Modify: `testing/source-contract-ledger.json`
- Delete: `src/lib/research-projects-route-contract.test.ts`
- Reference: `scripts/testing/extract-source-contract-ledger.mjs`

- [ ] **Step 1: Add the nine exact route-owned declarations**

Add the Task 3 titles from the Frozen Replacement Map as top-level `it` declarations. Use rendered pages and spies to prove:

- the main route calls Projects APIs and never substitutes analysis-source-group APIs for project ownership;
- main and list routes pass add-source callbacks through their rendered `ProjectsShell` boundary;
- the main and list delete-from-Library actions invoke the library-deletion command with their distinct route scenarios;
- membership removal and library deletion remain separate operations;
- the next route wires add-source and bulk Delete from Library;
- selected Workspace source sync reaches `syncYoutubeSource` with the selected source identity;
- source-job completion triggers a bounded Workspace refresh and listener teardown.

Use a small receiving-component mock only where the assertion concerns route-to-child prop delivery. The mock must expose the callback as a real button/action and the test must invoke it; inspecting component source or a captured prop object alone is insufficient route behavior.

- [ ] **Step 2: Run the complete Projects replacement suite**

Run:

`node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts`

Expected: PASS with 23 exact replacement declarations plus 14 smoke declarations.

- [ ] **Step 3: Apply the exact ledger correction set**

In `testing/source-contract-ledger.json`:

- change every Projects replacement path to `src/lib/components/research-projects/projects-workspace.behavior.component.test.ts` without changing the declaration title;
- change SC-000528 to `disposition: "behavior"` with replacement title `wires project source Library delete through the main projects route`;
- convert SC-000536 to mixed. Its delete subgroup owns `[1]`, invariant `Global default button styling is represented as stylesheet implementation rather than runtime component behavior.`, and deletion reason `jsdom cannot expose the global +layout.svelte selector as truthfully applied behavior.` Its behavior subgroup owns `[2,3,4,5,6]`, invariant `Destructive project actions retain destructive variants and accessible action names across Workspace, Projects, and Runs.`, and the existing replacement;
- convert SC-000537 to mixed. Its behavior subgroup owns `[1,2]`, invariant `Project navigation rows expose the shared row class and selected state only for the selected project.`, and the existing replacement. Its delete subgroup owns `[3,4]`, invariant `Project-row hover decoration is external stylesheet implementation shape.`, and deletion reason `jsdom cannot truthfully expose the external :hover selector or its applied inset box-shadow.`;
- convert SC-000539 to mixed. Its behavior subgroup owns `[1,2,3,4,5,6,7,8,9]`, invariant `Project data surfaces retain their accessible host names, direct column definitions, and labelled sections.`, and the existing replacement. Its delete subgroup owns `[10]`, invariant `Project-runs empty and loading overlays are rendered by the internal SVAR grid.`, and deletion reason `SVAR Grid internal overlay output is not a truthful jsdom observation boundary.`;
- remove top-level `disposition`, `replacementIds`, and `deletionReason` from those three mixed rows; retain stable IDs, hashes, lineage, assertion counts, row invariants, and exact original titles;
- leave SC-000525, SC-000538, and SC-000542 as their already-reasoned delete rows.

- [ ] **Step 4: Delete the legacy Projects contract and validate closure**

Delete `src/lib/research-projects-route-contract.test.ts`, then run:

`node scripts/validate-testing-transition.mjs`

Expected: PASS; all SC-000517--SC-000542 rows report closed and the open-row total falls from 483 to 457. The census remains exhaustive; one legacy Vitest file was replaced by one component Vitest file.

- [ ] **Step 5: Run the owner checkpoint**

Run:

`npm.cmd run test:component`

Expected: PASS, including the complete Projects replacement suite.

Run:

`npm.cmd run check`

Expected: PASS with no Svelte or TypeScript errors.

- [ ] **Step 6: Commit the Projects cutover**

Run `git diff --check`, inspect `git status --short`, then commit:

Run: `git add src/lib/components/research-projects/projects-workspace.behavior.component.test.ts testing/source-contract-ledger.json src/lib/research-projects-route-contract.test.ts`

Run: `git commit -m "test: migrate projects source contracts"`

---

## Task 4: Prove the report-canvas and analysis-route harness

**Files:**

- Create: `src/lib/analysis-report-canvas.behavior.component.test.ts`
- Create: `src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts`
- Reference: `src/lib/components/analysis/report-canvas.svelte`
- Reference: `src/routes/analysis/+page.svelte`
- Reference: `src/lib/analysis-report-canvas.test.ts`

**Fixture interfaces:**

- Derive `ReportCanvasProps` with `ComponentProps<typeof ReportCanvas>` and implement `reportCanvasProps(overrides)` once; do not duplicate the component's large inline prop type.
- Mock each direct analysis-route `$lib/api/*` dependency with stable empty/default values. Mutation functions are spies. Every analysis/source/NotebookLM listener resolves to a named unlisten spy.
- Child-component mocks are allowed only to prove route prop/callback delivery for SC-000151--SC-000154; canvas-owned rows render the real `ReportCanvas` and its ordinary children.
- Keep the real `ReportCanvas` statically available for its smoke and 16 component-owned declarations. Dynamically import the analysis route after its API mocks; route tests that substitute a receiving canvas use test-local `vi.doMock`, a fresh route import, and `vi.resetModules`/`vi.doUnmock` cleanup so the substitute cannot leak into real-canvas tests.

- [ ] **Step 1: Establish the focused RED boundary**

Run before creating the file:

`node scripts/run-vitest.mjs run --project component src/lib/analysis-report-canvas.behavior.component.test.ts`

Expected: FAIL/no test files because the approved replacement does not exist.

- [ ] **Step 2: Add the typed canvas fixture and minimal canvas smoke**

Create the file, render the real `ReportCanvas` with `reportCanvasProps()`, assert one stable canvas landmark, unmount, and clean up. Put the exact test `smoke renders report canvas` under `describe("report canvas render smoke")`, not under the frozen replacement describe.

- [ ] **Step 3: Add the analysis route mocks and route smoke**

Render `src/routes/analysis/+page.svelte` in the exact top-level test `smoke renders analysis route`, await its initial data/listener setup, assert one stable page landmark, unmount, and assert every registered unlisten callback. Keep API returns minimal and explicit so an unexpected call fails loudly rather than receiving a permissive proxy mock.

- [ ] **Step 4: Run focused GREEN and guardrails**

Run:

`node scripts/run-vitest.mjs run --project component src/lib/analysis-report-canvas.behavior.component.test.ts`

Expected: PASS with two non-empty smoke tests.

Run:

`node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts`

Expected: PASS; the new suite introduces no production-source reader.

- [ ] **Step 5: Commit the harness checkpoint**

Run `git diff --check`, inspect `git status --short`, then commit:

Run: `git add src/lib/analysis-report-canvas.behavior.component.test.ts`

Run: `git commit -m "test: establish report canvas harness"`

---

## Task 5: Replace Canvas component-owned contracts

**Files:**

- Modify: `src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts`
- Reference: `src/lib/analysis-report-canvas.test.ts`
- Reference: `src/lib/components/analysis/report-canvas.svelte`

- [ ] **Step 1: Create the exact replacement describe block**

Add the exact `describe("report canvas component contract")` block and the 16 exact declarations for SC-000138--SC-000150 and SC-000155--SC-000157. Do not nest another describe inside it.

- [ ] **Step 2: Implement observable component behavior**

Use real canvas rendering, semantic queries, callback spies, and rerender where appropriate to prove modes, setup/opened-run state, metadata, snapshot/live source basis, callback pass-through, Telegram topic state, source labels, pagination bounds, chat availability, tool reachability, NotebookLM availability, evidence-return affordance/context, and bounded live-source browser mode. Do not use analysis route mocks for facts owned entirely by the canvas.

- [ ] **Step 3: Run the focused characterization suite**

Run:

`node scripts/run-vitest.mjs run --project component src/lib/analysis-report-canvas.behavior.component.test.ts`

Expected: PASS with 16 exact component replacement declarations and two smoke tests.

- [ ] **Step 4: Confirm the cohort remains intentionally open**

Run:

`node scripts/validate-testing-transition.mjs`

Expected: PASS for the repository, while SC-000138--SC-000157 remain open because the legacy file still exists and SC-000151--SC-000154 are not yet implemented.

- [ ] **Step 5: Commit the component-behavior checkpoint**

Run `git diff --check`, inspect the exact describe/title hierarchy, then commit:

Run: `git add src/lib/analysis-report-canvas.behavior.component.test.ts`

Run: `git commit -m "test: preserve report canvas behavior"`

---

## Task 6: Replace analysis-route contracts and cut over the Canvas cohort

**Files:**

- Modify: `src/lib/analysis-report-canvas.behavior.component.test.ts`
- Modify: `testing/source-contract-ledger.json`
- Delete: `src/lib/analysis-report-canvas.test.ts`

- [ ] **Step 1: Add the four exact route-owned declarations**

In the dedicated route-receiver suite, add SC-000151--SC-000154 under the exact `report canvas component contract` describe. Render the analysis route with an intentionally narrow `ReportCanvas` receiving-component fixture and interact through its exposed controls to prove:

- NotebookLM export submits for either a current source or current Telegram source group with the exact API request;
- opening a Telegram-group export is not blocked by the old single-source guard;
- transient evidence-highlight tokens reach the source surface;
- focused group transcript segments reach the source surface.

The receiving fixture must render received values or expose callbacks as user actions. Do not close these rows by inspecting a mock call's opaque props without exercising the route behavior.

- [ ] **Step 2: Run the complete Canvas replacement suite**

Run:

`node scripts/run-vitest.mjs run --project component src/lib/analysis-report-canvas.behavior.component.test.ts src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts`

Expected: PASS with 16 exact declarations in the main Canvas suite, four exact SC-000151--SC-000154 declarations in the route-receiver suite, and two main-suite smoke tests.

- [ ] **Step 3: Update replacement paths and delete the legacy file**

Change SC-000138--SC-000150 and SC-000155--SC-000157 replacement paths to `src/lib/analysis-report-canvas.behavior.component.test.ts`; change SC-000151--SC-000154 to `src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts`, preserving every exact full title and all other ledger metadata. Delete `src/lib/analysis-report-canvas.test.ts`.

- [ ] **Step 4: Validate cohort closure**

Run:

`node scripts/validate-testing-transition.mjs`

Expected: PASS; SC-000138--SC-000157 report closed and the open-row total falls from 457 to 437. Expected census totals remain 194 candidates, 187 Vitest-owned files, and 7 Playwright-owned files.

- [ ] **Step 5: Run the owner and type checkpoints**

Run:

`npm.cmd run test:component`

Expected: PASS with all three Slice 3B replacement suites.

Run:

`npm.cmd run check`

Expected: PASS.

- [ ] **Step 6: Commit the Canvas cutover**

Run `git diff --check`, inspect `git status --short`, then commit:

Run: `git add src/lib/analysis-report-canvas.behavior.component.test.ts src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts testing/source-contract-ledger.json src/lib/analysis-report-canvas.test.ts`

Run: `git commit -m "test: migrate report canvas source contracts"`

---

## Task 7: Complete Slice 3B evidence and program handoff

**Files:**

- Modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`
- Create: `docs/superpowers/verification/2026-08-05-testing-redesign-slice-3b-component-contracts.md`

- [ ] **Step 1: Run the focused guardrail matrix**

Run in this order:

1. `node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts`
2. `node scripts/validate-testing-transition.mjs`
3. `npm.cmd run test:component`
4. `npm.cmd run check`

Expected: all PASS. Record exact test counts, ledger/census totals, exit codes, and observed durations. The transition result must show 671 total rows and 437 open rows; if repository state changed independently, explain the verified delta of exactly 46 closed Slice 3B rows instead of silently editing the expected number.

- [ ] **Step 2: Run the authoritative full gate**

Run:

`npm.cmd run verify`

Expected: PASS. This is the end-of-slice gate, including unchanged Rust workspace checks. A known flaky browser-gate retry, if needed, is recorded as a second independent observation; do not overwrite or conceal the first result.

- [ ] **Step 3: Write the verification record**

Create the verification document with:

- commit/worktree identity and environment;
- the two migrated legacy paths and exact row ranges;
- SC-000528 correction and the three mixed-row partitions;
- explicit intentional losses for SC-000536 ordinal 1, SC-000537 ordinals 3--4, and SC-000539 ordinal 10, plus the existing full delete rows SC-000525/538/542;
- evidence that neither replacement uses a production-source reader or SVAR internal assertion;
- focused and full command results, counts, exit codes, and observed durations;
- final ledger/census totals and confirmation that public commands and scheduler semantics did not change.

- [ ] **Step 4: Update the program index**

Mark Slice 3B as the latest completed detailed-plan checkpoint, record that it closed 46 rows and left 437 open, and identify Slice 3C planning from the live ledger as next. Preserve the explicit statement that Slice 2C/Nx remains deferred until after Slice 4.

- [ ] **Step 5: Perform the final repository audit**

Run:

`rg -n "analysis-report-canvas\.behavior\.test|projects-workspace\.behavior\.test" testing docs scripts src package.json vitest.config.ts`

Expected: no matches. The pre-component replacement paths must disappear from every `replacementIds` entry and live document/configuration.

Then load `testing/source-contract-ledger.json` in PowerShell and assert the durable legacy identities structurally:

```powershell
$ledger = Get-Content testing/source-contract-ledger.json -Raw | ConvertFrom-Json
$legacyPaths = @(
  'src/lib/research-projects-route-contract.test.ts',
  'src/lib/analysis-report-canvas.test.ts'
)
$legacyRows = @($ledger.rows | Where-Object { $_.path -in $legacyPaths })
if ($legacyRows.Count -ne 46) { throw "Expected 46 durable Slice 3B legacy path rows, found $($legacyRows.Count)" }
$badReplacement = @($ledger.rows | Where-Object {
  $replacementIds = @($_.replacementIds) + @($_.subgroups | ForEach-Object { $_.replacementIds })
  ($replacementIds -join ' ') -match 'research-projects-route-contract\.test|analysis-report-canvas\.test'
})
if ($badReplacement.Count -ne 0) { throw "Legacy paths remain in replacementIds" }
```

Expected: PASS. The 46 `row.path` values intentionally remain after their files are deleted; their absence from the filesystem is how the transition validator recognizes cutover.

Finally run:

`rg -n "research-projects-route-contract\.test|analysis-report-canvas\.test" vitest.config.ts package.json scripts`

Expected: exactly the two frozen `LEGACY_TEST_FILES` expectation entries in `scripts/testing/test-conventions.test.ts`, and no configuration, package-script, replacement, or other script references. `vitest.config.ts` intentionally derives `LEGACY_TEST_FILES` from all durable ledger paths, including closed rows; the convention snapshot therefore retains these two literal identities just as it retains deleted Slice 3A identities.

Run `git diff --check` and `git status --short`. Expected: only the program-index and Slice 3B verification-document changes remain after the implementation commits.

- [ ] **Step 6: Commit the documentation checkpoint**

Commit:

Run: `git add docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md docs/superpowers/verification/2026-08-05-testing-redesign-slice-3b-component-contracts.md`

Run: `git commit -m "docs: record testing redesign slice 3b"`

Do not push or merge. Hand the branch back with the final commit IDs, clean/dirty status, full verification result, and the next approved planning boundary.
