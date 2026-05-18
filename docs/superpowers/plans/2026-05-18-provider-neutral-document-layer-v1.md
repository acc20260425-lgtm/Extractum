# Provider-Neutral Document Layer v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `analysis_documents` as a provider-neutral materialized read model for live analysis corpus loading.

**Architecture:** Create a focused `analysis_documents` Rust module that owns schema DDL, document materialization, source rebuilds, and low-level writer helpers. Register migration 24 only after full schema + backfill support exists, then wire runtime writers and switch only `load_corpus_messages` to the new table. Source browsing, source item APIs, NotebookLM export, and saved run snapshots stay on their current paths.

**Tech Stack:** Rust, SQLx SQLite, Tauri, `tauri_plugin_sql` migrations, zstd-backed text/json compression.

---

## File Structure

- Create `src-tauri/migrations/24.sql`: runner-managed sentinel SQL for migration 24.
- Create `src-tauri/src/migrations/analysis_documents.rs`: runner-managed migration 24 orchestration, checksum validation, transaction/restart-safety, and migration history recording.
- Create `src-tauri/src/analysis_documents.rs`: schema DDL, document metadata helpers, source rebuild/backfill, item-backed upserts, transcript segment rebuilds, and YouTube description upserts/deletes.
- Modify `src-tauri/src/lib.rs`: register `mod analysis_documents;`.
- Modify `src-tauri/src/migrations.rs`: register v24 sentinel, call migration 24 runner after regular v23, and update test migration helper behavior.
- Modify `src-tauri/src/sources/test_support.rs`: add an `analysis_documents` fixture helper.
- Modify `src-tauri/src/sources/items.rs`: write Telegram and YouTube comment item-backed documents inside existing writer transactions.
- Modify `src-tauri/src/youtube/captions.rs`: rebuild transcript segment documents after replacing segment rows inside the existing transcript transaction.
- Modify `src-tauri/src/youtube/source_metadata.rs`: maintain synthetic YouTube description documents when typed video metadata is upserted.
- Modify `src-tauri/src/sources/store.rs`: update YouTube source store tests to create `analysis_documents` schema when metadata upsert now writes description docs.
- Modify `src-tauri/src/analysis/corpus.rs`: switch `load_corpus_messages` to `analysis_documents`; keep saved snapshot readers on `analysis_run_messages`.
- Modify `docs/database-schema.md`, `docs/database-schema-legacy-analysis.md`, and `docs/backlog.md`: document shipped document layer and remaining follow-ups.

---

### Task 1: Analysis Documents Schema Module

**Files:**
- Create: `src-tauri/src/analysis_documents.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Test: `src-tauri/src/analysis_documents.rs`
- Test: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Register the module shell**

In `src-tauri/src/lib.rs`, add this near the other backend modules:

```rust
mod analysis_documents;
```

- [x] **Step 2: Add failing schema tests**

Create `src-tauri/src/analysis_documents.rs` with this initial test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;

    #[tokio::test]
    async fn schema_creates_analysis_documents_constraints_and_indexes() {
        let pool = memory_pool_with_source_items_and_topics().await;

        create_analysis_documents_schema(&pool)
            .await
            .expect("create analysis document schema");

        let table_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'analysis_documents'",
        )
        .fetch_one(&pool)
        .await
        .expect("check table");
        assert_eq!(table_exists, 1);

        for index in [
            "idx_analysis_documents_source_key",
            "idx_analysis_documents_source_published",
            "idx_analysis_documents_kind_source_published",
            "idx_analysis_documents_ref",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 'tg1', 'Telegram', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (10, 1, '10', 'telegram_message', 'alice', 100, 100, 'text_only', 0, x'01')",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        sqlx::query(
            "INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind, source_type,
                source_subtype, external_id, author, published_at, document_order,
                ref, content_zstd, created_at, updated_at
             ) VALUES (
                1, 10, 'item:10', 'telegram_message', 'telegram',
                'supergroup', '10', 'alice', 100, 10,
                's1-i10', x'01', 100, 100
             )",
        )
        .execute(&pool)
        .await
        .expect("valid item-backed document");

        let invalid_synthetic = sqlx::query(
            "INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind, source_type,
                source_subtype, external_id, published_at, document_order,
                ref, content_zstd, created_at, updated_at
             ) VALUES (
                1, 10, 'youtube:description', 'youtube_description', 'youtube',
                'video', 'description:v1', 100, -1,
                's1-i0', x'01', 100, 100
             )",
        )
        .execute(&pool)
        .await;
        assert!(invalid_synthetic.is_err());
    }
}
```

- [x] **Step 3: Run schema tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes
```

Expected: compile failure because `create_analysis_documents_schema` does not exist.

- [x] **Step 4: Implement schema DDL helpers**

Add this production code to `src-tauri/src/analysis_documents.rs` above the tests:

```rust
use sqlx::{Executor, Sqlite};

use crate::error::{AppError, AppResult};

