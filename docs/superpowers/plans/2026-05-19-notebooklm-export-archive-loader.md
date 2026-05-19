# NotebookLM Export Archive Loader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Telegram NotebookLM export a readiness-gated consumer of `archive_read_items`, with old-path parity tests and no silent fallback after the archive loader is selected.

**Architecture:** `export_source_to_notebooklm` keeps validating the source as Telegram through `load_export_source` before loading messages. `load_export_messages` becomes a gate wrapper that selects either the existing items path or a new archive-read path once per call. The archive path owns the entire message load, including reply snippets outside the export period, and never joins back to canonical `items` after selection.

**Tech Stack:** Rust, SQLx SQLite, Tauri command backend, zstd compression helpers, existing NotebookLM export tests, `cargo test`.

---

## File Structure

- Modify `src-tauri/src/notebooklm_export/query.rs`: add loader selection enums, expose old-path and archive-path loaders for tests, implement archive message loading and archive reply snippet lookup, and make `load_export_messages` the gated wrapper.
- Modify `docs/database-schema.md`: document NotebookLM export as a readiness-gated archive-read consumer with old-path fallback for non-ready states.
- Modify `docs/database-schema-read-model-decision.md`: mark Telegram NotebookLM export archive-loader slice as implemented and keep YouTube export enrichment future-facing.
- Modify `docs/backlog.md`: remove Telegram NotebookLM export archive-model migration from open Database schema simplification items; keep YouTube playlist-entry read-model decision and current-schema baseline open.

---

### Task 1: Loader Selection Enum And Old-Path Naming

**Files:**
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Test: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Extend the export query test fixture for archive state**

In `src-tauri/src/notebooklm_export/query.rs`, update the test module import:

```rust
use super::{
    load_export_messages, load_export_messages_from_items_path, load_export_source,
    select_notebooklm_export_loader, ArchiveReadinessFallbackReason, ExportLoaderSelection,
};
```

Update the `items` table created in `export_pool()` so it can support archive
read-model rebuilds:

```rust
CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    item_kind TEXT NOT NULL DEFAULT 'telegram_message',
    author TEXT,
    published_at INTEGER NOT NULL,
    ingested_at INTEGER NOT NULL DEFAULT 0,
    content_zstd BLOB,
    raw_data_zstd BLOB,
    content_kind TEXT NOT NULL,
    has_media INTEGER NOT NULL,
    media_kind TEXT,
    media_metadata_zstd BLOB,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id TEXT,
    reply_to_top_id INTEGER,
    reaction_count INTEGER
)
```

After `seed_materialized_topic_schema(&pool).await;`, create the archive read
model tables:

```rust
crate::sources::test_support::create_archive_read_model_tables(&pool).await;
```

Add these helpers inside the test module:

```rust
async fn seed_export_source(pool: &sqlx::SqlitePool) {
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
    )
    .execute(pool)
    .await
    .expect("seed export source");
}

async fn seed_archive_state(pool: &sqlx::SqlitePool, status: &str, model_version: i64) {
    sqlx::query(
        "INSERT INTO archive_read_model_state (
            source_id, model_version, status, built_at, item_count, row_count
         ) VALUES (1, ?, ?, 100, 0, 0)",
    )
    .bind(model_version)
    .bind(status)
    .execute(pool)
    .await
    .expect("seed archive state");
}
```

- [x] **Step 2: Add failing loader selection tests**

Add these tests in the same test module:

