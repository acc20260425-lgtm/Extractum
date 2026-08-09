# Testing Redesign Slice 3C Source-Contract Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all 436 remaining source-contract rows, remove all 86 legacy files and the empty legacy runner, and hand a zero-open-row inventory to Slice 4.

**Architecture:** Implement the exact owners frozen in `testing/source-contract-redisposition-review.json`, but cut over by the 79 connected legacy-to-replacement components. Additive owner commits may precede cutover; a cutover deletes only complete components. Wave 2 is split into five bounded integration batches, the browser track starts early, and only the final acceptance runs complete `verify`.

**Tech Stack:** Node.js ESM, TypeScript, SvelteKit, Vitest 4.1.5, Testing Library/Svelte with jsdom, Playwright 1.61, Rust 2021/Cargo, Windows PowerShell, the source-contract ledger and transition validator.

## Global Constraints

- Implement only the classes, dispositions, paths, declaration titles, invariants, deletion reasons, and criticality decisions frozen in `testing/source-contract-redisposition-review.json` and `testing/source-contract-ledger.json`.
- The approved ownership corrections are exact: component behavior owners belong to `component`; SC-000464 belongs jointly to the comprehensive service Cargo owner and actual application execution-task owner listed in Task 4; SC-000441 is mixed across `rule:extractum-llm-public-api-boundary` for ordinals 1/2/3/6/8/9/10/11/12/13 and its Cargo behavior owner for ordinals 4/5/7; and SC-000515 is mixed across the structured repository rule for ordinals 1/11/19 and its real-wrapper component owner for the other 18 ordinals. SC-000025/435/552 also use real route/component owners. Task 9 additionally follows the exact 44-row amendment table in the approved design: A=9/63, B=26/212, C=3/21, D=6/45. SC-000076 is the sixth D row because its negative inner-layout constraint has no truthful jsdom seam. The post-evidence user adjudications delete SC-000087 as D2 implementation shape and mixed-redisposition SC-000441 while preserving their historical blind bytes. The final forecast is Node 139 rows/787 ordinals, jsdom 123/864, Cargo 18/147, and Playwright 3/21. Do not rewrite accepted blind-review bytes; none of the 44 Task 9 IDs is sampled, calibrated, or in a mandatory cohort, SC-000464 was not a sampled blind result, and SC-000515's prior blind packet remains historical evidence.
- Do not use `?raw`, `readFileSync`, or another direct production-source reader in a replacement test. Do not add a `sourceReaderExceptions` entry.
- A small behavior-neutral pure-function or adapter extraction is allowed only when the frozen behavior has no truthful existing seam. Stop the connected component if path, title, mechanism, invariant, disposition, or criticality would need to change.
- Do not add Nx, a migration scheduler, a second ledger, a subset transition validator, timing thresholds, warmups, medians, or benchmark artifacts.
- Use `npm.cmd` for repository npm commands on Windows. Run at most two focused frontend owner commands concurrently.
- Process-spawning replacement files enter `OS_INTEGRATION_FILES` in their additive commit. Their legacy paths leave that list only in the component cutover commit.
- A correctness failure is retained and fixed. A final `verify` retry is allowed only for a demonstrated spawn, sandbox, or process-infrastructure failure, and both observations must be recorded.
- Nx remains deferred until Slice 4 has committed.

---

## Frozen Interfaces and Work Graph

The decision artifact is the only row-to-owner map. Task 1 generates the complete map once into the ignored scratchpad; later tasks read their target sections and never edit or rerun a path-specific query.

```powershell
@'
const fs = require("fs");
const artifact = JSON.parse(fs.readFileSync("testing/source-contract-redisposition-review.json", "utf8"));
const ledger = JSON.parse(fs.readFileSync("testing/source-contract-ledger.json", "utf8"));
const rows = new Map(ledger.rows.map((row) => [row.id, row]));
const groups = new Map();
const covered = new Set();
for (const decision of artifact.decisions) {
  const row = rows.get(decision.id);
  const ids = decision.id === "SC-000441"
    ? decision.resolution?.subgroups?.flatMap((group) => group.replacementIds ?? []) ?? []
    : decision.resolution?.replacementIds ?? [];
  const keys = ids.length > 0
    ? [...new Set(ids.map((id) => id.split("#")[0]))]
    : [`delete:${row.path}`];
  for (const key of keys) {
    const entries = groups.get(key) ?? [];
    entries.push({
      id: decision.id,
      legacyPath: row.path,
      invariant: row.invariant,
      assertionCount: row.assertionCount,
      replacementIds: ids,
      class: decision.class,
      ownerEvidence: decision.ownerEvidence ?? null,
      criticalityRef: decision.criticalityRef ?? null,
      deletionReason: decision.resolution?.deletionReason ?? null,
    });
    groups.set(key, entries);
  }
  covered.add(decision.id);
}
const targetGroups = [...groups.keys()].filter((key) => !key.startsWith("delete:"));
if (covered.size !== 436 || groups.size !== 158 || targetGroups.length !== 106) {
  throw new Error(`incomplete replacement map: ${covered.size}/${groups.size}/${targetGroups.length}`);
}
const body = [...groups.entries()]
  .sort(([left], [right]) => left.localeCompare(right))
  .map(([key, entries]) => `## ${key}\n\n\`\`\`json\n${JSON.stringify(entries, null, 2)}\n\`\`\``)
  .join("\n\n");
