# Testing Redesign Slice 3A: Telegram and Analysis Source Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the first three largest legacy source-contract files while preserving their 187 reviewed behavior and architecture obligations through executable behavior tests and structured RepositoryIndex rules.

**Architecture:** The ledger remains the migration authority. Behavior rows become direct Vitest tests of public TypeScript/Tauri seams; architecture rows become named RepositoryIndex evaluators that inspect parsed TypeScript, Svelte, Cargo metadata, and declared repository fixtures instead of raw production-source regular expressions. Delete rows remove only the explicitly documented implementation-detail assertions. Each task removes one legacy file and leaves the transition validator as the proof that its ledger rows closed.

**Tech Stack:** Node.js ESM, TypeScript, Vitest 4.1.5, TypeScript compiler API, `svelte/compiler`, Cargo metadata and focused Cargo tests, existing transition validator.

## Global Constraints

- Nx remains unselected; do not add Nx packages, configuration, cache, or commands.
- Do not add timing eligibility, fingerprints, rolling counters, benchmark states, or timing-driven selection.
- No test may read arbitrary production source solely for `includes`, regular-expression, or text-match assertions.
- Fixture readers and RepositoryIndex tests stay exact, declared exceptions; parse failures are `INFRA_ERROR`, never zero violations.
- Preserve stable ledger IDs and reviewed invariants; close a row only through a present replacement or documented deletion. The user-approved SC-000648 correction may replace its inaccurate single Vitest resolution with exhaustive frontend, Cargo, and architecture subgroups while retaining the stable row ID and all 17 assertion ordinals.
- Preserve direct Cargo ownership for Rust behavior. Node tests must not parse `.rs` production text to prove Rust behavior.
- On Windows use `npm.cmd`; use the package-specific Rust loops in `AGENTS.md`; every task ends with `npm.cmd run verify` outside the sandbox when process-tree proof is needed.
- Do not change `TestingManifest`, selector behavior, scheduler behavior, public test commands, GitHub settings, or hosted CI.

---

## File Map

| Path | Responsibility |
| --- | --- |
| `scripts/testing/repository-index.mjs` | Parse each declared TypeScript/Svelte/Cargo input at most once per index snapshot; return typed facts or an explicit parse failure. |
| `scripts/testing/repository-index.test.ts` | Unit tests for parsed facts, snapshot reuse, and parse-failure behavior. |
| `scripts/testing/repository-rules.mjs` | Stable rule IDs and evaluators for the migrated Telegram and analysis architecture obligations. |
| `scripts/testing/repository-rules.test.ts` | Positive and mutation tests for each implemented rule ID. |
| `src/lib/telegram-checkpoint-2.behavior.test.ts` | Direct tests for SC-000648 through SC-000650. |
| `src/lib/telegram-takeout.behavior.test.ts` | Direct test for SC-000651 and focused Telegram behavior handoff. |
| `src/lib/telegram-session-persistence.behavior.test.ts` | Direct test for SC-000652. |
| `src-tauri/src/lib.rs` | Single-source Telegram Tauri command inventory, the real invoke handler expansion, and its Cargo-owned registration test. |
| `src/lib/analysis-source-readers.behavior.test.ts` | Direct component/route behavior tests for SC-000222 and SC-000224 through SC-000278. |
| `testing/source-contract-ledger.json` | Existing stable rows; no newly invented obligations. |
| `vitest.config.ts`, `testing/runner-census.json`, `package.json`, `scripts/verify.mjs` | Remove legacy ownership only when the last file in that owner has migrated; Slice 3A must retain the project because later rows remain. |

## Rust Verification Loops

- Affected package: `extractum` application crate; Task 2A changes only its private Tauri handler inventory and test.
- Exact RED/GREEN test: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib tests::telegram_command_registration_inventory_is_exact -- --exact`; list tests first if the module-qualified name differs, and never accept a zero-test run.
- Focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
- Task checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.
- End-of-slice workspace gates remain `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`, `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`, and the authoritative `npm.cmd run verify`.
- Use only canonical `src-tauri/target`; do not introduce a slice-specific target directory or test-support feature.

## Task 1: RepositoryIndex foundation and rule runner

**Files:**

- Create: `scripts/testing/repository-index.mjs`
- Create: `scripts/testing/repository-index.test.ts`
- Create: `scripts/testing/repository-rules.mjs`
- Create: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/test-conventions.test.ts`