pub(crate) const ANALYSIS_DOCUMENTS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS analysis_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    item_id INTEGER REFERENCES items(id) ON DELETE CASCADE,

    document_key TEXT NOT NULL,
    document_kind TEXT NOT NULL,

    source_type TEXT NOT NULL,
    source_subtype TEXT,
    external_id TEXT NOT NULL,

    author TEXT,
    published_at INTEGER NOT NULL,
    document_order INTEGER NOT NULL DEFAULT 0,

    ref TEXT NOT NULL,
    content_zstd BLOB NOT NULL,
    metadata_zstd BLOB,

    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    CHECK (document_kind IN (
        'telegram_message',
        'youtube_transcript',
        'youtube_comment',
        'youtube_description'
    )),
    CHECK (source_type IN ('telegram', 'youtube')),
    CHECK (
        (document_kind = 'telegram_message' AND source_type = 'telegram')
        OR
        (document_kind IN (
            'youtube_transcript',
            'youtube_comment',
            'youtube_description'
        ) AND source_type = 'youtube')
    ),
    CHECK (
        (source_type = 'telegram'
            AND COALESCE(source_subtype, '')
                IN ('channel', 'supergroup', 'group'))
        OR
        (source_type = 'youtube' AND COALESCE(source_subtype, '') = 'video')
    ),
    CHECK (
        (document_kind IN (
            'telegram_message',
            'youtube_transcript',
            'youtube_comment'
        ) AND item_id IS NOT NULL)
        OR
        (document_kind = 'youtube_description' AND item_id IS NULL)
    ),
    CHECK (
        (document_kind IN (
            'telegram_message',
            'youtube_transcript',
            'youtube_comment'
        ) AND document_key LIKE 'item:%')
        OR
        (document_kind = 'youtube_description'
            AND document_key = 'youtube:description')
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_analysis_documents_source_key
ON analysis_documents(source_id, document_key);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_source_published
ON analysis_documents(source_id, published_at, document_order, id);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_kind_source_published
ON analysis_documents(document_kind, source_id, published_at, document_order, id);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_ref
ON analysis_documents(ref);
"#;

pub(crate) async fn create_analysis_documents_schema<'e, E>(executor: E) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::raw_sql(ANALYSIS_DOCUMENTS_SCHEMA_SQL)
        .execute(executor)
        .await
        .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 5: Add test fixture helper**

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_analysis_documents_table(pool: &sqlx::SqlitePool) {
    crate::analysis_documents::create_analysis_documents_schema(pool)
        .await
        .expect("create analysis documents schema");
}
```

Update `source_fixture_creates_expected_tables`:

```rust
use super::{
    create_analysis_documents_table, create_canonical_telegram_identity_index,
    create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
};
```

Then call the helper before assertions:

```rust
create_analysis_documents_table(&pool).await;
```

And include `"analysis_documents"` in the table assertion list.

- [x] **Step 6: Run schema and fixture tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_documents::tests::
```

Because Cargo accepts only one test filter, run the fixture test separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::test_support::tests::source_fixture_creates_expected_tables
```

Expected: all selected tests pass.

- [x] **Step 7: Commit schema module**

Run:

```powershell
git add src-tauri/src/analysis_documents.rs src-tauri/src/lib.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: add analysis documents schema helpers"
```

Expected: commit succeeds.

---

### Task 2: Document Rebuild And Backfill Helpers

**Files:**
- Modify: `src-tauri/src/analysis_documents.rs`
- Test: `src-tauri/src/analysis_documents.rs`

- [x] **Step 1: Add failing rebuild/backfill tests**

Add these tests inside `#[cfg(test)] mod tests` in `src-tauri/src/analysis_documents.rs`:

```rust
use crate::compression::{compress_text, decompress_text, decompress_bytes};
use serde_json::Value;

async fn seed_sources(pool: &sqlx::SqlitePool) {
    crate::sources::test_support::create_youtube_typed_source_tables(pool).await;
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES
            (1, 'telegram', 'supergroup', 'tg1', 'Telegram', 1, 1, 1),
            (2, 'youtube', 'video', 'video2', 'Video 2', 1, 1, 1)",
    )
    .execute(pool)
    .await
    .expect("seed sources");
    sqlx::query(
        "INSERT INTO youtube_video_sources (
            source_id, video_id, canonical_url, title, channel_title, channel_handle,
            published_at, description, video_form, availability_status
         ) VALUES (
            2, 'video2', 'https://www.youtube.com/watch?v=video2',
            'Video 2', 'Channel', '@channel', '2026-05-01',
            'Description body', 'regular', 'available'
         )",
    )
    .execute(pool)
    .await
    .expect("seed youtube metadata");
}

async fn seed_text_item(
    pool: &sqlx::SqlitePool,
    id: i64,
    source_id: i64,
    external_id: &str,
    item_kind: &str,
    published_at: i64,
    text: &str,
) {
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at,
            ingested_at, content_kind, has_media, content_zstd
         ) VALUES (?, ?, ?, ?, 'Author', ?, ?, 'text_only', 0, ?)",
    )
    .bind(id)
    .bind(source_id)
    .bind(external_id)
    .bind(item_kind)
    .bind(published_at)
    .bind(published_at)
    .bind(compress_text(text).expect("compress text"))
    .execute(pool)
    .await
    .expect("seed item");
}

async fn seed_segment(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    source_id: i64,
    segment_index: i64,
    start_ms: i64,
    text: &str,
) {
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (
            item_id, source_id, segment_index, start_ms, end_ms, text,
            caption_language, caption_track_kind, is_auto_generated
         ) VALUES (?, ?, ?, ?, ?, ?, 'en', 'manual', 0)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(segment_index)
    .bind(start_ms)
    .bind(start_ms + 1_000)
    .bind(text)
    .execute(pool)
    .await
    .expect("seed segment");
}

#[tokio::test]
async fn rebuild_source_materializes_text_units_with_document_order() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_schema(&pool).await.expect("schema");
    seed_sources(&pool).await;
    seed_text_item(&pool, 10, 1, "10", "telegram_message", 1_700_000_000, "Telegram text").await;
    seed_text_item(
        &pool,
        20,
        2,
        "transcript:video2:en:manual",
        "youtube_transcript",
        1_700_000_000,
        "full transcript",
    )
    .await;
    seed_text_item(&pool, 30, 2, "comment:c1", "youtube_comment", 1_700_000_001, "Comment").await;
    seed_segment(&pool, 20, 2, 0, 900, "early").await;
    seed_segment(&pool, 20, 2, 1, 10_000, "late").await;

    rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild telegram");
    rebuild_analysis_documents_for_source(&pool, 2)
        .await
        .expect("rebuild youtube");

    let rows: Vec<(String, String, i64, String, Option<i64>)> = sqlx::query_as(
        "SELECT document_kind, ref, document_order, external_id, item_id
         FROM analysis_documents
         ORDER BY source_id, published_at, document_order, id",
    )
    .fetch_all(&pool)
    .await
    .expect("load docs");

    assert_eq!(
        rows,
        vec![
            (
                "telegram_message".to_string(),
                "s1-i10".to_string(),
                10,
                "10".to_string(),
                Some(10)
            ),
            (
                "youtube_description".to_string(),
                "s2-i0".to_string(),
                -1,
                "description:video2".to_string(),
                None
            ),
            (
                "youtube_transcript".to_string(),
                "s2-i20@900ms".to_string(),
                0,
                "transcript:video2:en:manual".to_string(),
                Some(20)
            ),
            (
                "youtube_transcript".to_string(),
                "s2-i20@10000ms".to_string(),
                1,
                "transcript:video2:en:manual".to_string(),
                Some(20)
            ),
            (
                "youtube_comment".to_string(),
                "s2-i30".to_string(),
                30,
                "comment:c1".to_string(),
                Some(30)
            ),
        ]
    );

    let content: Vec<String> = sqlx::query_scalar("SELECT content_zstd FROM analysis_documents ORDER BY source_id, published_at, document_order, id")
        .fetch_all(&pool)
        .await
        .expect("load content")
        .into_iter()
        .map(|bytes: Vec<u8>| decompress_text(&bytes).expect("decompress document"))
        .collect();
    assert_eq!(content[0], "Telegram text");
    assert_eq!(content[2], "early");
    assert_eq!(content[3], "late");
}

#[tokio::test]
async fn rebuild_source_removes_stale_documents_and_is_idempotent() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_schema(&pool).await.expect("schema");
    seed_sources(&pool).await;
    seed_text_item(&pool, 10, 1, "10", "telegram_message", 100, "First").await;

    rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("first rebuild");
    rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("second rebuild");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1")
        .fetch_one(&pool)
        .await
        .expect("count docs");
    assert_eq!(count, 1);

    sqlx::query("UPDATE items SET content_zstd = NULL WHERE id = 10")
        .execute(&pool)
        .await
        .expect("clear content");
    rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("third rebuild");

    let count_after_delete: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1")
            .fetch_one(&pool)
            .await
            .expect("count docs after delete");
    assert_eq!(count_after_delete, 0);
}

#[tokio::test]
async fn document_metadata_envelopes_match_current_evidence_shape() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_schema(&pool).await.expect("schema");
    seed_sources(&pool).await;
    seed_text_item(
        &pool,
        20,
        2,
        "transcript:video2:en:manual",
        "youtube_transcript",
        1_700_000_000,
        "full transcript",
    )
    .await;
    seed_segment(&pool, 20, 2, 0, 900, "segment").await;

    rebuild_analysis_documents_for_source(&pool, 2)
        .await
        .expect("rebuild youtube");

    let metadata_rows: Vec<Vec<u8>> = sqlx::query_scalar(
        "SELECT metadata_zstd FROM analysis_documents
         WHERE document_kind IN ('youtube_transcript', 'youtube_description')
         ORDER BY document_kind",
    )
    .fetch_all(&pool)
    .await
    .expect("load metadata");
    assert_eq!(metadata_rows.len(), 2);

    let decoded = metadata_rows
        .iter()
        .map(|bytes| serde_json::from_slice::<Value>(&decompress_bytes(bytes).expect("decompress json")).expect("json"))
        .collect::<Vec<_>>();
    assert!(decoded.iter().any(|value| value["item_kind"] == "youtube_description"));
    assert!(decoded.iter().any(|value| {
        value["item_kind"] == "youtube_transcript"
            && value["segment_start_ms"] == 900
            && value["canonical_url"] == "https://www.youtube.com/watch?v=video2"
    }));
}
```

- [x] **Step 2: Run rebuild tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_documents::tests::rebuild_source_
```

Because Cargo accepts only one test filter, run the metadata test separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape
```

Expected: compile failure for missing rebuild/backfill functions.

- [x] **Step 3: Implement document constants and metadata helpers**

Add these constants and helper functions in `src-tauri/src/analysis_documents.rs`:

```rust
use crate::compression::{compress_json_bytes, compress_text};

pub(crate) const DOCUMENT_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";
pub(crate) const DOCUMENT_KIND_YOUTUBE_TRANSCRIPT: &str = "youtube_transcript";
pub(crate) const DOCUMENT_KIND_YOUTUBE_COMMENT: &str = "youtube_comment";
pub(crate) const DOCUMENT_KIND_YOUTUBE_DESCRIPTION: &str = "youtube_description";
pub(crate) const YOUTUBE_DESCRIPTION_DOCUMENT_KEY: &str = "youtube:description";
pub(crate) const ANALYSIS_METADATA_VERSION: i64 = 1;

pub(crate) fn live_item_ref(source_id: i64, item_id: i64) -> String {
    format!("s{source_id}-i{item_id}")
}

pub(crate) fn transcript_segment_ref(source_id: i64, item_id: i64, start_ms: i64) -> String {
    format!("s{source_id}-i{item_id}@{start_ms}ms")
}

pub(crate) fn youtube_description_ref(source_id: i64) -> String {
    format!("s{source_id}-i0")
}

pub(crate) fn ymd_to_unix_midnight(value: &str) -> Option<i64> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn youtube_segment_metadata_zstd(row: &YoutubeTranscriptDocumentRow) -> AppResult<Vec<u8>> {
    let video_id = row
        .typed_video_id
        .as_deref()
        .unwrap_or(row.source_external_id.as_str());
    let canonical_url = row
        .typed_canonical_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={video_id}"));
    let title = row.typed_title.as_deref().or(row.source_title.as_deref());
    let metadata = serde_json::json!({
        "metadata_version": ANALYSIS_METADATA_VERSION,
        "video_id": video_id,
        "canonical_url": canonical_url,
        "title": title,
        "channel_title": &row.typed_channel_title,
        "channel_handle": &row.typed_channel_handle,
        "caption_language": &row.caption_language,
        "caption_track_kind": &row.caption_track_kind,
        "segment_start_ms": row.start_ms,
        "segment_end_ms": row.end_ms,
        "item_kind": DOCUMENT_KIND_YOUTUBE_TRANSCRIPT,
    });
    let json = serde_json::to_vec(&metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

fn youtube_description_content(row: &YoutubeDescriptionDocumentRow) -> String {
    let title = row.title.clone().unwrap_or_else(|| row.video_id.clone());
    let channel = row
        .channel_title
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let description = row.description.as_deref().unwrap_or_default().trim();
    format!(
        "YouTube video description\nTitle: {title}\nChannel: {channel}\nURL: {url}\n\n{description}",
        url = row.canonical_url,
    )
}

fn youtube_description_metadata_zstd(row: &YoutubeDescriptionDocumentRow) -> AppResult<Vec<u8>> {
    let metadata = serde_json::json!({
        "metadata_version": ANALYSIS_METADATA_VERSION,
        "video_id": &row.video_id,
        "canonical_url": &row.canonical_url,
        "title": &row.title,
        "channel_title": &row.channel_title,
        "channel_handle": &row.channel_handle,
        "item_kind": DOCUMENT_KIND_YOUTUBE_DESCRIPTION,
    });
    let json = serde_json::to_vec(&metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}
```

- [x] **Step 4: Implement row structs and source rebuild**

Add the row structs and public rebuild APIs:

```rust
#[derive(sqlx::FromRow)]
struct ItemDocumentRow {
    id: i64,
    source_id: i64,
    external_id: String,
    item_kind: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Vec<u8>,
    source_type: String,
    source_subtype: Option<String>,
}

#[derive(sqlx::FromRow)]
struct YoutubeTranscriptDocumentRow {
    item_id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    source_external_id: String,
    source_title: Option<String>,
    typed_video_id: Option<String>,
    typed_canonical_url: Option<String>,
    typed_title: Option<String>,
    typed_channel_title: Option<String>,
    typed_channel_handle: Option<String>,
    segment_index: i64,
    start_ms: i64,
    end_ms: Option<i64>,
    text: String,
    caption_language: Option<String>,
    caption_track_kind: Option<String>,
}

#[derive(sqlx::FromRow)]
struct YoutubeDescriptionDocumentRow {
    source_id: i64,
    video_id: String,
    canonical_url: String,
    title: Option<String>,
    channel_title: Option<String>,
    channel_handle: Option<String>,
    published_at: Option<String>,
    description: Option<String>,
}

pub(crate) async fn rebuild_analysis_documents_for_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    rebuild_analysis_documents_for_source_on_connection(&mut *tx, source_id).await?;
    tx.commit().await.map_err(AppError::database)
}

pub(crate) async fn rebuild_analysis_documents_for_source_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query("DELETE FROM analysis_documents WHERE source_id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    insert_item_backed_documents_for_source(conn, source_id).await?;
    insert_youtube_transcript_documents_for_source(conn, source_id).await?;
    upsert_youtube_description_document_on_connection(conn, source_id).await
}

pub(crate) async fn backfill_all_analysis_documents_on_connection(
    conn: &mut sqlx::SqliteConnection,
) -> AppResult<()> {
    let source_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM sources WHERE source_type IN ('telegram', 'youtube') ORDER BY id",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for source_id in source_ids {
        rebuild_analysis_documents_for_source_on_connection(conn, source_id).await?;
    }
    Ok(())
}
```

- [x] **Step 5: Implement item-backed document insertion**

Add this helper:

```rust
async fn insert_item_backed_documents_for_source(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    let rows: Vec<ItemDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.item_kind,
            items.author,
            items.published_at,
            items.content_zstd,
            sources.source_type,
            sources.source_subtype
        FROM items
        JOIN sources ON sources.id = items.source_id
        WHERE items.source_id = ?
          AND items.content_zstd IS NOT NULL
          AND items.content_kind IN ('text_only', 'text_with_media')
          AND items.item_kind IN ('telegram_message', 'youtube_comment')
        ORDER BY items.id
        "#,
    )
    .bind(source_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        let document_kind = match row.item_kind.as_str() {
            "telegram_message" => DOCUMENT_KIND_TELEGRAM_MESSAGE,
            "youtube_comment" => DOCUMENT_KIND_YOUTUBE_COMMENT,
            _ => continue,
        };
        sqlx::query(
            r#"
            INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind,
                source_type, source_subtype, external_id, author,
                published_at, document_order, ref, content_zstd,
                metadata_zstd, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, strftime('%s','now'), strftime('%s','now'))
            ON CONFLICT(source_id, document_key) DO UPDATE SET
                item_id = excluded.item_id,
                document_kind = excluded.document_kind,
                source_type = excluded.source_type,
                source_subtype = excluded.source_subtype,
                external_id = excluded.external_id,
                author = excluded.author,
                published_at = excluded.published_at,
                document_order = excluded.document_order,
                ref = excluded.ref,
                content_zstd = excluded.content_zstd,
                metadata_zstd = excluded.metadata_zstd,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(row.source_id)
        .bind(row.id)
        .bind(format!("item:{}", row.id))
        .bind(document_kind)
        .bind(&row.source_type)
        .bind(&row.source_subtype)
        .bind(&row.external_id)
        .bind(&row.author)
        .bind(row.published_at)
        .bind(row.id)
        .bind(live_item_ref(row.source_id, row.id))
        .bind(row.content_zstd)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    }
    Ok(())
}
```

- [x] **Step 6: Implement transcript and description document insertion**

Add transcript insertion:

```rust
async fn insert_youtube_transcript_documents_for_source(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    let rows: Vec<YoutubeTranscriptDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id AS item_id,
            items.source_id,
            items.external_id,
            items.author,
            items.published_at,
            sources.external_id AS source_external_id,
            sources.title AS source_title,
            yvs.video_id AS typed_video_id,
            yvs.canonical_url AS typed_canonical_url,
            yvs.title AS typed_title,
            yvs.channel_title AS typed_channel_title,
            yvs.channel_handle AS typed_channel_handle,
            segments.segment_index,
            segments.start_ms,
            segments.end_ms,
            segments.text,
            segments.caption_language,
            segments.caption_track_kind
        FROM items
        JOIN sources ON sources.id = items.source_id
        JOIN youtube_transcript_segments segments ON segments.item_id = items.id
        LEFT JOIN youtube_video_sources yvs ON yvs.source_id = sources.id
        WHERE items.source_id = ?
          AND items.item_kind = 'youtube_transcript'
          AND segments.text IS NOT NULL
        ORDER BY items.id ASC, segments.segment_index ASC
        "#,
    )
    .bind(source_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        let content_zstd = compress_text(&row.text).map_err(AppError::internal)?;
        let metadata_zstd = youtube_segment_metadata_zstd(&row)?;
        sqlx::query(
            r#"
            INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind,
                source_type, source_subtype, external_id, author,
                published_at, document_order, ref, content_zstd,
                metadata_zstd, created_at, updated_at
            )
            VALUES (?, ?, ?, 'youtube_transcript', 'youtube', 'video', ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), strftime('%s','now'))
            ON CONFLICT(source_id, document_key) DO UPDATE SET
                item_id = excluded.item_id,
                document_kind = excluded.document_kind,
                source_type = excluded.source_type,
                source_subtype = excluded.source_subtype,
                external_id = excluded.external_id,
                author = excluded.author,
                published_at = excluded.published_at,
                document_order = excluded.document_order,
                ref = excluded.ref,
                content_zstd = excluded.content_zstd,
                metadata_zstd = excluded.metadata_zstd,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(row.source_id)
        .bind(row.item_id)
        .bind(format!("item:{}:segment:{}", row.item_id, row.segment_index))
        .bind(&row.external_id)
        .bind(&row.author)
        .bind(row.published_at)
        .bind(row.segment_index)
        .bind(transcript_segment_ref(row.source_id, row.item_id, row.start_ms))
        .bind(content_zstd)
        .bind(metadata_zstd)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    }
    Ok(())
}
```

Add description upsert/delete:

```rust
pub(crate) async fn upsert_youtube_description_document_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    let row: Option<YoutubeDescriptionDocumentRow> = sqlx::query_as(
        r#"
        SELECT source_id, video_id, canonical_url, title, channel_title,
               channel_handle, published_at, description
        FROM youtube_video_sources
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let Some(row) = row else {
        delete_youtube_description_document_on_connection(conn, source_id).await?;
        return Ok(());
    };
    let Some(description) = row.description.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        delete_youtube_description_document_on_connection(conn, source_id).await?;
        return Ok(());
    };
    let Some(published_at) = row.published_at.as_deref().and_then(ymd_to_unix_midnight) else {
        delete_youtube_description_document_on_connection(conn, source_id).await?;
        return Ok(());
    };

    let mut materialized = row;
    materialized.description = Some(description.to_string());
    let content_zstd = compress_text(&youtube_description_content(&materialized))
        .map_err(AppError::internal)?;
    let metadata_zstd = youtube_description_metadata_zstd(&materialized)?;

    sqlx::query(
        r#"
        INSERT INTO analysis_documents (
            source_id, item_id, document_key, document_kind,
            source_type, source_subtype, external_id, author,
            published_at, document_order, ref, content_zstd,
            metadata_zstd, created_at, updated_at
        )
        VALUES (?, NULL, 'youtube:description', 'youtube_description', 'youtube', 'video', ?, ?, ?, -1, ?, ?, ?, strftime('%s','now'), strftime('%s','now'))
        ON CONFLICT(source_id, document_key) DO UPDATE SET
            item_id = excluded.item_id,
            document_kind = excluded.document_kind,
            source_type = excluded.source_type,
            source_subtype = excluded.source_subtype,
            external_id = excluded.external_id,
            author = excluded.author,
            published_at = excluded.published_at,
            document_order = excluded.document_order,
            ref = excluded.ref,
            content_zstd = excluded.content_zstd,
            metadata_zstd = excluded.metadata_zstd,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(materialized.source_id)
    .bind(format!("description:{}", materialized.video_id))
    .bind(&materialized.channel_title)
    .bind(published_at)
    .bind(youtube_description_ref(materialized.source_id))
    .bind(content_zstd)
    .bind(metadata_zstd)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn delete_youtube_description_document_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM analysis_documents
         WHERE source_id = ? AND document_key = 'youtube:description'",
    )
    .bind(source_id)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 7: Run rebuild/backfill tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_documents::tests::
```

Expected: all `analysis_documents` tests pass.

- [x] **Step 8: Commit rebuild helpers**

Run:

```powershell
git add src-tauri/src/analysis_documents.rs
git commit -m "feat: add analysis document rebuild helpers"
```

Expected: commit succeeds.

---

### Task 3: Runner-Managed Migration 24

**Files:**
- Create: `src-tauri/migrations/24.sql`
- Create: `src-tauri/src/migrations/analysis_documents.rs`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations/analysis_documents.rs`

- [x] **Step 1: Write failing migration registration tests**

In `src-tauri/src/migrations.rs`, add this test near the other migration registration tests:

```rust
#[test]
fn includes_runner_managed_analysis_documents_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 24)
        .expect("version 24 migration is registered");

    assert_eq!(migration.description, "add provider neutral analysis documents");
    assert!(migration.sql.contains("runner-managed"));
    assert!(migration.sql.contains("analysis_documents"));
    assert!(!migration.sql.contains("CREATE TABLE analysis_documents"));
}
```

Update `build_migrations_contains_all_versions_for_sqlx_validation`:

```rust
assert_eq!(versions, (1_i64..=24_i64).collect::<Vec<_>>());
```

- [x] **Step 2: Write failing runner/backfill tests**

Create `src-tauri/src/migrations/analysis_documents.rs` with this initial test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::{compress_text, decompress_text};
    use crate::migrations::build_migrations;
    use sha2::{Digest, Sha384};
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn migration_24_creates_schema_backfills_and_records_sentinel() {
        let mut conn = memory_conn_with_history_through_23().await;
        seed_source_and_item(&mut conn).await;

        apply_analysis_documents_on_connection(&mut conn)
            .await
            .expect("apply v24");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 24",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v24 history");
        assert_eq!(row.0, ANALYSIS_DOCUMENTS_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_analysis_documents_checksum());

        let doc: (String, Vec<u8>) = sqlx::query_as(
            "SELECT ref, content_zstd FROM analysis_documents WHERE source_id = 1",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load doc");
        assert_eq!(doc.0, "s1-i10");
        assert_eq!(decompress_text(&doc.1).expect("decompress"), "Telegram text");
    }

    #[tokio::test]
    async fn migration_24_is_restart_safe_when_schema_exists_but_version_is_unrecorded() {
        let mut conn = memory_conn_with_history_through_23().await;
        seed_source_and_item(&mut conn).await;
        crate::analysis_documents::create_analysis_documents_schema(&mut conn)
            .await
            .expect("precreate schema");

        apply_analysis_documents_on_connection(&mut conn)
            .await
            .expect("apply v24 after partial schema");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count docs");
        assert_eq!(count, 1);

        apply_analysis_documents_on_connection(&mut conn)
            .await
            .expect("second v24");
        let migration_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 24")
                .fetch_one(&mut conn)
                .await
                .expect("count v24");
        assert_eq!(migration_count, 1);
    }

    async fn memory_conn_with_history_through_23() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create migration history");

        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version < 19)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut conn)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
            )
            .bind(migration.version)
            .bind(migration.description)
            .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
            .execute(&mut conn)
            .await
            .expect("record standard migration");
        }

        crate::migrations::source_identity_cleanup::apply_source_identity_cleanup_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v19");
        crate::migrations::youtube_typed_source_metadata::apply_youtube_typed_source_metadata_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v20");
        crate::migrations::telegram_item_native_identity::apply_telegram_item_native_identity_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v21");
        crate::migrations::topic_membership_materialization::apply_topic_membership_materialization_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v22");
        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version == 23)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut conn)
                .await
                .expect("apply v23");
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
            )
            .bind(migration.version)
            .bind(migration.description)
            .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
            .execute(&mut conn)
            .await
            .expect("record v23");
        }
        conn
    }

    async fn seed_source_and_item(conn: &mut SqliteConnection) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 'tg1', 'Telegram', 1, 1, 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (10, 1, '10', 'telegram_message', 'alice', 100, 100, 'text_only', 0, ?)",
        )
        .bind(compress_text("Telegram text").expect("compress"))
        .execute(&mut *conn)
        .await
        .expect("seed item");
    }
}
```

