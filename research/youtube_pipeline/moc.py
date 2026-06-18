from __future__ import annotations

from dataclasses import dataclass
import math
import re
from typing import Any


MAX_REPORT_WORDS = 20000
MAX_MOC_NODES = 20
MIN_NODE_WORDS = 500
FALLBACK_NODE_TARGET_WORDS = 900
PROJECTION_WINDOW_MAX_WORDS = 2000
IMPORTANCE_RANK = {"low": 1, "medium": 2, "high": 3}

TIMESTAMP_RE = re.compile(
    r"^\[?(?:(\d{1,2}):)?(\d{2}):(\d{2})(?:[.,](\d{1,3}))?\]?(?=\s|$)"
)
TIMESTAMP_RANGE_ARROW_RE = re.compile(r"^\s*-->\s*")
TOKEN_RE = re.compile(r"\w+|[^\w\s]", re.UNICODE)
FACT_WORD_RE = re.compile(r"\w+", re.UNICODE)


@dataclass
class TranscriptSegment:
    segment_id: str
    start_ms: int | None
    end_ms: int | None
    speaker: str | None
    text: str


@dataclass
class SegmentChunk:
    chunk_index: int
    segments: list[TranscriptSegment]
    start_ms: int | None
    end_ms: int | None
    text: str


@dataclass
class MocBudget:
    transcript_words: int
    report_min_words: int
    report_max_words: int
    target_report_words: int
    expected_node_min: int
    expected_node_max: int


def normalize_importance(value: Any) -> str:
    if not isinstance(value, str):
        return "low"
    normalized = value.strip().lower()
    if normalized in IMPORTANCE_RANK:
        return normalized
    return "low"


def normalize_fact_key(text: str) -> str:
    if not isinstance(text, str):
        return ""
    return " ".join(FACT_WORD_RE.findall(text.lower()))


def normalize_term_set(values: Any) -> set[str]:
    if isinstance(values, str):
        candidates = [values]
    elif isinstance(values, list | tuple | set):
        candidates = list(values)
    else:
        return set()

    terms: set[str] = set()
    for value in candidates:
        if not isinstance(value, str):
            continue
        normalized = normalize_fact_key(value)
        if normalized:
            terms.add(normalized)
    return terms


def _coerce_ms(value: Any) -> int | None:
    if isinstance(value, bool) or not isinstance(value, int | float):
        return None
    return int(value)


def time_span_from_dict(payload: Any) -> dict[str, int | None] | None:
    if not hasattr(payload, "get"):
        return None

    start_ms = _coerce_ms(payload.get("start_ms"))
    end_ms = _coerce_ms(payload.get("end_ms"))
    if start_ms is None and end_ms is None:
        return None
    if start_ms is not None and end_ms is not None and end_ms < start_ms:
        start_ms, end_ms = end_ms, start_ms
    return {"start_ms": start_ms, "end_ms": end_ms}


def merge_time_span(a: Any, b: Any) -> dict[str, int | None] | None:
    spans = [span for span in (time_span_from_dict(a), time_span_from_dict(b)) if span is not None]
    if not spans:
        return None

    starts = [span["start_ms"] for span in spans if span["start_ms"] is not None]
    ends = [span["end_ms"] for span in spans if span["end_ms"] is not None]
    if not starts and not ends:
        return None
    return {
        "start_ms": min(starts) if starts else None,
        "end_ms": max(ends) if ends else None,
    }


def _normalize_kind(value: Any) -> str:
    if not isinstance(value, str):
        return "unknown"
    normalized = normalize_fact_key(value)
    return normalized or "unknown"


def _fact_text(fact: Any) -> str:
    if not hasattr(fact, "get"):
        return ""
    value = fact.get("text")
    return value if isinstance(value, str) else ""


