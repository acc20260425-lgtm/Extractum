from dataclasses import dataclass, field
import json
import time
from typing import Protocol

from research.youtube_pipeline.adaptive import (
    assemble_adaptive_markdown_report,
    build_outline_chunk_descriptors,
    compute_budget_plan,
    extract_previous_chapter_bridge,
    first_words,
    normalize_substance_score,
    partition_weighted_chunks,
    response_token_budget,
)
from research.youtube_pipeline.chunking import chunk_by_approx_tokens
from research.youtube_pipeline.llm_client import ChatMessage, LlmResponse
from research.youtube_pipeline.models import NormalizedResult
from research.youtube_pipeline.moc import (
    align_fact_clusters_to_moc,
    approximate_token_count,
    assemble_moc_markdown_report,
    build_evidence_slice,
    build_structured_result_from_facts,
    build_temporal_projection,
    chunk_segments_by_approx_tokens,
    compute_moc_budget,
    deduplicate_facts,
    fallback_moc_plan,
    format_segments_for_prompt,
    markdown_unaligned_facts,
    parse_timestamped_transcript,
    word_count,
)
from research.youtube_pipeline.prompts import (
    build_adaptive_chapter_expansion_messages,
    build_adaptive_chapter_generation_messages,
    build_adaptive_chapter_outline_messages,
    build_adaptive_chunk_analysis_messages,
    build_adaptive_conclusion_messages,
    build_adaptive_overview_messages,
    build_chunk_analysis_messages,
    build_chunk_reduce_messages,
    build_final_report_messages,
    build_one_shot_full_json_messages,
    build_antigravity_reduce_summary_messages,
    build_antigravity_reduce_timeline_messages,
    build_antigravity_reduce_claims_evidence_messages,
    build_antigravity_reduce_takeaways_messages,
    build_antigravity_chunk_analysis_messages,
    build_antigravity_chapter_summary_messages,
    build_antigravity_final_report_messages,
    build_moc_conclusion_messages,
    build_moc_map_extraction_messages,
    build_moc_node_expansion_messages,
    build_moc_node_section_messages,
    build_moc_overview_messages,
    build_moc_plan_messages,
)