- [x] **Step 3: Run migration tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_analysis_documents_migration
```

Because Cargo accepts only one test filter, run the runner tests separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::analysis_documents::tests::
```

Expected: compile failure because the migration module and v24 registration do not exist.

- [x] **Step 4: Add v24 sentinel SQL**

Create `src-tauri/migrations/24.sql`:

```sql
-- Version 24 is runner-managed by src-tauri/src/migrations/analysis_documents.rs.
-- The runner creates analysis_documents and backfills it from provider/archive truth
-- before recording this sentinel migration as successful.
```

- [x] **Step 5: Implement runner-managed migration module**

At the top of `src-tauri/src/migrations/analysis_documents.rs`, add:

```rust
use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const ANALYSIS_DOCUMENTS_VERSION: i64 = 24;
pub(super) const ANALYSIS_DOCUMENTS_DESCRIPTION: &str =
    "add provider neutral analysis documents";
pub(super) const ANALYSIS_DOCUMENTS_SENTINEL_SQL: &str =
    include_str!("../../migrations/24.sql");

pub(super) async fn apply_analysis_documents_if_needed(db_url: &str) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_analysis_documents_on_connection(&mut conn).await
}

pub(super) async fn apply_analysis_documents_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_24_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        crate::analysis_documents::create_analysis_documents_schema(&mut *conn).await?;
        crate::analysis_documents::backfill_all_analysis_documents_on_connection(&mut *conn).await
    }
    .await;

    match result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            return Err(error);
        }
    }

    record_migration_success(
        conn,
        expected_analysis_documents_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 23 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "Analysis documents migration 24 requires migration 23",
        ));
    }
    Ok(())
}

async fn migration_24_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_analysis_documents_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(ANALYSIS_DOCUMENTS_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 24 checksum does not match the runner-managed analysis documents sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 24 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
         VALUES (?, ?, 1, ?, ?)",
    )
    .bind(ANALYSIS_DOCUMENTS_VERSION)
    .bind(ANALYSIS_DOCUMENTS_DESCRIPTION)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_analysis_documents_checksum() -> Vec<u8> {
    Sha384::digest(ANALYSIS_DOCUMENTS_SENTINEL_SQL.as_bytes()).to_vec()
}
```

