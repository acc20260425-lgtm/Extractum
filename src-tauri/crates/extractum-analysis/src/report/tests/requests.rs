use super::super::{
    build_map_request, build_reduce_request, extract_json_payload, parse_chunk_summary,
    ReduceRequestParams,
};
use super::harness::{
    sample_chunk_summary, sample_corpus_message, sample_prompt_template, SAMPLE_JSON,
};

#[test]
fn extracts_json_with_text_before_and_after() {
    let response = format!("Preface\n{SAMPLE_JSON}\nTail");
    let payload = extract_json_payload(&response).expect("extract payload");

    assert_eq!(payload, SAMPLE_JSON);
}

#[test]
fn extracts_json_inside_markdown_fence() {
    let response = format!("```json\n{SAMPLE_JSON}\n```");
    let payload = extract_json_payload(&response).expect("extract fenced payload");

    assert_eq!(payload, SAMPLE_JSON);
}

#[test]
fn parse_chunk_summary_ignores_non_json_prefix_with_braces() {
    let summary =
        parse_chunk_summary(&format!("Note {{not json}}\n{SAMPLE_JSON}")).expect("parse summary");

    assert_eq!(summary.summary, "Brief");
    assert_eq!(summary.topics, vec!["sync".to_string()]);
}

#[test]
fn parse_chunk_summary_rejects_malformed_payload() {
    let error = parse_chunk_summary("```json\n{\"summary\": }\n```")
        .expect_err("malformed payload should fail");

    assert!(
        error.contains("Failed to parse chunk summary JSON")
            || error.contains("malformed JSON")
            || error.contains("valid JSON object")
    );
}

#[test]
fn build_map_request_keeps_run_scoped_request_and_profile() {
    let request = build_map_request(55, "default".to_string(), 2, 4, &[sample_corpus_message()]);

    assert!(request.request_id.starts_with("analysis-map-55-2-"));
    assert_eq!(request.profile_id.as_deref(), Some("default"));
    assert!(request.messages[0]
        .content
        .contains("source document excerpts"));
    assert!(request.messages[1].content.contains("Chunk 2 of 4."));
    assert!(request.messages[1].content.contains("Documents:"));
}

#[test]
fn build_reduce_request_keeps_run_scoped_request_and_profile() {
    let prompt_template = sample_prompt_template();
    let chunk_summaries = vec![sample_chunk_summary("alpha"), sample_chunk_summary("beta")];
    let request = build_reduce_request(ReduceRequestParams {
        run_id: 77,
        profile_id: "profile-a".to_string(),
        scope_label: "My scope",
        output_language: "Russian",
        prompt_template: &prompt_template,
        period_from: 10,
        period_to: 20,
        chunk_summaries: &chunk_summaries,
        model_override: Some("model-x".to_string()),
    });

    assert!(request.request_id.starts_with("analysis-reduce-77-"));
    assert_eq!(request.profile_id.as_deref(), Some("profile-a"));
    assert_eq!(request.model_override.as_deref(), Some("model-x"));
    assert!(request.messages[0].content.contains("[s12-i845]"));
    assert!(request.messages[1].content.contains("Chunk 1 summary"));
    assert!(request.messages[1].content.contains("Chunk 2 summary"));
}