```rust
#[tokio::test]
async fn notebooklm_export_loader_selection_reports_all_fallback_reasons() {
    let cases = [
        (
            "never_built",
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            ArchiveReadinessFallbackReason::NeverBuilt,
        ),
        (
            "building",
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            ArchiveReadinessFallbackReason::Building,
        ),
        (
            "stale",
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            ArchiveReadinessFallbackReason::Stale,
        ),
        (
            "failed",
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            ArchiveReadinessFallbackReason::Failed,
        ),
    ];

    for (status, version, expected_reason) in cases {
        let pool = export_pool().await;
        seed_export_source(&pool).await;
        seed_archive_state(&pool, status, version).await;

        let selection = select_notebooklm_export_loader(&pool, 1)
            .await
            .expect("select loader");

        assert_eq!(
            selection,
            ExportLoaderSelection::ItemsPath {
                reason: expected_reason
            },
            "unexpected selection for {status}"
        );
    }
}

#[tokio::test]
async fn notebooklm_export_loader_selection_reports_missing_and_old_version() {
    let pool = export_pool().await;
    seed_export_source(&pool).await;

    assert_eq!(
        select_notebooklm_export_loader(&pool, 1)
            .await
            .expect("select missing state"),
        ExportLoaderSelection::ItemsPath {
            reason: ArchiveReadinessFallbackReason::MissingState
        }
    );

    seed_archive_state(
        &pool,
        crate::archive_read_model::STATUS_READY,
        crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION - 1,
    )
    .await;

    assert_eq!(
        select_notebooklm_export_loader(&pool, 1)
            .await
            .expect("select old state"),
        ExportLoaderSelection::ItemsPath {
            reason: ArchiveReadinessFallbackReason::OldModelVersion {
                found: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION - 1,
                current: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            }
        }
    );
}

#[tokio::test]
async fn notebooklm_export_loader_selection_uses_archive_for_ready_current_state() {
    let pool = export_pool().await;
    seed_export_source(&pool).await;
    seed_archive_state(
        &pool,
        crate::archive_read_model::STATUS_READY,
        crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
    )
    .await;

    assert_eq!(
        select_notebooklm_export_loader(&pool, 1)
            .await
            .expect("select ready state"),
        ExportLoaderSelection::ArchiveReadModel {
            model_version: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        }
    );
}

#[tokio::test]
async fn load_export_source_rejects_non_telegram_before_message_loader_selection() {
    let pool = export_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (2, 'youtube', 'video', 'video-id', 'Video')",
    )
    .execute(&pool)
    .await
    .expect("seed youtube source");
    sqlx::query(
        "INSERT INTO archive_read_model_state (
            source_id, model_version, status, built_at, item_count, row_count
         ) VALUES (2, ?, 'ready', 100, 0, 0)",
    )
    .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
    .execute(&pool)
    .await
    .expect("seed ready youtube archive state");

    let error = load_export_source(&pool, 2)
        .await
        .expect_err("youtube source is rejected before message loading");

    assert!(error.to_string().contains("is not a Telegram source"));
}
```

