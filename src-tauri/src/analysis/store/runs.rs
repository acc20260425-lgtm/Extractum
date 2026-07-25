use std::collections::{BTreeSet, HashSet};

use sqlx::{Pool, QueryBuilder, Row, Sqlite, SqliteConnection, SqlitePool};

use super::super::corpus::YoutubeCorpusMode;
use super::super::models::AnalysisPromptTemplate;
use super::super::state::AnalysisState;
use super::super::{
    now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
};
use super::read_model::load_analysis_run_status;
use extractum_core::error::{AppError, AppResult};

pub struct AnalysisRunDiagnosticCount {
    provider: String,
    run_type: String,
    scope_type: String,
    status: String,
    snapshot_state: String,
    error_kind: String,
    count: i64,
}

impl AnalysisRunDiagnosticCount {
    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn run_type(&self) -> &str {
        &self.run_type
    }

    pub fn scope_type(&self) -> &str {
        &self.scope_type
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn snapshot_state(&self) -> &str {
        &self.snapshot_state
    }

    pub fn error_kind(&self) -> &str {
        &self.error_kind
    }

    pub fn count(&self) -> i64 {
        self.count
    }
}

#[derive(sqlx::FromRow)]
struct AnalysisRunScopeRow {
    id: i64,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
}

pub async fn analysis_run_ids_depending_on_sources(
    pool: &SqlitePool,
    candidate_run_ids: &HashSet<i64>,
    owned_source_ids: &[i64],
) -> AppResult<BTreeSet<i64>> {
    if candidate_run_ids.is_empty() || owned_source_ids.is_empty() {
        return Ok(BTreeSet::new());
    }

    let owned = owned_source_ids.iter().copied().collect::<HashSet<_>>();
    let mut blocked = BTreeSet::new();
    let rows = sqlx::query_as::<_, AnalysisRunScopeRow>(
        "SELECT id, source_id, source_group_id FROM analysis_runs ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        if !candidate_run_ids.contains(&row.id) {
            continue;
        }
        if row
            .source_id
            .is_some_and(|source_id| owned.contains(&source_id))
        {
            blocked.insert(row.id);
            continue;
        }
        if let Some(group_id) = row.source_group_id {
            if group_has_owned_source(pool, group_id, &owned).await? {
                blocked.insert(row.id);
            }
        }
    }
    Ok(blocked)
}

