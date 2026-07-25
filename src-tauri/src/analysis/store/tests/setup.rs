use super::super::ensure_sources_exist;
use crate::error::AppErrorKind as SourceStoreErrorKind;

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

    assert_eq!(error.kind, SourceStoreErrorKind::NotFound);
    assert_eq!(error.message, "Source 7 not found");
}
