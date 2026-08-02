# Testing Redesign Slice 2B: Project and Browser Ownership Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single Vitest owner with disjoint non-empty projects, make Playwright the only Chromium lifecycle owner, and prove the migrated lifecycle with twenty no-retry runs.

**Architecture:** `vitest.config.ts` directly owns five named projects: `unit-node`, `component`, `architecture`, `legacy-contract`, and `os-integration`. The frozen Slice 2A ledger supplies the explicit legacy file list; the live runner census independently proves total and disjoint ownership. The existing Playwright configuration remains one census owner and gains one path-filterable Chromium lifecycle spec using only Playwright's built-in `page` fixture.

**Tech Stack:** Node.js ESM, TypeScript, Vitest 4.1.5, Vite 6, Svelte Testing Library, Playwright 1.61, Windows PowerShell.

## Global Constraints

- Implement `docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md`, with the explicit transitional deviation below.
- Use `npm.cmd`, never plain `npm`, for repository commands on Windows.
- Keep `verify` sequential, fail-closed, and coverage-equivalent until Slice 6 replaces its scheduler.
- Add no timing measurement, timing state, cache, threshold, custom glob matcher, or third-party dependency.
- Chromium is Playwright-only after this slice. No Vitest candidate may import Playwright's `chromium` launcher.
- Keep the one existing Playwright census owner and the existing census schema. Do not add Playwright projects or owner arguments.
- Playwright retries remain `0`, worker count remains `1`, and browser commands do not overlap other gates.
- The twenty-run lifecycle audit is acceptance evidence, not a p95 calculation or a performance gate; it never retries a failed run.
- Do not add Nx, Nx Cloud, cargo-nextest, Allure, GitHub Actions, branch protection, remote cache, or hosted reporting.
- End the slice with clean `git status --short` and successful `npm.cmd run verify`.

## Proposed Transitional Deviation

The approved design says mixed source-contract files are split before project ownership. This plan deliberately defers that mechanical split to Slice 3:

- the 91-file legacy inventory is derived from the frozen ledger rather than encoded in filenames;
- semantic owners take precedence for the 19 component files and six OS-integration files;
- every other ledger path belongs to `legacy-contract` and is excluded from `unit-node`;
- behavioral assertions colocated in a legacy file temporarily run with `legacy-contract`;
- source-contract obligations colocated in component or OS-integration files remain tracked by stable ledger rows even though their file owner is the semantic project;
- no new path can enter `legacy-contract` without a committed ledger change, and the transition validator rejects unaccounted runner ownership.

This avoids renaming or splitting approximately 91 files in Slice 2B. Only the 19 actual jsdom component files are renamed so component ownership is visible and stable. Ledger paths/lineage are updated only for renamed component files that have rows.

## Exact Project Partition

`vitest.config.ts` derives these arrays and places them directly in Vitest project `include`/`exclude` fields; there is no parallel classifier:

1. `componentPattern`: `src/**/*.component.test.ts`; suffix ownership is the permanent component convention.
2. `osIntegrationFiles`: the six files collected by `scripts/process-shell-diagnostic/**/*.test.ts`.
3. `architectureFiles`: exactly `src/lib/lucide-direct-import-contract.test.ts`, confirmed absent from the frozen ledger.
4. `ledgerFiles`: unique normalized `row.path` values loaded from `testing/source-contract-ledger.json`, with renamed component paths updated by existing stable-ID lineage rules.
5. `legacyFiles`: `ledgerFiles` minus paths matching `componentPattern`, `osIntegrationFiles`, and `architectureFiles`.
6. `unit-node`: normal repository Vitest globs excluding `componentPattern`, the other owned arrays, and Playwright specs.

Vitest performs the actual glob and file collection. `node scripts/validate-testing-transition.mjs` remains the only totality/disjointness authority and rejects `empty owner`, `duplicate ownership`, `unowned filesystem candidate`, and runner failures against real collection.

## File Map

