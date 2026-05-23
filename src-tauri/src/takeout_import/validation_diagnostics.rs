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
    const SENTINEL_USERNAME: &str = "sentinel_private_username_do_not_emit";
    const SENTINEL_EXTERNAL_ID: &str = "sentinel_external_id_do_not_emit";
    const SENTINEL_MESSAGE: &str = "sentinel_message_text_do_not_emit";
    const SENTINEL_METADATA: &str = "sentinel_raw_metadata_do_not_emit";
    const SENTINEL_WARNING: &str = "sentinel_warning_body_do_not_emit";

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
            SENTINEL_USERNAME,
            SENTINEL_EXTERNAL_ID,
            SENTINEL_MESSAGE,
            SENTINEL_METADATA,
            SENTINEL_WARNING,
        ] {
            assert!(
                !json.contains(sentinel),
                "diagnostic output leaked sentinel {sentinel}: {json}"
            );
        }
    }

    #[tokio::test]
    async fn takeout_validation_source_snapshot_is_aggregate_and_sanitized() {
        let pool = fixture_pool().await;
        seed_canonical_message(&pool, 101, 1001, "text_only", None, Some(77), Some(2)).await;
        seed_canonical_message(&pool, 102, 1002, "media", Some("photo"), None, None).await;

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
        assert_eq!(
            summary.warning_codes,
            vec![
                "export_dc_fallback".to_string(),
                "migrated_history_deferred".to_string(),
                "only_my_messages_fallback".to_string(),
            ]
        );
        assert!(summary.used_export_dc);
        assert!(summary.fallback_used);
        assert!(summary.only_my_messages);
        assert!(summary.migrated_history_detected);
        assert!(!summary.migrated_history_imported);
        assert!(!summary.terminal_error_present);
        assert_no_sentinel_json(&summary);
    }
}
