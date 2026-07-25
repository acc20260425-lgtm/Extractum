use super::super::fetch_prompt_template;
use extractum_core::error::AppErrorKind;

async fn template_store_pool() -> sqlx::SqlitePool {
    super::super::super::test_schema::analysis_test_pool().await
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