- [x] **Step 3: Run loader selection tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_loader_selection_
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection
```

Expected: compile failures because `ExportLoaderSelection`,
`ArchiveReadinessFallbackReason`, `select_notebooklm_export_loader`, and
`load_export_messages_from_items_path` do not exist yet.

- [x] **Step 4: Implement selection enums and helper**

Near the row structs in `src-tauri/src/notebooklm_export/query.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExportLoaderSelection {
    ArchiveReadModel { model_version: i64 },
    ItemsPath {
        reason: ArchiveReadinessFallbackReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArchiveReadinessFallbackReason {
    MissingState,
    NeverBuilt,
    Building,
    Stale,
    Failed,
    OldModelVersion { found: i64, current: i64 },
}
```

Add:

```rust
pub(crate) async fn select_notebooklm_export_loader(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<ExportLoaderSelection> {
    let Some(state) = crate::archive_read_model::load_source_state(pool, source_id).await? else {
        return Ok(ExportLoaderSelection::ItemsPath {
            reason: ArchiveReadinessFallbackReason::MissingState,
        });
    };

    if state.status == crate::archive_read_model::STATUS_READY
        && state.model_version == crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION
    {
        return Ok(ExportLoaderSelection::ArchiveReadModel {
            model_version: state.model_version,
        });
    }

    let reason = if state.status == crate::archive_read_model::STATUS_READY {
        ArchiveReadinessFallbackReason::OldModelVersion {
            found: state.model_version,
            current: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        }
    } else {
        match state.status.as_str() {
            crate::archive_read_model::STATUS_NEVER_BUILT => {
                ArchiveReadinessFallbackReason::NeverBuilt
            }
            crate::archive_read_model::STATUS_BUILDING => {
                ArchiveReadinessFallbackReason::Building
            }
            crate::archive_read_model::STATUS_STALE => ArchiveReadinessFallbackReason::Stale,
            crate::archive_read_model::STATUS_FAILED => ArchiveReadinessFallbackReason::Failed,
            _ => ArchiveReadinessFallbackReason::Failed,
        }
    };

    Ok(ExportLoaderSelection::ItemsPath { reason })
}
```

Rename the existing `load_export_messages` implementation to:

```rust
pub(crate) async fn load_export_messages_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    // existing load_export_messages body
}
```

Add a temporary compatibility wrapper:

```rust
pub(crate) async fn load_export_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    load_export_messages_from_items_path(pool, source_id, period_from, period_to).await
}
```

- [x] **Step 5: Run Task 1 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_loader_selection_
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: selection tests pass, existing NotebookLM query tests still pass,
formatting passes, and diff check has no whitespace errors except Git
line-ending warnings.

- [x] **Step 6: Commit loader selection**

Run:

```powershell
git add src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: select notebooklm export loader"
```

Expected: commit succeeds.

---

### Task 2: Archive Export Loader And Parity Tests

**Files:**
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Test: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Add the parity fixture helpers**

In the test module imports, add:

```rust
use crate::media::{encode_media_metadata, ItemMediaMetadata};
```

Add this helper:

```rust
async fn seed_notebooklm_export_parity_fixture(pool: &sqlx::SqlitePool) {
    seed_export_source(pool).await;

    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, is_deleted
         ) VALUES (1, 200, 700, 'Roadmap', 0)",
    )
    .execute(pool)
    .await
    .expect("seed forum topic");

    let photo_metadata = encode_media_metadata(&ItemMediaMetadata {
        summary: Some("Photo".to_string()),
        file_name: Some("roadmap.png".to_string()),
        mime_type: Some("image/png".to_string()),
        size_bytes: Some(42),
        width: Some(640),
        height: Some(480),
        duration_seconds: None,
    })
    .expect("encode photo metadata");
    let document_metadata = encode_media_metadata(&ItemMediaMetadata {
        summary: Some("Document".to_string()),
        file_name: Some("notes.pdf".to_string()),
        mime_type: Some("application/pdf".to_string()),
        size_bytes: Some(128),
        width: None,
        height: None,
        duration_seconds: None,
    })
    .expect("encode document metadata");

    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, ingested_at,
            content_zstd, content_kind, has_media, media_kind, media_metadata_zstd
         ) VALUES
            (1, 1, '10', 'telegram_message', 'Bob', 10, 10, ?, 'text_only', 0, NULL, NULL),
            (2, 1, '20', 'telegram_message', 'Ada', 100, 100, ?, 'text_with_media', 1, 'photo', ?),
            (3, 1, '30', 'telegram_message', 'Cy', 110, 110, NULL, 'media_only', 1, 'document', ?),
            (4, 1, '40', 'telegram_message', 'Dana', 120, 120, ?, 'text_only', 0, NULL, NULL),
            (5, 1, '700a', 'telegram_message', 'Eve', 130, 130, ?, 'text_only', 0, NULL, NULL)",
    )
    .bind(compress_text("Original reply target").expect("compress original"))
    .bind(compress_text("Reply with link https://example.test").expect("compress reply"))
    .bind(photo_metadata)
    .bind(document_metadata)
    .bind(compress_text("Missing reply target").expect("compress missing reply"))
    .bind(compress_text("Looks numeric but is not").expect("compress nonnumeric"))
    .execute(pool)
    .await
    .expect("seed parity items");

    sqlx::query(
        "UPDATE items
         SET reply_to_msg_id = 10,
             reply_to_peer_kind = 'channel',
             reply_to_peer_id = '42',
             reply_to_top_id = 200,
             reaction_count = 3
         WHERE id = 2",
    )
    .execute(pool)
    .await
    .expect("update reply metadata");

    sqlx::query(
        "UPDATE items
         SET reply_to_msg_id = 999,
             reply_to_peer_kind = 'channel',
             reply_to_peer_id = '42',
             reaction_count = 1
         WHERE id = 4",
    )
    .execute(pool)
    .await
    .expect("update missing reply metadata");

    for item_id in [2_i64, 3_i64] {
        sqlx::query(
            "INSERT INTO item_topic_memberships (
                item_id, source_id, topic_id, match_kind, resolver_version
             ) VALUES (?, 1, 200, 'reply_to_top_id', 1)",
        )
        .bind(item_id)
        .execute(pool)
        .await
        .expect("seed topic membership");
    }
}
```

- [x] **Step 2: Add failing archive parity tests**

Add:

```rust
#[tokio::test]
async fn archive_export_loader_matches_items_path_for_notebooklm_messages() {
    let pool = export_pool().await;
    seed_notebooklm_export_parity_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive model");

    let old_rows = load_export_messages_from_items_path(&pool, 1, None, None)
        .await
        .expect("load old path");
    let archive_rows = load_export_messages_from_archive(&pool, 1, None, None)
        .await
        .expect("load archive path");

    assert_eq!(archive_rows, old_rows);
    assert_eq!(archive_rows.len(), 5);
    assert_eq!(archive_rows[1].reply_to_snippet.as_deref(), Some("Original reply target"));
    assert_eq!(archive_rows[1].reply_to_peer_kind.as_deref(), Some("channel"));
    assert_eq!(archive_rows[1].reply_to_peer_id.as_deref(), Some("42"));
    assert_eq!(archive_rows[1].reply_to_top_id, Some(200));
    assert_eq!(archive_rows[1].reaction_count, Some(3));
    assert_eq!(archive_rows[1].forum_topic_title.as_deref(), Some("Roadmap"));
    assert!(!archive_rows[1].media_placeholders.is_empty());
    assert!(!archive_rows[2].media_placeholders.is_empty());
    assert_eq!(archive_rows[4].forum_topic_id, None);
}