- [x] **Step 6: Register migration 24 in migrations.rs**

In `src-tauri/src/migrations.rs`, add the module:

```rust
pub(crate) mod analysis_documents;
```

In `patch_migrations`, call v24 after v22 and after applying regular SQL
migrations between runner-managed v22 and runner-managed v24. This is required
because `prepare_database()` runs before the SQL plugin, so the plugin cannot
be the component that records v23 before the v24 runner checks for it.

```rust
topic_membership_materialization::apply_topic_membership_materialization_if_needed(&url).await?;
apply_regular_sql_migrations_before_runner(&url, 22, 24).await?;
analysis_documents::apply_analysis_documents_if_needed(&url).await
```

If `patch_migrations` currently returns the v22 expression directly, convert the tail into the block above.

Add this helper in `src-tauri/src/migrations.rs` near `patch_migrations`:

```rust
async fn apply_regular_sql_migrations_before_runner(
    db_url: &str,
    after_version: i64,
    before_version: i64,
) -> crate::error::AppResult<()> {
    let mut conn = sqlx::SqliteConnection::connect(db_url)
        .await
        .map_err(crate::error::AppError::database)?;
    source_identity_cleanup::ensure_sqlx_migrations_table_for_runner(&mut conn).await?;

    for migration in build_migrations()
        .into_iter()
        .filter(|migration| {
            migration.version > after_version && migration.version < before_version
        })
    {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ? AND success = 1",
        )
        .bind(migration.version)
        .fetch_one(&mut conn)
        .await
        .map_err(crate::error::AppError::database)?;
        if exists != 0 {
            continue;
        }

        let started_at = std::time::Instant::now();
        sqlx::raw_sql(migration.sql)
            .execute(&mut conn)
            .await
            .map_err(crate::error::AppError::database)?;
        sqlx::query(
            "INSERT INTO _sqlx_migrations (
                version, description, success, checksum, execution_time
             ) VALUES (?, ?, 1, ?, ?)",
        )
        .bind(migration.version)
        .bind(migration.description)
        .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
        .bind(started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64)
        .execute(&mut conn)
        .await
        .map_err(crate::error::AppError::database)?;
    }

    Ok(())
}
```

