# Migrated History Scope Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit browsing, NotebookLM export, and analysis controls for Telegram migrated small-group history while keeping default behavior current-history-only.

**Architecture:** Preserve the existing storage contract: migrated rows stay in `items + telegram_messages` with `is_migrated_history = 1` and do not enter default `analysis_documents` or `archive_read_items`. Telegram browsing always uses a scoped direct item loader with row markers and backend-opaque deterministic cursors; archive first-page reads are not mixed with direct cursor paging while archive rows lack the full Telegram ordering tuple. Export gets an explicit opt-in that renders separate current/migrated sections; analysis gets an explicit opt-in that loads migrated rows directly into the saved snapshot and records the run-level decision.

**Tech Stack:** Rust/Tauri commands, SQLx SQLite migrations and tests, existing analysis/report and NotebookLM export modules, Svelte 5, TypeScript API wrappers, Vitest, Cargo tests.

---

## Spec

Read first:

- `docs/superpowers/specs/2026-05-28-migrated-history-scope-product-behavior-design.md`
- `docs/superpowers/plans/2026-05-27-takeout-migrated-history-opt-in.md`
- `docs/takeout-source-import.md`
- `docs/database-schema.md`

This plan implements these accepted decisions:

- Default browsing, analysis, and export remain current supergroup history only.
- User-facing scope labels are stable:
  - `Current supergroup history`
  - `Migrated small-group history`
  - `Merged timeline`
- API enum names are stable:

```ts
export type TelegramHistoryScope = "current" | "migrated" | "merged";
```

- Backend item calls default to `current` even when called directly without a scope.
- Migrated labels come from backend-owned DTO fields, not frontend heuristics.
- `merged` ordering is deterministic across paging and around-item loading:
  1. `published_at`
  2. history-scope order, current before migrated on ties
  3. `history_peer_kind`, `history_peer_id`, `telegram_message_id`
  4. local `item_id`
- The full cursor tuple stays backend-internal. The source-reader DTO exposes `page_cursor: string` / `before_cursor?: string | null` as an opaque encoded cursor so raw Telegram peer ids are not copied into frontend state, logs, or snapshots.
- Telegram source browsing uses the direct scoped query for first and subsequent pages. The archive read model may remain available for non-Telegram or legacy current-only paths, but it is not used for Telegram reader pagination until it stores the full ordering tuple.
- `analysis_runs.telegram_history_scope` is a nullable text/check migration; old runs map to `current`.
- Source-group analysis opt-in includes migrated rows only for group members that actually have imported migrated rows.
- Opt-in migrated rows participate in analysis preflight message count, chunk estimate, and estimated input chars.
- NotebookLM first slice uses separate current/migrated sections, not a silently merged export.

## File Structure

- Create: `src-tauri/migrations/0003_analysis_telegram_history_scope.sql`
  - Add nullable `analysis_runs.telegram_history_scope` with a check constraint.
- Modify: `src-tauri/src/migrations.rs`
  - Register migration `0003` and test the column exists after fresh and upgrade paths.
- Modify: `src-tauri/src/sources/types.rs`
  - Add shared Telegram history-scope enum, labels, cursor type, and source capability counts.
- Modify: `src-tauri/src/sources/store.rs`
  - Expose sanitized `migrated_history_row_count` and `migrated_history_import_completed`.
- Modify: `src-tauri/src/sources/items.rs`
  - Accept `history_scope`, opaque `before_cursor`, and return row-level scope markers plus opaque `page_cursor`.
- Modify: `src-tauri/src/sources/items/query.rs`
  - Add scoped current/migrated/merged item queries and deterministic cursor predicates.
- Modify: `src/lib/types/sources.ts`
  - Add `TelegramHistoryScope`, source migrated counts/state, item scope markers, and cursor shape.
- Modify: `src/lib/api/sources.ts`
  - Map scoped item request/response fields.
- Modify: `src/lib/api/sources.test.ts`
  - Pin direct API defaults and scoped request mapping.
- Modify: `src/lib/source-reader-model.ts`
  - Carry history scope labels into reader items.
- Modify: `src/lib/source-reader-model.test.ts`
  - Prove migrated labels survive live and snapshot normalization.
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
  - Render the single-source Telegram scope selector and empty/import states.
- Modify: `src/lib/components/analysis/telegram-timeline-reader.svelte`
  - Render migrated row badges.
- Modify: `src/routes/analysis/+page.svelte`
  - Store selected reader history scope, reset pagination on scope changes, and send scoped cursors.
- Modify: `src-tauri/src/notebooklm_export/model.rs`
  - Add export opt-in fields and message scope markers.
- Modify: `src-tauri/src/notebooklm_export/query.rs`
  - Add current/migrated scoped loaders and migrated-domain reply lookup.
- Modify: `src-tauri/src/notebooklm_export/renderer.rs`
  - Render section headings and YAML markers.
- Modify: `src-tauri/src/notebooklm_export/mod.rs`
  - Split opted-in export into current and migrated sections.
- Modify: `src/lib/components/analysis/notebooklm-export-dialog.svelte`
  - Add explicit export opt-in checkbox.
- Modify: `src/lib/analysis-state.ts`
  - Add export form state and report launch state fields for migrated-history opt-in.
- Modify: `src/lib/api/notebooklm-export.ts`
  - Keep wrapper shape unchanged except typed request now includes the opt-in boolean.
- Modify: `src/lib/api/notebooklm-export.test.ts`
  - Pin `include_migrated_history` request mapping.
- Modify: `src-tauri/src/analysis/corpus.rs`
  - Add migrated-history direct corpus loader and metadata markers.
- Modify: `src-tauri/src/analysis/report.rs`
  - Thread `include_migrated_history` through preflight, duplicate detection, run insert, and snapshot capture.
- Modify: `src-tauri/src/analysis/report_commands.rs`
  - Add `include_migrated_history` to the Tauri command.
- Modify: `src-tauri/src/analysis/store.rs`
  - Store and expose `telegram_history_scope`.
- Modify: `src-tauri/src/analysis/models.rs`
  - Add run summary/detail field.
- Modify: `src-tauri/src/analysis/chat.rs`
  - Keep follow-up chat snapshot-backed and include scope labels in context text.
- Modify: `src/lib/types/analysis.ts`
  - Add `telegram_history_scope` and report command opt-in field.
- Modify: `src/lib/api/analysis-runs.ts`
  - Pass the new command field.
- Modify: `src/lib/api/analysis-runs.test.ts`
  - Pin the new field.
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
  - Add explicit report opt-in for eligible Telegram scopes.
- Modify: `src/lib/components/analysis/report-run-header.svelte`
  - Show saved run historical-scope marker.
- Modify: `docs/takeout-source-import.md`
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/specs/2026-05-28-migrated-history-scope-product-behavior-design.md`
  - Move implemented behavior into current-state docs and mark the backlog item complete after verification.

---

### Task 1: Schema And Shared Scope Contract

**Files:**
- Create: `src-tauri/migrations/0003_analysis_telegram_history_scope.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Test: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/sources/store.rs`
- Test: `src/lib/api/sources.test.ts`

- [ ] **Step 1: Add the analysis run migration**

Create `src-tauri/migrations/0003_analysis_telegram_history_scope.sql`:

```sql
ALTER TABLE analysis_runs
ADD COLUMN telegram_history_scope TEXT
CHECK (
    telegram_history_scope IS NULL
    OR telegram_history_scope IN ('current', 'current_plus_migrated')
);
```

- [ ] **Step 2: Register migration 0003**

In `src-tauri/src/migrations.rs`, add the new migration constant near the existing migrated-history migration:

```rust
const ANALYSIS_TELEGRAM_HISTORY_SCOPE_SQL: &str =
    include_str!("../migrations/0003_analysis_telegram_history_scope.sql");
```

Add the migration constructor:

```rust
fn analysis_telegram_history_scope_migration() -> Migration {
    Migration {
        version: 3,
        description: "analysis telegram history scope",
        sql: ANALYSIS_TELEGRAM_HISTORY_SCOPE_SQL,
        kind: MigrationKind::Up,
    }
}
```

Add it after `migrated_history_opt_in_migration()` in the migration list:

```rust
vec![
    current_schema_baseline_migration(),
    migrated_history_opt_in_migration(),
    analysis_telegram_history_scope_migration(),
]
```

Update `build_migrations_starts_at_current_schema_baseline`:

```rust
assert_eq!(versions, vec![1, 2, 3]);
assert_eq!(
    migrations[2].description,
    "analysis telegram history scope"
);
assert!(migrations[2]
    .sql
    .contains("ADD COLUMN telegram_history_scope TEXT"));
```

- [ ] **Step 3: Add migration tests**

In `src-tauri/src/migrations.rs`, add:

```rust
#[tokio::test]
async fn analysis_telegram_history_scope_migration_adds_nullable_checked_column() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    sqlx::raw_sql(BASELINE_SQL)
        .execute(&pool)
        .await
        .expect("apply baseline");
    sqlx::raw_sql(MIGRATED_HISTORY_OPT_IN_SQL)
        .execute(&pool)
        .await
        .expect("apply migrated history migration");
    sqlx::raw_sql(ANALYSIS_TELEGRAM_HISTORY_SCOPE_SQL)
        .execute(&pool)
        .await
        .expect("apply telegram history scope migration");

    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM pragma_table_info('analysis_runs') ORDER BY cid",
    )
    .fetch_all(&pool)
    .await
    .expect("load columns");
    assert!(columns.contains(&"telegram_history_scope".to_string()));

    sqlx::query(
        "INSERT INTO analysis_runs (
            run_type, scope_type, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model,
            status, created_at, telegram_history_scope
         ) VALUES (
            'report', 'single_source', 1, 2, 'Russian', 1,
            'default', 'openai', 'gpt-test', 'queued', 3, 'current_plus_migrated'
         )",
    )
    .execute(&pool)
    .await
    .expect("valid scope");

    let invalid = sqlx::query(
        "INSERT INTO analysis_runs (
            run_type, scope_type, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model,
            status, created_at, telegram_history_scope
         ) VALUES (
            'report', 'single_source', 1, 2, 'Russian', 1,
            'default', 'openai', 'gpt-test', 'queued', 3, 'merged'
         )",
    )
    .execute(&pool)
    .await;
    assert!(invalid.is_err());
}
```

- [ ] **Step 4: Run the migration test and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml analysis_telegram_history_scope_migration_adds_nullable_checked_column
```

Expected: fail until migration registration and the version-list expectation are completed.

- [ ] **Step 5: Add shared backend scope types**