#[tokio::test]
async fn archive_export_loader_matches_items_path_for_bounded_periods() {
    let pool = export_pool().await;
    seed_notebooklm_export_parity_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive model");

    let old_rows = load_export_messages_from_items_path(&pool, 1, Some(50), Some(115))
        .await
        .expect("load old bounded path");
    let archive_rows = load_export_messages_from_archive(&pool, 1, Some(50), Some(115))
        .await
        .expect("load archive bounded path");

    assert_eq!(archive_rows, old_rows);
    assert_eq!(
        archive_rows.iter().map(|row| row.external_id.as_str()).collect::<Vec<_>>(),
        vec!["20", "30"]
    );
    assert_eq!(archive_rows[0].reply_to_snippet.as_deref(), Some("Original reply target"));
}

#[tokio::test]
async fn export_fixture_rejects_null_published_at_before_loader_parity() {
    let pool = export_pool().await;
    seed_export_source(&pool).await;

    let result = sqlx::query(
        "INSERT INTO items (
            source_id, external_id, author, published_at, content_zstd, content_kind, has_media
         ) VALUES (1, 'null-date', 'Ada', NULL, ?, 'text_only', 0)",
    )
    .bind(compress_text("Null date").expect("compress null date"))
    .execute(&pool)
    .await;

    assert!(result.is_err());
}
```

- [x] **Step 3: Run archive parity tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity
```

Expected: the first two tests fail to compile because
`load_export_messages_from_archive` does not exist. The null published-at test
should pass after fixture setup compiles.

- [x] **Step 4: Implement shared export row mapping**

Rename `ItemRow` to `ExportMessageRow` so both paths can map the same shape:

```rust
#[derive(FromRow)]
struct ExportMessageRow {
    id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Option<Vec<u8>>,
    content_kind: String,
    has_media: bool,
    media_kind: Option<String>,
    media_metadata_zstd: Option<Vec<u8>>,
    reply_to_msg_id: Option<i64>,
    reply_to_peer_kind: Option<String>,
    reply_to_peer_id: Option<String>,
    reply_to_top_id: Option<i64>,
    reaction_count: Option<i64>,
    forum_topic_id: Option<i64>,
    forum_topic_title: Option<String>,
    forum_topic_top_message_id: Option<i64>,
}
```

Extract row mapping from `load_export_messages_from_items_path`:

```rust
fn map_export_rows(
    rows: Vec<ExportMessageRow>,
    reply_contexts: HashMap<i64, ReplyContext>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    rows.into_iter()
        .map(|row| {
            let text = row
                .content_zstd
                .as_deref()
                .map(decompress_text)
                .transpose()?;
            let urls = text.as_deref().map(detect_urls).unwrap_or_default();
            let media_metadata = decode_media_metadata(row.media_metadata_zstd.as_deref())?;
            let media_placeholders =
                render_media_placeholders(row.media_kind.as_deref(), &media_metadata);
            let reply_context = row
                .reply_to_msg_id
                .and_then(|reply_to_msg_id| reply_contexts.get(&reply_to_msg_id));

            Ok(NotebookLmExportMessage {
                item_id: row.id,
                source_id: row.source_id,
                external_id: row.external_id,
                author: row.author,
                published_at: row.published_at,
                text,
                content_kind: row.content_kind,
                has_media: row.has_media,
                media_kind: row.media_kind,
                media_metadata,
                media_placeholders,
                urls,
                reply_to_msg_id: row.reply_to_msg_id,
                reply_to_author: reply_context.and_then(|context| context.author.clone()),
                reply_to_snippet: row.reply_to_msg_id.map(|_| {
                    reply_context
                        .map(|context| context.snippet.clone())
                        .unwrap_or_else(|| "Original message unavailable".to_string())
                }),
                reply_to_peer_kind: row.reply_to_peer_kind,
                reply_to_peer_id: row.reply_to_peer_id,
                reply_to_top_id: row.reply_to_top_id,
                reaction_count: row.reaction_count,
                forum_topic_id: row.forum_topic_id,
                forum_topic_title: row.forum_topic_title,
                forum_topic_top_message_id: row.forum_topic_top_message_id,
            })
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(AppError::from)
}
```

