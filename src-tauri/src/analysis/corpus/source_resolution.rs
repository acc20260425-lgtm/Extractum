use std::collections::HashSet;

use sqlx::{Pool, Sqlite};

#[cfg(test)]
use super::super::{
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
};
#[cfg(test)]
use crate::analysis::models::AnalysisRunDetail;
use crate::analysis::store::fetch_source_group;
use crate::error::{AppError, AppResult};

#[derive(Debug)]
pub(crate) struct ResolvedAnalysisSources {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    #[allow(dead_code)]
    pub(crate) skipped_unlinked_playlist_items: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnalysisSourceResolutionErrorCode {
    MixedProviderProject,
    NoLinkedYoutubeVideos,
}

impl AnalysisSourceResolutionErrorCode {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::MixedProviderProject => "mixed_provider_project_runs_not_supported",
            Self::NoLinkedYoutubeVideos => {
                "No linked YouTube videos are available for analysis in this scope"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisSourceResolutionError {
    code: Option<AnalysisSourceResolutionErrorCode>,
    error: AppError,
}

impl AnalysisSourceResolutionError {
    pub(crate) fn validation(code: AnalysisSourceResolutionErrorCode) -> Self {
        Self {
            code: Some(code),
            error: AppError::validation(code.message()),
        }
    }

    pub(crate) fn code(&self) -> Option<AnalysisSourceResolutionErrorCode> {
        self.code
    }

    pub(crate) fn into_app_error(self) -> AppError {
        self.error
    }
}

impl From<AppError> for AnalysisSourceResolutionError {
    fn from(error: AppError) -> Self {
        Self { code: None, error }
    }
}

#[derive(sqlx::FromRow)]
struct AnalysisSourceScopeRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
}

async fn load_source_scope_row(
    pool: &Pool<Sqlite>,
    source_id: i64,
) -> AppResult<AnalysisSourceScopeRow> {
    sqlx::query_as(
        r#"
        SELECT id, source_type, source_subtype
        FROM sources
        WHERE id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))
}

async fn linked_playlist_video_source_ids(
    pool: &Pool<Sqlite>,
    playlist_source_id: i64,
) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        r#"
        SELECT video_source_id
        FROM youtube_playlist_items
        WHERE playlist_source_id = ?
          AND video_source_id IS NOT NULL
          AND is_removed_from_playlist = 0
        ORDER BY COALESCE(position, 9223372036854775807), video_id
        "#,
    )
    .bind(playlist_source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

async fn count_skipped_unlinked_playlist_items(
    pool: &Pool<Sqlite>,
    playlist_source_id: i64,
) -> AppResult<usize> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM youtube_playlist_items
        WHERE playlist_source_id = ?
          AND video_source_id IS NULL
          AND is_removed_from_playlist = 0
        "#,
    )
    .bind(playlist_source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(count.max(0) as usize)
}

pub(crate) async fn resolve_analysis_sources(
    pool: &Pool<Sqlite>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<ResolvedAnalysisSources, AnalysisSourceResolutionError> {
    let selected_count = [
        source_id.is_some(),
        source_group_id.is_some(),
        project_id.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if selected_count != 1 {
        return Err(AppError::validation("Select exactly one analysis scope").into());
    }

    let source_type: String;
    let mut source_ids = Vec::new();
    let mut seen_source_ids = HashSet::new();
    let mut skipped_unlinked_playlist_items = 0usize;

    if let Some(source_id) = source_id {
        let source = load_source_scope_row(pool, source_id).await?;
        source_type = source.source_type.clone();
        push_scope_source(
            pool,
            source,
            &mut source_ids,
            &mut seen_source_ids,
            &mut skipped_unlinked_playlist_items,
        )
        .await?;
    } else if let Some(group_id) = source_group_id {
        let group = fetch_source_group(pool, group_id).await?.ok_or_else(|| {
            AppError::not_found(format!("Analysis source group {group_id} not found"))
        })?;
        source_type = group.source_type.clone();

        for member in group.members {
            let source = load_source_scope_row(pool, member.source_id).await?;
            push_scope_source(
                pool,
                source,
                &mut source_ids,
                &mut seen_source_ids,
                &mut skipped_unlinked_playlist_items,
            )
            .await?;
        }
    } else {
        let project_id = project_id.expect("validated project_id");
        let rows: Vec<AnalysisSourceScopeRow> = sqlx::query_as(
            r#"
            SELECT s.id, s.source_type, s.source_subtype
            FROM project_sources ps
            JOIN sources s ON s.id = ps.source_id
            WHERE ps.project_id = ?
            ORDER BY ps.added_at ASC, s.id ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

        if rows.is_empty() {
            return Err(AppError::validation("Project does not contain any sources").into());
        }

        let first_type = rows[0].source_type.clone();
        if rows.iter().any(|row| row.source_type != first_type) {
            return Err(AnalysisSourceResolutionError::validation(
                AnalysisSourceResolutionErrorCode::MixedProviderProject,
            ));
        }
        source_type = first_type;

        for source in rows {
            push_scope_source(
                pool,
                source,
                &mut source_ids,
                &mut seen_source_ids,
                &mut skipped_unlinked_playlist_items,
            )
            .await?;
        }
    }

    if source_type == "youtube" && source_ids.is_empty() {
        return Err(AnalysisSourceResolutionError::validation(
            AnalysisSourceResolutionErrorCode::NoLinkedYoutubeVideos,
        ));
    }

    Ok(ResolvedAnalysisSources {
        source_type,
        source_ids,
        skipped_unlinked_playlist_items,
    })
}

async fn push_scope_source(
    pool: &Pool<Sqlite>,
    source: AnalysisSourceScopeRow,
    source_ids: &mut Vec<i64>,
    seen_source_ids: &mut HashSet<i64>,
    skipped_unlinked_playlist_items: &mut usize,
) -> AppResult<()> {
    if source.source_type == "youtube" && source.source_subtype.as_deref() == Some("playlist") {
        *skipped_unlinked_playlist_items +=
            count_skipped_unlinked_playlist_items(pool, source.id).await?;
        for video_source_id in linked_playlist_video_source_ids(pool, source.id).await? {
            if seen_source_ids.insert(video_source_id) {
                source_ids.push(video_source_id);
            }
        }
    } else if seen_source_ids.insert(source.id) {
        source_ids.push(source.id);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) async fn resolve_run_source_ids(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<i64>, String> {
    let snapshot_source_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT DISTINCT source_id
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY source_id ASC
        "#,
    )
    .bind(run.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if !snapshot_source_ids.is_empty() {
        return Ok(snapshot_source_ids);
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE {
        let source_id = run
            .source_id
            .ok_or_else(|| format!("Analysis run {} is missing source_id", run.id))?;
        return Ok(vec![source_id]);
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
        let group_id = run
            .source_group_id
            .ok_or_else(|| format!("Analysis run {} is missing source_group_id", run.id))?;
        let group = fetch_source_group(pool, group_id)
            .await?
            .ok_or_else(|| format!("Analysis source group {group_id} not found"))?;
        return Ok(group
            .members
            .into_iter()
            .map(|member| member.source_id)
            .collect());
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_PROJECT {
        let project_id = run
            .project_id
            .ok_or_else(|| format!("Analysis run {} is missing project_id", run.id))?;
        return resolve_analysis_sources(pool, None, None, Some(project_id))
            .await
            .map(|resolved| resolved.source_ids)
            .map_err(|error| error.into_app_error().to_string());
    }

    Err(format!("Unsupported analysis scope '{}'", run.scope_type))
}
