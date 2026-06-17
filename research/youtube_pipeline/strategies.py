from dataclasses import dataclass
import json
import time
from typing import Protocol

from research.youtube_pipeline.chunking import chunk_by_approx_tokens
from research.youtube_pipeline.llm_client import ChatMessage, LlmResponse
from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.prompts import (
    build_chunk_analysis_messages,
    build_chunk_reduce_messages,
    build_final_report_messages,
    build_one_shot_full_json_messages,
    build_antigravity_reduce_summary_messages,
    build_antigravity_reduce_timeline_messages,
    build_antigravity_reduce_claims_evidence_messages,
    build_antigravity_reduce_takeaways_messages,
)


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
    chunk_token_limit: int = 3000,
) -> StrategyOutcome:
    chunks = chunk_by_approx_tokens(transcript, max_tokens=chunk_token_limit)
    if len(chunks) <= 1:
        return run_one_shot_full_json(
            client=client,
            transcript=transcript,
            output_language=output_language,
            max_tokens=max_tokens,
        )

    raw_requests: list[dict[str, object]] = []
    raw_responses: list[dict[str, object]] = []
    chunk_results: list[dict[str, object]] = []
    request_count = 0
    input_tokens = 0
    output_tokens = 0
    latency_seconds = 0.0
    json_valid = True

    for index, chunk in enumerate(chunks, start=1):
        messages = build_chunk_analysis_messages(
            chunk,
            chunk_index=index,
            total_chunks=len(chunks),
            output_language=output_language,
        )
        started = time.perf_counter()
        response = client.complete(messages, max_tokens=max_tokens)
        latency_seconds += time.perf_counter() - started
        request_count += 1
        input_tokens += response.input_tokens
        output_tokens += response.output_tokens
        raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens})
        raw_responses.append(
            {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
        )
        chunk_result, chunk_json_valid = parse_result_json(response.text)
        json_valid = json_valid and chunk_json_valid
        chunk_results.append(
            {
                "chunk_index": index,
                "total_chunks": len(chunks),
                "result": chunk_result.to_dict(),
            }
        )

    reduce_input = json.dumps(chunk_results, ensure_ascii=False, indent=2)
    messages = build_chunk_reduce_messages(reduce_input, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    result, reduce_json_valid = parse_result_json(response.text)
    json_valid = json_valid and reduce_json_valid

    structured_result_json = json.dumps(result.to_dict(), ensure_ascii=False, indent=2)
    messages = build_final_report_messages(
        reduce_input,
        structured_result_json,
        output_language=output_language,
    )
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    if response.text.strip():
        result.summary_text = response.text.strip()

    return StrategyOutcome(
        result=result,
        request_count=request_count,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        latency_seconds=latency_seconds,
        json_valid=json_valid,
        raw_requests=raw_requests,
        raw_responses=raw_responses,
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


def run_antigravity_chunk_map_reduce(
    *,
    client: LlmClient,
    transcript: str,
    output_language: str,
    max_tokens: int,
    chunk_token_limit: int = 3000,
) -> StrategyOutcome:
    chunks = chunk_by_approx_tokens(transcript, max_tokens=chunk_token_limit)
    if len(chunks) <= 1:
        return run_one_shot_full_json(
            client=client,
            transcript=transcript,
            output_language=output_language,
            max_tokens=max_tokens,
        )

    raw_requests: list[dict[str, object]] = []
    raw_responses: list[dict[str, object]] = []
    chunk_results: list[dict[str, object]] = []
    request_count = 0
    input_tokens = 0
    output_tokens = 0
    latency_seconds = 0.0
    json_valid = True

    # 1. Map step: Analyze chunks
    for index, chunk in enumerate(chunks, start=1):
        messages = build_chunk_analysis_messages(
            chunk,
            chunk_index=index,
            total_chunks=len(chunks),
            output_language=output_language,
        )
        started = time.perf_counter()
        response = client.complete(messages, max_tokens=max_tokens)
        latency_seconds += time.perf_counter() - started
        request_count += 1
        input_tokens += response.input_tokens
        output_tokens += response.output_tokens
        raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens})
        raw_responses.append(
            {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
        )
        chunk_result, chunk_json_valid = parse_result_json(response.text)
        json_valid = json_valid and chunk_json_valid
        chunk_results.append(
            {
                "chunk_index": index,
                "total_chunks": len(chunks),
                "result": chunk_result.to_dict(),
            }
        )

    reduce_input = json.dumps(chunk_results, ensure_ascii=False, indent=2)

    # 2a. Reduce Summary step
    summary_messages = build_antigravity_reduce_summary_messages(reduce_input, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(summary_messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in summary_messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    summary_text = response.text.strip()
    summary_json_valid = True
    json_valid = json_valid and summary_json_valid

    # 2b. Reduce Timeline step
    timeline_messages = build_antigravity_reduce_timeline_messages(reduce_input, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(timeline_messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in timeline_messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    timeline_result, timeline_json_valid = parse_result_json(response.text)
    json_valid = json_valid and timeline_json_valid

    # 2c. Reduce Claims & Evidence step
    claims_evidence_messages = build_antigravity_reduce_claims_evidence_messages(reduce_input, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(claims_evidence_messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in claims_evidence_messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    claims_evidence_result, claims_evidence_json_valid = parse_result_json(response.text)
    json_valid = json_valid and claims_evidence_json_valid

    # 2d. Reduce Takeaways step
    takeaways_messages = build_antigravity_reduce_takeaways_messages(reduce_input, output_language=output_language)
    started = time.perf_counter()
    response = client.complete(takeaways_messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in takeaways_messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    takeaways_result, takeaways_json_valid = parse_result_json(response.text)
    json_valid = json_valid and takeaways_json_valid

    # Combine summary and lists
    combined_result = NormalizedResult(
        summary_text=summary_text,
        timeline=timeline_result.timeline,
        claims=claims_evidence_result.claims,
        evidence=claims_evidence_result.evidence,
        action_items=takeaways_result.action_items,
        open_questions=takeaways_result.open_questions,
    )

    # 3. Final report formatting step
    structured_result_json = json.dumps(combined_result.to_dict(), ensure_ascii=False, indent=2)
    messages = build_final_report_messages(
        reduce_input,
        structured_result_json,
        output_language=output_language,
    )
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    if response.text.strip():
        combined_result.summary_text = response.text.strip()

    return StrategyOutcome(
        result=combined_result,
        request_count=request_count,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        latency_seconds=latency_seconds,
        json_valid=json_valid,
        raw_requests=raw_requests,
        raw_responses=raw_responses,
    )


STRATEGIES = {
    "antigravity_chunk_map_reduce": run_antigravity_chunk_map_reduce,
    "one_shot_full_json": run_one_shot_full_json,
    "one_shot_markdown_plus_json": run_one_shot_markdown_plus_json,
    "two_pass_summary_structure": run_two_pass_summary_structure,
    "chunk_map_reduce": run_chunk_map_reduce,
    "timeline_segment_reduce": run_timeline_segment_reduce,
}
