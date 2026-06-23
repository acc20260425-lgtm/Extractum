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
    pool: &SqlitePool,
    request: ApalisJobsListRequest,
) -> crate::error::AppResult<ApalisJobsListResponse> {
    let filters = filters_from_request(request);
    let Some(schema) = jobs_table_schema(pool).await? else {
        return Ok(empty_response(filters.limit));
    };

    let total_matching = count_jobs(pool, &schema, &filters, true, true).await?;
    let status_counts = status_counts(pool, &schema, &filters).await?;
    let job_type_counts = job_type_counts(pool, &schema, &filters).await?;
    let summaries = fetch_job_summaries(pool, &schema, &filters).await?;
    let payloads_by_id = fetch_payloads_for_ids(
        pool,
        &schema,
        summaries.iter().map(|row| row.id.as_str()).collect(),
    )
    .await?;
    let jobs = summaries
        .into_iter()
        .map(|row| {
            let payloads = payloads_by_id.get(&row.id).cloned().unwrap_or_default();
            row.into_dto(payloads)
        })
        .collect::<Vec<_>>();

    Ok(ApalisJobsListResponse {
        jobs,
        total_matching,
        status_counts,
        job_type_counts,
        refreshed_at: crate::time::now_rfc3339_utc(),
        limit: filters.limit,
    })
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

#[derive(Clone, Debug)]
struct ApalisJobsFilters {
    limit: u32,
    status: Option<String>,
    job_type: Option<String>,
    search: Option<String>,
}

#[derive(Clone, Debug)]
struct JobsTableSchema {
    columns: std::collections::HashSet<String>,
}

impl JobsTableSchema {
    fn has(&self, column: &str) -> bool {
        self.columns.contains(column)
    }
}

fn normalize_filter(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn filters_from_request(request: ApalisJobsListRequest) -> ApalisJobsFilters {
    ApalisJobsFilters {
        limit: normalized_limit(request.limit),
        status: normalize_filter(request.status),
        job_type: normalize_filter(request.job_type),
        search: normalize_filter(request.search),
    }
}

async fn jobs_table_schema(pool: &SqlitePool) -> crate::error::AppResult<Option<JobsTableSchema>> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ? LIMIT 1",
    )
    .bind(JOBS_TABLE)
    .fetch_optional(pool)
    .await
    .map_err(crate::error::AppError::database)?;

    if exists.is_none() {
        return Ok(None);
    }

    let rows = sqlx::query("PRAGMA table_info('Jobs')")
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    let columns = rows
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<std::collections::HashSet<_>>();

    Ok(Some(JobsTableSchema { columns }))
}

