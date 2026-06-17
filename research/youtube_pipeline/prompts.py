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


def build_antigravity_reduce_summary_messages(
    chunk_results_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You merge chunk-level YouTube transcript analyses' summaries into one coherent, highly detailed narrative report. "
                "Write in Markdown. Do not return JSON or any JSON wrappers. Write only prose."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Merge these chunk analyses into a long-form narrative report. Write at least 1500-2500 words. "
                "Do not summarize the summaries into a short abstract; expand from the chunk evidence and preserve concrete details "
                "from the beginning, middle, and end of the video. Use multiple paragraphs for each major topic to explain it in depth.\n\n"
                "Structure your report with readable Markdown headings:\n"
                "# Overview\n"
                "# Detailed narrative\n"
                "Use subheadings (## or ###) within the detailed narrative to separate major topics.\n\n"
                f"Chunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]


def build_antigravity_reduce_timeline_messages(
    chunk_results_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You merge chunk-level YouTube transcript analyses' timelines into one coherent, chronologically sorted timeline. "
                "Deduplicate closely overlapping segments and merge adjacent items if they cover the same topic. "
                "Return one JSON object containing ONLY the 'timeline' field, and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Merge and deduplicate these chunk analyses' timelines. Sort them chronologically. "
                "Aim for a detailed timeline that represents the entire flow of the video.\n\n"
                "Return exactly this JSON shape:\n"
                "{\n"
                '  "timeline": [{"start": "00:00:00", "end": "00:05:00", "title": "Topic", "summary": "Detailed summary"}]\n'
                "}\n\n"
                f"Chunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]


def build_antigravity_reduce_claims_evidence_messages(
    chunk_results_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You merge chunk-level YouTube transcript analyses' claims and evidence. "
                "Deduplicate repeated items. Keep evidence items grounded in timestamps and correctly linked to their supported claims. "
                "Return one JSON object containing ONLY 'claims' and 'evidence' fields, and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Merge and deduplicate these chunk analyses' claims and evidence. Maintain the evidence references "
                "and ensure they map correctly. Keep the statements concrete.\n\n"
                "Return exactly this JSON shape:\n"
                "{\n"
                '  "claims": [{"text": "Claim statement", "importance": "high", "evidence_refs": ["00:01:00"]}],\n'
                '  "evidence": [{"text": "Grounded evidence description", "timestamp": "00:01:00", "supports_claims": ["Claim statement"]}]\n'
                "}\n\n"
                f"Chunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]


def build_antigravity_reduce_takeaways_messages(
    chunk_results_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You merge chunk-level YouTube transcript analyses' action items and open questions. "
                "Deduplicate repeated items and preserve all unique nuances. "
                "Return one JSON object containing ONLY 'action_items' and 'open_questions' fields, and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Merge and deduplicate these chunk analyses' action items and open questions.\n\n"
                "Return exactly this JSON shape:\n"
                "{\n"
                '  "action_items": [{"text": "Action item", "target_audience": "Audience", "priority": "medium"}],\n'
                '  "open_questions": [{"text": "Open question", "why_it_matters": "Reason"}]\n'
                "}\n\n"
                f"Chunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]


def build_antigravity_chunk_analysis_messages(
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
                "Create dense chunk notes, not a short abstract. In summary_text write 600-1000 words "
                "when the chunk has enough substance. Capture the concrete details, argument flow, examples, "
                "named concepts, transitions, and practical implications from this chunk. do not compress "
                "the chunk into a few sentences.\n\n"
                "Also extract timeline, claims, evidence, action_items, and open_questions that are visible "
                "in this chunk. Keep evidence grounded in the chunk text.\n\n"
                f"{RESULT_CONTRACT}\n\nTranscript chunk:\n{chunk_text}"
            ),
        ),
    ]


def build_antigravity_chapter_summary_messages(
    chapter_chunks_json: str,
    chapter_index: int,
    total_chapters: int,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                f"You write Chapter {chapter_index} of a long-form detailed narrative report for a YouTube video. "
                "Write in Markdown. Do not return JSON. Write only prose."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                f"You are writing Chapter {chapter_index} of {total_chapters}. "
                "Write a highly detailed, comprehensive chapter narrative (600-800 words) based on the provided chunk analyses. "
                "Focus on explaining the argument flow, concrete details, examples, and points of discussion in this part of the video. "
                "Do not summarize or compress it into a short abstract; write a rich, multi-paragraph text.\n\n"
                "Use a markdown heading for this chapter, e.g.:\n"
                f"## Chapter {chapter_index}: [Descriptive Title of this Chapter]\n\n"
                f"Chunk analyses for this chapter:\n{chapter_chunks_json}"
            ),
        ),
    ]


def build_antigravity_final_report_messages(
    chunk_results_json: str,
    structured_result_json: str,
    *,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You are an expert long-form research report writer. Write detailed, academic-grade prose, not JSON. "
                "Do not summarize or shorten the information. Expand and write in detail."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Write a comprehensive, long-form research report based on the provided chunk analyses and structured results. "
                "To ensure maximum coverage of the 3-hour video, you MUST expand each section in detail. "
                "Do not write a short summary; write a detailed, thorough narrative that explains all concepts, arguments, and examples. "
                "Aim for a total length of 2000 to 4000 words.\n\n"
                "Use the following structure and guidelines for each section:\n\n"
                "# Overview\n"
                "Write 2-3 paragraphs (300-500 words) summarizing the main theme, context, and significance of the video.\n\n"
                "# Detailed narrative\n"
                "In this section, you MUST output the pre-generated chapter narrative from 'summary_text' in the Structured result JSON verbatim. "
                "Do not shorten, compress, or rewrite it. Preserve all its subheadings (## Chapter X) exactly as they are.\n\n"
                "# Timeline and development of ideas\n"
                "List all timeline items from the structured result. For each item, write a 2-3 sentence description of what was discussed, "
                "preserving the timestamps and details.\n\n"
                "# Major claims and evidence\n"
                "Present all claims and evidence from the structured result. Elaborate on each claim and explain how the evidence supports it, "
                "using details from the chunk analyses.\n\n"
                "# Actionable takeaways\n"
                "Detail all actionable takeaways, explaining their practical implications and target audiences.\n\n"
                "# Open questions\n"
                "List all open questions and write a paragraph for each, explaining why it matters and what its implications are.\n\n"
                "# Final synthesis\n"
                "Write 2-3 paragraphs of final synthesis, concluding the main takeaways, themes, and long-term implications.\n\n"
                f"Structured result JSON:\n{structured_result_json}\n\n"
                f"Chunk analyses JSON:\n{chunk_results_json}"
            ),
        ),
    ]

