from __future__ import annotations

from dataclasses import dataclass
import math
import re


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