#[derive(Clone, Debug)]
struct InternalJobSummary {
    id: String,
    job_type: String,
    status: String,
    attempts: u32,
    max_attempts: Option<u32>,
    run_at: Option<String>,
    lock_at: Option<String>,
    lock_by: Option<String>,
    done_at: Option<String>,
    last_activity_at: Option<String>,
    priority: Option<u32>,
    idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct PayloadColumns {
    job_raw: Option<Vec<u8>>,
    last_result_raw: Option<Vec<u8>>,
    metadata_raw: Option<Vec<u8>>,
}

impl InternalJobSummary {
    fn into_dto(self, _payloads: PayloadColumns) -> ApalisJobRow {
        ApalisJobRow {
            id: self.id,
            job_type: self.job_type,
            status: self.status,
            attempts: self.attempts,
            max_attempts: self.max_attempts,
            run_at: self.run_at,
            lock_at: self.lock_at,
            lock_by: self.lock_by,
            done_at: self.done_at,
            last_activity_at: self.last_activity_at,
            priority: self.priority,
            idempotency_key: self.idempotency_key,
            job_preview: None,
            job_truncated: false,
            job_json: None,
            last_result: None,
            last_result_truncated: false,
            metadata: None,
            metadata_truncated: false,
        }
    }
}

async fn fetch_job_summaries(
    pool: &SqlitePool,
    schema: &JobsTableSchema,
    filters: &ApalisJobsFilters,
) -> crate::error::AppResult<Vec<InternalJobSummary>> {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT ");
    builder
        .push(text_expr(schema, "id", "''"))
        .push(" AS id, ")
        .push(text_expr(schema, "job_type", "''"))
        .push(" AS job_type, ")
        .push(text_expr(schema, "status", "'unknown'"))
        .push(" AS status, ")
        .push(int_expr(schema, "attempts", "0"))
        .push(" AS attempts, ")
        .push(nullable_int_expr(schema, "max_attempts"))
        .push(" AS max_attempts, ")
        .push(nullable_text_expr(schema, "run_at"))
        .push(" AS run_at, ")
        .push(nullable_text_expr(schema, "lock_at"))
        .push(" AS lock_at, ")
        .push(nullable_text_expr(schema, "lock_by"))
        .push(" AS lock_by, ")
        .push(nullable_text_expr(schema, "done_at"))
        .push(" AS done_at, ")
        .push(nullable_int_expr(schema, "priority"))
        .push(" AS priority, ")
        .push(nullable_text_expr(schema, "idempotency_key"))
        .push(" AS idempotency_key, ")
        .push(last_activity_sort_expr(schema))
        .push(" AS last_activity_sort FROM Jobs");
    push_where(&mut builder, schema, filters, true, true);
    builder
        .push(" ORDER BY last_activity_sort DESC, id DESC LIMIT ")
        .push_bind(filters.limit as i64);

    let rows = builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(rows.into_iter().map(internal_summary_from_sql).collect())
}

async fn fetch_payloads_for_ids(
    pool: &SqlitePool,
    schema: &JobsTableSchema,
    ids: Vec<&str>,
) -> crate::error::AppResult<std::collections::HashMap<String, PayloadColumns>> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let mut builder = QueryBuilder::<Sqlite>::new("SELECT ");
    builder
        .push(text_expr(schema, "id", "''"))
        .push(" AS id, ")
        .push(blob_or_text_expr(schema, "job"))
        .push(" AS job, ")
        .push(blob_or_text_expr(schema, "last_result"))
        .push(" AS last_result, ")
        .push(blob_or_text_expr(schema, "metadata"))
        .push(" AS metadata FROM Jobs WHERE id IN (");
    let mut separated = builder.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let rows = builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id = row.get::<String, _>("id");
            let payloads = PayloadColumns {
                job_raw: optional_blob_or_text_alias(&row, "job"),
                last_result_raw: optional_blob_or_text_alias(&row, "last_result"),
                metadata_raw: optional_blob_or_text_alias(&row, "metadata"),
            };
            (id, payloads)
        })
        .collect())
}

fn internal_summary_from_sql(row: sqlx::sqlite::SqliteRow) -> InternalJobSummary {
    let run_at = normalize_timestamp(row.try_get::<Option<String>, _>("run_at").ok().flatten());
    let lock_at = normalize_timestamp(row.try_get::<Option<String>, _>("lock_at").ok().flatten());
    let done_at = normalize_timestamp(row.try_get::<Option<String>, _>("done_at").ok().flatten());
    let last_activity_at = latest_timestamp([
        done_at.as_deref(),
        lock_at.as_deref(),
        run_at.as_deref(),
    ]);

    InternalJobSummary {
        id: row.get::<String, _>("id"),
        job_type: row.get::<String, _>("job_type"),
        status: row.get::<String, _>("status"),
        attempts: row.get::<i64, _>("attempts").max(0) as u32,
        max_attempts: row
            .try_get::<Option<i64>, _>("max_attempts")
            .ok()
            .flatten()
            .map(|value| value.max(0) as u32),
        run_at,
        lock_at,
        lock_by: row.try_get::<Option<String>, _>("lock_by").ok().flatten(),
        done_at,
        last_activity_at,
        priority: row
            .try_get::<Option<i64>, _>("priority")
            .ok()
            .flatten()
            .map(|value| value.max(0) as u32),
        idempotency_key: row
            .try_get::<Option<String>, _>("idempotency_key")
            .ok()
            .flatten(),
    }
}

async fn count_jobs(
    pool: &SqlitePool,
    schema: &JobsTableSchema,
    filters: &ApalisJobsFilters,
    include_status: bool,
    include_job_type: bool,
) -> crate::error::AppResult<u32> {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM Jobs");
    push_where(
        &mut builder,
        schema,
        filters,
        include_status,
        include_job_type,
    );
    let count = builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    Ok(count.max(0) as u32)
}

