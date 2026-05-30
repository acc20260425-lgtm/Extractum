use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, State};

use super::AnalysisState;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::youtube::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
};

const FIXTURE_MARKER: &str = "__analysis_redesign_fixture__";
const FIXTURE_EXTERNAL_PREFIX: &str = "__analysis_redesign_fixture__:";
const FIXTURE_PROFILE_ID: &str = "__analysis_redesign_fixture__";
const FIXTURE_NOW: i64 = 1_778_400_000;
const FIXTURE_PERIOD_FROM: i64 = 1_777_968_000;
const FIXTURE_PERIOD_TO: i64 = 1_778_313_600;

const TELEGRAM_CHANNEL_LABEL: &str = "__analysis_redesign_fixture__ Telegram Channel";
const TELEGRAM_SUPERGROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Supergroup";
const YOUTUBE_VIDEO_LABEL: &str = "__analysis_redesign_fixture__ YouTube Video";
const YOUTUBE_PLAYLIST_LABEL: &str = "__analysis_redesign_fixture__ YouTube Playlist";
const YOUTUBE_FIXTURE_VIDEO_ID: &str = "analysis_fixture_video";
const YOUTUBE_FIXTURE_PLAYLIST_ID: &str = "PLanalysisfixture";
const TELEGRAM_FIXTURE_CHANNEL_PEER_ID: i64 = 10_000_001;
const TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID: i64 = 10_000_002;
const TELEGRAM_GROUP_LABEL: &str = "__analysis_redesign_fixture__ Telegram Source Group";
const COMPLETED_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Completed Snapshot Run";
const MISSING_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Missing Snapshot Run";
const RUNNING_RUN_LABEL: &str = "__analysis_redesign_fixture__ Running Run";
const FAILED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Failed Run";
const CANCELLED_RUN_LABEL: &str = "__analysis_redesign_fixture__ Cancelled Run";
const GROUP_SNAPSHOT_RUN_LABEL: &str = "__analysis_redesign_fixture__ Group Snapshot Run";
const LLM_PROFILE_LABEL: &str = "__analysis_redesign_fixture__ LLM Profile";
const FIXTURE_SNAPSHOT_CAPTURED_AT: &str = "2026-05-18T10:00:00Z";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisRedesignFixtureSummary {
    pub accounts: i64,
    pub llm_profiles: i64,
    pub sources: i64,
    pub source_groups: i64,
    pub prompt_templates: i64,
    pub runs: i64,
    pub snapshot_messages: i64,
    pub chat_messages: i64,
    pub youtube_transcript_segments: i64,
    pub youtube_playlist_items: i64,
}

#[tauri::command]
pub async fn seed_analysis_redesign_fixtures(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    let previous_run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &previous_run_ids).await;
    let summary = seed_analysis_redesign_fixtures_in_pool(&pool).await?;
    register_fixture_active_runs(&pool, state.inner()).await?;
    Ok(summary)
}

#[tauri::command]
pub async fn clear_analysis_redesign_fixtures(
    handle: AppHandle,
    state: State<'_, AnalysisState>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let pool = get_pool(&handle).await?;
    let run_ids = fixture_run_ids(&pool).await?;
    remove_fixture_active_runs(state.inner(), &run_ids).await;
    clear_analysis_redesign_fixtures_in_pool(&pool).await
}

async fn fixture_run_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    let marker_pattern = format!("{FIXTURE_MARKER}%");
    sqlx::query_scalar(
        "SELECT id FROM analysis_runs
         WHERE scope_label_snapshot LIKE ? OR provider_profile = ?",
    )
    .bind(marker_pattern)
    .bind(FIXTURE_PROFILE_ID)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn register_fixture_active_runs(pool: &Pool<Sqlite>, state: &AnalysisState) -> AppResult<()> {
    let run_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM analysis_runs
         WHERE scope_label_snapshot = ? AND status = 'running'",
    )
    .bind(RUNNING_RUN_LABEL)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    for run_id in run_ids {
        state.insert_active_report_run(run_id).await;
    }

    Ok(())
}

async fn remove_fixture_active_runs(state: &AnalysisState, run_ids: &[i64]) {
    for run_id in run_ids {
        state.remove_active_report_run(*run_id).await;
    }
}

fn json_zstd(value: serde_json::Value) -> AppResult<Vec<u8>> {
    let json = serde_json::to_vec(&value).map_err(|error| AppError::internal(error.to_string()))?;
    crate::compression::compress_json_bytes(&json).map_err(AppError::internal)
}

async fn insert_fixture_account(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO accounts (label, api_id, api_hash, phone, created_at)
         VALUES (?, 100001, '', NULL, ?)
         RETURNING id",
    )
    .bind(format!("{FIXTURE_MARKER} Telegram Account"))
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_fixture_prompt_template(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO analysis_prompt_templates (
            name, template_kind, body, version, is_builtin, created_at, updated_at
         )
         VALUES (?, 'report', ?, 1, 0, ?, ?)
         RETURNING id",
    )
    .bind(format!("{FIXTURE_MARKER} Report Template"))
    .bind(
        "Write a concise fixture report. Cite saved evidence refs and keep fixture labels visible.",
    )
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_fixture_llm_profile(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<()> {
    for (key, value) in [
        (
            format!("llm.profile.{FIXTURE_PROFILE_ID}.provider"),
            "gemini".to_string(),
        ),
        (
            format!("llm.profile.{FIXTURE_PROFILE_ID}.default_model"),
            "gemini-2.5-flash".to_string(),
        ),
        (
            format!("llm.profile.{FIXTURE_PROFILE_ID}.base_url"),
            String::new(),
        ),
    ] {
        sqlx::query(
            "INSERT INTO app_settings (key, value)
             VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }
    Ok(())
}

async fn insert_telegram_source(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    label: &str,
    kind: &str,
    peer_id: i64,
    external_suffix: &str,
    last_sync_state: i64,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         )
         VALUES ('telegram', ?, ?, ?, ?, ?, ?, ?, 1, 1, ?)
         RETURNING id",
    )
    .bind(kind)
    .bind(account_id)
    .bind(peer_id.to_string())
    .bind(label)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "peer_identity": {
            "strategy": "dialog",
            "username": external_suffix,
            "access_hash": 424242
        }
    }))?)
    .bind(last_sync_state)
    .bind(FIXTURE_NOW - 600)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_youtube_video_source(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    let video_id = YOUTUBE_FIXTURE_VIDEO_ID;
    let source_id = sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         )
         VALUES ('youtube', 'video', NULL, ?, ?, ?, NULL, ?, 1, 0, ?)
         RETURNING id",
    )
    .bind(video_id)
    .bind(YOUTUBE_VIDEO_LABEL)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "video_id": video_id,
        "canonical_url": "https://www.youtube.com/watch?v=analysis_fixture_video",
        "title": YOUTUBE_VIDEO_LABEL,
        "channel_title": "Fixture Channel",
        "channel_id": "UCfixture",
        "channel_handle": "@analysisfixture",
        "channel_url": "https://www.youtube.com/@analysisfixture",
        "author_display": "Fixture Channel",
        "published_at": "2026-05-01",
        "duration_seconds": 920,
        "description": "Fixture video description for analysis report verification.",
        "thumbnail_url": null,
        "tags": ["fixture", "analysis"],
        "chapters": [],
        "view_count": 1250,
        "like_count": 86,
        "comment_count": 2,
        "category": "Education",
        "video_form": "regular",
        "availability_status": "available",
        "raw_metadata_json": { "fixture": true }
    }))?)
    .bind(FIXTURE_NOW - 540)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    crate::youtube::source_metadata::upsert_video_source_metadata(
        tx,
        source_id,
        &YoutubeVideoMetadata {
            video_id: video_id.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
            title: Some(YOUTUBE_VIDEO_LABEL.to_string()),
            channel_title: Some("Fixture Channel".to_string()),
            channel_id: Some("UCfixture".to_string()),
            channel_handle: Some("@analysisfixture".to_string()),
            channel_url: Some("https://www.youtube.com/@analysisfixture".to_string()),
            author_display: Some("Fixture Channel".to_string()),
            published_at: Some("2026-05-01".to_string()),
            duration_seconds: Some(920),
            description: Some(
                "Fixture video description for analysis report verification.".to_string(),
            ),
            thumbnail_url: None,
            tags: vec!["fixture".to_string(), "analysis".to_string()],
            chapters: Vec::new(),
            view_count: Some(1250),
            like_count: Some(86),
            comment_count: Some(2),
            category: Some("Education".to_string()),
            video_form: YoutubeVideoForm::Regular,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: serde_json::json!({ "fixture": true }),
        },
    )
    .await?;

    Ok(source_id)
}

