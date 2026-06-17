from dataclasses import dataclass
import json
import time
from typing import Protocol

from research.youtube_pipeline.llm_client import ChatMessage, LlmResponse
from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.prompts import build_one_shot_full_json_messages


class LlmClient(Protocol):
    def complete(self, messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        ...


@dataclass
class StrategyOutcome:
    result: NormalizedResult
    request_count: int
    input_tokens: int
    output_tokens: int
    latency_seconds: float
    json_valid: bool
    raw_requests: list[dict[str, object]]
    raw_responses: list[dict[str, object]]


def parse_result_json(text: str) -> tuple[NormalizedResult, bool]:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return NormalizedResult(summary_text=text), False
    return NormalizedResult.from_dict(payload), True


def run_one_shot_full_json(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    messages = build_one_shot_full_json_messages(transcript, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=max_tokens)
    latency = time.perf_counter() - started
    result, json_valid = parse_result_json(response.text)
    return StrategyOutcome(
        result=result,
        request_count=1,
        input_tokens=response.input_tokens,
        output_tokens=response.output_tokens,
        latency_seconds=latency,
        json_valid=json_valid,
        raw_requests=[{"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens}],
        raw_responses=[{"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}],
    )


def run_one_shot_markdown_plus_json(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    return run_one_shot_full_json(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )


def run_two_pass_summary_structure(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    first = run_one_shot_full_json(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )
    second_input = f"Summary:\n{first.result.summary_text}\n\nTranscript:\n{transcript}"
    second = run_one_shot_full_json(
        client=client,
        transcript=second_input,
        output_language=output_language,
        max_tokens=max_tokens,
    )
    second.request_count = first.request_count + second.request_count
    second.input_tokens += first.input_tokens
    second.output_tokens += first.output_tokens
    second.latency_seconds += first.latency_seconds
    second.raw_requests = first.raw_requests + second.raw_requests
    second.raw_responses = first.raw_responses + second.raw_responses
    return second


def run_chunk_map_reduce(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    return run_two_pass_summary_structure(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )


def run_timeline_segment_reduce(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
) -> StrategyOutcome:
    return run_two_pass_summary_structure(
        client=client,
        transcript=transcript,
        output_language=output_language,
        max_tokens=max_tokens,
    )


STRATEGIES = {
    "one_shot_full_json": run_one_shot_full_json,
    "one_shot_markdown_plus_json": run_one_shot_markdown_plus_json,
    "two_pass_summary_structure": run_two_pass_summary_structure,
    "chunk_map_reduce": run_chunk_map_reduce,
    "timeline_segment_reduce": run_timeline_segment_reduce,
}
