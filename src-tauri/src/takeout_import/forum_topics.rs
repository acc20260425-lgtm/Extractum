use grammers_client::Client;
use grammers_session::types::PeerRef;

use crate::error::AppResult;
use crate::ingest_provenance::{record_ingest_batch_warning, TerminalBatchStatus};
use crate::sources::{refresh_forum_topics, SourceSyncTarget, TELEGRAM_KIND_SUPERGROUP};

pub(crate) const FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE: &str = "forum_topic_refresh_failed";
const FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING: &str =
    "Forum topic refresh after Takeout failed; existing topic catalog remains available.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeoutForumTopicRefreshPolicy {
    Refresh,
    Skip,
}

pub(crate) fn completed_takeout_forum_topic_refresh_policy(
    terminal_status: TerminalBatchStatus,
    source_subtype: &str,
) -> TakeoutForumTopicRefreshPolicy {
    match (terminal_status, source_subtype) {
        (TerminalBatchStatus::Completed, TELEGRAM_KIND_SUPERGROUP) => {
            TakeoutForumTopicRefreshPolicy::Refresh
        }
        _ => TakeoutForumTopicRefreshPolicy::Skip,
    }
}

pub(crate) async fn refresh_forum_topics_after_completed_takeout(
    pool: &sqlx::SqlitePool,
    batch_id: i64,
    client: &Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
    source_subtype: &str,
    warnings: &mut Vec<String>,
) -> AppResult<()> {
    if completed_takeout_forum_topic_refresh_policy(TerminalBatchStatus::Completed, source_subtype)
        == TakeoutForumTopicRefreshPolicy::Skip
    {
        return Ok(());
    }

    let refresh_warnings = refresh_forum_topics(pool, client, peer, source).await;
    record_takeout_forum_topic_refresh_failure_if_needed(
        pool,
        batch_id,
        warnings,
        &refresh_warnings,
    )
    .await
}

pub(crate) async fn record_takeout_forum_topic_refresh_failure_if_needed(
    pool: &sqlx::SqlitePool,
    batch_id: i64,
    warnings: &mut Vec<String>,
    refresh_warnings: &[String],
) -> AppResult<()> {
    if refresh_warnings.is_empty() {
        return Ok(());
    }

    if !warnings
        .iter()
        .any(|warning| warning == FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING)
    {
        warnings.push(FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING.to_string());
    }
    record_ingest_batch_warning(
        pool,
        batch_id,
        FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE,
        FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{
        completed_takeout_forum_topic_refresh_policy,
        record_takeout_forum_topic_refresh_failure_if_needed, TakeoutForumTopicRefreshPolicy,
        FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING, FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE,
    };
    use crate::ingest_provenance::{
        create_telegram_takeout_batch, finalize_ingest_batch, CreateTelegramTakeoutBatch,
        TerminalBatchStatus,
    };
    use crate::sources::test_support::{
        create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
    };
    use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};

    #[test]
    fn completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups() {
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Completed,
                TELEGRAM_KIND_SUPERGROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Refresh,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Completed,
                TELEGRAM_KIND_CHANNEL,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Completed,
                TELEGRAM_KIND_GROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Failed,
                TELEGRAM_KIND_SUPERGROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
        assert_eq!(
            completed_takeout_forum_topic_refresh_policy(
                TerminalBatchStatus::Cancelled,
                TELEGRAM_KIND_SUPERGROUP,
            ),
            TakeoutForumTopicRefreshPolicy::Skip,
        );
    }

    #[tokio::test]
    async fn takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let mut warnings = Vec::new();

        record_takeout_forum_topic_refresh_failure_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &[
                "Forum topic refresh failed for source 1: network".to_string(),
                "Forum topic refresh failed for source 1: retry".to_string(),
            ],
        )
        .await
        .expect("record refresh warning");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize completed batch");

        assert_eq!(
            warnings,
            vec![FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING.to_string()]
        );
        let warning_rows: Vec<(String, String)> =
            sqlx::query_as("SELECT code, message FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_all(&pool)
                .await
                .expect("load warning rows");
        assert_eq!(
            warning_rows,
            vec![(
                FORUM_TOPIC_REFRESH_FAILED_WARNING_CODE.to_string(),
                FORUM_TOPIC_REFRESH_FAILED_JOB_WARNING.to_string(),
            )],
        );
        let warning_count: i64 =
            sqlx::query_scalar("SELECT warning_count FROM ingest_batches WHERE id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load warning count");
        assert_eq!(warning_count, 1);
    }

    #[tokio::test]
    async fn takeout_forum_topic_refresh_success_records_no_warning() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let mut warnings = vec!["existing warning".to_string()];

        record_takeout_forum_topic_refresh_failure_if_needed(&pool, batch_id, &mut warnings, &[])
            .await
            .expect("record no refresh warning");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize completed batch");

        assert_eq!(warnings, vec!["existing warning".to_string()]);
        let warning_count: i64 =
            sqlx::query_scalar("SELECT warning_count FROM ingest_batches WHERE id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load warning count");
        assert_eq!(warning_count, 0);
    }

    async fn seed_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
    }
}