| File | Responsibility |
| --- | --- |
| Create `vitest.config.ts` | Defines five projects and derives the legacy list from the ledger. |
| Modify `vite.config.js` | Keeps only shared Vite/Svelte configuration. |
| Modify `scripts/run-vitest.mjs` | Uses the root config while preserving path normalization and worktree exclusion. |
| Create `scripts/testing/test-conventions.test.ts` | Proves component/config conventions and rejects Chromium launcher imports without creating ledger readers. |
| Rename 19 component tests | Makes the jsdom boundary explicit without touching other legacy files. |
| Modify `testing/source-contract-ledger.json` | Updates only rows affected by component renames. |
| Modify `testing/runner-census.json` | Replaces `vitest:root` with five Vitest owners; Playwright stays unchanged. |
| Create `research/gemini_browser_adapter/tests/chromium-lifecycle.spec.ts` | Ports answer-extractor assertions to Playwright's `page` fixture. |
| Delete `sidecars/gemini-browser/src/answer-extractor.test.ts` | Removes Chromium and teardown ownership from Vitest. |
| Create `scripts/verify-stability.mjs` and `scripts/verify-stability.test.ts` | Implements and tests the twenty-run lifecycle audit. |
| Modify `package.json`, `scripts/verify.mjs`, and `scripts/verify.test.ts` | Adds complete owner commands and preserves the sequential full gate. |
| Create `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md` | Records evidence without timing statistics. |
| Modify `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md` | Records the completed checkpoint. |

## Tasks

### Task 1: Establish component ownership and cleanup

**Files:** Rename these files from `.test.ts` to `.component.test.ts`:

```text
src/lib/components/research-projects/ComboSelect.test.ts
src/lib/components/research-projects/Inspector.test.ts
src/lib/components/research-projects/OptionsPanel.test.ts
src/lib/components/research-projects/PeriodPanel.test.ts
src/lib/components/research-projects/PeriodPopover.test.ts
src/lib/components/research-projects/ProjectRailPanel.test.ts
src/lib/components/research-projects/ProjectRow.test.ts
src/lib/components/research-projects/ProjectTabs.test.ts
src/lib/components/research-projects/ProjectToolbar.test.ts
src/lib/components/research-projects/ResearchProjectsShell.test.ts
src/lib/components/research-projects/RunDock.test.ts
src/lib/components/research-projects/SourcesBulkBar.test.ts
src/lib/components/research-projects/SourcesFilterBar.test.ts
src/lib/components/research-projects/SourcesFilterRow.test.ts
src/lib/components/research-projects/SourcesTab.test.ts
src/lib/components/research-projects/SourceStatusCell.test.ts
src/lib/components/research-projects/SourceTitleCell.test.ts
src/lib/ui/research-projects-source-keyboard.test.ts
src/lib/youtube-thumbnail.test.ts
```

Also modify `testing/source-contract-ledger.json` and create `scripts/testing/test-conventions.test.ts`.

**Interfaces:** Every component file has exactly one `afterEach(cleanup)`. Stable ledger IDs/titles stay unchanged; renamed rows retain the former path in lineage.

- [ ] **Step 1: Add a RED audit to new `scripts/testing/test-conventions.test.ts`.** Make four separate calls, each with one literal pattern, then merge their returned records:

```ts
const sourceTests = import.meta.glob("/src/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const scriptTests = import.meta.glob("/scripts/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const sidecarTests = import.meta.glob("/sidecars/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const researchTests = import.meta.glob("/research/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const testSources = { ...sourceTests, ...scriptTests, ...sidecarTests, ...researchTests } as Record<string, string>;
```

