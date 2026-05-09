use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::{AnalysisSourceGroup, AnalysisSourceGroupRow};
use super::now_secs;
use super::store::{ensure_sources_exist, fetch_source_group};

pub(crate) fn normalize_source_group_input(
    name: &str,
    source_ids: Vec<i64>,
) -> AppResult<(String, Vec<i64>)> {
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

    Ok((name, source_ids))
}

async fn validate_group_source_type(
    pool: &Pool<Sqlite>,
    group_source_type: &str,
    source_ids: &[i64],
) -> AppResult<()> {
    if !matches!(group_source_type, "telegram" | "youtube") {
        return Err(AppError::validation(
            "Analysis group source_type must be telegram or youtube",
        ));
    }

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

#[tauri::command]
pub async fn list_analysis_source_groups(handle: AppHandle) -> AppResult<Vec<AnalysisSourceGroup>> {
    let pool = get_pool(&handle).await?;
    let rows = sqlx::query_as::<_, AnalysisSourceGroupRow>(
        r#"
        SELECT id, name, source_type, created_at, updated_at
        FROM analysis_source_groups
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::database)?;

    let mut groups = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(group) = fetch_source_group(&pool, row.id).await? {
            groups.push(group);
        }
    }

    Ok(groups)
}

#[tauri::command]
pub async fn create_analysis_source_group(
    handle: AppHandle,
    name: String,
    source_type: String,
    source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroup> {
    let pool = get_pool(&handle).await?;
    let (name, source_ids) = normalize_source_group_input(&name, source_ids)?;
    ensure_sources_exist(&pool, &source_ids).await?;
    validate_group_source_type(&pool, &source_type, &source_ids).await?;

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let group_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&source_type)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::database)?;

    for source_id in source_ids {
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    }

    tx.commit().await.map_err(AppError::database)?;

    fetch_source_group(&pool, group_id).await?.ok_or_else(|| {
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
    let (name, source_ids) = normalize_source_group_input(&name, source_ids)?;
    ensure_sources_exist(&pool, &source_ids).await?;
    validate_group_source_type(&pool, &source_type, &source_ids).await?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM analysis_source_groups WHERE id = ?)",
    )
    .bind(group_id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::not_found(format!(
            "Analysis source group {group_id} not found"
        )));
    }

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query(
        r#"
        UPDATE analysis_source_groups
        SET name = ?, source_type = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(&source_type)
    .bind(now)
    .bind(group_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_source_group_members WHERE group_id = ?")
        .bind(group_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    for source_id in source_ids {
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    }

    tx.commit().await.map_err(AppError::database)?;

    fetch_source_group(&pool, group_id).await?.ok_or_else(|| {
        AppError::not_found(format!(
            "Analysis source group {group_id} not found after update"
        ))
    })
}

#[tauri::command]
pub async fn delete_analysis_source_group(handle: AppHandle, group_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let result = sqlx::query("DELETE FROM analysis_source_groups WHERE id = ?")
        .bind(group_id)
        .execute(&pool)
        .await
        .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Analysis source group {group_id} not found"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_group_source_type;
    use crate::error::AppErrorKind;

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
        sqlx::query("INSERT INTO sources (id, source_type) VALUES (1, 'telegram'), (2, 'youtube')")
            .execute(&pool)
            .await
            .expect("insert sources");
        pool
    }

    #[tokio::test]
    async fn validate_group_source_type_rejects_unknown_group_type() {
        let pool = source_type_pool().await;

        let error = validate_group_source_type(&pool, "rss", &[1])
            .await
            .expect_err("reject unsupported group source type");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(
            error.message,
            "Analysis group source_type must be telegram or youtube"
        );
    }

    #[tokio::test]
    async fn validate_group_source_type_rejects_mixed_provider_membership() {
        let pool = source_type_pool().await;

        let error = validate_group_source_type(&pool, "youtube", &[1, 2])
            .await
            .expect_err("reject telegram source in youtube group");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.message.contains("Source 1 has type 'telegram'"));
        assert!(error.message.contains("'youtube' analysis group"));
    }

    #[tokio::test]
    async fn validate_group_source_type_accepts_matching_provider_membership() {
        let pool = source_type_pool().await;

        validate_group_source_type(&pool, "youtube", &[2])
            .await
            .expect("matching youtube source");
        validate_group_source_type(&pool, "telegram", &[1])
            .await
            .expect("matching telegram source");
    }
}