Update the old path to call:

```rust
let reply_contexts = load_reply_contexts_from_items_path(pool, source_id, &rows).await?;
map_export_rows(rows, reply_contexts)
```

- [x] **Step 5: Implement archive path SQL and archive reply lookup**

Add:

```rust
pub(crate) async fn load_export_messages_from_archive(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    let rows: Vec<ExportMessageRow> = match (period_from, period_to) {
        (Some(from), Some(to)) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'
                 AND published_at >= ? AND published_at <= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (Some(from), None) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'
                 AND published_at >= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .bind(from)
                .fetch_all(pool)
                .await
        }
        (None, Some(to)) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'
                 AND published_at <= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (None, None) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .fetch_all(pool)
                .await
        }
    }
    .map_err(AppError::database)?;

    let reply_contexts = load_reply_contexts_from_archive(pool, source_id, &rows).await?;
    map_export_rows(rows, reply_contexts)
}
```

Add:

```rust
fn archive_base_query(where_clause: &str) -> String {
    format!(
        r#"
    SELECT
        item_id AS id,
        source_id,
        external_id,
        author,
        published_at,
        content_zstd,
        content_kind,
        has_media,
        media_kind,
        media_metadata_zstd,
        reply_to_msg_id,
        reply_to_peer_kind,
        reply_to_peer_id,
        reply_to_top_id,
        reaction_count,
        forum_topic_id,
        forum_topic_title,
        forum_topic_top_message_id
    FROM archive_read_items
    WHERE {where_clause}
    ORDER BY published_at ASC, item_id ASC
"#
    )
}
```

Rename the existing reply lookup to `load_reply_contexts_from_items_path`, then
add archive lookup:

```rust
async fn load_reply_contexts_from_archive(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    rows: &[ExportMessageRow],
) -> AppResult<HashMap<i64, ReplyContext>> {
    let mut reply_ids = rows
        .iter()
        .filter_map(|row| row.reply_to_msg_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    reply_ids.sort_unstable();

    let mut contexts = HashMap::new();
    for chunk in reply_ids.chunks(500) {
        if chunk.is_empty() {
            continue;
        }

        let placeholders = std::iter::repeat_n("?", chunk.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            r#"
            SELECT external_id, author, content_zstd, has_media, media_kind
            FROM archive_read_items
            WHERE source_id = ?
              AND model_version = ?
              AND item_kind = 'telegram_message'
              AND external_id IN ({placeholders})
            "#
        );

        let mut query = sqlx::query_as::<_, ReplyLookupRow>(&sql)
            .bind(source_id)
            .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION);
        for reply_id in chunk {
            query = query.bind(reply_id.to_string());
        }

        let lookup_rows = query.fetch_all(pool).await.map_err(AppError::database)?;
        for row in lookup_rows {
            let Ok(reply_id) = row.external_id.parse::<i64>() else {
                continue;
            };
            let snippet = reply_snippet(&row)?;
            contexts.insert(
                reply_id,
                ReplyContext {
                    author: row.author,
                    snippet,
                },
            );
        }
    }

    Ok(contexts)
}
```

This SQL intentionally matches `external_id IN (?)` with bound
`reply_to_msg_id.to_string()` values. It must not cast `archive_read_items.external_id`
to integer.

- [x] **Step 6: Run Task 2 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: archive parity tests pass, existing NotebookLM query tests pass,
formatting passes, and diff check has no whitespace errors except Git
line-ending warnings.

- [x] **Step 7: Commit archive export loader**

Run:

```powershell
git add src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: add archive notebooklm export loader"
```

Expected: commit succeeds.

---