const output = "tmp/analysis-smoke/slice-3c-replacement-map.md";
fs.mkdirSync("tmp/analysis-smoke", { recursive: true });
fs.writeFileSync(output, `# Slice 3C replacement map\n\n${body}\n`, "utf8");
console.log(`${output}: ${covered.size} decisions, ${groups.size} owner/delete sections, ${targetGroups.length} targets`);
'@ | node -
```

For every target file, the RED test declarations use the complete title after `#` verbatim. `class` distinguishes new behavior from deletion and duplicate evidence; a D4 row uses `ownerEvidence` rather than reimplementing that owner. A `null` `criticalityRef` means there is no row-specific citation, not that the program lacks criticality policy; the complete citations remain in the artifact's top-level `criticalitySources`. Delete sections carry their frozen `deletionReason`. A declaration may contain multiple observable assertions, but the plan does not create one process cycle per ledger row.

Cutover distribution:

| Bucket | Components | Legacy files | Rows closed | Accounting remainder | Integration batches |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wave 0 | 23 | 23 | 70 | 366 | 1 |
| Wave 1 | 14 | 14 | 29 | 337 | 1 |
| Wave 2A | included below | 7 | 42 | 295 | 1 |
| Wave 2B | included below | 7 | 50 | 245 | 1 |
| Wave 2C | included below | 8 | 58 | 187 | 1 |
| Wave 2D | included below | 7 | 56 | 131 | 1 |
| Wave 2E | included below | 8 | 65 | 66 | 1 |
| Wave 3 | 9 | 9 | 55 | 11 | 1 |
| Browser | 3 | 3 | 11 | 0 | 1 |
| **Total** | **79** | **86** | **436** | **0** | **9** |

The remainder column is accounting-only: it subtracts rows in table order and does not prescribe when the independent browser batch runs. Operational cutovers verify their exact row delta, not an absolute global count. The row counts include accepted delete rows in mixed files. No batch splits a connected component.

## File Map

Permanent runner and evidence files:

- Modify `vitest.config.ts`: add component behavior ownership; replace legacy exclusions only at final cutover.
- Modify `scripts/testing/test-conventions.test.ts`: guard component behavior ownership and source-reader policy.
- Modify `scripts/testing/source-contract-redisposition-review.mjs` and its test: separate execution forecast classification from historical packet classification.
- Modify `testing/source-contract-redisposition-review.json`, `testing/source-contract-ledger.json`, and the pre-Slice 3C verification record: the approved component forecast correction and exact SC-000464 owner correction.
- Create `e2e/playwright.config.ts` and `e2e/fixtures/tauri.ts`: isolated application Playwright owner and narrow IPC setup.
- Modify `testing/runner-census.json`, `package.json`, `scripts/verify.mjs`, `scripts/verify.test.ts`: add application E2E; remove legacy ownership only at the final cutover.
- Create `docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md`: final evidence and Slice 4 handoff.
- Modify `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`: record Slice 3 completion only after final verification.

All replacement and legacy paths are listed in their owning tasks below.

## Cutover Protocol

Every behavior task uses this order:

