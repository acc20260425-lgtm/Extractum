use crate::error::{AppError, AppResult};
use crate::gemini_browser::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserRunResult, GeminiBrowserRunStatus,
};

pub(crate) fn browser_result_to_completion_text(
    result: GeminiBrowserRunResult,
) -> AppResult<String> {
    match result.status {
        GeminiBrowserRunStatus::Ok => {
            if result.debug_summary.as_ref().is_some_and(|summary| {
                summary.answer_completion_reason
                    == GeminiBrowserAnswerCompletionReason::TimeoutLatest
            }) {
                return Err(AppError::validation(
                    "Gemini browser result is partial-risk (timeout_latest) and cannot be used as a prompt completion",
                ));
            }
            result
                .text
                .filter(|text| !text.trim().is_empty())
                .ok_or_else(|| AppError::internal("Gemini browser result did not include text"))
        }
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
    use crate::error::AppErrorKind;
    use crate::gemini_browser::{
        GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderMode,
        GeminiBrowserRunDebugSummary, GeminiBrowserRunResult, GeminiBrowserRunStatus,
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

    #[test]
    fn timeout_latest_ok_result_is_not_prompt_completion() {
        let mut result = result(GeminiBrowserRunStatus::Ok, Some("partial answer"));
        result.debug_summary = Some(GeminiBrowserRunDebugSummary {
            mode: GeminiBrowserProviderMode::CdpAttach,
            composer_found: true,
            send_button_found: true,
            generation_busy_observed: false,
            answer_found: true,
            answer_selector: Some("message-content".to_string()),
            waited_for_send_ms: 0,
            waited_for_answer_ms: 120_000,
            answer_stable_ms: 8_000,
            answer_completion_reason: GeminiBrowserAnswerCompletionReason::TimeoutLatest,
            final_text_length: 14,
            error_stage: None,
            extraction: None,
        });

        let error =
            browser_result_to_completion_text(result).expect_err("partial-risk must not complete");
        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.message.contains("partial"));
        assert!(error.message.contains("timeout_latest"));
    }
}
