# Takeout Forum-Topic Refresh Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refresh Telegram forum-topic catalog state after completed Takeout imports for eligible supergroup sources, including completed partial imports, without failing the Takeout batch when refresh itself fails.

**Architecture:** Reuse the existing Rust forum-topic refresh path from normal sync. Add a small Takeout-owned helper that decides when completed Takeout should invoke the refresh, maps actionable refresh warnings to one durable warning code, and wire it after successful Takeout session finish but before `finalize_ingest_batch`.

**Tech Stack:** Rust, Tauri backend, grammers Telegram client, SQLx SQLite, existing Takeout provenance tables, Markdown verification docs.

---

## File Structure

- Modify `src-tauri/src/sources/topics.rs`: widen `refresh_forum_topics` visibility from `pub(super)` to `pub(crate)`.
- Modify `src-tauri/src/sources/mod.rs`: re-export `refresh_forum_topics` for internal backend callers.
- Modify `src-tauri/src/sources/sync.rs`: import the re-exported helper so normal sync keeps the same behavior.
- Create `src-tauri/src/takeout_import/forum_topics.rs`: own Takeout-specific refresh policy, durable warning mapping, and unit tests.
- Modify `src-tauri/src/takeout_import/mod.rs`: declare the new module and call the helper in the completed Takeout path.
- Modify `docs/backlog.md`: close the forum-topic policy decision row and leave live evidence caveats visible.
- Modify `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`: record that the policy decision is now code-backed, while historical partial batches remain only supporting evidence.

No database migration, new Tauri command, frontend control, migrated-history import, or normal sync behavior change is part of this plan.

### Task 1: Expose The Existing Refresh Helper Internally

