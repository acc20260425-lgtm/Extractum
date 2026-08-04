# Testing Redesign Slice 3A: Telegram and Analysis Source Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the first three largest legacy source-contract files while preserving durable behavior and architecture obligations through executable tests, four narrow Telegram RepositoryIndex rules, one narrow analysis Cargo-manifest rule, Cargo metadata, and explicit retirement of historical source shape.

**Architecture:** The ledger remains the migration authority. Behavior rows become direct Vitest/Cargo tests; durable architecture rows use parsed TypeScript/Svelte facts, Cargo metadata, or generated repository authorities. Telegram keeps only four truthful structured rules and one transition-only bounded Cargo test-list comparison. The analysis application cutover keeps only a Cargo-metadata package/dependency boundary; it explicitly does not claim Rust visibility, call-flow, SQL, transaction, command-body, or module-reachability facts. Delete rows retire checkpoint simulations, exact source locations/counts, transaction topology, and scanner mutation cases instead of replacing either deleted scanner with another Rust parser. Each task removes one legacy file and leaves the transition validator as the proof that its ledger rows closed.

**Tech Stack:** Node.js ESM, TypeScript, Vitest 4.1.5, TypeScript compiler API, `svelte/compiler`, Cargo metadata and focused Cargo tests, existing transition validator.

## Global Constraints

- Nx remains unselected; do not add Nx packages, configuration, cache, or commands.
- Do not add timing eligibility, fingerprints, rolling counters, benchmark states, or timing-driven selection.
- No test may read arbitrary production source solely for `includes`, regular-expression, or text-match assertions.
- Fixture readers and RepositoryIndex tests stay exact, declared exceptions; parse failures are `INFRA_ERROR`, never zero violations.
- Preserve stable ledger IDs and reviewed invariants; close a row only through a present replacement or documented deletion. The user-approved Task 2A correction replaces nominal Vitest ownership with exhaustive Cargo, architecture, frontend, and deletion subgroups while retaining SC-000648 through SC-000652 and every assertion ordinal.
- Preserve direct Cargo ownership for Rust behavior. Node tests must not parse `.rs` production text to prove Rust behavior.
- Do not add a Rust lexer/parser, tree-sitter binding, `syn`, `ra_ap_syntax`, rustdoc/nightly inspector, or persistent Cargo test census for Slice 3A.
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
| `src/lib/telegram-checkpoint-2.behavior.test.ts` | Fast contract for the eleven public account/Telegram TypeScript wrappers; it does not claim backend state, event, Takeout, or session behavior. |
| `src/lib/telegram-contract-paths.behavior.test.ts` | Direct fast behavior for the retained Phase 8 lifecycle vocabulary and path resolution. |
| `src-tauri/src/lib.rs` | One complete application command inventory feeding the real invoke handler plus direct Cargo-owned exact registration checks for the account/Telegram and analysis/project/dev cohorts. |
| `src-tauri/src/telegram.rs`, `src-tauri/src/telegram_session_store.rs` | Direct application-crate tests for Telegram result/order/event and session-path/atomic-write behavior. |
| `src-tauri/src/takeout_import/mod.rs` | Direct application-crate cancellation mutation-before-event behavior. |
| `src-tauri/crates/extractum-telegram/src/session.rs` | Direct encrypted-session error behavior. |
| `src-tauri/crates/extractum-telegram/src/takeout/operations.rs`, `src-tauri/crates/extractum-telegram/src/takeout/raw_parse.rs` | Direct Checkpoint-6 lifecycle and raw peer-identity behavior. |
| `src-tauri/crates/extractum-analysis/Cargo.toml` | Live package/dependency input for `rule:analysis-crate-manifest-boundary`; the rule consumes Cargo metadata, not Rust source text. |
| `src/lib/analysis-source-readers.behavior.component.test.ts` | Direct jsdom component/route behavior tests for SC-000222 and SC-000224 through SC-000278; the suffix gives permanent component-project ownership without a Vitest special case. |
| `testing/source-contract-ledger.json` | Existing stable rows; no newly invented obligations. |
| `vitest.config.ts`, `testing/runner-census.json`, `package.json`, `scripts/verify.mjs` | Remove legacy ownership only when the last file in that owner has migrated; Slice 3A must retain the project because later rows remain. |

## Rust Verification Loops