1. Read the exact target path/title map from `tmp/analysis-smoke/slice-3c-replacement-map.md` generated in Task 1.
2. Add the complete target-file RED and run the exact owner selection; an empty selection is a failure.
3. Implement only the observable seam and run the target-file GREEN.
4. If the component is not yet complete, commit the additive owner without deleting the legacy file.
5. Before a ready batch cutover, stage all targets and legacy deletions, run `node scripts/validate-testing-transition.mjs` exactly once, and require the task's exact open-row delta. Run `npm.cmd run check` only when Svelte or TypeScript production files changed.
6. If that cutover makes the filesystem legacy inventory empty, remove `LEGACY_TEST_FILES` and the `legacy-contract` project from `vitest.config.ts`, its census owner from `testing/runner-census.json`, `test:legacy-contract` from `package.json`, and its gate from `scripts/verify.mjs`; update the owning runner/conventions tests in the same staged diff before validation and commit.
7. Run `git diff --check`, one read-only integration review, one combined fix wave if required, and commit the bounded batch with the task's listed subject.

Do not edit resolution metadata during ordinary cutover. The user-approved SC-000464 joint Cargo correction, SC-000515 mixed/component correction, and SC-000025/435/552 component-owner correction are the exact Task 4 amendments and are applied through the fail-closed carrier before Task 4 completes. A component closes because its legacy declaration disappears and its approved replacement already resolves.

## Rust Verification Loops

Affected packages are `extractum`, `extractum-analysis`, `extractum-llm`, and `extractum-prompt-packs`.

Task 4 affects `extractum-prompt-packs` and its immediate `extractum` consumer. Its service owner is `runtime::tests::start_service_preserves_idempotency_readiness_preflight_queue_and_execution_order`; its app-level RED/GREEN owner is `prompt_packs::runtime_commands::tests::build_youtube_summary_execution_task_defers_profile_resolution_until_spawned_future_is_polled`. The producer feature-off check proves the narrow ticket constructor remains behind the existing non-default `dev-fixtures` test edge, and the retained application adapter checks remain supplementary.

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
- Produces the component-corrected intermediate forecast Node `179/1109`, jsdom `90/598`, Cargo `18/156`, Playwright `3/21`, proposed-new-jsdom `0/0`; the first Task 4 corrections produce `178/1101`, `90/598`, `19/161`, `3/21`, the first real-component owner correction produces `174/1065`, `94/634`, `19/161`, `3/21`, the Task 9 exact amendment produces `139/787`, `123/864`, `19/161`, `3/21`, the SC-000087 adjudication produces `139/787`, `123/864`, `18/157`, `3/21`, and the SC-000441 mixed adjudication produces the final `139/787`, `123/864`, `18/147`, `3/21`.

- [ ] **Step 1: Generate the complete replacement map once**

Run the single command in Frozen Interfaces and require `tmp/analysis-smoke/slice-3c-replacement-map.md` to report all 436 decisions. Keep the file ignored and available to every later task; do not regenerate it unless the approved decision artifact changes.

- [ ] **Step 2: Add focused RED assertions**

In `scripts/testing/test-conventions.test.ts`, assert that a component behavior target is owned only by `component`, while the two existing Telegram behavior files remain owned only by `unit-node`. In the carrier test, mutate the execution classifier input so `src/lib/components/example.behavior.test.ts` must report `jsdom` and `src/lib/example.behavior.test.ts` must report `node`; assert the exact forecast counts above.

- [ ] **Step 3: Run the RED**

Run:

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts scripts/testing/source-contract-redisposition-review.test.ts -t "component behavior ownership|execution owner forecast"
```

Expected: FAIL because component behavior paths still fall through to `unit-node` and the carrier still classifies only `.component.test.ts` as jsdom.

- [ ] **Step 4: Implement the ownership correction**

In `vitest.config.ts`, add:

```ts
const COMPONENT_BEHAVIOR_PATTERN = "src/lib/components/**/*.behavior.test.ts";
```

Include it in the `component` project's `include`, exclude it from `unit-node`, and retain the existing setup file and `autoCleanup: false`. In the carrier, keep `mechanismForOwner` unchanged for content-addressed historical packet validation and add a separate execution classifier that treats both `.component.test.ts` and `src/lib/components/**/*.behavior.test.ts` as jsdom. Use only the execution classifier in `futureOwnerSummary`.

Update only the artifact forecast fields and append a dated forecast-correction note to the pre-Slice 3C verification document. Do not change decisions, review packet/output paths, hashes, or blind digests.

- [ ] **Step 5: Run focused GREEN and drift checks**

Run:

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/testing/test-conventions.test.ts scripts/testing/source-contract-redisposition-review.test.ts
node scripts/testing/source-contract-redisposition-review.mjs check
node scripts/validate-testing-transition.mjs
git diff --check
```