async fn status_counts(
    pool: &SqlitePool,
    schema: &JobsTableSchema,
    filters: &ApalisJobsFilters,
) -> crate::error::AppResult<Vec<ApalisJobStatusCount>> {
    if !schema.has("status") {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT COALESCE(CAST(status AS TEXT), 'unknown') AS status, COUNT(*) AS count FROM Jobs",
    );
    push_where(&mut builder, schema, filters, false, true);
    builder.push(" GROUP BY status ORDER BY status");
    let rows = builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(rows
        .into_iter()
        .map(|row| ApalisJobStatusCount {
            status: row.get::<String, _>("status"),
            count: row.get::<i64, _>("count").max(0) as u32,
        })
        .collect())
}

async fn job_type_counts(
    pool: &SqlitePool,
    schema: &JobsTableSchema,
    filters: &ApalisJobsFilters,
) -> crate::error::AppResult<Vec<ApalisJobTypeCount>> {
    if !schema.has("job_type") {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT COALESCE(CAST(job_type AS TEXT), '') AS job_type, COUNT(*) AS count FROM Jobs",
    );
    push_where(&mut builder, schema, filters, true, false);
    builder.push(" GROUP BY job_type ORDER BY job_type");
    let rows = builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(rows
        .into_iter()
        .map(|row| ApalisJobTypeCount {
            job_type: row.get::<String, _>("job_type"),
            count: row.get::<i64, _>("count").max(0) as u32,
        })
        .collect())
}

fn push_where<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    schema: &JobsTableSchema,
    filters: &'a ApalisJobsFilters,
    include_status: bool,
    include_job_type: bool,
) {
    if !(include_status && filters.status.is_some())
        && !(include_job_type && filters.job_type.is_some())
        && filters.search.is_none()
    {
        return;
    }

    builder.push(" WHERE ");
    let mut first = true;
    if include_status {
        if let Some(status) = filters.status.as_deref() {
            if !first {
                builder.push(" AND ");
            }
            first = false;
            if schema.has("status") {
                builder.push("status = ").push_bind(status);
            } else {
                builder.push("0 = ").push_bind(1_i64);
            }
        }
    }
    if include_job_type {
        if let Some(job_type) = filters.job_type.as_deref() {
            if !first {
                builder.push(" AND ");
            }
            first = false;
            if schema.has("job_type") {
                builder.push("job_type = ").push_bind(job_type);
            } else {
                builder.push("0 = ").push_bind(1_i64);
            }
        }
    }
    if let Some(search) = filters.search.as_deref() {
        if !first {
            builder.push(" AND ");
        }
        let pattern = format!("%{}%", search.to_ascii_lowercase());
        let id_expr = if schema.has("id") {
            "LOWER(CAST(id AS TEXT)) LIKE "
        } else {
            "0 = "
        };
        let key_expr = if schema.has("idempotency_key") {
            "LOWER(CAST(idempotency_key AS TEXT)) LIKE "
        } else {
            "0 = "
        };
        builder
            .push("(")
            .push(id_expr)
            .push_bind(pattern.clone())
            .push(" OR ")
            .push(key_expr)
            .push_bind(pattern)
            .push(")");
    }
}

fn optional_blob_or_text_alias(row: &sqlx::sqlite::SqliteRow, alias: &str) -> Option<Vec<u8>> {
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(alias) {
        return value;
    }
    row.try_get::<Option<String>, _>(alias)
        .ok()
        .flatten()
        .map(String::into_bytes)
}

fn nullable_text_expr(schema: &JobsTableSchema, column: &str) -> String {
    if schema.has(column) {
        format!("CAST({column} AS TEXT)")
    } else {
        "NULL".to_string()
    }
}

fn text_expr(schema: &JobsTableSchema, column: &str, fallback: &str) -> String {
    if schema.has(column) {
        format!("COALESCE(CAST({column} AS TEXT), {fallback})")
    } else {
        fallback.to_string()
    }
}

fn nullable_int_expr(schema: &JobsTableSchema, column: &str) -> String {
    if schema.has(column) {
        format!("CAST({column} AS INTEGER)")
    } else {
        "NULL".to_string()
    }
}

fn int_expr(schema: &JobsTableSchema, column: &str, fallback: &str) -> String {
    if schema.has(column) {
        format!("COALESCE(CAST({column} AS INTEGER), {fallback})")
    } else {
        fallback.to_string()
    }
}

