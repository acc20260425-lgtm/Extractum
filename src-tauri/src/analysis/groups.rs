use extractum_analysis::{
    create_analysis_source_group as create_analysis_source_group_in_pool,
    delete_analysis_source_group as delete_analysis_source_group_in_pool,
    update_analysis_source_group as update_analysis_source_group_in_pool, AnalysisSourceGroup,
    AnalysisSourceGroupInput, AnalysisSourceKind,
};
use extractum_core::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tauri::AppHandle;

use crate::db::get_pool;

use super::store::{
    ensure_sources_exist, get_analysis_source_group_response_in_pool,
    list_analysis_source_groups_in_pool,
};

pub(crate) use extractum_analysis::load_analysis_source_group_for_enrichment;

async fn validate_group_source_type(
    pool: &SqlitePool,
    source_kind: AnalysisSourceKind,
    source_ids: &[i64],
) -> AppResult<()> {
    let group_source_type = match source_kind {
        AnalysisSourceKind::Telegram => "telegram",
        AnalysisSourceKind::Youtube => "youtube",
    };
    let mut query =
        QueryBuilder::<Sqlite>::new("SELECT id, source_type FROM sources WHERE id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        source_type: String,
    }

    let rows: Vec<Row> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    if let Some(row) = rows.iter().find(|row| row.source_type != group_source_type) {
        return Err(AppError::validation(format!(
            "Source {} has type '{}' and cannot be added to a '{}' analysis group",
            row.id, row.source_type, group_source_type
        )));
    }

    Ok(())
}

fn parse_analysis_source_kind(source_type: &str) -> AppResult<AnalysisSourceKind> {
    match source_type {
        "telegram" => Ok(AnalysisSourceKind::Telegram),
        "youtube" => Ok(AnalysisSourceKind::Youtube),
        _ => Err(AppError::validation(
            "Analysis group source_type must be telegram or youtube",
        )),
    }
}

async fn prepare_analysis_source_group_input(
    pool: &SqlitePool,
    name: String,
    source_type: String,
    source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroupInput> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::validation("Source group name cannot be empty"));
    }

    let mut source_ids = source_ids
        .into_iter()
        .filter(|source_id| *source_id > 0)
        .collect::<Vec<_>>();
    source_ids.sort_unstable();
    source_ids.dedup();
    if source_ids.is_empty() {
        return Err(AppError::validation(
            "Select at least one source for the group",
        ));
    }

    ensure_sources_exist(pool, &source_ids).await?;
    let source_kind = parse_analysis_source_kind(&source_type)?;
    let input = AnalysisSourceGroupInput::new(name, source_kind, source_ids)?;
    validate_group_source_type(pool, input.source_kind(), input.source_ids()).await?;
    Ok(input)
}

#[tauri::command]
pub async fn list_analysis_source_groups(handle: AppHandle) -> AppResult<Vec<AnalysisSourceGroup>> {
    let pool = get_pool(&handle).await?;
    list_analysis_source_groups_in_pool(&pool).await
}

