# Takeout Migrated-History Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lock the selected migrated-history policy into tests, recovery copy, and current-state docs without enabling old small-group history import.

**Architecture:** Keep normal Takeout as a current-history importer. Treat old small-group history as a separate historical scope that remains detected and deferred unless a future explicit opt-in importer is designed. Add one Rust provenance guard for idempotent migrated deferment, update frontend warning copy, and align docs/backlog with the policy decision.

**Tech Stack:** Rust/Tauri backend with SQLite provenance, Svelte/TypeScript frontend state helpers, Vitest, Cargo tests, Markdown docs.

---

## Spec

Read first:

- `docs/superpowers/specs/2026-05-26-takeout-migrated-history-policy-design.md`

The implementation must preserve these policy decisions:

- old small-group history is a separate historical scope;
- normal Takeout reruns do not import old `chat` rows;
- `migrated_history_deferred` is intentional deferment, not an automatic retry promise;
- no UI button, command, schema migration, or live Telegram validation is added in this slice;
- warning bodies and private Telegram data remain hidden.

## File Structure

- Modify: `src-tauri/src/ingest_provenance.rs`
  - Add a focused idempotence helper for migrated-history warning insertion.
  - Add a regression test that finalizes a deferred migrated-history batch as partial with one durable warning row.
- Modify: `src/lib/analysis-state.ts`
  - Clarify the `migrated_history_deferred` explanation as a separate historical scope.
- Modify: `src/lib/analysis-state.test.ts`
  - Update warning-copy expectations to match the selected policy.
- Modify: `docs/backlog.md`
  - Replace "decide migrated-history import policy" with the next explicit opt-in outcome.
  - Mark the policy decision as closed and keep implementation enablement open.
- Modify: `docs/takeout-source-import.md`
  - Document the selected historical-scope policy in the Takeout import contract.
- Modify: `docs/architecture-deep-dive.md`
  - Replace stale "policy not decided" wording with the selected policy and future opt-in caveat.
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
  - Add a dated note tying source `115` / batch `18` evidence to the chosen policy without claiming old-history import support.
- Modify: `docs/superpowers/plans/2026-05-26-takeout-migrated-history-policy.md`
  - Track task completion during execution.

---

### Task 1: Backend Provenance Guard

**Files:**
- Modify: `src-tauri/src/ingest_provenance.rs`
- Modify: `docs/superpowers/plans/2026-05-26-takeout-migrated-history-policy.md`

- [x] **Step 1: Write the failing idempotence regression**

In `src-tauri/src/ingest_provenance.rs`, add this test inside the existing `#[cfg(test)] mod tests`, after `completed_zero_observation_batch_is_complete_without_partial_flags` and before `mixed_partial_scope_finalizes_as_partial`:

```rust
    #[tokio::test]
    async fn migrated_history_deferred_scope_finalizes_partial_and_records_warning_once() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        mark_takeout_migrated_history_deferred(&pool, batch_id, "historical scope detected")
            .await
            .expect("mark migrated deferred");
        mark_takeout_migrated_history_deferred(&pool, batch_id, "historical scope detected again")
            .await
            .expect("mark migrated deferred again");

        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize partial batch");

        let row: (String, String, i64, i64, i64) = sqlx::query_as(
            "SELECT b.completeness, t.history_scope, t.migrated_history_detected,
                    t.migrated_history_imported, b.warning_count
             FROM ingest_batches b
             JOIN telegram_takeout_batches t ON t.batch_id = b.id
             WHERE b.id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load final state");
        assert_eq!(
            row,
            (
                "partial".to_string(),
                "current_history_with_migrated_deferred".to_string(),
                1,
                0,
                1,
            )
        );

        let warning_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ingest_batch_warnings
             WHERE batch_id = ? AND code = 'migrated_history_deferred'",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("count migrated warnings");
        assert_eq!(warning_rows, 1);
    }
```