Each literal resolves to non-empty tracked test authority, so source-reader discovery classifies it as `test`. Build the environment marker as `"@vitest-environment " + "jsdom"` so the audit does not match its own source. Assert matching files have `.component.test.ts` paths, import `cleanup`, and contain one `afterEach(cleanup)`. Do not use an array glob, `node:fs`, dynamic file reads, directory enumeration, or subprocesses.
- [ ] **Step 2: Run `npm.cmd run test -- scripts/testing/test-conventions.test.ts`.** Expected: FAIL listing the 19 current paths.
- [ ] **Step 3: Rename the 19 files.** Add cleanup only when missing, remove duplicate registrations, and update only affected ledger paths/lineage. Do not alter assertions or production code.
- [ ] **Step 4: Run the convention test, ledger unit test, and live transition validator.**

```powershell
npm.cmd run test -- scripts/testing/test-conventions.test.ts
npm.cmd run test -- scripts/testing/validate-testing-transition.test.ts
node scripts/validate-testing-transition.mjs
```

Expected: all exit `0`; renamed paths and lineage validate before the task commit.
- [ ] **Step 5: Commit.**

```powershell
git add src testing/source-contract-ledger.json scripts/testing/test-conventions.test.ts
git commit -m "test: establish component test ownership"
```

### Task 2: Define real Vitest projects and update the census

**Files:** Create `vitest.config.ts`; modify `vite.config.js`, `scripts/run-vitest.mjs`, `scripts/testing/test-conventions.test.ts`, `testing/runner-census.json`, and `package.json`.

**Interfaces:** Named projects are `unit-node`, `component`, `architecture`, `legacy-contract`, and `os-integration`. Commands are `test:unit`, `test:component`, `test:architecture`, `test:legacy-contract`, and `test:integration:os`. Census owners use the already-supported Vitest `args` field with `--project <name>`.

- [ ] **Step 1: Write RED config assertions in `test-conventions.test.ts`.** Import `package.json` as JSON and named data exports `VITEST_PROJECT_DEFINITIONS` and `LEGACY_TEST_FILES` from `vitest.config.ts`; assert five commands, five names, and the expected deduplicated legacy paths. Never import the default Vitest config in this test, because doing so would execute SvelteKit/Tailwind plugin factories. Do not read files through `fs` or recreate glob matching.
- [ ] **Step 2: Run `npm.cmd run test -- scripts/testing/test-conventions.test.ts`.** Expected: FAIL because the config and commands do not exist.
- [ ] **Step 3: Implement `vitest.config.ts`.** Import `testing/source-contract-ledger.json` as JSON, normalize unique ledger paths, subtract paths ending in `.component.test.ts` plus exact OS/architecture owners, and export the resulting frozen data as named `LEGACY_TEST_FILES` and `VITEST_PROJECT_DEFINITIONS`. Make the default export `defineConfig(() => ({ ... }))` and instantiate SvelteKit/Tailwind/Svelte Testing plugins only inside that callback, so importing named data exports has no plugin side effects. Configure component include as `src/**/*.component.test.ts` and unit exclusion with that same glob; new component tests therefore need no config edit. Use Node/threads for unit, architecture, legacy; jsdom/threads for component; Node/forks for OS. Preserve `svelteTesting()` for the component project while reusing shared SvelteKit/Tailwind Vite plugins. Remove only the competing root `test` owner from `vite.config.js`; keep `.worktrees/**` excluded at the runner boundary.
- [ ] **Step 4: Update census JSON only.** Replace `vitest:root` with five project-filtered Vitest owners. Do not change `testing-transition.mjs`, its tests/schema, or the Playwright owner.
- [ ] **Step 5: Prove real collection.**

```powershell
npm.cmd run test -- scripts/testing/test-conventions.test.ts
node scripts/run-vitest.mjs list --filesOnly --no-color --project unit-node
node scripts/run-vitest.mjs list --filesOnly --no-color --project component
node scripts/run-vitest.mjs list --filesOnly --no-color --project architecture
node scripts/run-vitest.mjs list --filesOnly --no-color --project legacy-contract
node scripts/run-vitest.mjs list --filesOnly --no-color --project os-integration
node scripts/validate-testing-transition.mjs
```

Expected: every list is non-empty; transition validation exits `0` and owns every candidate once.

- [ ] **Step 6: Commit.**

