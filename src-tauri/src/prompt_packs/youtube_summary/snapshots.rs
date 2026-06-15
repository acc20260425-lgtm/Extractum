use sqlx::SqlitePool;

use super::preflight::preflight_youtube_summary_in_pool;
use super::run_store::{ensure_pack_version, load_run_by_client_request_id};
use super::sources::{load_source, transcript_text_for_source};
use super::{estimate_tokens, now_string, ModelBudget, SYNTHESIS_STAGE_NAME};
use crate::compression::{compress_text, decompress_text};
use crate::error::{AppError, AppResult};
use crate::prompt_packs::dto::{
    PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunRequest,
    YoutubeSummaryPreflightResponse, YoutubeSummaryPreflightVideo,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommentSelectionPolicy {
    pub comment_count_cap: usize,
    pub comment_token_cap: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommentMaterialRef {
    pub external_id: Option<String>,
    pub material_ref_id: String,
    pub token_estimate: i64,
}

pub(crate) async fn create_youtube_summary_run_skeleton_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
    _pack_version_id_hint: i64,
) -> AppResult<i64> {
    if request.client_request_id.trim().is_empty() {
        return Err(AppError::validation("client_request_id cannot be empty"));
    }
    if let Some(run) = load_run_by_client_request_id(pool, &request.client_request_id).await? {
        return Ok(run.run_id);
    }

    let pack_version_id = ensure_pack_version(pool).await?;
    let preflight = preflight_youtube_summary_in_pool(
        pool,
        PreflightYoutubeSummaryRunRequest {
            project_id: request.project_id,
            source_ids: request.source_ids.clone(),
            profile_id: request.profile_id.clone(),
            model_override: request.model_override.clone(),
            output_language: request.output_language.clone(),
            control_preset: request.control_preset.clone(),
            evidence_mode: request.evidence_mode.clone(),
            include_comments: request.include_comments,
        },
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await?;
    if preflight.included_videos.is_empty() || !preflight.blocking_failures.is_empty() {
        return Err(AppError::validation(
            "start preflight did not include runnable videos",
        ));
    }

    let now = now_string();
    let request_json = serde_json::to_string(&serde_json::json!({
        "clientRequestId": request.client_request_id,
        "projectId": request.project_id,
        "sourceIds": request.source_ids,
        "outputLanguage": request.output_language,
        "controlPreset": request.control_preset,
        "evidenceMode": request.evidence_mode,
        "includeComments": request.include_comments
    }))
    .map_err(|error| AppError::internal(format!("serialize request: {error}")))?;
    let preflight_json = serde_json::to_string(&preflight)
        .map_err(|error| AppError::internal(format!("serialize preflight: {error}")))?;

    let run_id: i64 = sqlx::query_scalar(
        "INSERT INTO prompt_pack_runs (
            project_id, pack_version_id, pack_id, pack_version, schema_version,
            run_status, result_status, request_json_zstd, preflight_json_zstd,
            provider_profile_id, model, output_language, control_preset, evidence_mode,
            include_comments, latest_message, progress_current, progress_total,
            created_at, updated_at, client_request_id
         )
         VALUES (?, ?, 'youtube_summary', '1.0.0', '1.0',
            'queued', 'none', ?, ?, ?, ?, ?, ?, ?, ?, 'Queued',
            0, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(request.project_id)
    .bind(pack_version_id)
    .bind(compress_text(&request_json).map_err(AppError::internal)?)
    .bind(compress_text(&preflight_json).map_err(AppError::internal)?)
    .bind(&request.profile_id)
    .bind(&request.model_override)
    .bind(&request.output_language)
    .bind(&request.control_preset)
    .bind(&request.evidence_mode)
    .bind(request.include_comments)
    .bind(preflight.included_videos.len() as i64)
    .bind(&now)
    .bind(&now)
    .bind(&request.client_request_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    for source_id in &request.source_ids {
        let Some(source) = load_source(pool, *source_id).await? else {
            continue;
        };
        let scope_kind = match source.source_subtype.as_deref() {
            Some("playlist") => "playlist",
            _ => "explicit_video",
        };
        sqlx::query(
            "INSERT INTO prompt_pack_run_scopes (
                run_id, source_id, source_type, source_subtype, scope_kind,
                title, created_at
             )
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(run_id)
        .bind(source.id)
        .bind(&source.source_type)
        .bind(source.source_subtype.as_deref().unwrap_or("video"))
        .bind(scope_kind)
        .bind(&source.title)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    }

    for (index, video) in preflight.included_videos.iter().enumerate() {
        let source_ref_id = format!("source_ref_{}", index + 1);
        insert_source_snapshot(pool, run_id, video, &source_ref_id, &now).await?;
        insert_material_snapshots(
            pool,
            run_id,
            video.source_id,
            &source_ref_id,
            request.include_comments,
            &now,
        )
        .await?;
    }
    insert_origins(pool, run_id, &request, &preflight, &now).await?;
    insert_stage_skeleton(pool, run_id, preflight.included_videos.len(), &now).await?;

    Ok(run_id)
}

async fn insert_source_snapshot(
    pool: &SqlitePool,
    run_id: i64,
    video: &YoutubeSummaryPreflightVideo,
    source_ref_id: &str,
    now: &str,
) -> AppResult<i64> {
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_run_source_snapshots (
            run_id, source_id, source_ref_id, video_id, title, channel_title,
            published_at, url, created_at
         )
         SELECT ?, yvs.source_id, ?, yvs.video_id, COALESCE(yvs.title, sources.title),
                yvs.channel_title, yvs.published_at, yvs.canonical_url, ?
         FROM youtube_video_sources yvs
         JOIN sources ON sources.id = yvs.source_id
         WHERE yvs.source_id = ?
        ",
    )
    .bind(run_id)
    .bind(source_ref_id)
    .bind(now)
    .bind(video.source_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ? AND source_id = ?",
    )
    .bind(run_id)
    .bind(video.source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

async fn insert_material_snapshots(
    pool: &SqlitePool,
    run_id: i64,
    source_id: i64,
    source_ref_id: &str,
    include_comments: bool,
    now: &str,
) -> AppResult<()> {
    let source_snapshot_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_run_source_snapshots WHERE run_id = ? AND source_id = ?",
    )
    .bind(run_id)
    .bind(source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let transcript = transcript_text_for_source(pool, source_id).await?;
    if !transcript.trim().is_empty() {
        insert_material(
            pool,
            run_id,
            source_snapshot_id,
            &format!("m_{}_transcript", source_ref_id),
            "transcript",
            None,
            0,
            &transcript,
            now,
        )
        .await?;
    }

    if let Some(description) = sqlx::query_scalar::<_, String>(
        "SELECT description FROM youtube_video_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    {
        insert_material(
            pool,
            run_id,
            source_snapshot_id,
            &format!("m_{}_description", source_ref_id),
            "description",
            None,
            1,
            &description,
            now,
        )
        .await?;
    }

    if include_comments {
        for (index, comment) in freeze_comment_material_refs(pool, source_id, test_comment_policy())
            .await?
            .into_iter()
            .enumerate()
        {
            let text = load_comment_text(pool, source_id, comment.external_id.as_deref()).await?;
            insert_material(
                pool,
                run_id,
                source_snapshot_id,
                &comment.material_ref_id,
                "comment",
                comment.external_id.as_deref(),
                10 + index as i64,
                &text,
                now,
            )
            .await?;
        }
    }

    Ok(())
}

async fn insert_material(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: i64,
    material_ref_id: &str,
    material_kind: &str,
    external_id: Option<&str>,
    sequence_index: i64,
    text: &str,
    now: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_run_material_snapshots (
            run_id, source_snapshot_id, material_ref_id, material_kind,
            external_id, sequence_index, text_zstd, token_estimate, created_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .bind(material_ref_id)
    .bind(material_kind)
    .bind(external_id)
    .bind(sequence_index)
    .bind(compress_text(text).map_err(AppError::internal)?)
    .bind(estimate_tokens(text))
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_origins(
    pool: &SqlitePool,
    run_id: i64,
    request: &StartYoutubeSummaryRunRequest,
    preflight: &YoutubeSummaryPreflightResponse,
    now: &str,
) -> AppResult<()> {
    for source_id in &request.source_ids {
        let scope_id: i64 = sqlx::query_scalar(
            "SELECT id FROM prompt_pack_run_scopes
             WHERE run_id = ? AND source_id = ?
             ORDER BY id DESC LIMIT 1",
        )
        .bind(run_id)
        .bind(source_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

        let Some(source) = load_source(pool, *source_id).await? else {
            continue;
        };
        if source.source_subtype.as_deref() == Some("playlist") {
            let rows = sqlx::query_as::<_, (Option<i64>, String)>(
                "SELECT video_source_id, video_id
                 FROM youtube_playlist_items
                 WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
                 ORDER BY position ASC, id ASC",
            )
            .bind(source_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?;
            for (video_source_id, video_id) in rows {
                insert_one_origin(
                    pool,
                    run_id,
                    scope_id,
                    video_source_id,
                    &video_id,
                    preflight,
                    now,
                )
                .await?;
            }
        } else {
            let video_id = sqlx::query_scalar::<_, String>(
                "SELECT video_id FROM youtube_video_sources WHERE source_id = ?",
            )
            .bind(source_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
            insert_one_origin(
                pool,
                run_id,
                scope_id,
                Some(*source_id),
                &video_id,
                preflight,
                now,
            )
            .await?;
        }
    }
    Ok(())
}

async fn insert_one_origin(
    pool: &SqlitePool,
    run_id: i64,
    scope_id: i64,
    video_source_id: Option<i64>,
    video_id: &str,
    preflight: &YoutubeSummaryPreflightResponse,
    now: &str,
) -> AppResult<()> {
    let source_snapshot_id = match video_source_id {
        Some(source_id)
            if preflight
                .included_videos
                .iter()
                .any(|video| video.source_id == source_id) =>
        {
            sqlx::query_scalar::<_, i64>(
                "SELECT id FROM prompt_pack_run_source_snapshots
                 WHERE run_id = ? AND source_id = ?",
            )
            .bind(run_id)
            .bind(source_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?
        }
        _ => None,
    };
    let inclusion_status = if source_snapshot_id.is_some() {
        "included"
    } else {
        "skipped"
    };
    let reason = if source_snapshot_id.is_some() {
        None
    } else {
        Some("not_included")
    };
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_run_source_origins (
            run_id, origin_scope_id, source_snapshot_id, video_source_id,
            video_id, inclusion_status, reason, created_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(scope_id)
    .bind(source_snapshot_id)
    .bind(video_source_id)
    .bind(video_id)
    .bind(inclusion_status)
    .bind(reason)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_stage_skeleton(
    pool: &SqlitePool,
    run_id: i64,
    included_count: usize,
    now: &str,
) -> AppResult<()> {
    let source_ids = sqlx::query_as::<_, (i64, i64)>(
        "SELECT id, source_id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    insert_stage(pool, run_id, None, "source_ingestion", 10, "succeeded", now).await?;
    for (index, (snapshot_id, _)) in source_ids.into_iter().enumerate() {
        insert_stage(
            pool,
            run_id,
            Some(snapshot_id),
            "youtube_summary/transcript_analysis",
            20 + index as i64,
            "pending",
            now,
        )
        .await?;
    }
    for (offset, name) in [
        "segment_extraction",
        "key_point_extraction",
        "quote_extraction",
    ]
    .iter()
    .enumerate()
    {
        insert_stage(
            pool,
            run_id,
            None,
            name,
            100 + offset as i64,
            "not_implemented",
            now,
        )
        .await?;
    }
    let synthesis_status = if included_count > 1 {
        "pending"
    } else {
        "skipped"
    };
    insert_stage(
        pool,
        run_id,
        None,
        SYNTHESIS_STAGE_NAME,
        103,
        synthesis_status,
        now,
    )
    .await?;
    insert_stage(pool, run_id, None, "final_synthesis", 200, "pending", now).await?;
    insert_stage(pool, run_id, None, "validation", 300, "pending", now).await?;

    sqlx::query(
        "UPDATE prompt_pack_runs
         SET progress_total = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(included_count as i64)
    .bind(now)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_stage(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: Option<i64>,
    stage_name: &str,
    stage_order: i64,
    stage_status: &str,
    now: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_stage_runs (
            run_id, source_snapshot_id, stage_name, stage_order, stage_status,
            created_at, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .bind(stage_name)
    .bind(stage_order)
    .bind(stage_status)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) fn test_comment_policy() -> CommentSelectionPolicy {
    CommentSelectionPolicy {
        comment_count_cap: 50,
        comment_token_cap: 4000,
    }
}

pub(crate) async fn freeze_comment_material_refs(
    pool: &SqlitePool,
    source_id: i64,
    policy: CommentSelectionPolicy,
) -> AppResult<Vec<CommentMaterialRef>> {
    let rows = sqlx::query_as::<_, (i64, String, Option<Vec<u8>>)>(
        "SELECT id, external_id, content_zstd
         FROM items
         WHERE source_id = ? AND item_kind = 'youtube_comment'
         ORDER BY published_at IS NULL ASC, published_at ASC, external_id ASC, id ASC
         LIMIT ?",
    )
    .bind(source_id)
    .bind(policy.comment_count_cap as i64)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut refs = Vec::with_capacity(rows.len());
    for (index, (_id, external_id, content_zstd)) in rows.into_iter().enumerate() {
        let text = match content_zstd {
            Some(bytes) => decompress_text(&bytes).unwrap_or_default(),
            None => String::new(),
        };
        refs.push(CommentMaterialRef {
            external_id: Some(external_id),
            material_ref_id: format!("m_comment_{}", index + 1),
            token_estimate: estimate_tokens(&text).min(policy.comment_token_cap),
        });
    }
    Ok(refs)
}

async fn load_comment_text(
    pool: &SqlitePool,
    source_id: i64,
    external_id: Option<&str>,
) -> AppResult<String> {
    let Some(external_id) = external_id else {
        return Ok(String::new());
    };
    let bytes = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT content_zstd FROM items
         WHERE source_id = ? AND item_kind = 'youtube_comment' AND external_id = ?
         LIMIT 1",
    )
    .bind(source_id)
    .bind(external_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;
    Ok(bytes
        .as_deref()
        .and_then(|bytes| decompress_text(bytes).ok())
        .unwrap_or_default())
}
