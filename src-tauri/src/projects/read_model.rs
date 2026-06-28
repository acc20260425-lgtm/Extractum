use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Ready,
    Running,
    NeedsAttention,
    Empty,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_count: i64,
    pub material_count: i64,
    pub status: ProjectStatus,
    pub last_run_at: Option<i64>,
    pub pinned: bool,
    pub archived: bool,
    pub updated_at: i64,
}

#[derive(sqlx::FromRow)]
struct ProjectSummaryRow {
    id: i64,
    name: String,
    description: Option<String>,
    source_count: i64,
    material_count: i64,
    latest_run_status: Option<String>,
    last_run_at: Option<i64>,
    has_active_run: i64,
    pinned: i64,
    archived_at: Option<i64>,
    updated_at: i64,
}

fn project_status(row: &ProjectSummaryRow) -> ProjectStatus {
    if row.has_active_run > 0 {
        ProjectStatus::Running
    } else if row.source_count == 0 {
        ProjectStatus::Empty
    } else if row.latest_run_status.as_deref() == Some("failed") {
        ProjectStatus::NeedsAttention
    } else {
        ProjectStatus::Ready
    }
}

fn map_project_summary(row: ProjectSummaryRow) -> ProjectSummary {
    let status = project_status(&row);
    ProjectSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        source_count: row.source_count,
        material_count: row.material_count,
        status,
        last_run_at: row.last_run_at,
        pinned: row.pinned != 0,
        archived: row.archived_at.is_some(),
        updated_at: row.updated_at,
    }
}