def fact_to_cluster(index: int, fact: Any) -> dict[str, object]:
    text = _fact_text(fact)
    time_span = time_span_from_dict(fact.get("time_span")) if hasattr(fact, "get") else None
    entities = sorted(normalize_term_set(fact.get("entities") if hasattr(fact, "get") else None))
    topic_tags = sorted(normalize_term_set(fact.get("topic_tags") if hasattr(fact, "get") else None))
    kind = _normalize_kind(fact.get("kind") if hasattr(fact, "get") else None)
    importance = normalize_importance(fact.get("importance") if hasattr(fact, "get") else None)
    mention = {
        "fact_id": fact.get("fact_id") if hasattr(fact, "get") else None,
        "text": text,
        "kind": kind,
        "time_span": time_span,
        "verbatim_quote": fact.get("verbatim_quote") if hasattr(fact, "get") else None,
        "importance": importance,
        "entities": entities,
        "topic_tags": topic_tags,
    }
    return {
        "cluster_id": f"cluster_{index + 1:06d}",
        "canonical_text": text,
        "kind": kind,
        "importance": importance,
        "time_span": time_span,
        "entities": entities,
        "topic_tags": topic_tags,
        "mentions": [mention],
    }


def _text_terms(text: Any) -> set[str]:
    if not isinstance(text, str):
        return set()
    return set(normalize_fact_key(text).split())


def _terms_from_similarity_input(value: Any) -> set[Any]:
    if isinstance(value, str):
        return _text_terms(value)
    if value is None:
        return set()
    try:
        return set(value)
    except TypeError:
        return set()


def jaccard_similarity(a: Any, b: Any) -> float:
    left = _terms_from_similarity_input(a)
    right = _terms_from_similarity_input(b)
    if not left and not right:
        return 1.0
    if not left or not right:
        return 0.0
    return len(left & right) / len(left | right)


def spans_near(a: Any, b: Any, max_distance_ms: int = 60000) -> bool:
    left = time_span_from_dict(a)
    right = time_span_from_dict(b)
    if left is None or right is None:
        return True

    left_start = left["start_ms"] if left["start_ms"] is not None else left["end_ms"]
    left_end = left["end_ms"] if left["end_ms"] is not None else left["start_ms"]
    right_start = right["start_ms"] if right["start_ms"] is not None else right["end_ms"]
    right_end = right["end_ms"] if right["end_ms"] is not None else right["start_ms"]
    if left_start is None or left_end is None or right_start is None or right_end is None:
        return True
    if left_end < right_start:
        return right_start - left_end <= max_distance_ms
    if right_end < left_start:
        return left_start - right_end <= max_distance_ms
    return True


def same_fact_cluster(a: Any, b: Any) -> bool:
    left_text = a.get("canonical_text", "") if hasattr(a, "get") else ""
    right_text = b.get("canonical_text", "") if hasattr(b, "get") else ""
    left_key = normalize_fact_key(left_text)
    right_key = normalize_fact_key(right_text)
    if left_key and left_key == right_key:
        return True
    return (
        jaccard_similarity(left_key, right_key) >= 0.88
        and spans_near(
            a.get("time_span") if hasattr(a, "get") else None,
            b.get("time_span") if hasattr(b, "get") else None,
        )
    )


def _merge_cluster_into(target: dict[str, object], source: dict[str, object]) -> None:
    target["time_span"] = merge_time_span(target.get("time_span"), source.get("time_span"))
    if IMPORTANCE_RANK[normalize_importance(source.get("importance"))] > IMPORTANCE_RANK[
        normalize_importance(target.get("importance"))
    ]:
        target["importance"] = normalize_importance(source.get("importance"))
    target["entities"] = sorted(set(target.get("entities", [])) | set(source.get("entities", [])))
    target["topic_tags"] = sorted(set(target.get("topic_tags", [])) | set(source.get("topic_tags", [])))
    target["mentions"] = list(target.get("mentions", [])) + list(source.get("mentions", []))


def deduplicate_facts(facts: Any) -> list[dict[str, object]]:
    clusters: list[dict[str, object]] = []
    for fact in facts or []:
        candidate = fact_to_cluster(len(clusters), fact)
        matching_cluster = next(
            (cluster for cluster in clusters if same_fact_cluster(cluster, candidate)),
            None,
        )
        if matching_cluster is None:
            clusters.append(candidate)
        else:
            _merge_cluster_into(matching_cluster, candidate)

    for index, cluster in enumerate(clusters, start=1):
        cluster["cluster_id"] = f"cluster_{index:06d}"
    return clusters


