use sqlx::{QueryBuilder, Sqlite};
use tauri::AppHandle;

use crate::analysis::{
    push_analysis_document_kind_filter, resolve_analysis_sources,
    resolve_analysis_telegram_history_scope, AnalysisSourceResolutionErrorCode, YoutubeCorpusMode,
};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ProjectDataRange {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

fn push_source_ids(query: &mut QueryBuilder<'_, Sqlite>, source_ids: &[i64]) {
    let mut separated = query.separated(", ");
    for source_id in source_ids {
        separated.push_bind(*source_id);
    }
}

pub(crate) async fn get_project_data_range_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<ProjectDataRange> {
    crate::projects::get_project_in_pool(pool, project_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found")))?;

    let youtube_corpus_mode = YoutubeCorpusMode::from_wire(youtube_corpus_mode.as_deref())
        .map_err(AppError::validation)?;

    let source_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_sources WHERE project_id = ?")
            .bind(project_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    if source_count == 0 {
        return Ok(ProjectDataRange {
            from: None,
            to: None,
        });
    }

    if include_migrated_history {
        let non_telegram_source_type: Option<String> = sqlx::query_scalar(
            r#"
            SELECT s.source_type
            FROM project_sources ps
            JOIN sources s ON s.id = ps.source_id
            WHERE ps.project_id = ?
              AND s.source_type <> 'telegram'
            ORDER BY s.id ASC
            LIMIT 1
            "#,
        )
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;
        if let Some(source_type) = non_telegram_source_type {
            resolve_analysis_telegram_history_scope(true, &source_type)?;
        }
    }

    let resolved = match resolve_analysis_sources(pool, None, None, Some(project_id)).await {
        Ok(resolved) => resolved,
        Err(error)
            if error.code() == Some(AnalysisSourceResolutionErrorCode::NoLinkedYoutubeVideos) =>
        {
            return Ok(ProjectDataRange {
                from: None,
                to: None,
            });
        }
        Err(error) => return Err(error.into_app_error()),
    };
    let (_, include_migrated_history) =
        resolve_analysis_telegram_history_scope(include_migrated_history, &resolved.source_type)?;

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT MIN(published_at) AS from_ts, MAX(published_at) AS to_ts
        FROM (
            SELECT d.published_at AS published_at
            FROM analysis_documents d
            WHERE d.source_id IN (
        "#,
    );
    push_source_ids(&mut query, &resolved.source_ids);
    query.push(")");
    push_analysis_document_kind_filter(
        &mut query,
        resolved.source_type.as_str(),
        youtube_corpus_mode,
        "d",
    )?;

    if include_migrated_history {
        query.push(
            r#"
            UNION ALL
            SELECT items.published_at AS published_at
            FROM items
            JOIN sources ON sources.id = items.source_id
            JOIN telegram_messages tm ON tm.item_id = items.id
            WHERE items.source_id IN (
            "#,
        );
        push_source_ids(&mut query, &resolved.source_ids);
        query.push(
            r#")
              AND sources.source_type = 'telegram'
              AND items.item_kind = 'telegram_message'
              AND tm.is_migrated_history = 1
              AND tm.migration_domain = 'migrated_from_chat'
              AND items.content_zstd IS NOT NULL
              AND items.content_kind IN ('text_only', 'text_with_media')
            "#,
        );
    }
    query.push(")");

    let row: (Option<i64>, Option<i64>) = query
        .build_query_as()
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    Ok(ProjectDataRange {
        from: row.0,
        to: row.1,
    })
}

#[tauri::command]
pub async fn get_project_data_range(
    handle: AppHandle,
    project_id: i64,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<ProjectDataRange> {
    let pool = get_pool(&handle).await?;
    get_project_data_range_in_pool(
        &pool,
        project_id,
        youtube_corpus_mode,
        include_migrated_history,
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

    async fn seed_project(pool: &sqlx::SqlitePool, project_id: i64) {
        sqlx::query(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, NULL, 1, 1)",
        )
        .bind(project_id)
        .bind(format!("Project {project_id}"))
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
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at, account_id) VALUES (?, ?, ?, ?, ?, 1, 0, 1, ?)",
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

    async fn attach(pool: &sqlx::SqlitePool, project_id: i64, source_id: i64) {
        sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (?, ?, 1)")
            .bind(project_id)
            .bind(source_id)
            .execute(pool)
            .await
            .expect("attach source");
    }

    async fn seed_document(
        pool: &sqlx::SqlitePool,
        id: i64,
        source_id: i64,
        source_type: &str,
        source_subtype: &str,
        document_kind: &str,
        published_at: i64,
    ) {
        let item_id = match document_kind {
            "telegram_message" | "youtube_transcript" | "youtube_comment" => Some(id + 10_000),
            "youtube_description" => None,
            other => panic!("unsupported test document kind {other}"),
        };
        let document_key = match item_id {
            Some(item_id) => format!("item:{item_id}"),
            None => "youtube:description".to_string(),
        };
        if let Some(item_id) = item_id {
            sqlx::query(
                "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (?, ?, ?, 'Author', ?, ?, x'01', ?)",
            )
            .bind(item_id)
            .bind(source_id)
            .bind(format!("item-{id}"))
            .bind(published_at)
            .bind(published_at + 1)
            .bind(document_kind)
            .execute(pool)
            .await
            .expect("seed backing item");
        }

        sqlx::query(
            r#"
            INSERT INTO analysis_documents (
                id, source_id, item_id, document_key, document_kind, source_type, source_subtype,
                external_id, author, published_at, document_order, ref, content_zstd, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'Author', ?, 0, ?, x'01', ?, ?)
            "#,
        )
        .bind(id)
        .bind(source_id)
        .bind(item_id)
        .bind(document_key)
        .bind(document_kind)
        .bind(source_type)
        .bind(source_subtype)
        .bind(format!("external-{id}"))
        .bind(published_at)
        .bind(format!("ref-{id}"))
        .bind(published_at)
        .bind(published_at)
        .execute(pool)
        .await
        .expect("seed document");
    }

    async fn seed_migrated_telegram_item(
        pool: &sqlx::SqlitePool,
        item_id: i64,
        source_id: i64,
        published_at: i64,
    ) {
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_zstd, item_kind) VALUES (?, ?, ?, 'Migrated Author', ?, ?, x'01', 'telegram_message')",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(format!("migrated-{item_id}"))
        .bind(published_at)
        .bind(published_at + 1)
        .execute(pool)
        .await
        .expect("seed migrated item");

        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id, migration_domain, is_migrated_history) VALUES (?, ?, 'chat', 777, ?, 'migrated_from_chat', 1)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(item_id)
        .execute(pool)
        .await
        .expect("seed migrated telegram metadata");
    }

    #[tokio::test]
    async fn project_data_range_returns_nulls_for_empty_project() {
        let pool = pool().await;
        seed_project(&pool, 4).await;

        let range = get_project_data_range_in_pool(&pool, 4, None, false)
            .await
            .expect("empty project range");

        assert_eq!(
            range,
            ProjectDataRange {
                from: None,
                to: None
            }
        );
    }

    #[tokio::test]
    async fn project_data_range_uses_youtube_mode_document_kinds() {
        let pool = pool().await;
        seed_project(&pool, 1).await;
        seed_source(&pool, 10, "youtube", "video").await;
        attach(&pool, 1, 10).await;
        seed_document(&pool, 1, 10, "youtube", "video", "youtube_transcript", 100).await;
        seed_document(&pool, 2, 10, "youtube", "video", "youtube_description", 50).await;
        seed_document(&pool, 3, 10, "youtube", "video", "youtube_comment", 200).await;

        let transcript_only =
            get_project_data_range_in_pool(&pool, 1, Some("transcript_only".to_string()), false)
                .await
                .expect("range transcript only");
        assert_eq!(
            transcript_only,
            ProjectDataRange {
                from: Some(100),
                to: Some(100)
            }
        );

        let all_text = get_project_data_range_in_pool(
            &pool,
            1,
            Some("transcript_description_comments".to_string()),
            false,
        )
        .await
        .expect("range all text");
        assert_eq!(
            all_text,
            ProjectDataRange {
                from: Some(50),
                to: Some(200)
            }
        );
    }

    #[tokio::test]
    async fn project_data_range_includes_telegram_migrated_history_when_requested() {
        let pool = pool().await;
        seed_project(&pool, 8).await;
        seed_source(&pool, 80, "telegram", "supergroup").await;
        attach(&pool, 8, 80).await;
        seed_document(
            &pool,
            8,
            80,
            "telegram",
            "supergroup",
            "telegram_message",
            100,
        )
        .await;
        seed_migrated_telegram_item(&pool, 80_001, 80, 10).await;

        let current_only = get_project_data_range_in_pool(&pool, 8, None, false)
            .await
            .expect("current telegram range");
        assert_eq!(
            current_only,
            ProjectDataRange {
                from: Some(100),
                to: Some(100)
            }
        );

        let with_migrated = get_project_data_range_in_pool(&pool, 8, None, true)
            .await
            .expect("telegram migrated range");
        assert_eq!(
            with_migrated,
            ProjectDataRange {
                from: Some(10),
                to: Some(100)
            }
        );
    }

    #[tokio::test]
    async fn project_data_range_expands_playlist_to_linked_video_sources() {
        let pool = pool().await;
        seed_project(&pool, 2).await;
        seed_source(&pool, 20, "youtube", "playlist").await;
        seed_source(&pool, 21, "youtube", "video").await;
        attach(&pool, 2, 20).await;
        sqlx::query(
            "INSERT INTO youtube_playlist_items (playlist_source_id, video_source_id, video_id, position, availability_status, is_removed_from_playlist) VALUES (20, 21, 'video-21', 1, 'available', 0)",
        )
        .execute(&pool)
        .await
        .expect("link playlist item");
        seed_document(&pool, 4, 21, "youtube", "video", "youtube_transcript", 777).await;

        let range = get_project_data_range_in_pool(&pool, 2, None, false)
            .await
            .expect("playlist range");

        assert_eq!(
            range,
            ProjectDataRange {
                from: Some(777),
                to: Some(777)
            }
        );
    }

    #[tokio::test]
    async fn project_data_range_returns_nulls_for_unmaterialized_playlist_project() {
        let pool = pool().await;
        seed_project(&pool, 5).await;
        seed_source(&pool, 50, "youtube", "playlist").await;
        attach(&pool, 5, 50).await;

        let range = get_project_data_range_in_pool(&pool, 5, None, false)
            .await
            .expect("unmaterialized playlist range");

        assert_eq!(
            range,
            ProjectDataRange {
                from: None,
                to: None
            }
        );
    }

    #[tokio::test]
    async fn project_data_range_rejects_mixed_provider_project() {
        let pool = pool().await;
        seed_project(&pool, 7).await;
        seed_source(&pool, 70, "youtube", "video").await;
        seed_source(&pool, 71, "telegram", "supergroup").await;
        attach(&pool, 7, 70).await;
        attach(&pool, 7, 71).await;

        let error = get_project_data_range_in_pool(&pool, 7, None, false)
            .await
            .expect_err("mixed provider project rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert_eq!(
            error.message,
            AnalysisSourceResolutionErrorCode::MixedProviderProject.message()
        );
    }

    #[tokio::test]
    async fn project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project() {
        let pool = pool().await;
        seed_project(&pool, 6).await;
        seed_source(&pool, 60, "youtube", "playlist").await;
        attach(&pool, 6, 60).await;

        let error = get_project_data_range_in_pool(&pool, 6, None, true)
            .await
            .expect_err("unmaterialized playlist migrated history rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("Migrated historical scope"));
    }

    #[tokio::test]
    async fn project_data_range_rejects_migrated_history_for_non_telegram() {
        let pool = pool().await;
        seed_project(&pool, 3).await;
        seed_source(&pool, 30, "youtube", "video").await;
        attach(&pool, 3, 30).await;

        let error = get_project_data_range_in_pool(&pool, 3, None, true)
            .await
            .expect_err("non-telegram migrated history rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("Migrated historical scope"));
    }
}