- [x] **Step 2: Run the focused test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
```

Expected: FAIL because `mark_takeout_migrated_history_deferred` currently records a warning row every time it is called, so `warning_count` and `warning_rows` are `2`.

- [x] **Step 3: Add a one-shot warning helper**

In `src-tauri/src/ingest_provenance.rs`, add this helper immediately after `record_ingest_batch_warning`:

```rust
async fn record_ingest_batch_warning_once(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    code: &str,
    message: &str,
) -> AppResult<()> {
    let existing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ingest_batch_warnings WHERE batch_id = ? AND code = ?",
    )
    .bind(batch_id)
    .bind(code)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    if existing > 0 {
        return Ok(());
    }
    record_ingest_batch_warning(pool, batch_id, code, message).await
}
```

- [x] **Step 4: Use the helper for migrated-history deferment**

In `mark_takeout_migrated_history_deferred`, replace:

```rust
    record_ingest_batch_warning(pool, batch_id, "migrated_history_deferred", message).await
```

with:

```rust
    record_ingest_batch_warning_once(pool, batch_id, "migrated_history_deferred", message).await
```

Do not change `record_ingest_batch_warning` itself and do not change the only-my-messages warning path.

- [x] **Step 5: Run the focused test to verify it passes**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
```

Expected: PASS.

- [x] **Step 6: Run related Rust regression tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml mixed_partial_scope_finalizes_as_partial
cargo test --manifest-path src-tauri\Cargo.toml takeout_validation_batch_summary_is_durable_and_sanitized
cargo test --manifest-path src-tauri\Cargo.toml insert_telegram_source_item_allows_same_message_id_in_different_history_domains
```

Expected:

- `mixed_partial_scope_finalizes_as_partial` passes and still preserves `history_scope = mixed_partial`;
- `takeout_validation_batch_summary_is_durable_and_sanitized` passes and still exposes warning codes without warning bodies;
- `insert_telegram_source_item_allows_same_message_id_in_different_history_domains` passes, confirming current and historical peer identities can coexist without relying on `items.external_id` uniqueness.

- [x] **Step 7: Mark Task 1 complete and commit**

Update this plan's Task 1 checkboxes to `[x]`, then run:

```powershell
git add src-tauri\src\ingest_provenance.rs docs\superpowers\plans\2026-05-26-takeout-migrated-history-policy.md
git commit -m "test: guard migrated history deferment provenance"
```

Expected: commit succeeds with the Rust provenance guard and this plan update.

---

### Task 2: Recovery Copy Policy

**Files:**
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `docs/superpowers/plans/2026-05-26-takeout-migrated-history-policy.md`

- [x] **Step 1: Update the frontend warning-copy test first**

In `src/lib/analysis-state.test.ts`, inside `explains known takeout recovery warning codes without inventing unknown explanations`, replace the expected migrated-history explanation:

```ts
      "Migrated small-group history was detected and intentionally deferred.",
```

with:

```ts
      "Migrated small-group history was detected as a separate historical scope. Normal Takeout reruns keep it deferred until an explicit historical import exists.",
```

- [x] **Step 2: Run the focused frontend test to verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: FAIL with a mismatch for the `migrated_history_deferred` explanation.

- [x] **Step 3: Update the recovery warning explanation**

In `src/lib/analysis-state.ts`, replace:

```ts
  migrated_history_deferred:
    "Migrated small-group history was detected and intentionally deferred.",
```

with:

```ts
  migrated_history_deferred:
    "Migrated small-group history was detected as a separate historical scope. Normal Takeout reruns keep it deferred until an explicit historical import exists.",
```

- [x] **Step 4: Run the focused frontend test to verify it passes**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: PASS.

- [x] **Step 5: Mark Task 2 complete and commit**

Update this plan's Task 2 checkboxes to `[x]`, then run:

```powershell
git add src\lib\analysis-state.ts src\lib\analysis-state.test.ts docs\superpowers\plans\2026-05-26-takeout-migrated-history-policy.md
git commit -m "feat: clarify migrated history recovery policy"
```

Expected: commit succeeds with only frontend recovery-copy and plan updates.

---

### Task 3: Current-State Documentation

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/takeout-source-import.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify: `docs/superpowers/plans/2026-05-26-takeout-migrated-history-policy.md`

- [ ] **Step 1: Update backlog priority snapshot**

In `docs/backlog.md`, change the `Updated` date:

```markdown
> **Updated:** 2026-05-26
```

Then replace the Takeout priority row with:

```markdown
| High | Takeout source import | define explicit opt-in behavior for migrated historical scope before enabling import |
```

- [ ] **Step 2: Replace the open migrated-history item**

In `docs/backlog.md`, replace:

```markdown
- [ ] enable migrated small-group history only after provenance and real-data
  validation prove the typed Telegram identity boundary is safe
