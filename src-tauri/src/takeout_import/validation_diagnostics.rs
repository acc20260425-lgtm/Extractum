use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationCount {
    pub(crate) key: String,
    pub(crate) count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationSourceSnapshot {
    pub(crate) source_id: i64,
    pub(crate) source_type: String,
    pub(crate) source_subtype: Option<String>,
    pub(crate) account_id: Option<i64>,
    pub(crate) last_sync_state: Option<i64>,
    pub(crate) last_synced_at: Option<i64>,
    pub(crate) item_count: i64,
    pub(crate) telegram_typed_row_count: i64,
    pub(crate) max_telegram_message_id: Option<i64>,
    pub(crate) content_zstd_present_count: i64,
    pub(crate) content_kind_distribution: Vec<TakeoutValidationCount>,
    pub(crate) media_kind_distribution: Vec<TakeoutValidationCount>,
    pub(crate) history_peer_kind_distribution: Vec<TakeoutValidationCount>,
    pub(crate) reply_to_msg_id_present_count: i64,
    pub(crate) reply_to_top_id_present_count: i64,
    pub(crate) reaction_count_present_count: i64,
    pub(crate) reaction_count_sum: i64,
    pub(crate) topic_membership_count: i64,
    pub(crate) topic_membership_topic_count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationBatchSummary {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) status: String,
    pub(crate) completeness: String,
    pub(crate) item_inserted_count: i64,
    pub(crate) item_duplicate_count: i64,
    pub(crate) item_skipped_count: i64,
    pub(crate) item_observed_count: i64,
    pub(crate) warning_count: i64,
    pub(crate) warning_codes: Vec<String>,
    pub(crate) terminal_error_present: bool,
    pub(crate) started_at: i64,
    pub(crate) finished_at: Option<i64>,
    pub(crate) updated_at: i64,
    pub(crate) account_id: i64,
    pub(crate) source_subtype: String,
    pub(crate) resolved_peer_kind_present: bool,
    pub(crate) resolved_peer_id_present: bool,
    pub(crate) history_peer_kind: Option<String>,
    pub(crate) history_peer_id_present: bool,
    pub(crate) takeout_id_present: bool,
    pub(crate) export_dc_id: Option<i64>,
    pub(crate) used_export_dc: bool,
    pub(crate) fallback_used: bool,
    pub(crate) history_scope: String,
    pub(crate) migrated_history_detected: bool,
    pub(crate) migrated_history_imported: bool,
    pub(crate) only_my_messages: bool,
    pub(crate) split_count: Option<i64>,
    pub(crate) selected_split_count: Option<i64>,
    pub(crate) message_count_estimate: Option<i64>,
    pub(crate) max_message_id: Option<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationMismatchCategory {
    pub(crate) category: String,
    pub(crate) count: i64,
    pub(crate) sample_ids: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationRowFidelityComparison {
    pub(crate) mode: String,
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) observed_identity_count: i64,
    pub(crate) matched_canonical_identity_count: i64,
    pub(crate) missing_canonical_identity_count: i64,
    pub(crate) canonical_without_observation_count: i64,
    pub(crate) matched_content_zstd_present_count: i64,
    pub(crate) matched_content_kind_distribution: Vec<TakeoutValidationCount>,
    pub(crate) matched_media_kind_distribution: Vec<TakeoutValidationCount>,
    pub(crate) matched_reply_to_msg_id_present_count: i64,
    pub(crate) matched_reply_to_top_id_present_count: i64,
    pub(crate) matched_reaction_count_present_count: i64,
    pub(crate) mismatch_categories: Vec<TakeoutValidationMismatchCategory>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationDuplicateSummary {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) inserted_count: i64,
    pub(crate) duplicate_observed_count: i64,
    pub(crate) skipped_count: i64,
    pub(crate) failed_count: i64,
    pub(crate) duplicate_identity_count: i64,
    pub(crate) has_duplicate_after_normal_sync_evidence: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationSnapshotDelta {
    pub(crate) mode: String,
    pub(crate) source_id: i64,
    pub(crate) item_count_delta: i64,
    pub(crate) telegram_typed_row_count_delta: i64,
    pub(crate) content_zstd_present_count_delta: i64,
    pub(crate) topic_membership_count_delta: i64,
    pub(crate) max_telegram_message_id_before: Option<i64>,
    pub(crate) max_telegram_message_id_after: Option<i64>,
    pub(crate) last_sync_state_before: Option<i64>,
    pub(crate) last_sync_state_after: Option<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutValidationWarningVisibility {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) provenance_warning_codes: Vec<String>,
    pub(crate) batch_is_latest_for_source: bool,
    pub(crate) durable_recovery_kind: Option<String>,
    pub(crate) recovery_candidate_warning_codes: Vec<String>,
}

#[derive(Debug, FromRow)]
struct SourceSnapshotBaseRow {
    source_id: i64,
    source_type: String,
    source_subtype: Option<String>,
    account_id: Option<i64>,
    last_sync_state: Option<i64>,
    last_synced_at: Option<i64>,
}

#[derive(Debug, FromRow)]
struct BatchSummaryRow {
    batch_id: i64,
    source_id: i64,
    status: String,
    completeness: String,
    item_inserted_count: i64,
    item_duplicate_count: i64,
    item_skipped_count: i64,
    item_observed_count: i64,
    warning_count: i64,
    terminal_error_present: i64,
    started_at: i64,
    finished_at: Option<i64>,
    updated_at: i64,
    account_id: i64,
    source_subtype: String,
    resolved_peer_kind_present: i64,
    resolved_peer_id_present: i64,
    history_peer_kind: Option<String>,
    history_peer_id_present: i64,
    takeout_id_present: i64,
    export_dc_id: Option<i64>,
    used_export_dc: i64,
    fallback_used: i64,
    history_scope: String,
    migrated_history_detected: i64,
    migrated_history_imported: i64,
    only_my_messages: i64,
    split_count: Option<i64>,
    selected_split_count: Option<i64>,
    message_count_estimate: Option<i64>,
    max_message_id: Option<i64>,
}

fn bool_from_sql(value: i64) -> bool {
    value != 0
}

pub(crate) async fn takeout_validation_source_snapshot(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<Option<TakeoutValidationSourceSnapshot>> {
    let Some(base) = sqlx::query_as::<_, SourceSnapshotBaseRow>(
        r#"
        SELECT
            id AS source_id,
            source_type,
            source_subtype,
            account_id,
            last_sync_state,
            last_synced_at
        FROM sources
        WHERE id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    else {
        return Ok(None);
    };

    let item_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM items WHERE source_id = ?",
        source_id,
    )
    .await?;
    let telegram_typed_row_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM telegram_messages WHERE source_id = ?",
        source_id,
    )
    .await?;
    let max_telegram_message_id = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MAX(telegram_message_id) FROM telegram_messages WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let content_zstd_present_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM items WHERE source_id = ? AND content_zstd IS NOT NULL",
        source_id,
    )
    .await?;
    let reply_to_msg_id_present_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM telegram_messages WHERE source_id = ? AND reply_to_msg_id IS NOT NULL",
        source_id,
    )
    .await?;
    let reply_to_top_id_present_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM telegram_messages WHERE source_id = ? AND reply_to_top_id IS NOT NULL",
        source_id,
    )
    .await?;
    let reaction_count_present_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM telegram_messages WHERE source_id = ? AND reaction_count IS NOT NULL",
        source_id,
    )
    .await?;
    let reaction_count_sum = scalar_i64(
        pool,
        "SELECT COALESCE(SUM(reaction_count), 0) FROM telegram_messages WHERE source_id = ?",
        source_id,
    )
    .await?;
    let topic_membership_count = scalar_i64(
        pool,
        "SELECT COUNT(*) FROM item_topic_memberships WHERE source_id = ?",
        source_id,
    )
    .await?;
    let topic_membership_topic_count = scalar_i64(
        pool,
        "SELECT COUNT(DISTINCT topic_id) FROM item_topic_memberships WHERE source_id = ?",
        source_id,
    )
    .await?;

    Ok(Some(TakeoutValidationSourceSnapshot {
        source_id: base.source_id,
        source_type: base.source_type,
        source_subtype: base.source_subtype,
        account_id: base.account_id,
        last_sync_state: base.last_sync_state,
        last_synced_at: base.last_synced_at,
        item_count,
        telegram_typed_row_count,
        max_telegram_message_id,
        content_zstd_present_count,
        content_kind_distribution: distribution(
            pool,
            "SELECT content_kind, COUNT(*) FROM items WHERE source_id = ? GROUP BY content_kind ORDER BY content_kind ASC",
            source_id,
        )
        .await?,
        media_kind_distribution: distribution(
            pool,
            "SELECT COALESCE(media_kind, 'none'), COUNT(*) FROM items WHERE source_id = ? GROUP BY COALESCE(media_kind, 'none') ORDER BY COALESCE(media_kind, 'none') ASC",
            source_id,
        )
        .await?,
        history_peer_kind_distribution: distribution(
            pool,
            "SELECT history_peer_kind, COUNT(*) FROM telegram_messages WHERE source_id = ? GROUP BY history_peer_kind ORDER BY history_peer_kind ASC",
            source_id,
        )
        .await?,
        reply_to_msg_id_present_count,
        reply_to_top_id_present_count,
        reaction_count_present_count,
        reaction_count_sum,
        topic_membership_count,
        topic_membership_topic_count,
    }))
}

