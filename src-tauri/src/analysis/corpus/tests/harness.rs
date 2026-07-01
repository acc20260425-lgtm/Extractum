use crate::analysis::corpus::{CorpusLoadRequest, YoutubeCorpusMode};
use crate::analysis::models::{AnalysisRunDetail, CorpusMessage};
use crate::compression::{compress_json_bytes, compress_text};
use crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubeVideoForm, YoutubeVideoMetadata};
pub(super) fn sample_corpus() -> Vec<CorpusMessage> {
    vec![
        CorpusMessage {
            item_id: 11,
            source_id: 2,
            external_id: "100".to_string(),
            published_at: 1_710_000_000,
            author: Some("Alice".to_string()),
            content: "First frozen message".to_string(),
            r#ref: "s2-i11".to_string(),
            item_kind: Some("youtube_transcript".to_string()),
            source_type: Some("youtube".to_string()),
            source_subtype: Some("video".to_string()),
            metadata_zstd: Some(
                compress_json_bytes(
                    br#"{"video_id":"video2","item_kind":"youtube_transcript"}"#,
                )
                .expect("compress metadata"),
            ),
        },
        CorpusMessage {
            item_id: 12,
            source_id: 4,
            external_id: "101".to_string(),
            published_at: 1_710_000_100,
            author: None,
            content: "Second frozen message".to_string(),
            r#ref: "s4-i12".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: Some("channel".to_string()),
            metadata_zstd: None,
        },
    ]
}