- Affected packages: `extractum`, `extractum-telegram`, and `extractum-analysis`.
- Task 2A narrow RED/GREEN runs use each exact test identity listed in the Task 2A subgroup tables with `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <module>::tests::<name> -- --exact`; list tests first if an identity differs, and never accept a zero-test run.
- Focused checks: `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets` and `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Task checkpoints: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets` and `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
- Task 4 narrow RED/GREEN runs are `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib tests::analysis_command_registration_inventory_is_exact -- --exact` and the focused RepositoryIndex rule test. Its package checkpoint also includes `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets` and `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets`.
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

Derive the pre-cutover 22-ID candidate set from the existing replacement IDs in SC-000029–SC-000059, SC-000221–SC-000278, and SC-000561–SC-000658. Assert that every currently registered evaluator belongs to that set and has its own positive fixture plus a mutation with a non-empty violation. Task 2B later corrects Telegram ownership and intentionally reduces the final set to 11.

- [ ] **Step 5: Implement named evaluators and convention guard**

Implement two end-to-end examples: `telegram-repository-path-safety` and `analysis-source-reader-surface-composition`. Extend `test-conventions.test.ts` so the rule runner is an approved structured source authority and direct production-root text readers outside approved index/fixture owners fail. Unknown rule IDs still throw; later tasks add the remaining registered IDs under the same per-ID test contract.

- [ ] **Step 6: Verify Task 1**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-index.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts`

Expected: PASS with exactly the two initial evaluators registered, including their independent mutations and explicit parse-failure coverage.

## Task 2A: Replace nominal Telegram tests with direct Rust behavior

**Files:**

- Modify: `src/lib/telegram-checkpoint-2.behavior.test.ts`
- Delete: `src/lib/telegram-takeout.behavior.test.ts`
- Delete: `src/lib/telegram-session-persistence.behavior.test.ts`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/telegram.rs`
- Modify: `src-tauri/src/telegram_session_store.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/session.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/operations.rs`
- Modify: `src-tauri/crates/extractum-telegram/src/takeout/raw_parse.rs`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- SC-000648 retains all 17 ordinals: delete source-only ordinals `{1,4,6}` with specific reasons; `{2,5}` resolve to `test:cargo:extractum::tests::telegram_command_registration_inventory_is_exact`; `{3}` resolves to `test:vitest:src/lib/telegram-checkpoint-2.behavior.test.ts#pins the eleven public account and Telegram wrapper IPC names and default camelCase keys`; `{7-12}` resolve to the existing exact `extractum::account_deletion::tests::*` behavior cases named in `.superpowers/sdd/task-2a-cargo-ownership-map.md`; `{13,14}` resolve to the three existing exact credential behavior tests named there. Task 2A provisionally leaves `{15,16}` and `{17}` on the original architecture IDs; Task 2B replaces those unenforceable source-shape claims with the approved `{15,16,17}` deletion subgroup. The unused backend-only `tg_is_authenticated` TypeScript wrapper/key fragment is not preserved by inventing public API.
- SC-000649 retains all 20 ordinals: `{1}` restore-event test; `{2,3}` send-code test; `{4,5}` sign-in order test; `{6,7}` existing missing-hash test; `{8}` logout order test; `{9,10}` session-path test; delete `{11,12}` as private call-count shape; `{13}` atomic-write outcome/error test; `{14,15,17,18,20}` encrypted-session error test; `{16}` existing invalid-key-length test; `{19}` existing missing-key test.
- SC-000650 `{1,2,3}` resolves jointly to `test:cargo:extractum::takeout_import::tests::cancelled_job_emits_persisted_terminal_record` and existing `test:cargo:extractum::takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact`.
- SC-000651 retains all 11 ordinals: Task 2A provisionally leaves `{1,2,10,11}` on `rule:telegram-crate-extraction-boundary`, and Task 2B replaces that transitional path/layout claim with its documented deletion subgroup. `{3,4}` use the existing TDesktop pagination test plus `test:cargo:extractum-telegram::takeout::operations::tests::checkpoint_six_remote_lifecycle_preserves_fallback_and_provenance_order`; `{5}` uses the existing descending-fallback pagination test plus that same remote-lifecycle identity; `{6,7,8,9}` use the new raw peer-identity test. The extracted identity proves the reviewed unordinalized remote validate, migration/split, count, fallback/import, finish, and provenance behavior; `test:cargo:extractum::takeout_import::tests::cancelled_job_emits_persisted_terminal_record` proves application-owned cancellation, durable batch finalization, terminal selection, persistence, and emission ordering.
- SC-000652 `{1,2}` resolves jointly to existing `test:cargo:extractum::telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact` and the new atomic-write outcome test; delete `{3,4,5}` as helper-call-count/delegation/source-shape constraints. No assertion ordinal may be omitted or overlap.

