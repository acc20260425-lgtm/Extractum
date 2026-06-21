use crate::error::{AppError, AppResult};
use crate::gemini_browser::{GeminiBrowserRunResult, GeminiBrowserRunStatus};

pub(crate) fn browser_result_to_completion_text(
    result: GeminiBrowserRunResult,
) -> AppResult<String> {
    match result.status {
        GeminiBrowserRunStatus::Ok => result
            .text
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| AppError::internal("Gemini browser result did not include text")),
        GeminiBrowserRunStatus::Ready => Err(AppError::internal(
            "Gemini browser readiness result cannot be used as a prompt completion",
        )),
        status => Err(AppError::internal(format!(
            "Gemini browser prompt failed with status {status:?}: {}",
            result.message.unwrap_or_else(|| "No message".to_string())
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gemini_browser::{
        GeminiBrowserArtifactRefs, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    };

    fn result(status: GeminiBrowserRunStatus, text: Option<&str>) -> GeminiBrowserRunResult {
        GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status,
            text: text.map(ToString::to_string),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 1,
            debug_summary: None,
        }
    }

    #[test]
    fn ok_browser_result_maps_to_completion_text() {
        assert_eq!(
            browser_result_to_completion_text(result(GeminiBrowserRunStatus::Ok, Some("answer")))
                .expect("completion"),
            "answer"
        );
    }

    #[test]
    fn ready_result_is_not_prompt_completion() {
        let error = browser_result_to_completion_text(result(GeminiBrowserRunStatus::Ready, None))
            .expect_err("ready is not completion");
        assert!(error.message.contains("readiness"));
    }
}