#[tauri::command]
pub async fn create_analysis_source_group(
    handle: AppHandle,
    name: String,
    source_type: String,
    source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroup> {
    let pool = get_pool(&handle).await?;
    let input = prepare_analysis_source_group_input(&pool, name, source_type, source_ids).await?;
    let group_id = create_analysis_source_group_in_pool(&pool, input).await?;

    let response = get_analysis_source_group_response_in_pool(&pool, group_id).await?;
    response.ok_or_else(|| {
        AppError::not_found(format!(
            "Analysis source group {group_id} not found after creation"
        ))
    })
}

#[tauri::command]
pub async fn update_analysis_source_group(
    handle: AppHandle,
    group_id: i64,
    name: String,
    source_type: String,
    source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroup> {
    let pool = get_pool(&handle).await?;
    let input = prepare_analysis_source_group_input(&pool, name, source_type, source_ids).await?;
    update_analysis_source_group_in_pool(&pool, group_id, input).await?;

    let response = get_analysis_source_group_response_in_pool(&pool, group_id).await?;
    response.ok_or_else(|| {
        AppError::not_found(format!(
            "Analysis source group {group_id} not found after update"
        ))
    })
}

#[tauri::command]
pub async fn delete_analysis_source_group(handle: AppHandle, group_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_analysis_source_group_in_pool(&pool, group_id).await
}

#[cfg(test)]
mod tests {
    use super::{
        parse_analysis_source_kind, prepare_analysis_source_group_input, validate_group_source_type,
    };
    use crate::error::AppErrorKind;
    use extractum_analysis::AnalysisSourceKind;

    async fn source_type_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");
        sqlx::query(
            "INSERT INTO sources (id, source_type)
             VALUES (1, 'telegram'), (2, 'youtube'), (4, 'youtube')",
        )
        .execute(&pool)
        .await
        .expect("insert sources");
        pool
    }

    #[tokio::test]
    async fn prepare_analysis_source_group_input_preserves_baseline_error_precedence() {
        let pool = source_type_pool().await;

        let blank_name = prepare_analysis_source_group_input(
            &pool,
            "   ".to_string(),
            "rss".to_string(),
            vec![2],
        )
        .await
        .expect_err("blank name wins before unsupported source type");
        assert_eq!(blank_name.kind, AppErrorKind::Validation);
        assert_eq!(blank_name.message, "Source group name cannot be empty");

        let no_positive_ids = prepare_analysis_source_group_input(
            &pool,
            "Group".to_string(),
            "rss".to_string(),
            vec![0, -1, -7],
        )
        .await
        .expect_err("empty normalized IDs win before unsupported source type");
        assert_eq!(no_positive_ids.kind, AppErrorKind::Validation);
        assert_eq!(
            no_positive_ids.message,
            "Select at least one source for the group"
        );

        let missing_source = prepare_analysis_source_group_input(
            &pool,
            "Group".to_string(),
            "rss".to_string(),
            vec![7, 7, -1],
        )
        .await
        .expect_err("missing normalized source wins before unsupported source type");
        assert_eq!(missing_source.kind, AppErrorKind::NotFound);
        assert_eq!(missing_source.message, "Source 7 not found");

        let unsupported_type = prepare_analysis_source_group_input(
            &pool,
            "Group".to_string(),
            "rss".to_string(),
            vec![2],
        )
        .await
        .expect_err("unsupported source type wins after an existing source");
        assert_eq!(unsupported_type.kind, AppErrorKind::Validation);
        assert_eq!(
            unsupported_type.message,
            "Analysis group source_type must be telegram or youtube"
        );

        let input = prepare_analysis_source_group_input(
            &pool,
            "  YouTube group  ".to_string(),
            "youtube".to_string(),
            vec![4, 2, 4, -1, 2],
        )
        .await
        .expect("prepare valid source group input");
        assert_eq!(input.name(), "YouTube group");
        assert_eq!(input.source_kind(), AnalysisSourceKind::Youtube);
        assert_eq!(input.source_ids(), &[2, 4]);
    }

    #[tokio::test]
    async fn validate_group_source_type_rejects_unknown_group_type() {
        let error =
            parse_analysis_source_kind("rss").expect_err("reject unsupported group source type");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(
            error.message,
            "Analysis group source_type must be telegram or youtube"
        );
    }

    #[tokio::test]
    async fn validate_group_source_type_rejects_mixed_provider_membership() {
        let pool = source_type_pool().await;

        let error = validate_group_source_type(&pool, AnalysisSourceKind::Youtube, &[1, 2])
            .await
            .expect_err("reject telegram source in youtube group");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.message.contains("Source 1 has type 'telegram'"));
        assert!(error.message.contains("'youtube' analysis group"));
    }

    #[tokio::test]
    async fn validate_group_source_type_accepts_matching_provider_membership() {
        let pool = source_type_pool().await;

        validate_group_source_type(&pool, AnalysisSourceKind::Youtube, &[2])
            .await
            .expect("matching youtube source");
        validate_group_source_type(&pool, AnalysisSourceKind::Telegram, &[1])
            .await
            .expect("matching telegram source");
    }
}
