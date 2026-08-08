# Testing Redesign Slice 3C Source-Contract Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all 436 remaining source-contract rows, remove all 86 legacy files and the empty legacy runner, and hand a zero-open-row inventory to Slice 4.

**Architecture:** Implement the exact owners frozen in `testing/source-contract-redisposition-review.json`, but cut over by the 84 connected legacy-to-replacement components. Additive owner commits may precede cutover; a cutover deletes only complete components. Wave 2 is split into five bounded integration batches, the browser track starts early, and only the final acceptance runs complete `verify`.

**Tech Stack:** Node.js ESM, TypeScript, SvelteKit, Vitest 4.1.5, Testing Library/Svelte with jsdom, Playwright 1.61, Rust 2021/Cargo, Windows PowerShell, the source-contract ledger and transition validator.

## Global Constraints

- Implement only the classes, dispositions, paths, declaration titles, invariants, deletion reasons, and criticality decisions frozen in `testing/source-contract-redisposition-review.json` and `testing/source-contract-ledger.json`.
- The one approved ownership correction is exact: `src/lib/components/**/*.behavior.test.ts` belongs to `component`; Node becomes 179 rows/1,109 ordinals and jsdom becomes 90 rows/598 ordinals. Do not rewrite accepted blind-review bytes.
- Do not use `?raw`, `readFileSync`, or another direct production-source reader in a replacement test. Do not add a `sourceReaderExceptions` entry.
- A small behavior-neutral pure-function or adapter extraction is allowed only when the frozen behavior has no truthful existing seam. Stop the connected component if path, title, mechanism, invariant, disposition, or criticality would need to change.
- Do not add Nx, a migration scheduler, a second ledger, a subset transition validator, timing thresholds, warmups, medians, or benchmark artifacts.
- Use `npm.cmd` for repository npm commands on Windows. Run at most two focused frontend owner commands concurrently.
- Process-spawning replacement files enter `OS_INTEGRATION_FILES` in their additive commit. Their legacy paths leave that list only in the component cutover commit.
- A correctness failure is retained and fixed. A final `verify` retry is allowed only for a demonstrated spawn, sandbox, or process-infrastructure failure, and both observations must be recorded.
- The commit that deletes the factually last legacy file must also remove the `legacy-contract` project, census owner, npm script, and verify gate.
- Nx remains deferred until Slice 4 has committed.

---

## Frozen Interfaces and Work Graph

The decision artifact is the only row-to-owner map. Before changing a target, print its exact rows with this read-only pattern, replacing the path literal with the task's exact target path:

```powershell
@'
const fs = require("fs");
const artifact = JSON.parse(fs.readFileSync("testing/source-contract-redisposition-review.json", "utf8"));
const ledger = JSON.parse(fs.readFileSync("testing/source-contract-ledger.json", "utf8"));
const rows = new Map(ledger.rows.map((row) => [row.id, row]));
const targetPath = "REPLACE_WITH_EXACT_TARGET_PATH";
for (const decision of artifact.decisions) {
  const ids = decision.resolution?.replacementIds ?? [];
  if (!ids.some((id) => id.includes(`:${targetPath}#`) || id.endsWith(`:${targetPath}`))) continue;
  const row = rows.get(decision.id);
  console.log(JSON.stringify({
    id: decision.id,
    invariant: row.invariant,
    assertionCount: row.assertionCount,
    replacementIds: ids,
    criticalityRef: decision.criticalityRef ?? null,
  }, null, 2));
}
'@ | node -
```

For every target file, the RED test declarations use the complete title after `#` verbatim. A declaration may contain multiple observable assertions, but the plan does not create one process cycle per ledger row.

Cutover distribution:

| Bucket | Components | Legacy files | Rows | Integration batches |
| --- | ---: | ---: | ---: | ---: |
| Wave 0 | 23 | 23 | 70 | 1 |
| Wave 1 | 14 | 14 | 29 | 1 |
| Wave 2A | included below | 7 | 42 | 1 |
| Wave 2B | included below | 7 | 50 | 1 |
| Wave 2C | included below | 8 | 58 | 1 |
| Wave 2D | included below | 7 | 56 | 1 |
| Wave 2E | included below | 8 | 65 | 1 |
| Wave 3 | 9 | 9 | 55 | 1 |
| Browser | 3 | 3 | 11 | 1 |
| **Total** | **84** | **86** | **436** | **9** |

The batch row counts are closure counts, including accepted delete rows in mixed files. No batch splits a connected component.

## File Map

Permanent runner and evidence files:

- Modify `vitest.config.ts`: add component behavior ownership; replace legacy exclusions only at final cutover.
- Modify `scripts/testing/test-conventions.test.ts`: guard component behavior ownership and source-reader policy.
- Modify `scripts/testing/source-contract-redisposition-review.mjs` and its test: separate execution forecast classification from historical packet classification.
- Modify `testing/source-contract-redisposition-review.json` and the pre-Slice 3C verification record: forecast-only Node/jsdom correction.
- Create `e2e/playwright.config.ts` and `e2e/fixtures/tauri.ts`: isolated application Playwright owner and narrow IPC setup.
- Modify `testing/runner-census.json`, `package.json`, `scripts/verify.mjs`, `scripts/verify.test.ts`: add application E2E; remove legacy ownership only at the final cutover.
- Create `docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md`: final evidence and Slice 4 handoff.
- Modify `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`: record Slice 3 completion only after final verification.

All replacement and legacy paths are listed in their owning tasks below.

## Cutover Protocol

Every behavior task uses this order:

1. Confirm the exact target path/title map from the artifact.
2. Add the complete target-file RED and run the exact owner selection; an empty selection is a failure.
3. Implement only the observable seam and run the target-file GREEN.
4. If the component is not yet complete, commit the additive owner without deleting the legacy file.
5. Before a ready batch cutover, stage all targets and legacy deletions, run the complete transition validator once, and inspect its exact open-row delta.
6. If that cutover makes the filesystem legacy inventory empty, remove the legacy runner in the same staged diff before validation and commit.
7. Run `git diff --check`, one read-only integration review, one combined fix wave if required, and commit the bounded batch.