### Task 3: Gated Wrapper And No-Silent-Fallback Contract

**Files:**
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Test: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Add failing wrapper gate tests**

Add:

```rust
#[tokio::test]
async fn notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states() {
    for status in [None, Some("stale"), Some("failed")] {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        if let Some(status) = status {
            seed_archive_state(
                &pool,
                status,
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            )
            .await;
        }

        let direct = load_export_messages_from_items_path(&pool, 1, Some(50), Some(125))
            .await
            .expect("load direct items path");
        let wrapped = load_export_messages(&pool, 1, Some(50), Some(125))
            .await
            .expect("load wrapped fallback");

        assert_eq!(wrapped, direct, "unexpected fallback result for {status:?}");
    }
}

#[tokio::test]
async fn notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection() {
    let pool = export_pool().await;
    seed_notebooklm_export_parity_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive model");

    sqlx::query("UPDATE items SET content_zstd = ? WHERE id = 1")
        .bind(compress_text("Canonical reply target should not be used").expect("compress old"))
        .execute(&pool)
        .await
        .expect("mutate canonical reply target");
    sqlx::query("UPDATE archive_read_items SET content_zstd = ? WHERE source_id = 1 AND item_id = 1")
        .bind(compress_text("Archive reply target wins").expect("compress archive"))
        .execute(&pool)
        .await
        .expect("mutate archive reply target");

    let messages = load_export_messages(&pool, 1, Some(50), Some(115))
        .await
        .expect("load wrapped archive path");

    assert_eq!(messages[0].reply_to_snippet.as_deref(), Some("Archive reply target wins"));
}

#[tokio::test]
async fn notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails() {
    let pool = export_pool().await;
    seed_notebooklm_export_parity_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive model");

    let direct = load_export_messages_from_items_path(&pool, 1, Some(50), Some(115))
        .await
        .expect("items path remains valid");
    assert!(!direct.is_empty());

    sqlx::query(
        "UPDATE archive_read_items
         SET content_zstd = X'00'
         WHERE source_id = 1 AND item_id = 2",
    )
    .execute(&pool)
    .await
    .expect("corrupt archive row");

    let error = load_export_messages(&pool, 1, Some(50), Some(115))
        .await
        .expect_err("archive decode failure is returned");

    assert!(
        error.to_string().contains("zstd") || error.to_string().contains("decode"),
        "unexpected archive error: {error}"
    );
}

#[tokio::test]
async fn corrupt_archive_reply_target_outside_period_fails_archive_loader() {
    let pool = export_pool().await;
    seed_notebooklm_export_parity_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive model");

    sqlx::query(
        "UPDATE archive_read_items
         SET content_zstd = X'00'
         WHERE source_id = 1 AND item_id = 1",
    )
    .execute(&pool)
    .await
    .expect("corrupt archive reply target");

    let error = load_export_messages(&pool, 1, Some(50), Some(115))
        .await
        .expect_err("corrupt reply target fails archive loader");

    assert!(
        error.to_string().contains("zstd") || error.to_string().contains("decode"),
        "unexpected archive reply error: {error}"
    );
}
```

- [x] **Step 2: Run wrapper tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader
```

Expected: at least the archive-selection tests fail because
`load_export_messages` still delegates directly to the items path from Task 1.

- [x] **Step 3: Wire the gated wrapper**

Replace the temporary `load_export_messages` wrapper with:

```rust
pub(crate) async fn load_export_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    match select_notebooklm_export_loader(pool, source_id).await? {
        ExportLoaderSelection::ArchiveReadModel { .. } => {
            load_export_messages_from_archive(pool, source_id, period_from, period_to).await
        }
        ExportLoaderSelection::ItemsPath { .. } => {
            load_export_messages_from_items_path(pool, source_id, period_from, period_to).await
        }
    }
}
```

Do not catch errors from `load_export_messages_from_archive` in this wrapper.

- [x] **Step 4: Run Task 3 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: wrapper tests pass, NotebookLM query tests pass, archive model tests
pass, formatting passes, and diff check has no whitespace errors except Git
line-ending warnings.

- [x] **Step 5: Commit gated wrapper**

Run:

```powershell
git add src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: gate notebooklm export on archive read model"
```

Expected: commit succeeds.

---

### Task 4: Documentation And Full Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/database-schema-read-model-decision.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-19-notebooklm-export-archive-loader.md`
- Test: full Rust and frontend verification