def time_overlap_score(node_span: Any, fact_span: Any) -> float:
    node = time_span_from_dict(node_span)
    fact = time_span_from_dict(fact_span)
    if node is None or fact is None:
        return 0.0

    node_start = node["start_ms"] if node["start_ms"] is not None else node["end_ms"]
    node_end = node["end_ms"] if node["end_ms"] is not None else node["start_ms"]
    fact_start = fact["start_ms"] if fact["start_ms"] is not None else fact["end_ms"]
    fact_end = fact["end_ms"] if fact["end_ms"] is not None else fact["start_ms"]
    if node_start is None or node_end is None or fact_start is None or fact_end is None:
        return 0.0
    if fact_start == fact_end:
        return 1.0 if node_start <= fact_start <= node_end else 0.0

    overlap = max(0, min(node_end, fact_end) - max(node_start, fact_start))
    fact_duration = max(1, fact_end - fact_start)
    if overlap > 0:
        return min(1.0, overlap / fact_duration)
    if node_end == fact_start or fact_end == node_start:
        return 0.5
    return 0.0


def term_overlap_score(node_terms: Any, fact_terms: Any) -> float:
    node_set = set(node_terms or [])
    fact_set = set(fact_terms or [])
    if not node_set or not fact_set:
        return 0.0
    return len(node_set & fact_set) / len(node_set)


def _cluster_terms(cluster: Any) -> set[str]:
    if not hasattr(cluster, "get"):
        return set()
    terms = _text_terms(cluster.get("canonical_text"))
    terms |= normalize_term_set(cluster.get("entities"))
    terms |= normalize_term_set(cluster.get("topic_tags"))
    return terms


def _node_terms(node: Any, global_terms: Any = None) -> set[str]:
    if not hasattr(node, "get"):
        return set()
    terms = _text_terms(node.get("title"))
    terms |= normalize_term_set(node.get("key_terms"))
    terms |= normalize_term_set(node.get("essential_key_terms"))
    return terms


def _global_term_score(global_terms: Any, cluster_terms: set[str]) -> float:
    normalized_global_terms = normalize_term_set(global_terms)
    if not normalized_global_terms or not cluster_terms:
        return 0.0
    return len(normalized_global_terms & cluster_terms) / len(normalized_global_terms)


def _alignment_node_evidence(node: Any, cluster: Any) -> tuple[float, float]:
    node_terms = _node_terms(node)
    cluster_terms = _cluster_terms(cluster)
    node_terms_score = term_overlap_score(node_terms, cluster_terms)
    time = time_overlap_score(
        node.get("time_span") if hasattr(node, "get") else None,
        cluster.get("time_span") if hasattr(cluster, "get") else None,
    )
    return node_terms_score, time


def alignment_score(node: Any, cluster: Any, global_terms: Any) -> float:
    node_terms_score, time = _alignment_node_evidence(node, cluster)
    cluster_terms = _cluster_terms(cluster)
    if node_terms_score <= 0 and time <= 0:
        return 0.0
    global_bonus = _global_term_score(global_terms, cluster_terms)
    return (node_terms_score * 0.60) + (time * 0.35) + (global_bonus * 0.05)


def align_fact_clusters_to_moc(
    moc_plan: Any,
    clusters: Any,
    threshold: float = 0.30,
) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    nodes = list(moc_plan.get("nodes", []) if hasattr(moc_plan, "get") else [])
    global_terms = moc_plan.get("global_key_terms", []) if hasattr(moc_plan, "get") else []
    aligned_nodes = [{"node": node, "aligned_fact_clusters": []} for node in nodes]
    unaligned: list[dict[str, object]] = []

    for cluster in clusters or []:
        best_index = None
        best_score = -1.0
        for index, node in enumerate(nodes):
            node_terms_score, time = _alignment_node_evidence(node, cluster)
            if node_terms_score <= 0 and time <= 0:
                continue
            score = alignment_score(node, cluster, global_terms)
            if score > best_score:
                best_index = index
                best_score = score
        if best_index is None or best_score < threshold:
            unaligned.append(cluster)
        else:
            aligned_nodes[best_index]["aligned_fact_clusters"].append(cluster)

    return aligned_nodes, unaligned