Do not edit resolution metadata during ordinary cutover. A component closes because its legacy declaration disappears and its frozen replacement already resolves.

## Rust Verification Loops

Affected packages are `extractum`, `extractum-analysis`, `extractum-llm`, and `extractum-prompt-packs`.

- Exact RED/GREEN: `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact`.
- If the exact name is not collected, first run `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib -- --list` and reject a zero-test selection.
- Focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`.
- Package checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`.
- Producer surface checks required by a test-support seam use `cargo check --manifest-path src-tauri/Cargo.toml -p <producer> --lib --no-default-features`.
- End of slice: the one final `npm.cmd run verify` supplies workspace `cargo check --workspace --all-targets` and `cargo test --workspace --all-targets`.

Cargo package commands and `node scripts/validate-testing-transition.mjs` never run concurrently because the validator lists Cargo evidence.

---

### Task 1: Correct component-project ownership and execution forecast

**Files:**
- Modify: `vitest.config.ts`
- Modify: `scripts/testing/test-conventions.test.ts`
- Modify: `scripts/testing/source-contract-redisposition-review.mjs`
- Modify: `scripts/testing/source-contract-redisposition-review.test.ts`
- Modify: `testing/source-contract-redisposition-review.json`
- Modify: `docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md`

**Interfaces:**
- Produces `COMPONENT_BEHAVIOR_PATTERN = "src/lib/components/**/*.behavior.test.ts"` in both component include and unit exclusion.
- Preserves historical packet `mechanism` bytes while deriving the execution forecast from the new component-directory rule.
- Produces forecast Node `179/1109`, jsdom `90/598`, Cargo `18/156`, Playwright `3/21`, proposed-new-jsdom `0/0`.

- [ ] **Step 1: Add focused RED assertions**

In `scripts/testing/test-conventions.test.ts`, assert that a component behavior target is owned only by `component`, while the two existing Telegram behavior files remain owned only by `unit-node`. In the carrier test, mutate the execution classifier input so `src/lib/components/example.behavior.test.ts` must report `jsdom` and `src/lib/example.behavior.test.ts` must report `node`; assert the exact forecast counts above.

- [ ] **Step 2: Run the RED**

Run:

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts scripts/testing/source-contract-redisposition-review.test.ts -t "component behavior ownership|execution owner forecast"
```

Expected: FAIL because component behavior paths still fall through to `unit-node` and the carrier still classifies only `.component.test.ts` as jsdom.

- [ ] **Step 3: Implement the ownership correction**

In `vitest.config.ts`, add:

```ts
const COMPONENT_BEHAVIOR_PATTERN = "src/lib/components/**/*.behavior.test.ts";
```

Include it in the `component` project's `include`, exclude it from `unit-node`, and retain the existing setup file and `autoCleanup: false`. In the carrier, keep `mechanismForOwner` unchanged for content-addressed historical packet validation and add a separate execution classifier that treats both `.component.test.ts` and `src/lib/components/**/*.behavior.test.ts` as jsdom. Use only the execution classifier in `futureOwnerSummary`.

Update only the artifact forecast fields and append a dated forecast-correction note to the pre-Slice 3C verification document. Do not change decisions, review packet/output paths, hashes, or blind digests.

- [ ] **Step 4: Run focused GREEN and drift checks**

Run:

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts scripts/testing/source-contract-redisposition-review.test.ts
node scripts/testing/source-contract-redisposition-review.mjs check
node scripts/validate-testing-transition.mjs
git diff --check
```

Expected: all tests pass; carrier changed paths are empty; transition remains 671 rows/436 open; census remains 196 candidates, 189 Vitest, 7 Playwright because no replacement file exists yet.

- [ ] **Step 5: Commit**

```powershell
git add vitest.config.ts scripts/testing/test-conventions.test.ts scripts/testing/source-contract-redisposition-review.mjs scripts/testing/source-contract-redisposition-review.test.ts testing/source-contract-redisposition-review.json docs/superpowers/verification/2026-08-06-testing-redesign-pre-slice-3c-redisposition.md
git commit -m "test: correct slice 3c component ownership"
```

### Task 2: Close Wave 0 without TDD

**Files — delete exactly:**

```text
src/lib/analysis-legacy-surfaces-cleanup.test.ts
src/lib/analysis-migration-fixture-contract.test.ts
src/lib/analysis-state-legacy-selection-cleanup.test.ts
src/lib/api/apalis-jobs.test.ts
src/lib/api/diagnostics.test.ts
src/lib/components/research-projects/OptionsPanel.component.test.ts
src/lib/components/research-projects/ProjectRailPanel.component.test.ts
src/lib/components/research-projects/ProjectRow.component.test.ts
src/lib/components/research-projects/ProjectTabs.component.test.ts
src/lib/components/research-projects/SourcesBulkBar.component.test.ts
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/development-loop-performance-contract.test.ts
src/lib/focused-rust-loop-contract.test.ts
src/lib/gemini-browser-crate-boundary-contract.test.ts
src/lib/media-metadata-core-contract.test.ts
src/lib/prompt-pack-completion-transport-contract.test.ts
src/lib/prompt-pack-run-store-contract.test.ts
src/lib/prompt-pack-stage-execution-contract.test.ts
src/lib/prompt-pack-stage-request-policy-contract.test.ts
src/lib/research-projects-foundation-contract.test.ts
src/lib/research-projects-lucide-import-contract.test.ts
src/lib/rust-workspace-core-contract.test.ts
src/lib/youtube-summary-smoke-fixture-contract.test.ts
```

**Interfaces:** Closes exactly 23 components, 23 files, and 70 delete rows. Creates no replacement and changes no ledger metadata.

- [ ] **Step 1: Verify the deletion-only precondition**

Run a read-only Node assertion that every decision belonging to the 23 paths above has `resolution.disposition === "delete"`, has no replacement ID, and has a non-empty deletion reason. Expected: 70 rows and no violation.