Expected: all tests pass; carrier changed paths are empty; transition remains 671 rows/436 open; census remains 196 candidates, 189 Vitest, 7 Playwright because no replacement file exists yet.

- [ ] **Step 6: Commit**

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

**Interfaces:** Closes exactly 23 components, 23 files, and 70 delete rows. Creates no replacement. This batch has one narrowly authorized ledger-envelope cleanup: remove only the top-level `sourceReaderExceptions` tuple for `src/lib/analysis-migration-fixture-contract.test.ts` at `14:13-14:74`, after its legacy reader file is deleted. The redisposition carrier must fail closed: this sole base exception may be absent only when its exact reader path is absent from the live current-present tracked-path set; it rejects a present path, any other exception addition/removal/mutation/reordering, and any `rows[]` change.

- [ ] **Step 1: Verify the deletion-only precondition**

Run a read-only Node assertion that every decision belonging to the 23 paths above has `resolution.disposition === "delete"`, has no replacement ID, and has a non-empty deletion reason. Expected: 70 rows and no violation.

- [ ] **Step 2: Delete the files**

Delete only the 23 exact paths above using the executor's normal file-edit mechanism. Then remove only the authorized top-level source-reader-exception tuple described above; preserve every `rows[]` byte and every other ledger-envelope field.

- [ ] **Step 3: Validate the free closure**

Run:

```powershell
node scripts/validate-testing-transition.mjs
git diff --check
```

Expected: transition passes and the open-row count decreases by exactly 70. The diff contains only 23 file deletions.

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
- Modify: `vitest.config.ts`
- Modify: `package.json`
- Modify: `testing/runner-census.json`
- Modify: `scripts/verify.mjs`
- Modify: `scripts/verify.test.ts`
- Modify: `scripts/testing/validate-testing-transition.test.ts`

**Interfaces:**
- Produces `test:app:e2e` and census owner `playwright:app-e2e` for `e2e/**/*.spec.ts`.
- Keeps `test:e2e` and `playwright:gemini-browser-adapter` unchanged.
- Adds `e2e/**/*.spec.ts` to the existing `PLAYWRIGHT_TESTS` exclusion so app specs are never collected by `unit-node`.
- Uses explicit `http://127.0.0.1:4178` with `--strictPort` and one narrow pre-navigation Tauri IPC installer.

- [ ] **Step 1: Write runner/census RED tests**

Add exact assertions that the census contains both Playwright owners, that `verify` contains distinct adapter and app gates, and that an `e2e/smoke.spec.ts` candidate is owned only by `playwright:app-e2e` and excluded from `unit-node`.

- [ ] **Step 2: Run the RED**

```powershell
node scripts/run-vitest.mjs run --project unit-node scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts -t "application Playwright owner|app e2e gate"
```

Expected: FAIL because the app config, script, census owner, verify gate, and Vitest exclusion do not exist; without the exclusion the candidate has duplicate Playwright/Vitest ownership.

- [ ] **Step 3: Add the minimal harness**

Create `e2e/playwright.config.ts` with Chromium, workers `1`, `fullyParallel: false`, retained-on-failure trace/screenshot, base URL `http://127.0.0.1:4178`, and a `webServer` command using `npm.cmd` on Windows and `npm` elsewhere:

```ts
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
// webServer.command = `${npmCommand} run dev -- --host 127.0.0.1 --port 4178 --strictPort`
```

`e2e/fixtures/tauri.ts` installs only the invoke/event responses requested by a scenario before navigation; unknown commands throw with the command name. Do not introduce a production test route or duplicate component markup.

Extend the existing Vitest exclusion in `vitest.config.ts`:

```ts
const PLAYWRIGHT_TESTS = Object.freeze([
  "research/gemini_browser_adapter/tests/**/*.spec.ts",
  "e2e/**/*.spec.ts",
]);
```

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
git add e2e/playwright.config.ts e2e/fixtures/tauri.ts e2e/app-harness-smoke.spec.ts vitest.config.ts package.json testing/runner-census.json scripts/verify.mjs scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts
git commit -m "test: establish application playwright owner"
```

Do not run complete `verify`.

### Task 4: Add the 17 non-component one-row Vitest targets and implement SC-000464 Cargo ownership

**Files — create or extend exactly:**

| Target | Frozen rows |
| --- | --- |
| `scripts/process-shell-diagnostic/git-state.behavior.test.ts` | SC-000010 |
| `scripts/process-shell-diagnostic/report.behavior.test.ts` | SC-000011 |
| `scripts/run-vitest.behavior.test.ts` | SC-000020 |
| `scripts/testing/run-observation.behavior.test.ts` | SC-000023 |
| `scripts/testing/slice-1-baseline.behavior.test.ts` | SC-000024 |
| `scripts/testing/research-projects-grid-date-formatting.behavior.test.ts` | SC-000514 |
| `src/lib/components/extractum-ui/extractum-grid-wrapper-boundary.behavior.component.test.ts` | SC-000515 |
| `src/routes/accounts/accounts-route-add-account-modal.behavior.component.test.ts` | SC-000025 |
| `src/lib/analysis-group-editor-props.behavior.test.ts` | SC-000095 |
| `src/lib/analysis-ui-smoke-contract.behavior.test.ts` | SC-000283 |
| `src/lib/gemini-browser-polling.test.ts` | SC-000416 |
| `src/lib/tauri-security-config.behavior.test.ts` | SC-000557 |
| `src/routes/projects/library/library-page.behavior.component.test.ts` | SC-000435 |
| `src/routes/projects/runs/project-runs-page.behavior.test.ts` | SC-000450 |
| `src/routes/projects/next/page-inspector.behavior.test.ts` | SC-000670 |
| `src/routes/projects/next/page-keyboard.behavior.test.ts` | SC-000671 |
| `src/routes/settings/settings-focus.behavior.component.test.ts` | SC-000552 |

**SC-000464 joint Cargo owners -- implement and verify exactly:**

- `test:cargo:extractum-prompt-packs::runtime::tests::start_service_preserves_idempotency_readiness_preflight_queue_and_execution_order`
- `test:cargo:extractum::prompt_packs::runtime_commands::tests::build_youtube_summary_execution_task_defers_profile_resolution_until_spawned_future_is_polled`

**SC-000515 exact mixed split:**

- Architecture ordinals `1, 11, 19`: `rule:extractum-grid-wrapper-boundary`.
- Behavior ordinals `2-10, 12-18, 20-21`: `test:vitest:src/lib/components/extractum-ui/extractum-grid-wrapper-boundary.behavior.component.test.ts#SVAR grid APIs stay inside Extractum wrappers`.

**Interfaces:** SC-000023 remains the real child/grandchild B3 owner in `os-integration`; the four approved component targets run only in `component`, and the remaining pure targets stay in `unit-node`. SC-000464 preserves ID, invariant, assertion count 5, class `B2_NEW_CHEAP_BEHAVIOR`, and behavior disposition, with joint service/application Cargo ownership. SC-000515 preserves ID, source hash, assertion count 21, lineage, class, and criticality with the exact mixed split above. The carrier accepts the immediately preceding Task 4 mapping only as the authorized correction input and fails closed on partial or arbitrary owner sets and ordinal partitions.

- [ ] **Step 1: Read and pin the exact titles**

Read all 17 Vitest target sections and SC-000464 from `tmp/analysis-smoke/slice-3c-replacement-map.md`. Record each complete Vitest replacement title in the `it` or `test` declaration without adding a `describe` prefix unless the frozen ID already contains one. Mechanically document SC-000515's exact 21-ordinal split before implementation.

- [ ] **Step 2: Add target-file REDs in two bounded groups**

Group A is `scripts/**`; Group B is `src/**`. Each Vitest target imports a production module or a behavior-neutral extracted adapter. Route targets test exported request/callback/selection logic under Node; they do not render Svelte and do not read route source text. SC-000464 uses an exact Rust RED/GREEN cycle for a behavior-neutral observer/dispatcher seam over the real service and application adapter; it must not use a JavaScript order model or Rust source-order assertion. SC-000515's behavior owner consumes a neutral configuration seam used by the real wrappers, while its architecture owner derives structured imports/styles from the repository index.

Run the exact new file paths through `node scripts/run-vitest.mjs run --project unit-node ...`, except SC-000023 through `--project os-integration`. Expected: every declaration is collected and fails for the missing observable behavior; no target reports zero tests.

- [ ] **Step 3: Implement the minimal observable seams**

Preserve the artifact invariant for each row:

- process/git/report targets exercise public serialization, pinning, and cleanup APIs with temporary fixtures;
- wrapper targets invoke exported runner/observation functions and assert real exit/cleanup outcomes;
- route targets call extracted request or callback builders with spies and assert arguments/results;
- grid/date/security targets import the public configuration object rather than production source text;
- SC-000416 drives the real polling controller and proves idle discovery through the light-refresh path.