- [ ] **Step 1: Repair the command-surface replacement under RED tests**

Trim `telegram-checkpoint-2.behavior.test.ts` to the eleven real public wrappers and the exact title above. Delete the two mock-only Takeout/session files: fabricated `invoke` results and manually injected events are not backend behavior evidence. Add a Cargo RED that fails while any account/Telegram command is duplicated in the complete application handler inventory. Refactor the entire invoke-handler command list into one private callback inventory expanded both into the real `tauri::generate_handler!` and the test; assert each of the twelve target names occurs exactly once. Remove the six manual duplicate `tg_*` entries. Do not add a `tg_is_authenticated` frontend wrapper.

- [ ] **Step 2: Add direct application Telegram/session Cargo behavior**

Add these exact tests under TDD, using narrow internal recorders/scripted adapters only where the production ordering cannot otherwise be observed:

- `extractum::telegram::tests::restore_emits_failure_event_for_each_failed_account`;
- `extractum::telegram::tests::send_code_success_is_not_auth_error`;
- `extractum::telegram::tests::sign_in_persists_session_before_ready_event`;
- `extractum::telegram::tests::logout_returns_true_after_runtime_and_session_cleanup`;
- `extractum::telegram_session_store::tests::session_path_uses_app_data_root_and_account_filename`;
- `extractum::telegram_session_store::tests::atomic_session_write_outcome_and_error_contract_is_exact`.

The session test must exercise success, write failure, and rename failure, assert final replacement/no residual temporary file, and assert the existing internal-error contract. Do not expose production capability through a test-support feature.

- [ ] **Step 3: Add extracted Telegram/session Cargo behavior**

Add exact test `extractum-telegram::session::tests::encrypted_session_error_contract_is_exact` under TDD. It covers SC-000649 `{14,15,17,18,20}` with deterministic invalid base64/key/nonce/format and encrypt/decrypt failure cases. Reuse the existing invalid-key-length, wrong-account, and missing-key tests only for the behavior they actually execute.

- [ ] **Step 4: Add direct Takeout Cargo behavior**

Add these exact tests under TDD:

- `extractum::takeout_import::tests::cancelled_job_emits_persisted_terminal_record`, with an internal event recorder proving state persistence before emission and cancellation-versus-failure terminal selection;
- `extractum-telegram::takeout::operations::tests::checkpoint_six_remote_lifecycle_preserves_fallback_and_provenance_order`, using the existing scripted transport/backend seam to record real validate, migration/split, count, import, fallback, provenance, and finish behavior. Application-owned cancellation, terminal selection, batch finalization, persistence, and emission ordering remain in `extractum::takeout_import::tests::cancelled_job_emits_persisted_terminal_record`;
- `extractum-telegram::takeout::raw_parse::tests::takeout_peer_identity_maps_user_chat_and_channel`, proving User/Chat/Channel identity and retained `i64` message IDs.

Reuse the two exact pagination tests and the existing application Takeout state contract named in the ownership map; do not duplicate their scenarios.

- [ ] **Step 5: Write the exhaustive ledger subgroups**

Apply the exact SC-000648–SC-000652 partitions above. Every delete subgroup has a specific `deletionReason`; every behavior replacement names a present exact Vitest/Cargo identity; rule-owned subgroups remain open until Task 2B. Preserve stable IDs, hashes, lineage, assertion counts, and the legacy source file.

- [ ] **Step 6: Verify Task 2A**

Run the one focused Vitest file, every new exact Cargo test at least once, both package-focused checks and package checkpoints from `## Rust Verification Loops`, and `node scripts/validate-testing-transition.mjs`.

Expected: all direct behavior tests pass. The transition validator itself passes while reporting rule-owned Telegram subgroups open until Task 2B; it must not report missing Cargo/Vitest identities or nominal mock-only replacements.

## Task 2B: Simplify Telegram architecture ownership and cut over the legacy file

**Files:**