def _timestamp_match_ms(match: re.Match[str]) -> int:
    hours_text, minutes_text, seconds_text, fraction_text = match.groups()
    hours = int(hours_text or 0)
    minutes = int(minutes_text)
    seconds = int(seconds_text)
    milliseconds = int((fraction_text or "0").ljust(3, "0")[:3])
    return ((hours * 60 + minutes) * 60 + seconds) * 1000 + milliseconds


def parse_timestamp_ms(line: str) -> tuple[int | None, int | None, str]:
    match = TIMESTAMP_RE.match(line)
    if not match:
        return None, None, line

    start_ms = _timestamp_match_ms(match)
    end_ms = None
    text = line[match.end() :]
    arrow_match = TIMESTAMP_RANGE_ARROW_RE.match(text)
    if arrow_match:
        range_text = text[arrow_match.end() :]
        end_match = TIMESTAMP_RE.match(range_text)
        if end_match:
            end_ms = _timestamp_match_ms(end_match)
            text = range_text[end_match.end() :]
    return start_ms, end_ms, text.lstrip()


def parse_timestamped_transcript(transcript: str) -> tuple[list[TranscriptSegment], list[str]]:
    segments: list[TranscriptSegment] = []
    has_missing_timestamps = False

    for raw_line in transcript.splitlines():
        line = raw_line.strip()
        if not line:
            continue

        start_ms, end_ms, text = parse_timestamp_ms(line)
        if start_ms is None:
            has_missing_timestamps = True

        segments.append(
            TranscriptSegment(
                segment_id=f"seg_{len(segments) + 1:06d}",
                start_ms=start_ms,
                end_ms=end_ms,
                speaker=None,
                text=text,
            )
        )

    following_start_ms: int | None = None
    for segment in reversed(segments):
        if segment.start_ms is None:
            segment.end_ms = None
            continue
        if segment.end_ms is None:
            segment.end_ms = following_start_ms if following_start_ms is not None else segment.start_ms
        following_start_ms = segment.start_ms

    warnings = ["missing_timestamps"] if has_missing_timestamps else []
    return segments, warnings


def word_count(text: str) -> int:
    return len(text.split())


def first_words(text: str, limit: int) -> str:
    return " ".join(text.split()[:limit])


def last_words(text: str, limit: int) -> str:
    if limit <= 0:
        return ""
    return " ".join(text.split()[-limit:])


def approximate_token_count(text: str) -> int:
    token_count = 0

    for token in TOKEN_RE.findall(text):
        if any(character.isalnum() or character == "_" for character in token):
            token_count += max(1, math.ceil(len(token) / 4))
        else:
            token_count += 1

    return token_count


def chunk_time_span(segments: list[TranscriptSegment]) -> tuple[int | None, int | None]:
    start_ms = next((segment.start_ms for segment in segments if segment.start_ms is not None), None)
    end_ms = next((segment.end_ms for segment in reversed(segments) if segment.end_ms is not None), None)
    return start_ms, end_ms


def format_ms(ms: int | None) -> str:
    if ms is None:
        return "--:--:--"

    total_seconds = ms // 1000
    hours = total_seconds // 3600
    minutes = (total_seconds % 3600) // 60
    seconds = total_seconds % 60
    return f"{hours:02d}:{minutes:02d}:{seconds:02d}"


def format_segments_for_prompt(segments: list[TranscriptSegment]) -> str:
    return "\n".join(f"[{format_ms(segment.start_ms)}] {segment.text}" for segment in segments)


def select_moc_base(transcript_words: int) -> tuple[int, int, int, int]:
    if transcript_words < 5000:
        return 1000, 1800, 1, 2
    if transcript_words < 15000:
        return 2000, 3500, 2, 4
    if transcript_words < 35000:
        return 4000, 6500, 4, 7
    if transcript_words < 70000:
        return 7000, 10000, 8, 12
    return 10000, 14000, 12, 18


def moc_depth_multiplier(target_depth: str) -> float:
    return {
        "auto": 1.0,
        "brief": 0.5,
        "standard": 1.0,
        "deep": 1.5,
        "book": 2.0,
    }.get(target_depth, 1.0)


