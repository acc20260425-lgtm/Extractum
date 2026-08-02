# Testing Redesign Slice 2A Migration Preflight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the current test inventory and every arbitrary-source test obligation explicit before tests move, while keeping daily timing infrastructure exactly as small as it was after Slice 1.

**Architecture:** First close two already-approved Slice 1 carry-overs in separate prerequisite commits, then treat their clean HEAD as the Slice 2A base. Add one dependency-free transition validator to the beginning of the current sequential `verify`. Its runner census dynamically compares Git-visible test candidates with list-only Vitest and Playwright collection, so the committed census stores owner definitions and exact exceptions rather than a duplicated file list. A bounded TypeScript-AST extractor freezes the current source-dependent declarations into a reviewed ledger; daily validation checks the ledger statically and never performs the expensive full Vitest declaration listing.

**Tech Stack:** Node.js ESM, the already-installed TypeScript compiler API, Vitest 4.1.5 list mode, Playwright 1.61 JSON list reporter, Git, Windows PowerShell.

## Global Constraints

- Tasks 1–2 are the separately reviewed signal-race and fresh-checkout carry-overs already accepted after Slice 1. Commit them first and record their resulting clean HEAD as the implementation base for Slice 2A proper (Tasks 3–6).
- Implement Slice 2A of `docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md` on that prerequisite base.
- Do not add or change any timing mechanism. Do not add per-test/per-file durations, startup probes, benchmarks, timestamps, timing thresholds, fingerprints for performance, rolling counters, timing summaries, or performance state.
- Do not modify `scripts/testing/timing-log.mjs`, `artifacts/testing/timings.jsonl`, the five-field timing schema, or the Rust feasibility classification.
- The only hashes in this slice are migration-ledger content hashes. They protect obligation identity; they are not performance fingerprints and never affect test eligibility.
- Do not add `TestingManifest`, Vitest projects, browser migration, a scheduler, selectors, watch behavior, `verify:performance`, dependency-cruiser, Knip, coverage policy, quarantine, GitHub Actions, or branch protection.
- Do not relocate, split, rewrite, replace, or delete existing source-contract tests in Slice 2A. This slice freezes obligations, not the physical legacy-file inventory.
- Keep current `verify` correctness coverage and fail-fast behavior. Add only two early correctness gates: sidecar prerequisite first, transition validation second.
- `verify` must never build the ignored sidecar. A fresh checkout uses one explicit `npm.cmd run bootstrap:testing` command before `verify`.
- Use `npm.cmd`, not plain `npm`, in Windows instructions. Package scripts may use npm's platform-neutral `npm run ...` internally.
- Use current dependencies only. The TypeScript compiler API is sufficient for bounded extraction; unsupported syntax becomes an explicit manual ledger row.
- Commit runner census configuration and the completed ledger. Store generated drafts and diagnostics only under ignored `artifacts/testing/slice-2a/`.
- Preserve unrelated user changes, stage only task files, and do not push.
- End the slice with a clean worktree and one successful `npm.cmd run verify`.

## Frozen Baseline and Deliberate Simplicity

- At `72a00b38`, the Git-visible filesystem census is 187 files: 181 Vitest `.test.ts` files and 6 Playwright `.spec.ts` files. The sets are disjoint and require no exceptions.
- This plan adds exactly three Vitest files before census freeze: `scripts/check-gemini-browser-sidecar-binary.test.ts`, `scripts/verify.test.ts`, and `scripts/testing/validate-testing-transition.test.ts`. The frozen Slice 2A census is therefore expected to close at 190 files: 184 Vitest plus 6 Playwright, with zero exceptions.
- Read-only AST audit at `72a00b38` found 84 potential source-reader files, 778 declarations in those files, and 652 source-dependent declarations. These are reconciliation expectations, not constants embedded in the validator. Nineteen files mix source-dependent and ordinary declarations, so file-level ledger rows are forbidden.
- The committed runner census does not repeat 190 paths. It stores two owner definitions plus exact exception arrays; the validator derives current file sets on every run.
- The expensive `vitest list --json` declaration inventory runs once during ledger freeze. Daily transition validation uses `vitest list --filesOnly` and static AST checks only.
- No command in this plan records elapsed time.

## Execution Protocol