async fn scalar_i64(pool: &SqlitePool, sql: &str, id: i64) -> AppResult<i64> {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)
}

async fn distribution(
    pool: &SqlitePool,
    sql: &str,
    id: i64,
) -> AppResult<Vec<TakeoutValidationCount>> {
    let rows = sqlx::query_as::<_, (String, i64)>(sql)
        .bind(id)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    Ok(rows
        .into_iter()
        .map(|(key, count)| TakeoutValidationCount { key, count })
        .collect())
}

pub(crate) async fn takeout_validation_batch_summary(
    pool: &SqlitePool,
    batch_id: i64,
) -> AppResult<Option<TakeoutValidationBatchSummary>> {
    let Some(row) = sqlx::query_as::<_, BatchSummaryRow>(
        r#"
        SELECT
            b.id AS batch_id,
            b.source_id,
            b.status,
            b.completeness,
            b.item_inserted_count,
            b.item_duplicate_count,
            b.item_skipped_count,
            b.item_observed_count,
            b.warning_count,
            CASE WHEN b.terminal_error IS NULL THEN 0 ELSE 1 END AS terminal_error_present,
            CAST(strftime('%s', b.started_at) AS INTEGER) AS started_at,
            CASE
              WHEN b.finished_at IS NULL THEN NULL
              ELSE CAST(strftime('%s', b.finished_at) AS INTEGER)
            END AS finished_at,
            CAST(strftime('%s', b.updated_at) AS INTEGER) AS updated_at,
            t.account_id,
            t.source_subtype,
            CASE WHEN t.resolved_peer_kind IS NULL THEN 0 ELSE 1 END AS resolved_peer_kind_present,
            CASE WHEN t.resolved_peer_id IS NULL THEN 0 ELSE 1 END AS resolved_peer_id_present,
            t.history_peer_kind,
            CASE WHEN t.history_peer_id IS NULL THEN 0 ELSE 1 END AS history_peer_id_present,
            CASE WHEN t.takeout_id IS NULL THEN 0 ELSE 1 END AS takeout_id_present,
            t.export_dc_id,
            t.used_export_dc,
            t.fallback_used,
            t.history_scope,
            t.migrated_history_detected,
            t.migrated_history_imported,
            t.only_my_messages,
            t.split_count,
            t.selected_split_count,
            t.message_count_estimate,
            t.max_message_id
        FROM ingest_batches b
        JOIN telegram_takeout_batches t ON t.batch_id = b.id
        WHERE b.id = ?
          AND b.provider = 'telegram'
          AND b.ingest_kind = 'takeout'
        "#,
    )
    .bind(batch_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    else {
        return Ok(None);
    };

    Ok(Some(TakeoutValidationBatchSummary {
        batch_id: row.batch_id,
        source_id: row.source_id,
        status: row.status,
        completeness: row.completeness,
        item_inserted_count: row.item_inserted_count,
        item_duplicate_count: row.item_duplicate_count,
        item_skipped_count: row.item_skipped_count,
        item_observed_count: row.item_observed_count,
        warning_count: row.warning_count,
        warning_codes: warning_codes_for_batch(pool, batch_id).await?,
        terminal_error_present: bool_from_sql(row.terminal_error_present),
        started_at: row.started_at,
        finished_at: row.finished_at,
        updated_at: row.updated_at,
        account_id: row.account_id,
        source_subtype: row.source_subtype,
        resolved_peer_kind_present: bool_from_sql(row.resolved_peer_kind_present),
        resolved_peer_id_present: bool_from_sql(row.resolved_peer_id_present),
        history_peer_kind: row.history_peer_kind,
        history_peer_id_present: bool_from_sql(row.history_peer_id_present),
        takeout_id_present: bool_from_sql(row.takeout_id_present),
        export_dc_id: row.export_dc_id,
        used_export_dc: bool_from_sql(row.used_export_dc),
        fallback_used: bool_from_sql(row.fallback_used),
        history_scope: row.history_scope,
        migrated_history_detected: bool_from_sql(row.migrated_history_detected),
        migrated_history_imported: bool_from_sql(row.migrated_history_imported),
        only_my_messages: bool_from_sql(row.only_my_messages),
        split_count: row.split_count,
        selected_split_count: row.selected_split_count,
        message_count_estimate: row.message_count_estimate,
        max_message_id: row.max_message_id,
    }))
}

async fn warning_codes_for_batch(pool: &SqlitePool, batch_id: i64) -> AppResult<Vec<String>> {
    sqlx::query_scalar(
        "SELECT code FROM ingest_batch_warnings WHERE batch_id = ? GROUP BY code ORDER BY code ASC",
    )
    .bind(batch_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn takeout_validation_batch_vs_canonical_source(
    pool: &SqlitePool,
    batch_id: i64,
) -> AppResult<Option<TakeoutValidationRowFidelityComparison>> {
    let Some(summary) = takeout_validation_batch_summary(pool, batch_id).await? else {
        return Ok(None);
    };
    let source_id = summary.source_id;

    let observed_identity_count = scalar_i64(
        pool,
        "SELECT COUNT(DISTINCT provider_identity) FROM ingest_item_observations WHERE batch_id = ?",
        batch_id,
    )
    .await?;
    let matched_canonical_identity_count = scalar_i64(
        pool,
        r#"
        SELECT COUNT(DISTINCT o.provider_identity)
        FROM ingest_item_observations o
        JOIN telegram_messages tm
          ON o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
        WHERE o.batch_id = ?
          AND tm.source_id = o.source_id
        "#,
        batch_id,
    )
    .await?;
    let missing_canonical_identity_count = scalar_i64(
        pool,
        r#"
        WITH missing AS (
            SELECT DISTINCT o.source_id, o.provider_identity
            FROM ingest_item_observations o
            LEFT JOIN telegram_messages tm
              ON o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
             AND tm.source_id = o.source_id
            WHERE o.batch_id = ?
              AND tm.item_id IS NULL
        )
        SELECT COUNT(*) FROM missing
        "#,
        batch_id,
    )
    .await?;
    let canonical_without_observation_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM telegram_messages tm
        LEFT JOIN ingest_item_observations o
          ON o.batch_id = ?
         AND o.source_id = tm.source_id
         AND o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
        WHERE tm.source_id = (SELECT source_id FROM ingest_batches WHERE id = ?)
          AND o.id IS NULL
        "#,
    )
    .bind(batch_id)
    .bind(batch_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let matched_content_zstd_present_count: i64 = sqlx::query_scalar(
        r#"
        WITH observed AS (
            SELECT DISTINCT source_id, provider_identity
            FROM ingest_item_observations
            WHERE batch_id = ?
        ),
        matched AS (
            SELECT DISTINCT tm.item_id
            FROM observed o
            JOIN telegram_messages tm
              ON o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
             AND tm.source_id = o.source_id
        )
        SELECT COUNT(*)
        FROM matched tm
        JOIN items i ON i.id = tm.item_id
        WHERE i.content_zstd IS NOT NULL
        "#,
    )
    .bind(batch_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let mut mismatch_categories = Vec::new();
    push_mismatch_category(
        pool,
        &mut mismatch_categories,
        "canonical_identity_missing_observation",
        canonical_without_observation_count,
        r#"
        SELECT tm.item_id
        FROM telegram_messages tm
        LEFT JOIN ingest_item_observations o
          ON o.batch_id = ?
         AND o.source_id = tm.source_id
         AND o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
        WHERE tm.source_id = (SELECT source_id FROM ingest_batches WHERE id = ?)
          AND o.id IS NULL
        ORDER BY tm.item_id ASC
        LIMIT 10
        "#,
        batch_id,
    )
    .await?;
    push_mismatch_category(
        pool,
        &mut mismatch_categories,
        "observed_identity_missing_canonical",
        missing_canonical_identity_count,
        r#"
        SELECT MIN(o.id)
        FROM ingest_item_observations o
        LEFT JOIN telegram_messages tm
          ON o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
         AND tm.source_id = o.source_id
        WHERE o.batch_id = ?
          AND tm.item_id IS NULL
          AND o.source_id = (SELECT source_id FROM ingest_batches WHERE id = ?)
        GROUP BY o.source_id, o.provider_identity
        ORDER BY MIN(o.id) ASC
        LIMIT 10
        "#,
        batch_id,
    )
    .await?;
    mismatch_categories.sort_by(|left, right| left.category.cmp(&right.category));

    Ok(Some(TakeoutValidationRowFidelityComparison {
        mode: "takeout_batch_vs_canonical_source".to_string(),
        batch_id,
        source_id,
        observed_identity_count,
        matched_canonical_identity_count,
        missing_canonical_identity_count,
        canonical_without_observation_count,
        matched_content_zstd_present_count,
        matched_content_kind_distribution: matched_distribution(pool, batch_id, "i.content_kind")
            .await?,
        matched_media_kind_distribution: matched_distribution(
            pool,
            batch_id,
            "COALESCE(i.media_kind, 'none')",
        )
        .await?,
        matched_reply_to_msg_id_present_count: matched_present_count(
            pool,
            batch_id,
            "tm.reply_to_msg_id",
        )
        .await?,
        matched_reply_to_top_id_present_count: matched_present_count(
            pool,
            batch_id,
            "tm.reply_to_top_id",
        )
        .await?,
        matched_reaction_count_present_count: matched_present_count(
            pool,
            batch_id,
            "tm.reaction_count",
        )
        .await?,
        mismatch_categories,
    }))
}

async fn push_mismatch_category(
    pool: &SqlitePool,
    categories: &mut Vec<TakeoutValidationMismatchCategory>,
    category: &str,
    count: i64,
    sample_sql: &str,
    batch_id: i64,
) -> AppResult<()> {
    if count == 0 {
        return Ok(());
    }
    let sample_ids = sqlx::query_scalar(sample_sql)
        .bind(batch_id)
        .bind(batch_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    categories.push(TakeoutValidationMismatchCategory {
        category: category.to_string(),
        count,
        sample_ids,
    });
    Ok(())
}

async fn matched_distribution(
    pool: &SqlitePool,
    batch_id: i64,
    expression: &str,
) -> AppResult<Vec<TakeoutValidationCount>> {
    let sql = format!(
        "WITH observed AS (
             SELECT DISTINCT source_id, provider_identity
             FROM ingest_item_observations
             WHERE batch_id = ?
         ),
         matched AS (
             SELECT DISTINCT tm.item_id
             FROM observed o
             JOIN telegram_messages tm
               ON o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
              AND tm.source_id = o.source_id
         )
         SELECT {expression}, COUNT(*)
         FROM matched tm
         JOIN items i ON i.id = tm.item_id
         GROUP BY {expression}
         ORDER BY {expression} ASC"
    );
    distribution(pool, &sql, batch_id).await
}

async fn matched_present_count(
    pool: &SqlitePool,
    batch_id: i64,
    expression: &str,
) -> AppResult<i64> {
    let sql = format!(
        "WITH observed AS (
             SELECT DISTINCT source_id, provider_identity
             FROM ingest_item_observations
             WHERE batch_id = ?
         ),
         matched AS (
             SELECT DISTINCT
                 tm.item_id,
                 tm.reply_to_msg_id,
                 tm.reply_to_top_id,
                 tm.reaction_count
             FROM observed o
             JOIN telegram_messages tm
               ON o.provider_identity = 'telegram:history_peer:' || tm.history_peer_kind || ':' || tm.history_peer_id || ':message:' || tm.telegram_message_id
              AND tm.source_id = o.source_id
         )
         SELECT COUNT(*)
         FROM matched tm
         WHERE {expression} IS NOT NULL"
    );
    scalar_i64(pool, &sql, batch_id).await
}

pub(crate) async fn takeout_validation_duplicate_after_normal_sync(
    pool: &SqlitePool,
    batch_id: i64,
) -> AppResult<Option<TakeoutValidationDuplicateSummary>> {
    let Some(summary) = takeout_validation_batch_summary(pool, batch_id).await? else {
        return Ok(None);
    };

    let inserted_count = outcome_count(pool, batch_id, "inserted").await?;
    let duplicate_observed_count = outcome_count(pool, batch_id, "duplicate_observed").await?;
    let skipped_count = outcome_count(pool, batch_id, "skipped").await?;
    let failed_count = outcome_count(pool, batch_id, "failed").await?;
    let duplicate_identity_count = scalar_i64(
        pool,
        "SELECT COUNT(DISTINCT provider_identity) FROM ingest_item_observations WHERE batch_id = ? AND outcome = 'duplicate_observed'",
        batch_id,
    )
    .await?;

    Ok(Some(TakeoutValidationDuplicateSummary {
        batch_id,
        source_id: summary.source_id,
        inserted_count,
        duplicate_observed_count,
        skipped_count,
        failed_count,
        duplicate_identity_count,
        has_duplicate_after_normal_sync_evidence: duplicate_observed_count > 0,
    }))
}

async fn outcome_count(pool: &SqlitePool, batch_id: i64, outcome: &str) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM ingest_item_observations WHERE batch_id = ? AND outcome = ?",
    )
    .bind(batch_id)
    .bind(outcome)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) fn takeout_validation_snapshot_delta(
    before: &TakeoutValidationSourceSnapshot,
    after: &TakeoutValidationSourceSnapshot,
) -> TakeoutValidationSnapshotDelta {
    TakeoutValidationSnapshotDelta {
        mode: "before_after_snapshot_delta".to_string(),
        source_id: after.source_id,
        item_count_delta: after.item_count - before.item_count,
        telegram_typed_row_count_delta: after.telegram_typed_row_count
            - before.telegram_typed_row_count,
        content_zstd_present_count_delta: after.content_zstd_present_count
            - before.content_zstd_present_count,
        topic_membership_count_delta: after.topic_membership_count - before.topic_membership_count,
        max_telegram_message_id_before: before.max_telegram_message_id,
        max_telegram_message_id_after: after.max_telegram_message_id,
        last_sync_state_before: before.last_sync_state,
        last_sync_state_after: after.last_sync_state,
    }
}

pub(crate) async fn takeout_validation_warning_visibility(
    pool: &SqlitePool,
    batch_id: i64,
) -> AppResult<Option<TakeoutValidationWarningVisibility>> {
    let Some(summary) = takeout_validation_batch_summary(pool, batch_id).await? else {
        return Ok(None);
    };
    let latest_batch_id: Option<i64> = sqlx::query_scalar(
        "SELECT b.id
         FROM ingest_batches b
         JOIN telegram_takeout_batches t ON t.batch_id = b.id
         WHERE b.source_id = ?
           AND b.provider = 'telegram'
           AND b.ingest_kind = 'takeout'
         ORDER BY b.started_at DESC, b.id DESC
         LIMIT 1",
    )
    .bind(summary.source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;
    let batch_is_latest_for_source = latest_batch_id == Some(batch_id);
    let durable_recovery_kind = if batch_is_latest_for_source {
        durable_recovery_kind(&summary.status, &summary.completeness).map(str::to_string)
    } else {
        None
    };
    let provenance_warning_codes = warning_codes_for_batch(pool, batch_id).await?;
    let recovery_candidate_warning_codes = if durable_recovery_kind.is_some() {
        provenance_warning_codes.clone()
    } else {
        Vec::new()
    };

    Ok(Some(TakeoutValidationWarningVisibility {
        batch_id,
        source_id: summary.source_id,
        provenance_warning_codes,
        batch_is_latest_for_source,
        durable_recovery_kind,
        recovery_candidate_warning_codes,
    }))
}

fn durable_recovery_kind(status: &str, completeness: &str) -> Option<&'static str> {
    match (status, completeness) {
        ("running", _) => Some("interrupted"),
        ("failed", _) => Some("failed"),
        ("cancelled", _) => Some("cancelled"),
        ("completed", "partial") => Some("partial_completed"),
        ("completed", _) => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest_provenance::{
        create_telegram_takeout_batch, finalize_ingest_batch, mark_takeout_export_dc_attempted,
        mark_takeout_export_dc_fallback, mark_takeout_migrated_history_deferred,
        mark_takeout_only_my_messages_fallback, record_ingest_observation,
        CreateTelegramTakeoutBatch, IngestObservation, TerminalBatchStatus,
    };
    use crate::sources::test_support::{
        create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
    };

    const SENTINEL_TITLE: &str = "sentinel_private_title_do_not_emit";
    const SENTINEL_TOPIC_TITLE: &str = "sentinel_private_topic_title_do_not_emit";
    const SENTINEL_USERNAME: &str = "sentinel_private_username_do_not_emit";
    const SENTINEL_EXTERNAL_ID: &str = "sentinel_external_id_do_not_emit";
    const SENTINEL_MESSAGE: &str = "sentinel_message_text_do_not_emit";
    const SENTINEL_METADATA: &str = "sentinel_raw_metadata_do_not_emit";
    const SENTINEL_WARNING: &str = "sentinel_warning_body_do_not_emit";
    const SENTINEL_REASON_CODE: &str = "sentinel_reason_code_do_not_emit";

    async fn fixture_pool() -> sqlx::SqlitePool {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool, 7, "supergroup").await;
        pool
    }

    async fn seed_source(pool: &sqlx::SqlitePool, source_id: i64, subtype: &str) {
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                last_sync_state, last_synced_at, is_active, is_member, created_at,
                metadata_zstd
            )
            VALUES (?, 'telegram', ?, 3, ?, ?, 42, 1700000500, 1, 1, 1700000000, ?)
            "#,
        )
        .bind(source_id)
        .bind(subtype)
        .bind(SENTINEL_EXTERNAL_ID)
        .bind(SENTINEL_TITLE)
        .bind(SENTINEL_METADATA.as_bytes())
        .execute(pool)
        .await
        .expect("seed source");

        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash
            )
            VALUES (?, 3, ?, 'channel', 7000, 'dialog', ?, 9000)
            "#,
        )
        .bind(source_id)
        .bind(subtype)
        .bind(SENTINEL_USERNAME)
        .execute(pool)
        .await
        .expect("seed telegram source");
    }

    async fn seed_canonical_message(
        pool: &sqlx::SqlitePool,
        item_id: i64,
        message_id: i64,
        content_kind: &str,
        media_kind: Option<&str>,
        reply_to_top_id: Option<i64>,
        reaction_count: Option<i64>,
    ) {
        sqlx::query(
            r#"
            INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media, media_kind,
                media_metadata_zstd, reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id,
                reply_to_top_id, reaction_count
            )
            VALUES (
                ?, 7, ?, 'telegram_message', NULL, 1700000100, 1700000200,
                ?, ?, ?, ?, ?, ?, 11, 'channel', '7000', ?, ?
            )
            "#,
        )
        .bind(item_id)
        .bind(message_id.to_string())
        .bind(SENTINEL_MESSAGE.as_bytes())
        .bind(SENTINEL_METADATA.as_bytes())
        .bind(content_kind)
        .bind(i64::from(media_kind.is_some()))
        .bind(media_kind)
        .bind(SENTINEL_METADATA.as_bytes())
        .bind(reply_to_top_id)
        .bind(reaction_count)
        .execute(pool)
        .await
        .expect("seed item");

        sqlx::query(
            r#"
            INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history,
                reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id,
                reply_to_top_id, reaction_count
            )
            VALUES (?, 7, 'channel', 7000, ?, NULL, 0, 11, 'channel', 7000, ?, ?)
            "#,
        )
        .bind(item_id)
        .bind(message_id)
        .bind(reply_to_top_id)
        .bind(reaction_count)
        .execute(pool)
        .await
        .expect("seed telegram message");
    }

    fn assert_no_sentinel_json<T: serde::Serialize>(value: &T) {
        let json = serde_json::to_string(value).expect("serialize diagnostic output");
        for sentinel in [
            SENTINEL_TITLE,
            SENTINEL_TOPIC_TITLE,
            SENTINEL_USERNAME,
            SENTINEL_EXTERNAL_ID,
            SENTINEL_MESSAGE,
            SENTINEL_METADATA,
            SENTINEL_WARNING,
            SENTINEL_REASON_CODE,
        ] {
            assert!(
                !json.contains(sentinel),
                "diagnostic output leaked sentinel {sentinel}: {json}"
            );
        }
    }

    async fn create_running_batch(pool: &sqlx::SqlitePool) -> i64 {
        create_telegram_takeout_batch(
            pool,
            CreateTelegramTakeoutBatch {
                source_id: 7,
                account_id: 3,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create takeout batch")
    }

    async fn record_observation(
        pool: &sqlx::SqlitePool,
        batch_id: i64,
        item_id: Option<i64>,
        message_id: i64,
        outcome: &'static str,
    ) {
        record_observation_with_reason(pool, batch_id, item_id, message_id, outcome, None).await;
    }

    async fn record_observation_with_reason(
        pool: &sqlx::SqlitePool,
        batch_id: i64,
        item_id: Option<i64>,
        message_id: i64,
        outcome: &'static str,
        reason_code: Option<&'static str>,
    ) {
        record_ingest_observation(
            pool,
            IngestObservation {
                batch_id,
                source_id: 7,
                item_id,
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: format!(
                    "telegram:history_peer:channel:7000:message:{message_id}"
                ),
                outcome,
                reason_code,
            },
        )
        .await
        .expect("record observation");
    }

    #[tokio::test]
    async fn takeout_validation_source_snapshot_is_aggregate_and_sanitized() {
        let pool = fixture_pool().await;
        seed_canonical_message(&pool, 101, 1001, "text_only", None, Some(77), Some(2)).await;
        seed_canonical_message(&pool, 102, 1002, "media", Some("photo"), None, None).await;

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, last_seen_at, updated_at
            )
            VALUES (7, 77, 77, ?, 1700000300, 1700000300)
            "#,
        )
        .bind(SENTINEL_TOPIC_TITLE)
        .execute(&pool)
        .await
        .expect("seed forum topic");

        sqlx::query(
            "INSERT INTO item_topic_memberships (item_id, source_id, topic_id, match_kind, resolver_version)
             VALUES (101, 7, 77, 'reply_to_top_id', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed topic membership");

        let snapshot = takeout_validation_source_snapshot(&pool, 7)
            .await
            .expect("source snapshot")
            .expect("source exists");

        assert_eq!(snapshot.source_id, 7);
        assert_eq!(snapshot.source_type, "telegram");
        assert_eq!(snapshot.source_subtype.as_deref(), Some("supergroup"));
        assert_eq!(snapshot.account_id, Some(3));
        assert_eq!(snapshot.last_sync_state, Some(42));
        assert_eq!(snapshot.last_synced_at, Some(1700000500));
        assert_eq!(snapshot.item_count, 2);
        assert_eq!(snapshot.telegram_typed_row_count, 2);
        assert_eq!(snapshot.max_telegram_message_id, Some(1002));
        assert_eq!(snapshot.content_zstd_present_count, 2);
        assert_eq!(snapshot.reply_to_msg_id_present_count, 2);
        assert_eq!(snapshot.reply_to_top_id_present_count, 1);
        assert_eq!(snapshot.reaction_count_present_count, 1);
        assert_eq!(snapshot.reaction_count_sum, 2);
        assert_eq!(snapshot.topic_membership_count, 1);
        assert_eq!(snapshot.topic_membership_topic_count, 1);
        assert!(snapshot
            .content_kind_distribution
            .iter()
            .any(|entry| entry.key == "media" && entry.count == 1));
        assert!(snapshot
            .media_kind_distribution
            .iter()
            .any(|entry| entry.key == "photo" && entry.count == 1));
        assert_no_sentinel_json(&snapshot);
    }

    #[tokio::test]
    async fn takeout_validation_batch_summary_is_durable_and_sanitized() {
        let pool = fixture_pool().await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 7,
                account_id: 3,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create takeout batch");

        mark_takeout_export_dc_attempted(&pool, batch_id, 5)
            .await
            .expect("mark export dc attempted");
        mark_takeout_export_dc_fallback(&pool, batch_id, SENTINEL_WARNING)
            .await
            .expect("mark export dc fallback");
        mark_takeout_export_dc_fallback(&pool, batch_id, SENTINEL_WARNING)
            .await
            .expect("mark duplicate export dc fallback");
        mark_takeout_only_my_messages_fallback(&pool, batch_id, SENTINEL_WARNING)
            .await
            .expect("mark only-my-messages fallback");
        mark_takeout_migrated_history_deferred(&pool, batch_id, SENTINEL_WARNING)
            .await
            .expect("mark migrated deferred");
        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 7,
                item_id: None,
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:7000:message:1001".to_string(),
                outcome: "skipped",
                reason_code: Some("validation_skip"),
            },
        )
        .await
        .expect("record observation");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize batch");

        let summary = takeout_validation_batch_summary(&pool, batch_id)
            .await
            .expect("batch summary")
            .expect("batch exists");

        assert_eq!(summary.batch_id, batch_id);
        assert_eq!(summary.source_id, 7);
        assert_eq!(summary.status, "completed");
        assert_eq!(summary.completeness, "partial");
        assert_eq!(summary.item_observed_count, 1);
        assert_eq!(summary.item_skipped_count, 1);
        assert_eq!(summary.warning_count, 4);
        assert_eq!(
            summary.warning_codes,
            vec![
                "export_dc_fallback".to_string(),
                "migrated_history_deferred".to_string(),
                "only_my_messages_fallback".to_string(),
            ]
        );
        assert_eq!(
            summary
                .warning_codes
                .iter()
                .filter(|code| code.as_str() == "export_dc_fallback")
                .count(),
            1
        );
        assert!(summary.used_export_dc);
        assert!(summary.fallback_used);
        assert!(summary.only_my_messages);
        assert!(summary.migrated_history_detected);
        assert!(!summary.migrated_history_imported);
        assert!(!summary.terminal_error_present);
        assert_no_sentinel_json(&summary);
    }

    #[tokio::test]
    async fn takeout_validation_row_fidelity_compares_batch_to_canonical_without_content() {
        let pool = fixture_pool().await;
        seed_canonical_message(&pool, 101, 1001, "text_only", None, Some(77), Some(2)).await;
        seed_canonical_message(&pool, 102, 1002, "media", Some("photo"), None, None).await;
        let batch_id = create_running_batch(&pool).await;
        record_observation(&pool, batch_id, Some(101), 1001, "duplicate_observed").await;
        record_observation_with_reason(
            &pool,
            batch_id,
            None,
            1999,
            "skipped",
            Some(SENTINEL_REASON_CODE),
        )
        .await;

        let comparison = takeout_validation_batch_vs_canonical_source(&pool, batch_id)
            .await
            .expect("row fidelity comparison")
            .expect("comparison exists");

        assert_eq!(comparison.mode, "takeout_batch_vs_canonical_source");
        assert_eq!(comparison.batch_id, batch_id);
        assert_eq!(comparison.source_id, 7);
        assert_eq!(comparison.observed_identity_count, 2);
        assert_eq!(comparison.matched_canonical_identity_count, 1);
        assert_eq!(comparison.missing_canonical_identity_count, 1);
        assert_eq!(comparison.canonical_without_observation_count, 1);
        assert_eq!(comparison.matched_content_zstd_present_count, 1);
        assert!(comparison
            .matched_content_kind_distribution
            .iter()
            .any(|entry| entry.key == "text_only" && entry.count == 1));
        assert!(comparison
            .matched_media_kind_distribution
            .iter()
            .any(|entry| entry.key == "none" && entry.count == 1));
        assert_eq!(comparison.matched_reply_to_msg_id_present_count, 1);
        assert_eq!(comparison.matched_reply_to_top_id_present_count, 1);
        assert_eq!(comparison.matched_reaction_count_present_count, 1);
        assert_eq!(
            comparison.mismatch_categories[0].category,
            "canonical_identity_missing_observation"
        );
        assert_eq!(comparison.mismatch_categories[0].sample_ids, vec![102]);
        assert_eq!(
            comparison.mismatch_categories[1].category,
            "observed_identity_missing_canonical"
        );
        assert_no_sentinel_json(&comparison);
    }

    #[tokio::test]
    async fn takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates() {
        let pool = fixture_pool().await;
        seed_canonical_message(&pool, 101, 1001, "text_only", None, Some(77), Some(2)).await;
        let batch_id = create_running_batch(&pool).await;
        record_observation(&pool, batch_id, Some(101), 1001, "duplicate_observed").await;
        record_observation(&pool, batch_id, Some(101), 1001, "duplicate_observed").await;
        record_observation(&pool, batch_id, Some(101), 1001, "duplicate_observed").await;

        let comparison = takeout_validation_batch_vs_canonical_source(&pool, batch_id)
            .await
            .expect("row fidelity comparison")
            .expect("comparison exists");

        assert_eq!(comparison.observed_identity_count, 1);
        assert_eq!(comparison.matched_canonical_identity_count, 1);
        assert_eq!(comparison.matched_content_zstd_present_count, 1);
        assert_eq!(
            comparison.matched_content_kind_distribution,
            vec![TakeoutValidationCount {
                key: "text_only".to_string(),
                count: 1,
            }]
        );
        assert_eq!(
            comparison.matched_media_kind_distribution,
            vec![TakeoutValidationCount {
                key: "none".to_string(),
                count: 1,
            }]
        );
        assert_eq!(comparison.matched_reply_to_msg_id_present_count, 1);
        assert_eq!(comparison.matched_reply_to_top_id_present_count, 1);
        assert_eq!(comparison.matched_reaction_count_present_count, 1);
        assert_no_sentinel_json(&comparison);
    }

    #[tokio::test]
    async fn takeout_validation_row_fidelity_dedupes_missing_observations_by_identity() {
        let pool = fixture_pool().await;
        let batch_id = create_running_batch(&pool).await;
        record_observation(&pool, batch_id, None, 1999, "skipped").await;
        record_observation(&pool, batch_id, None, 1999, "skipped").await;
        record_observation(&pool, batch_id, None, 2000, "skipped").await;

        let comparison = takeout_validation_batch_vs_canonical_source(&pool, batch_id)
            .await
            .expect("row fidelity comparison")
            .expect("comparison exists");

        assert_eq!(comparison.observed_identity_count, 2);
        assert_eq!(comparison.missing_canonical_identity_count, 2);
        let missing = comparison
            .mismatch_categories
            .iter()
            .find(|category| category.category == "observed_identity_missing_canonical")
            .expect("missing canonical category");
        assert_eq!(missing.count, 2);
        assert_eq!(missing.sample_ids, vec![1, 3]);
        assert_no_sentinel_json(&comparison);
    }

    #[tokio::test]
    async fn takeout_validation_row_fidelity_caps_samples_deterministically() {
        let pool = fixture_pool().await;
        let batch_id = create_running_batch(&pool).await;
        for offset in 0..12 {
            record_observation(&pool, batch_id, None, 2000 + offset, "skipped").await;
        }

        let comparison = takeout_validation_batch_vs_canonical_source(&pool, batch_id)
            .await
            .expect("row fidelity comparison")
            .expect("comparison exists");

        let missing = comparison
            .mismatch_categories
            .iter()
            .find(|category| category.category == "observed_identity_missing_canonical")
            .expect("missing canonical category");
        assert_eq!(missing.count, 12);
        assert_eq!(missing.sample_ids.len(), 10);
        assert_eq!(missing.sample_ids, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[tokio::test]
    async fn takeout_validation_duplicate_after_normal_sync_summarizes_outcomes() {
        let pool = fixture_pool().await;
        seed_canonical_message(&pool, 101, 1001, "text_only", None, None, None).await;
        seed_canonical_message(&pool, 102, 1002, "text_only", None, None, None).await;
        let batch_id = create_running_batch(&pool).await;
        record_observation(&pool, batch_id, Some(101), 1001, "duplicate_observed").await;
        record_observation(&pool, batch_id, Some(101), 1001, "duplicate_observed").await;
        record_observation(&pool, batch_id, Some(102), 1002, "inserted").await;
        record_observation(&pool, batch_id, None, 1003, "skipped").await;
        record_observation(&pool, batch_id, None, 1004, "failed").await;

        let summary = takeout_validation_duplicate_after_normal_sync(&pool, batch_id)
            .await
            .expect("duplicate summary")
            .expect("summary exists");

        assert_eq!(summary.batch_id, batch_id);
        assert_eq!(summary.source_id, 7);
        assert_eq!(summary.inserted_count, 1);
        assert_eq!(summary.duplicate_observed_count, 2);
        assert_eq!(summary.duplicate_identity_count, 1);
        assert_eq!(summary.skipped_count, 1);
        assert_eq!(summary.failed_count, 1);
        assert!(summary.has_duplicate_after_normal_sync_evidence);
        assert_no_sentinel_json(&summary);
    }

    #[tokio::test]
    async fn takeout_validation_snapshot_delta_uses_explicit_snapshots() {
        let before = TakeoutValidationSourceSnapshot {
            source_id: 7,
            source_type: "telegram".to_string(),
            source_subtype: Some("supergroup".to_string()),
            account_id: Some(3),
            last_sync_state: Some(10),
            last_synced_at: Some(1700000000),
            item_count: 2,
            telegram_typed_row_count: 2,
            max_telegram_message_id: Some(1002),
            content_zstd_present_count: 2,
            content_kind_distribution: vec![TakeoutValidationCount {
                key: "text_only".to_string(),
                count: 2,
            }],
            media_kind_distribution: vec![TakeoutValidationCount {
                key: "none".to_string(),
                count: 2,
            }],
            history_peer_kind_distribution: vec![TakeoutValidationCount {
                key: "channel".to_string(),
                count: 2,
            }],
            reply_to_msg_id_present_count: 0,
            reply_to_top_id_present_count: 0,
            reaction_count_present_count: 0,
            reaction_count_sum: 0,
            topic_membership_count: 0,
            topic_membership_topic_count: 0,
        };
        let mut after = before.clone();
        after.last_sync_state = Some(12);
        after.item_count = 5;
        after.telegram_typed_row_count = 5;
        after.max_telegram_message_id = Some(1005);
        after.content_zstd_present_count = 4;
        after.topic_membership_count = 2;

        let delta = takeout_validation_snapshot_delta(&before, &after);

        assert_eq!(delta.mode, "before_after_snapshot_delta");
        assert_eq!(delta.source_id, 7);
        assert_eq!(delta.item_count_delta, 3);
        assert_eq!(delta.telegram_typed_row_count_delta, 3);
        assert_eq!(delta.content_zstd_present_count_delta, 2);
        assert_eq!(delta.topic_membership_count_delta, 2);
        assert_eq!(delta.max_telegram_message_id_before, Some(1002));
        assert_eq!(delta.max_telegram_message_id_after, Some(1005));
        assert_eq!(delta.last_sync_state_before, Some(10));
        assert_eq!(delta.last_sync_state_after, Some(12));
    }

    #[tokio::test]
    async fn takeout_validation_warning_visibility_is_durable_only() {
        let pool = fixture_pool().await;
        let batch_id = create_running_batch(&pool).await;
        mark_takeout_only_my_messages_fallback(&pool, batch_id, SENTINEL_WARNING)
            .await
            .expect("mark fallback");
        mark_takeout_migrated_history_deferred(&pool, batch_id, SENTINEL_WARNING)
            .await
            .expect("mark migrated deferred");

        let visibility = takeout_validation_warning_visibility(&pool, batch_id)
            .await
            .expect("warning visibility")
            .expect("visibility exists");

        assert_eq!(visibility.batch_id, batch_id);
        assert_eq!(visibility.source_id, 7);
        assert_eq!(
            visibility.provenance_warning_codes,
            vec![
                "migrated_history_deferred".to_string(),
                "only_my_messages_fallback".to_string(),
            ]
        );
        assert!(visibility.batch_is_latest_for_source);
        assert_eq!(
            visibility.durable_recovery_kind.as_deref(),
            Some("interrupted")
        );
        assert_eq!(
            visibility.recovery_candidate_warning_codes,
            visibility.provenance_warning_codes
        );
        assert_no_sentinel_json(&visibility);
    }

    #[tokio::test]
    async fn takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates() {
        let pool = fixture_pool().await;
        let old_batch_id = create_running_batch(&pool).await;
        mark_takeout_only_my_messages_fallback(&pool, old_batch_id, SENTINEL_WARNING)
            .await
            .expect("mark old fallback");
        mark_takeout_migrated_history_deferred(&pool, old_batch_id, SENTINEL_WARNING)
            .await
            .expect("mark old migrated deferred");
        let _new_batch_id = create_running_batch(&pool).await;

        let visibility = takeout_validation_warning_visibility(&pool, old_batch_id)
            .await
            .expect("warning visibility")
            .expect("visibility exists");

        assert_eq!(visibility.batch_id, old_batch_id);
        assert_eq!(visibility.source_id, 7);
        assert_eq!(
            visibility.provenance_warning_codes,
            vec![
                "migrated_history_deferred".to_string(),
                "only_my_messages_fallback".to_string(),
            ]
        );
        assert!(!visibility.batch_is_latest_for_source);
        assert_eq!(visibility.durable_recovery_kind, None);
        assert!(visibility.recovery_candidate_warning_codes.is_empty());
        assert_no_sentinel_json(&visibility);
    }
}