Expose the existing `_sqlx_migrations` table helper from
`source_identity_cleanup.rs` so this code can reuse it:

```rust
pub(super) async fn ensure_sqlx_migrations_table_for_runner(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_sqlx_migrations_table(conn).await
}
```

Append v24 to `build_migrations()`:

```rust
Migration {
    version: 24,
    description: "add provider neutral analysis documents",
    sql: include_str!("../migrations/24.sql"),
    kind: MigrationKind::Up,
},
```

Update `apply_all_migrations_for_test_pool` so regular SQL migrations above v22 and below v24 run before the runner, then v24 runs, then any future regular SQL migrations above v24 run:

```rust
for migration in build_migrations()
    .into_iter()
    .filter(|migration| migration.version > 22 && migration.version < 24)
{
    sqlx::raw_sql(migration.sql)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::AppError::database)?;
}

analysis_documents::apply_analysis_documents_on_connection(conn).await?;

for migration in build_migrations()
    .into_iter()
    .filter(|migration| migration.version > 24)
{
    sqlx::raw_sql(migration.sql)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::AppError::database)?;
}
```

- [x] **Step 7: Update fresh-schema test**

In `fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints`, either rename the test to mention both v23/v24 or add a new async test:

```rust
#[tokio::test]
async fn fresh_schema_includes_analysis_documents_table_indexes_and_constraints() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    let table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'analysis_documents'",
    )
    .fetch_one(&pool)
    .await
    .expect("check table");
    assert_eq!(table_exists, 1);

    for index in [
        "idx_analysis_documents_source_key",
        "idx_analysis_documents_source_published",
        "idx_analysis_documents_kind_source_published",
        "idx_analysis_documents_ref",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
        )
        .bind(index)
        .fetch_one(&pool)
        .await
        .expect("check index");
        assert_eq!(exists, 1, "missing index {index}");
    }
}
```

- [x] **Step 8: Run migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::
```

Because Cargo accepts only one test filter, run the runner tests separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::analysis_documents::tests::
```

Expected: all migration tests pass.

- [x] **Step 9: Commit migration 24**

Run:

```powershell
git add src-tauri/migrations/24.sql src-tauri/src/migrations.rs src-tauri/src/migrations/analysis_documents.rs
git commit -m "feat: add analysis documents migration"
```

Expected: commit succeeds.

---

### Task 4: Runtime Maintenance For Item-Backed Documents

**Files:**
- Modify: `src-tauri/src/analysis_documents.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/youtube/captions.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Test: `src-tauri/src/sources/items.rs`
- Test: `src-tauri/src/youtube/captions.rs`

- [ ] **Step 1: Add failing Telegram and YouTube item-backed tests**

In `src-tauri/src/sources/items.rs`, update the test imports to include:

```rust
create_analysis_documents_table,
```

Add this test:

```rust
#[tokio::test]
async fn telegram_insert_writes_analysis_document_in_same_writer_transaction() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_table(&pool).await;
    seed_item_source(&pool, 1).await;

    let outcome = insert_telegram_source_item_outcome(
        &pool,
        1,
        telegram_identity(42),
        telegram_insert("42", "Document text"),
    )
    .await
    .expect("insert telegram item");

    let TelegramItemInsertOutcome::Inserted { item_id } = outcome else {
        panic!("expected insert");
    };

    let row: (String, String, i64, String, String) = sqlx::query_as(
        "SELECT document_kind, ref, document_order, source_type, source_subtype
         FROM analysis_documents WHERE item_id = ?",
    )
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("load document");
    assert_eq!(
        row,
        (
            "telegram_message".to_string(),
            format!("s1-i{item_id}"),
            item_id,
            "telegram".to_string(),
            "supergroup".to_string(),
        )
    );
}

