use sqlx::{Pool, Sqlite};

use super::{
    clear_analysis_redesign_fixtures_in_pool, AnalysisRedesignFixtureSummary, CANCELLED_RUN_LABEL,
    CAPTURE_FAILED_SNAPSHOT_ERROR, CAPTURE_FAILED_SNAPSHOT_RUN_LABEL, COMPLETED_SNAPSHOT_RUN_LABEL,
    FAILED_RUN_LABEL, FIXTURE_EXTERNAL_PREFIX, FIXTURE_MARKER, FIXTURE_NOW, FIXTURE_PERIOD_FROM,
    FIXTURE_PERIOD_TO, FIXTURE_PROFILE_ID, FIXTURE_SNAPSHOT_CAPTURED_AT, GROUP_SNAPSHOT_RUN_LABEL,
    LLM_PROFILE_LABEL, MISSING_SNAPSHOT_RUN_LABEL, RUNNING_RUN_LABEL, TELEGRAM_CHANNEL_LABEL,
    TELEGRAM_FIXTURE_CHANNEL_PEER_ID, TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID, TELEGRAM_GROUP_LABEL,
    TELEGRAM_SUPERGROUP_LABEL, YOUTUBE_FIXTURE_PLAYLIST_ID, YOUTUBE_FIXTURE_VIDEO_ID,
    YOUTUBE_PLAYLIST_LABEL, YOUTUBE_VIDEO_LABEL,
};
use crate::error::{AppError, AppResult};
use crate::youtube::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
};
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

async fn mark_fixture_snapshot_capture_failed(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    run_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE analysis_runs
         SET snapshot_captured_at = NULL, snapshot_error = ?
         WHERE id = ?",
    )
    .bind(CAPTURE_FAILED_SNAPSHOT_ERROR)
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

    let capture_failed_ref = format!("s{}-i999998", ids.telegram_channel_id);
    let capture_failed_run_id = insert_run(
        tx,
        CAPTURE_FAILED_SNAPSHOT_RUN_LABEL,
        "single_source",
        Some(ids.telegram_channel_id),
        None,
        ids.prompt_template_id,
        "failed",
        Some(&format!(
            "# {CAPTURE_FAILED_SNAPSHOT_RUN_LABEL}\n\nProvider fixture: {LLM_PROFILE_LABEL}.\n\nThis capture-failed fixture report remains readable.\n\nThis report cites capture-failed saved evidence [{capture_failed_ref}]."
        )),
        Some(trace_zstd(serde_json::json!([{
            "ref": capture_failed_ref,
            "item_id": 999998,
            "source_id": ids.telegram_channel_id,
            "external_id": "capture-failed-fixture-item",
            "published_at": FIXTURE_PERIOD_FROM + 200,
            "excerpt": "Capture failed fixture evidence",
            "youtube_url": null,
            "youtube_timestamp_seconds": null,
            "youtube_display_label": null,
            "is_synthetic": false
        }]))?),
        None,
        Some(FIXTURE_NOW + 40),
    )
    .await?;
    mark_fixture_snapshot_capture_failed(tx, capture_failed_run_id).await?;

    for (label, status, error, completed_at) in [
        (RUNNING_RUN_LABEL, "running", None, None),
        (
            FAILED_RUN_LABEL,
            "failed",
            Some("Fixture failure: provider request failed without changing user data"),
            Some(FIXTURE_NOW + 50),
        ),
        (
            CANCELLED_RUN_LABEL,
            "cancelled",
            Some("Fixture cancellation: run was cancelled before snapshot capture"),
            Some(FIXTURE_NOW + 60),
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
        Some(FIXTURE_NOW + 70),
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
        ("user", "Summarize the strongest fixture evidence.", FIXTURE_NOW + 80),
        (
            "assistant",
            "The fixture evidence highlights saved snapshots, YouTube timestamps, and Telegram source context.",
            FIXTURE_NOW + 81,
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

pub(super) async fn seed_analysis_redesign_fixtures_in_pool(
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
        runs: 7,
        snapshot_messages: 4,
        chat_messages: 2,
        youtube_transcript_segments: 3,
        youtube_playlist_items: 2,
    })
}