- Create: `src/lib/telegram-contract-paths.behavior.test.ts`
- Delete: `src/lib/telegram-crate-boundary-contract.test.ts`
- Modify: `scripts/testing/repository-index.mjs`
- Modify: `scripts/testing/repository-index.test.ts`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `scripts/testing/extract-source-contract-ledger.mjs`
- Modify: `scripts/testing/validate-testing-transition.test.ts`
- Modify: `scripts/validate-testing-transition.mjs`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- Preserve every SC-000561–SC-000658 ID, path, source/authority hash, lineage entry, and assertion count. Resolution fields may change under the approved ownership correction; only SC-000648 and SC-000651 require ordinal subgroup edits.
- The complete Telegram structured-rule set is exactly `rule:telegram-repository-path-safety`, `rule:telegram-phase-8b-authority-integrity`, `rule:telegram-crate-manifest-boundary`, and `rule:telegram-crate-dependency-ownership`.
- `tool:telegram-cargo-test-identity-ownership` is transition-only. It runs `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list` and the equivalent `-p extractum-telegram` command once each, compares exact module-qualified identities declared by `src/lib/telegram-8b-test-identities.json`, requires each identity under its declared package exactly once, requires absence from the other package, and persists no census or timing data.
- A `rule:` replacement resolves only when its evaluator exists and returns zero live violations. A `test:cargo:` replacement resolves only when its exact identity is present in the bounded owner list and that package gate remains in `verify`. The transition-only tool resolves only when both package comparisons and verify-owner checks pass.

- [ ] **Step 1: Add RED tests for the four truthful rules and the bounded identity tool**

Retain the independent path-safety fixture. Add positive and non-empty mutation fixtures for authority integrity, semantic manifest ownership, and dependency ownership. Authority integrity regenerates the symbol map and test-identity map from their declared plan inputs. It validates the frozen staging SHA artifact declaratively (schema, exact frozen content address, and referenced Phase 8B authority) but must not execute `telegram-staging-sha256.mjs --check`: that pre-move checker intentionally became stale after Phase 8C extraction and would read the retired `.rs` staging tree. The rule must not claim live Rust definition conformance. Manifest/dependency rules use Cargo metadata and must detect a missing producer target, a production edge that enables `app-test-support`, a missing dev-only feature edge, an app-owned direct Grammers dependency, and Grammers feature/source drift.

Add transition-validator cases proving that the Cargo identity tool fails on a missing identity, a duplicate/wrong-package identity, a failed list command, or a missing verify owner, and passes representative exact identities from both packages. Inject list results in unit tests; do not spawn Cargo from the unit-test fixture.

