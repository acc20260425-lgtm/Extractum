use super::models::AnalysisChatTurn;
use extractum_core::error::{AppError, AppResult};

pub(crate) use extractum_core::time::now_secs;

pub(crate) const TEMPLATE_KIND_REPORT: &str = "report";
pub(crate) const TEMPLATE_KIND_CHAT: &str = "chat";
pub(crate) const DEFAULT_REPORT_TEMPLATE_NAME: &str = "Default report";
pub(crate) const ANALYSIS_RUN_TYPE_REPORT: &str = "report";
pub(crate) const ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE: &str = "single_source";
pub(crate) const ANALYSIS_SCOPE_TYPE_SOURCE_GROUP: &str = "source_group";
pub(crate) const ANALYSIS_SCOPE_TYPE_PROJECT: &str = "project";
pub(crate) const ANALYSIS_STATUS_QUEUED: &str = "queued";
pub(crate) const ANALYSIS_STATUS_RUNNING: &str = "running";
pub(crate) const ANALYSIS_STATUS_COMPLETED: &str = "completed";
pub(crate) const ANALYSIS_STATUS_FAILED: &str = "failed";
pub(crate) const ANALYSIS_STATUS_CANCELLED: &str = "cancelled";
pub(crate) const ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS: usize = 16_000;

pub(crate) fn validate_chat_turns(history: &[AnalysisChatTurn]) -> AppResult<()> {
    for turn in history {
        match turn.role.as_str() {
            "user" | "assistant" => {}
            other => {
                return Err(AppError::validation(format!(
                    "Unsupported chat turn role '{other}'"
                )))
            }
        }
        if turn.content.trim().is_empty() {
            return Err(AppError::validation("Chat turns cannot be empty"));
        }
    }

    Ok(())
}

pub(crate) fn validate_chat_role(role: &str) -> AppResult<()> {
    match role {
        "user" | "assistant" => Ok(()),
        other => Err(AppError::validation(format!(
            "Unsupported chat role '{other}'"
        ))),
    }
}

pub(crate) fn default_report_template_body() -> &'static str {
    r#"Create a grounded report over the provided source documents.

Focus on:
- the main topics and recurring themes
- the most notable claims, updates, and shifts
- supporting examples from the source material

Always keep the report concise, readable, and useful for later follow-up analysis."#
}