#[tokio::test]
async fn youtube_comment_upsert_writes_analysis_document_and_updates_content() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_table(&pool).await;
    seed_youtube_video_source(&pool, 2).await;

    let mut tx = pool.begin().await.expect("begin tx");
    let first = upsert_youtube_comment_item(&mut tx, 2, &youtube_comment("c1", "First"))
        .await
        .expect("first comment");
    tx.commit().await.expect("commit first");

    let content: Vec<u8> =
        sqlx::query_scalar("SELECT content_zstd FROM analysis_documents WHERE item_id = ?")
            .bind(first)
            .fetch_one(&pool)
            .await
            .expect("load first document");
    assert_eq!(
        decompress_text(&content).expect("decompress first"),
        "First"
    );

    let mut tx = pool.begin().await.expect("begin tx");
    let second = upsert_youtube_comment_item(&mut tx, 2, &youtube_comment("c1", "Second"))
        .await
        .expect("second comment");
    tx.commit().await.expect("commit second");
    assert_eq!(first, second);

    let content: Vec<u8> =
        sqlx::query_scalar("SELECT content_zstd FROM analysis_documents WHERE item_id = ?")
            .bind(first)
            .fetch_one(&pool)
            .await
            .expect("load updated document");
    assert_eq!(
        decompress_text(&content).expect("decompress second"),
        "Second"
    );
}
```

Add this helper next to the existing `sources::items::tests` helpers:

```rust
async fn seed_youtube_video_source(pool: &sqlx::SqlitePool, source_id: i64) {
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES (?, 'youtube', 'video', ?, 'Video', 1, 1, 1)",
    )
    .bind(source_id)
    .bind(format!("video{source_id}"))
    .execute(pool)
    .await
    .expect("seed youtube source");
}
```

- [ ] **Step 2: Add failing transcript document maintenance test**

In `src-tauri/src/youtube/captions.rs`, extend the test fixture table setup to include `analysis_documents`, `sources`, and `items` when needed. Add:

```rust
#[tokio::test]
async fn replace_transcript_segments_rebuilds_analysis_documents_by_segment_order() {
    let pool = transcript_pool().await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    seed_video_source_and_transcript_item(&pool, 2, 20).await;

    let transcript = YoutubeTranscript {
        video_id: "video2".to_string(),
        language: Some("en".to_string()),
        is_auto_generated: false,
        track_kind: YoutubeCaptionTrackKind::Manual,
        raw_payload: serde_json::json!({ "events": [] }),
        segments: vec![
            YoutubeTranscriptSegment {
                index: 0,
                start_ms: 900,
                end_ms: Some(1_500),
                text: "early".to_string(),
                chapter_index: None,
            },
            YoutubeTranscriptSegment {
                index: 1,
                start_ms: 10_000,
                end_ms: Some(11_000),
                text: "late".to_string(),
                chapter_index: None,
            },
        ],
    };

    let mut tx = pool.begin().await.expect("begin tx");
    replace_transcript_segments(&mut tx, 20, 2, &transcript)
        .await
        .expect("replace segments");
    tx.commit().await.expect("commit");

    let refs: Vec<String> = sqlx::query_scalar(
        "SELECT ref FROM analysis_documents
         WHERE source_id = 2 AND document_kind = 'youtube_transcript'
         ORDER BY document_order ASC, id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("load refs");
    assert_eq!(refs, vec!["s2-i20@900ms", "s2-i20@10000ms"]);
}
```

Add helper:

```rust
async fn seed_video_source_and_transcript_item(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    item_id: i64,
) {
    crate::sources::test_support::create_youtube_typed_source_tables(pool).await;
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES (?, 'youtube', 'video', ?, 'Video', 1, 1, 1)",
    )
    .bind(source_id)
    .bind(format!("video{source_id}"))
    .execute(pool)
    .await
    .expect("seed source");
    sqlx::query(
        "INSERT INTO youtube_video_sources (
            source_id, video_id, canonical_url, title, channel_title,
            published_at, video_form, availability_status
         ) VALUES (?, ?, ?, 'Video', 'Channel', '2026-05-01', 'regular', 'available')",
    )
    .bind(source_id)
    .bind(format!("video{source_id}"))
    .bind(format!("https://www.youtube.com/watch?v=video{source_id}"))
    .execute(pool)
    .await
    .expect("seed typed video");
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at,
            ingested_at, content_kind, has_media, content_zstd
         ) VALUES (?, ?, 'transcript:video:en:manual', 'youtube_transcript',
            'Channel', 1704067200, 1704067200, 'text_only', 0, x'01')",
    )
    .bind(item_id)
    .bind(source_id)
    .execute(pool)
    .await
    .expect("seed transcript item");
}
```

- [ ] **Step 3: Run runtime writer tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction
```