- Execute tasks in order with a fresh implementation subagent per task and specification/code review before proceeding.
- Each task ends in the named commit. The implementing agent stages only the listed files.
- Reconcile the extractor result with the read-only 84/778/652 audit before promotion. If the result differs, preserve an exact added/removed declaration report under `artifacts/testing/slice-2a/`, determine whether the audit or bounded extractor is wrong, and amend the plan/evidence before freezing. Do not hard-code these counts or add exceptions merely to reproduce them.
- If the runner census does not close at 184 Vitest plus 6 Playwright files, treat the mismatch as a correctness failure. Do not add a broad exception to make it green.

---

### Task 1 (Approved Prerequisite): Close the Slice 1 Pre-Dispatch Signal Window

**Files:**
- Modify: `scripts/testing/slice-1-rust-feasibility.mjs`
- Modify: `scripts/testing/slice-1-rust-feasibility.test.ts`

**Interface:** Keep all public exports and the report schema unchanged. Add only a synchronous interruption check after the final pre-command await and before `runCommand` is scheduled.

- [ ] **Step 1: Write the failing regression**

Add a test whose injected `filesystem.stat` handles the pre-`endToEnd` executable mtime read by invoking the captured SIGINT handler. Assert that:

- the source had already been mutated;
- `runCommand` is not called for that `endToEnd` entry after the signal;
- the observation is marked interrupted rather than infrastructure-failed;
- original source bytes are restored and verified;
- the study exits 130.

- [ ] **Step 2: Run RED**

```powershell
node scripts/run-vitest.mjs run scripts/testing/slice-1-rust-feasibility.test.ts
```

Expected: the new test fails because the suspended loop currently starts Cargo after the signal handler has restored the source.

- [ ] **Step 3: Add the one guard**

Immediately before creating the `AbortController`/settlement, branch on the already-set `interrupted` flag. Produce the existing interrupted observation shape, skip `runCommand`, let the existing `finally` restoration execute, and break through the existing post-observation interruption condition. Do not add a timeout, timer, retry, signal generation counter, or new report field.

- [ ] **Step 4: Run GREEN and focused cancellation tests**

```powershell
node scripts/run-vitest.mjs run scripts/testing/slice-1-rust-feasibility.test.ts scripts/testing/run-observation.test.ts
```

Expected: PASS; no Cargo process is started by the injected-stat signal case.

- [ ] **Step 5: Commit**

```powershell
git add scripts/testing/slice-1-rust-feasibility.mjs scripts/testing/slice-1-rust-feasibility.test.ts
git commit -m "fix: stop rust study before post-signal dispatch"
```

---

### Task 2 (Approved Prerequisite): Make the Ignored Sidecar an Explicit Test Bootstrap

**Files:**
- Modify: `scripts/check-gemini-browser-sidecar-binary.mjs`
- Create: `scripts/check-gemini-browser-sidecar-binary.test.ts`
- Modify: `package.json`
- Modify: `docs/project.md`
- Modify: `README.md`
- Modify: `AGENTS.md`

**Interfaces:**
- Export `inspectGeminiBrowserSidecar({ repoRoot, targetTriple, platform, requestedTarget, lstatSyncImpl })`.
- Add package script `bootstrap:testing = npm run build:gemini-browser-sidecar && npm run check:gemini-browser-sidecar-binary`.
- Keep `build:tauri-prereqs`, Tauri `beforeBuildCommand`, `.gitignore`, and the build script unchanged.

- [ ] **Step 1: Write strict checker tests**

Cover missing path, empty file, directory, symlink, non-empty regular file, and requested-target/host-target mismatch. Inject `lstatSyncImpl` so symlink behavior does not require Windows symlink privileges. Every prerequisite error must include exactly:

```text
Run: npm.cmd run bootstrap:testing
```

- [ ] **Step 2: Run RED**

```powershell
node scripts/run-vitest.mjs run scripts/check-gemini-browser-sidecar-binary.test.ts
```

Expected: FAIL because the current script has no import-safe pure function and accepts any existing path.

- [ ] **Step 3: Implement a correctness-only check**

Use `lstat`, reject a missing path, symlink, directory, non-regular file, or zero-byte file, and return the normalized relative path plus size for a valid file. Keep the existing host-target rule. Do not add hashes, mtimes, freshness rules, or automatic rebuilds.

- [ ] **Step 4: Add and document explicit bootstrap**

Document the workflow:

```powershell
npm ci
npm.cmd run bootstrap:testing
npm.cmd run verify
```