In `src-tauri/src/sources/types.rs`, add:

```rust
pub(crate) const TELEGRAM_HISTORY_SCOPE_CURRENT: &str = "current";
pub(crate) const TELEGRAM_HISTORY_SCOPE_MIGRATED: &str = "migrated";
pub(crate) const TELEGRAM_HISTORY_SCOPE_MERGED: &str = "merged";

pub(crate) const TELEGRAM_HISTORY_SCOPE_LABEL_CURRENT: &str = "Current supergroup history";
pub(crate) const TELEGRAM_HISTORY_SCOPE_LABEL_MIGRATED: &str = "Migrated small-group history";

pub(crate) const ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT: &str = "current";
pub(crate) const ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED: &str =
    "current_plus_migrated";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelegramHistoryScope {
    Current,
    Migrated,
    Merged,
}

impl TelegramHistoryScope {
    pub(crate) fn from_optional(value: Option<Self>) -> Self {
        value.unwrap_or(Self::Current)
    }

    pub(crate) fn as_wire(self) -> &'static str {
        match self {
            Self::Current => TELEGRAM_HISTORY_SCOPE_CURRENT,
            Self::Migrated => TELEGRAM_HISTORY_SCOPE_MIGRATED,
            Self::Merged => TELEGRAM_HISTORY_SCOPE_MERGED,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SourceItemsCursor {
    pub(crate) published_at: i64,
    pub(crate) history_scope_order: i64,
    pub(crate) history_peer_kind: String,
    pub(crate) history_peer_id: i64,
    pub(crate) telegram_message_id: i64,
    pub(crate) item_id: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SourceItemsCursorEnvelope {
    version: u8,
    cursor: SourceItemsCursor,
}

impl SourceItemsCursor {
    pub(crate) fn encode_opaque(&self) -> crate::error::AppResult<String> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        let envelope = SourceItemsCursorEnvelope {
            version: 1,
            cursor: self.clone(),
        };
        let json = serde_json::to_vec(&envelope)
            .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
        Ok(URL_SAFE_NO_PAD.encode(json))
    }

    pub(crate) fn decode_opaque(value: &str) -> crate::error::AppResult<Self> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        let json = URL_SAFE_NO_PAD
            .decode(value)
            .map_err(|_| crate::error::AppError::validation("Invalid source item cursor"))?;
        let envelope: SourceItemsCursorEnvelope = serde_json::from_slice(&json)
            .map_err(|_| crate::error::AppError::validation("Invalid source item cursor"))?;
        if envelope.version != 1 {
            return Err(crate::error::AppError::validation("Unsupported source item cursor"));
        }
        Ok(envelope.cursor)
    }
}
```

Extend `SourceRecord` and `SourceRecordRow`:

```rust
pub migrated_history_row_count: i64,
pub migrated_history_import_completed: bool,
```

- [ ] **Step 6: Expose source migrated-history counts**

In every `src-tauri/src/sources/store.rs` source record SELECT, add these selected fields:

```sql
COALESCE((
    SELECT COUNT(*)
    FROM telegram_messages tm
    WHERE tm.source_id = s.id
      AND tm.is_migrated_history = 1
      AND tm.migration_domain = 'migrated_from_chat'
), 0) AS migrated_history_row_count,
EXISTS (
    SELECT 1
    FROM telegram_takeout_batches tt
    JOIN ingest_batches ib ON ib.id = tt.batch_id
    WHERE ib.source_id = s.id
      AND ib.status = 'completed'
      AND tt.history_scope = 'migrated_small_group_history'
      AND tt.migrated_history_imported = 1
) AS migrated_history_import_completed
```

Map them in the `SourceRecord` builder:

```rust
migrated_history_row_count: row.migrated_history_row_count.max(0),
migrated_history_import_completed: row.migrated_history_import_completed,
```

- [ ] **Step 7: Add source store tests**

In `src-tauri/src/sources/store.rs`, add:

```rust
#[tokio::test]
async fn list_sources_exposes_migrated_history_counts_without_old_chat_identity() {
    let pool = memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_ingest_provenance_tables(&pool).await;

    seed_telegram_source_identity(&pool, 1, 10, "supergroup", "channel", 12345).await;
    crate::takeout_import::migrated_history::upsert_migrated_history_available(
        &pool, 1, 777, 100,
    )
    .await
    .expect("mark capability");

    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, published_at, ingested_at,
            content_kind, has_media
         ) VALUES (10, 1, '42', 'telegram_message', 100, 100, 'text_only', 0)",
    )
    .execute(&pool)
    .await
    .expect("seed item");
    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history
         ) VALUES (10, 1, 'chat', 777, 42, 'migrated_from_chat', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed migrated row");

    let batch_id = crate::ingest_provenance::create_telegram_takeout_batch(
        &pool,
        crate::ingest_provenance::CreateTelegramTakeoutBatch {
            source_id: 1,
            account_id: 10,
            source_subtype: "supergroup".to_string(),
        },
    )
    .await
    .expect("create batch");
    crate::ingest_provenance::mark_takeout_migrated_history_imported(&pool, batch_id)
        .await
        .expect("mark imported");
    crate::ingest_provenance::finalize_ingest_batch(
        &pool,
        batch_id,
        crate::ingest_provenance::TerminalBatchStatus::Completed,
        None,
    )
    .await
    .expect("finalize");

    let row: SourceRecordRow = sqlx::query_as(
        "SELECT s.id, s.source_type, s.source_subtype, s.account_id, s.external_id,
                s.title, s.metadata_zstd,
                s.last_sync_state, s.last_synced_at, s.is_active, s.is_member, s.created_at,
                ts.username AS telegram_username,
                ts.avatar_cache_key AS telegram_avatar_cache_key,
                mhc.status AS migrated_history_status,
                mhc.detected_at AS migrated_history_detected_at,
                mhc.refreshed_at AS migrated_history_refreshed_at,
                COALESCE((
                    SELECT COUNT(*)
                    FROM telegram_messages tm
                    WHERE tm.source_id = s.id
                      AND tm.is_migrated_history = 1
                      AND tm.migration_domain = 'migrated_from_chat'
                ), 0) AS migrated_history_row_count,
                EXISTS (
                    SELECT 1
                    FROM telegram_takeout_batches tt
                    JOIN ingest_batches ib ON ib.id = tt.batch_id
                    WHERE ib.source_id = s.id
                      AND ib.status = 'completed'
                      AND tt.history_scope = 'migrated_small_group_history'
                      AND tt.migrated_history_imported = 1
                ) AS migrated_history_import_completed
         FROM sources s
         LEFT JOIN telegram_sources ts ON ts.source_id = s.id
         LEFT JOIN telegram_migrated_history_capabilities mhc ON mhc.source_id = s.id
         WHERE s.id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load row");

    let source = source_record_from_row_parts(row, None, None);

    assert_eq!(source.migrated_history_row_count, 1);
    assert!(source.migrated_history_import_completed);
    assert!(!format!("{source:?}").contains("777"));
}
```

- [ ] **Step 8: Add TypeScript DTO fields**

In `src/lib/types/sources.ts`, add:

```ts
export type TelegramHistoryScope = "current" | "migrated" | "merged";
export type TelegramItemHistoryScope = "current" | "migrated";
// Backend-opaque cursor. Do not parse, log, snapshot, or render it.
export type SourceItemsCursor = string;
```

Extend `Source`:

```ts
migratedHistoryRowCount: number;
migratedHistoryImportCompleted: boolean;
```

Extend `SourceItem`:

```ts
historyScope: TelegramItemHistoryScope;
isMigratedHistory: boolean;
migrationDomain: "migrated_from_chat" | null;
historyScopeLabel: "Current supergroup history" | "Migrated small-group history";
pageCursor: SourceItemsCursor;
```

Extend `ListSourceItemsInput`:

```ts
historyScope?: TelegramHistoryScope;
beforeCursor?: SourceItemsCursor | null;
```

- [ ] **Step 9: Map TypeScript source fields**

In `src/lib/api/sources.ts`, extend raw types:

```ts
migrated_history_row_count?: number | null;
migrated_history_import_completed?: boolean | null;
```

```ts
history_scope: "current" | "migrated";
is_migrated_history: boolean;
migration_domain: "migrated_from_chat" | null;
history_scope_label: "Current supergroup history" | "Migrated small-group history";
page_cursor: string;
```

Map source fields:

```ts
migratedHistoryRowCount: source.migrated_history_row_count ?? 0,
migratedHistoryImportCompleted: source.migrated_history_import_completed ?? false,
```

Map request cursor:

```ts
historyScope: input.historyScope ?? "current",
beforeCursor: input.beforeCursor
  ? input.beforeCursor
  : null,
```

Map item cursor:

```ts
historyScope: item.history_scope,
isMigratedHistory: item.is_migrated_history,
migrationDomain: item.migration_domain,
historyScopeLabel: item.history_scope_label,
pageCursor: item.page_cursor,
```

- [ ] **Step 10: Update API tests**

In `src/lib/api/sources.test.ts`, update the existing `listSourceItems` fixture with current-history fields and add:

```ts
it("passes explicit Telegram history scope and opaque cursor to source item loading", async () => {
  invokeMock.mockResolvedValueOnce([]);

  await expect(
    listSourceItems({
      sourceId: 7,
      limit: 50,
      beforePublishedAt: null,
      beforeCursor: "eyJ2ZXJzaW9uIjoxLCJjdXJzb3IiOnt9fQ",
      topicFilter: null,
      historyScope: "merged",
    }),
  ).resolves.toEqual([]);

  expect(invokeMock).toHaveBeenLastCalledWith("list_source_items", {
    request: {
      sourceId: 7,
      limit: 50,
      beforePublishedAt: null,
      beforeCursor: "eyJ2ZXJzaW9uIjoxLCJjdXJzb3IiOnt9fQ",
      topicFilter: null,
      historyScope: "merged",
    },
  });
});
```