**Interfaces:**

- `createRepositoryIndex({ root, readFile, ts, svelte, loadCargoMetadata })` returns `getTypeScript(path)`, `getSvelte(path)`, and `getCargoMetadata()` for one immutable snapshot.
- Each getter returns parsed data or throws an error with the repository-relative input path; callers convert it to `INFRA_ERROR`.
- `evaluateRule({ id, index })` returns `{ id, violations: string[] }`; unknown IDs throw.

- [ ] **Step 1: Write failing RepositoryIndex tests**

Cover a parsed TypeScript import fact, a parsed Svelte component fact, repeated access that parses each declared input once per index instance, and a malformed input that throws instead of returning an empty fact set. Do not add invalidation or a disk cache in Slice 3A; Slice 4 owns the warm watcher and its invalidation contract.

- [ ] **Step 2: Run the focused RED command**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-index.test.ts`

Expected: FAIL because the index module is absent.

- [ ] **Step 3: Implement the minimum parsed index**

Use the installed TypeScript compiler API and `svelte/compiler`; use `cargo metadata --format-version 1 --no-deps` only through one index method. Do not expose raw text search as an evaluator API.

- [ ] **Step 4: Write failing rule-runner tests**

Derive the allowed 22-ID set from the existing replacement IDs in SC-000029–SC-000059, SC-000221–SC-000278, and SC-000561–SC-000658. Assert that every currently registered evaluator belongs to that set and has its own positive fixture plus a mutation with a non-empty violation. This task does not assert that all 22 IDs are registered yet.

- [ ] **Step 5: Implement named evaluators and convention guard**

Implement two end-to-end examples: `telegram-repository-path-safety` and `analysis-source-reader-surface-composition`. Extend `test-conventions.test.ts` so the rule runner is an approved structured source authority and direct production-root text readers outside approved index/fixture owners fail. Unknown rule IDs still throw; later tasks add the remaining registered IDs under the same per-ID test contract.

- [ ] **Step 6: Verify Task 1**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-index.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts`

Expected: PASS with exactly the two initial evaluators registered, including their independent mutations and explicit parse-failure coverage.

## Task 2A: Add direct Telegram behavior replacements

**Files:**

- Create: `src/lib/telegram-checkpoint-2.behavior.test.ts`
- Create: `src/lib/telegram-takeout.behavior.test.ts`
- Create: `src/lib/telegram-session-persistence.behavior.test.ts`
- Modify: `src-tauri/src/lib.rs`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- SC-000649–SC-000652 resolve to the Vitest behavior identities already recorded in the ledger.
- SC-000648 retains its stable ID but becomes exhaustive subgroups: assertions 2–3 resolve to `test:vitest:src/lib/telegram-checkpoint-2.behavior.test.ts#pins frontend Telegram command IPC names and default camelCase keys`; assertions 1 and 4–14 resolve to `test:cargo:extractum::tests::telegram_command_registration_inventory_is_exact`; assertions 15–17 resolve to the already approved `rule:telegram-app-facade-import-boundary` and `rule:telegram-crate-extraction-boundary` replacements. No assertion ordinal may be omitted or overlap.

- [ ] **Step 1: Write failing direct Telegram behavior tests**

Use public frontend command wrappers, emitted events, session, and Takeout seams. Assert the observable TypeScript contracts named by SC-000648 through SC-000652 without importing or reading Rust source text. The SC-000648 Vitest title is the exact frontend subgroup identity above; it must not claim backend registration.

- [ ] **Step 2: Run focused RED tests**

Run: `node scripts/run-vitest.mjs run src/lib/telegram-checkpoint-2.behavior.test.ts src/lib/telegram-takeout.behavior.test.ts src/lib/telegram-session-persistence.behavior.test.ts`

Expected: FAIL because the replacement files are absent.

- [ ] **Step 3: Write the failing Cargo registration test**