State that bootstrap is required once in a fresh checkout/worktree and after sidecar-packaging changes; it may download the `pkg` runtime cache. State that `verify` only inspects the prerequisite and never builds it. Preserve the separate release behavior in which `tauri build` invokes `build:tauri-prereqs`.

- [ ] **Step 5: Exercise the explicit bootstrap**

```powershell
npm.cmd run bootstrap:testing
```

Expected: the host-target sidecar is built and the strict checker accepts it. This is an explicit prerequisite action, not a `verify` step and not a timed observation.

- [ ] **Step 6: Run GREEN**

```powershell
node scripts/run-vitest.mjs run scripts/check-gemini-browser-sidecar-binary.test.ts
npm.cmd run check:gemini-browser-sidecar-binary
```

Expected: unit tests pass and the current checkout's ignored non-empty regular sidecar is accepted.

- [ ] **Step 7: Commit**

```powershell
git add scripts/check-gemini-browser-sidecar-binary.mjs scripts/check-gemini-browser-sidecar-binary.test.ts package.json docs/project.md README.md AGENTS.md
git commit -m "test: make sidecar bootstrap explicit"
```

---

### Task 3: Atomically Add the Transition Carrier and Live Runner Census

**Files:**
- Create: `scripts/testing/testing-transition.mjs`
- Create: `scripts/testing/validate-testing-transition.test.ts`
- Create: `scripts/validate-testing-transition.mjs`
- Create: `scripts/verify.test.ts`
- Create: `testing/runner-census.json`
- Modify: `scripts/verify.mjs`

**Committed census schema:**

```json
{
  "schemaVersion": 1,
  "vitestOwners": [
    { "id": "vitest:root", "args": [], "ownerScript": "test" }
  ],
  "playwrightOwners": [
    {
      "id": "playwright:gemini-browser-adapter",
      "config": "research/gemini_browser_adapter/playwright.config.ts",
      "ownerScript": "test:gemini-browser-adapter:e2e"
    }
  ],
  "nonstandardTests": [],
  "fixtureExceptions": []
}
```

Do not add a generated `files` array.

**Interfaces:**
- Export `runTransitionValidation({ repoRoot, checks, stdout, stderr })` from `scripts/testing/testing-transition.mjs`.
- Export `createVerifySteps({ npmExecPath, platform })` and `runVerification({ steps, runStep })` from `scripts/verify.mjs`; keep direct CLI execution behind an import-safe guard.
- Preserve the npm script name as `npmScript` on npm-backed verify steps so ledger resolution can compare a census `ownerScript` without parsing a command string.
- `discoverFilesystemCandidates(repoRoot, runGit)` uses `git ls-files --cached --others --exclude-standard -z`, keeps existing regular files, normalizes `/`, and accepts `*.test.*`/`*.spec.*` extensions matching `\.(test|spec)\.[cm]?[jt]sx?$` case-insensitively.
- `collectVitestFiles(owner, ...)` runs `node scripts/run-vitest.mjs list --filesOnly --no-color` plus the owner args.
- `collectPlaywrightFiles(owner, ...)` resolves `@playwright/test/cli`, then runs `test -c <config> --list --reporter=json` without npm.
- `validateRunnerCensus(...)` returns sorted actionable issues and no timing information.
- The first two final verify steps are `check:gemini-browser-sidecar-binary` and direct `node scripts/validate-testing-transition.mjs`.

- [ ] **Step 1: Write carrier, verify-order, and census contract tests**

Prove that the carrier runs injected checks in order, prints issues, and exits non-zero on any issue. Prove that `verify` runs the sidecar checker first, the direct transition validator second, preserves the old gate order, remains fail-fast, never invokes a build/bootstrap command, and has no `preverify` hook.

Use injected Git and runner outputs to cover:

- exact happy-path ownership by one Vitest and one Playwright owner;
- missing filesystem candidate;
- collected non-candidate;
- duplicate ownership/intersection;
- empty owner even when Vitest exits 0;
- runner non-zero/spawn error;
- malformed Playwright JSON or non-empty Playwright `errors`;
- Windows/absolute path normalization and repository escape rejection;
- a supported symlink candidate is rejected with an explicit unsupported-candidate error rather than silently omitted;
- exact `nonstandardTests` and `fixtureExceptions` with path, reason, owner;
- rejection of glob characters, broad paths, duplicates, unused/stale exceptions, and exceptions that hide an ownership intersection.

- [ ] **Step 2: Run RED**

