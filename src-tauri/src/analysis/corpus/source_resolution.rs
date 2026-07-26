use std::collections::HashSet;

use sqlx::{Pool, QueryBuilder, Sqlite};

use crate::error::{AppError, AppResult};
#[cfg(test)]
use extractum_analysis::AnalysisRunDetail;
use extractum_analysis::{
    get_analysis_source_group_record, AnalysisSourceKind, ResolvedAnalysisScope,
};

pub(crate) struct AppAnalysisScopeResolution {
    scope: ResolvedAnalysisScope,
    skipped_unlinked_playlist_items: usize,
}

impl std::fmt::Debug for AppAnalysisScopeResolution {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppAnalysisScopeResolution")
            .field(
                "skipped_unlinked_playlist_items",
                &self.skipped_unlinked_playlist_items,
            )
            .finish_non_exhaustive()
    }
}

impl AppAnalysisScopeResolution {
    pub(crate) fn scope(&self) -> &ResolvedAnalysisScope {
        &self.scope
    }
    #[cfg(test)]
    pub(crate) fn skipped_unlinked_playlist_items(&self) -> usize {
        self.skipped_unlinked_playlist_items
    }
    pub(crate) fn into_scope(self) -> ResolvedAnalysisScope {
        self.scope
    }
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
    title: Option<String>,
}