class LlmClient(Protocol):
    def complete(self, messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        ...


@dataclass
class StrategyOptions:
    output_language: str = "ru"
    video_id: str = "video"
    max_tokens: int = 8192
    chunk_token_limit: int = 3000
    chunk_overlap_tokens: int = 700
    target_depth: str = "auto"
    min_report_words: int | None = None
    max_report_words: int | None = None
    chapter_target_words: int = 900
    planner_context_token_limit: int = 120000
    max_slice_tokens: int = 8000
    max_parallel_map_calls: int = 4
    max_parallel_node_calls: int = 3


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
    extra_metrics: dict[str, object] = field(default_factory=dict)
    extra_artifacts: dict[str, object | str] = field(default_factory=dict)


def parse_result_json(text: str) -> tuple[NormalizedResult, bool]:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return NormalizedResult(summary_text=text), False
    return NormalizedResult.from_dict(payload), True


def parse_json_payload(text: str) -> tuple[dict[str, object], bool]:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return {}, False
    if not isinstance(payload, dict):
        return {}, False
    return payload, True


def markdown_timeline(result: NormalizedResult) -> str:
    if not result.timeline:
        return "_No timeline items extracted._"
    lines = []
    for item in result.timeline:
        lines.append(f"- **{item.start}-{item.end}**: {item.title} - {item.summary}")
    return "\n".join(lines)


def markdown_claims(result: NormalizedResult) -> str:
    if not result.claims and not result.evidence:
        return "_No claims or evidence extracted._"
    lines = []
    for claim in result.claims:
        refs = ", ".join(claim.evidence_refs)
        suffix = f" Evidence refs: {refs}." if refs else ""
        lines.append(f"- **{claim.importance}**: {claim.text}.{suffix}")
    for evidence in result.evidence:
        lines.append(f"  - Evidence at {evidence.timestamp}: {evidence.text}")
    return "\n".join(lines)


def markdown_action_items(result: NormalizedResult) -> str:
    if not result.action_items:
        return "_No actionable takeaways extracted._"
    return "\n".join(
        f"- **{item.priority}** for {item.target_audience}: {item.text}" for item in result.action_items
    )


def markdown_open_questions(result: NormalizedResult) -> str:
    if not result.open_questions:
        return "_No open questions extracted._"
    return "\n".join(f"- {item.text} - {item.why_it_matters}" for item in result.open_questions)


def compact_json_items(items: object, *, limit: int, word_limit: int) -> list[object]:
    if not isinstance(items, list):
        return []
    compacted: list[object] = []
    for item in items[:limit]:
        if isinstance(item, dict):
            compacted.append(
                {
                    key: first_words(value, word_limit) if isinstance(value, str) else value
                    for key, value in item.items()
                }
            )
        else:
            compacted.append(first_words(str(item), word_limit))
    return compacted


def compact_chunk_result_for_chapter(row: dict[str, object]) -> dict[str, object]:
    result = row.get("result") if isinstance(row.get("result"), dict) else {}
    return {
        "chunk_index": row.get("chunk_index"),
        "total_chunks": row.get("total_chunks"),
        "substance_score": row.get("substance_score", 3),
        "summary_text": first_words(str(result.get("summary_text", "")), 250),
        "timeline": compact_json_items(result.get("timeline"), limit=5, word_limit=50),
        "claims": compact_json_items(result.get("claims"), limit=5, word_limit=50),
        "evidence": compact_json_items(result.get("evidence"), limit=5, word_limit=50),
        "action_items": compact_json_items(result.get("action_items"), limit=3, word_limit=50),
        "open_questions": compact_json_items(result.get("open_questions"), limit=3, word_limit=50),
    }


def run_one_shot_full_json(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    messages = build_one_shot_full_json_messages(transcript, output_language=options.output_language)
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=options.max_tokens)
    latency = time.perf_counter() - started
    result, json_valid = parse_result_json(response.text)
    return StrategyOutcome(
        result=result,
        request_count=1,
        input_tokens=response.input_tokens,
        output_tokens=response.output_tokens,
        latency_seconds=latency,
        json_valid=json_valid,
        raw_requests=[{"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens}],
        raw_responses=[{"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}],
    )


def run_one_shot_markdown_plus_json(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    return run_one_shot_full_json(
        client=client,
        transcript=transcript,
        options=options,
    )


def run_two_pass_summary_structure(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    first = run_one_shot_full_json(
        client=client,
        transcript=transcript,
        options=options,
    )
    second_input = f"Summary:\n{first.result.summary_text}\n\nTranscript:\n{transcript}"
    second = run_one_shot_full_json(
        client=client,
        transcript=second_input,
        options=options,
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
    options: StrategyOptions,
) -> StrategyOutcome:
    chunks = chunk_by_approx_tokens(transcript, max_tokens=options.chunk_token_limit)
    if len(chunks) <= 1:
        return run_one_shot_full_json(
            client=client,
            transcript=transcript,
            options=options,
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
            output_language=options.output_language,
        )
        started = time.perf_counter()
        response = client.complete(messages, max_tokens=options.max_tokens)
        latency_seconds += time.perf_counter() - started
        request_count += 1
        input_tokens += response.input_tokens
        output_tokens += response.output_tokens
        raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens})
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
    messages = build_chunk_reduce_messages(reduce_input, output_language=options.output_language)
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=options.max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    result, reduce_json_valid = parse_result_json(response.text)
    json_valid = json_valid and reduce_json_valid

    structured_result_json = json.dumps(result.to_dict(), ensure_ascii=False, indent=2)
    messages = build_final_report_messages(
        reduce_input,
        structured_result_json,
        output_language=options.output_language,
    )
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=options.max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens})
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
    options: StrategyOptions,
) -> StrategyOutcome:
    return run_two_pass_summary_structure(
        client=client,
        transcript=transcript,
        options=options,
    )


def run_antigravity_chunk_map_reduce(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    chunks = chunk_by_approx_tokens(transcript, max_tokens=options.chunk_token_limit)
    if len(chunks) <= 1:
        return run_one_shot_full_json(
            client=client,
            transcript=transcript,
            options=options,
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
        messages = build_antigravity_chunk_analysis_messages(
            chunk,
            chunk_index=index,
            total_chunks=len(chunks),
            output_language=options.output_language,
        )
        started = time.perf_counter()
        response = client.complete(messages, max_tokens=options.max_tokens)
        latency_seconds += time.perf_counter() - started
        request_count += 1
        input_tokens += response.input_tokens
        output_tokens += response.output_tokens
        raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens})
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

    # 2a. Reduce Summary step: Split chunk results into 3 chapters
    num_chapters = min(3, len(chunk_results))
    k, m = divmod(len(chunk_results), num_chapters)
    chapter_groups = [
        chunk_results[i * k + min(i, m) : (i + 1) * k + min(i + 1, m)]
        for i in range(num_chapters)
    ]

    chapter_summaries = []
    for ch_idx, ch_group in enumerate(chapter_groups, start=1):
        chapter_input = json.dumps(ch_group, ensure_ascii=False, indent=2)
        summary_messages = build_antigravity_chapter_summary_messages(
            chapter_input,
            chapter_index=ch_idx,
            total_chapters=num_chapters,
            output_language=options.output_language,
        )
        started = time.perf_counter()
        response = client.complete(summary_messages, max_tokens=options.max_tokens)
        latency_seconds += time.perf_counter() - started
        request_count += 1
        input_tokens += response.input_tokens
        output_tokens += response.output_tokens
        raw_requests.append({"messages": [message.__dict__ for message in summary_messages], "max_tokens": options.max_tokens})
        raw_responses.append(
            {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
        )
        chapter_summaries.append(response.text.strip())

    summary_text = "\n\n".join(chapter_summaries)
    summary_json_valid = True
    json_valid = json_valid and summary_json_valid

    # 2b. Reduce Timeline step
    timeline_messages = build_antigravity_reduce_timeline_messages(reduce_input, output_language=options.output_language)
    started = time.perf_counter()
    response = client.complete(timeline_messages, max_tokens=options.max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in timeline_messages], "max_tokens": options.max_tokens})
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    timeline_result, timeline_json_valid = parse_result_json(response.text)
    json_valid = json_valid and timeline_json_valid

    # 2c. Reduce Claims & Evidence step
    claims_evidence_messages = build_antigravity_reduce_claims_evidence_messages(
        reduce_input, output_language=options.output_language
    )
    started = time.perf_counter()
    response = client.complete(claims_evidence_messages, max_tokens=options.max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append(
        {"messages": [message.__dict__ for message in claims_evidence_messages], "max_tokens": options.max_tokens}
    )
    raw_responses.append(
        {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
    )
    claims_evidence_result, claims_evidence_json_valid = parse_result_json(response.text)
    json_valid = json_valid and claims_evidence_json_valid

    # 2d. Reduce Takeaways step
    takeaways_messages = build_antigravity_reduce_takeaways_messages(reduce_input, output_language=options.output_language)
    started = time.perf_counter()
    response = client.complete(takeaways_messages, max_tokens=options.max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in takeaways_messages], "max_tokens": options.max_tokens})
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
    messages = build_antigravity_final_report_messages(
        reduce_input,
        structured_result_json,
        output_language=options.output_language,
    )
    started = time.perf_counter()
    response = client.complete(messages, max_tokens=options.max_tokens)
    latency_seconds += time.perf_counter() - started
    request_count += 1
    input_tokens += response.input_tokens
    output_tokens += response.output_tokens
    raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens})
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