**Files:**
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/sync.rs`

- [ ] **Step 1: Widen the helper visibility**

In `src-tauri/src/sources/topics.rs`, change the function signature from:

```rust
pub(super) async fn refresh_forum_topics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
) -> Vec<String> {
```

to:

```rust
pub(crate) async fn refresh_forum_topics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
) -> Vec<String> {
```

- [ ] **Step 2: Re-export the helper for Rust-internal callers**

In `src-tauri/src/sources/mod.rs`, replace:

```rust
pub use topics::list_source_forum_topics;
```

with:

```rust
pub use topics::list_source_forum_topics;
pub(crate) use topics::refresh_forum_topics;
```

- [ ] **Step 3: Keep normal sync on the shared import path**

In `src-tauri/src/sources/sync.rs`, remove this import:

```rust
use super::topics::refresh_forum_topics;
```

and import the re-exported helper from the parent module by replacing:

```rust
use super::store::load_source;
use super::topics::refresh_forum_topics;
use super::types::{
```

with:

```rust
use super::store::load_source;
use super::refresh_forum_topics;
use super::types::{
```

If rustfmt reorders imports, keep the final code rustfmt-clean and do not change runtime logic.

- [ ] **Step 4: Verify the visibility-only refactor**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: exit 0.

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add src-tauri/src/sources/topics.rs src-tauri/src/sources/mod.rs src-tauri/src/sources/sync.rs
git commit -m "refactor: expose forum topic refresh internally"
```

### Task 2: Takeout Refresh Policy And Warning Mapping

**Files:**
- Create: `src-tauri/src/takeout_import/forum_topics.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Declare the new module**

In `src-tauri/src/takeout_import/mod.rs`, add the module declaration after `mod export_dc;`:

```rust
mod export_dc;
mod forum_topics;
mod pagination;
```

- [ ] **Step 2: Write the failing policy and durable-warning tests**

Create `src-tauri/src/takeout_import/forum_topics.rs` with compileable failing stubs and tests:

```rust
use crate::error::AppResult;
use crate::ingest_provenance::TerminalBatchStatus;

pub(crate) const FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE: &str =
    "forum_topic_refresh_failed";
const FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING: &str =
    "Forum topic refresh after Takeout failed; existing topic catalog remains available.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeoutForumTopicRefreshPolicy {
    Refresh,
    Skip,
}

pub(crate) fn completed_takeout_forum_topic_refresh_policy(
    _terminal_status: TerminalBatchStatus,
    _source_subtype: &str,
) -> TakeoutForumTopicRefreshPolicy {
    TakeoutForumTopicRefreshPolicy::Skip
}

pub(crate) async fn record_takeout_forum_topic_refresh_failure_if_needed(
    _pool: &sqlx::SqlitePool,
    _batch_id: i64,
    _warnings: &mut Vec<String>,
    _refresh_warnings: &[String],
) -> AppResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        completed_takeout_forum_topic_refresh_policy,
        record_takeout_forum_topic_refresh_failure_if_needed,
        TakeoutForumTopicRefreshPolicy, FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING,
        FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE,
    };
    use crate::ingest_provenance::{
        create_telegram_takeout_batch, finalize_ingest_batch, CreateTelegramTakeoutBatch,
        TerminalBatchStatus,
    };
    use crate::sources::test_support::{
        create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
    };
    use crate::sources::{
        TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
    };

    #[test]
    fn completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups() {
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Completed,
                TELEGRAM_KIND_SUPERGROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Refresh,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Completed,
                TELEGRAM_KIND_CHANNEL,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Completed,
                TELEGRAM_KIND_GROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Failed,
                TELEGRAM_KIND_SUPERGROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Cancelled,
                TELEGRAM_KIND_SUPERGROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
    }

    #[tokio::test]
    async fn takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let mut warnings = Vec::new();

        record_takeout_forum_topic_refresh_failure_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &[
                "Forum topic refresh failed for source 1: network".to_string(),
                "Forum topic refresh failed for source 1: retry".to_string(),
            ],
        )
        .await
        .expect("record refresh warning");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize completed batch");

        assert_eq!(
            warnings,
            vec![FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING.to_string()]
        );
        let warning_rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT code, message FROM ingest_batch_warnings WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_all(&pool)
        .await
        .expect("load warning rows");
        assert_eq!(
            warning_rows,
            vec![(
                FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE.to_string(),
                FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING.to_string(),
            )],
        );
        let warning_count: i64 =
            sqlx::query_scalar("SELECT warning_count FROM ingest_batches WHERE id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load warning count");
        assert_eq!(warning_count, 1);
    }

    #[tokio::test]
    async fn takeout_forum_topic_refresh_success_records_no_warning() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let mut warnings = vec!["existing warning".to_string()];

        record_takeout_forum_topic_refresh_failure_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &[],
        )
        .await
        .expect("record no refresh warning");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize completed batch");

        assert_eq!(warnings, vec!["existing warning".to_string()]);
        let warning_count: i64 =
            sqlx::query_scalar("SELECT warning_count FROM ingest_batches WHERE id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load warning count");
        assert_eq!(warning_count, 0);
    }

    async fn seed_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
    }
}
```

- [ ] **Step 3: Run the focused Rust tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_forum_topic
```

Expected: fail. At least `completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups` must fail because the stub returns `Skip`; the warning-count test should also fail because the stub records no durable warning.

- [ ] **Step 4: Implement the policy helper**

Replace the non-test body of `src-tauri/src/takeout_import/forum_topics.rs`, above `#[cfg(test)]`, with:

```rust
use grammers_client::Client;
use grammers_session::types::PeerRef;

use crate::error::AppResult;
use crate::ingest_provenance::{record_ingest_batch_warning, TerminalBatchStatus};
use crate::sources::{
    refresh_forum_topics, SourceSyncTarget, TELEGRAM_KIND_SUPERGROUP,
};

pub(crate) const FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE: &str =
    "forum_topic_refresh_failed";
const FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING: &str =
    "Forum topic refresh after Takeout failed; existing topic catalog remains available.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeoutForumTopicRefreshPolicy {
    Refresh,
    Skip,
}

pub(crate) fn completed_takeout_forum_topic_refresh_policy(
    terminal_status: TerminalBatchStatus,
    source_subtype: &str,
) -> TakeoutForumTopicRefreshPolicy {
    match (terminal_status, source_subtype) {
        (TerminalBatchStatus::Completed, TELEGRAM_KIND_SUPERGROUP) => {
            TakeoutForumTopicRefreshPolicy::Refresh
        }
        _ => TakeoutForumTopicRefreshPolicy::Skip,
    }
}

pub(crate) async fn refresh_forum_topics_after_completed_takeout(
    pool: &sqlx::SqlitePool,
    batch_id: i64,
    client: &Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
    source_subtype: &str,
    warnings: &mut Vec<String>,
) -> AppResult<()> {
    if completed_takeout_forum_topic_refresh_policy(
        TerminalBatchStatus::Completed,
        source_subtype,
    ) == TakeoutForumTopicRefreshPolicy::Skip
    {
        return Ok(());
    }

    let refresh_warnings = refresh_forum_topics(pool, client, peer, source).await;
    record_takeout_forum_topic_refresh_failure_if_needed(
        pool,
        batch_id,
        warnings,
        &refresh_warnings,
    )
    .await
}

pub(crate) async fn record_takeout_forum_topic_refresh_failure_if_needed(
    pool: &sqlx::SqlitePool,
    batch_id: i64,
    warnings: &mut Vec<String>,
    refresh_warnings: &[String],
) -> AppResult<()> {
    if refresh_warnings.is_empty() {
        return Ok(());
    }

    if !warnings
        .iter()
        .any(|warning| warning == FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING)
    {
        warnings.push(FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING.to_string());
    }
    record_ingest_batch_warning(
        pool,
        batch_id,
        FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE,
        FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING,
    )
    .await
}
```

