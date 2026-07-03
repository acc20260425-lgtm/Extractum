use super::super::{ensure_sources_exist, fetch_prompt_template};
use crate::error::AppErrorKind;

async fn template_store_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        r#"
            CREATE TABLE analysis_prompt_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                template_kind TEXT NOT NULL,
                body TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                is_builtin BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create templates");
    pool
}

async fn source_store_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query("CREATE TABLE sources (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .expect("create sources");
    pool
}

#[tokio::test]
async fn ensure_sources_exist_returns_typed_not_found_error() {
    let pool = source_store_pool().await;

    let error = ensure_sources_exist(&pool, &[7])
        .await
        .expect_err("missing source should fail");

    assert_eq!(error.kind, AppErrorKind::NotFound);
    assert_eq!(error.message, "Source 7 not found");
}

#[tokio::test]
async fn fetch_prompt_template_returns_typed_not_found_error() {
    let pool = template_store_pool().await;

    let error = match fetch_prompt_template(&pool, 99).await {
        Ok(_) => panic!("missing prompt template should fail"),
        Err(error) => error,
    };

    assert_eq!(error.kind, AppErrorKind::NotFound);
    assert_eq!(error.message, "Analysis prompt template 99 not found");
}