- [ ] **Step 2: Run the focused RED commands**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-index.test.ts scripts/testing/repository-rules.test.ts scripts/testing/validate-testing-transition.test.ts -t "telegram|Cargo test identity"`

Expected: FAIL because three Telegram evaluators, declared authority inputs, and the transition-only Cargo-list resolver are absent.

- [ ] **Step 3: Implement the minimum structured authority and Cargo resolver**

Extend RepositoryIndex only with declared JSON/text artifact access needed by the pure Telegram generators; cache each declared input once and fail closed on read/parse/generator errors. Implement the three new evaluators and the bounded tool resolver. Invoke each real Cargo list at most once per package during a live transition validation and only when a historical ledger row references `test:cargo:` or `tool:telegram-cargo-test-identity-ownership`.

Do not read or tokenize live `.rs` text. Do not persist the list, timing, fingerprint, or eligibility state. Keep the four evaluator IDs exact and reject unknown `rule:` or `tool:` IDs.

- [ ] **Step 4: Add direct lifecycle behavior and verify all named Cargo identities**

Create `src/lib/telegram-contract-paths.behavior.test.ts` with the exact title `recognizes the retained Phase 8 lifecycle vocabulary and paths`. Directly cover `phase8BCheckpointNumber`, `telegramLifecycleFromStatus`, and `resolveTelegramLifecyclePath`, including unknown lifecycle/status values.

Before changing ledger ownership, run both bounded package lists and confirm every Cargo identity named in `.superpowers/sdd/task-2b-ownership-map.md` is non-empty and exact. In particular verify the three SC-000561 identities, the encrypted-session identities for SC-000627 and SC-000629–SC-000631, and `runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt`. A missing identity is a blocking mapping error.

- [ ] **Step 5: Apply the approved ledger ownership map**

Use `.superpowers/sdd/task-2b-ownership-map.md` as the exact mapping authority:

- SC-000561 becomes direct Cargo behavior; SC-000562 stays path safety; SC-000563 becomes the new Vitest behavior; SC-000564–SC-000578 stay generated authority integrity.
- SC-000579–SC-000593 delete historical current-definition/checkpoint/source-shape simulation row by row.
- SC-000594 uses dependency ownership; SC-000595–SC-000602 use generated authority integrity; SC-000603/605/608 delete zero-assertion checkpoint history; SC-000604/607/609–SC-000613 use the bounded Cargo identity tool; SC-000606 requires both generated authority and the tool.
- SC-000614–SC-000626 delete exact reference sets, counts/sites, checkpoint fixtures, import spellings, and signature/source-placement scans.
- SC-000627 and SC-000629–SC-000631 become the exact encrypted-session/runtime Cargo behavior identities from the map. SC-000628 and SC-000632 delete the unenforceable whole-source negative scans. SC-000633–SC-000647 delete legacy scanner mutation/source-site assertions.
- Keep SC-000648 ordinals 1–14 unchanged and replace its architecture subgroups with one `{15,16,17}` delete subgroup. Keep SC-000649–SC-000650 unchanged. Keep SC-000651 ordinals 3–9 unchanged and replace `{1,2,10,11}` with its documented transitional-layout deletion. Keep SC-000652 unchanged.
- Delete SC-000653's partial-layout matrix; keep SC-000654 as manifest ownership; delete SC-000655's exact facade path/count; keep SC-000656 as dependency ownership; delete SC-000657's item-level fixture inventory while retaining its durable feature-edge coverage through SC-000654; make SC-000658 tool-owned.

Every delete row/subgroup receives the specific reason in the ownership map. Do not claim that ordinary Cargo compilation proves a negative source-shape assertion. Record SC-000628, SC-000655, and SC-000657 as intentional losses in the Slice 3A verification report.

- [ ] **Step 6: Remove the legacy file and prove full Telegram closure**

Delete `src/lib/telegram-crate-boundary-contract.test.ts`. Update the registry test from the pre-cutover 22-ID candidate set to the current partial set: the four Telegram IDs plus currently implemented analysis IDs only. Run `node scripts/validate-testing-transition.mjs` and require every SC-000561–SC-000658 row to report closed without a Node Rust parser or persisted Cargo census.

- [ ] **Step 7: Verify Task 2B**

Run: `node scripts/run-vitest.mjs run src/lib/telegram-checkpoint-2.behavior.test.ts src/lib/telegram-contract-paths.behavior.test.ts scripts/testing/repository-index.test.ts scripts/testing/repository-rules.test.ts scripts/testing/validate-testing-transition.test.ts`; run both package-focused checks and package checkpoints from `## Rust Verification Loops`; run `node scripts/validate-testing-transition.mjs`; then run authoritative `npm.cmd run verify`.

## Task 3: Migrate the analysis source-reader contract

**Files:**

- Delete: `src/lib/analysis-source-readers.test.ts`
- Create: `src/lib/analysis-source-readers.behavior.component.test.ts`
- Modify: `scripts/testing/repository-rules.mjs`
- Modify: `scripts/testing/repository-rules.test.ts`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- SC-000222, SC-000224–SC-000237, SC-000243–SC-000264, SC-000266, SC-000269–SC-000270, and SC-000272–SC-000278 resolve to `analysis-source-readers.behavior.component.test.ts`. Task 3 updates only those replacement paths from the pre-migration `.behavior.test.ts` draft; stable row IDs, hashes, lineage, dispositions, assertion counts, and exact test titles remain unchanged.
- SC-000221, SC-000223, SC-000265, SC-000267, SC-000268, and SC-000271 resolve to their existing `rule:analysis-*` IDs.
- SC-000239 through SC-000241 close only by their existing documented deletion reasons.
- SC-000238 deletes the exact sticky/z-index visual contract because jsdom cannot expose Svelte scoped layout styles truthfully and DOM ordering remains covered by direct behavior. SC-000242 deletes the exact CSS hyphenation/wrapping contract while retaining accessible long-word text behavior elsewhere; neither row may be replaced by production-source/CSS text inspection or a new browser harness.

- [ ] **Step 1: Write failing direct source-reader behavior tests**

Use rendered components and route-owned fixtures to cover live sources, snapshots, source groups, focus controls, items, metadata, activity, and evidence highlights. Keep assertions to user-visible state, props, emitted actions, and accessible DOM rather than component implementation text.

- [ ] **Step 2: Run focused RED tests**

Run: `node scripts/run-vitest.mjs run src/lib/analysis-source-readers.behavior.component.test.ts`

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
- Modify: `src-tauri/src/lib.rs`
- Modify: `testing/source-contract-ledger.json`

**Interfaces:**

