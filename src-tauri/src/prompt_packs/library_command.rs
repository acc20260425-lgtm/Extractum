use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::AppResult;
use extractum_prompt_packs::{get_prompt_pack_library_in_pool, PromptPackLibraryDto};

#[tauri::command]
pub async fn get_prompt_pack_library(handle: AppHandle) -> AppResult<PromptPackLibraryDto> {
    let pool = get_pool(&handle).await?;
    get_prompt_pack_library_in_pool(&pool).await
}
