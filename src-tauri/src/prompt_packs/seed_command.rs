use tauri::AppHandle;

use super::seed::seed_builtin_prompt_packs_in_pool;
use crate::db::get_pool;
use crate::error::AppResult;

pub async fn seed_builtin_prompt_packs(handle: AppHandle) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    seed_builtin_prompt_packs_in_pool(&pool).await
}