async fn insert_youtube_playlist_source(tx: &mut sqlx::Transaction<'_, Sqlite>) -> AppResult<i64> {
    let playlist_id = YOUTUBE_FIXTURE_PLAYLIST_ID;
    let source_id = sqlx::query_scalar(
        "INSERT INTO sources (
            source_type, source_subtype, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         )
         VALUES ('youtube', 'playlist', NULL, ?, ?, ?, NULL, ?, 1, 0, ?)
         RETURNING id",
    )
    .bind(playlist_id)
    .bind(YOUTUBE_PLAYLIST_LABEL)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "playlist_id": playlist_id,
        "canonical_url": "https://www.youtube.com/playlist?list=PLanalysisfixture",
        "title": YOUTUBE_PLAYLIST_LABEL,
        "channel_title": "Fixture Channel",
        "channel_id": "UCfixture",
        "channel_handle": "@analysisfixture",
        "channel_url": "https://www.youtube.com/@analysisfixture",
        "thumbnail_url": null,
        "video_count": 2,
        "items": [],
        "availability_status": "available",
        "raw_metadata_json": { "fixture": true }
    }))?)
    .bind(FIXTURE_NOW - 500)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    crate::youtube::source_metadata::upsert_playlist_source_metadata(
        tx,
        source_id,
        &YoutubePlaylistMetadata {
            playlist_id: playlist_id.to_string(),
            canonical_url: format!("https://www.youtube.com/playlist?list={playlist_id}"),
            title: Some(YOUTUBE_PLAYLIST_LABEL.to_string()),
            channel_title: Some("Fixture Channel".to_string()),
            channel_id: Some("UCfixture".to_string()),
            channel_handle: Some("@analysisfixture".to_string()),
            channel_url: Some("https://www.youtube.com/@analysisfixture".to_string()),
            thumbnail_url: None,
            video_count: Some(2),
            items: Vec::new(),
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: serde_json::json!({ "fixture": true }),
        },
    )
    .await?;

    Ok(source_id)
}

async fn insert_fixture_source_group(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    telegram_channel_id: i64,
    telegram_supergroup_id: i64,
) -> AppResult<i64> {
    let group_id: i64 = sqlx::query_scalar(
        "INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
         VALUES (?, 'telegram', ?, ?)
         RETURNING id",
    )
    .bind(TELEGRAM_GROUP_LABEL)
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    for source_id in [telegram_channel_id, telegram_supergroup_id] {
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (?, ?, ?)",
        )
        .bind(group_id)
        .bind(source_id)
        .bind(FIXTURE_NOW)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }
    Ok(group_id)
}

async fn insert_item(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    source_id: i64,
    external_suffix: &str,
    item_kind: &str,
    author: &str,
    published_at: i64,
    content: &str,
    content_kind: &str,
    media_kind: Option<&str>,
    media_metadata: Option<serde_json::Value>,
    reply_to_msg_id: Option<i64>,
    reply_to_top_id: Option<i64>,
    reaction_count: Option<i64>,
) -> AppResult<i64> {
    let raw = json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "external_suffix": external_suffix,
        "item_kind": item_kind
    }))?;
    let media_metadata_zstd = media_metadata.map(json_zstd).transpose()?;
    sqlx::query_scalar(
        "INSERT INTO items (
            source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd,
            raw_data_zstd, content_kind, has_media, media_kind, media_metadata_zstd,
            reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
            reaction_count
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)
         RETURNING id",
    )
    .bind(source_id)
    .bind(format!("{FIXTURE_EXTERNAL_PREFIX}{external_suffix}"))
    .bind(item_kind)
    .bind(author)
    .bind(published_at)
    .bind(FIXTURE_NOW - 480)
    .bind(crate::compression::compress_text(content).map_err(AppError::internal)?)
    .bind(raw)
    .bind(content_kind)
    .bind(media_metadata_zstd.is_some())
    .bind(media_kind)
    .bind(media_metadata_zstd)
    .bind(reply_to_msg_id)
    .bind(reply_to_top_id)
    .bind(reaction_count)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_telegram_content(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    telegram_channel_id: i64,
    telegram_supergroup_id: i64,
) -> AppResult<()> {
    insert_item(
        tx,
        telegram_channel_id,
        "tg-channel-1",
        "telegram_message",
        "Fixture Editor",
        FIXTURE_PERIOD_FROM + 1_200,
        "fixture channel update: result-first analysis now has source evidence",
        "text_only",
        None,
        None,
        None,
        None,
        Some(4),
    )
    .await?;
    insert_item(
        tx,
        telegram_channel_id,
        "tg-channel-2",
        "telegram_message",
        "Fixture Analyst",
        FIXTURE_PERIOD_FROM + 2_400,
        "fixture channel update: browser verification should show this timeline row",
        "text_only",
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, icon_color, icon_emoji_id,
            is_closed, is_pinned, is_hidden, is_deleted, sort_order, last_seen_at, updated_at
         )
         VALUES (?, 501, 7001, ?, 7322096, NULL, 0, 1, 0, 0, 1, ?, ?)",
    )
    .bind(telegram_supergroup_id)
    .bind(format!("{FIXTURE_MARKER} Topic"))
    .bind(FIXTURE_NOW - 420)
    .bind(FIXTURE_NOW - 420)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;

    insert_item(
        tx,
        telegram_supergroup_id,
        "tg-supergroup-topic",
        "telegram_message",
        "Fixture Moderator",
        FIXTURE_PERIOD_FROM + 3_600,
        "fixture supergroup topic: grouped source reader should preserve topic metadata",
        "text_only",
        None,
        None,
        None,
        None,
        None,
    )
    .await?;
    insert_item(
        tx,
        telegram_supergroup_id,
        "tg-supergroup-media",
        "telegram_message",
        "Fixture Member",
        FIXTURE_PERIOD_FROM + 4_800,
        "fixture supergroup media placeholder: no binary preview is available",
        "text_with_media",
        Some("photo"),
        Some(serde_json::json!({
            "analysis_redesign_fixture": true,
            "summary": "Fixture image placeholder",
            "file_name": "fixture-proof.jpg",
            "mime_type": "image/jpeg",
            "size_bytes": 204800,
            "width": 1280,
            "height": 720
        })),
        Some(7001),
        Some(501),
        Some(2),
    )
    .await?;
    Ok(())
}

