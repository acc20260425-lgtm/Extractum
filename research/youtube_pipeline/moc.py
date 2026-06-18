from __future__ import annotations

from dataclasses import dataclass
import re


TIMESTAMP_RE = re.compile(
    r"^(?:\[(\d{1,2}):(\d{2})(?::(\d{2}))?\]|(\d{1,2}):(\d{2})(?::(\d{2}))?)(?=\s)"
)


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


def parse_timestamp_ms(line: str) -> tuple[int | None, str]:
    match = TIMESTAMP_RE.match(line)
    if not match:
        return None, line

    parts = [part for part in match.groups()[:3] if part is not None]
    if not parts:
        parts = [part for part in match.groups()[3:] if part is not None]

    if len(parts) == 2:
        hours = 0
        minutes, seconds = (int(part) for part in parts)
    else:
        hours, minutes, seconds = (int(part) for part in parts)

    start_ms = ((hours * 60 + minutes) * 60 + seconds) * 1000
    return start_ms, line[match.end() :].lstrip()


def parse_timestamped_transcript(transcript: str) -> tuple[list[TranscriptSegment], list[str]]:
    segments: list[TranscriptSegment] = []
    has_missing_timestamps = False

    for raw_line in transcript.splitlines():
        line = raw_line.strip()
        if not line:
            continue

        start_ms, text = parse_timestamp_ms(line)
        if start_ms is None:
            has_missing_timestamps = True

        segments.append(
            TranscriptSegment(
                segment_id=f"seg_{len(segments) + 1:06d}",
                start_ms=start_ms,
                end_ms=None,
                speaker=None,
                text=text,
            )
        )

    following_start_ms: int | None = None
    for segment in reversed(segments):
        if segment.start_ms is None:
            segment.end_ms = None
            continue
        segment.end_ms = following_start_ms if following_start_ms is not None else segment.start_ms
        following_start_ms = segment.start_ms

    warnings = ["missing_timestamps"] if has_missing_timestamps else []
    return segments, warnings


def word_count(text: str) -> int:
    return len(text.split())


def approximate_token_count(text: str) -> int:
    token_count = 0
    in_ascii_word = False

    for character in text:
        if character.isspace():
            in_ascii_word = False
        elif character.isascii() and character.isalnum():
            if not in_ascii_word:
                token_count += 1
                in_ascii_word = True
        else:
            token_count += 1
            in_ascii_word = False

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
            segment_tokens = approximate_token_count(segments[cursor].text)
            if chunk_segments and token_total + segment_tokens > max_tokens:
                break
            chunk_segments.append(segments[cursor])
            token_total += segment_tokens
            cursor += 1

        chunks.append(make_segment_chunk(len(chunks), chunk_segments))
        if cursor >= len(segments):
            break

        next_start = cursor
        overlap_total = 0
        for index in range(cursor - 1, start_index - 1, -1):
            if overlap_total >= overlap_tokens:
                break
            overlap_total += approximate_token_count(segments[index].text)
            next_start = index

        if next_start <= start_index:
            next_start = start_index + 1
        start_index = next_start

    return chunks
