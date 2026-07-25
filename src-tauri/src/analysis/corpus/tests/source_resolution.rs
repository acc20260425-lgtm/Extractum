use super::harness::{create_project_scope_schema, snapshot_pool};
use crate::analysis::corpus::{resolve_analysis_sources, AnalysisSourceResolutionErrorCode};
use crate::migrations::apply_all_migrations_for_test_pool;
use extractum_analysis::AnalysisSourceKind;

use super::super::source_resolution::resolve_run_source_ids;

#[tokio::test]
async fn resolve_run_source_ids_loads_project_sources_without_snapshot() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    sqlx::query(
        r#"
        INSERT INTO projects (id, name, created_at, updated_at)
        VALUES (9, 'Live project', 1, 1)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert project");
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, external_id, title,
            is_active, is_member, created_at
        )
        VALUES
            (2, 'youtube', 'video', 'video-2', 'Source 2', 1, 0, 1),
            (4, 'youtube', 'video', 'video-4', 'Source 4', 1, 0, 1)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert sources");
    sqlx::query(
        r#"
        INSERT INTO project_sources (project_id, source_id, added_at)
        VALUES (9, 2, 1), (9, 4, 2)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert project sources");

    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id, run_type, scope_type, project_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, youtube_corpus_mode, status, created_at
        )
        VALUES (
            1, 'report', 'project', 9, 1700000000, 1800000000,
            'English', 1, 'default', 'gemini',
            'gemini-2.5-flash', 'transcript_description', 'completed', 1710000500
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert project analysis run");
    let run = crate::analysis::store::get_analysis_run_in_pool(&pool, 1)
        .await
        .expect("load project analysis run")
        .expect("project analysis run exists");

    let source_ids = resolve_run_source_ids(&pool, &run)
        .await
        .expect("resolve project source ids");

    assert_eq!(source_ids, vec![2, 4]);
}

#[tokio::test]
async fn playlist_expansion_excludes_unlinked_and_removed_rows() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (10, 'youtube', 'playlist', 'PLdemo', 'Playlist'),
                (20, 'youtube', 'video', 'video1', 'Video 1'),
                (21, 'youtube', 'video', 'video2', 'Video 2')",
    )
    .execute(&pool)
    .await
    .expect("insert sources");
    sqlx::query(
        "INSERT INTO youtube_playlist_items (
            playlist_source_id, video_id, video_source_id, position, availability_status, is_removed_from_playlist
         )
         VALUES (10, 'video1', 20, 1, 'available', 0),
                (10, 'missing', NULL, 2, 'unavailable_unknown', 0),
                (10, 'removed', 21, 3, 'removed_from_playlist', 1)",
    )
    .execute(&pool)
    .await
    .expect("insert playlist rows");

    let resolved = resolve_analysis_sources(&pool, Some(10), None, None)
        .await
        .expect("resolve playlist scope");

    assert_eq!(resolved.scope().source_kind(), AnalysisSourceKind::Youtube);
    assert_eq!(resolved.scope().source_ids(), &[20]);
    assert_eq!(resolved.skipped_unlinked_playlist_items(), 1);
}

#[tokio::test]
async fn resolve_analysis_sources_rejects_mixed_provider_project() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    create_project_scope_schema(&pool).await;
    sqlx::query(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES (9, 'Mixed', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert project");
    sqlx::query("INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (1, 'youtube', 'video', 'v1', 'Video', 1, 0, 1), (2, 'telegram', 'supergroup', 'tg2', 'Telegram', 1, 0, 1)")
        .execute(&pool)
        .await
        .expect("insert sources");
    sqlx::query(
        "INSERT INTO project_sources (project_id, source_id, added_at) VALUES (9, 1, 1), (9, 2, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert project sources");

    let error = resolve_analysis_sources(&pool, None, None, Some(9))
        .await
        .expect_err("mixed project rejected");
    assert_eq!(
        error.code(),
        Some(AnalysisSourceResolutionErrorCode::MixedProviderProject)
    );
}

#[tokio::test]
async fn resolve_analysis_sources_preserves_no_linked_youtube_error_message() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    create_project_scope_schema(&pool).await;
    sqlx::query(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES (9, 'Playlist', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert project");
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (1, 'youtube', 'playlist', 'pl1', 'Playlist', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert playlist source");
    sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (9, 1, 1)")
        .execute(&pool)
        .await
        .expect("insert project source");

    let error = resolve_analysis_sources(&pool, None, None, Some(9))
        .await
        .expect_err("unmaterialized playlist rejected");

    assert_eq!(
        error.code(),
        Some(AnalysisSourceResolutionErrorCode::NoLinkedYoutubeVideos)
    );
    let error = error.into_app_error();
    assert_eq!(
        error.message,
        "No linked YouTube videos are available for analysis in this scope"
    );
}

#[tokio::test]
async fn resolve_analysis_sources_loads_single_provider_project() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    create_project_scope_schema(&pool).await;
    sqlx::query(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES (9, 'YouTube', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert project");
    sqlx::query("INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (1, 'youtube', 'video', 'v1', 'Video 1', 1, 0, 1), (2, 'youtube', 'video', 'v2', 'Video 2', 1, 0, 1)")
        .execute(&pool)
        .await
        .expect("insert sources");
    sqlx::query(
        "INSERT INTO project_sources (project_id, source_id, added_at) VALUES (9, 1, 1), (9, 2, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert project sources");

    let resolved = resolve_analysis_sources(&pool, None, None, Some(9))
        .await
        .expect("resolve project");
    assert_eq!(resolved.scope().source_kind(), AnalysisSourceKind::Youtube);
    assert_eq!(resolved.scope().source_ids(), &[1, 2]);
}