pub(crate) async fn list_research_projects_in_pool(
    pool: &sqlx::SqlitePool,
) -> AppResult<Vec<ProjectSummary>> {
    let rows: Vec<ProjectSummaryRow> = sqlx::query_as(
        r#"
        WITH resolved_material_sources AS (
            SELECT
                ps.project_id,
                CASE
                    WHEN s.source_type = 'youtube'
                     AND s.source_subtype = 'playlist'
                    THEN ypi.video_source_id
                    ELSE ps.source_id
                END AS material_source_id
            FROM project_sources ps
            JOIN sources s ON s.id = ps.source_id
            LEFT JOIN youtube_playlist_items ypi
                ON ypi.playlist_source_id = ps.source_id
               AND ypi.video_source_id IS NOT NULL
               AND ypi.is_removed_from_playlist = 0
            WHERE s.source_type <> 'youtube'
               OR s.source_subtype <> 'playlist'
               OR ypi.video_source_id IS NOT NULL
        ),
        source_counts AS (
            SELECT project_id, COUNT(*) AS source_count
            FROM project_sources
            GROUP BY project_id
        ),
        material_counts AS (
            SELECT
                rms.project_id,
                COUNT(DISTINCT items.id) AS material_count
            FROM resolved_material_sources rms
            JOIN items ON items.source_id = rms.material_source_id
            WHERE items.content_zstd IS NOT NULL
            GROUP BY rms.project_id
        )
        SELECT
            p.id,
            p.name,
            p.description,
            COALESCE(source_counts.source_count, 0) AS source_count,
            COALESCE(material_counts.material_count, 0) AS material_count,
            (
                SELECT ar.status
                FROM analysis_runs ar
                WHERE ar.project_id = p.id
                ORDER BY ar.created_at DESC, ar.id DESC
                LIMIT 1
            ) AS latest_run_status,
            (
                SELECT ar.created_at
                FROM analysis_runs ar
                WHERE ar.project_id = p.id
                ORDER BY ar.created_at DESC, ar.id DESC
                LIMIT 1
            ) AS last_run_at,
            CASE WHEN EXISTS (
                SELECT 1
                FROM analysis_runs ar
                WHERE ar.project_id = p.id
                  AND ar.status IN ('queued', 'running')
            ) THEN 1 ELSE 0 END AS has_active_run,
            p.pinned,
            p.archived_at,
            p.updated_at
        FROM projects p
        LEFT JOIN source_counts ON source_counts.project_id = p.id
        LEFT JOIN material_counts ON material_counts.project_id = p.id
        ORDER BY
            CASE WHEN p.archived_at IS NULL THEN 0 ELSE 1 END ASC,
            p.pinned DESC,
            p.updated_at DESC,
            p.id DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(rows.into_iter().map(map_project_summary).collect())
}

#[tauri::command]
pub async fn list_research_projects(handle: AppHandle) -> AppResult<Vec<ProjectSummary>> {
    let pool = get_pool(&handle).await?;
    list_research_projects_in_pool(&pool).await
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

    async fn seed_project(pool: &sqlx::SqlitePool, id: i64, name: &str, updated_at: i64) {
        sqlx::query(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, NULL, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(updated_at)
        .bind(updated_at)
        .execute(pool)
        .await
        .expect("seed project");
    }

    async fn seed_source(pool: &sqlx::SqlitePool, id: i64, provider: &str, subtype: &str) {
        let account_id = if provider == "telegram" {
            sqlx::query(
                "INSERT OR IGNORE INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'Test account', 1, 'hash', 1)",
            )
            .execute(pool)
            .await
            .expect("seed account");
            Some(1_i64)
        } else {
            None
        };
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at, account_id) VALUES (?, ?, ?, ?, ?, 1, 0, 100, ?)",
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

    async fn attach_source(pool: &sqlx::SqlitePool, project_id: i64, source_id: i64) {
        sqlx::query(
            "INSERT INTO project_sources (project_id, source_id, added_at) VALUES (?, ?, 100)",
        )
        .bind(project_id)
        .bind(source_id)
        .execute(pool)
        .await
        .expect("attach source");
    }

    async fn seed_item(pool: &sqlx::SqlitePool, id: i64, source_id: i64) {
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (?, ?, ?, 'Author', 1000, 1001, x'01', 'telegram_message')",
        )
        .bind(id)
        .bind(source_id)
        .bind(format!("item-{id}"))
        .execute(pool)
        .await
        .expect("seed item");
    }

    async fn seed_run(
        pool: &sqlx::SqlitePool,
        id: i64,
        project_id: i64,
        status: &str,
        created_at: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, project_id, period_from, period_to,
                output_language, prompt_template_id, prompt_template_version,
                provider_profile, provider, model, status, created_at
            )
            VALUES (?, 'report', 'project', ?, 1, 2, 'en', 1, 1, 'default', 'openai', 'gpt', ?, ?)
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(status)
        .bind(created_at)
        .execute(pool)
        .await
        .expect("seed run");
    }

    #[tokio::test]
    async fn list_research_projects_derives_counts_status_and_last_run_without_fanout() {
        let pool = pool().await;
        seed_project(&pool, 1, "Ready", 10).await;
        seed_project(&pool, 2, "Failed", 20).await;
        seed_source(&pool, 10, "telegram", "channel").await;
        seed_source(&pool, 11, "telegram", "channel").await;
        attach_source(&pool, 1, 10).await;
        attach_source(&pool, 1, 11).await;
        attach_source(&pool, 2, 10).await;
        seed_item(&pool, 100, 10).await;
        seed_item(&pool, 101, 10).await;
        seed_item(&pool, 102, 11).await;
        seed_run(&pool, 500, 1, "completed", 1000).await;
        seed_run(&pool, 501, 2, "completed", 1000).await;
        seed_run(&pool, 502, 2, "failed", 1000).await;

        let rows = list_research_projects_in_pool(&pool)
            .await
            .expect("list research projects");

        let ready = rows.iter().find(|row| row.id == 1).expect("ready project");
        assert_eq!(ready.source_count, 2);
        assert_eq!(ready.material_count, 3);
        assert_eq!(ready.status, ProjectStatus::Ready);
        assert_eq!(ready.last_run_at, Some(1000));

        let failed = rows
            .iter()
            .find(|row| row.id == 2)
            .expect("failed project");
        assert_eq!(failed.source_count, 1);
        assert_eq!(failed.material_count, 2);
        assert_eq!(failed.status, ProjectStatus::NeedsAttention);
    }

    #[tokio::test]
    async fn list_research_projects_counts_playlist_linked_video_materials() {
        let pool = pool().await;
        seed_project(&pool, 4, "Playlist", 40).await;
        seed_source(&pool, 40, "youtube", "playlist").await;
        seed_source(&pool, 41, "youtube", "video").await;
        attach_source(&pool, 4, 40).await;
        sqlx::query(
            "INSERT INTO youtube_playlist_items (playlist_source_id, video_source_id, video_id, position, availability_status, is_removed_from_playlist) VALUES (40, 41, 'video-41', 1, 'available', 0)",
        )
        .execute(&pool)
        .await
        .expect("link playlist video");
        seed_item(&pool, 410, 41).await;
        seed_item(&pool, 411, 41).await;

        let rows = list_research_projects_in_pool(&pool)
            .await
            .expect("list research projects");

        let playlist = rows
            .iter()
            .find(|row| row.id == 4)
            .expect("playlist project");
        assert_eq!(playlist.source_count, 1);
        assert_eq!(playlist.material_count, 2);
    }

    #[tokio::test]
    async fn list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first() {
        let pool = pool().await;
        seed_project(&pool, 1, "Archived", 30).await;
        seed_project(&pool, 2, "Pinned", 20).await;
        seed_project(&pool, 3, "Running empty", 10).await;
        sqlx::query("UPDATE projects SET archived_at = 300 WHERE id = 1")
            .execute(&pool)
            .await
            .expect("archive project");
        sqlx::query("UPDATE projects SET pinned = 1 WHERE id = 2")
            .execute(&pool)
            .await
            .expect("pin project");
        seed_run(&pool, 700, 3, "running", 2000).await;

        let rows = list_research_projects_in_pool(&pool)
            .await
            .expect("list research projects");

        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
        let running = rows
            .iter()
            .find(|row| row.id == 3)
            .expect("running project");
        assert_eq!(running.status, ProjectStatus::Running);
        assert_eq!(running.source_count, 0);
    }
}
