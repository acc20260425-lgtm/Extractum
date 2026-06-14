use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

pub(crate) async fn require_prompt_pack_version_id(
    pool: &SqlitePool,
    pack_id: &str,
    pack_version: &str,
) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_versions WHERE pack_id = ? AND pack_version = ?",
    )
    .bind(pack_id)
    .bind(pack_version)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Prompt pack {pack_id}@{pack_version} not found")))
}
