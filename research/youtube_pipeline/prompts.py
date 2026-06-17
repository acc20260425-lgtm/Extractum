from research.youtube_pipeline.llm_client import ChatMessage


RESULT_CONTRACT = """Return JSON with this shape:
{
  "summary_text": "detailed readable summary",
  "timeline": [{"start": "00:00:00", "end": "00:05:00", "title": "", "summary": ""}],
  "claims": [{"text": "", "importance": "high", "evidence_refs": []}],
  "evidence": [{"text": "", "timestamp": "00:00:00", "supports_claims": []}],
  "action_items": [{"text": "", "target_audience": "", "priority": "medium"}],
  "open_questions": [{"text": "", "why_it_matters": ""}]
}
"""


def build_one_shot_full_json_messages(transcript: str, *, output_language: str) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You analyze YouTube transcripts for research. Use only the transcript. "
                "Return one JSON object and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Create a detailed summary and fill timeline, claims, evidence, action_items, "
                "and open_questions. If action items are absent, return an empty action_items array.\n\n"
                f"{RESULT_CONTRACT}\n\nTranscript:\n{transcript}"
            ),
        ),
    ]


def build_chunk_analysis_messages(
    chunk_text: str,
    *,
    chunk_index: int,
    total_chunks: int,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You analyze one chunk of a longer YouTube transcript. Use only this chunk. "
                "Return one JSON object and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Chunk {chunk_index} of {total_chunks}\n\n"
                "Create dense chunk notes, not a short abstract. In summary_text write 300-600 words "
                "when the chunk has enough substance. Capture the concrete details, argument flow, examples, "
                "named concepts, transitions, and practical implications from this chunk. do not compress "
                "the chunk into a few sentences.\n\n"
                "Also extract timeline, claims, evidence, action_items, and open_questions that are visible "
                "in this chunk. Keep evidence grounded in the chunk text.\n\n"
                f"{RESULT_CONTRACT}\n\nTranscript chunk:\n{chunk_text}"
            ),
        ),
    ]


def build_chunk_reduce_messages(
    chunk_results_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You merge chunk-level YouTube transcript analyses into one coherent research result. "
                "Deduplicate repeated claims and evidence. Return one JSON object and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Merge these chunk analyses into a long-form report. In summary_text write 1200-2500 words "
                "when the source video has enough substance. do not summarize the summaries into a short "
                "abstract; expand from the chunk evidence and preserve concrete details from the beginning, "
                "middle, and end of the video.\n\n"
                "Structure summary_text with readable Markdown headings inside the JSON string:\n"
                "## Overview\n"
                "## Detailed narrative\n"
                "## Major claims and evidence\n"
                "## Actionable takeaways\n"
                "## Open questions\n\n"
                "Also produce consolidated timeline, claims, evidence, action_items, and open_questions. "
                "Deduplicate repeated items, but do not drop important nuance just because it appears in "
                "only one chunk.\n\n"
                f"{RESULT_CONTRACT}\n\nChunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]


def build_final_report_messages(
    chunk_results_json: str,
    structured_result_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You are a long-form research report writer. Write prose, not JSON. "
                "Use only the provided chunk analyses and structured result."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Please write the final report as Markdown. Do not return JSON. Aim for 2000-4000 words "
                "when the source material has enough substance. The report should be stronger than a short "
                "summary: explain the argument flow, important details, examples, evidence, tensions, "
                "actionable takeaways, and unresolved questions.\n\n"
                "Use this structure:\n"
                "# Overview\n"
                "# Detailed narrative\n"
                "# Timeline and development of ideas\n"
                "# Major claims and evidence\n"
                "# Actionable takeaways\n"
                "# Open questions\n"
                "# Final synthesis\n\n"
                f"Structured result JSON:\n{structured_result_json}\n\n"
                f"Chunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]