- [ ] **Step 11: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml analysis_telegram_history_scope_migration_adds_nullable_checked_column
cargo test --manifest-path src-tauri\Cargo.toml list_sources_exposes_migrated_history_counts_without_old_chat_identity
npm.cmd test -- src/lib/api/sources.test.ts
```

Expected: all pass.

- [ ] **Step 12: Commit Task 1**

Run:

```powershell
git add src-tauri\migrations\0003_analysis_telegram_history_scope.sql src-tauri\src\migrations.rs src-tauri\src\sources\types.rs src-tauri\src\sources\store.rs src\lib\types\sources.ts src\lib\api\sources.ts src\lib\api\sources.test.ts docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "feat: define migrated history scope contract"
```

Expected: commit succeeds.

---

### Task 2: Browsing Backend Scoped Reader

**Files:**
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Test: `src-tauri/src/sources/items/query.rs`

- [ ] **Step 1: Extend item request and response DTOs**

In `src-tauri/src/sources/items.rs`, add the fields to `ItemRecord`:

```rust
pub history_scope: String,
pub is_migrated_history: bool,
pub migration_domain: Option<String>,
pub history_scope_label: String,
pub page_cursor: String,
```

Update imports:

```rust
use super::types::{
    now_secs, SourceItemsCursor, StoredItemRow, TELEGRAM_HISTORY_SCOPE_CURRENT,
    TELEGRAM_HISTORY_SCOPE_LABEL_CURRENT, TELEGRAM_SOURCE_TYPE, TelegramHistoryScope,
    TelegramMessageIdentity,
    ITEM_KIND_TELEGRAM_MESSAGE, ITEM_KIND_YOUTUBE_COMMENT, ITEM_KIND_YOUTUBE_TRANSCRIPT,
};
```

Extend `ListSourceItemsRequest`:

```rust
pub history_scope: Option<TelegramHistoryScope>,
pub before_cursor: Option<String>,
```

- [ ] **Step 2: Add scoped row type**

In `src-tauri/src/sources/items/query.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub(super) struct BrowsableItemRow {
    pub(super) id: i64,
    pub(super) source_id: i64,
    pub(super) external_id: String,
    pub(super) item_kind: String,
    pub(super) author: Option<String>,
    pub(super) published_at: i64,
    pub(super) content_kind: String,
    pub(super) has_media: bool,
    pub(super) media_kind: Option<String>,
    pub(super) content_zstd: Option<Vec<u8>>,
    pub(super) media_metadata_zstd: Option<Vec<u8>>,
    pub(super) has_raw_data: bool,
    pub(super) forum_topic_id: Option<i64>,
    pub(super) forum_topic_title: Option<String>,
    pub(super) forum_topic_top_message_id: Option<i64>,
    pub(super) reply_to_msg_id: Option<i64>,
    pub(super) reply_to_peer_kind: Option<String>,
    pub(super) reply_to_peer_id: Option<String>,
    pub(super) reply_to_top_id: Option<i64>,
    pub(super) reaction_count: Option<i64>,
    pub(super) history_scope: String,
    pub(super) is_migrated_history: bool,
    pub(super) migration_domain: Option<String>,
    pub(super) history_scope_label: String,
    pub(super) history_scope_order: i64,
    pub(super) history_peer_kind: String,
    pub(super) history_peer_id: i64,
    pub(super) telegram_message_id: i64,
}
```

- [ ] **Step 3: Add backend tests first**

In `src-tauri/src/sources/items/query.rs`, add:

```rust
#[tokio::test]
async fn scoped_browsing_defaults_to_current_rows() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
    seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
    seed_telegram_identity(&pool, 10, "channel", 12345, 10, None, false).await;
    seed_telegram_identity(
        &pool,
        11,
        "chat",
        777,
        10,
        Some("migrated_from_chat"),
        true,
    )
    .await;

    let rows = load_item_rows_from_pool(
        &pool,
        1,
        "telegram",
        20,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("load rows");

    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10]);
    assert_eq!(rows[0].history_scope, "current");
}

#[tokio::test]
async fn scoped_browsing_can_load_only_migrated_rows_with_labels() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
    seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
    seed_telegram_identity(&pool, 10, "channel", 12345, 10, None, false).await;
    seed_telegram_identity(
        &pool,
        11,
        "chat",
        777,
        10,
        Some("migrated_from_chat"),
        true,
    )
    .await;

    let rows = load_item_rows_from_pool(
        &pool,
        1,
        "telegram",
        20,
        None,
        None,
        None,
        Some(TelegramHistoryScope::Migrated),
        None,
    )
    .await
    .expect("load rows");

    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![11]);
    assert_eq!(rows[0].history_scope, "migrated");
    assert_eq!(rows[0].history_scope_label, "Migrated small-group history");
    assert_eq!(rows[0].migration_domain.as_deref(), Some("migrated_from_chat"));
}

#[tokio::test]
async fn merged_browsing_uses_full_cursor_tuple_for_equal_timestamps() {
    let pool = memory_pool_with_source_items_and_topics().await;
    for (item_id, external_id, content) in [
        (10_i64, "40", "current low"),
        (11_i64, "41", "current high"),
        (12_i64, "40", "migrated old"),
    ] {
        seed_direct_item(&pool, 1, item_id, external_id, 1000, content).await;
    }
    seed_telegram_identity(&pool, 10, "channel", 12345, 40, None, false).await;
    seed_telegram_identity(&pool, 11, "channel", 12345, 41, None, false).await;
    seed_telegram_identity(
        &pool,
        12,
        "chat",
        777,
        40,
        Some("migrated_from_chat"),
        true,
    )
    .await;

    let first_page = load_item_rows_from_pool(
        &pool,
        1,
        "telegram",
        2,
        None,
        None,
        None,
        Some(TelegramHistoryScope::Merged),
        None,
    )
    .await
    .expect("first page");

    assert_eq!(first_page.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10, 11]);

    let cursor = first_page[1].cursor();
    let encoded_cursor = cursor.encode_opaque().expect("encode opaque cursor");
    assert!(!encoded_cursor.contains("12345"));
    assert_eq!(
        SourceItemsCursor::decode_opaque(&encoded_cursor).expect("decode opaque cursor"),
        cursor
    );
    let second_page = load_item_rows_from_pool(
        &pool,
        1,
        "telegram",
        2,
        None,
        None,
        None,
        Some(TelegramHistoryScope::Merged),
        Some(cursor),
    )
    .await
    .expect("second page");

    assert_eq!(second_page.iter().map(|row| row.id).collect::<Vec<_>>(), vec![12]);
}

#[tokio::test]
async fn topic_filters_are_rejected_for_non_current_history_scope() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;

    let error = load_item_rows_from_pool(
        &pool,
        1,
        "telegram",
        20,
        None,
        Some(ForumTopicFilter::Topic { topic_id: 200 }),
        None,
        Some(TelegramHistoryScope::Merged),
        None,
    )
    .await
    .expect_err("reject topic filter");

    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
}
```

Add helper functions used by the tests:

```rust
async fn seed_telegram_identity(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    history_peer_kind: &str,
    history_peer_id: i64,
    telegram_message_id: i64,
    migration_domain: Option<&str>,
    is_migrated_history: bool,
) {
    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history
         ) VALUES (?, 1, ?, ?, ?, ?, ?)",
    )
    .bind(item_id)
    .bind(history_peer_kind)
    .bind(history_peer_id)
    .bind(telegram_message_id)
    .bind(migration_domain)
    .bind(i64::from(is_migrated_history))
    .execute(pool)
    .await
    .expect("seed telegram identity");
}
```

- [ ] **Step 4: Run tests and verify failures**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml scoped_browsing_defaults_to_current_rows
cargo test --manifest-path src-tauri\Cargo.toml scoped_browsing_can_load_only_migrated_rows_with_labels
cargo test --manifest-path src-tauri\Cargo.toml merged_browsing_uses_full_cursor_tuple_for_equal_timestamps
cargo test --manifest-path src-tauri\Cargo.toml topic_filters_are_rejected_for_non_current_history_scope
```

Expected: fail until the scoped query API exists.

- [ ] **Step 5: Implement deterministic cursor helpers**

In `src-tauri/src/sources/items/query.rs`, add:

```rust
impl BrowsableItemRow {
    pub(super) fn cursor(&self) -> SourceItemsCursor {
        SourceItemsCursor {
            published_at: self.published_at,
            history_scope_order: self.history_scope_order,
            history_peer_kind: self.history_peer_kind.clone(),
            history_peer_id: self.history_peer_id,
            telegram_message_id: self.telegram_message_id,
            item_id: self.id,
        }
    }
}

fn push_after_cursor_predicate(
    sql: &mut String,
    cursor: &SourceItemsCursor,
    inclusive: bool,
) {
    let item_operator = if inclusive { ">=" } else { ">" };
    sql.push_str(&format!(
        " AND (
            published_at < ?
            OR (
                published_at = ?
                AND (
                    history_scope_order > ?
                    OR (history_scope_order = ? AND history_peer_kind > ?)
                    OR (history_scope_order = ? AND history_peer_kind = ? AND history_peer_id > ?)
                    OR (history_scope_order = ? AND history_peer_kind = ? AND history_peer_id = ? AND telegram_message_id > ?)
                    OR (history_scope_order = ? AND history_peer_kind = ? AND history_peer_id = ? AND telegram_message_id = ? AND id {item_operator} ?)
                )
            )
        )"
    ));
}
```

When binding a cursor, bind values in this order:

```rust
query = query
    .bind(cursor.published_at)
    .bind(cursor.published_at)
    .bind(cursor.history_scope_order)
    .bind(cursor.history_scope_order)
    .bind(&cursor.history_peer_kind)
    .bind(cursor.history_scope_order)
    .bind(&cursor.history_peer_kind)
    .bind(cursor.history_peer_id)
    .bind(cursor.history_scope_order)
    .bind(&cursor.history_peer_kind)
    .bind(cursor.history_peer_id)
    .bind(cursor.telegram_message_id)
    .bind(cursor.history_scope_order)
    .bind(&cursor.history_peer_kind)
    .bind(cursor.history_peer_id)
    .bind(cursor.telegram_message_id)
    .bind(cursor.item_id);
```

The opaque cursor is encoded only at the command DTO boundary:

```rust
let encoded = row.cursor().encode_opaque()?;
let decoded = SourceItemsCursor::decode_opaque(&encoded)?;
assert_eq!(decoded, row.cursor());
```

Never expose `history_peer_id` or the decoded tuple to TypeScript types, UI state, logs, or snapshots.

- [ ] **Step 6: Implement scoped direct query**

Replace the existing direct items-path SQL builder with a scoped builder. Use a CTE so cursor predicates can refer to the computed ordering fields:

```sql
WITH scoped_items AS (
  SELECT
    items.id,
    items.source_id,
    items.external_id,
    items.item_kind,
    items.author,
    items.published_at,
    items.content_kind,
    items.has_media,
    items.media_kind,
    items.content_zstd,
    items.media_metadata_zstd,
    items.raw_data_zstd IS NOT NULL AS has_raw_data,
    items.reply_to_msg_id,
    items.reply_to_peer_kind,
    items.reply_to_peer_id,
    items.reply_to_top_id,
    items.reaction_count,
    forum_topics.topic_id AS forum_topic_id,
    forum_topics.title AS forum_topic_title,
    forum_topics.top_message_id AS forum_topic_top_message_id,
    CASE WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 'migrated' ELSE 'current' END AS history_scope,
    COALESCE(tm.is_migrated_history, 0) AS is_migrated_history,
    tm.migration_domain AS migration_domain,
    CASE
      WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 'Migrated small-group history'
      ELSE 'Current supergroup history'
    END AS history_scope_label,
    CASE WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 1 ELSE 0 END AS history_scope_order,
    COALESCE(tm.history_peer_kind, '') AS history_peer_kind,
    COALESCE(tm.history_peer_id, 0) AS history_peer_id,
    COALESCE(tm.telegram_message_id, 0) AS telegram_message_id
  FROM items
  LEFT JOIN telegram_messages tm ON tm.item_id = items.id
  LEFT JOIN item_topic_memberships AS memberships
    ON memberships.item_id = items.id
  LEFT JOIN telegram_forum_topics AS forum_topics
    ON forum_topics.source_id = memberships.source_id
   AND forum_topics.topic_id = memberships.topic_id
)
SELECT *
FROM scoped_items
WHERE source_id = ?
```

The CTE fields are the source of truth for marker values:

```sql
CASE WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 'migrated' ELSE 'current' END AS history_scope,
COALESCE(tm.is_migrated_history, 0) AS is_migrated_history,
tm.migration_domain AS migration_domain,
CASE
  WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 'Migrated small-group history'
  ELSE 'Current supergroup history'
END AS history_scope_label,
CASE WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 1 ELSE 0 END AS history_scope_order,
COALESCE(tm.history_peer_kind, '') AS history_peer_kind,
COALESCE(tm.history_peer_id, 0) AS history_peer_id,
COALESCE(tm.telegram_message_id, 0) AS telegram_message_id
```

Use this scope filter:

```rust
match scope {
    TelegramHistoryScope::Current => {
        sql.push_str(" AND is_migrated_history = 0");
    }
    TelegramHistoryScope::Migrated => {
        sql.push_str(
            " AND is_migrated_history = 1
              AND migration_domain = 'migrated_from_chat'",
        );
    }
    TelegramHistoryScope::Merged => {}
}
```

Use this ordering:

```sql
ORDER BY
  published_at DESC,
  history_scope_order ASC,
  history_peer_kind ASC,
  history_peer_id ASC,
  telegram_message_id ASC,
  id ASC
LIMIT ?
```

For `around_item_id`, resolve the selected row through the same CTE and scope filter into a `SourceItemsCursor`, then call `push_after_cursor_predicate(&mut sql, &cursor, true)`. For normal paging, call `push_after_cursor_predicate(&mut sql, &cursor, false)`.

- [ ] **Step 7: Use direct scoped query for Telegram browsing**

Change `load_item_rows_from_pool` so callers pass the source type:

```rust
pub(super) async fn load_item_rows_from_pool(
    pool: &SqlitePool,
    source_id: i64,
    source_type: &str,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
    history_scope: Option<TelegramHistoryScope>,
    before_cursor: Option<SourceItemsCursor>,
) -> AppResult<Vec<BrowsableItemRow>>
```

Use the direct scoped query whenever `source_type == TELEGRAM_SOURCE_TYPE`:

```rust
let scope = TelegramHistoryScope::from_optional(history_scope);

if source_type == TELEGRAM_SOURCE_TYPE {
    return load_scoped_telegram_item_rows(
        pool,
        source_id,
        limit,
        topic_filter,
        around_item_id,
        scope,
        before_cursor,
    )
    .await;
}
```

Keep archive only for non-Telegram, current-history requests that still use the old published-at cursor:

```rust
if source_type != TELEGRAM_SOURCE_TYPE
    && scope == TelegramHistoryScope::Current
    && before_cursor.is_none()
    && crate::archive_read_model::source_archive_model_is_ready(pool, source_id).await?
{
    return crate::archive_read_model::load_item_rows_from_archive(
        pool,
        source_id,
        limit,
        before_published_at,
        topic_filter,
        around_item_id,
    )
    .await
    .map(|rows| rows.into_iter().map(non_telegram_item_row_from_archive).collect());
}
```

Add the non-Telegram archive conversion with synthetic non-private cursor fields:

```rust
fn non_telegram_item_row_from_archive(row: StoredItemRow) -> BrowsableItemRow {
    BrowsableItemRow {
        id: row.id,
        source_id: row.source_id,
        external_id: row.external_id,
        item_kind: row.item_kind,
        author: row.author,
        published_at: row.published_at,
        content_kind: row.content_kind,
        has_media: row.has_media,
        media_kind: row.media_kind,
        content_zstd: row.content_zstd,
        media_metadata_zstd: row.media_metadata_zstd,
        has_raw_data: row.has_raw_data,
        forum_topic_id: row.forum_topic_id,
        forum_topic_title: row.forum_topic_title,
        forum_topic_top_message_id: row.forum_topic_top_message_id,
        reply_to_msg_id: row.reply_to_msg_id,
        reply_to_peer_kind: row.reply_to_peer_kind,
        reply_to_peer_id: row.reply_to_peer_id,
        reply_to_top_id: row.reply_to_top_id,
        reaction_count: row.reaction_count,
        history_scope: TELEGRAM_HISTORY_SCOPE_CURRENT.to_string(),
        is_migrated_history: false,
        migration_domain: None,
        history_scope_label: TELEGRAM_HISTORY_SCOPE_LABEL_CURRENT.to_string(),
        history_scope_order: 0,
        history_peer_kind: String::new(),
        history_peer_id: 0,
        telegram_message_id: row.id,
    }
}
```

Do not convert archive rows into Telegram `BrowsableItemRow` values. Archive rows do not contain the real `history_peer_kind`, `history_peer_id`, and `telegram_message_id`, so mixing an archive first page with a direct `before_cursor` page would make equal-timestamp paging unstable.

- [ ] **Step 8: Map scoped row to item record**

In `src-tauri/src/sources/items.rs`, change `item_record_from_row` to accept `BrowsableItemRow` and include:

```rust
history_scope: row.history_scope,
is_migrated_history: row.is_migrated_history,
migration_domain: row.migration_domain,
history_scope_label: row.history_scope_label,
page_cursor: row.cursor().encode_opaque()?,
```

- [ ] **Step 9: Validate non-current scope requests**

In `list_source_items`, load the source type before dispatch:

```rust
let source_type: String = sqlx::query_scalar("SELECT source_type FROM sources WHERE id = ?")
    .bind(request.source_id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("Source not found"))?;
let is_telegram_source = source_type == TELEGRAM_SOURCE_TYPE;
```

Reject non-current topic filters for Telegram historical scopes:

```rust
let history_scope = TelegramHistoryScope::from_optional(request.history_scope);
if !is_telegram_source && history_scope != TelegramHistoryScope::Current {
    return Err(AppError::validation(
        "Telegram history scope applies only to Telegram source browsing",
    ));
}
if is_telegram_source
    && history_scope != TelegramHistoryScope::Current
    && request.topic_filter.is_some()
{
    return Err(AppError::validation(
        "Telegram forum topic filters apply only to current supergroup history",
    ));
}
```

Decode opaque cursors only for Telegram source browsing:

```rust
let before_cursor = match (is_telegram_source, request.before_cursor.as_deref()) {
    (true, Some(cursor)) => Some(SourceItemsCursor::decode_opaque(cursor)?),
    (true, None) => None,
    (false, Some(_)) => {
        return Err(AppError::validation(
            "Opaque source item cursors are only supported for Telegram source browsing",
        ));
    }
    (false, None) => None,
};
let before_published_at = if is_telegram_source {
    None
} else {
    request.before_published_at
};
```

Then call:

```rust
let rows = load_item_rows_from_pool(
    &pool,
    request.source_id,
    &source_type,
    limit,
    before_published_at,
    request.topic_filter,
    request.around_item_id,
    Some(history_scope),
    before_cursor,
)
.await?;
```

- [ ] **Step 10: Run focused browsing backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml scoped_browsing_defaults_to_current_rows
cargo test --manifest-path src-tauri\Cargo.toml scoped_browsing_can_load_only_migrated_rows_with_labels
cargo test --manifest-path src-tauri\Cargo.toml merged_browsing_uses_full_cursor_tuple_for_equal_timestamps
cargo test --manifest-path src-tauri\Cargo.toml topic_filters_are_rejected_for_non_current_history_scope
cargo test --manifest-path src-tauri\Cargo.toml default_source_browsing_does_not_surface_migrated_rows_after_archive_ready
```

Expected: all pass.

- [ ] **Step 11: Commit Task 2**

Run:

```powershell
git add src-tauri\src\sources\items.rs src-tauri\src\sources\items\query.rs docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "feat: add scoped Telegram source browsing"
```

Expected: commit succeeds.

---

### Task 3: Browsing Frontend Scope Control

**Files:**
- Modify: `src/lib/source-reader-model.ts`
- Modify: `src/lib/source-reader-model.test.ts`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/telegram-timeline-reader.svelte`
- Modify: `src/routes/analysis/+page.svelte`
- Test: `src/lib/analysis-source-readers.test.ts`
- Test: `src/lib/analysis-source-readers-route.test.ts`

- [ ] **Step 1: Add reader item history labels**

In `src/lib/source-reader-model.ts`, extend `SourceReaderItem`:

```ts
historyScope: "current" | "migrated";
historyScopeLabel: string | null;
isMigratedHistory: boolean;
```

In `sourceItemToReaderItem`, set:

```ts
historyScope: item.historyScope,
historyScopeLabel: item.isMigratedHistory ? item.historyScopeLabel : null,
isMigratedHistory: item.isMigratedHistory,
```

In `analysisRunMessageToReaderItem`, read snapshot metadata:

```ts
const historyScope = stringValue(metadata.history_scope) === "migrated" ? "migrated" : "current";
```

and set:

```ts
historyScope,
historyScopeLabel:
  historyScope === "migrated" ? "Migrated small-group history" : null,
isMigratedHistory: historyScope === "migrated",
```

- [ ] **Step 2: Add model tests**

In `src/lib/source-reader-model.test.ts`, extend `sourceItem()` defaults:

```ts
historyScope: "current",
isMigratedHistory: false,
migrationDomain: null,
historyScopeLabel: "Current supergroup history",
pageCursor: "eyJ2ZXJzaW9uIjoxLCJjdXJzb3IiOnt9fQ",
```