- [ ] **Step 2: Delete the files**

Delete only the 23 exact paths above using the executor's normal file-edit mechanism. Do not edit the ledger.

- [ ] **Step 3: Validate the free closure**

Run:

```powershell
node scripts/validate-testing-transition.mjs
git diff --check
```

Expected: transition passes and reports 366 open rows. The diff contains only 23 file deletions.

- [ ] **Step 4: Review and commit**

After one read-only deletion-scope review:

```powershell
git add --all -- src/lib/analysis-legacy-surfaces-cleanup.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-state-legacy-selection-cleanup.test.ts src/lib/api/apalis-jobs.test.ts src/lib/api/diagnostics.test.ts src/lib/components/research-projects/OptionsPanel.component.test.ts src/lib/components/research-projects/ProjectRailPanel.component.test.ts src/lib/components/research-projects/ProjectRow.component.test.ts src/lib/components/research-projects/ProjectTabs.component.test.ts src/lib/components/research-projects/SourcesBulkBar.component.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/development-loop-performance-contract.test.ts src/lib/focused-rust-loop-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/media-metadata-core-contract.test.ts src/lib/prompt-pack-completion-transport-contract.test.ts src/lib/prompt-pack-run-store-contract.test.ts src/lib/prompt-pack-stage-execution-contract.test.ts src/lib/prompt-pack-stage-request-policy-contract.test.ts src/lib/research-projects-foundation-contract.test.ts src/lib/research-projects-lucide-import-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/youtube-summary-smoke-fixture-contract.test.ts
git commit -m "test: remove accepted legacy-only contracts"
```

### Task 3: Establish the independent application Playwright harness

**Files:**
- Create: `e2e/playwright.config.ts`
- Create: `e2e/fixtures/tauri.ts`
- Create: `e2e/app-harness-smoke.spec.ts` (temporary tracked owner; removed in Task 6)
- Modify: `package.json`
- Modify: `testing/runner-census.json`
- Modify: `scripts/verify.mjs`
- Modify: `scripts/verify.test.ts`
- Modify: `scripts/testing/validate-testing-transition.test.ts`

**Interfaces:**
- Produces `test:app:e2e` and census owner `playwright:app-e2e` for `e2e/**/*.spec.ts`.
- Keeps `test:e2e` and `playwright:gemini-browser-adapter` unchanged.
- Uses explicit `http://127.0.0.1:4178` with `--strictPort` and one narrow pre-navigation Tauri IPC installer.

- [ ] **Step 1: Write runner/census RED tests**

Add exact assertions that the census contains both Playwright owners, that `verify` contains distinct adapter and app gates, and that an `e2e/smoke.spec.ts` candidate is owned only by `playwright:app-e2e`.

- [ ] **Step 2: Run the RED**

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts -t "application Playwright owner|app e2e gate"
```

Expected: FAIL because the config, script, census owner, and gate do not exist.

- [ ] **Step 3: Add the minimal harness**

Create `e2e/playwright.config.ts` with Chromium, workers `1`, `fullyParallel: false`, retained-on-failure trace/screenshot, base URL `http://127.0.0.1:4178`, and a `webServer` command using `npm.cmd` on Windows and `npm` elsewhere:

```ts
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
// webServer.command = `${npmCommand} run dev -- --host 127.0.0.1 --port 4178 --strictPort`
```

`e2e/fixtures/tauri.ts` installs only the invoke/event responses requested by a scenario before navigation; unknown commands throw with the command name. Do not introduce a production test route or duplicate component markup.

- [ ] **Step 4: Prove one real route smoke and cleanup**

Create `e2e/app-harness-smoke.spec.ts`, navigate to the real app shell, assert a stable application landmark, then run:

```powershell
npm.cmd run test:app:e2e -- --grep "app e2e harness smoke"
```

Expected: one Chromium test passes. Run the documented PowerShell PID/CommandLine audit and require no repository-owned Vite or Playwright Chromium descendants. Keep the smoke tracked so the new census owner is non-empty; Task 6 removes it only after the three permanent specs are present and green.