```powershell
git add vitest.config.ts vite.config.js scripts/run-vitest.mjs scripts/testing/test-conventions.test.ts testing/runner-census.json package.json
git commit -m "test: split Vitest project ownership"
```

### Task 3: Move answer extraction to the existing Playwright owner

**Files:** Create `research/gemini_browser_adapter/tests/chromium-lifecycle.spec.ts`; delete `sidecars/gemini-browser/src/answer-extractor.test.ts`; modify `scripts/testing/test-conventions.test.ts`.

**Interfaces:** The new spec uses built-in `{ page }`, production answer-extractor functions, and `page.setContent()`. It starts no server, browser, context, or page and registers no teardown. Existing adapter specs/config remain unchanged.

- [ ] **Step 1: Port the suite as a RED Playwright spec.** Copy `shell()` and all behavior assertions; replace Vitest browser globals/hooks with Playwright test callbacks receiving `{ page }`.
- [ ] **Step 2: Run `npm.cmd run test:gemini-browser-adapter:e2e -- chromium-lifecycle`.** Expect a meaningful RED during conversion, then a non-empty PASS after completing the port. The basename filter avoids Windows path-separator ambiguity.
- [ ] **Step 3: Delete the Vitest file and add a static prohibition.** Reuse the four static raw-source glob records in `scripts/testing/test-conventions.test.ts`. Match import-statement syntax—an import clause containing identifier `chromium` whose module specifier is `@playwright/test`—rather than independent substrings. The regex/predicate source must not itself resemble a matching import statement, so the audit cannot match itself. Do not enumerate or read files through `fs`.
- [ ] **Step 4: Verify.**

```powershell
npm.cmd run test -- scripts/testing/test-conventions.test.ts
npm.cmd run test:gemini-browser-adapter:e2e -- chromium-lifecycle
node scripts/validate-testing-transition.mjs
```

Expected: PASS; census retains one Playwright owner and moves the candidate from Vitest to it.

- [ ] **Step 5: Commit.**

```powershell
git add research/gemini_browser_adapter/tests/chromium-lifecycle.spec.ts sidecars/gemini-browser/src/answer-extractor.test.ts scripts/testing/test-conventions.test.ts
git commit -m "test: move Chromium extraction to Playwright"
```

### Task 4: Add the minimal twenty-run lifecycle audit

**Files:** Create `scripts/verify-stability.mjs` and `scripts/verify-stability.test.ts`; modify `package.json`.

**Interfaces:** Public command is `npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20`. Only those values are accepted; invalid input exits `2`. `runChromiumLifecycleAudit({ runCommand, listProcesses })` never retries. The Playwright child command is `process.execPath` plus `require.resolve("@playwright/test/cli")`, never a `.cmd` shim.

- [ ] **Step 1: Write RED tests.** Cover exact argument parsing, resolved Node/Playwright CLI arguments, stop after first failed normal run, spawn/enumeration error `3`, and before/after process comparison. A newly created process with Playwright headless-Chromium and temporary `--user-data-dir` markers fails; a pre-existing match and ordinary user Chrome do not.
- [ ] **Step 2: Run `npm.cmd run test -- scripts/verify-stability.test.ts`.** Expected: FAIL because the module does not exist.
- [ ] **Step 3: Implement the bounded command.** Resolve `@playwright/test/cli` through `createRequire(import.meta.url)` and spawn `process.execPath` with these arguments twenty times sequentially using `shell: false`:

```text
<node> <resolved @playwright/test/cli> test -c research/gemini_browser_adapter/playwright.config.ts chromium-lifecycle
```

Take before/after snapshots by spawning `powershell.exe -NoProfile -NonInteractive -Command "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine | ConvertTo-Json -Compress"`. Print only `chromium-lifecycle run N/20: pass|fail`. Stop at the first failure. Attribute only a PID absent from the before snapshot whose command line has Playwright headless-Chromium plus its temporary `--user-data-dir` marker. The lifecycle spec starts no server, so do not add mock-server matching. Exit `1` for failed run/leak, `2` for invalid input, `3` for spawn/CIM/JSON error.