async fn group_has_owned_source(
    pool: &SqlitePool,
    group_id: i64,
    owned_source_ids: &HashSet<i64>,
) -> AppResult<bool> {
    let source_ids = sqlx::query_scalar::<_, i64>(
        "SELECT source_id FROM analysis_source_group_members WHERE group_id = ?",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    Ok(source_ids
        .into_iter()
        .any(|source_id| owned_source_ids.contains(&source_id)))
}

pub async fn load_analysis_run_diagnostics(
    pool: &SqlitePool,
) -> AppResult<Vec<AnalysisRunDiagnosticCount>> {
    // Raw analysis error text is read only to derive a coarse error_kind.
    // It must never be selected into, copied into, or summarized in the DTO.
    let rows = sqlx::query(
        "SELECT
            provider,
            run_type,
            scope_type,
            status,
            CASE
                WHEN snapshot_captured_at IS NOT NULL THEN 'captured'
                WHEN snapshot_error IS NOT NULL THEN 'failed'
                ELSE 'not_captured'
            END AS snapshot_state,
            CASE
                WHEN error IS NULL OR TRIM(error) = '' THEN 'none'
                WHEN LOWER(error) LIKE '%timeout%' OR LOWER(error) LIKE '%network%' THEN 'network'
                WHEN LOWER(error) LIKE '%unauthorized%' OR LOWER(error) LIKE '%forbidden%' OR LOWER(error) LIKE '%api key%' THEN 'auth'
                WHEN LOWER(error) LIKE '%invalid%' THEN 'validation'
                ELSE 'internal'
            END AS error_kind,
            COUNT(*) AS count
         FROM analysis_runs
         GROUP BY provider, run_type, scope_type, status, snapshot_state, error_kind
         ORDER BY provider, run_type, scope_type, status, snapshot_state, error_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(AnalysisRunDiagnosticCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                run_type: row.try_get("run_type").map_err(AppError::database)?,
                scope_type: row.try_get("scope_type").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                snapshot_state: row
                    .try_get("snapshot_state")
                    .map_err(AppError::database)?,
                error_kind: row.try_get("error_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

pub struct ProjectAnalysisRunAggregate {
    project_id: i64,
    latest_run_status: Option<String>,
    last_run_at: Option<i64>,
    has_active_run: bool,
}

impl ProjectAnalysisRunAggregate {
    pub fn project_id(&self) -> i64 {
        self.project_id
    }

    pub fn latest_run_status(&self) -> Option<&str> {
        self.latest_run_status.as_deref()
    }

    pub fn last_run_at(&self) -> Option<i64> {
        self.last_run_at
    }

    pub fn has_active_run(&self) -> bool {
        self.has_active_run
    }
}

#[derive(sqlx::FromRow)]
struct ProjectAnalysisRunAggregateRow {
    project_id: i64,
    latest_run_status: Option<String>,
    last_run_at: Option<i64>,
    has_active_run: i64,
}

pub async fn load_project_analysis_run_aggregates(
    conn: &mut SqliteConnection,
    project_ids: &[i64],
) -> AppResult<Vec<ProjectAnalysisRunAggregate>> {
    if project_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            runs.project_id,
            (
                SELECT latest.status
                FROM analysis_runs latest
                WHERE latest.project_id = runs.project_id
                ORDER BY latest.created_at DESC, latest.id DESC
                LIMIT 1
            ) AS latest_run_status,
            (
                SELECT latest.created_at
                FROM analysis_runs latest
                WHERE latest.project_id = runs.project_id
                ORDER BY latest.created_at DESC, latest.id DESC
                LIMIT 1
            ) AS last_run_at,
            MAX(
                CASE
                    WHEN runs.status IN ('queued', 'running') THEN 1
                    ELSE 0
                END
            ) AS has_active_run
        FROM analysis_runs runs
        WHERE runs.project_id IN (
        "#,
    );
    {
        let mut separated = query.separated(", ");
        for project_id in project_ids {
            separated.push_bind(*project_id);
        }
    }
    query.push(
        r#"
        )
        GROUP BY runs.project_id
        ORDER BY runs.project_id ASC
        "#,
    );

    let rows = query
        .build_query_as::<ProjectAnalysisRunAggregateRow>()
        .fetch_all(conn)
        .await
        .map_err(AppError::database)?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectAnalysisRunAggregate {
            project_id: row.project_id,
            latest_run_status: row.latest_run_status,
            last_run_at: row.last_run_at,
            has_active_run: row.has_active_run != 0,
        })
        .collect())
}

pub async fn delete_project_analysis_runs(
    conn: &mut SqliteConnection,
    project_id: i64,
) -> AppResult<()> {
    sqlx::query("DELETE FROM analysis_runs WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub(crate) struct DuplicateRunLookup<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template_id: i64,
    pub(crate) provider_profile: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) telegram_history_scope: &'a str,
}

pub(crate) async fn find_active_duplicate_run(
    pool: &Pool<Sqlite>,
    lookup: &DuplicateRunLookup<'_>,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM analysis_runs
        WHERE run_type = ?
          AND scope_type = ?
          AND (source_id = ? OR (source_id IS NULL AND ? IS NULL))
          AND (source_group_id = ? OR (source_group_id IS NULL AND ? IS NULL))
          AND (project_id = ? OR (project_id IS NULL AND ? IS NULL))
          AND period_from = ?
          AND period_to = ?
          AND output_language = ?
          AND prompt_template_id = ?
          AND provider_profile = ?
          AND model = ?
          AND youtube_corpus_mode = ?
          AND COALESCE(telegram_history_scope, 'current') = ?
          AND status IN (?, ?)
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(lookup.scope_type)
    .bind(lookup.source_id)
    .bind(lookup.source_id)
    .bind(lookup.source_group_id)
    .bind(lookup.source_group_id)
    .bind(lookup.project_id)
    .bind(lookup.project_id)
    .bind(lookup.period_from)
    .bind(lookup.period_to)
    .bind(lookup.output_language)
    .bind(lookup.prompt_template_id)
    .bind(lookup.provider_profile)
    .bind(lookup.model)
    .bind(lookup.youtube_corpus_mode.as_wire())
    .bind(lookup.telegram_history_scope)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) struct AnalysisRunInsert<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template: &'a AnalysisPromptTemplate,
    pub(crate) provider_profile: &'a str,
    pub(crate) provider: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) telegram_history_scope: &'a str,
    pub(crate) scope_label_snapshot: Option<&'a str>,
}

pub(crate) async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    insert: &AnalysisRunInsert<'_>,
) -> AppResult<i64> {
    let created_at = now_secs();
    sqlx::query_scalar(
        r#"
        INSERT INTO analysis_runs (
            run_type,
            scope_type,
            source_id,
            source_group_id,
            project_id,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            youtube_corpus_mode,
            telegram_history_scope,
            status,
            scope_label_snapshot,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(insert.scope_type)
    .bind(insert.source_id)
    .bind(insert.source_group_id)
    .bind(insert.project_id)
    .bind(insert.period_from)
    .bind(insert.period_to)
    .bind(insert.output_language)
    .bind(insert.prompt_template.id)
    .bind(insert.prompt_template.version)
    .bind(insert.provider_profile)
    .bind(insert.provider)
    .bind(insert.model)
    .bind(insert.youtube_corpus_mode.as_wire())
    .bind(insert.telegram_history_scope)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(insert.scope_label_snapshot)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn set_run_status(
    pool: &Pool<Sqlite>,
    run_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<&[u8]>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            result_markdown = COALESCE(?, result_markdown),
            trace_data_zstd = COALESCE(?, trace_data_zstd),
            error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(error)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_chat_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    let deleted = sqlx::query("DELETE FROM analysis_runs WHERE id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected();

    if deleted == 0 {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }

    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

pub async fn delete_analysis_run(
    pool: &SqlitePool,
    state: &AnalysisState,
    run_id: i64,
) -> AppResult<()> {
    let status = load_analysis_run_status(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;
    if status == ANALYSIS_STATUS_QUEUED || status == ANALYSIS_STATUS_RUNNING {
        return Err(AppError::conflict(
            "Queued or running analysis runs cannot be deleted",
        ));
    }

    delete_saved_run(pool, run_id).await?;
    state.remove_active_report_run(run_id).await;
    Ok(())
}