```powershell
node scripts/run-vitest.mjs run scripts/testing/validate-testing-transition.test.ts scripts/verify.test.ts
```

Expected: FAIL because the carrier, census discovery/parsers, import-safe verify API, and early gates are missing.

- [ ] **Step 3: Implement one real carrier check without a snapshot**

Capture subprocess output with `shell: false`. Vitest stdout is one path per line. Parse Playwright's recursive `suites`/`specs` file fields relative to `config.rootDir`; list-mode `skipped` results are not failures. Require every owner to collect at least one file and Playwright `errors` to be empty.

Git index entries that no longer exist are ignored as deleted candidates. An existing supported symlink is a loud unsupported-candidate error in Slice 2A; do not silently follow it or classify it as an exception.

Enforce both directions:

1. every collected file is a filesystem candidate or exact `nonstandardTest`;
2. every filesystem candidate is owned once or exact `fixtureException`;
3. no collected file has two owners;
4. every exception is exact, necessary, and owned.

- [ ] **Step 4: Wire the carrier into current verify**

Make the direct CLI load `testing/runner-census.json` and run the census as its required check. It must fail if the JSON is missing, malformed, or has unknown fields. Refactor `verify` only enough to export/test its step list and runner; preserve sequential spawning, inherited output, fail-fast propagation, and the existing success message. There is no commit in which an always-green empty carrier is presented as protection.

- [ ] **Step 5: Run focused and live collection checks**

```powershell
node scripts/run-vitest.mjs run scripts/testing/validate-testing-transition.test.ts scripts/verify.test.ts
node scripts/validate-testing-transition.mjs
npm.cmd run verify
```

Expected live summary: 190 filesystem candidates, 184 Vitest files, 6 Playwright files, zero intersections, zero unowned files, zero collected non-candidates, and zero exceptions. Full verify starts with the sidecar checker and real census before the old gates. Output contains counts only, never durations.

- [ ] **Step 6: Commit**

```powershell
git add testing/runner-census.json scripts/testing/testing-transition.mjs scripts/testing/validate-testing-transition.test.ts scripts/validate-testing-transition.mjs scripts/verify.test.ts scripts/verify.mjs
git commit -m "test: enforce bidirectional runner census"
```

---

### Task 4: Build the Bounded Source-Contract Extractor and Ledger Validator

**Files:**
- Create: `scripts/testing/extract-source-contract-ledger.mjs`
- Modify: `scripts/testing/testing-transition.mjs`
- Modify: `scripts/testing/validate-testing-transition.test.ts`
- Modify: `docs/value-registry.md`

**Interfaces:**
- `discoverTestDeclarations(sourceFiles, typescript)` returns declaration path, full static nested title, normalized source slice, source offset, assertion ordinals, `.each` authority text, and referenced symbols.
- `discoverSourceReaders(program, trackedPaths)` marks only readers whose statically resolved authority is a Git-tracked production/configuration/documentation path, plus the exact source-reading exports from `analysis-contract-paths.ts`, `telegram-contract-paths.ts`, and `prompt-pack-contract-paths.ts`. A `node:fs` call alone is not a source-contract seed.
- The bounded dependency pass follows direct identifier references through module-local variable initializers and named helper bodies. It performs no control-flow, alias, callback, or general cross-module analysis; only the three named helper modules cross a module boundary. Anything else becomes an exact manual requirement.
- `buildLedgerDraft(...)` assigns `SC-000001` IDs once in `(path, sourceOffset)` order and writes only to an explicitly supplied ignored artifact path.
- `validateSourceContractLedger(...)` proves bidirectional cohort coverage, content hashes, resolution fields, and derived open/closed state.

**Ledger envelope:**

```json
{
  "schemaVersion": 1,
  "frozenAtCommit": "<full parent commit>",
  "sourceReaderExceptions": [],
  "rows": []
}
```

`sourceReaderExceptions` contains only exact `{ path, sourceRange, reason, owner }` call sites that read tracked test fixtures or diagnostic artifacts rather than production/configuration/documentation source. Temp-directory reads proven from `tmpdir`/`mkdtemp`, ignored `artifacts/`, runner result files, and generated outputs are non-production provenance and do not create obligations. An unresolved reader is never silently excluded: it requires an exact exception or manual row.

For this scanner, a production/configuration/documentation authority is a Git-tracked repository path that is not a supported test/spec file, an exact test-fixture path, an ignored/generated artifact, or dependency/vendor output. Directory readers are classified from the tracked files they actually enumerate, not merely from use of `readdir`.

