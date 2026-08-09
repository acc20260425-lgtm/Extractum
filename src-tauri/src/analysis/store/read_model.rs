use std::collections::HashSet;

use extractum_analysis::{
    prepare_active_analysis_run_summaries, prepare_analysis_run_detail,
    prepare_analysis_run_summaries, prepare_legacy_analysis_chat_run, AnalysisChatRun,
    AnalysisForeignLabelMatch, AnalysisForeignLabelRef, AnalysisForeignLabels,
    AnalysisProjectLabel, AnalysisRunDetail, AnalysisRunListFilters, AnalysisRunSummary,
    AnalysisSourceLabel,
};
use extractum_core::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};

fn escaped_foreign_label_like_contains(value: &str) -> String {
    format!(
        "%{}%",
        value
            .trim()
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

async fn match_foreign_labels(
    conn: &mut SqliteConnection,
    terms: &[String],
) -> AppResult<Vec<AnalysisForeignLabelMatch>> {
    let mut matches = Vec::with_capacity(terms.len());
    for term in terms {
        let pattern = escaped_foreign_label_like_contains(term);
        let source_ids = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM sources WHERE lower(coalesce(title, '')) LIKE ? ESCAPE '\\' ORDER BY id",
        )
        .bind(&pattern)
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;
        let project_ids = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM projects WHERE lower(coalesce(name, '')) LIKE ? ESCAPE '\\' ORDER BY id",
        )
        .bind(&pattern)
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;
        matches.push(AnalysisForeignLabelMatch::new(
            term.clone(),
            source_ids,
            project_ids,
        )?);
    }
    Ok(matches)
}

async fn load_foreign_labels(
    conn: &mut SqliteConnection,
    refs: Vec<AnalysisForeignLabelRef>,
) -> AppResult<AnalysisForeignLabels> {
    let mut source_ids = Vec::new();
    let mut project_ids = Vec::new();
    for reference in refs {
        match reference {
            AnalysisForeignLabelRef::Source(id) => source_ids.push(id),
            AnalysisForeignLabelRef::Project(id) => project_ids.push(id),
        }
    }

    let mut source_rows = Vec::<(i64, Option<String>)>::new();
    if !source_ids.is_empty() {
        let mut query = QueryBuilder::<Sqlite>::new("SELECT id, title FROM sources WHERE id IN (");
        {
            let mut separated = query.separated(", ");
            for id in &source_ids {
                separated.push_bind(*id);
            }
        }
        query.push(")");
        source_rows = query
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .map_err(AppError::database)?;
    }

    let mut project_rows = Vec::<(i64, Option<String>)>::new();
    if !project_ids.is_empty() {
        let mut query = QueryBuilder::<Sqlite>::new("SELECT id, name FROM projects WHERE id IN (");
        {
            let mut separated = query.separated(", ");
            for id in &project_ids {
                separated.push_bind(*id);
            }
        }
        query.push(")");
        project_rows = query
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .map_err(AppError::database)?;
    }

    let sources = source_ids
        .into_iter()
        .map(|id| {
            let title = source_rows
                .iter()
                .find(|row| row.0 == id)
                .and_then(|row| row.1.clone());
            AnalysisSourceLabel::new(id, title)
        })
        .collect::<AppResult<Vec<_>>>()?;
    let projects = project_ids
        .into_iter()
        .map(|id| {
            let name = project_rows
                .iter()
                .find(|row| row.0 == id)
                .and_then(|row| row.1.clone());
            AnalysisProjectLabel::new(id, name)
        })
        .collect::<AppResult<Vec<_>>>()?;
    AnalysisForeignLabels::new(sources, projects)
}

pub(crate) async fn list_analysis_runs_in_pool(
    pool: &SqlitePool,
    filters: AnalysisRunListFilters,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let matches =
        match_foreign_labels(&mut *transaction, filters.foreign_label_search_terms()).await?;
    let enrichment = prepare_analysis_run_summaries(&mut *transaction, filters, matches).await?;
    let labels = load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs()).await?;
    let runs = enrichment.finish(labels)?;
    transaction.commit().await.map_err(AppError::database)?;
    Ok(runs)
}

pub(crate) async fn list_active_analysis_runs_in_pool(
    pool: &SqlitePool,
    run_ids: &HashSet<i64>,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let enrichment = prepare_active_analysis_run_summaries(&mut *transaction, run_ids).await?;
    let labels = load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs()).await?;
    let runs = enrichment.finish(labels)?;
    transaction.commit().await.map_err(AppError::database)?;
    Ok(runs)
}

pub(crate) async fn get_analysis_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<AnalysisRunDetail>> {
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let enrichment = prepare_analysis_run_detail(&mut *transaction, run_id).await?;
    let labels = load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs()).await?;
    let run = enrichment.finish(labels)?;
    transaction.commit().await.map_err(AppError::database)?;
    Ok(run)
}

pub(crate) async fn resolve_legacy_analysis_chat_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<AnalysisChatRun> {
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let enrichment = prepare_legacy_analysis_chat_run(&mut *transaction, run_id).await?;
    let labels = load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs()).await?;
    let run = enrichment
        .finish(labels)?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;
    transaction.commit().await.map_err(AppError::database)?;
    Ok(run)
}