Add:

```ts
it("marks migrated live source rows with backend-owned history labels", () => {
  const readerItem = sourceItemToReaderItem(
    sourceItem({
      historyScope: "migrated",
      isMigratedHistory: true,
      migrationDomain: "migrated_from_chat",
      historyScopeLabel: "Migrated small-group history",
    }),
    { sourceTitle: "Telegram A" },
  );

  expect(readerItem.historyScope).toBe("migrated");
  expect(readerItem.historyScopeLabel).toBe("Migrated small-group history");
  expect(readerItem.isMigratedHistory).toBe(true);
});

it("marks migrated run snapshot rows from metadata", () => {
  const readerItem = analysisRunMessageToReaderItem(
    runMessage({
      item_kind: "telegram_message",
      source_type: "telegram",
      source_subtype: "supergroup",
      metadata_json: {
        history_scope: "migrated",
        migration_domain: "migrated_from_chat",
      },
    }),
    { sourceTitle: "Telegram A" },
  );

  expect(readerItem.historyScope).toBe("migrated");
  expect(readerItem.historyScopeLabel).toBe("Migrated small-group history");
});
```

- [ ] **Step 3: Render timeline badges**

In `src/lib/components/analysis/telegram-timeline-reader.svelte`, inside the message metadata/header area, add:

```svelte
{#if item.historyScopeLabel}
  <span class="history-scope-badge">{item.historyScopeLabel}</span>
{/if}
```

Add CSS:

```css
.history-scope-badge {
  align-self: flex-start;
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  border-radius: 999px;
  color: var(--muted);
  font-size: 0.6875rem;
  line-height: 1;
  padding: 0.1875rem 0.375rem;
}
```

- [ ] **Step 4: Add source-reader scope props**

In `src/lib/components/analysis/report-source-surface.svelte`, add props:

```ts
telegramHistoryScope: TelegramHistoryScope;
onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
```

Import the type:

```ts
import type { TelegramHistoryScope } from "$lib/types/sources";
```

Add scope options:

```ts
const telegramHistoryScopeOptions = $derived.by(() => {
  if (!currentSource || currentSource.sourceType !== "telegram") return [];
  if (currentSource.migratedHistoryRowCount <= 0) return [];
  return [
    { value: "current" as const, label: "Current supergroup history" },
    { value: "migrated" as const, label: "Migrated small-group history" },
    { value: "merged" as const, label: "Merged timeline" },
  ];
});
```

Render before the single-source Telegram timeline:

```svelte
{#if currentSource.sourceType === "telegram" && telegramHistoryScopeOptions.length > 0}
  <div class="history-scope-control" role="group" aria-label="Telegram history scope">
    {#each telegramHistoryScopeOptions as option (option.value)}
      <button
        type="button"
        class:active={telegramHistoryScope === option.value}
        aria-pressed={telegramHistoryScope === option.value}
        onclick={() => onChangeTelegramHistoryScope(option.value)}
      >
        {option.label}
      </button>
    {/each}
  </div>
{/if}

{#if currentSource.sourceType === "telegram"
  && currentSource.migratedHistoryImportCompleted
  && currentSource.migratedHistoryRowCount === 0
  && telegramHistoryScope !== "current"}
  <StatusMessage tone="muted">Migrated small-group history imported; no messages were found.</StatusMessage>
{:else}
  <TelegramTimelineReader
    items={liveReaderItems}
    loading={loadingItems}
    hasMore={sourceItemsHasMore}
    contentLabel={currentSourceContentLabel}
    {formatTimestamp}
    onLoadMore={onLoadMoreSourceItems}
  />
{/if}
```

Add CSS:

```css
.history-scope-control {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.history-scope-control button {
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--surface);
  color: var(--text);
  cursor: pointer;
  font: inherit;
  padding: 0.4rem 0.55rem;
}

.history-scope-control button.active {
  border-color: var(--accent);
  color: var(--accent);
}
```

- [ ] **Step 5: Thread props through `ReportCanvas`**

In `src/lib/components/analysis/report-canvas.svelte`, add props and pass them into `ReportSourceSurface`:

```svelte
{telegramHistoryScope}
{onChangeTelegramHistoryScope}
```

- [ ] **Step 6: Update page state and pagination**

In `src/routes/analysis/+page.svelte`, add state:

```ts
let telegramHistoryScope = $state<TelegramHistoryScope>("current");
let sourceItemsCursor = $state<SourceItemsCursor | null>(null);
let sourceItemsBeforePublishedAt = $state<number | null>(null);
```

Replace the previous numeric cursor assignment with separate Telegram and legacy cursors:

```ts
const previousCursor = sourceItemsCursor;
const previousBeforePublishedAt = sourceItemsBeforePublishedAt;
const lastItem = items.at(-1);
sourceItemsCursor = lastItem?.pageCursor ?? (append ? previousCursor : null);
sourceItemsBeforePublishedAt =
  lastItem?.publishedAt ?? (append ? previousBeforePublishedAt : null);
```

When loading source items, send:

```ts
const isTelegramSource = source.sourceType === "telegram";
historyScope: source.sourceType === "telegram" ? telegramHistoryScope : "current",
beforeCursor: null,
beforePublishedAt: null,
```

When loading more:

```ts
const isTelegramSource = source.sourceType === "telegram";
const canPage = isTelegramSource
  ? sourceItemsCursor !== null
  : sourceItemsBeforePublishedAt !== null;
if (!canPage) return;
historyScope: isTelegramSource ? telegramHistoryScope : "current",
beforeCursor: isTelegramSource ? sourceItemsCursor : null,
beforePublishedAt: isTelegramSource ? null : sourceItemsBeforePublishedAt,
```

Add a scope-change handler:

```ts
function changeTelegramHistoryScope(scope: TelegramHistoryScope) {
  if (telegramHistoryScope === scope) return;
  telegramHistoryScope = scope;
  selectedTopicKey = ALL_TOPICS_KEY;
  resetSourceItemsReader();
  const source = currentSource();
  if (source?.sourceType === "telegram") {
    void loadItems(source.id);
  }
}
```

Reset to current when selecting a different source:

```ts
telegramHistoryScope = "current";
sourceItemsCursor = null;
sourceItemsBeforePublishedAt = null;
```

- [ ] **Step 7: Update frontend contract tests**

In `src/lib/analysis-source-readers.test.ts`, add:

```ts
it("renders Telegram history scope controls and migrated row badges", () => {
  expect(reportSourceSurfaceSource).toContain("telegramHistoryScopeOptions");
  expect(reportSourceSurfaceSource).toContain("Current supergroup history");
  expect(reportSourceSurfaceSource).toContain("Migrated small-group history");
  expect(reportSourceSurfaceSource).toContain("Merged timeline");
  expect(telegramTimelineSource).toContain("history-scope-badge");
  expect(telegramTimelineSource).toContain("item.historyScopeLabel");
});
```

In `src/lib/analysis-source-readers-route.test.ts`, add:

```ts
it("passes Telegram history scope and cursor through live source item loading", () => {
  expect(analysisPageSource).toContain("telegramHistoryScope");
  expect(analysisPageSource).toContain("sourceItemsBeforePublishedAt");
  expect(analysisPageSource).toContain("const canPage = isTelegramSource");
  expect(analysisPageSource).toContain("beforeCursor: isTelegramSource ? sourceItemsCursor : null");
  expect(analysisPageSource).toContain("beforePublishedAt: isTelegramSource ? null : sourceItemsBeforePublishedAt");
  expect(analysisPageSource).toContain("historyScope: isTelegramSource ? telegramHistoryScope : \"current\"");
  expect(analysisPageSource).toContain("function changeTelegramHistoryScope");
});
```

- [ ] **Step 8: Run frontend browsing tests**

Run:

```powershell
npm.cmd test -- src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/api/sources.test.ts
npm.cmd run check
```

Expected: all pass.

- [ ] **Step 9: Commit Task 3**

Run:

```powershell
git add src\lib\source-reader-model.ts src\lib\source-reader-model.test.ts src\lib\components\analysis\report-source-surface.svelte src\lib\components\analysis\report-canvas.svelte src\lib\components\analysis\telegram-timeline-reader.svelte src\routes\analysis\+page.svelte src\lib\analysis-source-readers.test.ts src\lib\analysis-source-readers-route.test.ts docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "feat: add migrated history reader controls"
```

Expected: commit succeeds.

---

### Task 4: NotebookLM Export Opt-In

**Files:**
- Modify: `src-tauri/src/notebooklm_export/model.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Modify: `src-tauri/src/notebooklm_export/renderer.rs`
- Modify: `src-tauri/src/notebooklm_export/mod.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/components/analysis/notebooklm-export-dialog.svelte`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/api/notebooklm-export.test.ts`
- Modify: `src/routes/analysis/+page.svelte`
- Test: `src-tauri/src/notebooklm_export/query.rs`
- Test: `src-tauri/src/notebooklm_export/renderer.rs`
- Test: `src-tauri/src/notebooklm_export/mod.rs`

- [ ] **Step 1: Add export request/config fields**

In `src-tauri/src/notebooklm_export/model.rs`, add to `NotebookLmExportRequest` and `NotebookLmExportConfig`:

```rust
pub include_migrated_history: bool,
```

Extend `NotebookLmExportMessage`:

```rust
pub(crate) history_scope: String,
pub(crate) migration_domain: Option<String>,
```

- [ ] **Step 2: Add export query tests**

In `src-tauri/src/notebooklm_export/query.rs`, add:

```rust
#[tokio::test]
async fn opted_in_export_loads_migrated_rows_separately_with_markers() {
    let pool = export_pool().await;
    seed_notebooklm_export_parity_fixture(&pool).await;
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, ingested_at,
            content_zstd, raw_data_zstd, content_kind, has_media
         ) VALUES (30, 1, '42', 'telegram_message', 'Old', 130, 130, ?, NULL, 'text_only', 0)",
    )
    .bind(crate::compression::compress_text("old history").expect("compress"))
    .execute(&pool)
    .await
    .expect("seed migrated item");
    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history
         ) VALUES (30, 1, 'chat', 777, 42, 'migrated_from_chat', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed migrated identity");

    let current = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
        .await
        .expect("current messages");
    let migrated = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Migrated)
        .await
        .expect("migrated messages");

    assert!(current.iter().all(|message| message.history_scope == "current_supergroup_history"));
    assert_eq!(migrated.iter().map(|message| message.item_id).collect::<Vec<_>>(), vec![30]);
    assert_eq!(migrated[0].history_scope, "migrated_small_group_history");
    assert_eq!(migrated[0].migration_domain.as_deref(), Some("migrated_from_chat"));
}
```

