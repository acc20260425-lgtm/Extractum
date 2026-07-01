use crate::llm::{LlmChatRequest, LlmMessage};

use super::super::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage};
use super::super::trace::normalize_ref;
use super::super::{now_secs, ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS};

const ANALYSIS_CHUNK_PROMPT_OVERHEAD_TOKENS: usize = 1_500;
const ANALYSIS_CHUNK_OUTPUT_RESERVE_TOKENS: usize = 2_000;
const ANALYSIS_CHUNK_SAFETY_PERCENT: usize = 80;
const ANALYSIS_CHUNK_ESTIMATED_CHARS_PER_TOKEN: usize = 3;
const ANALYSIS_CHUNK_MIN_TARGET_CHARS: usize = 2_000;

pub(super) fn chunk_messages(
    messages: &[CorpusMessage],
    max_chars: usize,
) -> Vec<Vec<CorpusMessage>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_chars = 0usize;

    for message in messages {
        let estimated_len = message.content.len()
            + message.r#ref.len()
            + message.author.as_deref().unwrap_or("").len()
            + 64;

        if !current.is_empty() && current_chars + estimated_len > max_chars {
            chunks.push(current);
            current = Vec::new();
            current_chars = 0;
        }

        current_chars += estimated_len;
        current.push(message.clone());
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn format_chunk_corpus(messages: &[CorpusMessage]) -> String {
    messages
        .iter()
        .map(|message| {
            format!(
                "[{ref}]\nDate: {published_at}\nAuthor: {author}\nContent:\n{content}",
                ref = message.r#ref,
                published_at = message.published_at,
                author = message.author.as_deref().unwrap_or("unknown"),
                content = message.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

pub(super) fn build_map_request(
    run_id: i64,
    profile_id: String,
    chunk_index: usize,
    total_chunks: usize,
    messages: &[CorpusMessage],
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!("analysis-map-{run_id}-{chunk_index}-{}", now_secs()),
        profile_id: Some(profile_id),
        model_override: None,
        max_output_tokens: None,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "You analyze source document excerpts. Return a strict JSON object only with keys: summary, topics, notable_points, candidate_refs. Do not wrap JSON in markdown fences. Use only refs that appear in the provided documents.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Chunk {chunk_index} of {total_chunks}.\nSummarize the source documents below for later reduction.\n\nDocuments:\n\n{}",
                    format_chunk_corpus(messages)
                ),
            },
        ],
    }
}

pub(super) fn extract_json_payload(text: &str) -> Result<&str, String> {
    let mut search_from = 0usize;
    let mut saw_candidate = false;

    while let Some(relative_start) = text[search_from..].find('{') {
        let start = search_from + relative_start;
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaping = false;

        for (offset, character) in text[start..].char_indices() {
            if in_string {
                if escaping {
                    escaping = false;
                    continue;
                }
                match character {
                    '\\' => escaping = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match character {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        saw_candidate = true;
                        let end = start + offset + character.len_utf8();
                        let candidate = &text[start..end];
                        if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                            return Ok(candidate);
                        }
                        search_from = start + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if search_from <= start {
            return Err("LLM response contained malformed JSON boundaries".to_string());
        }
    }

    if saw_candidate {
        Err("LLM response did not contain a valid JSON object".to_string())
    } else {
        Err("LLM response did not contain JSON".to_string())
    }
}

pub(super) fn parse_chunk_summary(text: &str) -> Result<ChunkSummary, String> {
    let payload = extract_json_payload(text)?;
    serde_json::from_str(payload).map_err(|e| format!("Failed to parse chunk summary JSON: {e}"))
}

fn summarize_chunk_for_reduce(summary: &ChunkSummary) -> String {
    let topics = if summary.topics.is_empty() {
        "- none".to_string()
    } else {
        summary
            .topics
            .iter()
            .map(|topic| format!("- {topic}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let notable_points = if summary.notable_points.is_empty() {
        "- none".to_string()
    } else {
        summary
            .notable_points
            .iter()
            .map(|point| format!("- {point}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let refs = if summary.candidate_refs.is_empty() {
        "- none".to_string()
    } else {
        summary
            .candidate_refs
            .iter()
            .filter_map(|candidate| normalize_ref(candidate))
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Summary:\n{}\n\nTopics:\n{}\n\nNotable points:\n{}\n\nCandidate refs:\n{}",
        summary.summary.trim(),
        topics,
        notable_points,
        refs
    )
}

pub(super) struct ReduceRequestParams<'a> {
    pub(super) run_id: i64,
    pub(super) profile_id: String,
    pub(super) scope_label: &'a str,
    pub(super) output_language: &'a str,
    pub(super) prompt_template: &'a AnalysisPromptTemplate,
    pub(super) period_from: i64,
    pub(super) period_to: i64,
    pub(super) chunk_summaries: &'a [ChunkSummary],
    pub(super) model_override: Option<String>,
}

pub(super) fn build_reduce_request(params: ReduceRequestParams<'_>) -> LlmChatRequest {
    let combined = params
        .chunk_summaries
        .iter()
        .enumerate()
        .map(|(index, summary)| {
            format!(
                "Chunk {} summary\n{}\n",
                index + 1,
                summarize_chunk_for_reduce(summary)
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n");

    LlmChatRequest {
        request_id: format!("analysis-reduce-{}-{}", params.run_id, now_secs()),
        profile_id: Some(params.profile_id),
        model_override: params.model_override,
        max_output_tokens: None,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: format!(
                    "You write grounded markdown reports over already-summarized source documents.\nAnswer in {}.\nUse markdown only.\nEvery important conclusion must cite one or more refs like [s12-i845].\nDo not invent facts beyond the provided chunk summaries.",
                    params.output_language
                ),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Analysis scope: {scope_label}\nPeriod: {period_from} to {period_to}\n\nUser report template:\n{template}\n\nChunk summaries:\n\n{combined}",
                    scope_label = params.scope_label,
                    period_from = params.period_from,
                    period_to = params.period_to,
                    template = params.prompt_template.body
                ),
            },
        ],
    }
}

pub(super) fn chunk_target_chars_for_model_input_limit(
    model_input_token_limit: Option<usize>,
) -> usize {
    let Some(model_input_token_limit) = model_input_token_limit else {
        return ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS;
    };

    let reserved_tokens =
        ANALYSIS_CHUNK_PROMPT_OVERHEAD_TOKENS + ANALYSIS_CHUNK_OUTPUT_RESERVE_TOKENS;
    if model_input_token_limit <= reserved_tokens {
        return ANALYSIS_CHUNK_MIN_TARGET_CHARS;
    }

    let usable_tokens =
        (model_input_token_limit - reserved_tokens) * ANALYSIS_CHUNK_SAFETY_PERCENT / 100;
    (usable_tokens * ANALYSIS_CHUNK_ESTIMATED_CHARS_PER_TOKEN).max(ANALYSIS_CHUNK_MIN_TARGET_CHARS)
}
