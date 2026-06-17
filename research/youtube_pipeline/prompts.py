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