Add reply-domain test:

```rust
#[tokio::test]
async fn migrated_export_reply_lookup_stays_inside_old_history_domain() {
    let pool = export_pool().await;
    seed_export_source(&pool).await;
    for (id, external_id, text, reply_to) in [
        (20_i64, "7", "current seven", None),
        (30_i64, "7", "old seven", None),
        (31_i64, "8", "old reply", Some(7_i64)),
    ] {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media, reply_to_msg_id
             ) VALUES (?, 1, ?, 'telegram_message', 'A', ?, ?, ?, NULL, 'text_only', 0, ?)",
        )
        .bind(id)
        .bind(external_id)
        .bind(id)
        .bind(id)
        .bind(crate::compression::compress_text(text).expect("compress"))
        .bind(reply_to)
        .execute(&pool)
        .await
        .expect("seed item");
    }
    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history
         ) VALUES
            (20, 1, 'channel', 12345, 7, NULL, 0),
            (30, 1, 'chat', 777, 7, 'migrated_from_chat', 1),
            (31, 1, 'chat', 777, 8, 'migrated_from_chat', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed identities");

    let migrated = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Migrated)
        .await
        .expect("migrated messages");
    let reply = migrated.iter().find(|message| message.item_id == 31).expect("reply");

    assert_eq!(reply.reply_to_snippet.as_deref(), Some("old seven"));
}
```

- [ ] **Step 3: Implement export scope enum and loaders**

In `src-tauri/src/notebooklm_export/query.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportHistoryScope {
    Current,
    Migrated,
}
```

Change `load_export_messages` signature:

```rust
pub(crate) async fn load_export_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
    scope: ExportHistoryScope,
) -> AppResult<Vec<NotebookLmExportMessage>>
```

For `Current`, keep existing loader selection. For `Migrated`, always use the items path with:

```sql
AND tm.is_migrated_history = 1
AND tm.migration_domain = 'migrated_from_chat'
```

For current items-path rows, select:

```sql
'current_supergroup_history' AS history_scope,
NULL AS migration_domain
```

For migrated rows, select:

```sql
'migrated_small_group_history' AS history_scope,
tm.migration_domain AS migration_domain
```

Extend `ExportMessageRow` and `map_export_rows` to carry those fields into `NotebookLmExportMessage`.

- [ ] **Step 4: Make migrated reply lookup domain-aware**

For items-path scoped reply lookup, join `telegram_messages` and match replies by original history domain:

```sql
SELECT
  target_items.external_id,
  target_items.author,
  target_items.content_zstd,
  target_items.has_media,
  target_items.media_kind
FROM telegram_messages reply_tm
JOIN telegram_messages target_tm
  ON target_tm.source_id = reply_tm.source_id
 AND target_tm.history_peer_kind = reply_tm.history_peer_kind
 AND target_tm.history_peer_id = reply_tm.history_peer_id
 AND target_tm.telegram_message_id = reply_tm.reply_to_msg_id
JOIN items target_items
  ON target_items.id = target_tm.item_id
WHERE reply_tm.item_id = ?
```

Use one query per reply row in the first implementation. Keep archive reply lookup unchanged because archive is current-only.

- [ ] **Step 5: Render YAML markers and section headings**

In `src-tauri/src/notebooklm_export/renderer.rs`, add to `render_message_block`:

```rust
markdown.push_str(&format!(
    "history_scope: {}\n",
    yaml_string(&message.history_scope)
));
markdown.push_str(&format!(
    "migration_domain: {}\n",
    yaml_optional_string(message.migration_domain.as_deref())
));
```

Extend `DocumentRenderContext`:

```rust
pub(crate) history_scope_heading: Option<&'a str>,
```

At the top of `render_document_header`, before the existing Telegram export heading:

```rust
if let Some(heading) = context.history_scope_heading {
    output.push_str(&format!("# {heading}\n\n"));
}
```

- [ ] **Step 6: Split opted-in export into sections**

In `src-tauri/src/notebooklm_export/mod.rs`, load messages like this:

```rust
let current_messages = load_export_messages(
    &pool,
    config.source_id,
    config.period_from,
    config.period_to,
    query::ExportHistoryScope::Current,
)
.await?;
let migrated_messages = if config.include_migrated_history {
    load_export_messages(
        &pool,
        config.source_id,
        config.period_from,
        config.period_to,
        query::ExportHistoryScope::Migrated,
    )
    .await?
} else {
    Vec::new()
};
```

When `include_migrated_history` is false, keep the existing output path. When true, run chunking and file writing twice:

```rust
struct ExportSection {
    heading: &'static str,
    filename_prefix: &'static str,
    messages: Vec<NotebookLmExportMessage>,
}
```

Use:

```rust
vec![
    ExportSection {
        heading: "Current supergroup history",
        filename_prefix: "current-supergroup-history",
        messages: current_messages,
    },
    ExportSection {
        heading: "Migrated small-group history",
        filename_prefix: "migrated-small-group-history",
        messages: migrated_messages,
    },
]
```

Skip writing an empty migrated section file and add warning:

```rust
"Migrated small-group history was included, but no migrated messages matched the export range."
```

- [ ] **Step 7: Add renderer/mod tests**

In `src-tauri/src/notebooklm_export/renderer.rs`, add:

```rust
#[test]
fn renders_migrated_history_scope_metadata() {
    let block = render_message_block(&NotebookLmExportMessage {
        item_id: 1,
        source_id: 2,
        external_id: "3".to_string(),
        author: Some("Ada".to_string()),
        published_at: 0,
        text: Some("Old message".to_string()),
        content_kind: "text_only".to_string(),
        has_media: false,
        media_kind: None,
        media_metadata: ItemMediaMetadata::default(),
        media_placeholders: Vec::new(),
        urls: Vec::new(),
        reply_to_msg_id: None,
        reply_to_author: None,
        reply_to_snippet: None,
        reply_to_peer_kind: None,
        reply_to_peer_id: None,
        reply_to_top_id: None,
        reaction_count: None,
        forum_topic_id: None,
        forum_topic_title: None,
        forum_topic_top_message_id: None,
        history_scope: "migrated_small_group_history".to_string(),
        migration_domain: Some("migrated_from_chat".to_string()),
    });

    assert!(block.markdown.contains("history_scope: \"migrated_small_group_history\""));
    assert!(block.markdown.contains("migration_domain: \"migrated_from_chat\""));
}
```

- [ ] **Step 8: Add frontend export opt-in**

In `src/lib/types/sources.ts`, extend `NotebookLmExportRequest`:

```ts
include_migrated_history: boolean;
```

In `src/lib/analysis-state.ts`, extend `NotebookLmExportFormState`:

```ts
includeMigratedHistory: boolean;
```

In `notebookLmExportRequestFromForm`, add:

```ts
include_migrated_history: form.includeMigratedHistory,
```

In `src/lib/components/analysis/notebooklm-export-dialog.svelte`, extend the form type and add a checkbox:

```svelte
{#if source?.sourceType === "telegram" && source.migratedHistoryRowCount > 0}
  <CheckboxRow
    title="Include migrated historical scope"
    description="Export current and migrated history as separate sections."
    checked={form.includeMigratedHistory}
    disabled={exporting}
    onchange={(event) => updateForm({ includeMigratedHistory: (event.currentTarget as HTMLInputElement).checked })}
  />
{/if}
```

In `src/routes/analysis/+page.svelte`, add the default:

```ts
includeMigratedHistory: false,
```

- [ ] **Step 9: Update export tests**

In `src/lib/api/notebooklm-export.test.ts` or `src/lib/analysis-state.test.ts`, add:

```ts
it("maps NotebookLM migrated history opt-in to the backend request", () => {
  const request = notebookLmExportRequestFromForm("export-1", 7, {
    outputDir: "C:\\Export",
    range: "entire_history",
    fromDate: "",
    toDate: "",
    includeMediaPlaceholders: true,
    includeMigratedHistory: true,
    minMessageLength: 3,
    maxWordsPerFile: 300000,
    maxBytesPerFile: 50000000,
    overwriteExisting: false,
  });

  expect(request.include_migrated_history).toBe(true);
});
```

- [ ] **Step 10: Run export tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml opted_in_export_loads_migrated_rows_separately_with_markers
cargo test --manifest-path src-tauri\Cargo.toml migrated_export_reply_lookup_stays_inside_old_history_domain
cargo test --manifest-path src-tauri\Cargo.toml renders_migrated_history_scope_metadata
cargo test --manifest-path src-tauri\Cargo.toml notebooklm_default_export_excludes_migrated_history_rows
npm.cmd test -- src/lib/api/notebooklm-export.test.ts src/lib/analysis-state.test.ts
npm.cmd run check
```

Expected: all pass.

- [ ] **Step 11: Commit Task 4**

Run:

```powershell
git add src-tauri\src\notebooklm_export\model.rs src-tauri\src\notebooklm_export\query.rs src-tauri\src\notebooklm_export\renderer.rs src-tauri\src\notebooklm_export\mod.rs src\lib\types\sources.ts src\lib\components\analysis\notebooklm-export-dialog.svelte src\lib\analysis-state.ts src\lib\api\notebooklm-export.test.ts src\routes\analysis\+page.svelte docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "feat: add migrated history export opt-in"
```

Expected: commit succeeds.

---

### Task 5: Analysis Backend Opt-In And Snapshot Metadata

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/report_commands.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Test: `src-tauri/src/analysis/corpus.rs`
- Test: `src-tauri/src/analysis/report.rs`
- Test: `src-tauri/src/analysis/store.rs`

- [ ] **Step 1: Add backend request fields and run model fields**

In `src-tauri/src/analysis/report.rs`, extend `StartAnalysisReportRequest`:

```rust
pub(crate) include_migrated_history: bool,
```

In `src-tauri/src/analysis/report_commands.rs`, add the Tauri command argument and pass it into `StartAnalysisReportRequest`:

```rust
include_migrated_history: bool,
```

In `src-tauri/src/analysis/corpus.rs`, extend `CorpusLoadRequest`:

```rust
pub(crate) include_migrated_history: bool,
```

In `src-tauri/src/analysis/models.rs`, add to `AnalysisRunSummary`, `AnalysisRunDetail`, and `AnalysisRunRow`:

```rust
pub telegram_history_scope: String,
```

- [ ] **Step 2: Add corpus tests**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
#[tokio::test]
async fn opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight() {
    let pool = snapshot_pool().await;
    seed_analysis_source(&pool, 1, "telegram", "supergroup").await;
    seed_telegram_item(&pool, 10, 1, "10", 100, "current", false).await;
    seed_telegram_item(&pool, 11, 1, "11", 90, "migrated", true).await;
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild docs");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![1],
        period_from: 1,
        period_to: 200,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: true,
    };

    let corpus = load_corpus_messages(&pool, &request).await.expect("load corpus");
    assert_eq!(corpus.iter().map(|message| message.item_id).collect::<Vec<_>>(), vec![11, 10]);

    let migrated_metadata = decode_optional_metadata_json(corpus[0].metadata_zstd.as_deref())
        .expect("decode metadata")
        .expect("metadata");
    assert_eq!(migrated_metadata["history_scope"], "migrated");
    assert_eq!(migrated_metadata["migration_domain"], "migrated_from_chat");

    let preflight = preflight_analysis_run(
        &pool,
        &request,
        16000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");
    assert_eq!(preflight.message_count, 2);
}

#[tokio::test]
async fn source_group_opt_in_includes_only_members_with_migrated_rows() {
    let pool = snapshot_pool().await;
    seed_analysis_source(&pool, 1, "telegram", "supergroup").await;
    seed_analysis_source(&pool, 2, "telegram", "supergroup").await;
    seed_telegram_item(&pool, 10, 1, "10", 100, "current one", false).await;
    seed_telegram_item(&pool, 11, 1, "11", 90, "migrated one", true).await;
    seed_telegram_item(&pool, 20, 2, "20", 80, "current two", false).await;
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild source 1");
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 2)
        .await
        .expect("rebuild source 2");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![1, 2],
        period_from: 1,
        period_to: 200,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: true,
    };

    let corpus = load_corpus_messages(&pool, &request).await.expect("load corpus");

    assert_eq!(corpus.iter().map(|message| message.item_id).collect::<Vec<_>>(), vec![11, 20, 10]);
}
```