Because Cargo accepts only one test filter, run the other red tests separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content
cargo test --manifest-path src-tauri/Cargo.toml youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order
```

Expected: tests fail because runtime write paths do not maintain `analysis_documents`.

- [ ] **Step 4: Add item-specific upsert helper**

In `src-tauri/src/analysis_documents.rs`, add:

```rust
pub(crate) async fn upsert_item_backed_document_on_connection(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<()> {
    let row: Option<ItemDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.item_kind,
            items.author,
            items.published_at,
            items.content_zstd,
            sources.source_type,
            sources.source_subtype
        FROM items
        JOIN sources ON sources.id = items.source_id
        WHERE items.id = ?
          AND items.content_zstd IS NOT NULL
          AND items.content_kind IN ('text_only', 'text_with_media')
          AND items.item_kind IN ('telegram_message', 'youtube_comment')
        "#,
    )
    .bind(item_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let Some(row) = row else {
        sqlx::query("DELETE FROM analysis_documents WHERE item_id = ?")
            .bind(item_id)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
        return Ok(());
    };

    upsert_item_document_row_on_connection(conn, row).await
}

async fn upsert_item_document_row_on_connection(
    conn: &mut sqlx::SqliteConnection,
    row: ItemDocumentRow,
) -> AppResult<()> {
    let document_kind = match row.item_kind.as_str() {
        "telegram_message" => DOCUMENT_KIND_TELEGRAM_MESSAGE,
        "youtube_comment" => DOCUMENT_KIND_YOUTUBE_COMMENT,
        _ => return Ok(()),
    };
    sqlx::query(
        r#"
        INSERT INTO analysis_documents (
            source_id, item_id, document_key, document_kind,
            source_type, source_subtype, external_id, author,
            published_at, document_order, ref, content_zstd,
            metadata_zstd, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, strftime('%s','now'), strftime('%s','now'))
        ON CONFLICT(source_id, document_key) DO UPDATE SET
            item_id = excluded.item_id,
            document_kind = excluded.document_kind,
            source_type = excluded.source_type,
            source_subtype = excluded.source_subtype,
            external_id = excluded.external_id,
            author = excluded.author,
            published_at = excluded.published_at,
            document_order = excluded.document_order,
            ref = excluded.ref,
            content_zstd = excluded.content_zstd,
            metadata_zstd = excluded.metadata_zstd,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(row.source_id)
    .bind(row.id)
    .bind(format!("item:{}", row.id))
    .bind(document_kind)
    .bind(&row.source_type)
    .bind(&row.source_subtype)
    .bind(&row.external_id)
    .bind(&row.author)
    .bind(row.published_at)
    .bind(row.id)
    .bind(live_item_ref(row.source_id, row.id))
    .bind(row.content_zstd)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [ ] **Step 5: Wire Telegram and comment item writes**

In `insert_telegram_source_item_on_connection`, after topic membership resolution and before returning inserted:

```rust
crate::analysis_documents::upsert_item_backed_document_on_connection(conn, item_id).await?;
```

In `upsert_youtube_comment_item`, store the returned id, call the helper, then return it:

```rust
let item_id: i64 = sqlx::query_scalar(/* existing query */)
    // existing binds
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

crate::analysis_documents::upsert_item_backed_document_on_connection(&mut **tx, item_id).await?;
Ok(item_id)
```

Do not write documents for duplicate Telegram observations or skipped empty payloads unless the insert path actually created a new `items` row.

- [ ] **Step 6: Wire transcript segment replacement**

In `src-tauri/src/analysis_documents.rs`, add:

```rust
pub(crate) async fn rebuild_youtube_transcript_documents_for_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<()> {
    let source_id: Option<i64> =
        sqlx::query_scalar("SELECT source_id FROM items WHERE id = ? AND item_kind = 'youtube_transcript'")
            .bind(item_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;
    let Some(source_id) = source_id else {
        sqlx::query(
            "DELETE FROM analysis_documents
             WHERE item_id = ? AND document_kind = 'youtube_transcript'",
        )
        .bind(item_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
        return Ok(());
    };

    sqlx::query(
        "DELETE FROM analysis_documents
         WHERE source_id = ? AND item_id = ? AND document_kind = 'youtube_transcript'",
    )
    .bind(source_id)
    .bind(item_id)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    insert_youtube_transcript_documents_for_source_and_item(conn, source_id, item_id).await
}
```

Implement `insert_youtube_transcript_documents_for_source_and_item` by reusing the SQL from `insert_youtube_transcript_documents_for_source` plus an `AND items.id = ?` predicate.

In `src-tauri/src/youtube/captions.rs`, after the segment insert loop and before `Ok(())`:

```rust
crate::analysis_documents::rebuild_youtube_transcript_documents_for_item_on_connection(
    &mut **tx,
    item_id,
)
.await?;
```

- [ ] **Step 7: Run writer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::
```

Because Cargo accepts only one test filter, run captions tests separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::captions::tests::
```

Expected: all selected item and captions tests pass.

- [ ] **Step 8: Commit item-backed runtime maintenance**

Run:

```powershell
git add src-tauri/src/analysis_documents.rs src-tauri/src/sources/items.rs src-tauri/src/youtube/captions.rs
git commit -m "feat: maintain item analysis documents at write time"
```

Expected: commit succeeds.

---

### Task 5: Runtime Maintenance For YouTube Description Documents

**Files:**
- Modify: `src-tauri/src/youtube/source_metadata.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Test: `src-tauri/src/youtube/source_metadata.rs`
- Test: `src-tauri/src/sources/store.rs`

- [ ] **Step 1: Add failing description maintenance tests**

In `src-tauri/src/youtube/source_metadata.rs`, add tests near existing typed metadata tests:

```rust
#[tokio::test]
async fn upsert_video_metadata_maintains_description_document() {
    let pool = crate::sources::test_support::memory_pool_with_sources().await;
    crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    seed_video_source_for_metadata_test(&pool, 2, "video2").await;

    let mut metadata = video_metadata_for_test("video2");
    metadata.description = Some("First description".to_string());
    metadata.published_at = Some("2026-05-01".to_string());

    let mut tx = pool.begin().await.expect("begin tx");
    upsert_video_source_metadata(&mut tx, 2, &metadata)
        .await
        .expect("upsert metadata");
    tx.commit().await.expect("commit");

    let content: Vec<u8> = sqlx::query_scalar(
        "SELECT content_zstd FROM analysis_documents
         WHERE source_id = 2 AND document_key = 'youtube:description'",
    )
    .fetch_one(&pool)
    .await
    .expect("load description doc");
    let text = crate::compression::decompress_text(&content).expect("decompress");
    assert!(text.contains("First description"));

    metadata.description = Some("   ".to_string());
    let mut tx = pool.begin().await.expect("begin tx");
    upsert_video_source_metadata(&mut tx, 2, &metadata)
        .await
        .expect("clear metadata");
    tx.commit().await.expect("commit clear");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM analysis_documents
         WHERE source_id = 2 AND document_key = 'youtube:description'",
    )
    .fetch_one(&pool)
    .await
    .expect("count docs");
    assert_eq!(count, 0);
}
```

Add helpers in the same test module if they do not already exist:

```rust
async fn seed_video_source_for_metadata_test(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    video_id: &str,
) {
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES (?, 'youtube', 'video', ?, 'Video', 1, 1, 1)",
    )
    .bind(source_id)
    .bind(video_id)
    .execute(pool)
    .await
    .expect("seed source");
}
```

- [ ] **Step 2: Run description maintenance test and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document
```

Expected: test fails because typed video metadata upsert does not maintain the synthetic document.

- [ ] **Step 3: Wire typed video metadata upsert**

In `src-tauri/src/youtube/source_metadata.rs`, update `upsert_video_source_metadata`:

```rust
pub(crate) async fn upsert_video_source_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<()> {
    let columns = YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    insert_video_source_columns(&mut **tx, source_id, &columns).await?;
    crate::analysis_documents::upsert_youtube_description_document_on_connection(
        &mut **tx,
        source_id,
    )
    .await
}
```

Do not change `insert_video_source_metadata_on_connection`; migration 20 uses it before `analysis_documents` exists.

- [ ] **Step 4: Update source store tests that call YouTube video upsert**

In `src-tauri/src/sources/store.rs`, update memory DB setup for tests that call `upsert_youtube_video_source` so they create analysis documents schema before invoking the upsert:

```rust
crate::sources::test_support::create_analysis_documents_table(&pool).await;
```

Apply this to existing tests around:

- `upsert_youtube_video_source_handles_legacy_not_null_telegram_kind`
- `upsert_youtube_video_source_writes_typed_row_and_null_source_metadata`
- `upsert_youtube_video_source_conflict_clears_existing_legacy_blob`

Do not add the helper to the invalid canonical URL test if it asserts no source row is created before typed metadata insertion; keep that test focused on validation.

- [ ] **Step 5: Run YouTube source metadata/store tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::source_metadata::tests::
```

Because Cargo accepts only one test filter, run source store tests separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests::upsert_youtube_video_source_
```

Expected: selected tests pass.

- [ ] **Step 6: Commit description maintenance**

Run:

```powershell
git add src-tauri/src/youtube/source_metadata.rs src-tauri/src/sources/store.rs
git commit -m "feat: maintain youtube description analysis documents"
```

Expected: commit succeeds.

---

### Task 6: Switch Live Corpus Reader To Analysis Documents

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Test: `src-tauri/src/analysis/corpus.rs`

- [ ] **Step 1: Add failing reader ordering and containment tests**

In `src-tauri/src/analysis/corpus.rs`, add or update tests in the existing test module:

```rust
async fn rebuild_documents_for_sources(pool: &sqlx::SqlitePool, source_ids: &[i64]) {
    crate::sources::test_support::create_analysis_documents_table(pool).await;
    for source_id in source_ids {
        crate::analysis_documents::rebuild_analysis_documents_for_source(pool, *source_id)
            .await
            .unwrap_or_else(|error| panic!("rebuild source {source_id}: {error}"));
    }
}

#[tokio::test]
async fn load_corpus_messages_orders_transcript_segments_by_document_order_not_ref() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source(&pool, 20).await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, 'youtube_transcript', 'Channel', ?, ?)",
    )
    .bind(21_i64)
    .bind(20_i64)
    .bind("transcript:v1:en:manual")
    .bind(1_710_000_000_i64)
    .bind(compress_text("full transcript").expect("compress"))
    .execute(&pool)
    .await
    .expect("insert transcript item");
    insert_youtube_transcript_segment(&pool, 21, 20, 900, "early").await;
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (
            item_id, source_id, segment_index, start_ms, end_ms, text,
            caption_language, caption_track_kind, is_auto_generated
         ) VALUES (21, 20, 1, 10000, 11000, 'late', 'en', 'manual', 0)",
    )
    .execute(&pool)
    .await
    .expect("insert late segment");
    rebuild_documents_for_sources(&pool, &[20]).await;

    let corpus = load_corpus_messages(
        &pool,
        &corpus_request("youtube", vec![20], YoutubeCorpusMode::TranscriptOnly),
    )
    .await
    .expect("load corpus");

    assert_eq!(
        corpus.iter().map(|message| message.r#ref.as_str()).collect::<Vec<_>>(),
        vec!["s20-i21@900ms", "s20-i21@10000ms"]
    );
}
```

Update existing reader tests that seed `items` or YouTube typed metadata directly to call `rebuild_documents_for_sources` before `load_corpus_messages` or `preflight_analysis_run`. Keep tests for `load_youtube_transcript_segment_messages` and `load_youtube_description_messages` only until those helper functions are removed in the implementation step.

- [ ] **Step 2: Run reader tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
```

Expected: test fails or compile fails because `load_corpus_messages` still reads provider tables and the test expects document rebuild behavior.

- [ ] **Step 3: Add document row and loader**

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
#[derive(sqlx::FromRow)]
struct AnalysisDocumentRow {
    item_id: Option<i64>,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    ref_: String,
    content_zstd: Vec<u8>,
    document_kind: String,
    source_type: String,
    source_subtype: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
}

async fn load_analysis_document_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, String> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            item_id,
            source_id,
            external_id,
            author,
            published_at,
            ref AS ref_,
            content_zstd,
            document_kind,
            source_type,
            source_subtype,
            metadata_zstd
        FROM analysis_documents
        WHERE published_at >=
        "#,
    );
    query.push_bind(request.period_from);
    query.push(" AND published_at <= ");
    query.push_bind(request.period_to);
    query.push(" AND source_id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in &request.source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");
    match request.source_type.as_str() {
        "telegram" => {
            query.push(" AND source_type = 'telegram' AND document_kind = 'telegram_message'");
        }
        "youtube" => {
            query.push(" AND source_type = 'youtube' AND document_kind IN (");
            query.push("'youtube_transcript'");
            if request.youtube_corpus_mode.includes_description() {
                query.push(", 'youtube_description'");
            }
            if request.youtube_corpus_mode.includes_comments() {
                query.push(", 'youtube_comment'");
            }
            query.push(")");
        }
        other => return Err(format!("Unsupported analysis corpus source_type '{other}'")),
    }
    query.push(" ORDER BY published_at ASC, source_id ASC, document_order ASC, id ASC");

    let rows: Vec<AnalysisDocumentRow> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id.unwrap_or(0),
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd)?,
                r#ref: row.ref_,
                item_kind: Some(row.document_kind),
                source_type: Some(row.source_type),
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}
```

- [ ] **Step 4: Switch `load_corpus_messages` to the document loader**

Replace the body of `load_corpus_messages` with:

```rust
pub(crate) async fn load_corpus_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, String> {
    if request.source_ids.is_empty() {
        return Ok(Vec::new());
    }

    load_analysis_document_messages(pool, request).await
}
```

Keep `load_run_snapshot_messages`, `load_run_corpus_messages`, and `list_run_snapshot_messages_page` unchanged. Saved run snapshots must still read `analysis_run_messages`.

Remove the old live-only provider helpers from `corpus.rs` after their tests are converted:

- `load_item_messages`
- `load_youtube_transcript_segment_messages`
- `load_youtube_description_messages`
- local `youtube_segment_metadata_zstd`
- local `youtube_description_metadata_zstd`
- local `ymd_to_unix_midnight`

Keep `live_corpus_ref` as a compatibility wrapper if tests or other modules still call it, but implement it through `crate::analysis_documents::live_item_ref`:

```rust
pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String {
    crate::analysis_documents::live_item_ref(source_id, item_id)
}
```

- [ ] **Step 5: Update reader tests for rebuild-backed setup**

For each existing test that seeds provider/archive rows then calls `load_corpus_messages` or `preflight_analysis_run`, call:

```rust
rebuild_documents_for_sources(&pool, &[/* source ids used by the request */]).await;
```

Do this for at least these tests:

- `live_corpus_refs_use_local_item_ids`
- `preflight_counts_eligible_text_messages_for_sources`
- `preflight_ref_format_matches_corpus_loader_ref_format`
- `preflight_ignores_media_only_items_without_text_content`
- `load_corpus_messages_filters_telegram_to_telegram_message`
- `load_corpus_messages_filters_youtube_transcript_only_to_transcripts`
- `load_corpus_messages_includes_youtube_comment_only_in_comments_mode`
- `description_mode_creates_synthetic_description_message`
- `preflight_count_matches_loader_for_youtube_corpus_modes`

For source-group and playlist tests, rebuild the linked source ids, not the selected playlist source id, matching the spec.

- [ ] **Step 6: Add containment scans to the plan execution notes**

Before committing Task 6, run:

```powershell
rg -n "FROM analysis_documents|analysis_documents" src-tauri/src
rg -n "FROM items|JOIN items|list_source_items|notebooklm_export" src-tauri/src/sources src-tauri/src/notebooklm_export src-tauri/src/analysis/corpus.rs
```

Expected:

- production reads of `analysis_documents` are limited to `src-tauri/src/analysis/corpus.rs` and helper/storage code;
- `list_source_items` and NotebookLM export still read `items`;
- saved run snapshot readers still read `analysis_run_messages`.

- [ ] **Step 7: Run analysis corpus tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: all analysis corpus tests pass.

- [ ] **Step 8: Commit reader switch**

Run:

```powershell
git add src-tauri/src/analysis/corpus.rs
git commit -m "feat: read live analysis corpus from documents"
```

Expected: commit succeeds.

---

### Task 7: Documentation And Full Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/database-schema-legacy-analysis.md`
- Modify: `docs/backlog.md`
- Test: full Rust suite and containment scans

- [ ] **Step 1: Update database schema docs**

In `docs/database-schema.md`, add an `analysis_documents` section near the analysis storage/source sections. Include these facts:

```markdown
### `analysis_documents`

`analysis_documents` is a provider-neutral materialized read model for live
analysis corpus loading. Provider/archive truth remains in `items` plus typed
provider tables such as `telegram_messages`, `youtube_video_sources`,
`youtube_playlist_sources`, and `youtube_transcript_segments`.

The table is rebuildable. Runtime writers maintain it synchronously for
Telegram messages, YouTube comments, YouTube transcript segment rows, and
YouTube video descriptions. Source browsing, source item APIs, and NotebookLM
export continue to read their current provider/archive paths.

`document_order` is the numeric order key inside one
`(published_at, source_id)` bucket. The live corpus reader orders by
`published_at ASC, source_id ASC, document_order ASC, id ASC` and does not use
`ref` as an ordering tie-breaker.

Item-backed documents use `item:<item_id>` keys and public refs shaped like
`s<source_id>-i<item_id>`. YouTube transcript segment documents use
`item:<item_id>:segment:<segment_index>` keys and refs shaped like
`s<source_id>-i<item_id>@<start_ms>ms`. Synthetic YouTube descriptions use the
source-scoped key `youtube:description` and ref `s<source_id>-i0`.
```

Update the migration history table with:

```markdown
| 24 | `24.sql` runner-managed | Add provider-neutral analysis document layer and backfill live analysis corpus documents |
```

- [ ] **Step 2: Update legacy analysis and backlog docs**

In `docs/database-schema-legacy-analysis.md`, mark provider-neutral analysis document layer v1 as shipped and keep NotebookLM export/source browsing migration as follow-up.

In `docs/backlog.md`, update any item that says the provider-neutral analysis document layer is pending so it points to remaining follow-ups only:

```markdown
- [ ] consider moving NotebookLM export and source browsing onto provider-neutral document/read models after v1 analysis corpus migration has settled
```

- [ ] **Step 3: Run containment scans**

Run:

```powershell
rg -n "FROM analysis_documents|analysis_documents" src-tauri/src
rg -n "FROM items|JOIN items|list_source_items|notebooklm_export" src-tauri/src/sources src-tauri/src/notebooklm_export src-tauri/src/analysis/corpus.rs
rg -n "ref ASC|ORDER BY published_at ASC, source_id ASC, ref ASC|T[O]DO|T[B]D|FIX[M]E" docs src-tauri/src
```

Expected:

- only live analysis corpus/storage/migration code uses `analysis_documents`;
- source browsing and NotebookLM export still use their current paths;
- no new `ref ASC` ordering remains for live corpus documents;
- no new placeholder markers are introduced.

- [ ] **Step 4: Run targeted test suites**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_documents::tests::
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::
cargo test --manifest-path src-tauri/Cargo.toml migrations::analysis_documents::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::
cargo test --manifest-path src-tauri/Cargo.toml youtube::captions::tests::
cargo test --manifest-path src-tauri/Cargo.toml youtube::source_metadata::tests::
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: all targeted tests pass.

- [ ] **Step 5: Run full Rust verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git status --short
```

Expected: full Rust suite passes, formatting passes, diff check has no whitespace errors, and status shows only intended docs/plan changes before commit.

- [ ] **Step 6: Commit docs and final verification**

Run:

```powershell
git add docs/database-schema.md docs/database-schema-legacy-analysis.md docs/backlog.md
git commit -m "docs: document analysis documents layer"
```

Expected: commit succeeds.

---

## Self-Review Checklist

- Spec coverage: Task 1 covers schema shape and indexes; Task 2 covers rebuild/backfill, document keys, metadata envelopes, source consistency, description generation, and `document_order`; Task 3 covers runner-managed migration 24, restart safety, idempotence, and fresh schema; Tasks 4 and 5 cover synchronous runtime writer maintenance; Task 6 switches only `load_corpus_messages`; Task 7 covers docs and containment scans.
- Reader containment: source browsing, `list_source_items`, `get_items`, NotebookLM export, and saved run snapshot readers remain off `analysis_documents`.
- Ordering: Task 6 includes the `@900ms` before `@10000ms` regression and uses `document_order ASC, id ASC`, not `ref ASC`.
- Migration safety: migration 24 is not registered until after storage rebuild/backfill helpers exist, and the runner records success only after schema + backfill commit.
- TDD: every implementation task starts with failing tests and includes exact commands for red/green verification.