fn blob_or_text_expr(schema: &JobsTableSchema, column: &str) -> String {
    if schema.has(column) {
        column.to_string()
    } else {
        "NULL".to_string()
    }
}

const UNIX_MILLISECONDS_THRESHOLD: i64 = 100_000_000_000;

fn timestamp_epoch_expr(schema: &JobsTableSchema, column: &str) -> String {
    if !schema.has(column) {
        return "NULL".to_string();
    }
    format!(
        "COALESCE(unixepoch({column}), CASE WHEN ABS(CAST({column} AS INTEGER)) >= {UNIX_MILLISECONDS_THRESHOLD} THEN CAST({column} AS INTEGER) / 1000 WHEN CAST({column} AS INTEGER) > 1000000000 THEN CAST({column} AS INTEGER) ELSE NULL END)"
    )
}

fn last_activity_sort_expr(schema: &JobsTableSchema) -> String {
    format!(
        "max(COALESCE({}, -9223372036854775808), COALESCE({}, -9223372036854775808), COALESCE({}, -9223372036854775808))",
        timestamp_epoch_expr(schema, "done_at"),
        timestamp_epoch_expr(schema, "lock_at"),
        timestamp_epoch_expr(schema, "run_at"),
    )
}

fn normalize_timestamp(value: Option<String>) -> Option<String> {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime, PrimitiveDateTime};

    let value = value?.trim().to_string();
    if value.is_empty() {
        return None;
    }

    if let Ok(epoch) = value.parse::<i64>() {
        return OffsetDateTime::from_unix_timestamp(unix_timestamp_seconds(epoch))
            .ok()
            .and_then(|timestamp| timestamp.format(&Rfc3339).ok());
    }

    if let Ok(timestamp) = OffsetDateTime::parse(&value, &Rfc3339) {
        return timestamp
            .to_offset(time::UtcOffset::UTC)
            .format(&Rfc3339)
            .ok();
    }

    if let Ok(timestamp) = PrimitiveDateTime::parse(
        &value,
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    ) {
        return timestamp.assume_utc().format(&Rfc3339).ok();
    }

    None
}

fn unix_timestamp_seconds(value: i64) -> i64 {
    if value.abs() >= UNIX_MILLISECONDS_THRESHOLD {
        value / 1000
    } else {
        value
    }
}

