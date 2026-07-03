use sqlx::Sqlite;

use super::super::{
    CANCELLED_RUN_LABEL, CAPTURE_FAILED_SNAPSHOT_ERROR, CAPTURE_FAILED_SNAPSHOT_RUN_LABEL,
    COMPLETED_SNAPSHOT_RUN_LABEL, FAILED_RUN_LABEL, FIXTURE_EXTERNAL_PREFIX, FIXTURE_NOW,
    FIXTURE_PERIOD_FROM, FIXTURE_PERIOD_TO, FIXTURE_PROFILE_ID, FIXTURE_SNAPSHOT_CAPTURED_AT,
    GROUP_SNAPSHOT_RUN_LABEL, LLM_PROFILE_LABEL, MISSING_SNAPSHOT_RUN_LABEL, RUNNING_RUN_LABEL,
    YOUTUBE_VIDEO_LABEL,
};
use super::json_zstd;
use crate::error::{AppError, AppResult};
pub(super) struct FixtureIds {
    pub(super) prompt_template_id: i64,
    pub(super) telegram_channel_id: i64,
    pub(super) telegram_supergroup_id: i64,
    pub(super) youtube_video_id: i64,
    pub(super) source_group_id: i64,
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

pub(super) async fn insert_analysis_runs(
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