- [ ] **Step 4: Run focused GREEN**

Run Group A and Group B as at most two concurrent frontend commands, then run SC-000023 under `os-integration` separately if it was not part of the OS command. Run the SC-000464 exact Cargo selection and both affected-package checks/checkpoints sequentially from the Rust Verification Loops; require one collected exact test. Run `npm.cmd run check` because TypeScript/Svelte production seams changed. Expected: all 17 Vitest declarations, the repository rule, and the comprehensive Cargo identity pass.

- [ ] **Step 5: Commit the additive targets**

Stage only the 17 Vitest targets, any narrowly required production seam files, the SC-000023 `OS_INTEGRATION_FILES` addition, and the approved SC-000464/SC-000515 plan/spec/artifact/carrier/ledger corrections. Remove the false SC-000464 Vitest target. Do not delete any legacy file in a later-wave component.

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

Expected: transition passes and the open-row count decreases by exactly 29. Review must confirm that SC-000383, SC-000384, and SC-000379 are present but their connected legacy components remain open.

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

Expected: transition passes and the open-row count decreases by exactly 11. If the browser harness is blocked, preserve the evidence, leave these three legacy files present, and continue Tasks 7–14 without claiming Slice 3C completion. A deferred browser batch resumes as this task after Wave 3 and before Task 15; its cutover then applies the Cutover Protocol's factually-last runner-removal rule.

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

- [ ] **Batch-specific execution:** Closes exactly 42 rows. Use direct module/API spies for Node route orchestration and rendered accessible behavior for `ApalisJobsPanel`; reuse the green diagnostics-count target from Task 5. Run `unit-node` and `component` as the two permitted frontend commands, then `os-integration` separately. Keep both new process paths in `OS_INTEGRATION_FILES` and remove the two deleted legacy process paths in this cutover. Follow the Cutover Protocol and commit as `test: cut over source contracts wave 2a`.

### Task 9: Cut over Wave 2B analysis contracts

Task 9 is corrected in two phases. The 44-row/per-ordinal table under
“Task 9 Exact Redisposition Amendment” in the approved design is incorporated
into this plan as the normative execution table; implementations must use its
complete `path#title` identities and may not reconstruct owners from the
provisional seven-file list. Its exact accounting table is:

| Category | Rows | Ordinals | Exact row IDs |
| --- | ---: | ---: | --- |
| A — unit reducer/API | 9 | 63 | SC-000093, SC-000094, SC-000108, SC-000114, SC-000117, SC-000119, SC-000124, SC-000126, SC-000127 |
| B — component behavior | 26 | 212 | SC-000060, SC-000061, SC-000062, SC-000063, SC-000065, SC-000066, SC-000067, SC-000068, SC-000069, SC-000070, SC-000097, SC-000098, SC-000100, SC-000101, SC-000102, SC-000103, SC-000104, SC-000109, SC-000110, SC-000111, SC-000112, SC-000113, SC-000120, SC-000122, SC-000123, SC-000125 |
| C — component/route composition | 3 | 21 | SC-000071, SC-000106, SC-000107 |
| D — delete D3 visual | 6 | 45 | SC-000072, SC-000073, SC-000074, SC-000075, SC-000076, SC-000105 |
| **Total** | **44** | **341** | — |

SC-000076 is the sixth D row because its negative inner-layout constraint is
non-observable in jsdom; no Playwright owner is added.

**Phase 1 — amendment and nine A owners only:**

- Preserve every row's `id`, `sourceHash`, `authorityHash`, `assertionCount`,
  and `lineage`, and preserve every historical review-evidence byte.
- Amend the artifact, carrier, carrier tests, and ledger atomically. The carrier
  accepts only the fully applied provisional Task 9 state or the exact corrected
  state, and rejects partial tables, arbitrary mutation, or partial ledger
  application.
- Implement only these existing `unit-node` targets:

```text
src/lib/analysis-evidence-source-navigation.behavior.test.ts           SC-000093-SC-000094
src/lib/analysis-redesign-route-contract.behavior.test.ts              SC-000108, SC-000114
src/lib/analysis-redesign-safety-contract.behavior.test.ts             SC-000117, SC-000119, SC-000124, SC-000126, SC-000127
```

- Remove the dead route-projection seam and the four provisional false
  unit-node B/C/D owners. Do not create any B/C target and do not run the
  transition validator while those future owners are unresolved.
- Prove focused carrier RED/GREEN, the exact A declaration titles and direct
  assertion counts `5+3+12+9+9+9+12+2+2=63`, all A owners together, and the
  carrier check. Do not stage or commit Phase 1.

