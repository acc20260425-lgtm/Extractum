use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use tauri::AppHandle;

const DEFAULT_LIMIT: u32 = 100;
const MAX_LIMIT: u32 = 500;
const JOBS_TABLE: &str = "Jobs";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobsListRequest {
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub search: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobsListResponse {
    pub jobs: Vec<ApalisJobRow>,
    pub total_matching: u32,
    pub status_counts: Vec<ApalisJobStatusCount>,
    pub job_type_counts: Vec<ApalisJobTypeCount>,
    pub refreshed_at: String,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobRow {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub attempts: u32,
    pub max_attempts: Option<u32>,
    pub run_at: Option<String>,
    pub lock_at: Option<String>,
    pub lock_by: Option<String>,
    pub done_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub priority: Option<u32>,
    pub idempotency_key: Option<String>,
    pub job_preview: Option<String>,
    pub job_truncated: bool,
    pub job_json: Option<Value>,
    pub last_result: Option<Value>,
    pub last_result_truncated: bool,
    pub metadata: Option<Value>,
    pub metadata_truncated: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobStatusCount {
    pub status: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobTypeCount {
    pub job_type: String,
    pub count: u32,
}

#[tauri::command]
pub(crate) async fn apalis_jobs_list(
    handle: AppHandle,
    request: Option<ApalisJobsListRequest>,
) -> crate::error::AppResult<ApalisJobsListResponse> {
    let pool = crate::db::get_pool(&handle).await?;
    apalis_jobs_list_from_pool(&pool, request.unwrap_or_default()).await
}

async fn apalis_jobs_list_from_pool(
    _pool: &SqlitePool,
    request: ApalisJobsListRequest,
) -> crate::error::AppResult<ApalisJobsListResponse> {
    Ok(empty_response(normalized_limit(request.limit)))
}

fn normalized_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn empty_response(limit: u32) -> ApalisJobsListResponse {
    ApalisJobsListResponse {
        jobs: Vec::new(),
        total_matching: 0,
        status_counts: Vec::new(),
        job_type_counts: Vec::new(),
        refreshed_at: crate::time::now_rfc3339_utc(),
        limit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("open memory sqlite")
    }

    fn test_job(run_id: &str) -> crate::gemini_browser::GeminiBrowserJob {
        crate::gemini_browser::GeminiBrowserJob {
            run_id: run_id.to_string(),
            prompt: format!("Prompt for {run_id}"),
            source: "apalis-jobs-test".to_string(),
            artifact_mode: crate::gemini_browser::GeminiBrowserArtifactMode::Reduced,
            browser_config: None,
        }
    }

    async fn seed_apalis_job(pool: &SqlitePool, run_id: &str) {
        crate::gemini_browser::setup_gemini_browser_apalis_storage(pool)
            .await
            .expect("setup apalis sqlite storage");
        let mut storage = crate::gemini_browser::open_gemini_browser_job_storage(pool)
            .await
            .expect("open gemini browser storage");
        crate::gemini_browser::enqueue_gemini_browser_job_to_storage(&mut storage, test_job(run_id))
            .await
            .expect("enqueue apalis task");
    }

    async fn table_columns(pool: &SqlitePool, table: &str) -> Vec<String> {
        sqlx::query(&format!("PRAGMA table_info('{table}')"))
            .fetch_all(pool)
            .await
            .expect("read table info")
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect()
    }

    #[tokio::test]
    async fn apalis_jobs_schema_probe_documents_local_jobs_table_shape() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "schema-probe").await;

        let columns = table_columns(&pool, JOBS_TABLE).await;
        for expected in [
            "id",
            "job_type",
            "status",
            "attempts",
            "max_attempts",
            "run_at",
            "last_result",
            "lock_at",
            "lock_by",
            "done_at",
            "priority",
            "job",
            "metadata",
            "idempotency_key",
        ] {
            assert!(
                columns.iter().any(|column| column == expected),
                "missing Apalis Jobs column {expected}; actual columns: {columns:?}"
            );
        }

        let row = sqlx::query("SELECT job_type, status, idempotency_key FROM Jobs LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("read seeded job");
        assert_eq!(row.get::<String, _>("job_type"), "gemini-browser");
        assert_eq!(row.get::<String, _>("status"), "Pending");
        assert_eq!(row.get::<String, _>("idempotency_key"), "schema-probe");
    }

    #[tokio::test]
    async fn apalis_jobs_list_returns_empty_when_jobs_table_missing() {
        let pool = memory_pool().await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("missing Jobs table is not fatal");

        assert!(response.jobs.is_empty());
        assert_eq!(response.total_matching, 0);
        assert!(response.status_counts.is_empty());
        assert!(response.job_type_counts.is_empty());
        assert_eq!(response.limit, DEFAULT_LIMIT);
        assert!(response.refreshed_at.ends_with('Z'));
    }
}