fn latest_timestamp(values: [Option<&str>; 3]) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .max_by_key(|value| *value)
        .map(ToOwned::to_owned)
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

    async fn update_job_row(
        pool: &SqlitePool,
        idempotency_key: &str,
        status: &str,
        run_at: Option<&str>,
        lock_at: Option<&str>,
        done_at: Option<&str>,
    ) {
        sqlx::query(
            "UPDATE Jobs
             SET status = ?, run_at = COALESCE(?, run_at), lock_at = ?, done_at = ?
             WHERE idempotency_key = ?",
        )
        .bind(status)
        .bind(run_at)
        .bind(lock_at)
        .bind(done_at)
        .bind(idempotency_key)
        .execute(pool)
        .await
        .expect("update Jobs row");
    }

    #[tokio::test]
    async fn apalis_jobs_list_returns_rows_from_jobs_table() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "row-1").await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list apalis jobs");

        assert_eq!(response.jobs.len(), 1);
        let job = &response.jobs[0];
        assert!(!job.id.is_empty());
        assert_eq!(job.job_type, "gemini-browser");
        assert_eq!(job.status, "Pending");
        assert_eq!(job.attempts, 0);
        assert_eq!(job.idempotency_key.as_deref(), Some("row-1"));
        assert!(job.run_at.is_some());
        assert_eq!(response.total_matching, 1);
    }

    #[tokio::test]
    async fn apalis_jobs_list_filters_by_status_job_type_and_search() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "search-one").await;
        seed_apalis_job(&pool, "search-two").await;
        update_job_row(
            &pool,
            "search-two",
            "Failed",
            None,
            None,
            Some("2026-06-23T10:10:00Z"),
        )
        .await;

        let response = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: None,
                status: Some("Failed".to_string()),
                job_type: Some("gemini-browser".to_string()),
                search: Some("two".to_string()),
            },
        )
        .await
        .expect("list filtered apalis jobs");

        assert_eq!(response.jobs.len(), 1);
        assert_eq!(response.jobs[0].idempotency_key.as_deref(), Some("search-two"));
        assert_eq!(response.jobs[0].status, "Failed");
        assert_eq!(response.total_matching, 1);
    }

    #[tokio::test]
    async fn apalis_jobs_list_clamps_limit() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "limit-1").await;

        let low = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: Some(0),
                ..Default::default()
            },
        )
        .await
        .expect("list with low limit");
        let high = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: Some(999),
                ..Default::default()
            },
        )
        .await
        .expect("list with high limit");

        assert_eq!(low.limit, 1);
        assert_eq!(high.limit, MAX_LIMIT);
    }

    #[tokio::test]
    async fn apalis_jobs_list_does_not_mutate_jobs() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "no-mutate").await;
        let before: Vec<(String, String, i64)> =
            sqlx::query_as("SELECT id, status, attempts FROM Jobs ORDER BY id")
                .fetch_all(&pool)
                .await
                .expect("read before");

        let _response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list jobs");

        let after: Vec<(String, String, i64)> =
            sqlx::query_as("SELECT id, status, attempts FROM Jobs ORDER BY id")
                .fetch_all(&pool)
                .await
                .expect("read after");
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn apalis_jobs_list_sorts_by_latest_activity_timestamp() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "old-done").await;
        seed_apalis_job(&pool, "new-lock").await;
        seed_apalis_job(&pool, "new-done").await;
        update_job_row(
            &pool,
            "old-done",
            "Done",
            Some("2026-06-23T08:00:00Z"),
            None,
            Some("2026-06-23T09:00:00Z"),
        )
        .await;
        update_job_row(
            &pool,
            "new-lock",
            "Running",
            Some("2026-06-23T07:00:00Z"),
            Some("2026-06-23T11:00:00Z"),
            None,
        )
        .await;
        update_job_row(
            &pool,
            "new-done",
            "Done",
            Some("2026-06-23T06:00:00Z"),
            Some("2026-06-23T08:30:00Z"),
            Some("1782216000000"),
        )
        .await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list sorted jobs");

        let keys = response
            .jobs
            .iter()
            .map(|job| job.idempotency_key.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![Some("new-done"), Some("new-lock"), Some("old-done")]
        );
        assert_eq!(
            response.jobs[0].last_activity_at.as_deref(),
            Some("2026-06-23T12:00:00Z")
        );
    }

    #[tokio::test]
    async fn apalis_jobs_list_returns_rfc3339_utc_timestamps() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "timestamps").await;
        update_job_row(
            &pool,
            "timestamps",
            "Done",
            Some("2026-06-23 08:00:00"),
            Some("1719129600000"),
            Some("2026-06-23T12:00:00+03:00"),
        )
        .await;

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list timestamp jobs");
        let job = &response.jobs[0];

        assert_eq!(job.run_at.as_deref(), Some("2026-06-23T08:00:00Z"));
        assert_eq!(job.lock_at.as_deref(), Some("2024-06-23T08:00:00Z"));
        assert_eq!(job.done_at.as_deref(), Some("2026-06-23T09:00:00Z"));
    }

    #[tokio::test]
    async fn apalis_jobs_counts_ignore_their_own_active_filter() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "count-pending").await;
        seed_apalis_job(&pool, "count-failed").await;
        update_job_row(
            &pool,
            "count-failed",
            "Failed",
            None,
            None,
            Some("2026-06-23T10:00:00Z"),
        )
        .await;

        let response = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: None,
                status: Some("Failed".to_string()),
                job_type: Some("gemini-browser".to_string()),
                search: Some("count".to_string()),
            },
        )
        .await
        .expect("list count jobs");

        assert_eq!(response.total_matching, 1);
        assert_eq!(
            response.status_counts,
            vec![
                ApalisJobStatusCount {
                    status: "Failed".to_string(),
                    count: 1,
                },
                ApalisJobStatusCount {
                    status: "Pending".to_string(),
                    count: 1,
                },
            ]
        );
        assert_eq!(
            response.job_type_counts,
            vec![ApalisJobTypeCount {
                job_type: "gemini-browser".to_string(),
                count: 1,
            }]
        );
    }
}