- `rule:analysis-crate-manifest-boundary` consumes only `RepositoryIndex.getCargoMetadata()` and returns deterministic violations for a missing/malformed `extractum-analysis` workspace library, a missing normal path dependency from `extractum`, or an application-only dependency such as Tauri on the analysis crate. It does not inspect Rust source and does not claim visibility, call-flow, SQL, transaction, command-body, or module-reachability facts.
- `test:cargo:extractum::tests::analysis_command_registration_inventory_is_exact` consumes the existing `application_command_inventory!` macro and proves the exact 21 analysis, three project, and three dev registration names occur once. Real handler expansion remains compiler proof that each registered symbol exists.
- Every SC-000029 through SC-000059 row retains its stable ID, hashes, lineage, and `assertionCount`. For a mixed row, enumerate the original assertions in source order and place every ordinal from `1` through `assertionCount` in exactly one subgroup. `validateLedgerShape` must prove that subgroup ordinals are total, disjoint, in range, and non-empty. Do not infer an ordinal from a replacement count or renumber after deleting the file.

### Approved row ownership and deletion partition

The `behavior` entries below are the only permitted direct replacement families. Expand every selected family to exact non-empty Cargo/Vitest identities in `replacementIds`; a package command, wildcard, prefix, or copied list is not a replacement identity. For each mixed row, behavior ordinals are only the source-order assertions whose observable outcome is proved by the named owner; all residual ordinals form one or more `delete` subgroups with the stated reason.

