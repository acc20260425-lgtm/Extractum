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

TIMESTAMP_RE = re.compile(
    r"^\[?(?:(\d{1,2}):)?(\d{2}):(\d{2})(?:[.,](\d{1,3}))?\]?(?=\s|$)"
)
TIMESTAMP_RANGE_ARROW_RE = re.compile(r"^\s*-->\s*")
TOKEN_RE = re.compile(r"\w+|[^\w\s]", re.UNICODE)


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
                if window_segments:
                    windows.append(_projection_window(len(windows) + 1, window_segments))
                    window_segments = []
                    window_word_count = 0
                window_start = window_end
                window_end = window_start + window_ms

            segment_word_count = word_count(segment.text)
            if (
                window_segments
                and window_word_count + segment_word_count > PROJECTION_WINDOW_MAX_WORDS
            ):
                windows.append(_projection_window(len(windows) + 1, window_segments))
                window_segments = []
                window_word_count = 0
                window_start = segment_start if segment_start is not None else segment.end_ms or window_end
                window_end = window_start + window_ms

            window_segments.append(segment)
            window_word_count += segment_word_count

        if window_segments:
            windows.append(_projection_window(len(windows) + 1, window_segments))

    return {
        "projection_kind": "temporal_skeleton",
        "source_word_count": source_word_count,
        "source_segment_count": len(segments),
        "window_ms": window_ms,
        "windows": windows,
    }


def _projection_window(window_index: int, segments: list[TranscriptSegment]) -> dict[str, object]:
    start_ms, end_ms = chunk_time_span(segments)
    text = " ".join(segment.text for segment in segments)
    return {
        "window_id": f"window_{window_index:03d}",
        "start_ms": start_ms,
        "end_ms": end_ms,
        "segment_count": len(segments),
        "word_count": word_count(text),
        "first_words": first_words(text, 80),
        "last_words": last_words(text, 80),
        "sampled_timestamped_lines": format_segments_for_prompt(segments[:3]).splitlines(),
    }


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