pub(super) async fn create_project_scope_schema(pool: &sqlx::SqlitePool) {
    for statement in [
        r#"
        CREATE TABLE sources (
            id INTEGER PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            external_id TEXT,
            title TEXT,
            is_active INTEGER NOT NULL,
            is_member INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
        r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
        r#"
        CREATE TABLE project_sources (
            project_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            added_at INTEGER NOT NULL
        )
        "#,
        r#"
        CREATE TABLE youtube_playlist_items (
            playlist_source_id INTEGER NOT NULL,
            video_source_id INTEGER,
            video_id TEXT NOT NULL,
            position INTEGER,
            is_removed_from_playlist INTEGER NOT NULL DEFAULT 0
        )
        "#,
    ] {
        sqlx::query(statement)
            .execute(pool)
            .await
            .expect("create project scope test schema");
    }
}

pub(super) async fn snapshot_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    sqlx::query(
        r#"
        CREATE TABLE sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type TEXT NOT NULL DEFAULT 'telegram',
            source_subtype TEXT,
            external_id TEXT NOT NULL DEFAULT '',
            title TEXT,
            metadata_zstd BLOB
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create sources");
    sqlx::query(
        r#"
        INSERT INTO sources (id, source_type, source_subtype, external_id, title)
        VALUES (2, 'telegram', 'channel', 'telegram-2', 'Telegram 2'),
               (4, 'telegram', 'channel', 'telegram-4', 'Telegram 4')
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert default telegram sources");

    sqlx::query(
        r#"
        CREATE TABLE items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            item_kind TEXT NOT NULL DEFAULT 'telegram_message',
            author TEXT,
            published_at INTEGER NOT NULL,
            ingested_at INTEGER NOT NULL DEFAULT 0,
            content_kind TEXT NOT NULL DEFAULT 'text_only',
            has_media INTEGER NOT NULL DEFAULT 0,
            content_zstd BLOB,
            raw_data_zstd BLOB,
            media_kind TEXT,
            media_metadata_zstd BLOB
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create items");
    crate::sources::test_support::create_telegram_messages_table(&pool).await;

    sqlx::query(
        r#"
        CREATE TABLE analysis_source_groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            source_type TEXT NOT NULL DEFAULT 'telegram',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create groups");

    sqlx::query(
        r#"
        CREATE TABLE analysis_source_group_members (
            group_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create group members");

    sqlx::query(
        r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create projects");

    sqlx::query(
        r#"
        CREATE TABLE project_sources (
            project_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            added_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create project sources");

    sqlx::query(
        r#"
        CREATE TABLE youtube_playlist_items (
            playlist_source_id INTEGER NOT NULL,
            video_id TEXT NOT NULL,
            video_source_id INTEGER,
            position INTEGER,
            availability_status TEXT NOT NULL,
            is_removed_from_playlist BOOLEAN NOT NULL DEFAULT 0,
            metadata_zstd BLOB
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create youtube playlist items");

    sqlx::query(
        r#"
        CREATE TABLE youtube_transcript_segments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            segment_index INTEGER NOT NULL,
            start_ms INTEGER NOT NULL,
            end_ms INTEGER,
            text TEXT NOT NULL,
            chapter_index INTEGER,
            caption_language TEXT,
            caption_track_kind TEXT,
            is_auto_generated INTEGER NOT NULL DEFAULT 0,
            metadata_zstd BLOB,
            UNIQUE(item_id, segment_index)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create youtube transcript segments");
    crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;

    sqlx::query(
        r#"
        CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_type TEXT NOT NULL,
            scope_type TEXT NOT NULL,
            source_id INTEGER,
            source_group_id INTEGER,
            project_id INTEGER,
            period_from INTEGER NOT NULL,
            period_to INTEGER NOT NULL,
            output_language TEXT NOT NULL,
            prompt_template_id INTEGER,
            prompt_template_version INTEGER NOT NULL,
            provider_profile TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
            status TEXT NOT NULL,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            error TEXT,
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create runs");

    sqlx::query(
        r#"
        CREATE TABLE analysis_run_messages (
            run_id INTEGER NOT NULL,
            item_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            author TEXT,
            published_at INTEGER NOT NULL,
            ref TEXT NOT NULL,
            content_zstd BLOB NOT NULL,
            item_kind TEXT,
            source_type TEXT,
            source_subtype TEXT,
            metadata_zstd BLOB,
            PRIMARY KEY (run_id, ref)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create run messages");

    pool
}

pub(super) fn corpus_request(
    source_type: &str,
    source_ids: Vec<i64>,
    youtube_corpus_mode: YoutubeCorpusMode,
) -> CorpusLoadRequest {
    CorpusLoadRequest {
        source_type: source_type.to_string(),
        source_ids,
        period_from: 1_700_000_000,
        period_to: 1_800_000_000,
        youtube_corpus_mode,
        include_migrated_history: false,
    }
}

pub(super) async fn rebuild_documents_for_sources(pool: &sqlx::SqlitePool, source_ids: &[i64]) {
    crate::sources::test_support::create_analysis_documents_table(pool).await;
    for source_id in source_ids {
        crate::analysis_documents::rebuild_analysis_documents_for_source(pool, *source_id)
            .await
            .unwrap_or_else(|error| panic!("rebuild source {source_id}: {error}"));
    }
}

pub(super) async fn seed_analysis_source(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    source_type: &str,
    source_subtype: &str,
) {
    sqlx::query(
        "INSERT OR REPLACE INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(source_id)
    .bind(source_type)
    .bind(source_subtype)
    .bind(format!("{source_type}-{source_id}"))
    .bind(format!("Source {source_id}"))
    .execute(pool)
    .await
    .expect("seed analysis source");
}

pub(super) async fn seed_telegram_item(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    source_id: i64,
    external_id: &str,
    published_at: i64,
    text: &str,
    migrated: bool,
) {
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, ingested_at,
            content_kind, has_media, content_zstd
         ) VALUES (?, ?, ?, 'telegram_message', 'Ada', ?, ?, 'text_only', 0, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(external_id)
    .bind(published_at)
    .bind(published_at)
    .bind(compress_text(text).expect("compress telegram item"))
    .execute(pool)
    .await
    .expect("seed telegram item");

    let (peer_kind, peer_id, migration_domain, is_migrated_history) = if migrated {
        ("chat", 777_i64, Some("migrated_from_chat"), 1_i64)
    } else {
        ("channel", 12345_i64, None, 0_i64)
    };
    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history
         ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(peer_kind)
    .bind(peer_id)
    .bind(external_id.parse::<i64>().expect("numeric telegram id"))
    .bind(migration_domain)
    .bind(is_migrated_history)
    .execute(pool)
    .await
    .expect("seed telegram identity");
}

pub(super) fn youtube_metadata_zstd(video_id: &str, title: &str, description: Option<&str>) -> Vec<u8> {
    let metadata = YoutubeVideoMetadata {
        video_id: video_id.to_string(),
        canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
        title: Some(title.to_string()),
        channel_title: Some("Channel".to_string()),
        channel_id: Some("UCdemo".to_string()),
        channel_handle: Some("@channel".to_string()),
        channel_url: Some("https://www.youtube.com/@channel".to_string()),
        author_display: Some("Channel".to_string()),
        published_at: Some("2026-05-01".to_string()),
        duration_seconds: Some(120),
        description: description.map(ToString::to_string),
        thumbnail_url: None,
        tags: Vec::new(),
        chapters: Vec::new(),
        view_count: None,
        like_count: None,
        comment_count: None,
        category: None,
        video_form: YoutubeVideoForm::Regular,
        availability_status: YoutubeAvailabilityStatus::Available,
        raw_metadata_json: serde_json::json!({ "id": video_id }),
    };
    let json = serde_json::to_vec(&metadata).expect("serialize youtube metadata");
    compress_json_bytes(&json).expect("compress youtube metadata")
}

pub(super) async fn insert_youtube_video_source(pool: &sqlx::SqlitePool, source_id: i64) {
    insert_youtube_video_source_with_typed_metadata(
        pool,
        source_id,
        &format!("video{source_id}"),
        &format!("Video {source_id}"),
        None,
        Some("2026-05-01"),
    )
    .await;
}

pub(super) async fn insert_youtube_video_source_with_typed_metadata(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    video_id: &str,
    title: &str,
    description: Option<&str>,
    published_at: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd)
         VALUES (?, 'youtube', 'video', ?, ?, ?)",
    )
    .bind(source_id)
    .bind(video_id)
    .bind(title)
    .bind(youtube_metadata_zstd(
        video_id,
        title,
        description,
    ))
    .execute(pool)
    .await
    .expect("insert youtube video source");
    insert_typed_youtube_video_source(
        pool,
        source_id,
        video_id,
        title,
        description,
        published_at,
    )
    .await;
}

pub(super) async fn insert_typed_youtube_video_source(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    video_id: &str,
    title: &str,
    description: Option<&str>,
    published_at: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO youtube_video_sources (
            source_id, video_id, canonical_url, title, channel_title,
            channel_handle, published_at, description, video_form, availability_status
        )
        VALUES (?, ?, ?, ?, 'Channel', '@channel', ?, ?, 'regular', 'available')
        "#,
    )
    .bind(source_id)
    .bind(video_id)
    .bind(format!("https://www.youtube.com/watch?v={video_id}"))
    .bind(title)
    .bind(published_at)
    .bind(description)
    .execute(pool)
    .await
    .expect("insert typed youtube video source");
}

pub(super) async fn insert_youtube_transcript_segment(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    source_id: i64,
    start_ms: i64,
    text: &str,
) {
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (
            item_id, source_id, segment_index, start_ms, end_ms, text,
            caption_language, caption_track_kind, is_auto_generated
         )
         VALUES (?, ?, 0, ?, ?, ?, 'en', 'manual', 0)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(start_ms)
    .bind(start_ms + 1_000)
    .bind(text)
    .execute(pool)
    .await
    .expect("insert youtube transcript segment");
}

pub(super) fn decode_message_metadata_for_test(message: &CorpusMessage) -> serde_json::Value {
    let bytes = message.metadata_zstd.as_deref().expect("message metadata");
    let decoded = crate::compression::decompress_bytes(bytes).expect("decompress metadata");
    serde_json::from_slice(&decoded).expect("parse metadata")
}

pub(super) fn sample_run() -> AnalysisRunDetail {
    AnalysisRunDetail {
        id: 1,
        run_type: "report".to_string(),
        scope_type: "source_group".to_string(),
        source_id: None,
        source_title: None,
        source_group_id: Some(9),
        source_group_name: Some("Live group".to_string()),
        project_id: None,
        project_name: None,
        scope_label: "Frozen group".to_string(),
        period_from: 1_700_000_000,
        period_to: 1_800_000_000,
        output_language: "English".to_string(),
        prompt_template_id: Some(1),
        prompt_template_name: Some("Default".to_string()),
        prompt_template_version: 1,
        provider_profile: "default".to_string(),
        provider: "gemini".to_string(),
        model: "gemini-2.5-flash".to_string(),
        youtube_corpus_mode: "transcript_description".to_string(),
        telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT
            .to_string(),
        status: "completed".to_string(),
        result_markdown: Some("Saved report".to_string()),
        error: None,
        has_trace_data: true,
        snapshot_state: Some(crate::analysis::models::AnalysisSnapshotState::Captured),
        snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
        snapshot_error: None,
        created_at: 1_710_000_500,
        completed_at: Some(1_710_000_600),
        scope_label_snapshot: Some("Frozen group".to_string()),
        snapshot_message_count: 1,
    }
}