| Rows | Behavior or structured owner | Required deletion subgroup reason |
| --- | --- | --- |
| SC-000029 | none | The dual-owner selector is completed transition-harness behavior; the live owner is unconditionally the extracted crate. Delete all seven ordinals. |
| SC-000030 | none | Exact checkpoint-four `include!`, module, signature, and adapter spellings are completed staging shape. Delete all 31 ordinals; Cargo compilation and package tests exercise the final modules. |
| SC-000031 | `rule:analysis-crate-manifest-boundary` for the application-free package/dependency assertion | Delete residual function, module-depth, re-export, and path spellings because Cargo metadata cannot prove them. |
| SC-000032 | none | Exact `TauriAnalysisEventSink` bodies and absence of hidden side effects are Rust body shape. Delete all six ordinals and record the accepted risk; trait conformance remains compiler-owned and event outcomes remain covered by `extractum-analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order`. |
| SC-000033 | `extractum-analysis::state::tests::analysis_state_cancels_report_run_child_tokens`; `extractum::analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled` | Delete private-field, derive, accessor-visibility, and exact fixture call-order ordinals. |
| SC-000034 | none | The 95/48 Appendix-A partition and 143/169 identity counts are a completed extraction census. Delete all 13 ordinals; do not add a persistent Cargo test census. |
| SC-000035 | none | Singleton-`Arc` count and exact consumer spellings are application call-graph shape unavailable without Rust analysis. Delete all eight ordinals. |
| SC-000036 | `extractum::analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects`; `extractum::analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads`; `extractum-analysis::report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes`; `extractum-analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes`; `rule:analysis-crate-manifest-boundary` | Delete checkpoint include/path, exact signature/order, private-field, compression-import, and source-placement ordinals. |
| SC-000037 | `extractum-analysis::report::tests::scope::report_run_input_carries_resolved_profile_snapshot`; `extractum-analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order`; `extractum-analysis::report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails` | Delete opaque-ticket field/derive, exact signature, exact enum-layout, and re-export spellings. |
| SC-000038 | `extractum-analysis::chat::tests::chat_execution_persists_turns_before_completed_event`; `extractum-analysis::chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message`; `extractum::analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id`; `extractum::analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot` | Delete ticket visibility/derive, exact signatures, exact `tokio::spawn` topology, and forbidden-call spellings. |
| SC-000039–SC-000040 | `extractum-analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes` and the exact `extractum-analysis::store::tests::read_model::list_analysis_run_summaries_*` identities selected from Cargo `--list` | Delete private-field inventory, `Default`-absence, exact method-list, and whole-repository struct-literal scan ordinals. |
| SC-000041 | the nine exact `extractum::projects::data_range::tests::project_data_range_*` identities present in Cargo `--list` | Delete forbidden import, helper-name, literal SQL predicate, and error-message implementation ordinals. |
| SC-000042 | `extractum::analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion`; `extractum::analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs`; `extractum::analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field`; exact crate `corpus::tests::*` and `store::tests::read_model::*` identities selected from Cargo `--list` | Delete checkpoint include/module identities, adapter placement, exact transaction text, import depth, and forbidden-token inventories. |
| SC-000043 | none | Exact `#[path]`, `include!`, module ordering, and relative import depth are completed staging mechanics. Delete all 21 ordinals; the crate package gate compiles and executes the final store modules. |
| SC-000044 | exact crate `corpus::tests::*`, `report::tests::corpus_port::*`, and `corpus::tests::snapshot::*` behavior identities selected from Cargo `--list` | Delete exact facade/re-export/accessor inventory, private-field spelling, and legacy-type token-absence ordinals. |
| SC-000045 | exact crate `store::tests::read_model::*`, `store::tests::runs::*`, and `report::tests::lifecycle::*` behavior identities selected from Cargo `--list` | Delete raw-row/mapper placement, visibility, function-name, facade export, and consumer token-absence ordinals. |
| SC-000046 | `extractum-analysis::report::tests::scope::report_run_input_carries_resolved_profile_snapshot`; `extractum-analysis::report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities`; `extractum-analysis::report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label` | Delete exact ticket fields/derive, call spellings, and forbidden SQL/source lookup ordinals. |
| SC-000047 | `extractum-analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads`; `extractum-analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure`; `extractum-analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure` | Delete only duplicate source-string, helper-name, and occurrence-count ordinals not proved by those outcomes. |
| SC-000048 | exact app `analysis::corpus::tests::preflight::*` / `analysis::corpus::tests::live::*` and crate `corpus::tests::preflight::*` / `corpus::tests::live::*` identities selected from Cargo `--list` | Delete exact test counts, include directives, aliases, import depth, and retained/portable file-placement ordinals. |
| SC-000049 | new `extractum::tests::analysis_command_registration_inventory_is_exact` | Delete regex scanning of `#[tauri::command]` declarations. The direct macro test owns exact registration names and 21/3/3 counts; compiler expansion owns symbol existence. |
| SC-000050 | the three exact `extractum::projects::read_model::tests::list_research_projects_*` identities; `extractum::projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources`; the three exact `extractum::notebooklm_export::query::tests::load_export_source_group_*` identities | Delete exact startup invocation, compatibility alias, function-name, and cross-domain call-order ordinals. |
| SC-000051 | `extractum::account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only`; `extractum::account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned`; `extractum::diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates`; `extractum-analysis::state::tests::analysis_state_cancels_report_run_child_tokens`; `extractum::analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled` | Delete exact SQL tables/text, DTO/accessor/re-export spellings, call order, cleanup body, and duplicate-code absence ordinals. |
| SC-000052 | none | The Family-1 inclusion/exclusion table is a historical design classification plus a bounded `get_pool` source scan. Delete all five ordinals; individual chat, trace, run, deletion, cancellation, and snapshot behaviors remain under their direct tests. |
| SC-000053 | `extractum::notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation`; `extractum::notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic`; `extractum::notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id` | Delete exact participant/coordinator signatures, SQL-table absence, begin/commit topology, and query/order spelling ordinals. |
| SC-000054 | `extractum::projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials`; `extractum::projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout`; `extractum::projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first` | Delete exact aggregate DTO/accessors, query text, schema table census, re-exports, and single-transaction topology ordinals. |
| SC-000055 | `extractum::projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources` | Delete exact SQL statement counts/order, re-export/import spellings, await counts, and transaction body topology ordinals. |
| SC-000056 | `extractum::analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects`; every exact Vitest identity in `src/lib/api/analysis-runs.test.ts`, `analysis-chat.test.ts`, `analysis-trace.test.ts`, `analysis-source-groups.test.ts`, and `analysis-workspace.test.ts` | Delete regex-derived Rust command signatures, private request-ID construction, internal event-construction order, terminal-message source strings, and exact error-struct spelling ordinals. |
| SC-000057 | `rule:analysis-crate-manifest-boundary` only for final package/dependency ownership | Delete the 23/21/44/35/79 path counts, frozen move map, inventory-drift helper, forbidden Rust-token scan, and relative-depth spelling ordinals. |
| SC-000058 | none | Delete all 54 ordinals. They are the legacy Rust/SQL scanner itself: module reachability, SQL reconstruction, table ownership, async call topology, transaction-control detection, and fingerprints. Record the explicit high accepted risk; do not recreate the scanner. |
| SC-000059 | `extractum-analysis::corpus::tests::snapshot::list_run_snapshot_messages_page_does_not_fall_back_to_live_source`; `extractum-analysis::corpus::tests::snapshot::load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows`; `extractum-analysis::corpus::tests::snapshot::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot`; `extractum-analysis::corpus::tests::snapshot::source_group_membership_drift_after_capture_does_not_change_saved_run_corpus`; `extractum-analysis::test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys`; `extractum::tx::tests::begin_immediate_with_foreign_keys_enforces_cascade`; `extractum::tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction` | Delete SQL-parser probes, exact authorized-range census, negative-sentinel source scan, helper-body order, fingerprints, source reachability, and global foreign-SQL scan ordinals. |