**Common row identity fields:**

- `id`: stable `SC-000001`-style identifier, never recalculated after freeze;
- `path`: current path;
- `sourceHash`: SHA-256 of the LF-normalized exact declaration source slice;
- `assertionCount`: statically counted Vitest/Node assertion calls;
- optional `authorityHash`: SHA-256 of canonical `.each` parameters and/or source-authority initializer text;
- `lineage`: prior paths only, empty at freeze;
- either static `title`, or `manual: { sourceRange, reason, runnerTitles }` for a declaration the bounded extractor cannot name structurally. `runnerTitles` comes only from the freeze-time Vitest list. The current baseline is expected to need zero manual rows.

**Resolution union:**

- A simple row has top-level `invariant`, exactly one `disposition` (`behavior`, `architecture`, `tool_owned`, or `delete`), and either `replacementIds` or a `deletionReason` for `delete`. It has no `subgroups`.
- A mixed row has one umbrella `invariant` plus `subgroups`, but no top-level disposition or resolution. Subgroups cover every assertion ordinal exactly once without overlap; each subgroup owns its invariant, disposition, and resolution.
- Accepted replacement namespaces are `test:vitest:`, `test:playwright:`, `test:cargo:`, `rule:`, and `tool:`. Slice 2A resolves only `test:vitest:` against static current declarations plus live census ownership. The other namespaces are syntactically valid but remain unresolved until their inventory adapters arrive, so they cannot close a row in Slice 2A.

Do not store `status`, `state`, `pending`, `dualRun`, `retired`, timestamps, durations, or counters.

- [ ] **Step 1: Write extractor tests**

Cover static nested titles; raw imports whose targets are tracked production files; `node:fs` reads whose path resolves to tracked production/configuration/documentation files; the three existing contract-path helpers; the bounded module-local identifier graph; raw `import.meta.glob`; inline and named `.each` tables; ordinary behavior in a mixed file; assertion counting; LF-normalized hashes; and deterministic initial IDs.

Prove that temp files, ignored/generated artifacts, runner output, and tracked fixture reads do not become obligations. Tracked fixture/artifact readers require exact non-stale `sourceReaderExceptions`; broad paths and globs are rejected.

Cover bounded failure behavior: computed titles, dynamic/factory-created tests, unresolved dynamic `.each` tables, unknown source-reader wrappers, or ambiguous declaration ownership must produce an exact manual-row requirement with path and source range. They must never be silently skipped and must not expand the extractor into a general interpreter.

- [ ] **Step 2: Write ledger-validation tests**

Prove rejection of a current declaration with no row, duplicate IDs, duplicate current identities, hash/count drift, missing resolution, invalid disposition, overlapping/incomplete subgroup ordinals, unnecessary subgroups, invalid lineage, unknown replacement namespaces, stored lifecycle state, and manual rows without exact reason/range/runner titles.

Prove derived closure:

- a present legacy declaration is open even if replacement IDs are written;
- an absent declaration with unresolved resolution is an error, not an "extra row";
- an absent declaration with every replacement resolved is a valid closed historical row retained in the ledger;
- an absent `delete` row closes only with a specific deletion reason;
- a mixed row closes only when all subgroups close.

- [ ] **Step 3: Run RED**

```powershell
node scripts/run-vitest.mjs run scripts/testing/validate-testing-transition.test.ts
```

Expected: FAIL because source-reader discovery, draft generation, and ledger validation do not exist.

- [ ] **Step 4: Implement with the existing TypeScript dependency**

Keep analysis lexical and module-local except for the three named helper modules. Do not add control-flow or general inter-module data-flow to reproduce an audit count. Support the current static `.each` and raw `import.meta.glob` forms; unknown wrappers, dynamic paths, titles, tables, and factories become exact manual rows. The default validator never runs full Vitest declaration listing.

The extractor plus validator has a hard two-implementation-day limit. When that boundary is reached, stop extending syntax support and represent every remaining exact declaration with the manual-row form; do not defer the whole slice and do not add a general parser framework.

Resolve `test:vitest:<path>#<full-title>` from the static AST declaration inventory only when the live census assigns that path to a Vitest owner and that owner's `ownerScript` equals `npmScript` on a step from `createVerifySteps()`. Do not resolve Playwright or Cargo IDs from a file-only census. Rule/tool/Playwright/Cargo IDs remain open until later slices add their exact inventory adapters.