- [ ] **Step 5: Run the focused Rust tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_forum_topic
```

Expected: all `takeout_forum_topic` tests pass.

- [ ] **Step 6: Commit Task 2**

Run:

```powershell
git add src-tauri/src/takeout_import/mod.rs src-tauri/src/takeout_import/forum_topics.rs
git commit -m "feat: add takeout forum topic refresh policy"
```

### Task 3: Wire Refresh Into Completed Takeout

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Import the helper**

In `src-tauri/src/takeout_import/mod.rs`, add this import near the existing internal module imports:

```rust
use forum_topics::refresh_forum_topics_after_completed_takeout;
```

The nearby imports should look like:

```rust
use export_dc::{
    export_dc_invoke, finish_takeout_session, prepare_export_dc_alias,
    takeout_init_request_for_source_subtype, ExportDcAlias, ExportDcAttemptState,
};
use forum_topics::refresh_forum_topics_after_completed_takeout;
use pagination::{
```

- [ ] **Step 2: Call refresh after successful Takeout finish and before batch finalization**

In `run_started_takeout_source_import_inner`, find this block:

```rust
    record_export_dc_fallback_if_needed(
        pool,
        batch_id,
        warnings,
        fallback_before,
        *fallback_used,
        export_attempts,
    )
    .await?;
    finalize_sync(
        &pool,
        &source,
        source.last_sync_state.unwrap_or(0),
        import.max_message_id,
        resolved_peer.refreshed_avatar_cache_key,
    )
    .await?;
    finalize_ingest_batch(pool, batch_id, TerminalBatchStatus::Completed, None).await?;
```

Replace it with:

```rust
    record_export_dc_fallback_if_needed(
        pool,
        batch_id,
        warnings,
        fallback_before,
        *fallback_used,
        export_attempts,
    )
    .await?;
    refresh_forum_topics_after_completed_takeout(
        pool,
        batch_id,
        client,
        resolved_peer.peer,
        source,
        telegram_source_subtype,
        warnings,
    )
    .await?;
    finalize_sync(
        &pool,
        &source,
        source.last_sync_state.unwrap_or(0),
        import.max_message_id,
        resolved_peer.refreshed_avatar_cache_key,
    )
    .await?;
    finalize_ingest_batch(pool, batch_id, TerminalBatchStatus::Completed, None).await?;
```

This preserves the chosen ordering: successful Takeout finish, export-DC provenance, forum-topic refresh, source sync finalization, then completed batch finalization.

- [ ] **Step 3: Run focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_forum_topic
cargo test --manifest-path src-tauri/Cargo.toml non_forum_topic_refresh_errors_are_detected
cargo test --manifest-path src-tauri/Cargo.toml topic_refresh_rebuilds_materialized_memberships
```

Expected: all focused tests pass.

- [ ] **Step 4: Run cargo check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: exit 0.

- [ ] **Step 5: Commit Task 3**

Run:

```powershell
git add src-tauri/src/takeout_import/mod.rs
git commit -m "feat: refresh forum topics after completed takeout"
```

### Task 4: Documentation And Final Verification

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`
- Modify: `docs/superpowers/plans/2026-05-25-takeout-forum-topic-refresh-policy.md`

- [ ] **Step 1: Update the backlog decision**

In `docs/backlog.md`, replace:

```markdown
- [ ] decide whether Takeout import should refresh the forum-topic catalog after successful finish
  - Source `21` / batch `4` partial Takeout materially increased topic
    memberships without refreshing the topic catalog; completed supergroup
    evidence is still needed before changing behavior.
  - Source `22` / batch `11` partial Takeout added `10030` topic memberships
    while the topic catalog aggregate remained unchanged; this strengthens the
    decision input but still does not justify behavior changes without
    completed supergroup evidence.
```

with:

```markdown
- [x] decide whether Takeout import should refresh the forum-topic catalog after successful finish
  - Policy implemented: completed Takeout imports refresh forum topics for
    eligible supergroup sources, including completed partial imports, while
    failed and cancelled attempts do not refresh.
  - Refresh failures preserve completed Takeout status and record durable
    warning code `forum_topic_refresh_failed`.
  - Source `21` / batch `4` and source `22` / batch `11` remain sanitized
    partial-run decision input, not proof of completed live behavior.
```

- [ ] **Step 2: Update the verification matrix row**

In `docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md`, replace the row:

```markdown
| Forum-topic decision input | needs follow-up | 22 | 11 | topic catalog/membership aggregate counters from bounded partial Takeout | Bounded partial runs materially increased topic memberships without refreshing the topic catalog, which is useful decision input; completed supergroup Takeout evidence is still needed before changing behavior |
```

with:

```markdown
| Forum-topic decision input | passed | 22 | 11 | topic catalog/membership aggregate counters from bounded partial Takeout plus code-level refresh-policy tests | Policy is now decided and code-backed: completed supergroup Takeout refreshes the topic catalog and refresh failure records `forum_topic_refresh_failed`; source 21/22 partial runs remain supporting decision input, not completed live proof |
```

- [ ] **Step 3: Add an implementation note near the source 22 conclusion**

In the same verification doc, replace:

```markdown
watermark behavior. The forum-topic decision input row remains
`needs follow-up`: the partial run added `10030` topic memberships while the
topic catalog aggregate stayed unchanged.
```

with:

```markdown
watermark behavior. The historical partial run added `10030` topic memberships
while the topic catalog aggregate stayed unchanged. The forum-topic policy
decision is now closed by code-level tests and runtime wiring: completed
supergroup Takeout refreshes the topic catalog, while cancelled and failed
Takeout attempts still do not refresh.
```

- [ ] **Step 4: Mark this implementation plan complete as work lands**

In `docs/superpowers/plans/2026-05-25-takeout-forum-topic-refresh-policy.md`, change every completed checkbox from `- [ ]` to `- [x]`. Do this only after the corresponding command or edit has actually completed.

- [ ] **Step 5: Run final verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- `cargo test` exits 0.
- `npm.cmd test` exits 0.
- `npm.cmd run check` exits 0.
- `git diff --check` exits 0.

- [ ] **Step 6: Commit Task 4**

Run:

```powershell
git add docs/backlog.md docs/superpowers/verification/takeout-representative-validation-and-fallback-coverage.md docs/superpowers/plans/2026-05-25-takeout-forum-topic-refresh-policy.md
git commit -m "docs: record takeout forum topic refresh policy"
```

## Final Acceptance Checklist

- [ ] Completed Takeout imports call the shared forum-topic refresh helper for eligible supergroup sources.
- [ ] Completed partial Takeout imports follow the same refresh policy because the hook runs before completeness classification and only depends on completed terminal status plus source subtype.
- [ ] Failed and cancelled Takeout paths do not call the completed-path helper.
- [ ] Actionable refresh failures record durable warning code `forum_topic_refresh_failed`.
- [ ] Refresh failure does not turn a completed Takeout batch into a failed batch.
- [ ] Non-forum outcomes remain silent no-ops through the existing refresh helper.
- [ ] No private Telegram content, raw provider data, warning bodies, source titles, usernames, phone numbers, or session material are added to docs or tests.
- [ ] Normal sync still uses the same forum-topic refresh behavior as before.