**Phase 2 — future B/C component owners (not created in Phase 1):**

```text
src/lib/components/analysis/compact-source-rail.behavior.component.test.ts
src/lib/components/analysis/report-canvas.behavior.component.test.ts
src/lib/components/analysis/report-run-header.behavior.component.test.ts
src/lib/components/analysis/report-setup-panel.behavior.component.test.ts
src/lib/components/analysis/report-source-surface.behavior.component.test.ts
src/lib/components/analysis/report-workspace-tools.behavior.component.test.ts
src/lib/components/analysis/run-companion-runs-tab.behavior.component.test.ts
src/lib/components/analysis/run-companion-tabs.behavior.component.test.ts
src/lib/components/analysis/source-activity-view.behavior.component.test.ts
src/lib/components/analysis/source-group-sources-view.behavior.component.test.ts
src/lib/components/analysis/source-reader-header.behavior.component.test.ts
src/lib/components/analysis/source-switcher-panel.behavior.component.test.ts
src/lib/components/analysis/universal-items-view.behavior.component.test.ts
src/lib/components/analysis/youtube-playlist-videos-view.behavior.component.test.ts
src/lib/components/analysis/youtube-transcript-reader.behavior.component.test.ts
src/routes/analysis/analysis-route-composition.behavior.component.test.ts
src/routes/analysis/analysis-route-llm-controls.behavior.component.test.ts
```

SC-000071 and SC-000106 have exhaustive, non-overlapping ordinal subgroups and
no top-level disposition. SC-000107 is the third C row but is a whole-row
route-composition behavior owner. The six D rows and the delete
subgroups of SC-000071/106 gain no Playwright replacement.

**Phase 2 delete exactly after all A/B/C owners are green:**

```text
src/lib/analysis-compact-source-rail.test.ts
src/lib/analysis-companion-layout.test.ts
src/lib/analysis-evidence-source-navigation.test.ts
src/lib/analysis-llm-run-controls.test.ts
src/lib/analysis-priority-ux-contract.test.ts
src/lib/analysis-redesign-route-contract.test.ts
src/lib/analysis-redesign-safety-contract.test.ts
```

- [ ] **Phase-2 batch-specific execution:** Closes exactly 44 validator-open
  rows while deleting seven legacy files containing 50 total ledger rows; the
  six omitted rows were already closed earlier and must not be duplicated.
  Run the three A files in one non-empty `unit-node` selection and all 17 B/C
  files in one non-empty `component` selection. Follow the Cutover Protocol and
  commit as `test: cut over source contracts wave 2b` only after both selections
  are green and the exact transition validator succeeds.

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

- [ ] **Batch-specific execution:** Closes exactly 58 rows. Run the eight targets together in `unit-node`. Route tests use module/API fixtures, not rendered `+page.svelte` source inspection; saved-run restoration, source-window loading, tab state, and report-canvas props remain independently observable. Follow the Cutover Protocol and commit as `test: cut over source contracts wave 2c`.

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

- [ ] **Batch-specific execution:** Closes exactly 56 rows. Run the three `src/lib/analysis-*` targets in one `unit-node` command and the four `src/lib/components/**` targets in one `component` command. Use API/event fixtures for source readers and YouTube specialization; rendered grids prove accessible host state, row/cell behavior, callbacks, and jsdom-observable responsive decisions, not clipping or layering. SC-000323 and SC-000344 remain owned by Task 6. Follow the Cutover Protocol and commit as `test: cut over source contracts wave 2d`.

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

- [ ] **Batch-specific execution:** Closes exactly 65 rows. Run all Node paths in one `unit-node` command and all component paths in one `component` command; four connected components require both owner selections and remain in this single cutover: `gemini-browser-provider-panel`, `project-runs-screen`, `research-projects-import-boundary`, and `source-access-placement`. Use actual API adapters and accessible workflows. The import-boundary target uses only its approved structured seam. Follow the Cutover Protocol and commit as `test: cut over source contracts wave 2e`.

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