async fn insert_youtube_content(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    youtube_video_id: i64,
    youtube_playlist_id: i64,
) -> AppResult<()> {
    let transcript_item_id = insert_item(
        tx,
        youtube_video_id,
        "youtube-transcript",
        "youtube_transcript",
        "Fixture Channel",
        FIXTURE_PERIOD_FROM + 6_000,
        "Fixture transcript full text for result-first report evidence.",
        "text_only",
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    for (index, start_ms, text) in [
        (
            0_i64,
            0_i64,
            "Fixture opening segment introduces the redesign.",
        ),
        (
            1_i64,
            754_000_i64,
            "Fixture timestamp segment supports Show in source.",
        ),
        (
            2_i64,
            790_000_i64,
            "Fixture closing segment mentions evidence tabs.",
        ),
    ] {
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                chapter_index, caption_language, caption_track_kind, is_auto_generated,
                metadata_zstd
             )
             VALUES (?, ?, ?, ?, ?, ?, NULL, 'en', 'manual', 0, ?)",
        )
        .bind(transcript_item_id)
        .bind(youtube_video_id)
        .bind(index)
        .bind(start_ms)
        .bind(start_ms + 25_000)
        .bind(text)
        .bind(json_zstd(
            serde_json::json!({ "analysis_redesign_fixture": true }),
        )?)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    insert_item(
        tx,
        youtube_video_id,
        "youtube-comment",
        "youtube_comment",
        "Fixture Commenter",
        FIXTURE_PERIOD_FROM + 7_200,
        "Fixture comment validates transcript_description_comments mode.",
        "text_only",
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    sqlx::query(
        "INSERT INTO youtube_playlist_items (
            playlist_source_id, video_source_id, video_id, position, title_snapshot, url,
            thumbnail_url, availability_status, is_removed_from_playlist, last_seen_at,
            metadata_zstd, created_at, updated_at
         )
         VALUES
            (?, ?, ?, 1, ?, 'https://www.youtube.com/watch?v=analysis_fixture_video', NULL,
             'available', 0, ?, ?, ?, ?),
            (?, NULL, ?, 2, ?, 'https://www.youtube.com/watch?v=analysis_fixture_missing', NULL,
             'private_or_auth_required', 0, ?, ?, ?, ?)",
    )
    .bind(youtube_playlist_id)
    .bind(youtube_video_id)
    .bind(YOUTUBE_FIXTURE_VIDEO_ID)
    .bind(YOUTUBE_VIDEO_LABEL)
    .bind(FIXTURE_NOW - 360)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "linked": true
    }))?)
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .bind(youtube_playlist_id)
    .bind("analysis_fixture_missing")
    .bind(format!("{FIXTURE_MARKER} Unavailable Playlist Item"))
    .bind(FIXTURE_NOW - 360)
    .bind(json_zstd(serde_json::json!({
        "analysis_redesign_fixture": true,
        "linked": false
    }))?)
    .bind(FIXTURE_NOW)
    .bind(FIXTURE_NOW)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

struct FixtureIds {
    prompt_template_id: i64,
    telegram_channel_id: i64,
    telegram_supergroup_id: i64,
    youtube_video_id: i64,
    source_group_id: i64,
}