- [ ] **Step 5: Register the four ledger dispositions**

Add a tooling-only section to `docs/value-registry.md` for `behavior`, `architecture`, `tool_owned`, and `delete`. Record that they are persisted only in the checked-in migration ledger, have no product/database/API/UI impact, and are removed from active workflow when the completed ledger becomes historical evidence.

- [ ] **Step 6: Run GREEN**

```powershell
node scripts/run-vitest.mjs run scripts/testing/validate-testing-transition.test.ts
```

Expected: PASS with all extraction and validation fixtures; no timing artifact is written.

- [ ] **Step 7: Commit the extractor before freezing data**

```powershell
git add scripts/testing/extract-source-contract-ledger.mjs scripts/testing/testing-transition.mjs scripts/testing/validate-testing-transition.test.ts docs/value-registry.md
git commit -m "test: add bounded source contract ledger checks"
```

---

### Task 5: Freeze and Review the Source-Contract Obligations in Bounded Cohorts

**Files:**
- Create: `testing/source-contract-ledger.json`
- Modify: `scripts/validate-testing-transition.mjs`
- Modify only if a real unsupported current construct requires a bounded fix: `scripts/testing/extract-source-contract-ledger.mjs`
- Modify only if the same bounded fix needs coverage: `scripts/testing/validate-testing-transition.test.ts`

- [ ] **Step 1: Require a clean pre-freeze checkpoint**

```powershell
git status --short
git rev-parse HEAD
```

Expected: clean tree. Record this full HEAD as `frozenAtCommit`; it must be the parent of the ledger-data commit and must already contain all three new Slice 2A test files.

- [ ] **Step 2: Generate one ignored draft**

```powershell
node scripts/testing/extract-source-contract-ledger.mjs --output artifacts/testing/slice-2a/source-contract-ledger.draft.json
```

The extractor may invoke `node scripts/run-vitest.mjs list --json --no-color` once to reconcile declarations with runner full titles. It records no duration and is never called by `verify`.

The prior read-only audit expects 84 reader-candidate files, 778 declarations examined, 652 source-dependent rows, 19 mixed files, 10 `.each` declarations, and 3 raw `import.meta.glob` sites. The draft must print an exact reconciliation report rather than embedding these counts as validator policy.

- [ ] **Step 3: Reconcile provenance before reviewing dispositions**

Inspect every added/removed declaration versus the prior audit. Confirm that temp/generated/runtime reads are absent, tracked fixture/artifact reads have exact call-site exceptions, and every tracked production/configuration/documentation reader is represented. The current supported forms should yield zero manual rows; any manual requirement must name the exact unsupported syntax and remain inside the approved two-day extractor timebox.

If the reconciled row count is not 652, update this plan's frozen expectation with the reviewed declaration diff before promotion. Never change the extractor or add a broad exception solely to force the old count.

- [ ] **Step 4: Review the draft in deterministic ID cohorts**

Use a fresh review subagent for each non-overlapping range: `SC-000001..SC-000100`, `SC-000101..SC-000200`, `SC-000201..SC-000300`, `SC-000301..SC-000400`, `SC-000401..SC-000500`, `SC-000501..SC-000600`, and the remaining IDs. Each subagent edits only its assigned rows in the ignored draft; the coordinator reviews the range diff before assigning the next range. Do not commit an incomplete ledger or add persisted review-state fields.

For each row, replace generated draft prompts with a plain invariant and one approved disposition. Assign planned stable replacement IDs for `behavior`, `architecture`, and `tool_owned`, or a specific reason for `delete`. Split assertion subgroups only where one declaration truly mixes dispositions, owners, or partial retirement. Preserve the generated stable ID, content hashes, current path, title, and empty lineage.

No final row may contain null values, unresolved review prompts, placeholder text, or an unprefixed replacement ID.

- [ ] **Step 5: Promote and activate only a complete reviewed ledger**

Create `testing/source-contract-ledger.json` from the reviewed draft. In the same change, make the direct transition CLI load it unconditionally and run `validateSourceContractLedger` after the census. Before this step the carrier has a real census gate but no ledger gate; after this step a missing ledger is always an error.

Run:

```powershell
node scripts/validate-testing-transition.mjs
```