- [ ] **Step 5: Run focused GREEN and commit**

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts
node scripts/validate-testing-transition.mjs
git diff --check
git add e2e/playwright.config.ts e2e/fixtures/tauri.ts e2e/app-harness-smoke.spec.ts package.json testing/runner-census.json scripts/verify.mjs scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts
git commit -m "test: establish application playwright owner"
```

Do not run complete `verify`.

### Task 4: Add the 18 non-component one-row Vitest targets

**Files — create or extend exactly:**

| Target | Frozen rows |
| --- | --- |
| `scripts/process-shell-diagnostic/git-state.behavior.test.ts` | SC-000010 |
| `scripts/process-shell-diagnostic/report.behavior.test.ts` | SC-000011 |
| `scripts/run-vitest.behavior.test.ts` | SC-000020 |
| `scripts/testing/run-observation.behavior.test.ts` | SC-000023 |
| `scripts/testing/slice-1-baseline.behavior.test.ts` | SC-000024 |
| `scripts/testing/research-projects-grid-date-formatting.behavior.test.ts` | SC-000514 |
| `scripts/testing/extractum-grid-wrapper-boundary.behavior.test.ts` | SC-000515 |
| `src/lib/accounts-route-add-account-modal.behavior.test.ts` | SC-000025 |
| `src/lib/analysis-group-editor-props.behavior.test.ts` | SC-000095 |
| `src/lib/analysis-ui-smoke-contract.behavior.test.ts` | SC-000283 |
| `src/lib/gemini-browser-polling.test.ts` | SC-000416 |
| `src/lib/prompt-packs/start-youtube-summary-run.behavior.test.ts` | SC-000464 |
| `src/lib/tauri-security-config.behavior.test.ts` | SC-000557 |
| `src/routes/projects/library/library-page.behavior.test.ts` | SC-000435 |
| `src/routes/projects/runs/project-runs-page.behavior.test.ts` | SC-000450 |
| `src/routes/projects/next/page-inspector.behavior.test.ts` | SC-000670 |
| `src/routes/projects/next/page-keyboard.behavior.test.ts` | SC-000671 |
| `src/routes/settings/settings-focus.behavior.test.ts` | SC-000552 |

**Interfaces:** All targets are Node/Vitest. SC-000023 is a real child/grandchild B3 test and must be assigned to `os-integration`; the remaining pure files stay in `unit-node`. Targets whose connected component belongs to Wave 2 or Wave 3 are additive only in this task.

- [ ] **Step 1: Print and pin the exact titles**

Run the Frozen Interfaces command for every target path. Record the complete replacement title in the new `it` or `test` declaration without adding a `describe` prefix unless the frozen ID already contains one.

- [ ] **Step 2: Add target-file REDs in two bounded groups**

Group A is `scripts/**`; Group B is `src/**`. Each target imports a production module or a behavior-neutral extracted adapter. Route targets test exported request/callback/selection logic under Node; they do not render Svelte and do not read route source text.

Run the exact new file paths through `node scripts/run-vitest.mjs run --project unit-node ...`, except SC-000023 through `--project os-integration`. Expected: every declaration is collected and fails for the missing observable behavior; no target reports zero tests.

- [ ] **Step 3: Implement the minimal observable seams**

Preserve the artifact invariant for each row:

- process/git/report targets exercise public serialization, pinning, and cleanup APIs with temporary fixtures;
- wrapper targets invoke exported runner/observation functions and assert real exit/cleanup outcomes;
- route targets call extracted request or callback builders with spies and assert arguments/results;
- grid/date/security targets import the public configuration object rather than production source text;
- SC-000416 drives the real polling controller and proves idle discovery through the light-refresh path.

- [ ] **Step 4: Run focused GREEN**

Run Group A and Group B as at most two concurrent frontend commands, then run SC-000023 under `os-integration` separately if it was not part of the OS command. Run `npm.cmd run check` only if a TypeScript/Svelte production seam changed. Expected: all 18 exact declarations pass.

- [ ] **Step 5: Commit the additive targets**

Stage only the 18 targets, any narrowly required production seam files, and the SC-000023 `OS_INTEGRATION_FILES` addition. Do not delete any legacy file in a later-wave component.

```powershell
git commit -m "test: add single-row node contract owners"
```

### Task 5: Add the 12 component one-row targets and cut over Wave 1

**Files — create exactly:**

| Target | Frozen rows |
| --- | --- |
| `src/lib/components/desktop-dialog.behavior.test.ts` | SC-000383 |
| `src/lib/components/diagnostics/DiagnosticCountTable.behavior.test.ts` | SC-000379 |
| `src/lib/components/modal-host.behavior.test.ts` | SC-000384 |
| `src/lib/components/research-projects/Inspector.behavior.test.ts` | SC-000325 |
| `src/lib/components/research-projects/LibraryFilterRail.behavior.test.ts` | SC-000436 |
| `src/lib/components/research-projects/LibraryInspector.behavior.test.ts` | SC-000438 |
| `src/lib/components/research-projects/LibraryScreen.behavior.test.ts` | SC-000439 |
| `src/lib/components/research-projects/LibraryWorkspace.behavior.test.ts` | SC-000437 |
| `src/lib/components/research-projects/LibraryYoutubeAddPanel.behavior.test.ts` | SC-000423 |
| `src/lib/components/research-projects/ProjectRunsTab.behavior.test.ts` | SC-000461 |
| `src/lib/components/research-projects/ProjectToolbar.behavior.test.ts` | SC-000332 |
| `src/lib/components/research-projects/YoutubeSummaryRunsPanel.behavior.test.ts` | SC-000462 |

**Delete in the Wave 1 cutover:**

```text
scripts/process-shell-diagnostic/git-state.test.ts
scripts/process-shell-diagnostic/report.test.ts
scripts/run-vitest.test.ts
scripts/testing/run-observation.test.ts
scripts/testing/slice-1-baseline.test.ts
src/lib/accounts-route-add-account-modal.test.ts
src/lib/analysis-group-editor-props.test.ts
src/lib/analysis-ui-smoke-contract.test.ts
src/lib/components/research-projects/Inspector.component.test.ts
src/lib/components/research-projects/ProjectToolbar.component.test.ts
src/lib/library-prototype-contract.test.ts
src/lib/project-runs-tab-delete-contract.test.ts
src/routes/projects/next/page-inspector.test.ts
src/routes/projects/next/page-keyboard.test.ts
```

**Interfaces:** All 12 targets inherit `setup-component-tests.ts` and `autoCleanup: false`. The dialog targets remain additive because their component also needs Playwright. The diagnostics target remains additive because its `(2,2)` component also needs the multi-row diagnostics page target.

- [ ] **Step 1: Write one focused RED per target file**

Use Testing Library/Svelte render and user interaction. Assert accessible DOM, callback arguments, disabled state, selected context, and explicit dismissal. Do not assert Svelte source, class implementation details, or SVAR internals.

Run:

```powershell
node scripts/run-vitest.mjs run --project component <the-12-exact-target-paths>
```

Expected: 12 collected declarations fail before their observable fixtures/seams are complete.

- [ ] **Step 2: Implement minimal component fixtures and GREEN**

Reuse the existing component setup and local fixture builders. For modal/dialog tests, prove outside dismissal is rejected and explicit cancel/confirm works. For Library and runs targets, use domain-shaped props and spies to prove the exact callbacks. For toolbar/inspector, assert accessible state at the frozen responsive/context boundary.

Run the same 12-path command. Expected: PASS with no manual cleanup leak.

- [ ] **Step 3: Perform the Wave 1 cutover**

Delete only the 14 Wave 1 legacy paths above. Remove the four replaced process legacy paths from `OS_INTEGRATION_FILES` only if their new behavior owners are already listed; `run-observation.behavior.test.ts` remains listed. Do not delete dialog, diagnostics, or any later-wave legacy file.

- [ ] **Step 4: Validate, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
```

Expected: transition passes with 337 open rows. Review must confirm that SC-000383, SC-000384, and SC-000379 are present but their connected legacy components remain open.

```powershell
git commit -m "test: cut over single-row vitest components"
```

### Task 6: Complete and cut over the browser-connected components

**Files:**
- Create: `src/lib/components/app-sidebar.behavior.test.ts` — SC-000306–SC-000311
- Create: `e2e/app-shell-responsive.spec.ts` — SC-000312
- Create: `e2e/research-projects-sources-filter-row.spec.ts` — SC-000344
- Create: `e2e/dialog-layering.spec.ts` — SC-000385
- Delete after permanent-spec GREEN: `e2e/app-harness-smoke.spec.ts`
- Delete at cutover: `src/lib/app-sidebar-behavior.test.ts`
- Delete at cutover: `src/lib/components/research-projects/SourcesFilterRow.component.test.ts`
- Delete at cutover: `src/lib/dialog-bits-ui-migration.test.ts`

**Interfaces:** Uses the Task 3 app harness and the Task 5 desktop-dialog/modal-host targets. Closes exactly 3 components, 3 legacy files, and 11 rows when all jsdom and browser targets are green.

- [ ] **Step 1: Write the component and Playwright REDs**

The app-sidebar component file renders the real sidebar/top-bar composition and uses exact frozen titles for SC-000306–SC-000311. The Playwright files use exact top-level titles from the artifact and prove:

- SC-000312: the real mobile menu trigger's responsive visibility;
- SC-000344: filters remain available across responsive layouts;
- SC-000385: real dialog content remains visible and interactive above its overlay.

Run the component file and each Playwright file with an exact path or `--grep`. Expected: every selection is non-empty and fails before implementation/fixtures are complete.

- [ ] **Step 2: Implement truthful browser behavior**

Use real routes and real application styles. Install only the Tauri responses needed for app startup and the target interaction. Do not use `page.setContent`, copied component markup, CSS string inspection, screenshot pixel thresholds, or retries.

- [ ] **Step 3: Run focused GREEN and process audit**

```powershell
node scripts/run-vitest.mjs run --project component src/lib/components/app-sidebar.behavior.test.ts src/lib/components/desktop-dialog.behavior.test.ts src/lib/components/modal-host.behavior.test.ts
npm.cmd run test:app:e2e -- e2e/app-shell-responsive.spec.ts e2e/research-projects-sources-filter-row.spec.ts e2e/dialog-layering.spec.ts
```

Expected: component tests and exactly three Playwright tests pass. Run the process postflight and require no owned Vite/Chromium descendants.

- [ ] **Step 4: Cut over, validate, review, and commit**

Delete `e2e/app-harness-smoke.spec.ts` only after all three permanent specs are green, so `playwright:app-e2e` is never empty. Delete the three legacy files only after both sides of the app-sidebar and dialog components pass.

```powershell
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
git commit -m "test: replace browser-owned source contracts"
```

Expected: transition passes with 326 open rows. If the browser harness is blocked, preserve the evidence, leave these three legacy files present, and continue Tasks 7–14 without claiming Slice 3C completion.

### Task 7: Add the Wave 2 process owners to `os-integration`

**Files:**
- Create: `scripts/process-shell-diagnostic/coordinator.behavior.test.ts` - SC-000001-SC-000009
- Create: `scripts/process-shell-diagnostic/runtime.behavior.test.ts` - SC-000012-SC-000019
- Modify: `vitest.config.ts`
- Modify: `scripts/testing/test-conventions.test.ts`

**Interfaces:** Both new files are real Windows process-lifecycle owners. Add both paths to `OS_INTEGRATION_FILES` in this additive commit while the legacy coordinator/runtime paths remain in the same array. They must never run in the threaded `unit-node` pool.

- [ ] **Step 1: Add ownership and behavior REDs**

In the conventions test, assert exclusive `os-integration` ownership for both new paths. In the two target files, use the exact frozen declaration titles and real owned processes to prove coordinator session completion, JSON single-write behavior, descendant cleanup, and explicit Windows failure reporting.

Run:

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts -t "process behavior ownership"
node scripts/run-vitest.mjs run --project os-integration scripts/process-shell-diagnostic/coordinator.behavior.test.ts scripts/process-shell-diagnostic/runtime.behavior.test.ts
```

Expected: the ownership assertion fails before the array change, and the non-empty behavior selection fails before the complete seam is implemented.

- [ ] **Step 2: Implement and run focused GREEN**

Reuse the existing process fixtures and cleanup helpers. Do not mock PID liveness or replace descendant cleanup with source assertions. Run the same two commands, then audit for owned process survivors.

- [ ] **Step 3: Commit additively**

```powershell
node scripts/validate-testing-transition.mjs
git diff --check
git commit -m "test: add process diagnostic behavior owners"
```

Expected: transition remains at the current open-row count because neither legacy process file is deleted.

### Task 8: Cut over Wave 2A

**Replacement targets and frozen rows:**

| Target | Rows |
| --- | --- |
| `scripts/process-shell-diagnostic/coordinator.behavior.test.ts` | SC-000001-SC-000009 |
| `scripts/process-shell-diagnostic/runtime.behavior.test.ts` | SC-000012-SC-000019 |
| `src/lib/accounts-ux-contract.behavior.test.ts` | SC-000026-SC-000028 |
| `src/lib/apalis-jobs-route-contract.behavior.test.ts` | SC-000297, SC-000300 |
| `src/lib/components/jobs/ApalisJobsPanel.behavior.test.ts` | SC-000301-SC-000303 |
| `src/routes/diagnostics/diagnostics-page.behavior.test.ts` | SC-000372-SC-000378, SC-000380-SC-000382 |
| `src/lib/components/diagnostics/DiagnosticCountTable.behavior.test.ts` | SC-000379, already added in Task 5 |
| `src/routes/settings/provider-test-console.behavior.test.ts` | SC-000505-SC-000507 |

**Delete exactly:**

```text
scripts/process-shell-diagnostic/coordinator.test.ts
scripts/process-shell-diagnostic/runtime.test.ts
src/lib/accounts-ux-contract.test.ts
src/lib/apalis-jobs-route-contract.test.ts
src/lib/diagnostics-route-contract.test.ts
src/lib/diagnostics-ux-contract.test.ts
src/lib/provider-test-console-placement.test.ts
```

**Interfaces:** Closes 7 legacy files and 42 rows. The two new process paths remain in `OS_INTEGRATION_FILES`; the two deleted process paths leave it in this cutover commit.

- [ ] **Step 1: Write REDs for the four not-yet-implemented target groups**

Use direct module/API spies for Node route orchestration and rendered accessible behavior for `ApalisJobsPanel`. Reuse the green diagnostics count target from Task 5. Run exact path selections in their owning projects; no selection may be empty.

- [ ] **Step 2: Implement GREEN and complete process ownership migration**

Run at most two frontend owner commands concurrently: one `unit-node` selection and one `component` selection. Run `os-integration` separately. Remove only the two legacy process paths from `OS_INTEGRATION_FILES` after all owners are green.

- [ ] **Step 3: Cut over, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
git commit -m "test: cut over source contracts wave 2a"
```

Expected: 284 open rows if Task 6 cut over the browser components; otherwise 295. The full validator runs exactly once for this cutover batch.

### Task 9: Cut over Wave 2B analysis contracts

**Replacement targets:**

```text
src/lib/analysis-compact-source-rail.behavior.test.ts                  SC-000060-SC-000073 except SC-000064
src/lib/analysis-companion-layout.behavior.test.ts                     SC-000074-SC-000076
src/lib/analysis-evidence-source-navigation.behavior.test.ts           SC-000093-SC-000094
src/lib/analysis-llm-run-controls.behavior.test.ts                     SC-000097-SC-000098
src/lib/analysis-priority-ux-contract.behavior.test.ts                 SC-000100-SC-000105
src/lib/analysis-redesign-route-contract.behavior.test.ts              SC-000106-SC-000114
src/lib/analysis-redesign-safety-contract.behavior.test.ts             SC-000117, SC-000119-SC-000120, SC-000122-SC-000127
```

**Delete exactly:**

```text
src/lib/analysis-compact-source-rail.test.ts
src/lib/analysis-companion-layout.test.ts
src/lib/analysis-evidence-source-navigation.test.ts
src/lib/analysis-llm-run-controls.test.ts
src/lib/analysis-priority-ux-contract.test.ts
src/lib/analysis-redesign-route-contract.test.ts
src/lib/analysis-redesign-safety-contract.test.ts
```

**Interfaces:** Seven Node/Vitest owners close 7 legacy files and 50 rows. Rows absent from the ranges above are already owned by earlier Slice 3 replacements and must not be duplicated.

- [ ] **Step 1: Write one complete RED file per target**

Prefer public state reducers, route adapters, and component-facing props. A pure seam extraction is allowed only when no observable seam exists. Use every exact title printed from the artifact.

- [ ] **Step 2: Run one focused Node GREEN selection**

```powershell
node scripts/run-vitest.mjs run --project unit-node <the-seven-exact-target-paths>
```

Expected: seven non-empty files pass with no source-reader obligations.

- [ ] **Step 3: Cut over, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
git diff --check
git commit -m "test: cut over source contracts wave 2b"
```

Expected: 234 open rows if browser cutover completed; otherwise 245.

### Task 10: Cut over Wave 2C report and companion contracts

**Replacement targets:**

```text
src/lib/analysis-report-canvas-route.behavior.test.ts
src/lib/analysis-report-setup-props.behavior.test.ts
src/lib/analysis-report-workspace-selection-props.behavior.test.ts
src/lib/analysis-route-effects.behavior.test.ts
src/lib/analysis-route-workspace-state.behavior.test.ts
src/lib/analysis-run-companion-route.behavior.test.ts
src/lib/analysis-run-companion-tabs.behavior.test.ts
src/lib/analysis-source-access-placement.behavior.test.ts
```

These targets own SC-000129-SC-000137, SC-000158-SC-000175, SC-000180-SC-000183, SC-000185-SC-000193, SC-000195-SC-000202, and SC-000204-SC-000206 exactly as assigned in the artifact.

**Delete exactly:**

```text
src/lib/analysis-report-canvas-route.test.ts
src/lib/analysis-report-setup-props.test.ts
src/lib/analysis-report-workspace-selection-props.test.ts
src/lib/analysis-route-effects.test.ts
src/lib/analysis-route-workspace-state.test.ts
src/lib/analysis-run-companion-route.test.ts
src/lib/analysis-run-companion-tabs.test.ts
src/lib/analysis-source-access-placement.test.ts
```

**Interfaces:** Eight Node/Vitest owners close 8 files and 58 rows. Route tests use module/API fixtures, not rendered `+page.svelte` source inspection.

- [ ] **Step 1: Add RED declarations and minimal route fixtures**

Use exact frozen titles. Keep saved-run restoration, source-window loading, tab state, and report-canvas props independently observable. Reject a fixture that proves the assertion only by reproducing route implementation text.

- [ ] **Step 2: Run focused GREEN**

```powershell
node scripts/run-vitest.mjs run --project unit-node <the-eight-exact-target-paths>
```

- [ ] **Step 3: Cut over, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
git commit -m "test: cut over source contracts wave 2c"
```

Expected: 176 open rows if browser cutover completed; otherwise 187.

### Task 11: Cut over Wave 2D source-browser and project-grid contracts

**Replacement targets:**

```text
src/lib/analysis-source-readers-route.behavior.test.ts
src/lib/analysis-workspace-tools.behavior.test.ts
src/lib/analysis-youtube-source-specialization.behavior.test.ts
src/lib/components/analysis/source-browser-shell.behavior.test.ts
src/lib/components/extractum-ui/DataGrid.behavior.test.ts
src/lib/components/research-projects/ResearchProjectsShell.behavior.test.ts
src/lib/components/research-projects/SourcesGrid.behavior.test.ts
```

**Delete exactly:**

```text
src/lib/analysis-source-readers-route.test.ts
src/lib/analysis-workspace-tools.test.ts
src/lib/analysis-youtube-source-specialization.test.ts
src/lib/components/analysis/source-browser-shell.test.ts
src/lib/components/extractum-ui/DataGrid.test.ts
src/lib/components/research-projects/ResearchProjectsShell.component.test.ts
src/lib/components/research-projects/SourcesGrid.test.ts
```

**Interfaces:** The three `src/lib/analysis-*` targets run in `unit-node`; the four `src/lib/components/**` targets run in `component`. Together they close 7 files and 56 rows. SC-000323 and SC-000344 are not reimplemented here: their accepted visual/browser ownership remains in Task 6 or its deferred cutover.

- [ ] **Step 1: Write Node and component REDs**

For source readers and YouTube specialization, use API/event fixtures. For rendered grid shells, assert accessible host state, row/cell behavior, callbacks, and responsive column decisions that jsdom can truthfully expose; do not claim visual clipping or layering.

- [ ] **Step 2: Run focused GREEN with at most two commands**

Run the three Node paths in one `unit-node` command and the four component paths in one `component` command. Both selections must be non-empty.

- [ ] **Step 3: Cut over, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
git commit -m "test: cut over source contracts wave 2d"
```

Expected: 120 open rows if browser cutover completed; otherwise 131.

### Task 12: Cut over Wave 2E settings, library, runs, and YouTube-summary contracts

**Replacement targets not already added in Tasks 4-5:**

```text
src/lib/components/settings/gemini-browser-provider-panel.behavior.test.ts
src/lib/components/research-projects/LibraryYoutubeSmartImport.behavior.test.ts
src/lib/components/research-projects/LibraryYoutubePlaylistImport.behavior.test.ts
src/lib/components/research-projects/LibraryTelegramDialogImport.behavior.test.ts
src/lib/components/research-projects/LibraryAddSourceDialog.behavior.test.ts
src/lib/components/research-projects/ProjectRunsScreen.behavior.test.ts
src/lib/components/research-projects/ProjectRunReportPanel.behavior.test.ts
scripts/testing/research-projects-import-boundary.behavior.test.ts
src/routes/settings/settings-profiles.behavior.test.ts
src/routes/accounts/source-access.behavior.test.ts
src/lib/components/research-projects/youtube-summary-launch.behavior.test.ts
src/lib/components/research-projects/youtube-summary-result.behavior.test.ts
```

Also consume the already-green one-row owners from Task 4 (`gemini-browser-polling`, grid formatting, wrapper boundary, runs route, settings focus) and Task 5 (`LibraryYoutubeAddPanel`, `ProjectRunsTab`, `YoutubeSummaryRunsPanel`).

**Delete exactly:**

```text
src/lib/gemini-browser-provider-panel.test.ts
src/lib/library-add-source-contract.test.ts
src/lib/project-runs-screen-contract.test.ts
src/lib/research-projects-import-boundary.test.ts
src/lib/settings-profile-ux-contract.test.ts
src/lib/source-access-placement.test.ts
src/lib/youtube-summary-launch-contract.test.ts
src/lib/youtube-summary-result-view-contract.test.ts
```

**Interfaces:** Mixed Node/component ownership closes 8 files and 65 rows. Five components require both commands; keep each connected component in one cutover even though its owners were implemented in different tasks.

- [ ] **Step 1: Add remaining REDs**

Use actual API adapters and rendered accessible workflows. The import-boundary target may inspect the repository through its approved structured test seam only; do not add a raw-source exception. Use the exact artifact title for every row.

- [ ] **Step 2: Run focused GREEN**

Run all Node paths in one `unit-node` command and all component paths in one `component` command. At most those two frontend commands may run concurrently.

- [ ] **Step 3: Cut over, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
git commit -m "test: cut over source contracts wave 2e"
```

Expected: 55 open rows if browser cutover completed; otherwise 66.

### Task 13: Implement root `extractum` Cargo identities

**Files:** Modify the owning Rust modules and their test modules under `src-tauri/src/`; create only behavior-neutral seams required by the frozen invariants.

**Exact identities:**

```text
SC-000084 extractum::analysis::tests_application::analysis_coordinators_share_one_app_owned_transaction
SC-000090 extractum::analysis::tests_application::analysis_command_event_and_app_error_wire_contracts_are_exact
SC-000387 extractum::process_tree::tests::windows_guard_contains_and_reaps_owned_descendants
SC-000389 extractum::gemini_browser::tests::cdp_child_guard_reaps_the_complete_owned_tree
SC-000390 extractum::shutdown::tests::exit_requests_delegate_once_and_complete_bounded_cleanup
SC-000420 extractum::child_process::tests::gemini_sidecar_launches_hide_console_window_on_windows
SC-000443 extractum::llm::tests::secure_profile_and_transport_ownership_remains_application_scoped
SC-000465 extractum::tests::startup_seeds_prompt_packs_before_interrupted_run_cleanup
SC-000555 extractum::security_config_tests::base_tauri_configuration_is_production_restrictive
SC-000556 extractum::security_config_tests::production_image_csp_rejects_remote_origins
SC-000558 extractum::security_config_tests::frontend_capabilities_contain_no_sql_permission
SC-000559 extractum::security_config_tests::mcp_and_fixture_commands_are_localhost_dev_only
SC-000560 extractum::security_config_tests::production_devtools_require_csp_verification
```

**Interfaces:** This task is additive. It does not delete any of the nine Wave 3 legacy files because five extracted-package identities are still missing.

- [ ] **Step 1: List and write exact REDs**

For each identity, first list tests if its module/name does not yet exist. Run the exact test and require a non-zero selection. The process-tree and child-process tests use real Windows ownership/flags; security tests parse committed Tauri/capability JSON through Rust test fixtures, not TypeScript raw-source assertions.

- [ ] **Step 2: Implement minimal seams and exact GREEN**

Likely owning modules are `analysis/tests_application.rs`, `process_tree.rs`, `gemini_browser`, `shutdown`, `child_process`, `llm`, `lib.rs`, and `security_config_tests.rs`. Do not create a generic test-only production API.

- [ ] **Step 3: Check and checkpoint the package**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
node scripts/validate-testing-transition.mjs
git diff --check
git commit -m "test: add application cargo contract owners"
```

Expected: the Cargo tests pass, while open-row count is unchanged because the connected legacy files remain.

### Task 14: Implement extracted-package Cargo identities and cut over Wave 3

**Exact identities:**

```text
SC-000087 extractum-analysis::events::tests::event_adapter_is_bounded_and_nonblocking
SC-000441 extractum-llm::public_api_tests::curated_api_keeps_credentials_non_serializable_and_inaccessible
SC-000474 extractum-prompt-packs::public_api_tests::cancellation_smoke_services_remain_test_only
SC-000484 extractum-prompt-packs::run_control::tests::terminal_events_clear_required_run_state
SC-000492 extractum-prompt-packs::runtime_config::tests::invalid_persisted_runtime_configuration_is_reported
```

**Delete exactly after all 18 Cargo identities and Task 4 Node companions are green:**

```text
src/lib/analysis-crate-boundary-contract.test.ts
src/lib/external-process-lifecycle-contract.test.ts
src/lib/hidden-child-process-contract.test.ts
src/lib/llm-crate-boundary-contract.test.ts
src/lib/prompt-pack-application-contract.test.ts
src/lib/prompt-pack-crate-boundary-contract.test.ts
src/lib/prompt-pack-run-control-contract.test.ts
src/lib/prompt-pack-runtime-config-contract.test.ts
src/lib/tauri-security-config-contract.test.ts
```

**Interfaces:** Closes 9 files and 55 rows. Package ownership is `extractum-analysis` for SC-000087, `extractum-llm` for SC-000441, and `extractum-prompt-packs` for SC-000474/484/492.

- [ ] **Step 1: Add exact RED/GREEN tests per package**

Use the Rust Verification Loops section. SC-000441 must prove the full curated public API plus non-serializable/inaccessible credentials, not only a narrow credential assertion. If a test-support feature is required, follow the producer feature-off and consumer feature-on rules from `AGENTS.md`.

- [ ] **Step 2: Run package checks and checkpoints sequentially**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
```

- [ ] **Step 3: Delete the nine legacy files and handle the actual-last condition**

If Task 6 already cut over the browser components, this is the factually last legacy batch. In the same staged diff:

- remove `LEGACY_TEST_FILES` and the `legacy-contract` project from `vitest.config.ts`;
- remove the legacy owner from `testing/runner-census.json`;
- remove `test:legacy-contract` from `package.json`;
- remove its gate from `scripts/verify.mjs` and update `scripts/verify.test.ts` and conventions tests.

If Task 6 is still deferred, leave all legacy-runner wiring intact and expect 11 open rows. The later browser cutover must perform this removal instead.

- [ ] **Step 4: Validate, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
git diff --check
git commit -m "test: cut over source contracts wave 3"
```

Expected: 0 open rows and no legacy owner if browser cutover is complete; otherwise 11 open rows and the legacy owner remains.

### Task 15: Remove the factually last legacy runner, verify once, and hand off Slice 4

**Files:**
- Modify as needed at the last cutover: `vitest.config.ts`, `testing/runner-census.json`, `package.json`, `scripts/verify.mjs`, `scripts/verify.test.ts`, `scripts/testing/test-conventions.test.ts`
- Create: `docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md`
- Modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`

**Interfaces:** This task runs only after the browser and Cargo tracks have both cut over. It records no benchmark series: one preflight, one retained wall-clock observation for the single final `verify`, and no warmup/median/threshold.

- [ ] **Step 1: Complete a deferred browser cutover if necessary**

If Task 6 was deferred, resume it now. In the same commit that deletes its last three legacy files, remove all legacy-runner wiring listed in Task 14 Step 3. Run the transition validator and require zero open rows before committing.

- [ ] **Step 2: Prove the zero-inventory state with focused gates**

Run sequentially:

```powershell
node scripts/testing/source-contract-redisposition-review.mjs check
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
```

Expected: carrier changed paths empty; ledger 671 rows/0 open; census has no `vitest:legacy-contract`; no tracked legacy file exists; Svelte/TypeScript check is clean.

- [ ] **Step 3: Bootstrap the fresh worktree and run the process preflight**

```powershell
npm.cmd run bootstrap:testing
```

Use the established PowerShell/CIM preflight to require zero competing repository-owned Vitest, Playwright, Cargo, Rust, Vite, or Chromium processes. Record the exact command and result. Do not start a warmup.

- [ ] **Step 4: Run exactly one final full verification**

Run unsandboxed on Windows:

```powershell
npm.cmd run verify
```

Retain its exit code and one wall-clock duration. Do not rerun for a correctness failure. A retry is allowed only for a demonstrated spawn/sandbox/process-infrastructure failure, and both observations must remain in evidence.

- [ ] **Step 5: Write final evidence and update the program index**

The verification document records:

- all 9 cutover batches and their commits;
- 436 closed rows, 86 deleted legacy files, 84 completed components;
- final Node/jsdom/Cargo/Playwright owner counts;
- the process preflight and sole full-verify observation;
- zero open ledger rows and removal of the legacy runner;
- any browser deferral and its eventual factual-last cutover;
- no Nx decision and no performance threshold claim.

Update the program index to mark Slice 3 complete and identify Slice 4 as next. Do not claim Slice 3C complete if any browser component, legacy file, or legacy runner remains.

- [ ] **Step 6: Final read-only review and evidence commit**

Run one read-only spec/quality review over the complete Slice 3C commit range and verification record. Resolve findings in one bounded fix wave, rerun only the directly affected focused gate, and do not repeat full `verify` unless the retry exception above applies.

```powershell
git diff --check
git commit -m "docs: record slice 3c source-contract completion"
```

Expected final state: tracked worktree clean, source-contract ledger `671/0`, no legacy runner, final verification recorded, and Slice 4 ready to begin.