def run_adaptive_book_report(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    if not transcript.strip():
        raise ValueError("transcript is empty")

    transcript_words = len(transcript.split())
    if transcript_words < 1000:
        outcome = run_one_shot_full_json(client=client, transcript=transcript, options=options)
        outcome.extra_metrics.update(
            {
                "strategy_variant": "adaptive_book_report_short_fallback",
                "transcript_words": transcript_words,
            }
        )
        return outcome

    chunks = chunk_by_approx_tokens(transcript, max_tokens=options.chunk_token_limit)
    raw_requests: list[dict[str, object]] = []
    raw_responses: list[dict[str, object]] = []
    request_count = 0
    input_tokens = 0
    output_tokens = 0
    latency_seconds = 0.0
    json_valid = True
    chunk_results: list[dict[str, object]] = []
    substance_scores: list[int] = []

    def call_llm(messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        nonlocal request_count, input_tokens, output_tokens, latency_seconds
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
        return response

    for index, chunk in enumerate(chunks, start=1):
        messages = build_adaptive_chunk_analysis_messages(
            chunk,
            chunk_index=index,
            total_chunks=len(chunks),
            output_language=options.output_language,
        )
        response = call_llm(messages, options.max_tokens)
        payload, valid = parse_json_payload(response.text)
        json_valid = json_valid and valid
        score = normalize_substance_score(payload.get("substance_score"))
        result = NormalizedResult.from_dict(payload)
        substance_scores.append(score)
        chunk_results.append(
            {
                "chunk_index": index,
                "total_chunks": len(chunks),
                "substance_score": score,
                "result": result.to_dict(),
            }
        )

    budget_plan = compute_budget_plan(
        transcript_words=transcript_words,
        substance_scores=substance_scores,
        options=options,
        chunk_count=len(chunks),
    )
    weights = [
        len(chunks[index].split()) * substance_scores[index]
        for index in range(len(chunks))
    ]
    groups = partition_weighted_chunks(weights, budget_plan.chapter_count)
    chapter_groups = [
        {
            "chapter_index": group_index,
            "assigned_chunk_indexes": [index + 1 for index in range(start, end)],
        }
        for group_index, (start, end) in enumerate(groups, start=1)
    ]

    descriptors_json = json.dumps(build_outline_chunk_descriptors(chunk_results), ensure_ascii=False, indent=2)
    chapter_groups_json = json.dumps(chapter_groups, ensure_ascii=False, indent=2)
    outline_messages = build_adaptive_chapter_outline_messages(
        chunk_descriptors_json=descriptors_json,
        chapter_groups_json=chapter_groups_json,
        report_min_words=budget_plan.report_min_words,
        report_max_words=budget_plan.report_max_words,
        output_language=options.output_language,
    )
    outline_response = call_llm(outline_messages, min(options.max_tokens, 2000))
    outline_payload, outline_valid = parse_json_payload(outline_response.text)
    json_valid = json_valid and outline_valid
    outline_fallback_used = False
    if not outline_valid:
        outline_fallback_used = True
        outline_payload = {
            "report_thesis": first_words(chunk_results[0]["result"].get("summary_text", ""), 30),
            "key_terms": [],
            "chapters": [
                {
                    "chapter_index": row["chapter_index"],
                    "title": f"Chapter {row['chapter_index']}",
                    "one_liner": "Covers the assigned transcript chunks.",
                    "assigned_chunk_indexes": row["assigned_chunk_indexes"],
                }
                for row in chapter_groups
            ],
        }
    if not str(outline_payload.get("report_thesis", "")).strip():
        outline_payload["report_thesis"] = first_words(chunk_results[0]["result"].get("summary_text", ""), 30)
    if not isinstance(outline_payload.get("key_terms"), list):
        outline_payload["key_terms"] = []
    outline_json = json.dumps(outline_payload, ensure_ascii=False, indent=2)

    chapters: list[str] = []
    chapter_titles: list[str] = []
    expansion_call_count = 0
    chapter_expansion_shortfall = False
    previous_bridge = ""
    outline_chapters = outline_payload.get("chapters") if isinstance(outline_payload.get("chapters"), list) else []
    for group_index, (start, end) in enumerate(groups, start=1):
        outline_entry = next(
            (
                item for item in outline_chapters
                if isinstance(item, dict) and int(item.get("chapter_index", 0) or 0) == group_index
            ),
            {
                "chapter_index": group_index,
                "title": f"Chapter {group_index}",
                "one_liner": "Covers the assigned transcript chunks.",
                "assigned_chunk_indexes": [index + 1 for index in range(start, end)],
            },
        )
        title = str(outline_entry.get("title") or f"Chapter {group_index}")
        chapter_titles.append(title)
        assigned_notes = [compact_chunk_result_for_chapter(row) for row in chunk_results[start:end]]
        assigned_notes_json = json.dumps(assigned_notes, ensure_ascii=False, indent=2)
        chapter_messages = build_adaptive_chapter_generation_messages(
            chapter_index=group_index,
            total_chapters=len(groups),
            chapter_word_target=budget_plan.chapter_word_target,
            assigned_notes_json=assigned_notes_json,
            outline_json=outline_json,
            previous_bridge=previous_bridge,
            output_language=options.output_language,
        )
        chapter_response = call_llm(
            chapter_messages,
            response_token_budget(budget_plan.chapter_word_target, options.output_language, options.max_tokens),
        )
        chapter_text = chapter_response.text.strip()
        chapter_word_count = len(chapter_text.split())
        if chapter_word_count < int(0.8 * budget_plan.chapter_word_target):
            expansion_call_count += 1
            expansion_messages = build_adaptive_chapter_expansion_messages(
                chapter_index=group_index,
                chapter_word_target=budget_plan.chapter_word_target,
                current_word_count=chapter_word_count,
                chapter_draft=chapter_text,
                assigned_notes_json=assigned_notes_json,
                outline_entry_json=json.dumps(outline_entry, ensure_ascii=False, indent=2),
                report_thesis=str(outline_payload.get("report_thesis", "")),
                key_terms=[str(term) for term in outline_payload.get("key_terms", [])],
                previous_bridge=previous_bridge,
                output_language=options.output_language,
            )
            expanded = call_llm(
                expansion_messages,
                response_token_budget(budget_plan.chapter_word_target, options.output_language, options.max_tokens),
            ).text.strip()
            if expanded:
                chapter_text = expanded
        if len(chapter_text.split()) < int(0.8 * budget_plan.chapter_word_target):
            chapter_expansion_shortfall = True
        chapters.append(chapter_text)
        previous_bridge = extract_previous_chapter_bridge(chapter_text)

    reduce_input = json.dumps(chunk_results, ensure_ascii=False, indent=2)

    timeline_response = call_llm(
        build_antigravity_reduce_timeline_messages(reduce_input, output_language=options.output_language),
        options.max_tokens,
    )
    timeline_result, timeline_valid = parse_result_json(timeline_response.text)
    json_valid = json_valid and timeline_valid

    claims_response = call_llm(
        build_antigravity_reduce_claims_evidence_messages(reduce_input, output_language=options.output_language),
        options.max_tokens,
    )
    claims_result, claims_valid = parse_result_json(claims_response.text)
    json_valid = json_valid and claims_valid

    takeaways_response = call_llm(
        build_antigravity_reduce_takeaways_messages(reduce_input, output_language=options.output_language),
        options.max_tokens,
    )
    takeaways_result, takeaways_valid = parse_result_json(takeaways_response.text)
    json_valid = json_valid and takeaways_valid

    combined_result = NormalizedResult(
        timeline=timeline_result.timeline,
        claims=claims_result.claims,
        evidence=claims_result.evidence,
        action_items=takeaways_result.action_items,
        open_questions=takeaways_result.open_questions,
    )
    structured_result_json = json.dumps(combined_result.to_dict(), ensure_ascii=False, indent=2)

    overview = call_llm(
        build_adaptive_overview_messages(
            outline_json=outline_json,
            structured_result_json=structured_result_json,
            output_language=options.output_language,
        ),
        min(options.max_tokens, 2000),
    ).text.strip()
    conclusion = call_llm(
        build_adaptive_conclusion_messages(
            outline_json=outline_json,
            structured_result_json=structured_result_json,
            output_language=options.output_language,
        ),
        min(options.max_tokens, 2000),
    ).text.strip()

    combined_result.summary_text = assemble_adaptive_markdown_report(
        overview=overview,
        chapters=chapters,
        chapter_titles=chapter_titles,
        timeline_markdown=markdown_timeline(combined_result),
        claims_markdown=markdown_claims(combined_result),
        action_items_markdown=markdown_action_items(combined_result),
        open_questions_markdown=markdown_open_questions(combined_result),
        conclusion=conclusion,
    )

    identical_score_count = max((substance_scores.count(score) for score in set(substance_scores)), default=0)
    score_warning = bool(substance_scores and identical_score_count / len(substance_scores) > 0.8)
    return StrategyOutcome(
        result=combined_result,
        request_count=request_count,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        latency_seconds=latency_seconds,
        json_valid=json_valid,
        raw_requests=raw_requests,
        raw_responses=raw_responses,
        extra_metrics={
            "strategy_variant": "adaptive_book_report",
            "transcript_words": budget_plan.transcript_words,
            "report_min_words": budget_plan.report_min_words,
            "report_max_words": budget_plan.report_max_words,
            "target_report_words": budget_plan.target_report_words,
            "chapter_count": budget_plan.chapter_count,
            "chapter_word_target": budget_plan.chapter_word_target,
            "expansion_call_count": expansion_call_count,
            "outline_fallback_used": outline_fallback_used,
            "chapter_expansion_shortfall": chapter_expansion_shortfall,
            "average_substance_score": budget_plan.average_substance_score,
            "substance_multiplier": budget_plan.substance_multiplier,
            "substance_score_calibration_warning": score_warning,
        },
    )


def jsonl_rows(rows: list[dict[str, object]]) -> str:
    return "\n".join(json.dumps(row, ensure_ascii=False) for row in rows)


def node_target_word_count(node: object, default: int) -> int:
    if hasattr(node, "get"):
        value = node.get("target_word_count")
        if isinstance(value, int | float) and not isinstance(value, bool) and value > 0:
            return int(value)
    return default


def build_moc_global_context(moc_plan: dict[str, object]) -> dict[str, object]:
    nodes = moc_plan.get("nodes") if isinstance(moc_plan.get("nodes"), list) else []
    return {
        "video_id": moc_plan.get("video_id"),
        "report_thesis": moc_plan.get("report_thesis"),
        "global_key_terms": moc_plan.get("global_key_terms", []),
        "nodes": [
            {
                "node_id": node.get("node_id"),
                "title": node.get("title"),
                "time_span": node.get("time_span"),
                "target_word_count": node.get("target_word_count"),
            }
            for node in nodes
            if isinstance(node, dict)
        ],
    }


def run_moc_guided_map_reduce(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    if not transcript.strip():
        raise ValueError("transcript is empty")

    raw_requests: list[dict[str, object]] = []
    raw_responses: list[dict[str, object]] = []
    request_count = 0
    input_tokens = 0
    output_tokens = 0
    latency_seconds = 0.0
    json_valid = True

    def call_llm(messages: list[ChatMessage], max_tokens: int) -> LlmResponse:
        nonlocal request_count, input_tokens, output_tokens, latency_seconds
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
        return response

    video_id = options.video_id
    segments, transcript_warnings = parse_timestamped_transcript(transcript)
    transcript_words = word_count(transcript)
    budget = compute_moc_budget(transcript_words, options)
    estimated_tokens = approximate_token_count(transcript)
    moc_projection_used = estimated_tokens > options.planner_context_token_limit
    if moc_projection_used:
        transcript_context = json.dumps(
            build_temporal_projection(segments, source_word_count=transcript_words),
            ensure_ascii=False,
            indent=2,
        )
    else:
        transcript_context = format_segments_for_prompt(segments)

    plan_messages = build_moc_plan_messages(
        transcript_context=transcript_context,
        video_id=video_id,
        report_min_words=budget.report_min_words,
        report_max_words=budget.report_max_words,
        target_report_words=budget.target_report_words,
        expected_node_min=budget.expected_node_min,
        expected_node_max=budget.expected_node_max,
        output_language=options.output_language,
    )
    plan_response = call_llm(plan_messages, min(options.max_tokens, 4000))
    moc_plan, plan_valid = parse_json_payload(plan_response.text)
    moc_fallback_used = False
    nodes = moc_plan.get("nodes") if isinstance(moc_plan.get("nodes"), list) else None
    if not plan_valid:
        json_valid = False
    if not plan_valid or not nodes or any(not isinstance(node, dict) for node in nodes):
        json_valid = False
        moc_fallback_used = True
        moc_plan = fallback_moc_plan(video_id, segments, budget)
    else:
        moc_plan["video_id"] = moc_plan.get("video_id") or video_id

    chunks = chunk_segments_by_approx_tokens(
        segments,
        max_tokens=options.chunk_token_limit,
        overlap_tokens=options.chunk_overlap_tokens,
    )
    mapped_rows: list[dict[str, object]] = []
    all_facts: list[object] = []
    action_items: list[object] = []
    open_questions: list[object] = []
    map_json_warning_count = 0

    for chunk in chunks:
        messages = build_moc_map_extraction_messages(
            chunk_text=chunk.text,
            chunk_index=chunk.chunk_index,
            total_chunks=len(chunks),
            output_language=options.output_language,
        )
        response = call_llm(messages, options.max_tokens)
        payload, valid = parse_json_payload(response.text)
        if not valid:
            map_json_warning_count += 1
            retry_messages = messages + [
                ChatMessage(role="user", content="The previous response was invalid. Return valid JSON only.")
            ]
            response = call_llm(retry_messages, options.max_tokens)
            payload, valid = parse_json_payload(response.text)
        if not valid:
            json_valid = False
            payload = {"facts": [], "action_items": [], "open_questions": []}

        facts = payload.get("facts") if isinstance(payload.get("facts"), list) else []
        chunk_action_items = payload.get("action_items") if isinstance(payload.get("action_items"), list) else []
        chunk_open_questions = payload.get("open_questions") if isinstance(payload.get("open_questions"), list) else []
        all_facts.extend(facts)
        action_items.extend(chunk_action_items)
        open_questions.extend(chunk_open_questions)
        mapped_rows.append(
            {
                "chunk_index": chunk.chunk_index,
                "chunk_time_span": {"start_ms": chunk.start_ms, "end_ms": chunk.end_ms},
                "payload": payload,
            }
        )

    clusters = deduplicate_facts(all_facts)
    aligned_nodes, unaligned = align_fact_clusters_to_moc(moc_plan, clusters)
    structured = build_structured_result_from_facts(
        moc_plan,
        aligned_nodes,
        action_items=action_items,
        open_questions=open_questions,
    )
    structured_result_json = json.dumps(structured.to_dict(), ensure_ascii=False, indent=2)
    moc_json = json.dumps(moc_plan, ensure_ascii=False, indent=2)
    global_context_json = json.dumps(build_moc_global_context(moc_plan), ensure_ascii=False, indent=2)

    section_rows: list[dict[str, object]] = []
    section_summaries: list[dict[str, object]] = []
    expansion_count = 0
    slice_truncated_count = 0
    sections: list[dict[str, object]] = []
    for index, entry in enumerate(aligned_nodes, start=1):
        node = entry.get("node", {}) if isinstance(entry, dict) else {}
        aligned_clusters = entry.get("aligned_fact_clusters", []) if isinstance(entry, dict) else []
        target_word_count = node_target_word_count(node, options.chapter_target_words)
        raw_slice, slice_truncated = build_evidence_slice(
            node=node,
            clusters=aligned_clusters,
            segments=segments,
            max_slice_tokens=options.max_slice_tokens,
        )
        if slice_truncated:
            slice_truncated_count += 1
        node_json = json.dumps(node, ensure_ascii=False, indent=2)
        aligned_facts_json = json.dumps(aligned_clusters, ensure_ascii=False, indent=2)
        section_response = call_llm(
            build_moc_node_section_messages(
                node_json=node_json,
                aligned_facts_json=aligned_facts_json,
                raw_transcript_slice=raw_slice,
                global_context_json=global_context_json,
                output_language=options.output_language,
            ),
            min(options.max_tokens, max(1000, target_word_count * 3)),
        )
        section_text = section_response.text.strip()
        section_word_count = word_count(section_text)
        if section_word_count < int(0.8 * target_word_count):
            expansion_count += 1
            expanded = call_llm(
                build_moc_node_expansion_messages(
                    section_draft=section_text,
                    node_json=node_json,
                    aligned_facts_json=aligned_facts_json,
                    raw_transcript_slice=raw_slice,
                    current_word_count=section_word_count,
                    target_word_count=target_word_count,
                    output_language=options.output_language,
                ),
                min(options.max_tokens, max(1000, target_word_count * 3)),
            ).text.strip()
            if expanded:
                section_text = expanded
        if not section_text:
            section_text = str(node.get("description_outline", node.get("source_focus", ""))) if hasattr(node, "get") else ""

        title = str(node.get("title", f"Section {index}")) if hasattr(node, "get") else f"Section {index}"
        sections.append({"title": title, "content": section_text})
        section_word_count = word_count(section_text)
        section_rows.append(
            {
                "node_id": node.get("node_id") if hasattr(node, "get") else None,
                "title": title,
                "word_count": section_word_count,
                "slice_truncated": slice_truncated,
                "content": section_text,
            }
        )
        section_summaries.append(
            {
                "node_id": node.get("node_id") if hasattr(node, "get") else None,
                "title": title,
                "word_count": section_word_count,
                "first_words": " ".join(section_text.split()[:80]),
            }
        )

    section_summaries_json = json.dumps(section_summaries, ensure_ascii=False, indent=2)
    overview = call_llm(
        build_moc_overview_messages(
            moc_json=moc_json,
            structured_result_json=structured_result_json,
            section_summaries_json=section_summaries_json,
            output_language=options.output_language,
        ),
        min(options.max_tokens, 2000),
    ).text.strip()
    if not overview:
        overview = str(moc_plan.get("report_thesis", "Executive overview unavailable."))

    conclusion = call_llm(
        build_moc_conclusion_messages(
            moc_json=moc_json,
            structured_result_json=structured_result_json,
            section_summaries_json=section_summaries_json,
            output_language=options.output_language,
        ),
        min(options.max_tokens, 2000),
    ).text.strip()
    if not conclusion:
        conclusion = "Final synthesis unavailable."

    structured_markdown = {
        "timeline": markdown_timeline(structured),
        "claims": markdown_claims(structured),
        "action_items": markdown_action_items(structured),
        "open_questions": markdown_open_questions(structured),
        "unaligned_facts": markdown_unaligned_facts(unaligned),
    }
    structured.summary_text = assemble_moc_markdown_report(
        video_id=video_id,
        overview=overview,
        sections=sections,
        structured_markdown=structured_markdown,
        conclusion=conclusion,
    )
    quality_checks = {
        "map_json_warning_count": map_json_warning_count,
        "transcript_warnings": transcript_warnings,
        "slice_truncated_node_count": slice_truncated_count,
        "node_expansion_count": expansion_count,
        "actual_report_words": len(structured.summary_text.split()),
    }

    return StrategyOutcome(
        result=structured,
        request_count=request_count,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        latency_seconds=latency_seconds,
        json_valid=json_valid,
        raw_requests=raw_requests,
        raw_responses=raw_responses,
        extra_metrics={
            "strategy_variant": "moc_guided_map_reduce",
            "transcript_words": transcript_words,
            "report_min_words": budget.report_min_words,
            "report_max_words": budget.report_max_words,
            "target_report_words": budget.target_report_words,
            "actual_report_words": len(structured.summary_text.split()),
            "estimated_transcript_tokens": estimated_tokens,
            "moc_node_count": len(moc_plan.get("nodes", [])) if isinstance(moc_plan.get("nodes"), list) else 0,
            "moc_fallback_used": moc_fallback_used,
            "moc_projection_used": moc_projection_used,
            "map_chunk_count": len(chunks),
            "extracted_fact_count": len(all_facts),
            "deduplicated_fact_count": len(clusters),
            "aligned_fact_count": sum(len(row["aligned_fact_clusters"]) for row in aligned_nodes),
            "unaligned_fact_count": len(unaligned),
            "node_expansion_count": expansion_count,
            "slice_truncated_node_count": slice_truncated_count,
            "parallelism_enabled": False,
            "parallelizable_map_call_count": len(chunks),
            "parallelizable_node_call_count": len(aligned_nodes),
            "max_parallel_map_calls": options.max_parallel_map_calls,
            "max_parallel_node_calls": options.max_parallel_node_calls,
            "coverage_warnings": transcript_warnings,
            "map_json_warning_count": map_json_warning_count,
        },
        extra_artifacts={
            "moc.json": moc_plan,
            "mapped_facts.jsonl": jsonl_rows(mapped_rows),
            "deduplicated_facts.json": clusters,
            "alignment.json": {"aligned_nodes": aligned_nodes, "unaligned_facts": unaligned},
            "node_sections.jsonl": jsonl_rows(section_rows),
            "quality_checks.json": quality_checks,
        },
    )


STRATEGIES = {
    "adaptive_book_report": run_adaptive_book_report,
    "antigravity_chunk_map_reduce": run_antigravity_chunk_map_reduce,
    "one_shot_full_json": run_one_shot_full_json,
    "one_shot_markdown_plus_json": run_one_shot_markdown_plus_json,
    "two_pass_summary_structure": run_two_pass_summary_structure,
    "chunk_map_reduce": run_chunk_map_reduce,
    "moc_guided_map_reduce": run_moc_guided_map_reduce,
    "timeline_segment_reduce": run_timeline_segment_reduce,
}
