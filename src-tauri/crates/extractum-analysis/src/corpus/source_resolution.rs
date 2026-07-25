use extractum_core::error::{AppError, AppResult};

use super::super::models::AnalysisSourceKind;

pub fn resolve_analysis_telegram_history_scope(
    include_migrated_history: bool,
    source_kind: AnalysisSourceKind,
) -> AppResult<(&'static str, bool)> {
    if include_migrated_history && source_kind != AnalysisSourceKind::Telegram {
        return Err(AppError::validation(
            "Migrated historical scope can be included only for Telegram analysis",
        ));
    }
    if include_migrated_history {
        return Ok(("current_plus_migrated", true));
    }
    Ok(("current", false))
}
