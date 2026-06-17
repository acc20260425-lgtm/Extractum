from typing import Any

from research.youtube_pipeline.models import NormalizedResult


def count_words(text: str) -> int:
    return len([part for part in text.split() if part.strip()])


def build_metrics(
    *,
    strategy: str,
    video_id: str,
    result: NormalizedResult,
    request_count: int,
    input_tokens: int,
    output_tokens: int,
    latency_seconds: float,
    json_valid: bool,
    notes: str = "",
) -> dict[str, Any]:
    return {
        "strategy": strategy,
        "video_id": video_id,
        "request_count": request_count,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "latency_seconds": latency_seconds,
        "summary_words": count_words(result.summary_text),
        "timeline_segments_count": len(result.timeline),
        "claims_count": len(result.claims),
        "evidence_count": len(result.evidence),
        "action_items_count": len(result.action_items),
        "open_questions_count": len(result.open_questions),
        "json_valid": json_valid,
        "notes": notes,
    }
