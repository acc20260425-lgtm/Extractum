use super::GemAnalysisPart;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartOutput {
    pub(crate) part: GemAnalysisPart,
    pub(crate) markdown: String,
}

pub(crate) fn parse_gem_analysis_part_output(
    raw: &str,
    expected_part: GemAnalysisPart,
) -> AppResult<GemAnalysisPartOutput> {
    let value = crate::prompt_packs::stage_io::extract_json_payload(raw)?;
    let part = value
        .get("part")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| AppError::validation("Gem analysis part output is missing part"))?;
    if part != expected_part.as_str() {
        return Err(AppError::validation(format!(
            "Gem analysis part output expected part {} but got {part}",
            expected_part.as_str()
        )));
    }
    let markdown = value
        .get("markdown")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|markdown| !markdown.is_empty())
        .ok_or_else(|| AppError::validation("Gem analysis part output markdown is empty"))?
        .to_string();
    Ok(GemAnalysisPartOutput {
        part: expected_part,
        markdown,
    })
}

#[cfg(test)]
mod gem_analysis_part_tests {
    use super::parse_gem_analysis_part_output;
    use crate::prompt_packs::youtube_summary::{
        GemAnalysisInputBudget, GemAnalysisPart, GemAnalysisPartRepairRequest,
        GemAnalysisPartStageExecutionRequest, YoutubeSummaryStageExecutionRequest,
    };

    #[test]
    fn gem_analysis_part_types_cover_comments_and_stage_variants() {
        let budget = GemAnalysisInputBudget {
            max_input_tokens: 24_000,
        };
        assert_eq!(budget.max_input_tokens, 24_000);
        assert_eq!(GemAnalysisPart::Comments.as_str(), "comments");
        assert_eq!(GemAnalysisPart::Comments.slug(), "comments");

        let part_request = GemAnalysisPartStageExecutionRequest {
            run_id: 1,
            stage_run_id: 2,
            source_snapshot_id: 3,
            source_ref_id: "source_ref_1".to_string(),
            part: GemAnalysisPart::Comments,
            prompt_input_json: "{}".to_string(),
        };
        let stage_request =
            YoutubeSummaryStageExecutionRequest::GemAnalysisPart(part_request.clone());
        assert!(matches!(
            stage_request,
            YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request)
                if request.part == GemAnalysisPart::Comments
        ));

        let repair_request = GemAnalysisPartRepairRequest {
            run_id: 1,
            stage_run_id: 2,
            source_snapshot_id: 3,
            source_ref_id: "source_ref_1".to_string(),
            part: GemAnalysisPart::Comments,
            attempt_number: 2,
            prompt_input_json: "{}".to_string(),
            raw_output: "not json".to_string(),
            error_message: "parse failed".to_string(),
        };
        let stage_request =
            YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(repair_request.clone());
        assert!(matches!(
            stage_request,
            YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(request)
                if request.attempt_number == 2
        ));
    }

    #[test]
    fn parse_part_output_accepts_matching_non_empty_markdown() {
        let raw = serde_json::json!({
            "part": "passport",
            "markdown": "### Section\nText",
        })
        .to_string();

        let parsed =
            parse_gem_analysis_part_output(&raw, GemAnalysisPart::Passport).expect("parse");

        assert_eq!(parsed.part, GemAnalysisPart::Passport);
        assert_eq!(parsed.markdown, "### Section\nText");
    }

    #[test]
    fn parse_part_output_rejects_wrong_part() {
        let raw = serde_json::json!({
            "part": "comments",
            "markdown": "### Section",
        })
        .to_string();

        let error = parse_gem_analysis_part_output(&raw, GemAnalysisPart::Passport)
            .expect_err("wrong part");

        assert!(error.message.contains("expected part passport"));
    }

    #[test]
    fn parse_part_output_rejects_empty_markdown() {
        let error = parse_gem_analysis_part_output(
            r#"{"part":"passport","markdown":"   "}"#,
            GemAnalysisPart::Passport,
        )
        .expect_err("empty markdown");

        assert!(error.message.contains("markdown"));
    }

    #[test]
    fn parse_part_output_accepts_json_fence_with_internal_markdown_code_block() {
        let raw = "```json\n{\"part\":\"deep_recap\",\"markdown\":\"### Code\\n```rust\\nfn main() {}\\n```\\nFormula: $E=mc^2$\"}\n```";

        let parsed = parse_gem_analysis_part_output(raw, GemAnalysisPart::DeepRecap)
            .expect("parse fenced JSON with code block inside markdown string");

        assert!(parsed.markdown.contains("```rust"));
        assert!(parsed.markdown.contains("$E=mc^2$"));
    }
}