async fn insert_run(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    label: &str,
    scope_type: &str,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    prompt_template_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<Vec<u8>>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "INSERT INTO analysis_runs (
            run_type, scope_type, source_id, source_group_id, period_from, period_to,
            output_language, prompt_template_id, prompt_template_version, provider_profile,
            provider, model, youtube_corpus_mode, status, result_markdown, trace_data_zstd,
            scope_label_snapshot, error, created_at, completed_at
         )
         VALUES (
            'report', ?, ?, ?, ?, ?, 'English', ?, 1, ?, 'gemini',
            'gemini-2.5-flash', 'transcript_description_comments', ?, ?, ?, ?, ?, ?, ?
         )
         RETURNING id",
    )
    .bind(scope_type)
    .bind(source_id)
    .bind(source_group_id)
    .bind(FIXTURE_PERIOD_FROM)
    .bind(FIXTURE_PERIOD_TO)
    .bind(prompt_template_id)
    .bind(FIXTURE_PROFILE_ID)
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(label)
    .bind(error)
    .bind(FIXTURE_NOW)
    .bind(completed_at)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn insert_snapshot_message(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    run_id: i64,
    item_id: i64,
    source_id: i64,
    external_id: &str,
    author: &str,
    published_at: i64,
    reference: &str,
    content: &str,
    item_kind: &str,
    source_type: &str,
    source_subtype: &str,
    metadata: Option<serde_json::Value>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO analysis_run_messages (
            run_id, item_id, source_id, external_id, author, published_at, ref, content_zstd,
            item_kind, source_type, source_subtype, metadata_zstd
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(item_id)
    .bind(source_id)
    .bind(external_id)
    .bind(author)
    .bind(published_at)
    .bind(reference)
    .bind(crate::compression::compress_text(content).map_err(AppError::internal)?)
    .bind(item_kind)
    .bind(source_type)
    .bind(source_subtype)
    .bind(metadata.map(json_zstd).transpose()?)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn mark_fixture_snapshot_captured(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    run_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE analysis_runs
         SET snapshot_captured_at = ?, snapshot_error = NULL
         WHERE id = ?",
    )
    .bind(FIXTURE_SNAPSHOT_CAPTURED_AT)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn trace_zstd(refs: serde_json::Value) -> AppResult<Vec<u8>> {
    json_zstd(serde_json::json!({ "refs": refs }))
}

async fn first_item_id(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    external_suffix: &str,
) -> AppResult<i64> {
    sqlx::query_scalar("SELECT id FROM items WHERE external_id = ?")
        .bind(format!("{FIXTURE_EXTERNAL_PREFIX}{external_suffix}"))
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::database)
}

async fn insert_analysis_runs(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    ids: FixtureIds,
) -> AppResult<()> {
    let youtube_item_id = first_item_id(tx, "youtube-transcript").await?;
    let telegram_item_id = first_item_id(tx, "tg-channel-1").await?;
    let youtube_ref = format!("s{}-i{}@754000ms", ids.youtube_video_id, youtube_item_id);
    let telegram_ref = format!("s{}-i{}", ids.telegram_channel_id, telegram_item_id);

    let completed_youtube_run_id = insert_run(
        tx,
        COMPLETED_SNAPSHOT_RUN_LABEL,
        "single_source",
        Some(ids.youtube_video_id),
        None,
        ids.prompt_template_id,
        "completed",
        Some(&format!(
            "# {COMPLETED_SNAPSHOT_RUN_LABEL}\n\nProvider fixture: {LLM_PROFILE_LABEL}.\n\nYouTube evidence is available at [{youtube_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": youtube_ref,
            "item_id": youtube_item_id,
            "source_id": ids.youtube_video_id,
            "external_id": format!("{FIXTURE_EXTERNAL_PREFIX}youtube-transcript"),
            "published_at": FIXTURE_PERIOD_FROM + 6_000,
            "excerpt": "Fixture timestamp segment supports Show in source.",
            "youtube_url": "https://www.youtube.com/watch?v=analysis_fixture_video&t=754",
            "youtube_timestamp_seconds": 754,
            "youtube_display_label": format!("{YOUTUBE_VIDEO_LABEL} at 12:34"),
            "is_synthetic": false
        }]))?),
        None,
        Some(FIXTURE_NOW + 20),
    )
    .await?;
    insert_snapshot_message(
        tx,
        completed_youtube_run_id,
        youtube_item_id,
        ids.youtube_video_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}youtube-transcript"),
        "Fixture Channel",
        FIXTURE_PERIOD_FROM + 6_000,
        &youtube_ref,
        "Fixture timestamp segment supports Show in source.",
        "youtube_transcript",
        "youtube",
        "video",
        Some(serde_json::json!({
            "analysis_redesign_fixture": true,
            "canonical_url": "https://www.youtube.com/watch?v=analysis_fixture_video",
            "title": YOUTUBE_VIDEO_LABEL,
            "segment_start_ms": 754000,
            "segment_end_ms": 779000
        })),
    )
    .await?;
    mark_fixture_snapshot_captured(tx, completed_youtube_run_id).await?;

    let missing_ref = format!("s{}-i999999", ids.telegram_channel_id);
    insert_run(
        tx,
        MISSING_SNAPSHOT_RUN_LABEL,
        "single_source",
        Some(ids.telegram_channel_id),
        None,
        ids.prompt_template_id,
        "completed",
        Some(&format!(
            "# {MISSING_SNAPSHOT_RUN_LABEL}\n\nProvider fixture: {LLM_PROFILE_LABEL}.\n\nThis report cites missing saved evidence [{missing_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": missing_ref,
            "item_id": 999999,
            "source_id": ids.telegram_channel_id,
            "external_id": "missing-fixture-item",
            "published_at": FIXTURE_PERIOD_FROM + 100,
            "excerpt": "Missing fixture evidence",
            "youtube_url": null,
            "youtube_timestamp_seconds": null,
            "youtube_display_label": null,
            "is_synthetic": false
        }]))?),
        None,
        Some(FIXTURE_NOW + 30),
    )
    .await?;

    for (label, status, error, completed_at) in [
        (RUNNING_RUN_LABEL, "running", None, None),
        (
            FAILED_RUN_LABEL,
            "failed",
            Some("Fixture failure: provider request failed without changing user data"),
            Some(FIXTURE_NOW + 40),
        ),
        (
            CANCELLED_RUN_LABEL,
            "cancelled",
            Some("Fixture cancellation: run was cancelled before snapshot capture"),
            Some(FIXTURE_NOW + 50),
        ),
    ] {
        insert_run(
            tx,
            label,
            "single_source",
            Some(ids.telegram_channel_id),
            None,
            ids.prompt_template_id,
            status,
            None,
            None,
            error,
            completed_at,
        )
        .await?;
    }

    let group_run_id = insert_run(
        tx,
        GROUP_SNAPSHOT_RUN_LABEL,
        "source_group",
        None,
        Some(ids.source_group_id),
        ids.prompt_template_id,
        "completed",
        Some(&format!(
            "# {GROUP_SNAPSHOT_RUN_LABEL}\n\nProvider fixture: {LLM_PROFILE_LABEL}.\n\nTelegram evidence is available at [{telegram_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": telegram_ref,
            "item_id": telegram_item_id,
            "source_id": ids.telegram_channel_id,
            "external_id": format!("{FIXTURE_EXTERNAL_PREFIX}tg-channel-1"),
            "published_at": FIXTURE_PERIOD_FROM + 1_200,
            "excerpt": "fixture channel update: result-first analysis now has source evidence",
            "youtube_url": null,
            "youtube_timestamp_seconds": null,
            "youtube_display_label": null,
            "source_type": "telegram",
            "is_synthetic": false
        }]))?),
        None,
        Some(FIXTURE_NOW + 60),
    )
    .await?;
    insert_snapshot_message(
        tx,
        group_run_id,
        telegram_item_id,
        ids.telegram_channel_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}tg-channel-1"),
        "Fixture Editor",
        FIXTURE_PERIOD_FROM + 1_200,
        &telegram_ref,
        "fixture channel update: result-first analysis now has source evidence",
        "telegram_message",
        "telegram",
        "channel",
        Some(serde_json::json!({ "analysis_redesign_fixture": true })),
    )
    .await?;

    let supergroup_topic_id = first_item_id(tx, "tg-supergroup-topic").await?;
    let supergroup_media_id = first_item_id(tx, "tg-supergroup-media").await?;
    insert_snapshot_message(
        tx,
        group_run_id,
        supergroup_topic_id,
        ids.telegram_supergroup_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}tg-supergroup-topic"),
        "Fixture Moderator",
        FIXTURE_PERIOD_FROM + 3_600,
        &format!("s{}-i{}", ids.telegram_supergroup_id, supergroup_topic_id),
        "fixture supergroup topic: grouped source reader should preserve topic metadata",
        "telegram_message",
        "telegram",
        "supergroup",
        Some(serde_json::json!({ "analysis_redesign_fixture": true })),
    )
    .await?;
    insert_snapshot_message(
        tx,
        group_run_id,
        supergroup_media_id,
        ids.telegram_supergroup_id,
        &format!("{FIXTURE_EXTERNAL_PREFIX}tg-supergroup-media"),
        "Fixture Member",
        FIXTURE_PERIOD_FROM + 4_800,
        &format!("s{}-i{}", ids.telegram_supergroup_id, supergroup_media_id),
        "fixture supergroup media placeholder: no binary preview is available",
        "telegram_message",
        "telegram",
        "supergroup",
        Some(serde_json::json!({ "analysis_redesign_fixture": true })),
    )
    .await?;
    mark_fixture_snapshot_captured(tx, group_run_id).await?;

    for (role, content, created_at) in [
        ("user", "Summarize the strongest fixture evidence.", FIXTURE_NOW + 70),
        (
            "assistant",
            "The fixture evidence highlights saved snapshots, YouTube timestamps, and Telegram source context.",
            FIXTURE_NOW + 71,
        ),
    ] {
        sqlx::query(
            "INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(group_run_id)
        .bind(role)
        .bind(content)
        .bind(created_at)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    Ok(())
}

async fn seed_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let _ = clear_analysis_redesign_fixtures_in_pool(pool).await?;
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let account_id = insert_fixture_account(&mut tx).await?;
    let prompt_template_id = insert_fixture_prompt_template(&mut tx).await?;
    insert_fixture_llm_profile(&mut tx).await?;
    let telegram_channel_id = insert_telegram_source(
        &mut tx,
        account_id,
        TELEGRAM_CHANNEL_LABEL,
        "channel",
        TELEGRAM_FIXTURE_CHANNEL_PEER_ID,
        "telegram-channel",
        9001,
    )
    .await?;
    let telegram_supergroup_id = insert_telegram_source(
        &mut tx,
        account_id,
        TELEGRAM_SUPERGROUP_LABEL,
        "supergroup",
        TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID,
        "telegram-supergroup",
        9101,
    )
    .await?;
    let youtube_video_id = insert_youtube_video_source(&mut tx).await?;
    let youtube_playlist_id = insert_youtube_playlist_source(&mut tx).await?;
    let source_group_id =
        insert_fixture_source_group(&mut tx, telegram_channel_id, telegram_supergroup_id).await?;
    insert_telegram_content(&mut tx, telegram_channel_id, telegram_supergroup_id).await?;
    insert_youtube_content(&mut tx, youtube_video_id, youtube_playlist_id).await?;
    insert_analysis_runs(
        &mut tx,
        FixtureIds {
            prompt_template_id,
            telegram_channel_id,
            telegram_supergroup_id,
            youtube_video_id,
            source_group_id,
        },
    )
    .await?;

    let _ = youtube_playlist_id;

    tx.commit().await.map_err(AppError::database)?;

    Ok(AnalysisRedesignFixtureSummary {
        accounts: 1,
        llm_profiles: 1,
        sources: 4,
        source_groups: 1,
        prompt_templates: 1,
        runs: 6,
        snapshot_messages: 4,
        chat_messages: 2,
        youtube_transcript_segments: 3,
        youtube_playlist_items: 2,
    })
}