The helper `seed_telegram_item` inserts an item plus `telegram_messages`; for migrated rows set:

```rust
history_peer_kind = 'chat'
history_peer_id = 777
migration_domain = 'migrated_from_chat'
is_migrated_history = 1
```

For current rows set:

```rust
history_peer_kind = 'channel'
history_peer_id = 12345
migration_domain = NULL
is_migrated_history = 0
```

- [ ] **Step 3: Run corpus tests and verify failures**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
cargo test --manifest-path src-tauri\Cargo.toml source_group_opt_in_includes_only_members_with_migrated_rows
```

Expected: fail until the direct migrated corpus loader exists.

- [ ] **Step 4: Implement Telegram metadata helper**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
fn telegram_history_metadata_zstd(
    history_scope: &str,
    migration_domain: Option<&str>,
    history_peer_kind: &str,
    history_peer_id: i64,
) -> AppResult<Vec<u8>> {
    crate::compression::compress_json_bytes(
        &serde_json::to_vec(&serde_json::json!({
            "history_scope": history_scope,
            "migration_domain": migration_domain,
            "history_peer_kind": history_peer_kind,
            "history_peer_id": history_peer_id
        }))
        .map_err(internal_error)?,
    )
    .map_err(internal_error)
}
```

- [ ] **Step 5: Implement direct migrated corpus loader**

In `src-tauri/src/analysis/corpus.rs`, add a row type:

```rust
#[derive(sqlx::FromRow)]
struct TelegramCorpusRow {
    item_id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Vec<u8>,
    source_type: String,
    source_subtype: Option<String>,
    history_scope: String,
    migration_domain: Option<String>,
    history_peer_kind: String,
    history_peer_id: i64,
}
```

For Telegram requests, load current documents by joining `analysis_documents` to `telegram_messages` and building metadata with `history_scope = current`. When `include_migrated_history` is true, append direct migrated rows from `items + telegram_messages` with `history_scope = migrated`.

Use this migrated predicate:

```sql
tm.is_migrated_history = 1
AND tm.migration_domain = 'migrated_from_chat'
AND items.content_zstd IS NOT NULL
AND items.content_kind IN ('text_only', 'text_with_media')
```

Use this final sort:

```rust
messages.sort_by(|left, right| {
    left.published_at
        .cmp(&right.published_at)
        .then_with(|| left.source_id.cmp(&right.source_id))
        .then_with(|| left.r#ref.cmp(&right.r#ref))
});
```

- [ ] **Step 6: Store run-level telegram history scope**

In `src-tauri/src/analysis/store.rs`, extend `DuplicateRunLookup` and `AnalysisRunInsert`:

```rust
pub(crate) telegram_history_scope: &'a str,
```

Add duplicate predicate:

```sql
AND COALESCE(telegram_history_scope, 'current') = ?
```

Add insert column:

```sql
telegram_history_scope,
```

and bind:

```rust
.bind(insert.telegram_history_scope)
```

Update every run SELECT to include:

```sql
COALESCE(runs.telegram_history_scope, 'current') AS telegram_history_scope,
```

Map summary/detail:

```rust
telegram_history_scope: row.telegram_history_scope,
```

- [ ] **Step 7: Thread report opt-in through preflight and insert**

In `start_analysis_report_run`, after resolving sources:

```rust
let include_migrated_history =
    request.include_migrated_history && resolved_sources.source_type == "telegram";
if request.include_migrated_history && resolved_sources.source_type != "telegram" {
    return Err(AppError::validation(
        "Migrated historical scope can be included only for Telegram analysis",
    ));
}
let telegram_history_scope = if include_migrated_history {
    crate::sources::types::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED
} else {
    crate::sources::types::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT
};
```

Build `CorpusLoadRequest` with:

```rust
include_migrated_history,
```

Pass `telegram_history_scope` to duplicate lookup and run insert.

- [ ] **Step 8: Add store/report tests**

In `src-tauri/src/analysis/store.rs`, add a test that inserts two active runs with identical fields except `telegram_history_scope` and proves duplicate lookup only matches the same scope.

Use this assertion:

```rust
assert_eq!(current_duplicate, Some(current_run_id));
assert_eq!(current_plus_migrated_duplicate, Some(migrated_run_id));
```

In `src-tauri/src/analysis/report.rs`, add:

```rust
#[test]
fn report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape() {
    let request = StartAnalysisReportRequest {
        source_id: Some(1),
        source_group_id: None,
        period_from: 1,
        period_to: 2,
        output_language: "Russian".to_string(),
        prompt_template_id: 1,
        model_override: None,
        profile_id: None,
        youtube_corpus_mode: None,
        include_migrated_history: true,
    };

    assert!(request.include_migrated_history);
}
```

- [ ] **Step 9: Include scope in follow-up chat context**

In `src-tauri/src/analysis/chat.rs`, update `format_chat_context_messages`:

```rust
let history_scope = message
    .metadata_zstd
    .as_deref()
    .and_then(history_scope_label_from_metadata)
    .unwrap_or("Current supergroup history");
```

Include it in the context text:

```rust
"[{ref}] Date: {published_at}\nHistory scope: {history_scope}\nAuthor: {author}\nExcerpt:\n{excerpt}"
```

Add helper:

```rust
fn history_scope_label_from_metadata(metadata_zstd: &[u8]) -> Option<&'static str> {
    let bytes = crate::compression::decompress_bytes(metadata_zstd).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    match value.get("history_scope").and_then(|value| value.as_str()) {
        Some("migrated") => Some("Migrated small-group history"),
        Some("current") => Some("Current supergroup history"),
        _ => None,
    }
}
```

- [ ] **Step 10: Run backend analysis tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
cargo test --manifest-path src-tauri\Cargo.toml source_group_opt_in_includes_only_members_with_migrated_rows
cargo test --manifest-path src-tauri\Cargo.toml default_analysis_corpus_excludes_migrated_history_documents
cargo test --manifest-path src-tauri\Cargo.toml capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
cargo test --manifest-path src-tauri\Cargo.toml duplicate
```

Expected: all targeted tests pass. The broad `duplicate` filter may run unrelated duplicate tests; failures there must be investigated before committing.

- [ ] **Step 11: Commit Task 5**

Run:

```powershell
git add src-tauri\src\analysis\corpus.rs src-tauri\src\analysis\report.rs src-tauri\src\analysis\report_commands.rs src-tauri\src\analysis\store.rs src-tauri\src\analysis\models.rs src-tauri\src\analysis\chat.rs docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "feat: include migrated history in opted-in analysis snapshots"
```

Expected: commit succeeds.

---

### Task 6: Analysis Frontend Opt-In And Saved Run Labels

**Files:**
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Modify: `src/lib/api/analysis-runs.test.ts`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
- Modify: `src/lib/components/analysis/report-run-header.svelte`
- Modify: `src/routes/analysis/+page.svelte`
- Test: `src/lib/analysis-redesign-route-contract.test.ts`
- Test: `src/lib/analysis-redesign-safety-contract.test.ts`

- [ ] **Step 1: Add TypeScript run and command fields**

In `src/lib/types/analysis.ts`, add:

```ts
export type AnalysisTelegramHistoryScope = "current" | "current_plus_migrated";
```

Add to `AnalysisRunSummary`:

```ts
telegram_history_scope: AnalysisTelegramHistoryScope;
```

Add to `AnalysisReportStartCommand`:

```ts
includeMigratedHistory: boolean;
```

- [ ] **Step 2: Pass report command field**

In `src/lib/api/analysis-runs.ts`, keep spreading the command. In `src/lib/api/analysis-runs.test.ts`, update both `startAnalysisReport` fixtures:

```ts
includeMigratedHistory: true,
```

Expected Tauri invocation includes:

```ts
includeMigratedHistory: true,
```

- [ ] **Step 3: Extend report start state**

In `src/lib/analysis-state.ts`, extend `AnalysisReportStartState`:

```ts
includeMigratedHistory: boolean;
```

In `analysisReportStartCommand`, add:

```ts
includeMigratedHistory: state.includeMigratedHistory,
```

Add helper:

```ts
export function canIncludeMigratedHistoryInReport(
  state: Pick<ReportLaunchPreflightState, "analysisScope" | "currentSource" | "currentGroup" | "sourceCatalog">,
) {
  if (state.analysisScope === "single_source") {
    return !!state.currentSource
      && state.currentSource.sourceType === "telegram"
      && state.currentSource.migratedHistoryRowCount > 0;
  }

  if (!state.currentGroup || state.currentGroup.source_type !== "telegram") {
    return false;
  }

  return state.currentGroup.members.some((member) => {
    const source = state.sourceCatalog.find((candidate) => candidate.id === member.source_id);
    return source?.sourceType === "telegram" && source.migratedHistoryRowCount > 0;
  });
}
```

- [ ] **Step 4: Add analysis-state tests**

In `src/lib/analysis-state.test.ts`, add:

```ts
it("passes migrated historical scope opt-in into report start command", () => {
  const decision = analysisReportStartCommand({
    analysisScope: "single_source",
    selectedSourceId: "7",
    selectedGroupId: "",
    selectedTemplateId: "5",
    periodFrom: "2026-05-01",
    periodTo: "2026-05-02",
    outputLanguage: "Russian",
    profileId: null,
    modelOverride: "",
    youtubeCorpusMode: "transcript_description",
    includeMigratedHistory: true,
  });

  expect(decision).toMatchObject({
    ok: true,
    command: {
      includeMigratedHistory: true,
    },
  });
});