In `src-tauri/src/lib.rs`, add `tests::telegram_command_registration_inventory_is_exact` before introducing the inventory seam. The test names exactly the twelve frozen account/Telegram commands and must fail because no single-source inventory exists yet.

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib tests::telegram_command_registration_inventory_is_exact -- --exact`

Expected: FAIL because the command inventory used by both the handler and test is absent.

- [ ] **Step 4: Implement the smallest frontend fixtures and single-source Rust inventory**

Do not duplicate Rust production logic in TypeScript. In Rust, define the twelve-command inventory once through a local macro or equivalent compile-time construct; expand that same inventory into the real `tauri::generate_handler!` expression and into the Cargo assertion. Do not maintain a second string list disconnected from the handler. Keep the inventory private to the application crate. Update SC-000648 to the approved exhaustive subgroups without changing its stable ID, source hashes, lineage, or assertion count. Do not delete the legacy contract in this task.

- [ ] **Step 5: Verify Task 2A**

Run the focused Vitest command, the exact Cargo test, `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`, `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`, and `node scripts/validate-testing-transition.mjs`.

Expected: PASS. The transition validator remains open for SC-000561–SC-000647 and SC-000653–SC-000658 until Task 2B adds their structured rules.

## Task 2B: Migrate Telegram architecture rules and cut over the legacy file

**Files:**

- Delete: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/extract-source-contract-ledger.mjs`
- Modify: `scripts/testing/validate-testing-transition.test.ts`
- Modify: `scripts/validate-testing-transition.mjs`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- SC-000561–SC-000647 and SC-000653–SC-000658 resolve only to their existing 15 `rule:telegram-*` IDs.
- Every rule ID has one positive fixture and at least one mutation that produces a non-empty violation.
- A `rule:` replacement resolves only when the named evaluator exists and returns zero violations on the live RepositoryIndex. A `test:cargo:` replacement resolves only when the exact package/test identity appears in a bounded `cargo test -- --list` result and the owning Cargo workspace gate remains in `verify`.

- [ ] **Step 1: Write failing evaluator cases for every Telegram rule ID**

Extend the registry table to all 15 Telegram rule IDs. For each newly added ID, add the smallest mutation fixture that violates that exact ID; retain the independent Task 1 coverage for `telegram-repository-path-safety`. Do not use a family-level mutation as evidence for another evaluator.

- [ ] **Step 2: Run the focused RED command**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts -t "telegram"`

Expected: FAIL because 14 of the 15 Telegram evaluators and their mutation coverage are incomplete; `telegram-repository-path-safety` was established as the Task 1 example.

- [ ] **Step 3: Implement all Telegram architecture evaluators**

Make the existing `rule:telegram-*` IDs evaluate parsed structure and Cargo metadata. Preserve each positive fact and return deterministic non-empty violations for its paired mutation fixture.

- [ ] **Step 4: Remove the legacy file and prove full Telegram closure**

Before deletion, add transition-validator RED/GREEN cases for a missing rule ID, a violating live rule, a missing Cargo test identity, and the exact passing SC-000648 Cargo identity. Obtain the Cargo list once per transition validation only when the ledger contains a historical `test:cargo:` replacement; do not persist a Cargo census or timing data.

Delete `src/lib/telegram-crate-boundary-contract.test.ts`; do not alter its row IDs, invariants, dispositions, or replacement IDs. Run `node scripts/validate-testing-transition.mjs` and require SC-000561–SC-000658 to report closed.

- [ ] **Step 5: Verify Task 2B**

Run: `node scripts/run-vitest.mjs run src/lib/telegram-checkpoint-2.behavior.test.ts src/lib/telegram-takeout.behavior.test.ts src/lib/telegram-session-persistence.behavior.test.ts scripts/testing/repository-rules.test.ts`; then run the owning Telegram package check/test commands from `AGENTS.md` and `npm.cmd run verify`.

## Task 3: Migrate the analysis source-reader contract

**Files:**

- Delete: `src/lib/analysis-source-readers.test.ts`
- Create: `src/lib/analysis-source-readers.behavior.test.ts`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- SC-000222 and SC-000224 through SC-000278 resolve to `analysis-source-readers.behavior.test.ts` as recorded in the ledger.
- SC-000221, SC-000223, SC-000265, SC-000267, SC-000268, and SC-000271 resolve to their existing `rule:analysis-*` IDs.
- SC-000239 through SC-000241 close only by their existing documented deletion reasons.

- [ ] **Step 1: Write failing direct source-reader behavior tests**

Use rendered components and route-owned fixtures to cover live sources, snapshots, source groups, focus controls, items, metadata, activity, and evidence highlights. Keep assertions to user-visible state, props, emitted actions, and accessible DOM rather than component implementation text.

- [ ] **Step 2: Run focused RED tests**

Run: `node scripts/run-vitest.mjs run src/lib/analysis-source-readers.behavior.test.ts`

Expected: FAIL because the replacement file is absent.

- [ ] **Step 3: Implement the minimum behavior fixtures and structured rules**

Use existing component fixtures and Testing Library; extend the index rules to all six declared `rule:analysis-*` IDs, retaining the Task 1 coverage for `analysis-source-reader-surface-composition`. Every rule ID has a mutation with a deterministic non-empty violation.

- [ ] **Step 4: Remove the legacy file and close its rows**

Delete `src/lib/analysis-source-readers.test.ts`; preserve its ledger IDs and deletion reasons. Run the transition validator and require all rows for that path to report closed.

- [ ] **Step 5: Verify Task 3**

Run: `npm.cmd run test:component`; `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts`; `npm.cmd run check`; `npm.cmd run verify`.

## Task 4: Migrate the analysis application boundary contract

**Files:**

- Delete: `src/lib/analysis-application-contract.test.ts`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- SC-000029 through SC-000059 resolve to `rule:repository-index:analysis-application-boundary`.

- [ ] **Step 1: Write failing boundary rule tests**

Add positive facts for the current analysis application boundary and mutations covering selector ownership, portable engine placement, event-sink shape, opaque capability visibility, Cargo ownership, and command inventory drift.

- [ ] **Step 2: Run the focused RED command**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts -t "analysis application boundary"`