async fn clear_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let marker_pattern = format!("{FIXTURE_MARKER}%");
    let external_pattern = format!("{FIXTURE_EXTERNAL_PREFIX}%");
    let profile_settings_pattern = format!("llm.profile.{FIXTURE_PROFILE_ID}.%");

    let mut summary = AnalysisRedesignFixtureSummary::default();

    summary.chat_messages = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_chat_messages
             WHERE run_id IN (
                SELECT id FROM analysis_runs
                WHERE scope_label_snapshot LIKE ? OR provider_profile = ?
             )",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.snapshot_messages = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_run_messages
             WHERE run_id IN (
                SELECT id FROM analysis_runs
                WHERE scope_label_snapshot LIKE ? OR provider_profile = ?
             )",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.runs = rows_to_i64(
        sqlx::query(
            "DELETE FROM analysis_runs
             WHERE scope_label_snapshot LIKE ? OR provider_profile = ?",
        )
        .bind(&marker_pattern)
        .bind(FIXTURE_PROFILE_ID)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    let fixture_profile_setting_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM app_settings WHERE key LIKE ?")
            .bind(&profile_settings_pattern)
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::database)?;
    sqlx::query("DELETE FROM app_settings WHERE key LIKE ?")
        .bind(&profile_settings_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    summary.llm_profiles = if fixture_profile_setting_count > 0 {
        1
    } else {
        0
    };

    summary.prompt_templates = rows_to_i64(
        sqlx::query("DELETE FROM analysis_prompt_templates WHERE name LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    sqlx::query(
        "DELETE FROM analysis_source_group_members
         WHERE group_id IN (SELECT id FROM analysis_source_groups WHERE name LIKE ?)
            OR source_id IN (
                SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
            )",
    )
    .bind(&marker_pattern)
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    summary.source_groups = rows_to_i64(
        sqlx::query("DELETE FROM analysis_source_groups WHERE name LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    summary.youtube_playlist_items = rows_to_i64(
        sqlx::query(
            "DELETE FROM youtube_playlist_items
             WHERE playlist_source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR video_source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR video_id LIKE ?",
        )
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&external_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    summary.youtube_transcript_segments = rows_to_i64(
        sqlx::query(
            "DELETE FROM youtube_transcript_segments
             WHERE source_id IN (
                    SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
                )
                OR item_id IN (
                    SELECT items.id
                    FROM items
                    JOIN sources ON sources.id = items.source_id
                    WHERE sources.title LIKE ? OR sources.external_id LIKE ?
                )",
        )
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .bind(&marker_pattern)
        .bind(&external_pattern)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected(),
    );

    sqlx::query(
        "DELETE FROM telegram_forum_topics
         WHERE source_id IN (
            SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
         )",
    )
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "DELETE FROM items
         WHERE source_id IN (
            SELECT id FROM sources WHERE title LIKE ? OR external_id LIKE ?
         )",
    )
    .bind(&marker_pattern)
    .bind(&external_pattern)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    summary.sources = rows_to_i64(
        sqlx::query("DELETE FROM sources WHERE title LIKE ? OR external_id LIKE ?")
            .bind(&marker_pattern)
            .bind(&external_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    summary.accounts = rows_to_i64(
        sqlx::query("DELETE FROM accounts WHERE label LIKE ?")
            .bind(&marker_pattern)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?
            .rows_affected(),
    );

    tx.commit().await.map_err(AppError::database)?;
    Ok(summary)
}

fn rows_to_i64(rows: u64) -> i64 {
    rows as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn fixture_pool() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("enable foreign keys");
        pool
    }

    async fn count(pool: &Pool<Sqlite>, sql: &str) -> i64 {
        sqlx::query_scalar::<_, i64>(sql)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|error| panic!("count query failed: {sql}: {error}"))
    }

    #[tokio::test]
    async fn summary_serializes_with_camel_case_keys() {
        let summary = AnalysisRedesignFixtureSummary {
            accounts: 1,
            llm_profiles: 1,
            sources: 4,
            source_groups: 1,
            prompt_templates: 1,
            runs: 6,
            snapshot_messages: 4,
            chat_messages: 2,
            youtube_transcript_segments: 3,
            youtube_playlist_items: 2,
        };

        let value = serde_json::to_value(summary).expect("serialize summary");

        assert_eq!(value["llmProfiles"], 1);
        assert_eq!(value["sourceGroups"], 1);
        assert_eq!(value["promptTemplates"], 1);
        assert_eq!(value["snapshotMessages"], 4);
        assert_eq!(value["youtubeTranscriptSegments"], 3);
        assert_eq!(value["youtubePlaylistItems"], 2);
    }

    #[tokio::test]
    async fn fixture_test_pool_has_required_tables() {
        let pool = fixture_pool().await;

        for table in [
            "accounts",
            "sources",
            "items",
            "telegram_forum_topics",
            "youtube_transcript_segments",
            "youtube_playlist_items",
            "analysis_prompt_templates",
            "analysis_source_groups",
            "analysis_source_group_members",
            "analysis_runs",
            "analysis_run_messages",
            "analysis_chat_messages",
            "app_settings",
        ] {
            let exists = count(
                &pool,
                &format!(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{table}'"
                ),
            )
            .await;
            assert_eq!(exists, 1, "missing table {table}");
        }
    }

    async fn insert_minimal_clear_fixture(pool: &Pool<Sqlite>) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at)
             VALUES (10, '__analysis_redesign_fixture__ Account', 100001, '', 10)",
        )
        .execute(pool)
        .await
        .expect("insert fixture account");
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, last_synced_at, is_active, is_member, created_at
             )
             VALUES (
                20, 'youtube', 'video', NULL,
                '__analysis_redesign_fixture__:clear-source',
                '__analysis_redesign_fixture__ Clear Source',
                10, 1, 0, 10
             )",
        )
        .execute(pool)
        .await
        .expect("insert fixture source");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_kind, has_media
             )
             VALUES (
                30, 20, '__analysis_redesign_fixture__:clear-item',
                'youtube_transcript', 'Fixture', 10, 10, 'text_only', 0
             )",
        )
        .execute(pool)
        .await
        .expect("insert fixture item");
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text
             )
             VALUES (30, 20, 0, 1000, 2000, 'Fixture clear segment')",
        )
        .execute(pool)
        .await
        .expect("insert fixture transcript segment");
        sqlx::query(
            "INSERT INTO youtube_playlist_items (
                playlist_source_id, video_source_id, video_id, position, availability_status,
                is_removed_from_playlist, created_at, updated_at
             )
             VALUES (20, NULL, '__analysis_redesign_fixture__:video', 1, 'available', 0, 10, 10)",
        )
        .execute(pool)
        .await
        .expect("insert fixture playlist item");
        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, last_seen_at, updated_at
             )
             VALUES (20, 1, 1, '__analysis_redesign_fixture__ Topic', 10, 10)",
        )
        .execute(pool)
        .await
        .expect("insert fixture topic");
        sqlx::query(
            "INSERT INTO analysis_prompt_templates (
                id, name, template_kind, body, version, is_builtin, created_at, updated_at
             )
             VALUES (
                40, '__analysis_redesign_fixture__ Template', 'report', 'Body', 1, 0, 10, 10
             )",
        )
        .execute(pool)
        .await
        .expect("insert fixture template");
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (50, '__analysis_redesign_fixture__ Group', 'youtube', 10, 10)",
        )
        .execute(pool)
        .await
        .expect("insert fixture group");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (50, 20, 10)",
        )
        .execute(pool)
        .await
        .expect("insert fixture group member");
        sqlx::query(
            "INSERT INTO analysis_runs (
                id, run_type, scope_type, source_id, period_from, period_to, output_language,
                prompt_template_id, prompt_template_version, provider_profile, provider, model,
                youtube_corpus_mode, status, result_markdown, scope_label_snapshot, created_at,
                completed_at
             )
             VALUES (
                60, 'report', 'single_source', 20, 1, 2, 'English', 40, 1,
                '__analysis_redesign_fixture__', 'gemini', 'model', 'transcript_description',
                'completed', 'Fixture result', '__analysis_redesign_fixture__ Run', 10, 11
             )",
        )
        .execute(pool)
        .await
        .expect("insert fixture run");
        sqlx::query(
            "INSERT INTO analysis_run_messages (
                run_id, item_id, source_id, external_id, author, published_at, ref, content_zstd
             )
             VALUES (
                60, 30, 20, '__analysis_redesign_fixture__:clear-item',
                'Fixture', 10, 's20-i30', x'28B52FFD0000010000'
             )",
        )
        .execute(pool)
        .await
        .expect("insert fixture run message");
        sqlx::query(
            "INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
             VALUES (60, 'user', 'Fixture chat', 10)",
        )
        .execute(pool)
        .await
        .expect("insert fixture chat message");
        for key in [
            "llm.profile.__analysis_redesign_fixture__.provider",
            "llm.profile.__analysis_redesign_fixture__.default_model",
            "llm.profile.__analysis_redesign_fixture__.base_url",
        ] {
            sqlx::query("INSERT INTO app_settings (key, value) VALUES (?, 'fixture')")
                .bind(key)
                .execute(pool)
                .await
                .expect("insert fixture profile setting");
        }
    }

    #[tokio::test]
    async fn clear_removes_only_fixture_rows_and_is_idempotent() {
        let pool = fixture_pool().await;
        sqlx::query(
            "INSERT INTO accounts (label, api_id, api_hash, created_at)
             VALUES ('Personal', 12345, '', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert non-fixture account");
        sqlx::query(
            "INSERT INTO sources (
                source_type, source_subtype, account_id, external_id,
                title, is_active, is_member, created_at
             )
             VALUES ('telegram', 'channel', 1, 'real-source', 'Real Source', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert non-fixture source");

        insert_minimal_clear_fixture(&pool).await;

        let cleared = clear_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("clear fixtures");
        let second_clear = clear_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("clear fixtures again");

        assert_eq!(cleared.accounts, 1);
        assert_eq!(cleared.sources, 1);
        assert_eq!(cleared.source_groups, 1);
        assert_eq!(cleared.prompt_templates, 1);
        assert_eq!(cleared.runs, 1);
        assert_eq!(cleared.snapshot_messages, 1);
        assert_eq!(cleared.chat_messages, 1);
        assert_eq!(cleared.youtube_transcript_segments, 1);
        assert_eq!(cleared.youtube_playlist_items, 1);
        assert_eq!(second_clear, AnalysisRedesignFixtureSummary::default());
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM accounts WHERE label = 'Personal'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM sources WHERE title = 'Real Source'"
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn clear_preserves_non_fixture_groups_and_members() {
        let pool = fixture_pool().await;

        let real_account_id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (label, api_id, api_hash, created_at)
             VALUES ('Personal', 1, '', 1)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("insert non-fixture account");

        let real_source_id: i64 = sqlx::query_scalar(
            "INSERT INTO sources (
                source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             )
             VALUES ('telegram', 'channel', ?, 'real-source', 'Real Source', 1, 1, 1)
             RETURNING id",
        )
        .bind(real_account_id)
        .fetch_one(&pool)
        .await
        .expect("insert non-fixture source");

        let real_group_id: i64 = sqlx::query_scalar(
            "INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
             VALUES ('Real Group', 'telegram', 1, 1)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("insert non-fixture group");

        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (?, ?, 1)",
        )
        .bind(real_group_id)
        .bind(real_source_id)
        .execute(&pool)
        .await
        .expect("insert non-fixture group member");

        insert_minimal_clear_fixture(&pool).await;

        clear_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("clear fixtures");

        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_source_groups WHERE name = 'Real Group'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_source_group_members member
                 JOIN analysis_source_groups group_row ON group_row.id = member.group_id
                 JOIN sources source_row ON source_row.id = member.source_id
                 WHERE group_row.name = 'Real Group' AND source_row.title = 'Real Source'",
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn clear_deletes_child_rows_through_fixture_parent_ids() {
        let pool = fixture_pool().await;
        insert_minimal_clear_fixture(&pool).await;

        sqlx::query(
            "INSERT INTO analysis_runs (
                run_type, scope_type, source_id, period_from, period_to, output_language,
                prompt_template_version, provider_profile, provider, model, youtube_corpus_mode,
                status, result_markdown, scope_label_snapshot, created_at, completed_at
             )
             VALUES (
                'report', 'single_source', NULL, 1, 2, 'English', 1, 'default', 'gemini',
                'model', 'transcript_description', 'completed',
                '__analysis_redesign_fixture__ text in user content',
                'User Run', 3, 4
             )",
        )
        .execute(&pool)
        .await
        .expect("insert non-fixture run with marker text");

        clear_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("clear fixtures");

        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE scope_label_snapshot = 'User Run'"
            )
            .await,
            1
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM analysis_run_messages").await,
            0
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM analysis_chat_messages").await,
            0
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM youtube_transcript_segments").await,
            0
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM youtube_playlist_items").await,
            0
        );
    }

    #[tokio::test]
    async fn seed_creates_safe_account_prompt_profile_sources_and_group() {
        let pool = fixture_pool().await;

        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let account: (String, i64, String, Option<String>) =
            sqlx::query_as("SELECT label, api_id, api_hash, phone FROM accounts WHERE label = ?")
                .bind(TELEGRAM_CHANNEL_LABEL.replace("Telegram Channel", "Telegram Account"))
                .fetch_one(&pool)
                .await
                .expect("load fixture account");
        assert!(account.0.starts_with(FIXTURE_MARKER));
        assert_eq!(account.1, 100_001);
        assert_eq!(account.2, "");
        assert_eq!(account.3, None);

        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%'"
            )
            .await,
            4
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM sources WHERE source_type = 'telegram' AND account_id IS NOT NULL"
            )
            .await,
            2
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_source_groups WHERE name = '__analysis_redesign_fixture__ Telegram Source Group'"
            )
            .await,
            1
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM analysis_source_group_members").await,
            2
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_prompt_templates WHERE name LIKE '__analysis_redesign_fixture__%' AND template_kind = 'report'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM app_settings WHERE key LIKE 'llm.profile.__analysis_redesign_fixture__.%'"
            )
            .await,
            3
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM app_settings WHERE key LIKE 'llm.profile.__analysis_redesign_fixture__.api_key'"
            )
            .await,
            0
        );
    }

    #[tokio::test]
    async fn seed_creates_post_sync_reader_content() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM items WHERE item_kind = 'telegram_message'"
            )
            .await,
            4
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM telegram_forum_topics").await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM items WHERE has_media = 1 AND media_metadata_zstd IS NOT NULL"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM items WHERE reply_to_top_id IS NOT NULL OR reply_to_msg_id IS NOT NULL"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM items WHERE reaction_count IS NOT NULL"
            )
            .await,
            2
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM items WHERE item_kind = 'youtube_transcript'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM items WHERE item_kind = 'youtube_comment'"
            )
            .await,
            1
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM youtube_transcript_segments").await,
            3
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM youtube_playlist_items").await,
            2
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%' AND last_synced_at IS NOT NULL"
            )
            .await,
            4
        );
    }

    #[tokio::test]
    async fn seed_creates_valid_typed_youtube_detail_metadata() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let video_source_id: i64 = sqlx::query_scalar("SELECT id FROM sources WHERE title = ?")
            .bind(YOUTUBE_VIDEO_LABEL)
            .fetch_one(&pool)
            .await
            .expect("load fixture video source");
        let video_detail =
            crate::youtube::detail::get_youtube_video_detail_from_pool(&pool, video_source_id)
                .await
                .expect("load fixture video detail");

        assert_eq!(
            video_detail.source_metadata.video_id,
            "analysis_fixture_video"
        );
        assert_eq!(
            video_detail.source_metadata.raw_metadata_json,
            Some(serde_json::json!({ "fixture": true }))
        );

        let playlist_source_id: i64 = sqlx::query_scalar("SELECT id FROM sources WHERE title = ?")
            .bind(YOUTUBE_PLAYLIST_LABEL)
            .fetch_one(&pool)
            .await
            .expect("load fixture playlist source");
        let playlist_detail = crate::youtube::detail::get_youtube_playlist_detail_from_pool(
            &pool,
            playlist_source_id,
        )
        .await
        .expect("load fixture playlist detail");

        assert_eq!(playlist_detail.items.len(), 2);
        assert_eq!(playlist_detail.items[0].video_id, "analysis_fixture_video");
    }

    #[tokio::test]
    async fn seed_creates_sources_that_pass_identity_repair() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let report = crate::sources::identity_repair::repair_source_identity(
            &pool,
            crate::sources::identity_repair::SourceIdentityRepairMode::Apply,
        )
        .await
        .expect("repair seeded fixture identities");

        assert!(report.fatal_errors.is_empty());
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM telegram_sources").await,
            2
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM telegram_sources WHERE source_subtype = 'channel' AND peer_kind = 'channel'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM telegram_sources WHERE source_subtype = 'supergroup' AND peer_kind = 'channel'"
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn compressed_fixture_fields_are_readable() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let content: Vec<u8> = sqlx::query_scalar(
            "SELECT content_zstd FROM items WHERE external_id LIKE '__analysis_redesign_fixture__:tg-channel-1'",
        )
        .fetch_one(&pool)
        .await
        .expect("load content");
        let media: Vec<u8> = sqlx::query_scalar(
            "SELECT media_metadata_zstd FROM items WHERE media_metadata_zstd IS NOT NULL LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load media metadata");
        let raw: Vec<u8> = sqlx::query_scalar(
            "SELECT raw_data_zstd FROM items WHERE raw_data_zstd IS NOT NULL LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load raw data");

        assert!(crate::compression::decompress_text(&content)
            .expect("decompress content")
            .contains("fixture channel update"));
        assert!(String::from_utf8(
            crate::compression::decompress_bytes(&media).expect("decompress media")
        )
        .expect("media utf8")
        .contains("image/jpeg"));
        assert!(String::from_utf8(
            crate::compression::decompress_bytes(&raw).expect("decompress raw")
        )
        .expect("raw utf8")
        .contains("analysis_redesign_fixture"));
    }

    #[tokio::test]
    async fn seed_creates_fixture_runs_with_statuses_templates_and_snapshots() {
        let pool = fixture_pool().await;
        let summary = seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        assert_eq!(summary.runs, 6);
        assert_eq!(summary.snapshot_messages, 4);
        assert_eq!(summary.chat_messages, 2);
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE prompt_template_id IS NOT NULL"
            )
            .await,
            6
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(DISTINCT status) FROM analysis_runs WHERE scope_label_snapshot LIKE '__analysis_redesign_fixture__%'"
            )
            .await,
            4
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE status = 'completed'"
            )
            .await,
            3
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE status = 'running'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE status = 'failed'"
            )
            .await,
            1
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE status = 'cancelled'"
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn seeded_snapshot_runs_expose_captured_snapshot_state() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        for label in [COMPLETED_SNAPSHOT_RUN_LABEL, GROUP_SNAPSHOT_RUN_LABEL] {
            let run_id: i64 =
                sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                    .bind(label)
                    .fetch_one(&pool)
                    .await
                    .expect("load fixture run id");
            let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
                .await
                .expect("fetch fixture run")
                .map(crate::analysis::store::map_run_detail)
                .expect("fixture run exists");

            assert_eq!(
                detail.snapshot_state,
                Some(crate::analysis::models::AnalysisSnapshotState::Captured),
                "{label} should expose captured snapshot state"
            );
            assert!(
                detail.snapshot_captured_at.is_some(),
                "{label} should expose snapshot capture marker"
            );
            assert_eq!(detail.snapshot_error, None);
        }
    }

    #[tokio::test]
    async fn fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let youtube_trace: Vec<u8> = sqlx::query_scalar(
            "SELECT trace_data_zstd FROM analysis_runs WHERE scope_label_snapshot = ?",
        )
        .bind(COMPLETED_SNAPSHOT_RUN_LABEL)
        .fetch_one(&pool)
        .await
        .expect("load youtube trace");
        let telegram_trace: Vec<u8> = sqlx::query_scalar(
            "SELECT trace_data_zstd FROM analysis_runs WHERE scope_label_snapshot = ?",
        )
        .bind(GROUP_SNAPSHOT_RUN_LABEL)
        .fetch_one(&pool)
        .await
        .expect("load telegram trace");

        let youtube_json: serde_json::Value = serde_json::from_slice(
            &crate::compression::decompress_bytes(&youtube_trace)
                .expect("decompress youtube trace"),
        )
        .expect("parse youtube trace");
        let telegram_json: serde_json::Value = serde_json::from_slice(
            &crate::compression::decompress_bytes(&telegram_trace)
                .expect("decompress telegram trace"),
        )
        .expect("parse telegram trace");

        assert!(youtube_json["refs"]
            .as_array()
            .expect("youtube refs")
            .iter()
            .any(|value| value["ref"]
                .as_str()
                .unwrap_or_default()
                .contains("@754000ms")));
        assert!(telegram_json["refs"]
            .as_array()
            .expect("telegram refs")
            .iter()
            .any(|value| value["source_type"] == "telegram"));
    }

    #[tokio::test]
    async fn missing_snapshot_run_has_trace_but_no_saved_messages() {
        let pool = fixture_pool().await;
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        let run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(MISSING_SNAPSHOT_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load missing snapshot run");

        assert_eq!(
            count(
                &pool,
                &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
            )
            .await,
            0
        );
        assert_eq!(
            count(
                &pool,
                &format!(
                    "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
                )
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn fixture_active_state_tracks_seeded_running_run() {
        let pool = fixture_pool().await;
        let state = super::super::AnalysisState::new();
        seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures");

        register_fixture_active_runs(&pool, &state)
            .await
            .expect("register active fixture runs");

        let active_run_ids = state.active_report_run_ids().await;
        let running_run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(RUNNING_RUN_LABEL)
                .fetch_one(&pool)
                .await
                .expect("load running run");

        assert_eq!(active_run_ids.len(), 1);
        assert!(active_run_ids.contains(&running_run_id));

        let fixture_run_ids = fixture_run_ids(&pool).await.expect("load fixture run ids");
        remove_fixture_active_runs(&state, &fixture_run_ids).await;

        assert!(state.active_report_run_ids().await.is_empty());
    }

    #[tokio::test]
    async fn seed_twice_keeps_one_deterministic_fixture_set() {
        let pool = fixture_pool().await;

        let first = seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures once");
        let second = seed_analysis_redesign_fixtures_in_pool(&pool)
            .await
            .expect("seed fixtures twice");

        assert_eq!(first, second);
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM sources WHERE title LIKE '__analysis_redesign_fixture__%'"
            )
            .await,
            4
        );
        assert_eq!(
            count(
                &pool,
                "SELECT COUNT(*) FROM analysis_runs WHERE scope_label_snapshot LIKE '__analysis_redesign_fixture__%'"
            )
            .await,
            6
        );
        assert_eq!(
            count(&pool, "SELECT COUNT(*) FROM analysis_run_messages").await,
            4
        );
    }
}