def compute_moc_budget(transcript_words: int, options: Any) -> MocBudget:
    base_min, base_max, base_node_min, base_node_max = select_moc_base(transcript_words)
    multiplier = moc_depth_multiplier(getattr(options, "target_depth", "auto"))

    min_override = getattr(options, "min_report_words", None)
    max_override = getattr(options, "max_report_words", None)
    scaled_min = max(MIN_NODE_WORDS, round(base_min * multiplier))
    scaled_max = round(base_max * multiplier)
    if min_override is not None and int(min_override) <= 0:
        raise ValueError("min_report_words must be positive")
    if max_override is not None and int(max_override) <= 0:
        raise ValueError("max_report_words must be positive")
    report_min_words = int(min_override) if min_override is not None else scaled_min
    report_max_words = int(max_override) if max_override is not None else scaled_max
    report_min_words = min(report_min_words, MAX_REPORT_WORDS)
    report_max_words = min(report_max_words, MAX_REPORT_WORDS)

    if report_min_words > report_max_words:
        raise ValueError("min_report_words cannot be greater than max_report_words")

    expected_node_min = base_node_min
    expected_node_max = min(base_node_max, MAX_MOC_NODES)

    return MocBudget(
        transcript_words=transcript_words,
        report_min_words=report_min_words,
        report_max_words=report_max_words,
        target_report_words=round((report_min_words + report_max_words) / 2),
        expected_node_min=expected_node_min,
        expected_node_max=expected_node_max,
    )


def build_temporal_projection(
    segments: list[TranscriptSegment],
    *,
    source_word_count: int,
    window_ms: int = 300000,
) -> dict[str, object]:
    if window_ms <= 0:
        raise ValueError("window_ms must be positive")

    timestamped_segments = [
        segment for segment in segments if segment.start_ms is not None or segment.end_ms is not None
    ]
    windows: list[dict[str, object]] = []
    if timestamped_segments:
        window_segments: list[TranscriptSegment] = []
        window_word_count = 0
        window_start = timestamped_segments[0].start_ms
        if window_start is None:
            window_start = timestamped_segments[0].end_ms or 0
        window_end = window_start + window_ms

        for segment in timestamped_segments:
            segment_start = segment.start_ms if segment.start_ms is not None else segment.end_ms
            while segment_start is not None and segment_start >= window_end:
                windows.append(
                    _projection_window(
                        len(windows) + 1,
                        window_start,
                        window_end,
                        window_segments,
                    )
                )
                window_segments = []
                window_word_count = 0
                window_start = window_end
                window_end = window_start + window_ms

            segment_word_count = word_count(segment.text)
            if (
                window_segments
                and window_word_count + segment_word_count > PROJECTION_WINDOW_MAX_WORDS
            ):
                split_end = segment_start if segment_start is not None else window_end
                windows.append(
                    _projection_window(
                        len(windows) + 1,
                        window_start,
                        split_end,
                        window_segments,
                    )
                )
                window_segments = []
                window_word_count = 0
                window_start = split_end
                window_end = window_start + window_ms

            window_segments.append(segment)
            window_word_count += segment_word_count

        if window_segments:
            windows.append(
                _projection_window(
                    len(windows) + 1,
                    window_start,
                    window_end,
                    window_segments,
                )
            )

    return {
        "projection_kind": "temporal_skeleton",
        "source_word_count": source_word_count,
        "source_segment_count": len(segments),
        "window_ms": window_ms,
        "windows": windows,
    }


def _projection_window(
    window_index: int,
    start_ms: int | None,
    end_ms: int | None,
    segments: list[TranscriptSegment],
) -> dict[str, object]:
    text = " ".join(segment.text for segment in segments)
    return {
        "window_id": f"window_{window_index:03d}",
        "start_ms": start_ms,
        "end_ms": end_ms,
        "segment_count": len(segments),
        "word_count": word_count(text),
        "first_words": first_words(text, 80),
        "last_words": last_words(text, 80),
        "sampled_timestamped_lines": _sample_timestamped_lines(segments[:3]),
    }


def _sample_timestamped_lines(segments: list[TranscriptSegment]) -> list[str]:
    return [f"[{format_ms(segment.start_ms)}] {first_words(segment.text, 80)}" for segment in segments]


