use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, serde::Serialize, sqlx::FromRow, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, sqlx::FromRow, PartialEq, Eq)]
pub struct ProjectSourceRecord {
    pub project_id: i64,
    pub source_id: i64,
    pub provider: String,
    pub source_subtype: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub item_count: i64,
    pub added_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct AddProjectSourcesOutcome {
    pub added_count: i64,
    pub already_present_count: i64,
}

fn normalize_project_name(name: &str) -> AppResult<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::validation("Project name cannot be empty"));
    }
    Ok(name)
}

fn normalize_description(description: Option<String>) -> Option<String> {
    description
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn ensure_project_exists(pool: &sqlx::SqlitePool, project_id: i64) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if exists == 0 {
        return Err(AppError::not_found(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}

async fn ensure_sources_exist(pool: &sqlx::SqlitePool, source_ids: &[i64]) -> AppResult<()> {
    if source_ids.is_empty() {
        return Ok(());
    }

    let mut query =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT COUNT(*) FROM sources WHERE id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");

    let found: i64 = query
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if found != source_ids.len() as i64 {
        return Err(AppError::validation(
            "One or more selected sources do not exist",
        ));
    }
    Ok(())
}

pub(crate) async fn list_projects_in_pool(
    pool: &sqlx::SqlitePool,
) -> AppResult<Vec<ProjectRecord>> {
    sqlx::query_as(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM projects
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn create_project_in_pool(
    pool: &sqlx::SqlitePool,
    name: &str,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let name = normalize_project_name(name)?;
    let description = normalize_description(description);
    let now = crate::time::now_secs();

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO projects (name, description, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|error| {
        if error.to_string().to_lowercase().contains("unique") {
            AppError::validation("A project with this name already exists")
        } else {
            AppError::database(error)
        }
    })?;

    get_project_in_pool(pool, id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Project {id} not found after creation")))
}

pub(crate) async fn get_project_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
) -> AppResult<Option<ProjectRecord>> {
    sqlx::query_as(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM projects
        WHERE id = ?
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn update_project_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    name: &str,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let name = normalize_project_name(name)?;
    let description = normalize_description(description);
    let now = crate::time::now_secs();

    let result = sqlx::query(
        r#"
        UPDATE projects
        SET name = ?, description = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(now)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(|error| {
        if error.to_string().to_lowercase().contains("unique") {
            AppError::validation("A project with this name already exists")
        } else {
            AppError::database(error)
        }
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Project {project_id} not found"
        )));
    }

    get_project_in_pool(pool, project_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found after update")))
}

pub(crate) async fn set_project_pinned_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    pinned: bool,
) -> AppResult<()> {
    let now = crate::time::now_secs();
    let result = sqlx::query(
        r#"
        UPDATE projects
        SET pinned = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(if pinned { 1_i64 } else { 0_i64 })
    .bind(now)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}

pub(crate) async fn set_project_archived_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    archived: bool,
) -> AppResult<()> {
    let now = crate::time::now_secs();
    let archived_at = archived.then_some(now);
    let result = sqlx::query(
        r#"
        UPDATE projects
        SET archived_at = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(archived_at)
    .bind(now)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}

pub(crate) async fn add_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    mut source_ids: Vec<i64>,
) -> AppResult<AddProjectSourcesOutcome> {
    ensure_project_exists(pool, project_id).await?;
    source_ids.retain(|source_id| *source_id > 0);
    source_ids.sort_unstable();
    source_ids.dedup();
    ensure_sources_exist(pool, &source_ids).await?;

    let now = crate::time::now_secs();
    let mut added_count = 0;
    let mut already_present_count = 0;
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    for source_id in source_ids {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO project_sources (project_id, source_id, added_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(project_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

        if result.rows_affected() == 0 {
            already_present_count += 1;
        } else {
            added_count += 1;
        }
    }

    sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;
    Ok(AddProjectSourcesOutcome {
        added_count,
        already_present_count,
    })
}

pub(crate) async fn remove_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    mut source_ids: Vec<i64>,
) -> AppResult<()> {
    ensure_project_exists(pool, project_id).await?;
    source_ids.retain(|source_id| *source_id > 0);
    source_ids.sort_unstable();
    source_ids.dedup();

    if source_ids.is_empty() {
        return Ok(());
    }

    let mut query =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("DELETE FROM project_sources WHERE project_id = ");
    query.push_bind(project_id);
    query.push(" AND source_id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in &source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");
    query
        .build()
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
        .bind(crate::time::now_secs())
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    Ok(())
}

pub(crate) async fn list_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
) -> AppResult<Vec<ProjectSourceRecord>> {
    ensure_project_exists(pool, project_id).await?;
    sqlx::query_as(
        r#"
        SELECT
            ps.project_id,
            s.id AS source_id,
            s.source_type AS provider,
            s.source_subtype,
            s.title,
            CASE
                WHEN s.account_id IS NOT NULL THEN 'Account #' || s.account_id
                ELSE NULL
            END AS subtitle,
            COUNT(items.content_zstd) AS item_count,
            ps.added_at
        FROM project_sources ps
        JOIN sources s ON s.id = ps.source_id
        LEFT JOIN items ON items.source_id = s.id
        WHERE ps.project_id = ?
        GROUP BY ps.project_id, s.id, s.source_type, s.source_subtype, s.title, s.account_id, ps.added_at
        ORDER BY ps.added_at DESC, s.id DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn delete_project_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    sqlx::query("DELETE FROM analysis_runs WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    sqlx::query("DELETE FROM project_sources WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    tx.commit().await.map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}

#[tauri::command]
pub async fn list_projects(handle: AppHandle) -> AppResult<Vec<ProjectRecord>> {
    let pool = get_pool(&handle).await?;
    list_projects_in_pool(&pool).await
}

#[tauri::command]
pub async fn create_project(
    handle: AppHandle,
    name: String,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let pool = get_pool(&handle).await?;
    create_project_in_pool(&pool, &name, description).await
}

#[tauri::command]
pub async fn update_project(
    handle: AppHandle,
    project_id: i64,
    name: String,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let pool = get_pool(&handle).await?;
    update_project_in_pool(&pool, project_id, &name, description).await
}

#[tauri::command]
pub async fn delete_project(handle: AppHandle, project_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_project_in_pool(&pool, project_id).await
}

#[tauri::command]
pub async fn set_project_pinned(handle: AppHandle, project_id: i64, pinned: bool) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    set_project_pinned_in_pool(&pool, project_id, pinned).await
}

#[tauri::command]
pub async fn set_project_archived(
    handle: AppHandle,
    project_id: i64,
    archived: bool,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    set_project_archived_in_pool(&pool, project_id, archived).await
}

#[tauri::command]
pub async fn list_project_sources(
    handle: AppHandle,
    project_id: i64,
) -> AppResult<Vec<ProjectSourceRecord>> {
    let pool = get_pool(&handle).await?;
    list_project_sources_in_pool(&pool, project_id).await
}

#[tauri::command]
pub async fn add_project_sources(
    handle: AppHandle,
    project_id: i64,
    source_ids: Vec<i64>,
) -> AppResult<AddProjectSourcesOutcome> {
    let pool = get_pool(&handle).await?;
    add_project_sources_in_pool(&pool, project_id, source_ids).await
}

#[tauri::command]
pub async fn remove_project_sources(
    handle: AppHandle,
    project_id: i64,
    source_ids: Vec<i64>,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    remove_project_sources_in_pool(&pool, project_id, source_ids).await
}

#[tauri::command]
#[expect(
    clippy::too_many_arguments,
    reason = "Tauri command signature mirrors start_analysis_report for project scope."
)]
pub async fn start_project_analysis(
    handle: AppHandle,
    state: tauri::State<'_, crate::analysis::AnalysisState>,
    project_id: i64,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<i64> {
    crate::analysis::report::start_analysis_report_run(
        handle,
        state.inner(),
        crate::analysis::report::StartAnalysisReportRequest {
            source_id: None,
            source_group_id: None,
            project_id: Some(project_id),
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            model_override,
            profile_id,
            youtube_corpus_mode,
            include_migrated_history,
        },
    )
    .await
}

#[tauri::command]
pub async fn list_project_runs(
    handle: AppHandle,
    project_id: i64,
) -> AppResult<Vec<crate::analysis::models::AnalysisRunSummary>> {
    let pool = get_pool(&handle).await?;
    ensure_project_exists(&pool, project_id).await?;
    crate::analysis::store::list_analysis_run_summaries(
        &pool,
        crate::analysis::store::AnalysisRunListFilters {
            source_id: None,
            source_group_id: None,
            project_id: Some(project_id),
            limit: 5,
            query: None,
            status: Some("all".to_string()),
            provider: None,
            model: None,
            template: None,
            date_from: None,
            date_to: None,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::apply_all_migrations_for_test_pool;

    async fn pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    async fn seed_account(pool: &sqlx::SqlitePool, id: i64) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (?, ?, 1, 'hash', 100)",
        )
        .bind(id)
        .bind(format!("Account {id}"))
        .execute(pool)
        .await
        .expect("seed account");
    }

    async fn seed_source(pool: &sqlx::SqlitePool, id: i64, provider: &str, subtype: &str) {
        let account_id = if provider == "telegram" {
            Some(1_i64)
        } else {
            None
        };
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at, account_id
            )
            VALUES (?, ?, ?, ?, ?, 1, 0, 100, ?)
            "#,
        )
        .bind(id)
        .bind(provider)
        .bind(subtype)
        .bind(format!("{provider}-{id}"))
        .bind(format!("Source {id}"))
        .bind(account_id)
        .execute(pool)
        .await
        .expect("seed source");
    }

    #[tokio::test]
    async fn create_project_trims_and_rejects_duplicate_names_case_insensitively() {
        let pool = pool().await;

        let created = create_project_in_pool(&pool, "  Alpha  ", Some("Desc".to_string()))
            .await
            .expect("create project");
        assert_eq!(created.name, "Alpha");
        assert_eq!(created.description.as_deref(), Some("Desc"));

        let duplicate = create_project_in_pool(&pool, "alpha", None)
            .await
            .expect_err("duplicate rejected");
        assert_eq!(duplicate.kind, crate::error::AppErrorKind::Validation);
    }

    #[tokio::test]
    async fn add_project_sources_is_idempotent_and_lists_ui_ready_rows() {
        let pool = pool().await;
        seed_account(&pool, 1).await;
        seed_source(&pool, 10, "youtube", "video").await;
        seed_source(&pool, 11, "telegram", "supergroup").await;
        let project = create_project_in_pool(&pool, "Mixed", None)
            .await
            .expect("create project");

        let first = add_project_sources_in_pool(&pool, project.id, vec![10, 11])
            .await
            .expect("add sources");
        assert_eq!(first.added_count, 2);
        assert_eq!(first.already_present_count, 0);

        let second = add_project_sources_in_pool(&pool, project.id, vec![10, 11])
            .await
            .expect("add sources again");
        assert_eq!(second.added_count, 0);
        assert_eq!(second.already_present_count, 2);

        let sources = list_project_sources_in_pool(&pool, project.id)
            .await
            .expect("list project sources");
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_id, 11);
        assert_eq!(sources[0].provider, "telegram");
        assert_eq!(sources[1].source_id, 10);
        assert_eq!(sources[1].provider, "youtube");
    }

    #[tokio::test]
    async fn set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project() {
        let pool = pool().await;
        let project = create_project_in_pool(&pool, "Pinned", None)
            .await
            .expect("create project");
        sqlx::query("UPDATE projects SET updated_at = 1 WHERE id = ?")
            .bind(project.id)
            .execute(&pool)
            .await
            .expect("seed stale updated_at");

        set_project_pinned_in_pool(&pool, project.id, true)
            .await
            .expect("pin project");
        let row: (i64, i64) =
            sqlx::query_as("SELECT pinned, updated_at FROM projects WHERE id = ?")
                .bind(project.id)
                .fetch_one(&pool)
                .await
                .expect("load pinned project");
        assert_eq!(row.0, 1);
        assert!(row.1 > 1);

        sqlx::query("UPDATE projects SET updated_at = 1 WHERE id = ?")
            .bind(project.id)
            .execute(&pool)
            .await
            .expect("reseed stale updated_at");

        set_project_pinned_in_pool(&pool, project.id, false)
            .await
            .expect("unpin project");
        let row: (i64, i64) =
            sqlx::query_as("SELECT pinned, updated_at FROM projects WHERE id = ?")
                .bind(project.id)
                .fetch_one(&pool)
                .await
                .expect("load unpinned project");
        assert_eq!(row.0, 0);
        assert!(row.1 > 1);

        let missing = set_project_pinned_in_pool(&pool, 404_404, true)
            .await
            .expect_err("missing project rejected");
        assert_eq!(missing.kind, crate::error::AppErrorKind::NotFound);
    }

    #[tokio::test]
    async fn set_project_archived_toggles_timestamp_and_rejects_missing_project() {
        let pool = pool().await;
        let project = create_project_in_pool(&pool, "Archive", None)
            .await
            .expect("create project");
        sqlx::query("UPDATE projects SET updated_at = 1 WHERE id = ?")
            .bind(project.id)
            .execute(&pool)
            .await
            .expect("seed stale updated_at");

        set_project_archived_in_pool(&pool, project.id, true)
            .await
            .expect("archive project");
        let row: (Option<i64>, i64) =
            sqlx::query_as("SELECT archived_at, updated_at FROM projects WHERE id = ?")
                .bind(project.id)
                .fetch_one(&pool)
                .await
                .expect("load archived project");
        assert!(row.0.is_some());
        assert!(row.1 > 1);

        sqlx::query("UPDATE projects SET updated_at = 1 WHERE id = ?")
            .bind(project.id)
            .execute(&pool)
            .await
            .expect("reseed stale updated_at");

        set_project_archived_in_pool(&pool, project.id, false)
            .await
            .expect("restore project");
        let row: (Option<i64>, i64) =
            sqlx::query_as("SELECT archived_at, updated_at FROM projects WHERE id = ?")
                .bind(project.id)
                .fetch_one(&pool)
                .await
                .expect("load restored project");
        assert_eq!(row.0, None);
        assert!(row.1 > 1);

        let missing = set_project_archived_in_pool(&pool, 404_404, true)
            .await
            .expect_err("missing project rejected");
        assert_eq!(missing.kind, crate::error::AppErrorKind::NotFound);
    }

    #[tokio::test]
    async fn delete_project_removes_membership_and_project_runs_but_keeps_sources() {
        let pool = pool().await;
        seed_source(&pool, 10, "youtube", "video").await;
        let project = create_project_in_pool(&pool, "Delete me", None)
            .await
            .expect("create project");
        add_project_sources_in_pool(&pool, project.id, vec![10])
            .await
            .expect("add source");
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, project_id, period_from, period_to,
                output_language, prompt_template_id, prompt_template_version,
                provider_profile, provider, model, status, created_at
            )
            VALUES (500, 'report', 'project', ?, 1, 2, 'en', 1, 1, 'default', 'openai', 'gpt', 'completed', 100)
            "#,
        )
        .bind(project.id)
        .execute(&pool)
        .await
        .expect("seed project run");

        delete_project_in_pool(&pool, project.id)
            .await
            .expect("delete project");

        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await
            .expect("count projects");
        let membership_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_sources")
            .fetch_one(&pool)
            .await
            .expect("count project sources");
        let run_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_runs")
            .fetch_one(&pool)
            .await
            .expect("count runs");
        let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources")
            .fetch_one(&pool)
            .await
            .expect("count sources");

        assert_eq!(project_count, 0);
        assert_eq!(membership_count, 0);
        assert_eq!(run_count, 0);
        assert_eq!(source_count, 1);
    }
}