- [ ] **Step 4: Add `"verify:stability": "node scripts/verify-stability.mjs"` and run the unit test.** Expected: PASS.
- [ ] **Step 5: Run `npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20`.** Expected: twenty passes, no retry, no newly leaked Playwright browser process. Preserve failures rather than replacing them with reruns.
- [ ] **Step 6: Commit.**

```powershell
git add package.json scripts/verify-stability.mjs scripts/verify-stability.test.ts
git commit -m "test: add Chromium lifecycle audit"
```

### Task 5: Preserve complete verification and record evidence

**Files:** Modify `package.json`, `scripts/verify.mjs`, `scripts/verify.test.ts`, and `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`; create `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md`.

**Interfaces:** `test` remains an npm compatibility command for manual all-project Vitest use, but it is removed from `verify`. `test:e2e` runs the one existing complete Playwright owner. `verify` replaces its old `test` step with the five Vitest project commands, then runs `test:e2e` and all preserved non-Vitest static/Rust gates sequentially and fail-closed. No Vitest file runs twice.

- [ ] **Step 1: Write RED verify ordering assertions.** Require `test:unit`, `test:component`, `test:architecture`, `test:legacy-contract`, `test:integration:os`, and `test:e2e`; assert `test` is absent from `createVerifySteps()`. Retain the binary check, transition validator, `check`, rustfmt, Cargo check/test, and `git diff --check`.
- [ ] **Step 2: Run `npm.cmd run test -- scripts/verify.test.ts`.** Expected: FAIL until commands are wired.
- [ ] **Step 3: Add `"test:e2e": "npm.cmd run test:gemini-browser-adapter:e2e"` as an alias and replace the old `test` verify step with five project steps.** Keep `test` available outside `verify` and keep the existing adapter command as the single Playwright command definition. Do not add projects, scheduler, timing logger, or retry wrapper.
- [ ] **Step 4: Run focused owners.**

```powershell
npm.cmd run test -- scripts/verify.test.ts
npm.cmd run test:unit
npm.cmd run test:component
npm.cmd run test:architecture
npm.cmd run test:legacy-contract
npm.cmd run test:integration:os
npm.cmd run test:e2e
node scripts/validate-testing-transition.mjs
```

Expected: all pass with non-empty selection and exact-once census ownership.

- [ ] **Step 5: Run `npm.cmd run verify`.** Expected: exit `0`. Record final commit, exact commands, five Vitest counts, one Playwright count, twenty lifecycle outcomes, process-audit result, and final verify result. Record no durations, percentiles, CPU, or memory.
- [ ] **Step 6: Commit the checkpoint.**

```powershell
git add package.json scripts/verify.mjs scripts/verify.test.ts docs/superpowers/verification/2026-08-02-testing-redesign-slice-2b-project-browser-ownership.md docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md
git commit -m "docs: record Slice 2B ownership evidence"
git status --short
```

Expected: no tracked changes.

## Plan Self-Review

- Only 19 component files are renamed; the 91-file ledger inventory is not pre-migrated.
- Vitest/Playwright config and the live census are the only classification authorities.
- One Playwright census owner collects existing adapter specs and the lifecycle spec.
- Existing adapter specs, mock-server hooks, and Playwright config stay unchanged.
- `architecture` is non-empty through exact non-ledger `src/lib/lucide-direct-import-contract.test.ts`.
- Permanent convention guards live in non-ledger `scripts/testing/test-conventions.test.ts` and use only static raw/module imports, so they create no source-reader obligations.
- `verify` replaces, rather than duplicates, the old all-Vitest gate; every Vitest file runs once.
- Windows launches Playwright through Node's resolved CLI and audits narrow CIM command-line markers for newly leaked headless Chromium only.
- Slice 2C remains the next checkpoint and is still a disposable Nx decision spike.