```

with:

```markdown
- [x] decide migrated-history import policy on top of persisted provenance
  - Policy selected on 2026-05-26: migrated small-group history is a separate
    historical scope, not part of normal current supergroup Takeout reruns.
    Normal reruns keep `migrated_history_deferred`; any future import requires
    an explicit opt-in historical-scope design.
- [ ] define explicit opt-in behavior for migrated historical scope before
  enabling old small-group history import
```

- [ ] **Step 3: Update Takeout import docs**

In `docs/takeout-source-import.md`, replace this paragraph:

```markdown
For supergroups, `channels.getFullChannel` is used to detect
`migrated_from_chat_id`. Migrated small-group history is currently not
imported. The storage layer can represent overlapping Telegram message ids
through `telegram_messages`, but product enablement is deferred until durable
Takeout provenance and real-data validation are designed.
```

with:

```markdown
For supergroups, `channels.getFullChannel` is used to detect
`migrated_from_chat_id`. Migrated small-group history is treated as a separate
historical scope. Normal Takeout imports keep importing current supergroup
history only and record `migrated_history_deferred` when the historical scope is
detected. The storage layer can represent overlapping Telegram message ids
through `telegram_messages`, but importing old `chat` history requires a future
explicit opt-in historical-scope design.
```

In the recovery section, replace:

```markdown
Migrated supergroup history remains disabled in this foundation slice. When it
is detected, Takeout records `migrated_history_detected = 1`,
`migrated_history_imported = 0`, a `migrated_history_deferred` warning, and
partial completeness. Read-only recovery state does not enable migrated-history
import, resume, purge, or automatic retry.
```

with:

```markdown
Migrated supergroup history remains disabled for normal Takeout reruns. When it
is detected, Takeout records `migrated_history_detected = 1`,
`migrated_history_imported = 0`, a `migrated_history_deferred` warning, and
partial completeness. The selected policy treats old small-group history as a
separate historical scope, so read-only recovery state does not enable
migrated-history import, resume, purge, or automatic retry.
```

Finally replace the open-validation sentence:

```markdown
Open validation still belongs in the backlog: broader real-account coverage for
supergroups, groups, private/left sources, shifted export DC behavior,
completed forum-topic catalog deltas, and migrated-history import policy.
```

with:

```markdown
Open validation still belongs in the backlog: broader real-account coverage for
supergroups, groups, private/left sources, and explicit opt-in behavior for the
migrated historical scope before old small-group history import is enabled.
```

- [ ] **Step 4: Update architecture docs**

In `docs/architecture-deep-dive.md`, replace the source-kind bullet:

```markdown
- `supergroup`: import the last split only and warn if migrated small-group history is detected;
```

with:

```markdown
- `supergroup`: import the last current-history split only and warn if migrated small-group history is detected as a separate historical scope;
```

Then replace the known-debt bullet:

```markdown
- migrated supergroup history is detected but not imported until the import
  policy and real-data validation prove the typed history boundary is safe;
```

with:

```markdown
- migrated supergroup history is detected as a separate historical scope; old
  small-group import still needs explicit opt-in behavior before it can be
  enabled;
```

- [ ] **Step 5: Update the representative validation matrix note**

In `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`, add this dated note immediately after `### 2026-05-25 Shifted Export DC Fallback Code-Backed Validation` section, before the `2026-05-24 Source 115 Migrated Takeout Smoke Pre-Run Plan` section:

```markdown
### 2026-05-26 Migrated-History Policy Decision

Policy decision: migrated small-group history is a separate historical scope,
not part of normal current supergroup Takeout reruns.

Existing source `115` / batch `18` evidence remains the representative
detect-and-defer proof: `migrated_history_detected = 1`,
`migrated_history_imported = 0`, `history_scope =
current_history_with_migrated_deferred`, one `migrated_history_deferred`
warning code, and zero old `chat` rows imported.

Result: normal Takeout reruns keep old small-group history deferred. Any future
old-history import requires a separate explicit opt-in historical-scope design
and must not be inferred from the passed smoke row.
```

- [ ] **Step 6: Search docs for stale policy wording**

Run:

```powershell
rg -n "migrated-history import policy|policy and real-data validation|product enablement is deferred|Migrated supergroup history remains disabled|enable migrated small-group history" docs\backlog.md docs\takeout-source-import.md docs\architecture-deep-dive.md docs\superpowers\verification\takeout-representative-validation-and-fallback-coverage.md
```

Expected: no stale wording remains in the edited current-state docs. It is acceptable for the new backlog item to mention `enabling old small-group history import`.

- [ ] **Step 7: Verify docs whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code `0`.

- [ ] **Step 8: Mark Task 3 complete and commit**

Update this plan's Task 3 checkboxes to `[x]`, then run:

```powershell
git add docs\backlog.md docs\takeout-source-import.md docs\architecture-deep-dive.md docs\superpowers\verification\takeout-representative-validation-and-fallback-coverage.md docs\superpowers\plans\2026-05-26-takeout-migrated-history-policy.md
git commit -m "docs: record migrated history historical-scope policy"
```

Expected: commit succeeds with only docs and plan updates.

---

### Task 4: Final Verification And Plan Completion

**Files:**
- Modify: `docs/superpowers/plans/2026-05-26-takeout-migrated-history-policy.md`

- [ ] **Step 1: Run focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
cargo test --manifest-path src-tauri\Cargo.toml mixed_partial_scope_finalizes_as_partial
cargo test --manifest-path src-tauri\Cargo.toml takeout_validation_batch_summary_is_durable_and_sanitized
cargo test --manifest-path src-tauri\Cargo.toml insert_telegram_source_item_allows_same_message_id_in_different_history_domains
```

Expected: all focused Rust tests pass.

- [ ] **Step 2: Run completed Rust slice verification**

Run:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
cargo test --manifest-path src-tauri\Cargo.toml
```

Expected: `cargo check` passes and full Rust tests pass.

Known caveat: do not use full `cargo fmt --manifest-path src-tauri\Cargo.toml --check` as a completion gate for this slice unless the unrelated pre-existing rustfmt drift has already been handled.

- [ ] **Step 3: Run frontend verification**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
npm.cmd test
npm.cmd run check
```

Expected: focused Vitest passes, full Vitest passes, and Svelte check reports zero errors.

- [ ] **Step 4: Run repository hygiene checks**

Run:

```powershell
git diff --check
git status --short --branch
git log --oneline -5
```

Expected: no whitespace errors; branch is `main`; tracked changes are only this plan's unchecked-to-checked completion marks before the final commit.

- [ ] **Step 5: Mark Task 4 complete and commit**

Update this plan's Task 4 checkboxes and any remaining unchecked completed steps to `[x]`, then run:

```powershell
git add docs\superpowers\plans\2026-05-26-takeout-migrated-history-policy.md
git commit -m "docs: mark migrated history policy plan complete"
```

Expected: commit succeeds with only the plan completion update.

## Self-Review

- Spec coverage: Task 1 covers backend provenance and idempotent warning durability; Task 2 covers recovery semantics; Task 3 covers backlog/current-state docs and validation framing; Task 4 covers Rust, frontend, and repository verification.
- Placeholder scan: this plan contains no placeholder markers or vague unfinished steps.
- Type consistency: warning code `migrated_history_deferred`, history scope `current_history_with_migrated_deferred`, `migrated_history_imported`, and `is_migrated_history` match existing schema and code vocabulary.
- Scope check: no task enables old small-group import, adds UI opt-in controls, changes schema, or runs live Telegram validation.