Expected: FAIL because the evaluator does not yet implement the boundary facts.

- [ ] **Step 3: Implement the one named evaluator**

Implement `repository-index:analysis-application-boundary` from parsed facts and Cargo metadata. It must report every violated invariant deterministically and throw on an unavailable or malformed declared input.

- [ ] **Step 4: Remove the legacy contract and validate closure**

Delete `src/lib/analysis-application-contract.test.ts`. Run `node scripts/validate-testing-transition.mjs` and require SC-000029 through SC-000059 to report closed.

- [ ] **Step 5: Verify Task 4**

Run the focused rule suite, `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`, `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`, and `npm.cmd run verify`.

## Task 5: Slice 3A completeness, evidence, and handoff

**Files:**

- Create: `docs/superpowers/verification/2026-08-03-testing-redesign-slice-3a-source-contracts.md`
- Modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`

- [ ] **Step 1: Add the final 22-ID completeness assertion**

In `scripts/testing/repository-rules.test.ts`, derive the exact rule-ID set from SC-000029–SC-000059, SC-000221–SC-000278, and SC-000561–SC-000658. Assert exact set equality with the registered evaluator IDs and retain the per-ID positive/mutation requirement.

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts`

Expected: PASS with exactly 22 implemented rule IDs; an absent, extra, or mutation-free rule fails.

- [ ] **Step 2: Record the migration evidence**

Record the exact closed row ranges, removed paths, replacement tests/rules, focused command results, final authoritative `npm.cmd run verify` result, and any environmental distinction between sandboxed and unsandboxed Windows process-tree proof.

- [ ] **Step 3: Re-enumerate remaining ledger work**

Record the live remaining open-row count and the next largest path cohort. Do not plan or modify the next cohort in this task.

- [ ] **Step 4: Validate documentation and repository state**

Run: `git diff --check` and `git status --short`.

Expected: only the Slice 3A implementation and evidence files are changed before their commit.

## Verification Matrix

| Checkpoint | Evidence |
| --- | --- |
| Structured authority | RepositoryIndex and rule mutation tests pass; malformed inputs surface as errors. |
| Telegram migration | SC-000561–SC-000658 close without a Node source scan of Rust production files; SC-000648 resolves through exhaustive frontend, Cargo, and architecture subgroups under its stable ID. |
| Source-reader migration | SC-000221–SC-000278 close, including three explicit deletion rows. |
| Analysis application migration | SC-000029–SC-000059 close through one deterministic structured rule. |
| Transition integrity | Census remains bidirectional and ledger closes the removed paths. |
| Completion gate | Unsandboxed `npm.cmd run verify` passes; sandbox-only taskkill denial is recorded separately rather than misclassified as a product failure. |