it("enables migrated analysis opt-in for Telegram groups when at least one member has imported rows", () => {
  expect(canIncludeMigratedHistoryInReport({
    analysisScope: "source_group",
    currentSource: null,
    currentGroup: {
      id: 1,
      name: "Group",
      source_type: "telegram",
      members: [
        { source_id: 7, source_title: "A", item_count: 10 },
        { source_id: 8, source_title: "B", item_count: 10 },
      ],
      created_at: 1,
      updated_at: 1,
    },
    sourceCatalog: [
      sourceRecord({ id: 7, sourceType: "telegram", migratedHistoryRowCount: 0 }),
      sourceRecord({ id: 8, sourceType: "telegram", migratedHistoryRowCount: 3 }),
    ],
  })).toBe(true);
});
```

- [ ] **Step 5: Add report setup checkbox**

In `src/lib/components/analysis/report-setup-panel.svelte`, add props:

```ts
includeMigratedHistory: boolean;
canIncludeMigratedHistory: boolean;
onChangeIncludeMigratedHistory: (value: boolean) => void;
```

Import `CheckboxRow` if not already imported.

Render near YouTube corpus controls:

```svelte
{#if canIncludeMigratedHistory}
  <CheckboxRow
    title="Include migrated historical scope"
    description="Add imported small-group history to this saved snapshot."
    checked={includeMigratedHistory}
    disabled={startingReport}
    onchange={(event) => onChangeIncludeMigratedHistory((event.currentTarget as HTMLInputElement).checked)}
  />
{/if}
```

- [ ] **Step 6: Wire page state**

In `src/routes/analysis/+page.svelte`, add:

```ts
let includeMigratedHistoryInReport = $state(false);
```

When starting report, pass:

```ts
includeMigratedHistory:
  canIncludeMigratedHistoryInReport(currentReportLaunchState())
    ? includeMigratedHistoryInReport
    : false,
```

Pass into `ReportCanvas`:

```svelte
includeMigratedHistory={includeMigratedHistoryInReport}
canIncludeMigratedHistory={canIncludeMigratedHistoryInReport(currentReportLaunchState())}
onChangeIncludeMigratedHistory={(value) => (includeMigratedHistoryInReport = value)}
```

Reset to false when the workspace selection changes to a non-eligible scope:

```ts
if (!canIncludeMigratedHistoryInReport(currentReportLaunchState())) {
  includeMigratedHistoryInReport = false;
}
```

- [ ] **Step 7: Show saved run marker**

In `src/lib/components/analysis/report-run-header.svelte`, render a meta cell when the saved run included migrated history:

```svelte
{#if currentRun.telegram_history_scope === "current_plus_migrated"}
  <MetaCell label="Telegram history">Current + migrated historical scope</MetaCell>
{/if}
```

- [ ] **Step 8: Update route and safety tests**

In `src/lib/analysis-redesign-route-contract.test.ts`, add:

```ts
it("passes migrated historical scope opt-in through report setup", () => {
  expect(analysisPageSource).toContain("includeMigratedHistoryInReport");
  expect(analysisPageSource).toContain("canIncludeMigratedHistoryInReport");
  expect(reportSetupPanelSource).toContain("Include migrated historical scope");
});
```

In `src/lib/analysis-redesign-safety-contract.test.ts`, add:

```ts
it("surfaces saved Telegram historical scope instead of treating it as ordinary current history", () => {
  expect(reportRunHeaderSource).toContain("telegram_history_scope");
  expect(reportRunHeaderSource).toContain("Current + migrated historical scope");
  expect(analysisTypesSource).toContain("AnalysisTelegramHistoryScope");
});
```

- [ ] **Step 9: Run frontend analysis tests**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts src/lib/analysis-state.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts
npm.cmd run check
```

Expected: all pass.

- [ ] **Step 10: Commit Task 6**

Run:

```powershell
git add src\lib\types\analysis.ts src\lib\api\analysis-runs.ts src\lib\api\analysis-runs.test.ts src\lib\analysis-state.ts src\lib\analysis-state.test.ts src\lib\components\analysis\report-setup-panel.svelte src\lib\components\analysis\report-run-header.svelte src\routes\analysis\+page.svelte src\lib\analysis-redesign-route-contract.test.ts src\lib\analysis-redesign-safety-contract.test.ts docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "feat: add migrated history analysis opt-in"
```

Expected: commit succeeds.

---

### Task 7: Documentation, Backlog, And Final Verification

**Files:**
- Modify: `docs/takeout-source-import.md`
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/specs/2026-05-28-migrated-history-scope-product-behavior-design.md`
- Modify: `docs/superpowers/plans/2026-05-28-migrated-history-scope-controls.md`

- [ ] **Step 1: Update Takeout/source docs**

In `docs/takeout-source-import.md`, extend the migrated-history section with:

```markdown
### Historical Scope Usage

Imported migrated small-group history remains outside default browsing,
analysis, and NotebookLM export. The source reader defaults to
`Current supergroup history`. Users can explicitly switch to
`Migrated small-group history` or `Merged timeline` when imported migrated rows
exist.

Analysis and export use separate explicit opt-ins. Analysis records the
run-level decision in `analysis_runs.telegram_history_scope`; exported
NotebookLM files render current and migrated history as separate sections.
```

- [ ] **Step 2: Update database schema docs**

In `docs/database-schema.md`, document:

```markdown
`analysis_runs.telegram_history_scope` is nullable for backward compatibility.
`NULL` means `current`. New runs store either `current` or
`current_plus_migrated`.
```

Document source DTO counts:

```markdown
Source records expose sanitized migrated-history availability fields:
`migrated_history_status`, `migrated_history_row_count`, and
`migrated_history_import_completed`. They do not expose old chat ids.
```

- [ ] **Step 3: Update backlog**

In `docs/backlog.md`, mark the item complete:

```markdown
- [x] implement explicit browsing, analysis, and export controls for migrated
  historical scope
  - Browsing defaults to current history, provides explicit current/migrated/
    merged scope selection for imported migrated rows, and labels migrated
    rows.
  - NotebookLM export defaults to current history and uses an explicit opt-in
    with separate current and migrated sections.
  - Analysis defaults to current history and stores opt-in runs as
    `telegram_history_scope = current_plus_migrated` with row-level snapshot
    metadata.
```

- [ ] **Step 4: Update the spec implementation status**

Append to `docs/superpowers/specs/2026-05-28-migrated-history-scope-product-behavior-design.md`:

```markdown
## Implementation Status

Implemented by `docs/superpowers/plans/2026-05-28-migrated-history-scope-controls.md`.
Default behavior remains current-history-only. Opted-in browsing, export, and
analysis preserve visible migrated-history labels and saved metadata.
```

- [ ] **Step 5: Run backend verification**

Run:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
cargo test --manifest-path src-tauri\Cargo.toml
```

Expected: both commands exit 0.

- [ ] **Step 6: Run frontend verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
```

Expected: both commands exit 0.

- [ ] **Step 7: Run whitespace verification**

Run:

```powershell
git diff --check
```

Expected: no output and exit 0.

- [ ] **Step 8: Commit Task 7**

Run:

```powershell
git add docs\takeout-source-import.md docs\database-schema.md docs\backlog.md docs\superpowers\specs\2026-05-28-migrated-history-scope-product-behavior-design.md docs\superpowers\plans\2026-05-28-migrated-history-scope-controls.md
git commit -m "docs: document migrated history scope controls"
```

Expected: commit succeeds.

---

## Final Acceptance Checklist

- [ ] Direct `list_source_items` calls without `history_scope` return current history only.
- [ ] Telegram browsing never mixes archive first pages with direct cursor pages; it uses the direct scoped query for every Telegram reader page.
- [ ] Current Telegram browsing excludes migrated rows through the direct scoped items path.
- [ ] Migrated browsing returns only `is_migrated_history = 1` rows with backend-owned labels.
- [ ] Merged browsing returns both current and migrated rows and labels every migrated row.
- [ ] Merged paging and around-item loading use the full ordering tuple, not only `published_at` or `item_id`.
- [ ] Frontend source item cursors are opaque strings and never expose `history_peer_id` or other raw Telegram peer ids.
- [ ] Current forum-topic filters are not applied to migrated small-group history.
- [ ] Source DTOs expose row counts and import-completed state without old chat ids.
- [ ] Available-but-not-imported and imported-zero-row states render explanatory UI states.
- [ ] Default NotebookLM export excludes migrated rows.
- [ ] Opted-in NotebookLM export writes current and migrated history as separate sections.
- [ ] Migrated export rows contain `history_scope: migrated_small_group_history` and `migration_domain: migrated_from_chat`.
- [ ] Migrated export reply lookup stays inside the old-history domain.
- [ ] Default analysis corpus excludes migrated rows.
- [ ] Opted-in Telegram analysis includes migrated rows from every selected source that has them.
- [ ] Source-group analysis opt-in does not fail when only some group members have migrated rows.
- [ ] Analysis preflight counts include opted-in migrated rows.
- [ ] `analysis_runs.telegram_history_scope` stores `current` or `current_plus_migrated`; old `NULL` runs map to `current`.
- [ ] Snapshot rows include message-level historical markers in `analysis_run_messages.metadata_zstd`.
- [ ] Follow-up chat remains snapshot-backed and can describe migrated evidence scope.
- [ ] `cargo check --manifest-path src-tauri\Cargo.toml` passes.
- [ ] `cargo test --manifest-path src-tauri\Cargo.toml` passes.
- [ ] `npm.cmd test` passes.
- [ ] `npm.cmd run check` passes.
- [ ] `git diff --check` passes.