Expected: census still reports 184 Vitest plus 6 Playwright files. The ledger row count equals the reconciled freeze report (expected 652), every current source-dependent declaration has exactly one open row, no historical row is yet absent, and there are zero missing, duplicate, drifting, or schema-invalid rows. Planned replacement IDs may be unresolved and therefore open; that is expected before migration.

- [ ] **Step 6: Run the source-reader audit cross-check**

```powershell
rg -l --glob "*.test.ts" "(node:fs|\\?raw|import\\.meta\\.glob|analysis-contract-paths|telegram-contract-paths|prompt-pack-contract-paths)" src scripts sidecars research
node scripts/validate-testing-transition.mjs
```

Inspect the exact file diff against the extractor's candidate files. Every difference must be explained by resolved production provenance, proven temp/generated provenance, an exact source-reader exception, or an exact manual row; no broad file-level exception is allowed.

- [ ] **Step 7: Commit the frozen ledger**

```powershell
git add testing/source-contract-ledger.json scripts/validate-testing-transition.mjs
git commit -m "test: freeze source contract migration ledger"
```

---

### Task 6: Record Slice 2A Evidence and Run the Authoritative Gate

**Files:**
- Create: `docs/superpowers/verification/2026-08-02-testing-redesign-slice-2a-migration-preflight.md`
- Modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`

- [ ] **Step 1: Run focused transition verification**

```powershell
node scripts/run-vitest.mjs run scripts/check-gemini-browser-sidecar-binary.test.ts scripts/verify.test.ts scripts/testing/validate-testing-transition.test.ts scripts/testing/slice-1-rust-feasibility.test.ts
npm.cmd run check
node scripts/validate-testing-transition.mjs
git diff HEAD --check
```

Expected: all focused tests and static checks pass; transition output contains only inventory/ledger counts and issues, never durations.

- [ ] **Step 2: Run the full gate once**

```powershell
npm.cmd run verify
```

Expected: sidecar prerequisite passes first, transition validation passes second, then all pre-existing frontend/Svelte/Rust/git gates pass in their original order. Do not run a performance benchmark or add a timing report for this slice.

- [ ] **Step 3: Confirm no timing or scope drift**

```powershell
git diff --name-only 72a00b38..HEAD
git status --short --ignored
git diff --exit-code 72a00b38 -- scripts/testing/timing-log.mjs
rg -n "timing-log|recordTiming|appendTiming" scripts/validate-testing-transition.mjs scripts/testing/testing-transition.mjs scripts/testing/extract-source-contract-ledger.mjs testing
```

Expected: ignored generated drafts may exist only under `artifacts/testing/slice-2a/`; the timing writer diff is empty; the final `rg` returns no matches (exit 1); no tracked timing data or new timing call exists. `sourceHash`/`authorityHash` are ledger identity fields only and must be described that way in evidence.

- [ ] **Step 4: Write the verification record**

Record:

- base and final commit identities;
- 190 filesystem candidates = 184 Vitest + 6 Playwright;
- zero runner intersections/exceptions/unowned/extra files;
- the reconciled ledger row count (expected 652), exact source-reader exception count, and counts by disposition/manual/subgroup;
- exact fresh-checkout bootstrap contract;
- focused and full correctness outcomes;
- explicit statement that Slice 2A added no timing mechanism and captured no duration.

Do not include machine timing claims.

- [ ] **Step 5: Advance the program index**

Mark Slice 2A as the completed current detailed plan, link its verification record, and state that Slice 2B may be authored only from this committed census/ledger checkpoint.

- [ ] **Step 6: Commit documentation and verify cleanliness**

```powershell
git add docs/superpowers/verification/2026-08-02-testing-redesign-slice-2a-migration-preflight.md docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md
git commit -m "docs: record testing redesign slice 2a"
git status --short
```

Expected: clean worktree. Do not push.

## Completion Criteria

- `npm.cmd run verify` passes with the sidecar prerequisite first and transition validation second.
- The live census owns exactly every Git-visible test candidate once, every owner is non-empty, and exact exception arrays remain empty unless a separately reviewed current fact requires one.
- The committed ledger accounts for the reconciled baseline source-dependent declarations (expected 652) with stable IDs, reviewed invariants/dispositions, hashes, assertion counts, planned resolution, and no stored lifecycle state.
- No existing source-contract test has moved or been removed.
- The only daily runner collection added is list-only file census; full declaration listing is freeze-only.
- No timing field, benchmark, threshold, performance state, or timing-derived selection behavior was added.
