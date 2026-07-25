use sqlx::{SqliteConnection, SqlitePool};

use super::domain::now_secs;
use super::models::AnalysisSourceKind;
use extractum_core::error::{AppError, AppResult};

#[derive(Debug)]
pub struct AnalysisSourceGroupInput {
    name: String,
    source_kind: AnalysisSourceKind,
    source_ids: Vec<i64>,
}

impl AnalysisSourceGroupInput {
    pub fn new(
        name: String,
        source_kind: AnalysisSourceKind,
        source_ids: Vec<i64>,
    ) -> AppResult<Self> {
        let (name, source_ids) = normalize_source_group_input(&name, source_ids)?;
        Ok(Self {
            name,
            source_kind,
            source_ids,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_kind(&self) -> AnalysisSourceKind {
        self.source_kind
    }

    pub fn source_ids(&self) -> &[i64] {
        &self.source_ids
    }
}

#[derive(Debug)]
pub struct AnalysisSourceGroupRecord {
    id: i64,
    name: String,
    source_kind: AnalysisSourceKind,
    member_source_ids: Vec<i64>,
    created_at: i64,
    updated_at: i64,
}

impl AnalysisSourceGroupRecord {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_kind(&self) -> AnalysisSourceKind {
        self.source_kind
    }

    pub fn member_source_ids(&self) -> &[i64] {
        &self.member_source_ids
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    pub fn updated_at(&self) -> i64 {
        self.updated_at
    }
}

fn normalize_source_group_input(name: &str, source_ids: Vec<i64>) -> AppResult<(String, Vec<i64>)> {
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

fn analysis_source_kind_from_wire(value: &str) -> AppResult<AnalysisSourceKind> {
    match value {
        "telegram" => Ok(AnalysisSourceKind::Telegram),
        "youtube" => Ok(AnalysisSourceKind::Youtube),
        _ => Err(AppError::validation(
            "Analysis group source_type must be telegram or youtube",
        )),
    }
}

fn analysis_source_kind_as_wire(source_kind: AnalysisSourceKind) -> &'static str {
    match source_kind {
        AnalysisSourceKind::Telegram => "telegram",
        AnalysisSourceKind::Youtube => "youtube",
    }
}

#[derive(sqlx::FromRow)]
struct AnalysisSourceGroupRecordRow {
    id: i64,
    name: String,
    source_type: String,
    created_at: i64,
    updated_at: i64,
}

fn build_analysis_source_group_record(
    row: AnalysisSourceGroupRecordRow,
    member_source_ids: Vec<i64>,
) -> AppResult<AnalysisSourceGroupRecord> {
    Ok(AnalysisSourceGroupRecord {
        id: row.id,
        name: row.name,
        source_kind: analysis_source_kind_from_wire(&row.source_type)?,
        member_source_ids,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn load_analysis_source_group_member_ids(
    conn: &mut SqliteConnection,
    group_id: i64,
) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        r#"
        SELECT source_id
        FROM analysis_source_group_members
        WHERE group_id = ?
        ORDER BY source_id ASC
        "#,
    )
    .bind(group_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)
}

pub async fn load_analysis_source_groups_for_enrichment(
    conn: &mut SqliteConnection,
) -> AppResult<Vec<AnalysisSourceGroupRecord>> {
    let rows = sqlx::query_as::<_, AnalysisSourceGroupRecordRow>(
        r#"
        SELECT id, name, source_type, created_at, updated_at
        FROM analysis_source_groups
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let mut groups = Vec::with_capacity(rows.len());
    for row in rows {
        let member_source_ids = load_analysis_source_group_member_ids(&mut *conn, row.id).await?;
        groups.push(build_analysis_source_group_record(row, member_source_ids)?);
    }
    Ok(groups)
}

pub async fn load_analysis_source_group_for_enrichment(
    conn: &mut SqliteConnection,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroupRecord>> {
    let row = sqlx::query_as::<_, AnalysisSourceGroupRecordRow>(
        r#"
        SELECT id, name, source_type, created_at, updated_at
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(group_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let Some(row) = row else {
        return Ok(None);
    };
    let member_source_ids = load_analysis_source_group_member_ids(&mut *conn, row.id).await?;
    build_analysis_source_group_record(row, member_source_ids).map(Some)
}

pub async fn create_analysis_source_group_in_pool(
    pool: &SqlitePool,
    input: AnalysisSourceGroupInput,
) -> AppResult<i64> {
    let now = now_secs();
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    let group_id = sqlx::query_scalar(
        r#"
        INSERT INTO analysis_source_groups (name, source_type, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(input.name())
    .bind(analysis_source_kind_as_wire(input.source_kind()))
    .bind(now)
    .bind(now)
    .fetch_one(&mut *transaction)
    .await
    .map_err(AppError::database)?;

    for source_id in input.source_ids() {
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(AppError::database)?;
    }

    transaction.commit().await.map_err(AppError::database)?;
    Ok(group_id)
}

pub async fn update_analysis_source_group_in_pool(
    pool: &SqlitePool,
    group_id: i64,
    input: AnalysisSourceGroupInput,
) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM analysis_source_groups WHERE id = ?)",
    )
    .bind(group_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::not_found(format!(
            "Analysis source group {group_id} not found"
        )));
    }

    let now = now_secs();
    let mut transaction = pool.begin().await.map_err(AppError::database)?;
    sqlx::query(
        r#"
        UPDATE analysis_source_groups
        SET name = ?, source_type = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name())
    .bind(analysis_source_kind_as_wire(input.source_kind()))
    .bind(now)
    .bind(group_id)
    .execute(&mut *transaction)
    .await
    .map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_source_group_members WHERE group_id = ?")
        .bind(group_id)
        .execute(&mut *transaction)
        .await
        .map_err(AppError::database)?;

    for source_id in input.source_ids() {
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(AppError::database)?;
    }

    transaction.commit().await.map_err(AppError::database)?;
    Ok(())
}

pub async fn delete_analysis_source_group_in_pool(
    pool: &SqlitePool,
    group_id: i64,
) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM analysis_source_groups WHERE id = ?")
        .bind(group_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Analysis source group {group_id} not found"
        )));
    }

    Ok(())
}

pub async fn get_analysis_source_group_record(
    pool: &SqlitePool,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroupRecord>> {
    let row = sqlx::query_as::<_, AnalysisSourceGroupRecordRow>(
        r#"
        SELECT id, name, source_type, created_at, updated_at
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let Some(row) = row else {
        return Ok(None);
    };
    let member_source_ids = sqlx::query_scalar(
        r#"
        SELECT source_id
        FROM analysis_source_group_members
        WHERE group_id = ?
        ORDER BY source_id ASC
        "#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    build_analysis_source_group_record(row, member_source_ids).map(Some)
}