- [ ] **Step 1: Update schema docs**

In `docs/database-schema.md`, update the archive/read model notes so they say:

```markdown
- source browsing and Telegram NotebookLM export use the gated archive/read UI
  model when `archive_read_model_state` is ready and current;
- for missing, building, stale, failed, or old-version archive states,
  NotebookLM export preserves the existing local provider/archive items path;
- once NotebookLM export selects the archive loader, archive row decode and
  invariant failures are surfaced as errors rather than silently falling back.
```

- [ ] **Step 2: Update decision note and backlog**

In `docs/database-schema-read-model-decision.md`, update the follow-up section:

```markdown
4. [x] Migrate Telegram NotebookLM export after export parity tests pass.
5. [ ] Decide whether future YouTube playlist-entry browsing needs archive rows
   or typed detail only.
6. [ ] Consider a current-schema baseline after the read-model boundary
   settles.
```

Keep YouTube NotebookLM export enrichment described as future-facing.

In `docs/backlog.md`, change the Database schema simplification open list to:

```markdown
- [ ] decide whether future YouTube playlist-entry browsing needs archive rows or typed detail only
- [ ] consider current-schema baseline after archive read model boundary stabilizes
```

Do not remove separate NotebookLM export follow-ups for optional enrichment,
source-group export, or YouTube-specific export enrichment outside the Database
schema simplification slice.

- [ ] **Step 3: Mark plan checkboxes complete as work lands**

Before the docs commit, update this plan so completed steps use `[x]`. Do not
mark a step complete before its verification command has passed.

- [ ] **Step 4: Run containment scans**

Run:

```powershell
rg -n "load_export_messages_from_archive|select_notebooklm_export_loader|ArchiveReadinessFallbackReason|ExportLoaderSelection" src-tauri/src docs
rg -n "CAST\\(archive_read_items\\.external_id AS INTEGER\\)|CAST\\(external_id AS INTEGER\\)" src-tauri/src/notebooklm_export src-tauri/src/archive_read_model.rs
rg -n "youtube.*NotebookLM|playlist-entry|current-schema baseline|source-group export|link enrichment" src-tauri/src docs
rg -n "T[O]DO|T[B]D|FIX[M]E|todo!\\(" src-tauri/src docs
```

Expected:

- new loader symbols are limited to NotebookLM export query tests/docs;
- no archive reply lookup casts `external_id` to integer;
- YouTube export enrichment, playlist canonical cleanup, current-schema
  baseline, source-group export, and link enrichment were not implemented in
  this slice;
- no unfinished-work markers or Rust incomplete macro calls are introduced.

- [ ] **Step 5: Run targeted Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::tests::
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::
```

Expected: all targeted tests pass.

- [ ] **Step 6: Run full verification**

Run commands serially, not in parallel, because `npm.cmd run check` runs
`svelte-kit sync` and can interfere with concurrent Vitest module loading:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
npm.cmd test
npm.cmd run check
git status --short
```

Expected:

- Rust suite passes;
- formatting passes;
- diff check has no whitespace errors except Git line-ending warnings;
- Vitest passes;
- Svelte diagnostics report 0 errors and 0 warnings;
- `git status --short` shows only intended docs/plan changes before the docs
  commit.

- [ ] **Step 7: Commit docs and final plan state**

Run:

```powershell
git add docs/database-schema.md docs/database-schema-read-model-decision.md docs/backlog.md docs/superpowers/plans/2026-05-19-notebooklm-export-archive-loader.md
git commit -m "docs: document notebooklm archive export loader"
```

Expected: commit succeeds.

---

## Self-Review Checklist

- Source validation remains before message loading in `export_source_to_notebooklm`.
- Loader selection uses an enum with explicit fallback reasons.
- Non-ready/current states preserve the old items path.
- Ready/current state uses the archive loader once and does not silently
  fallback after archive decode or invariant failures.
- Archive export SQL filters `item_kind = 'telegram_message'`.
- Archive reply lookup uses `external_id IN` with `reply_to_msg_id.to_string()`
  values and does not cast archive external ids to integers.
- Archive reply snippets come from `archive_read_items`, not canonical `items`.
- Missing reply targets keep `"Original message unavailable"`.
- Corrupt reply target rows fail the archive loader.
- No YouTube playlist canonical cleanup, current-schema baseline, YouTube
  NotebookLM enrichment, source-group export, or link enrichment is included.