def split_budget_evenly(total: int, count: int) -> list[int]:
    if count <= 0:
        return []
    base = total // count
    remainder = total % count
    return [base + (1 if index < remainder else 0) for index in range(count)]


def fallback_moc_plan(
    video_id: str,
    segments: list[TranscriptSegment],
    budget: MocBudget,
) -> dict[str, object]:
    node_count = max(
        1,
        min(
            MAX_MOC_NODES,
            round(budget.target_report_words / FALLBACK_NODE_TARGET_WORDS),
        ),
    )
    partitions = _partition_segments_evenly(segments, node_count)
    targets = split_budget_evenly(budget.target_report_words, len(partitions))
    nodes: list[dict[str, object]] = []
    last_time_ms = chunk_time_span(segments)[0]

    for index, node_segments in enumerate(partitions, start=1):
        start_ms, end_ms = chunk_time_span(node_segments)
        if start_ms is None and end_ms is None:
            start_ms = last_time_ms
            end_ms = last_time_ms
        else:
            last_time_ms = end_ms if end_ms is not None else start_ms
        text = " ".join(segment.text for segment in node_segments)
        nodes.append(
            {
                "node_id": f"node_{index:03d}",
                "title": f"Section {index}",
                "time_span": {"start_ms": start_ms, "end_ms": end_ms},
                "importance": "high" if index == 1 else "medium",
                "target_word_count": targets[index - 1],
                "description_outline": first_words(text, 60),
                "essential_key_terms": [],
                "required_questions": [],
                "expected_fact_types": ["claims", "evidence", "timeline"],
            }
        )

    return {
        "video_id": video_id,
        "projection_kind": "fallback_time_windows",
        "target_report_words": budget.target_report_words,
        "nodes": nodes,
    }


def _partition_segments_evenly(
    segments: list[TranscriptSegment],
    count: int,
) -> list[list[TranscriptSegment]]:
    count = max(1, count)
    if not segments:
        return [[] for _ in range(count)]
    if count >= len(segments):
        return [[segment] for segment in segments] + [[] for _ in range(count - len(segments))]
    partitions: list[list[TranscriptSegment]] = []
    for index in range(count):
        start = index * len(segments) // count
        end = (index + 1) * len(segments) // count
        partitions.append(segments[start:end])
    return partitions


def _segment_emitted_token_count(segment: TranscriptSegment) -> int:
    text = f"[{format_ms(segment.start_ms)}] {segment.text}"
    return approximate_token_count(text) + 1


def make_segment_chunk(index: int, segments: list[TranscriptSegment]) -> SegmentChunk:
    start_ms, end_ms = chunk_time_span(segments)
    return SegmentChunk(
        chunk_index=index,
        segments=list(segments),
        start_ms=start_ms,
        end_ms=end_ms,
        text=format_segments_for_prompt(segments),
    )


def chunk_segments_by_approx_tokens(
    segments: list[TranscriptSegment],
    *,
    max_tokens: int,
    overlap_tokens: int,
) -> list[SegmentChunk]:
    if max_tokens <= 0:
        raise ValueError("max_tokens must be positive")
    if overlap_tokens < 0:
        raise ValueError("overlap_tokens must be non-negative")

    chunks: list[SegmentChunk] = []
    start_index = 0

    while start_index < len(segments):
        chunk_segments: list[TranscriptSegment] = []
        token_total = 0
        cursor = start_index

        while cursor < len(segments):
            segment_tokens = _segment_emitted_token_count(segments[cursor])
            if chunk_segments and token_total + segment_tokens > max_tokens:
                break
            chunk_segments.append(segments[cursor])
            token_total += segment_tokens
            cursor += 1

        chunks.append(make_segment_chunk(len(chunks) + 1, chunk_segments))
        if cursor >= len(segments):
            break

        next_start = cursor
        overlap_total = 0
        for index in range(cursor - 1, start_index - 1, -1):
            if overlap_total >= overlap_tokens:
                break
            overlap_total += _segment_emitted_token_count(segments[index])
            next_start = index

        if next_start <= start_index:
            next_start = start_index + 1
        start_index = next_start

    return chunks