- [ ] **Step 1: Write the two narrow RED tests**

Add a positive Cargo-metadata fixture plus independent mutations for missing workspace membership, missing `extractum` normal path dependency, missing library target, and an application-only Tauri dependency on `extractum-analysis`. Add `tests::analysis_command_registration_inventory_is_exact` against the existing macro; assert the exact 21 analysis, three project, and three dev names and one occurrence of each.

- [ ] **Step 2: Run the focused RED command**

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts -t "analysis crate manifest boundary"`

Expected: FAIL because the narrow evaluator is not registered.

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib tests::analysis_command_registration_inventory_is_exact -- --exact`

Expected: FAIL because the exact inventory test does not exist. A zero-test result is RED, not passing evidence.

- [ ] **Step 3: Implement only the narrow owners**

Implement `analysis-crate-manifest-boundary` from Cargo metadata only. It must report every declared package/dependency violation deterministically and convert unavailable or malformed metadata to `INFRA_ERROR`. Add the direct macro inventory test without scanning Rust source or duplicating the handler list outside the macro/test expected array.

- [ ] **Step 4: Apply the mechanical ledger partition**

For every SC-000029 through SC-000059 row, enumerate the frozen source-order assertion ordinals and apply the approved row table. Behavior/structured subgroups contain only ordinals truthfully covered by their exact replacement IDs; every residual ordinal has the table's explicit deletion reason. Before deleting the legacy file, run `node scripts/validate-testing-transition.mjs` and require these 31 rows to remain open only because their source path still exists; any missing, duplicate, out-of-range, empty, or wildcard-owned subgroup is a failure.

- [ ] **Step 5: Remove the legacy contract and validate closure**

Delete `src/lib/analysis-application-contract.test.ts`. Run `node scripts/validate-testing-transition.mjs` and require SC-000029 through SC-000059 to report closed.

- [ ] **Step 6: Verify Task 4**

Run the focused rule suite; both exact RED/GREEN Cargo identities; `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets`; `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets`; `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`; `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`; and authoritative unsandboxed `npm.cmd run verify`.

## Task 5: Slice 3A completeness, evidence, and handoff

**Files:**

- Create: `docs/superpowers/verification/2026-08-03-testing-redesign-slice-3a-source-contracts.md`
- Modify: `docs/superpowers/plans/2026-08-02-testing-redesign-program-index.md`

- [ ] **Step 1: Add the final 11-ID completeness assertion**

In `scripts/testing/repository-rules.test.ts`, derive the exact post-cutover rule-ID set from SC-000029–SC-000059, SC-000221–SC-000278, and SC-000561–SC-000658. Assert exact set equality with the registered evaluator IDs and retain the per-ID positive/mutation requirement. The set is four Telegram rules, six analysis source-reader rules, and `rule:analysis-crate-manifest-boundary`; the rejected `rule:repository-index:analysis-application-boundary` is absent.

Run: `node scripts/run-vitest.mjs run scripts/testing/repository-rules.test.ts`

Expected: PASS with exactly 11 implemented rule IDs; an absent, extra, or mutation-free rule fails.

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
| Telegram migration | SC-000561–SC-000658 close without a Node Rust parser or persistent Cargo census; four structured rules and one bounded transition-only Cargo identity comparison remain. SC-000648–SC-000652 retain their approved direct behavior and explicit deletion subgroups under stable IDs. |
| Source-reader migration | SC-000221–SC-000278 close, including three explicit deletion rows. |
| Analysis application migration | SC-000029–SC-000059 close through direct Cargo/Vitest behavior, one narrow Cargo-metadata manifest rule, and explicit ordinal-complete deletion subgroups; no Rust/SQL scanner remains. |
| Transition integrity | Census remains bidirectional and ledger closes the removed paths. |
| Completion gate | Unsandboxed `npm.cmd run verify` passes; sandbox-only taskkill denial is recorded separately rather than misclassified as a product failure. |
