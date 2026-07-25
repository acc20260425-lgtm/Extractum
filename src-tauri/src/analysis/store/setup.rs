use sqlx::{QueryBuilder, Sqlite, SqliteConnection};

use super::super::groups::{
    load_analysis_source_group_for_enrichment, load_analysis_source_groups_for_enrichment,
    AnalysisSourceGroupRecord,
};
use super::super::models::{AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceKind};

include!("owned_setup.rs");

pub(crate) async fn ensure_sources_exist(pool: &SqlitePool, source_ids: &[i64]) -> AppResult<()> {
    for source_id in source_ids {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)")
                .bind(source_id)
                .fetch_one(pool)
                .await
                .map_err(AppError::database)?;

        if exists == 0 {
            return Err(AppError::not_found(format!("Source {source_id} not found")));
        }
    }

    Ok(())
}

async fn enrich_analysis_source_group(
    conn: &mut SqliteConnection,
    record: AnalysisSourceGroupRecord,
) -> AppResult<AnalysisSourceGroup> {
    let mut members = Vec::new();
    if !record.member_source_ids().is_empty() {
        let mut query = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
                sources.id AS source_id,
                sources.title AS source_title,
                COUNT(items.content_zstd) AS item_count
            FROM sources
            LEFT JOIN items ON items.source_id = sources.id
            WHERE sources.id IN (
            "#,
        );
        {
            let mut separated = query.separated(", ");
            for source_id in record.member_source_ids() {
                separated.push_bind(source_id);
            }
        }
        query.push(
            r#"
            )
            GROUP BY sources.id, sources.title
            ORDER BY COALESCE(sources.title, ''), sources.id
            "#,
        );
        members = query
            .build_query_as::<AnalysisSourceGroupMember>()
            .fetch_all(&mut *conn)
            .await
            .map_err(AppError::database)?;
    }

    let source_type = match record.source_kind() {
        AnalysisSourceKind::Telegram => "telegram",
        AnalysisSourceKind::Youtube => "youtube",
    }
    .to_string();
    Ok(AnalysisSourceGroup {
        id: record.id(),
        name: record.name().to_string(),
        source_type,
        members,
        created_at: record.created_at(),
        updated_at: record.updated_at(),
    })
}

pub(crate) async fn list_analysis_source_groups_in_pool(
    pool: &SqlitePool,
) -> AppResult<Vec<AnalysisSourceGroup>> {
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let records = load_analysis_source_groups_for_enrichment(&mut *transaction).await?;
    let mut groups = Vec::with_capacity(records.len());
    for record in records {
        groups.push(enrich_analysis_source_group(&mut *transaction, record).await?);
    }
    transaction.commit().await.map_err(AppError::database)?;
    Ok(groups)
}

pub(crate) async fn get_analysis_source_group_response_in_pool(
    pool: &SqlitePool,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroup>> {
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let record =
        load_analysis_source_group_for_enrichment(&mut *transaction, group_id).await?;
    let group = match record {
        Some(record) => Some(enrich_analysis_source_group(&mut *transaction, record).await?),
        None => None,
    };
    transaction.commit().await.map_err(AppError::database)?;
    Ok(group)
}
