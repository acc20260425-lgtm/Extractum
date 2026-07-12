use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use tauri::AppHandle;

const DEFAULT_LIMIT: u32 = 100;
const MAX_LIMIT: u32 = 500;
const TERMINAL_PRUNE_OLDER_THAN_HOURS: u32 = 24;
const JOBS_TABLE: &str = "Jobs";
const MAX_PAYLOAD_JSON_BYTES: usize = 64 * 1024;
const TRUNCATED_PREVIEW_CHARS: usize = 2000;
const DECODE_FAILURE_PREVIEW_CHARS: usize = 500;
const REDACTED: &str = "[redacted]";
const SENSITIVE_KEY_FRAGMENTS: &[&str] = &[
    "apikey",
    "authorization",
    "bearer",
    "cookie",
    "credentials",
    "password",
    "secret",
    "session",
    "token",
    "apihash",
    "refreshtoken",
];

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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobsPruneTerminalRequest {
    pub older_than_hours: Option<u32>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApalisJobsPruneTerminalResponse {
    pub deleted_count: u64,
    pub cutoff_at: String,
    pub older_than_hours: u32,
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

#[tauri::command]
pub(crate) async fn apalis_jobs_prune_terminal(
    handle: AppHandle,
    request: Option<ApalisJobsPruneTerminalRequest>,
) -> crate::error::AppResult<ApalisJobsPruneTerminalResponse> {
    let pool = crate::db::get_pool(&handle).await?;
    let older_than_hours = normalized_prune_hours(request.unwrap_or_default().older_than_hours);
    apalis_jobs_prune_terminal_from_pool_with_hours(
        &pool,
        crate::time::now_secs(),
        older_than_hours,
    )
    .await
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

#[cfg(test)]
async fn apalis_jobs_prune_terminal_from_pool(
    pool: &SqlitePool,
    now_secs: i64,
) -> crate::error::AppResult<ApalisJobsPruneTerminalResponse> {
    apalis_jobs_prune_terminal_from_pool_with_hours(pool, now_secs, TERMINAL_PRUNE_OLDER_THAN_HOURS)
        .await
}

async fn apalis_jobs_prune_terminal_from_pool_with_hours(
    pool: &SqlitePool,
    now_secs: i64,
    older_than_hours: u32,
) -> crate::error::AppResult<ApalisJobsPruneTerminalResponse> {
    let cutoff_secs = prune_cutoff_secs(now_secs, older_than_hours);
    let cutoff_at = normalize_timestamp(Some(cutoff_secs.to_string()))
        .unwrap_or_else(crate::time::now_rfc3339_utc);
    let Some(schema) = jobs_table_schema(pool).await? else {
        return Ok(ApalisJobsPruneTerminalResponse {
            deleted_count: 0,
            cutoff_at,
            older_than_hours,
        });
    };

    if !schema.has("status") || !schema.has("done_at") {
        return Ok(ApalisJobsPruneTerminalResponse {
            deleted_count: 0,
            cutoff_at,
            older_than_hours,
        });
    }

    let done_at_epoch = timestamp_epoch_expr(&schema, "done_at");
    let mut builder = QueryBuilder::<Sqlite>::new("DELETE FROM Jobs WHERE ");
    builder
        .push(&done_at_epoch)
        .push(" IS NOT NULL AND ")
        .push(&done_at_epoch)
        .push(" < ")
        .push_bind(cutoff_secs)
        .push(" AND (status = 'Done' OR status = 'Killed'");
    if schema.has("attempts") && schema.has("max_attempts") {
        builder.push(
            " OR (status = 'Failed' AND CAST(max_attempts AS INTEGER) <= CAST(attempts AS INTEGER))",
        );
    }
    builder.push(")");

    let result = builder
        .build()
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;

    Ok(ApalisJobsPruneTerminalResponse {
        deleted_count: result.rows_affected(),
        cutoff_at,
        older_than_hours,
    })
}

fn normalized_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn normalized_prune_hours(hours: Option<u32>) -> u32 {
    hours
        .unwrap_or(TERMINAL_PRUNE_OLDER_THAN_HOURS)
        .max(TERMINAL_PRUNE_OLDER_THAN_HOURS)
}

fn prune_cutoff_secs(now_secs: i64, older_than_hours: u32) -> i64 {
    now_secs.saturating_sub(i64::from(older_than_hours).saturating_mul(60 * 60))
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
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ? LIMIT 1")
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

#[derive(Clone, Debug, PartialEq)]
struct PayloadView {
    json: Option<Value>,
    preview: Option<String>,
    truncated: bool,
}

fn payload_view(raw: Option<Vec<u8>>, decode_failure_preview: bool) -> PayloadView {
    let Some(raw) = raw else {
        return PayloadView {
            json: None,
            preview: None,
            truncated: false,
        };
    };

    match serde_json::from_slice::<Value>(&raw) {
        Ok(value) => bounded_json_payload(redact_json(value)),
        Err(_) => {
            let text = String::from_utf8_lossy(&raw);
            let text_was_truncated =
                decode_failure_preview && text.chars().count() > DECODE_FAILURE_PREVIEW_CHARS;
            PayloadView {
                json: None,
                preview: if decode_failure_preview {
                    Some(redacted_text_preview(&text, DECODE_FAILURE_PREVIEW_CHARS))
                } else {
                    None
                },
                truncated: text_was_truncated,
            }
        }
    }
}

fn bounded_json_payload(value: Value) -> PayloadView {
    let serialized = serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string());
    if serialized.len() <= MAX_PAYLOAD_JSON_BYTES {
        return PayloadView {
            json: Some(value),
            preview: None,
            truncated: false,
        };
    }

    PayloadView {
        json: Some(serde_json::json!({
            "truncated": true,
            "preview": redacted_text_preview(&serialized, TRUNCATED_PREVIEW_CHARS),
        })),
        preview: None,
        truncated: true,
    }
}

fn redact_json(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    if is_sensitive_key(&key) {
                        (key, Value::String(REDACTED.to_string()))
                    } else {
                        (key, redact_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_json).collect()),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    SENSITIVE_KEY_FRAGMENTS
        .iter()
        .any(|fragment| normalized.contains(fragment))
}

fn redacted_text_preview(text: &str, max_chars: usize) -> String {
    let redacted = redact_text_fragments(text);
    redacted.chars().take(max_chars).collect()
}

fn redact_text_fragments(text: &str) -> String {
    text.lines()
        .map(redact_text_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn redact_text_line(line: &str) -> String {
    line.split(';')
        .map(redact_text_segment)
        .collect::<Vec<_>>()
        .join("; ")
}

fn redact_text_segment(segment: &str) -> String {
    if let Some((key, separator)) = sensitive_assignment(segment) {
        let spacer = if separator == ':' { " " } else { "" };
        return format!("{}{}{}{}", key.trim_end(), separator, spacer, REDACTED);
    }
    redact_inline_secret_tokens(segment)
}

fn sensitive_assignment(segment: &str) -> Option<(&str, char)> {
    for (index, character) in segment.char_indices() {
        if index > 120 {
            return None;
        }
        if character == ':' || character == '=' {
            let key = segment[..index].trim();
            return is_sensitive_key(key).then_some((&segment[..index], character));
        }
    }
    None
}

fn redact_inline_secret_tokens(line: &str) -> String {
    let mut redact_next_count = 0usize;
    let mut output = Vec::new();

    for word in line.split_whitespace() {
        if redact_next_count > 0 {
            output.push(REDACTED.to_string());
            redact_next_count -= 1;
            continue;
        }

        let normalized = word
            .trim_matches(|character: char| !character.is_ascii_alphanumeric())
            .to_ascii_lowercase();
        if normalized == "authorization" {
            output.push(word.to_string());
            redact_next_count = 2;
        } else if normalized == "bearer" || normalized == "cookie" || normalized == "setcookie" {
            output.push(word.to_string());
            redact_next_count = 1;
        } else {
            output.push(word.to_string());
        }
    }

    output.join(" ")
}

impl InternalJobSummary {
    fn into_dto(self, payloads: PayloadColumns) -> ApalisJobRow {
        let job = payload_view(payloads.job_raw, true);
        let last_result = payload_view(payloads.last_result_raw, false);
        let metadata = payload_view(payloads.metadata_raw, false);

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
            job_preview: job.preview,
            job_truncated: job.truncated,
            job_json: job.json,
            last_result: last_result.json,
            last_result_truncated: last_result.truncated,
            metadata: metadata.json,
            metadata_truncated: metadata.truncated,
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
    let last_activity_at =
        latest_timestamp([done_at.as_deref(), lock_at.as_deref(), run_at.as_deref()]);

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
        crate::gemini_browser::enqueue_gemini_browser_job_to_storage(
            &mut storage,
            test_job(run_id),
        )
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

    #[tokio::test]
    async fn apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs() {
        let pool = memory_pool().await;
        let now = 1_800_000_000;
        let cutoff = now - 24 * 60 * 60;
        let old = (cutoff - 10).to_string();
        let old_millis = ((cutoff - 20) * 1000).to_string();
        let recent = (cutoff + 10).to_string();

        for run_id in [
            "prune-old-done",
            "prune-old-killed",
            "prune-terminal-failed",
            "keep-retriable-failed",
            "keep-recent-done",
            "keep-old-running",
            "keep-missing-done-at",
            "keep-old-run-recent-done",
        ] {
            seed_apalis_job(&pool, run_id).await;
        }

        update_job_terminal_row(&pool, "prune-old-done", "Done", 1, 25, None, Some(&old)).await;
        update_job_terminal_row(
            &pool,
            "prune-old-killed",
            "Killed",
            1,
            25,
            None,
            Some(&old_millis),
        )
        .await;
        update_job_terminal_row(
            &pool,
            "prune-terminal-failed",
            "Failed",
            5,
            5,
            None,
            Some(&old),
        )
        .await;
        update_job_terminal_row(
            &pool,
            "keep-retriable-failed",
            "Failed",
            4,
            5,
            None,
            Some(&old),
        )
        .await;
        update_job_terminal_row(
            &pool,
            "keep-recent-done",
            "Done",
            1,
            25,
            None,
            Some(&recent),
        )
        .await;
        update_job_terminal_row(
            &pool,
            "keep-old-running",
            "Running",
            1,
            25,
            None,
            Some(&old),
        )
        .await;
        update_job_terminal_row(&pool, "keep-missing-done-at", "Done", 1, 25, None, None).await;
        update_job_terminal_row(
            &pool,
            "keep-old-run-recent-done",
            "Done",
            1,
            25,
            Some(&old),
            Some(&recent),
        )
        .await;

        let response = apalis_jobs_prune_terminal_from_pool(&pool, now)
            .await
            .expect("prune old terminal jobs");

        assert_eq!(response.deleted_count, 3);
        assert_eq!(response.older_than_hours, 24);
        assert_eq!(
            response.cutoff_at,
            normalize_timestamp(Some(cutoff.to_string())).expect("normalized cutoff")
        );

        let remaining: Vec<String> =
            sqlx::query_scalar("SELECT idempotency_key FROM Jobs ORDER BY idempotency_key")
                .fetch_all(&pool)
                .await
                .expect("read remaining jobs");
        assert_eq!(
            remaining,
            vec![
                "keep-missing-done-at",
                "keep-old-run-recent-done",
                "keep-old-running",
                "keep-recent-done",
                "keep-retriable-failed",
            ]
        );
    }

    #[tokio::test]
    async fn apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing() {
        let pool = memory_pool().await;

        let response = apalis_jobs_prune_terminal_from_pool(&pool, 1_800_000_000)
            .await
            .expect("missing Jobs table is not fatal");

        assert_eq!(response.deleted_count, 0);
        assert_eq!(response.older_than_hours, 24);
        assert!(response.cutoff_at.ends_with('Z'));
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

    async fn update_job_terminal_row(
        pool: &SqlitePool,
        idempotency_key: &str,
        status: &str,
        attempts: i64,
        max_attempts: i64,
        run_at: Option<&str>,
        done_at: Option<&str>,
    ) {
        sqlx::query(
            "UPDATE Jobs
             SET status = ?, attempts = ?, max_attempts = ?, run_at = COALESCE(?, run_at), done_at = ?
             WHERE idempotency_key = ?",
        )
        .bind(status)
        .bind(attempts)
        .bind(max_attempts)
        .bind(run_at)
        .bind(done_at)
        .bind(idempotency_key)
        .execute(pool)
        .await
        .expect("update terminal Jobs row");
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
        assert_eq!(
            response.jobs[0].idempotency_key.as_deref(),
            Some("search-two")
        );
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

    #[tokio::test]
    async fn apalis_jobs_payloads_are_redacted_and_truncated() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "payload-secret").await;
        let large_text = "x".repeat(70 * 1024);
        sqlx::query(
            "UPDATE Jobs
             SET job = ?, last_result = ?, metadata = ?
             WHERE idempotency_key = ?",
        )
        .bind(r#"{"apiKey":"sk-secret","nested":{"refresh-token":"rt-secret"},"prompt":"safe prompt"}"#)
        .bind(format!(
            r#"{{"message":"{large_text}","authorization":"Bearer secret"}}"#
        ))
        .bind(r#"{"API Key":"another-secret","normal":"visible"}"#)
        .bind("payload-secret")
        .execute(&pool)
        .await
        .expect("update payload columns");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list payload jobs");
        let job = &response.jobs[0];

        assert_eq!(job.job_json.as_ref().unwrap()["apiKey"], "[redacted]");
        assert_eq!(
            job.job_json.as_ref().unwrap()["nested"]["refresh-token"],
            "[redacted]"
        );
        assert_eq!(job.metadata.as_ref().unwrap()["API Key"], "[redacted]");
        assert_eq!(job.metadata.as_ref().unwrap()["normal"], "visible");
        assert!(job.last_result_truncated);
        assert_eq!(job.last_result.as_ref().unwrap()["truncated"], true);
        assert!(
            job.last_result.as_ref().unwrap()["preview"]
                .as_str()
                .unwrap()
                .chars()
                .count()
                <= 2000
        );
        let serialized = serde_json::to_string(job).expect("serialize response row");
        assert!(!serialized.contains("sk-secret"));
        assert!(!serialized.contains("rt-secret"));
        assert!(!serialized.contains("another-secret"));
        assert!(!serialized.contains("Bearer secret"));
    }

    #[tokio::test]
    async fn apalis_jobs_limit_excludes_large_payloads_outside_limited_rows() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "payload-included").await;
        seed_apalis_job(&pool, "payload-outside-limit").await;
        update_job_row(
            &pool,
            "payload-included",
            "Done",
            Some("2026-06-23T10:00:00Z"),
            None,
            Some("2026-06-23T12:00:00Z"),
        )
        .await;
        update_job_row(
            &pool,
            "payload-outside-limit",
            "Done",
            Some("2026-06-23T09:00:00Z"),
            None,
            Some("2026-06-23T11:00:00Z"),
        )
        .await;
        let outside_large_payload = format!(
            r#"{{"apiKey":"outside-secret","body":"{}"}}"#,
            "x".repeat(256 * 1024)
        );
        sqlx::query("UPDATE Jobs SET job = ? WHERE idempotency_key = ?")
            .bind(r#"{"safe":"included"}"#)
            .bind("payload-included")
            .execute(&pool)
            .await
            .expect("update included payload");
        sqlx::query("UPDATE Jobs SET job = ? WHERE idempotency_key = ?")
            .bind(outside_large_payload)
            .bind("payload-outside-limit")
            .execute(&pool)
            .await
            .expect("update outside payload");

        let response = apalis_jobs_list_from_pool(
            &pool,
            ApalisJobsListRequest {
                limit: Some(1),
                ..Default::default()
            },
        )
        .await
        .expect("list limited payload jobs");

        assert_eq!(response.total_matching, 2);
        assert_eq!(response.jobs.len(), 1);
        assert_eq!(
            response.jobs[0].idempotency_key.as_deref(),
            Some("payload-included")
        );
        assert_eq!(
            response.jobs[0].job_json.as_ref().unwrap()["safe"],
            "included"
        );
        let serialized = serde_json::to_string(&response).expect("serialize response");
        assert!(!serialized.contains("outside-secret"));
        assert!(!serialized.contains("payload-outside-limit"));
    }

    #[tokio::test]
    async fn apalis_jobs_decode_failure_returns_redacted_preview_without_json() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "payload-invalid").await;
        let long_tail = "z".repeat(DECODE_FAILURE_PREVIEW_CHARS + 100);
        sqlx::query("UPDATE Jobs SET job = ? WHERE idempotency_key = ?")
            .bind(format!("Authorization: Bearer raw-secret\nCookie: session=raw-cookie\nAuthorization: Bearer semicolon-secret; harmless context\nAuthorization Bearer raw-inline\napiKey=sk-secret; plain text payload; {long_tail}"))
            .bind("payload-invalid")
            .execute(&pool)
            .await
            .expect("update invalid payload");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list payload jobs");
        let job = &response.jobs[0];

        assert_eq!(job.job_json, None);
        assert!(job.job_truncated);
        let preview = job.job_preview.as_deref().expect("redacted preview");
        assert!(preview.chars().count() <= DECODE_FAILURE_PREVIEW_CHARS);
        assert!(preview.contains("Authorization: [redacted]"));
        assert!(preview.contains("Cookie: [redacted]"));
        assert!(preview.contains("apiKey=[redacted]"));
        assert!(preview.contains("harmless context"));
        assert!(preview.contains("plain text payload"));
        assert!(!preview.contains("raw-secret"));
        assert!(!preview.contains("raw-cookie"));
        assert!(!preview.contains("semicolon-secret"));
        assert!(!preview.contains("raw-inline"));
        assert!(!preview.contains("sk-secret"));
    }

    #[tokio::test]
    async fn apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1() {
        let pool = memory_pool().await;
        seed_apalis_job(&pool, "payload-non-json-secondary").await;
        sqlx::query("UPDATE Jobs SET last_result = ?, metadata = ? WHERE idempotency_key = ?")
            .bind("Authorization: Bearer result-secret")
            .bind("apiKey=metadata-secret")
            .bind("payload-non-json-secondary")
            .execute(&pool)
            .await
            .expect("update secondary payload columns");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list payload jobs");
        let job = &response.jobs[0];

        assert_eq!(job.last_result, None);
        assert_eq!(job.metadata, None);
        assert!(!job.last_result_truncated);
        assert!(!job.metadata_truncated);
        let serialized = serde_json::to_string(job).expect("serialize row");
        assert!(!serialized.contains("result-secret"));
        assert!(!serialized.contains("metadata-secret"));
    }

    #[tokio::test]
    async fn apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent() {
        let pool = memory_pool().await;
        sqlx::query(
            "CREATE TABLE Jobs (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                attempts INTEGER NOT NULL,
                job TEXT
             )",
        )
        .execute(&pool)
        .await
        .expect("create reduced Jobs table");
        sqlx::query(
            "INSERT INTO Jobs (id, status, attempts, job) VALUES ('job-1', 'Queued', 2, '{\"safe\":true}')",
        )
        .execute(&pool)
        .await
        .expect("insert reduced job");

        let response = apalis_jobs_list_from_pool(&pool, ApalisJobsListRequest::default())
            .await
            .expect("list reduced jobs");
        let serialized = serde_json::to_value(&response.jobs[0]).expect("serialize row");

        for key in [
            "id",
            "jobType",
            "status",
            "attempts",
            "maxAttempts",
            "runAt",
            "lockAt",
            "lockBy",
            "doneAt",
            "lastActivityAt",
            "priority",
            "idempotencyKey",
            "jobPreview",
            "jobTruncated",
            "jobJson",
            "lastResult",
            "lastResultTruncated",
            "metadata",
            "metadataTruncated",
        ] {
            assert!(
                serialized.get(key).is_some(),
                "missing serialized key {key}"
            );
        }

        assert_eq!(response.jobs[0].job_type, "");
        assert_eq!(response.jobs[0].status, "Queued");
        assert_eq!(response.jobs[0].last_result, None);
        assert_eq!(response.jobs[0].metadata, None);
    }
}