**Interfaces:** This task is additive. It does not delete any of the nine Wave 3 legacy files because four extracted-package identities are still missing.

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
SC-000441 extractum-llm::public_api_tests::curated_api_keeps_credentials_non_serializable_and_inaccessible
SC-000474 extractum-prompt-packs::public_api_tests::cancellation_smoke_services_remain_test_only
SC-000484 extractum-prompt-packs::run_control::tests::terminal_events_clear_required_run_state
SC-000492 extractum-prompt-packs::runtime_config::tests::invalid_persisted_runtime_configuration_is_reported
```

**Delete exactly after the 17 Wave 3 Cargo identities, the comprehensive SC-000464 Cargo owner, and Task 4 Node companions are green:**

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

**Interfaces:** Closes 9 files and 55 rows. Package ownership is `extractum-llm` for SC-000441 and `extractum-prompt-packs` for SC-000474/484/492. SC-000087 is an exact D2 deletion with no replacement owner.

- [ ] **Step 1: Add exact RED/GREEN tests per package**

Use the Rust Verification Loops section. SC-000441's Cargo owner proves only the behavior ordinals: negative Serialize, Deserialize, Debug, and ExposeSecret traits plus URL-userinfo rejection; its external dependent-crate compile-pass/compile-fail probes remain supplementary. The exhaustive curated root/type/field/method boundary is owned by `rule:extractum-llm-public-api-boundary`. SC-000474 must combine real private seed/clear behavior with external feature-off compile failure and `dev-fixtures` feature-on compile success. If a test-support feature is required, follow the producer feature-off and consumer feature-on rules from `AGENTS.md`.

- [ ] **Step 2: Run package checks and checkpoints sequentially**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-llm --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets
```

- [ ] **Step 3: Perform the Wave 3 cutover**

Delete the nine exact legacy files above and follow the Cutover Protocol, including its factually-last runner-removal rule. The batch closes exactly 55 rows whether the independent browser batch ran earlier or remains deferred.

- [ ] **Step 4: Validate, review, and commit**

```powershell
node scripts/validate-testing-transition.mjs
git diff --check
git commit -m "test: cut over source contracts wave 3"
```

Expected: transition passes and the open-row count decreases by exactly 55.

### Task 15: Verify once and hand off Slice 4

**Files:**
- Create: `docs/superpowers/verification/2026-08-08-testing-redesign-slice-3c.md`
- Modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`

**Interfaces:** This task starts only after Tasks 6 and 14 have both cut over and the Cutover Protocol has removed the empty legacy runner. It records no benchmark series: one preflight, one retained wall-clock observation for the single final `verify`, and no warmup/median/threshold.

- [ ] **Step 1: Prove the zero-inventory state with focused gates**

Run sequentially:

```powershell
node scripts/testing/source-contract-redisposition-review.mjs check
node scripts/validate-testing-transition.mjs
npm.cmd run check
git diff --check
```

Expected: carrier changed paths empty; ledger 671 rows/0 open; census has no `vitest:legacy-contract`; no tracked legacy file exists; Svelte/TypeScript check is clean.

- [ ] **Step 2: Bootstrap the fresh worktree and run the process preflight**

```powershell
npm.cmd run bootstrap:testing
```

Use the established PowerShell/CIM preflight to require zero competing repository-owned Vitest, Playwright, Cargo, Rust, Vite, or Chromium processes. Record the exact command and result. Do not start a warmup.

- [ ] **Step 3: Run exactly one final full verification**

Run unsandboxed on Windows:

```powershell
npm.cmd run verify
```

Retain its exit code and one wall-clock duration. Do not rerun for a correctness failure. A retry is allowed only for a demonstrated spawn/sandbox/process-infrastructure failure, and both observations must remain in evidence.

- [ ] **Step 4: Write final evidence and update the program index**

The verification document records:

- all 9 cutover batches and their commits;
- 436 closed rows, 86 deleted legacy files, 79 completed components;
- final Node/jsdom/Cargo/Playwright owner counts;
- the process preflight and sole full-verify observation;
- zero open ledger rows and removal of the legacy runner;
- any browser deferral and its eventual factual-last cutover;
- no Nx decision and no performance threshold claim.

Update the program index to mark Slice 3 complete and identify Slice 4 as next. Do not claim Slice 3C complete if any browser component, legacy file, or legacy runner remains.

- [ ] **Step 5: Final read-only review and evidence commit**

Run one read-only spec/quality review over the complete Slice 3C commit range and verification record. Resolve findings in one bounded fix wave, rerun only the directly affected focused gate, and do not repeat full `verify` unless the retry exception above applies.

```powershell
git diff --check
git commit -m "docs: record slice 3c source-contract completion"
```

Expected final state: tracked worktree clean, source-contract ledger `671/0`, no legacy runner, final verification recorded, and Slice 4 ready to begin.