async fn load_source_scope_row(
    pool: &Pool<Sqlite>,
    source_id: i64,
) -> AppResult<AnalysisSourceScopeRow> {
    sqlx::query_as(
        r#"
        SELECT id, source_type, source_subtype, title
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

async fn load_source_scope_rows(
    pool: &Pool<Sqlite>,
    source_ids: &[i64],
) -> AppResult<Vec<AnalysisSourceScopeRow>> {
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id, source_type, source_subtype, title FROM sources WHERE id IN (",
    );
    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(") ORDER BY COALESCE(title, ''), id");

    let rows: Vec<AnalysisSourceScopeRow> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    if rows.len() != source_ids.len() {
        let loaded_ids = rows.iter().map(|row| row.id).collect::<HashSet<_>>();
        if let Some(source_id) = source_ids
            .iter()
            .find(|source_id| !loaded_ids.contains(source_id))
        {
            return Err(AppError::not_found(format!("Source {source_id} not found")));
        }
    }
    Ok(rows)
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
) -> Result<AppAnalysisScopeResolution, AnalysisSourceResolutionError> {
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
    let scope_label: String;
    let mut source_ids = Vec::new();
    let mut seen_source_ids = HashSet::new();
    let mut skipped_unlinked_playlist_items = 0usize;

    if let Some(source_id) = source_id {
        let source = load_source_scope_row(pool, source_id).await?;
        source_type = source.source_type.clone();
        scope_label = source
            .title
            .clone()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| format!("Source {source_id}"));
        push_scope_source(
            pool,
            source,
            &mut source_ids,
            &mut seen_source_ids,
            &mut skipped_unlinked_playlist_items,
        )
        .await?;
    } else if let Some(group_id) = source_group_id {
        let group = get_analysis_source_group_record(pool, group_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!("Analysis source group {group_id} not found"))
            })?;
        source_type = match group.source_kind() {
            AnalysisSourceKind::Telegram => "telegram",
            AnalysisSourceKind::Youtube => "youtube",
        }
        .to_string();
        scope_label = group.name().to_string();

        if group.member_source_ids().is_empty() {
            return Err(AppError::validation(
                "The selected source group does not contain any sources",
            )
            .into());
        }

        let sources = load_source_scope_rows(pool, group.member_source_ids()).await?;
        for source in sources {
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
        scope_label = sqlx::query_scalar::<_, String>("SELECT name FROM projects WHERE id = ?")
            .bind(project_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found")))?;
        let rows: Vec<AnalysisSourceScopeRow> = sqlx::query_as(
            r#"
            SELECT s.id, s.source_type, s.source_subtype, s.title
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

    let source_kind = match source_type.as_str() {
        "telegram" => AnalysisSourceKind::Telegram,
        "youtube" => AnalysisSourceKind::Youtube,
        other => {
            return Err(AppError::validation(format!(
                "Unsupported analysis corpus source_type '{other}'"
            ))
            .into())
        }
    };
    let scope = if let Some(source_id) = source_id {
        ResolvedAnalysisScope::for_source(source_id, source_kind, source_ids, scope_label)
    } else if let Some(group_id) = source_group_id {
        ResolvedAnalysisScope::for_source_group(group_id, source_kind, source_ids, scope_label)
    } else {
        ResolvedAnalysisScope::for_project(
            project_id.expect("validated project"),
            source_kind,
            source_ids,
            scope_label,
        )
    }
    .map_err(AnalysisSourceResolutionError::from)?;
    Ok(AppAnalysisScopeResolution {
        scope,
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

    if run.scope_type == "single_source" {
        let source_id = run
            .source_id
            .ok_or_else(|| format!("Analysis run {} is missing source_id", run.id))?;
        return Ok(vec![source_id]);
    }

    if run.scope_type == "source_group" {
        let group_id = run
            .source_group_id
            .ok_or_else(|| format!("Analysis run {} is missing source_group_id", run.id))?;
        let group = get_analysis_source_group_record(pool, group_id)
            .await?
            .ok_or_else(|| format!("Analysis source group {group_id} not found"))?;
        return Ok(group.member_source_ids().to_vec());
    }

    if run.scope_type == "project" {
        let project_id = run
            .project_id
            .ok_or_else(|| format!("Analysis run {} is missing project_id", run.id))?;
        return resolve_analysis_sources(pool, None, None, Some(project_id))
            .await
            .map(|resolved| resolved.into_scope().source_ids().to_vec())
            .map_err(|error| error.into_app_error().to_string());
    }

    Err(format!("Unsupported analysis scope '{}'", run.scope_type))
}

#[cfg(test)]
mod tests {
    use super::resolve_analysis_sources;
    use crate::migrations::apply_all_migrations_for_test_pool;

    #[tokio::test]
    async fn source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, created_at
            )
            VALUES
                (10, 'youtube', 'playlist', 'playlist-10', 'Charlie playlist', 1),
                (20, 'youtube', 'video', 'video-20', 'Bravo video', 1),
                (21, 'youtube', 'video', 'video-21', 'Linked video 21', 1),
                (30, 'youtube', 'video', 'video-30', 'Linked video 30', 1),
                (40, 'youtube', 'video', 'video-40', 'Alpha video', 1),
                (50, 'youtube', 'video', 'video-50', '', 1),
                (60, 'youtube', 'video', 'video-60', NULL, 1),
                (70, 'youtube', 'video', 'video-70', 'Delta video', 1),
                (71, 'youtube', 'video', 'video-71', 'Delta video', 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert youtube sources");
        sqlx::query(
            r#"
            INSERT INTO analysis_source_groups (
                id, name, source_type, created_at, updated_at
            )
            VALUES (9, 'Ordered YouTube group', 'youtube', 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert youtube source group");
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES
                (9, 71, 1),
                (9, 60, 1),
                (9, 50, 1),
                (9, 10, 1),
                (9, 20, 1),
                (9, 40, 1),
                (9, 70, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert group members");
        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_items (
                playlist_source_id, video_source_id, video_id, position,
                availability_status, is_removed_from_playlist
            )
            VALUES
                (10, 30, 'video-30', 1, 'available', 0),
                (10, 21, 'video-21', 2, 'available', 0)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert playlist members");

        let resolved = resolve_analysis_sources(&pool, None, Some(9), None)
            .await
            .expect("resolve source group");

        assert_eq!(
            resolved.scope().source_ids(),
            &[50, 60, 40, 20, 30, 21, 70, 71]
        );
    }
}
