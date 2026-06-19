# YouTube MoC-Guided Map-Reduce Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Python research strategy `moc_guided_map_reduce` that creates a timestamp-grounded long-form report using a global Map of Content plus fact-level Map-Reduce.

**Architecture:** Add focused MoC helper modules for transcript segments, planning artifacts, fact extraction structures, deduplication, alignment, node assembly, and quality checks. Register a new strategy that orchestrates MoC planning, overlapping map extraction, fact dedupe/alignment, node section generation with expansion, structured result derivation, overview/conclusion, extra artifacts, and metrics. Keep existing strategies working through the current `StrategyOptions` interface.

**Tech Stack:** Python standard library, `unittest`, existing OpenAI-compatible `LlmClient`, existing `NormalizedResult`, existing research runner artifact format.

---

## File Structure

Create:

- `research/youtube_pipeline/moc.py`
  - Pure Python data structures and deterministic helpers for `moc_guided_map_reduce`.
  - Responsibilities: transcript parsing, segment chunking, projection, budget selection, MoC parsing/fallback, fact normalization/dedupe, alignment, evidence slice building, structured result derivation, markdown assembly, quality checks.

- `research/youtube_pipeline/tests/test_moc.py`
  - Unit tests for deterministic helpers in `moc.py`.

Modify:

- `research/youtube_pipeline/strategies.py`
  - Extend `StrategyOptions`.
  - Extend `StrategyOutcome` with `extra_artifacts`.
  - Add `run_moc_guided_map_reduce`.
  - Register `moc_guided_map_reduce`.

- `research/youtube_pipeline/prompts.py`
  - Add MoC-specific prompt builders.

- `research/youtube_pipeline/runner.py`
  - Add CLI flags required by the spec.
  - Write `extra_artifacts` files.

- `research/youtube_pipeline/tests/test_strategies.py`
  - Add orchestration tests for the new strategy.

- `research/youtube_pipeline/tests/test_runner.py`
  - Add CLI and artifact serialization tests.

- `research/youtube_pipeline/README.md`
  - Document the new strategy and useful run command.

Do not modify production Rust/Tauri code.

---

### Task 1: Extend Strategy Contracts and Runner Artifacts

**Files:**
- Modify: `research/youtube_pipeline/strategies.py`
- Modify: `research/youtube_pipeline/runner.py`
- Test: `research/youtube_pipeline/tests/test_runner.py`

- [ ] **Step 1: Write failing tests for new options and extra artifacts**

Add these tests to `research/youtube_pipeline/tests/test_runner.py`:

```python
    def test_build_strategy_options_reads_moc_cli_flags(self):
        parser = build_parser()
        args = parser.parse_args(
            [
                "--input",
                "input.txt",
                "--video-id",
                "video1",
                "--strategy",
                "chunk_map_reduce",
                "--chunk-overlap-tokens",
                "700",
                "--planner-context-token-limit",
                "120000",
                "--max-slice-tokens",
                "8000",
            ]
        )

        options = build_strategy_options(args)

        self.assertEqual(options.chunk_overlap_tokens, 700)
        self.assertEqual(options.planner_context_token_limit, 120000)
        self.assertEqual(options.max_slice_tokens, 8000)
        self.assertEqual(options.video_id, "video1")

    def test_write_run_artifacts_writes_extra_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
                extra_artifacts={
                    "moc.json": {"nodes": []},
                    "node_sections.jsonl": '{"node_id":"node_001"}\n',
                },
            )

            output_dir = write_run_artifacts(
                root=Path(tmp),
                strategy="moc_guided_map_reduce",
                video_id="video1",
                outcome=outcome,
            )

            self.assertEqual(
                json.loads((output_dir / "moc.json").read_text(encoding="utf-8")),
                {"nodes": []},
            )
            self.assertEqual(
                (output_dir / "node_sections.jsonl").read_text(encoding="utf-8"),
                '{"node_id":"node_001"}\n',
            )
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: FAIL because `StrategyOptions` does not have the new fields and `StrategyOutcome` does not accept `extra_artifacts`.

- [ ] **Step 3: Extend `StrategyOptions` and `StrategyOutcome`**

In `research/youtube_pipeline/strategies.py`, update the dataclasses:

```python
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
```

- [ ] **Step 4: Extend runner CLI and artifact writing**

In `research/youtube_pipeline/runner.py`, add parser arguments:

```python
    parser.add_argument("--chunk-overlap-tokens", type=int, default=700)
    parser.add_argument("--planner-context-token-limit", type=int, default=120000)
    parser.add_argument("--max-slice-tokens", type=int, default=8000)
```

Update `build_strategy_options()`:

```python
    return StrategyOptions(
        output_language=args.output_language,
        video_id=args.video_id,
        max_tokens=args.max_tokens,
        chunk_token_limit=args.chunk_token_limit,
        chunk_overlap_tokens=args.chunk_overlap_tokens,
        target_depth=args.target_depth,
        min_report_words=args.min_report_words,
        max_report_words=args.max_report_words,
        chapter_target_words=args.chapter_target_words,
        planner_context_token_limit=args.planner_context_token_limit,
        max_slice_tokens=args.max_slice_tokens,
    )
```

Add this helper near `write_jsonl()`:

```python
def write_extra_artifact(path: Path, payload: Any) -> None:
    if path.name != str(path) or path.name in {"", ".", ".."}:
        raise ValueError(f"extra artifact filename must be a simple relative name: {path}")
    if isinstance(payload, str):
        path.write_text(payload, encoding="utf-8")
    else:
        write_json(path, payload)
```

At the end of `write_run_artifacts()`, before `return output_dir`, add:

```python
    for filename, payload in outcome.extra_artifacts.items():
        write_extra_artifact(output_dir / filename, payload)
```

- [ ] **Step 5: Run tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add research/youtube_pipeline/strategies.py research/youtube_pipeline/runner.py research/youtube_pipeline/tests/test_runner.py
git commit -m "feat: extend youtube research strategy options"
```

---

### Task 2: Add Transcript Segment Parser and Overlapping Chunker

**Files:**
- Create: `research/youtube_pipeline/moc.py`
- Create: `research/youtube_pipeline/tests/test_moc.py`

- [ ] **Step 1: Write failing tests for segment parsing and overlapping chunks**

Create `research/youtube_pipeline/tests/test_moc.py`:

```python
import unittest

from research.youtube_pipeline.moc import (
    TranscriptSegment,
    approximate_token_count,
    chunk_segments_by_approx_tokens,
    format_segments_for_prompt,
    parse_timestamped_transcript,
)


class MocTranscriptTests(unittest.TestCase):
    def test_parse_timestamped_transcript_accepts_supported_formats(self):
        transcript = "\n".join(
            [
                "[00:01:02] first line",
                "[01:02:03] second line",
                "02:03 third line",
                "01:02:03 fourth line",
                "untimed line",
            ]
        )

        segments, warnings = parse_timestamped_transcript(transcript)

        self.assertEqual([segment.start_ms for segment in segments], [62000, 3723000, 123000, 3723000, None])
        self.assertEqual(segments[0].segment_id, "seg_000001")
        self.assertEqual(segments[4].text, "untimed line")
        self.assertEqual(warnings, ["missing_timestamps"])

    def test_chunk_segments_by_approx_tokens_preserves_overlap_and_time_span(self):
        segments = [
            TranscriptSegment("seg_000001", 0, 1000, None, "one two"),
            TranscriptSegment("seg_000002", 1000, 2000, None, "three four"),
            TranscriptSegment("seg_000003", 2000, 3000, None, "five six"),
            TranscriptSegment("seg_000004", 3000, 4000, None, "seven eight"),
        ]

        chunks = chunk_segments_by_approx_tokens(segments, max_tokens=4, overlap_tokens=2)

        self.assertEqual(len(chunks), 3)
        self.assertEqual([segment.segment_id for segment in chunks[0].segments], ["seg_000001", "seg_000002"])
        self.assertEqual([segment.segment_id for segment in chunks[1].segments], ["seg_000002", "seg_000003"])
        self.assertEqual([segment.segment_id for segment in chunks[2].segments], ["seg_000003", "seg_000004"])
        self.assertEqual(chunks[0].start_ms, 0)
        self.assertEqual(chunks[2].end_ms, 4000)

    def test_format_segments_for_prompt_keeps_timestamps(self):
        text = format_segments_for_prompt(
            [TranscriptSegment("seg_000001", 62000, 64000, None, "hello world")]
        )

        self.assertEqual(text, "[00:01:02] hello world")

    def test_approximate_token_count_is_not_plain_word_count(self):
        self.assertGreater(approximate_token_count("one, two. three!"), 3)
        self.assertGreater(
            approximate_token_count("\u0440\u0443\u0441\u0441\u043a\u0438\u0439 \u0442\u0435\u043a\u0441\u0442"),
            2,
        )


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: FAIL because `research.youtube_pipeline.moc` does not exist.

- [ ] **Step 3: Implement transcript helpers**

Create `research/youtube_pipeline/moc.py`:

```python
from dataclasses import asdict, dataclass, field
import json
import math
import re
import string
from typing import Any


TIMESTAMP_PREFIX_RE = re.compile(r"^\[?(?:(\d{1,2}):)?(\d{2}):(\d{2})\]?\s+")


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
    match = TIMESTAMP_PREFIX_RE.match(line.strip())
    if not match:
        return None, line.strip()
    hours = int(match.group(1) or 0)
    minutes = int(match.group(2))
    seconds = int(match.group(3))
    timestamp_ms = (hours * 3600 + minutes * 60 + seconds) * 1000
    return timestamp_ms, line.strip()[match.end():].strip()


def parse_timestamped_transcript(transcript: str) -> tuple[list[TranscriptSegment], list[str]]:
    segments: list[TranscriptSegment] = []
    missing_timestamp = False
    for index, raw_line in enumerate([line for line in transcript.splitlines() if line.strip()], start=1):
        start_ms, text = parse_timestamp_ms(raw_line)
        if start_ms is None:
            missing_timestamp = True
        segments.append(
            TranscriptSegment(
                segment_id=f"seg_{index:06d}",
                start_ms=start_ms,
                end_ms=None,
                speaker=None,
                text=text,
            )
        )
    for index, segment in enumerate(segments):
        if segment.start_ms is None:
            continue
        next_timestamp = next(
            (candidate.start_ms for candidate in segments[index + 1 :] if candidate.start_ms is not None),
            None,
        )
        segment.end_ms = next_timestamp if next_timestamp is not None else segment.start_ms
    warnings = ["missing_timestamps"] if missing_timestamp else []
    return segments, warnings


def word_count(text: str) -> int:
    return len(text.split())


def approximate_token_count(text: str) -> int:
    words = len(text.split())
    punctuation = len(re.findall(r"[^\w\s]", text, flags=re.UNICODE))
    non_ascii = sum(1 for char in text if ord(char) > 127)
    return max(words, math.ceil(words * 1.15 + punctuation * 0.5 + non_ascii * 0.15))


def chunk_time_span(segments: list[TranscriptSegment]) -> tuple[int | None, int | None]:
    starts = [segment.start_ms for segment in segments if segment.start_ms is not None]
    ends = [segment.end_ms for segment in segments if segment.end_ms is not None]
    return (min(starts) if starts else None, max(ends) if ends else None)


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
        segments=segments,
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
        raise ValueError("overlap_tokens cannot be negative")
    chunks: list[SegmentChunk] = []
    start = 0
    while start < len(segments):
        total = 0
        end = start
        while end < len(segments) and (end == start or total + word_count(segments[end].text) <= max_tokens):
            total += word_count(segments[end].text)
            end += 1
        chunks.append(make_segment_chunk(len(chunks) + 1, segments[start:end]))
        if end >= len(segments):
            break
        overlap = 0
        next_start = end
        while next_start > start and overlap < overlap_tokens:
            next_start -= 1
            overlap += word_count(segments[next_start].text)
        if next_start <= start:
            next_start = end
        start = next_start
    return chunks
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add research/youtube_pipeline/moc.py research/youtube_pipeline/tests/test_moc.py
git commit -m "feat: add moc transcript chunking helpers"
```

---

### Task 3: Add MoC Budget, Projection, and Plan Fallback Helpers

**Files:**
- Modify: `research/youtube_pipeline/moc.py`
- Test: `research/youtube_pipeline/tests/test_moc.py`

- [ ] **Step 1: Write failing tests for budget, projection, and fallback MoC**

Append these tests to `MocTranscriptTests` in `research/youtube_pipeline/tests/test_moc.py`:

```python
    def test_moc_budget_for_tucker_range(self):
        from research.youtube_pipeline.moc import compute_moc_budget
        from research.youtube_pipeline.strategies import StrategyOptions

        budget = compute_moc_budget(transcript_words=41384, options=StrategyOptions())

        self.assertEqual(budget.report_min_words, 7000)
        self.assertEqual(budget.report_max_words, 10000)
        self.assertEqual(budget.target_report_words, 8500)
        self.assertEqual(budget.expected_node_min, 8)
        self.assertEqual(budget.expected_node_max, 12)

    def test_temporal_projection_preserves_window_edges(self):
        from research.youtube_pipeline.moc import build_temporal_projection

        segments = [
            TranscriptSegment(f"seg_{index:06d}", index * 60000, (index + 1) * 60000, None, f"line {index} " + "word " * 20)
            for index in range(12)
        ]

        projection = build_temporal_projection(segments, source_word_count=240, window_ms=300000)

        self.assertEqual(projection["projection_kind"], "temporal_skeleton")
        self.assertEqual(len(projection["windows"]), 3)
        self.assertEqual(projection["windows"][0]["start_ms"], 0)
        self.assertIn("line 0", projection["windows"][0]["first_words"])
        self.assertIn("line 4", projection["windows"][0]["last_words"])

    def test_fallback_moc_nodes_are_contiguous_and_budgeted(self):
        from research.youtube_pipeline.moc import compute_moc_budget, fallback_moc_plan
        from research.youtube_pipeline.strategies import StrategyOptions

        segments = [
            TranscriptSegment(f"seg_{index:06d}", index * 1000, (index + 1) * 1000, None, "one two three")
            for index in range(10)
        ]
        budget = compute_moc_budget(transcript_words=1200, options=StrategyOptions(chapter_target_words=500))

        plan = fallback_moc_plan(video_id="video1", segments=segments, budget=budget)

        self.assertEqual(plan["video_id"], "video1")
        self.assertGreaterEqual(len(plan["nodes"]), 1)
        self.assertEqual(plan["nodes"][0]["time_span"]["start_ms"], 0)
        self.assertEqual(plan["nodes"][-1]["time_span"]["end_ms"], 10000)
        self.assertEqual(sum(node["target_word_count"] for node in plan["nodes"]), budget.target_report_words)
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: FAIL because `compute_moc_budget`, `build_temporal_projection`, and `fallback_moc_plan` are missing.

- [ ] **Step 3: Add budget and MoC fallback code**

Append to `research/youtube_pipeline/moc.py`:

```python
MAX_REPORT_WORDS = 20000
MAX_MOC_NODES = 20
MIN_NODE_WORDS = 500


@dataclass
class MocBudget:
    transcript_words: int
    report_min_words: int
    report_max_words: int
    target_report_words: int
    expected_node_min: int
    expected_node_max: int


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


def compute_moc_budget(*, transcript_words: int, options: Any) -> MocBudget:
    if options.min_report_words is not None and options.max_report_words is not None:
        if options.min_report_words > options.max_report_words:
            raise ValueError("min_report_words cannot be greater than max_report_words")
    base_min, base_max, node_min, node_max = select_moc_base(transcript_words)
    multiplier = moc_depth_multiplier(options.target_depth)
    scaled_min = round(base_min * multiplier)
    scaled_max = round(base_max * multiplier)
    report_min = options.min_report_words if options.min_report_words is not None else scaled_min
    report_max = options.max_report_words if options.max_report_words is not None else scaled_max
    report_min = min(report_min, MAX_REPORT_WORDS)
    report_max = min(report_max, MAX_REPORT_WORDS)
    if report_min > report_max:
        raise ValueError("min_report_words cannot be greater than max_report_words")
    return MocBudget(
        transcript_words=transcript_words,
        report_min_words=report_min,
        report_max_words=report_max,
        target_report_words=round((report_min + report_max) / 2),
        expected_node_min=node_min,
        expected_node_max=min(node_max, MAX_MOC_NODES),
    )


def first_n_words(text: str, limit: int) -> str:
    return " ".join(text.split()[:limit])


def last_n_words(text: str, limit: int) -> str:
    return " ".join(text.split()[-limit:])


def build_temporal_projection(
    segments: list[TranscriptSegment],
    *,
    source_word_count: int,
    window_ms: int = 300000,
) -> dict[str, object]:
    timed = [segment for segment in segments if segment.start_ms is not None]
    if not timed:
        return {
            "projection_kind": "temporal_skeleton",
            "source_segment_count": len(segments),
            "source_word_count": source_word_count,
            "windows": [],
        }
    start_ms = min(segment.start_ms or 0 for segment in timed)
    end_ms = max(segment.end_ms if segment.end_ms is not None else segment.start_ms or 0 for segment in timed)
    windows: list[dict[str, object]] = []
    cursor = start_ms
    while cursor < end_ms:
        window_end = min(cursor + window_ms, end_ms)
        window_segments = [
            segment
            for segment in segments
            if segment.start_ms is not None and cursor <= segment.start_ms < window_end
        ]
        text = " ".join(segment.text for segment in window_segments)
        windows.append(
            {
                "window_id": f"window_{len(windows) + 1:03d}",
                "start_ms": cursor,
                "end_ms": window_end,
                "word_count": word_count(text),
                "first_words": first_n_words(text, 80),
                "last_words": last_n_words(text, 80),
                "sampled_timestamped_lines": format_segments_for_prompt(window_segments[:3]).splitlines(),
            }
        )
        cursor = window_end
    return {
        "projection_kind": "temporal_skeleton",
        "source_segment_count": len(segments),
        "source_word_count": source_word_count,
        "windows": windows,
    }


def split_budget_evenly(total: int, count: int) -> list[int]:
    base = total // max(1, count)
    remainder = total - base * count
    return [base + (1 if index < remainder else 0) for index in range(count)]


def fallback_moc_plan(*, video_id: str, segments: list[TranscriptSegment], budget: MocBudget) -> dict[str, object]:
    node_count = max(1, min(round(budget.target_report_words / 900), MAX_MOC_NODES, len(segments) or 1))
    node_budgets = split_budget_evenly(budget.target_report_words, node_count)
    nodes: list[dict[str, object]] = []
    for index in range(node_count):
        start = round(index * len(segments) / node_count)
        end = round((index + 1) * len(segments) / node_count)
        group = segments[start:end] or segments[:1]
        start_ms, end_ms = chunk_time_span(group)
        nodes.append(
            {
                "node_id": f"node_{index + 1:03d}",
                "title": f"Section {index + 1}",
                "time_span": {"start_ms": start_ms, "end_ms": end_ms},
                "importance": "medium",
                "target_word_count": node_budgets[index],
                "description_outline": "Deterministic fallback section based on transcript time range.",
                "essential_key_terms": [],
                "required_questions": [],
                "expected_fact_types": ["claims", "examples", "quotes"],
            }
        )
    return {
        "video_id": video_id,
        "report_thesis": "Fallback MoC generated from transcript time windows.",
        "global_key_terms": [],
        "nodes": nodes,
    }
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add research/youtube_pipeline/moc.py research/youtube_pipeline/tests/test_moc.py
git commit -m "feat: add moc planning helpers"
```

---

### Task 4: Add Fact Normalization, Deduplication, and Alignment

**Files:**
- Modify: `research/youtube_pipeline/moc.py`
- Test: `research/youtube_pipeline/tests/test_moc.py`

- [ ] **Step 1: Write failing tests for dedupe and alignment**

Append:

```python
    def test_deduplicate_facts_merges_nearby_overlapping_mentions(self):
        from research.youtube_pipeline.moc import deduplicate_facts

        facts = [
            {
                "fact_id": "f1",
                "kind": "claim",
                "text": "The media system serves state power.",
                "importance": "high",
                "time_span": {"start_ms": 1000, "end_ms": 2000},
                "verbatim_quote": "media system",
                "entities": ["media"],
                "topic_tags": ["state"],
            },
            {
                "fact_id": "f2",
                "kind": "claim",
                "text": "The media system serves state power!",
                "importance": "high",
                "time_span": {"start_ms": 2500, "end_ms": 3000},
                "verbatim_quote": "serves state power",
                "entities": ["media"],
                "topic_tags": ["state"],
            },
        ]

        clusters = deduplicate_facts(facts)

        self.assertEqual(len(clusters), 1)
        self.assertEqual(clusters[0]["kind"], "claim")
        self.assertEqual(len(clusters[0]["mentions"]), 2)

    def test_align_fact_clusters_uses_time_and_terms(self):
        from research.youtube_pipeline.moc import align_fact_clusters_to_moc

        moc = {
            "nodes": [
                {
                    "node_id": "node_001",
                    "title": "Media and state power",
                    "time_span": {"start_ms": 0, "end_ms": 10000},
                    "target_word_count": 900,
                    "essential_key_terms": ["media"],
                },
                {
                    "node_id": "node_002",
                    "title": "Family and technology",
                    "time_span": {"start_ms": 10000, "end_ms": 20000},
                    "target_word_count": 900,
                    "essential_key_terms": ["family"],
                },
            ],
            "global_key_terms": ["state"],
        }
        clusters = [
            {
                "cluster_id": "cluster_000001",
                "canonical_text": "Media serves state power",
                "kind": "claim",
                "importance": "high",
                "mentions": [{"time_span": {"start_ms": 1000, "end_ms": 2000}, "verbatim_quote": "media"}],
                "entities": ["media"],
                "topic_tags": ["state"],
            }
        ]

        aligned, unaligned = align_fact_clusters_to_moc(moc, clusters)

        self.assertEqual(unaligned, [])
        self.assertEqual(aligned[0]["node"]["node_id"], "node_001")
        self.assertEqual(aligned[0]["aligned_fact_clusters"][0]["cluster_id"], "cluster_000001")
        self.assertEqual(aligned[1]["aligned_fact_clusters"], [])
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: FAIL because dedupe/alignment functions are missing.

- [ ] **Step 3: Implement fact dedupe and alignment**

Append to `research/youtube_pipeline/moc.py`:

```python
VALID_FACT_KINDS = {
    "claim",
    "evidence",
    "quote",
    "example",
    "definition",
    "warning",
    "action_item",
    "open_question",
}
VALID_IMPORTANCE = {"high", "medium", "low"}


def normalize_kind(value: object) -> str:
    text = str(value or "").strip().lower()
    return text if text in VALID_FACT_KINDS else "evidence"


def normalize_importance(value: object) -> str:
    text = str(value or "").strip().lower()
    return text if text in VALID_IMPORTANCE else "medium"


def normalized_tokens(text: str) -> set[str]:
    table = str.maketrans("", "", string.punctuation)
    tokens = text.lower().translate(table).split()
    return {token for token in tokens if len(token) >= 3 or token.isnumeric()}


def normalized_text(text: str) -> str:
    return " ".join(sorted(normalized_tokens(text)))


def jaccard(a: set[str], b: set[str]) -> float:
    if not a and not b:
        return 1.0
    return len(a & b) / max(1, len(a | b))


def time_span_from_dict(payload: dict[str, object]) -> tuple[int | None, int | None]:
    start = payload.get("start_ms")
    end = payload.get("end_ms")
    return (int(start) if isinstance(start, int) else None, int(end) if isinstance(end, int) else None)


def spans_near(a: dict[str, object], b: dict[str, object], max_distance_ms: int = 60000) -> bool:
    a_start, a_end = time_span_from_dict(a)
    b_start, b_end = time_span_from_dict(b)
    if a_start is None or a_end is None or b_start is None or b_end is None:
        return False
    if max(a_start, b_start) <= min(a_end, b_end):
        return True
    return min(abs(a_start - b_end), abs(b_start - a_end)) <= max_distance_ms


def fact_should_merge(a: dict[str, object], b: dict[str, object]) -> bool:
    a_text = str(a.get("text", ""))
    b_text = str(b.get("text", ""))
    if normalized_text(a_text) == normalized_text(b_text):
        return True
    a_span = a.get("time_span") if isinstance(a.get("time_span"), dict) else {}
    b_span = b.get("time_span") if isinstance(b.get("time_span"), dict) else {}
    return jaccard(normalized_tokens(a_text), normalized_tokens(b_text)) >= 0.60 and spans_near(a_span, b_span)


def fact_to_cluster(index: int, fact: dict[str, object]) -> dict[str, object]:
    time_span = fact.get("time_span") if isinstance(fact.get("time_span"), dict) else {}
    return {
        "cluster_id": f"cluster_{index:06d}",
        "canonical_text": str(fact.get("text", "")),
        "kind": normalize_kind(fact.get("kind")),
        "importance": normalize_importance(fact.get("importance")),
        "mentions": [
            {
                "fact_id": str(fact.get("fact_id", f"fact_{index:06d}")),
                "time_span": time_span,
                "verbatim_quote": str(fact.get("verbatim_quote", "")),
            }
        ],
        "entities": [str(item) for item in fact.get("entities", []) if isinstance(item, str)],
        "topic_tags": [str(item) for item in fact.get("topic_tags", []) if isinstance(item, str)],
    }


def merge_fact_into_cluster(cluster: dict[str, object], fact: dict[str, object]) -> None:
    mentions = cluster.setdefault("mentions", [])
    if isinstance(mentions, list):
        mentions.append(
            {
                "fact_id": str(fact.get("fact_id", "")),
                "time_span": fact.get("time_span") if isinstance(fact.get("time_span"), dict) else {},
                "verbatim_quote": str(fact.get("verbatim_quote", "")),
            }
        )
    for key in ("entities", "topic_tags"):
        existing = set(cluster.get(key, [])) if isinstance(cluster.get(key), list) else set()
        incoming = {str(item) for item in fact.get(key, []) if isinstance(item, str)}
        cluster[key] = sorted(existing | incoming)


def deduplicate_facts(facts: list[dict[str, object]]) -> list[dict[str, object]]:
    clusters: list[dict[str, object]] = []
    representatives: list[dict[str, object]] = []
    for fact in facts:
        match_index = next(
            (index for index, representative in enumerate(representatives) if fact_should_merge(representative, fact)),
            None,
        )
        if match_index is None:
            representatives.append(fact)
            clusters.append(fact_to_cluster(len(clusters) + 1, fact))
        else:
            merge_fact_into_cluster(clusters[match_index], fact)
    return clusters


def interval_overlap_score(mention_span: dict[str, object], node_span: dict[str, object]) -> float:
    m_start, m_end = time_span_from_dict(mention_span)
    n_start, n_end = time_span_from_dict(node_span)
    if m_start is None or m_end is None or n_start is None or n_end is None:
        return 0.0
    mention_duration = max(m_end - m_start, 1)
    intersection = max(0, min(m_end, n_end) - max(m_start, n_start))
    if intersection > 0:
        return intersection / mention_duration
    distance = min(abs(m_start - n_end), abs(n_start - m_end))
    if distance <= 120000:
        return 0.5 * (1 - distance / 120000)
    return 0.0


def cluster_terms(cluster: dict[str, object]) -> set[str]:
    terms = set()
    terms.update(str(item).lower() for item in cluster.get("entities", []) if isinstance(item, str))
    terms.update(str(item).lower() for item in cluster.get("topic_tags", []) if isinstance(item, str))
    terms.update(normalized_tokens(str(cluster.get("canonical_text", ""))))
    return terms


def node_terms(node: dict[str, object], global_terms: list[str]) -> set[str]:
    terms = set(str(item).lower() for item in node.get("essential_key_terms", []) if isinstance(item, str))
    terms.update(str(item).lower() for item in global_terms)
    terms.update(normalized_tokens(str(node.get("title", ""))))
    return terms


def alignment_score(node: dict[str, object], cluster: dict[str, object], global_terms: list[str]) -> float:
    node_span = node.get("time_span") if isinstance(node.get("time_span"), dict) else {}
    mentions = cluster.get("mentions") if isinstance(cluster.get("mentions"), list) else []
    time_score = max(
        (
            interval_overlap_score(
                mention.get("time_span") if isinstance(mention, dict) and isinstance(mention.get("time_span"), dict) else {},
                node_span,
            )
            for mention in mentions
            if isinstance(mention, dict)
        ),
        default=0.0,
    )
    fact_terms = cluster_terms(cluster)
    lexical_score = len(fact_terms & node_terms(node, global_terms)) / max(1, len(fact_terms))
    hint_score = 0.0
    return 0.65 * time_score + 0.25 * lexical_score + 0.10 * hint_score


def align_fact_clusters_to_moc(
    moc_plan: dict[str, object],
    clusters: list[dict[str, object]],
    *,
    threshold: float = 0.30,
) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    nodes = moc_plan.get("nodes") if isinstance(moc_plan.get("nodes"), list) else []
    global_terms = [str(item) for item in moc_plan.get("global_key_terms", []) if isinstance(item, str)]
    aligned = [
        {
            "node": node,
            "aligned_fact_clusters": [],
            "raw_transcript_slice": "",
            "coverage_warnings": [],
        }
        for node in nodes
        if isinstance(node, dict)
    ]
    unaligned: list[dict[str, object]] = []
    for cluster in clusters:
        scores = [
            (index, alignment_score(row["node"], cluster, global_terms))
            for index, row in enumerate(aligned)
        ]
        best_index, best_score = max(scores, key=lambda item: item[1], default=(-1, 0.0))
        if best_index >= 0 and best_score >= threshold:
            aligned[best_index]["aligned_fact_clusters"].append(cluster)
        else:
            unaligned.append(cluster)
    return aligned, unaligned
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add research/youtube_pipeline/moc.py research/youtube_pipeline/tests/test_moc.py
git commit -m "feat: add moc fact alignment helpers"
```

---

### Task 5: Add Evidence Slice, Structured Outputs, Quality Checks, and Assembly

**Files:**
- Modify: `research/youtube_pipeline/moc.py`
- Test: `research/youtube_pipeline/tests/test_moc.py`

- [ ] **Step 1: Write failing tests for slice cap, structured result, and assembly**

Append:

```python
    def test_build_evidence_slice_truncates_large_node_slice(self):
        from research.youtube_pipeline.moc import build_evidence_slice

        segments = [
            TranscriptSegment(f"seg_{index:06d}", index * 10000, (index + 1) * 10000, None, "word " * 100)
            for index in range(20)
        ]
        node = {"time_span": {"start_ms": 0, "end_ms": 200000}}
        clusters = [
            {
                "mentions": [
                    {"time_span": {"start_ms": 50000, "end_ms": 60000}, "verbatim_quote": "quote"}
                ]
            }
        ]

        text, truncated = build_evidence_slice(node=node, clusters=clusters, segments=segments, max_slice_tokens=200)

        self.assertTrue(truncated)
        self.assertIn("[00:00:50]", text)
        self.assertLess(len(text.split()), 450)

    def test_build_structured_result_from_moc_facts(self):
        from research.youtube_pipeline.moc import build_structured_result_from_facts

        moc = {"nodes": [{"node_id": "node_001", "title": "Topic", "time_span": {"start_ms": 0, "end_ms": 1000}}]}
        aligned = [
            {
                "node": moc["nodes"][0],
                "aligned_fact_clusters": [
                    {
                        "cluster_id": "cluster_000001",
                        "canonical_text": "Important claim",
                        "kind": "claim",
                        "importance": "high",
                        "mentions": [{"time_span": {"start_ms": 0, "end_ms": 1000}, "verbatim_quote": "quote"}],
                        "entities": [],
                        "topic_tags": [],
                    }
                ],
                "coverage_warnings": [],
            }
        ]

        result = build_structured_result_from_facts(moc, aligned, action_items=[], open_questions=[])

        self.assertEqual(result.timeline[0].title, "Topic")
        self.assertEqual(result.claims[0].text, "Important claim")
        self.assertEqual(result.evidence[0].timestamp, "00:00:00")

    def test_assemble_moc_markdown_report_includes_sections(self):
        from research.youtube_pipeline.moc import assemble_moc_markdown_report

        report = assemble_moc_markdown_report(
            video_id="video1",
            overview="Overview",
            sections=[{"title": "One", "content": "Section body"}],
            structured_markdown={
                "timeline": "Timeline",
                "claims": "Claims",
                "action_items": "Actions",
                "open_questions": "Questions",
                "unaligned_facts": "- Unassigned fact",
            },
            conclusion="Conclusion",
        )

        self.assertIn("Generated via `moc_guided_map_reduce`", report)
        self.assertIn("Coverage Appendix: Unaligned Facts", report)
        self.assertIn("- Unassigned fact", report)
        self.assertIn("## Section 1: One", report)
        self.assertIn("Timeline", report)
        self.assertIn("Conclusion", report)
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: FAIL because functions are missing.

- [ ] **Step 3: Implement slice, structured result, checks, and assembly**

Append to `research/youtube_pipeline/moc.py`:

```python
from research.youtube_pipeline.models import ActionItem, Claim, Evidence, NormalizedResult, OpenQuestion, TimelineItem


def segment_overlaps(segment: TranscriptSegment, start_ms: int | None, end_ms: int | None) -> bool:
    if start_ms is None or end_ms is None or segment.start_ms is None:
        return False
    segment_end = segment.end_ms if segment.end_ms is not None else segment.start_ms
    return max(segment.start_ms, start_ms) <= min(segment_end, end_ms)


def segments_in_range(
    segments: list[TranscriptSegment],
    *,
    start_ms: int | None,
    end_ms: int | None,
) -> list[TranscriptSegment]:
    return [segment for segment in segments if segment_overlaps(segment, start_ms, end_ms)]


def mention_windows(clusters: list[dict[str, object]], context_ms: int = 30000) -> list[tuple[int, int]]:
    windows: list[tuple[int, int]] = []
    for cluster in clusters:
        mentions = cluster.get("mentions") if isinstance(cluster.get("mentions"), list) else []
        for mention in mentions:
            if not isinstance(mention, dict) or not isinstance(mention.get("time_span"), dict):
                continue
            start_ms, end_ms = time_span_from_dict(mention["time_span"])
            if start_ms is None or end_ms is None:
                continue
            windows.append((max(0, start_ms - context_ms), end_ms + context_ms))
    return windows


def build_evidence_slice(
    *,
    node: dict[str, object],
    clusters: list[dict[str, object]],
    segments: list[TranscriptSegment],
    max_slice_tokens: int,
) -> tuple[str, bool]:
    span = node.get("time_span") if isinstance(node.get("time_span"), dict) else {}
    start_ms, end_ms = time_span_from_dict(span)
    full_segments = segments_in_range(segments, start_ms=start_ms, end_ms=end_ms)
    full_text = format_segments_for_prompt(full_segments)
    if word_count(full_text) <= max_slice_tokens:
        return full_text, False
    selected: list[TranscriptSegment] = []
    for window_start, window_end in mention_windows(clusters):
        selected.extend(segments_in_range(segments, start_ms=window_start, end_ms=window_end))
    selected.extend(full_segments[:3])
    selected.extend(full_segments[-3:])
    unique = {segment.segment_id: segment for segment in selected}
    ordered = sorted(unique.values(), key=lambda segment: segment.start_ms if segment.start_ms is not None else 0)
    return format_segments_for_prompt(ordered), True


def timestamp_from_mention(cluster: dict[str, object]) -> str:
    mentions = cluster.get("mentions") if isinstance(cluster.get("mentions"), list) else []
    for mention in mentions:
        if isinstance(mention, dict) and isinstance(mention.get("time_span"), dict):
            start_ms, _ = time_span_from_dict(mention["time_span"])
            return format_ms(start_ms)
    return ""


def build_structured_result_from_facts(
    moc_plan: dict[str, object],
    aligned_nodes: list[dict[str, object]],
    *,
    action_items: list[dict[str, object]],
    open_questions: list[dict[str, object]],
) -> NormalizedResult:
    timeline: list[TimelineItem] = []
    claims: list[Claim] = []
    evidence: list[Evidence] = []
    for row in aligned_nodes:
        node = row["node"]
        span = node.get("time_span") if isinstance(node.get("time_span"), dict) else {}
        timeline.append(
            TimelineItem(
                start=format_ms(span.get("start_ms") if isinstance(span.get("start_ms"), int) else None),
                end=format_ms(span.get("end_ms") if isinstance(span.get("end_ms"), int) else None),
                title=str(node.get("title", "")),
                summary=str(node.get("description_outline", "")),
            )
        )
        for cluster in row.get("aligned_fact_clusters", []):
            if not isinstance(cluster, dict):
                continue
            if cluster.get("kind") == "claim":
                claims.append(
                    Claim(
                        text=str(cluster.get("canonical_text", "")),
                        importance=str(cluster.get("importance", "medium")),
                        evidence_refs=[timestamp_from_mention(cluster)] if timestamp_from_mention(cluster) else [],
                    )
                )
            evidence.append(
                Evidence(
                    text=str(cluster.get("canonical_text", "")),
                    timestamp=timestamp_from_mention(cluster),
                    supports_claims=[str(cluster.get("canonical_text", ""))] if cluster.get("kind") == "claim" else [],
                )
            )
    return NormalizedResult(
        timeline=timeline,
        claims=claims[:20],
        evidence=evidence[:40],
        action_items=[
            ActionItem(
                text=str(item.get("text", "")),
                target_audience=str(item.get("target_audience", "")),
                priority=normalize_importance(item.get("priority")),
            )
            for item in action_items[:20]
        ],
        open_questions=[
            OpenQuestion(
                text=str(item.get("text", "")),
                why_it_matters=str(item.get("why_it_matters", "")),
            )
            for item in open_questions[:20]
        ],
    )


def markdown_unaligned_facts(unaligned: list[dict[str, object]]) -> str:
    if not unaligned:
        return ""
    lines = [
        "These extracted facts were not confidently assigned to a MoC section and should be reviewed manually."
    ]
    for cluster in unaligned[:20]:
        lines.append(f"- {cluster.get('canonical_text', '')}")
    return "\n".join(lines)


def assemble_moc_markdown_report(
    *,
    video_id: str,
    overview: str,
    sections: list[dict[str, str]],
    structured_markdown: dict[str, str],
    conclusion: str,
) -> str:
    toc_lines = ["## Table of Contents", "", "- Executive Overview", "- Part I: Detailed MoC-Guided Narrative"]
    for index, section in enumerate(sections, start=1):
        toc_lines.append(f"  - Section {index}: {section['title']}")
    toc_lines.extend(
        [
            "- Part II: Structured Analysis",
            "  - Timeline and Development of Ideas",
            "  - Major Claims and Evidence",
            "  - Actionable Takeaways",
            "  - Open Questions",
            "- Conclusion and Synthesis",
        ]
    )
    section_text = "\n\n".join(
        f"## Section {index}: {section['title']}\n\n{section['content'].strip()}"
        for index, section in enumerate(sections, start=1)
    )
    parts = [
        f"# {video_id} MoC-Guided Deep Digest",
        "",
        "Generated via `moc_guided_map_reduce`.",
        "",
        "\n".join(toc_lines),
        "",
        "## Executive Overview",
        "",
        overview.strip(),
        "",
        "# Part I: Detailed MoC-Guided Narrative",
        "",
        section_text,
        "",
        "# Part II: Structured Analysis",
        "",
        "## Timeline and Development of Ideas",
        "",
        structured_markdown["timeline"],
        "",
        "## Major Claims and Evidence",
        "",
        structured_markdown["claims"],
        "",
        "## Actionable Takeaways",
        "",
        structured_markdown["action_items"],
        "",
        "## Open Questions",
        "",
        structured_markdown["open_questions"],
        "",
        "## Conclusion and Synthesis",
        "",
        conclusion.strip(),
    ]
    unaligned_facts = structured_markdown.get("unaligned_facts", "").strip()
    if unaligned_facts:
        parts[-3:-3] = ["## Coverage Appendix: Unaligned Facts", "", unaligned_facts, ""]
    return "\n".join(parts).strip() + "\n"
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_moc -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add research/youtube_pipeline/moc.py research/youtube_pipeline/tests/test_moc.py
git commit -m "feat: add moc assembly helpers"
```

---

### Task 6: Add MoC Prompt Builders

**Files:**
- Modify: `research/youtube_pipeline/prompts.py`
- Test: `research/youtube_pipeline/tests/test_prompts.py`

- [ ] **Step 1: Write failing prompt tests**

Add to `research/youtube_pipeline/tests/test_prompts.py`:

```python
    def test_moc_plan_prompt_requests_global_map_not_long_prose(self):
        from research.youtube_pipeline.prompts import build_moc_plan_messages

        messages = build_moc_plan_messages(
            transcript_context="Transcript",
            video_id="video1",
            report_min_words=7000,
            report_max_words=10000,
            target_report_words=8500,
            expected_node_min=8,
            expected_node_max=12,
            output_language="ru",
        )

        self.assertIn("Map of Content", messages[0].content)
        self.assertIn("Return JSON", messages[0].content)
        self.assertIn("do not write the report", messages[1].content)
        self.assertIn("target_word_count", messages[1].content)

    def test_moc_map_prompt_requests_atomic_facts(self):
        from research.youtube_pipeline.prompts import build_moc_map_extraction_messages

        messages = build_moc_map_extraction_messages(
            chunk_text="[00:00:01] hello",
            chunk_index=1,
            total_chunks=2,
            output_language="ru",
        )

        self.assertIn("atomic facts", messages[1].content)
        self.assertIn("verbatim_quote", messages[1].content)
        self.assertIn("action_items", messages[1].content)

    def test_moc_node_prompt_uses_aligned_facts_and_slice(self):
        from research.youtube_pipeline.prompts import build_moc_node_section_messages

        messages = build_moc_node_section_messages(
            node_json='{"title":"Topic","target_word_count":900}',
            aligned_facts_json='{"facts":[]}',
            raw_transcript_slice="[00:00:01] source",
            global_context_json='{"report_thesis":"Thesis"}',
            output_language="ru",
        )

        self.assertIn("Markdown prose only", messages[0].content)
        self.assertIn("aligned facts", messages[1].content)
        self.assertIn("raw transcript slice", messages[1].content)
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_prompts -v
```

Expected: FAIL because prompt builders are missing.

- [ ] **Step 3: Implement prompt builders**

Append to `research/youtube_pipeline/prompts.py`:

```python
def build_moc_plan_messages(
    *,
    transcript_context: str,
    video_id: str,
    report_min_words: int,
    report_max_words: int,
    target_report_words: int,
    expected_node_min: int,
    expected_node_max: int,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You create a global Map of Content for a long YouTube transcript. "
                "Return JSON only. Do not write the report prose."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Video id: {video_id}\n"
                f"Report word range: {report_min_words}-{report_max_words}\n"
                f"Target report words: {target_report_words}\n"
                f"Expected node count: {expected_node_min}-{expected_node_max}\n\n"
                "Build a flat ordered Map of Content. Identify major logical sections, time ranges, "
                "key terms, required questions, expected fact types, and target_word_count for each node. "
                "Do not write the report itself; this is planning only.\n\n"
                "Return exactly this JSON shape:\n"
                "{\n"
                '  "video_id": "video id",\n'
                '  "report_thesis": "One-sentence throughline",\n'
                '  "global_key_terms": ["term"],\n'
                '  "nodes": [{"node_id": "node_001", "title": "Title", "time_span": {"start_ms": 0, "end_ms": 1000}, "importance": "high", "target_word_count": 900, "description_outline": "What to cover", "essential_key_terms": ["term"], "required_questions": ["question"], "expected_fact_types": ["claims"]}]\n'
                "}\n\n"
                f"Transcript context:\n{transcript_context}"
            ),
        ),
    ]


def build_moc_map_extraction_messages(
    *,
    chunk_text: str,
    chunk_index: int,
    total_chunks: int,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content="You extract source-grounded facts from one transcript chunk. Return JSON only.",
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Chunk {chunk_index} of {total_chunks}\n\n"
                "Extract atomic facts, claims, evidence, quotes, examples, definitions, warnings, action_items, "
                "and open_questions. Preserve timestamps and short verbatim_quote values when available. "
                "Do not summarize the chunk into one paragraph.\n\n"
                "Return exactly this JSON shape:\n"
                "{\n"
                '  "chunk_index": 1,\n'
                '  "chunk_time_span": {"start_ms": 0, "end_ms": 1000},\n'
                '  "facts": [{"fact_id": "chunk_001_fact_001", "kind": "claim", "text": "Atomic fact", "importance": "high", "time_span": {"start_ms": 0, "end_ms": 1000}, "verbatim_quote": "quote", "speaker": null, "entities": ["entity"], "topic_tags": ["topic"], "moc_node_hint": null}],\n'
                '  "action_items": [{"text": "Action", "target_audience": "Audience", "priority": "medium"}],\n'
                '  "open_questions": [{"text": "Question", "why_it_matters": "Reason"}]\n'
                "}\n\n"
                f"Transcript chunk:\n{chunk_text}"
            ),
        ),
    ]


def build_moc_node_section_messages(
    *,
    node_json: str,
    aligned_facts_json: str,
    raw_transcript_slice: str,
    global_context_json: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content="You write one MoC-guided report section. Markdown prose only. Do not return JSON.",
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Write a detailed analytical section using the node metadata, aligned facts, compact global context, "
                "and raw transcript slice. Cite timestamps for important claims. Avoid generic filler and duplicated prose.\n\n"
                f"Node JSON:\n{node_json}\n\n"
                f"Aligned facts JSON:\n{aligned_facts_json}\n\n"
                f"Global context JSON:\n{global_context_json}\n\n"
                f"Raw transcript slice:\n{raw_transcript_slice}"
            ),
        ),
    ]


def build_moc_node_expansion_messages(
    *,
    section_draft: str,
    node_json: str,
    aligned_facts_json: str,
    raw_transcript_slice: str,
    current_word_count: int,
    target_word_count: int,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content="You expand one MoC-guided report section with source-grounded details. Markdown only.",
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Current words: {current_word_count}\n"
                f"Target words: {target_word_count}\n\n"
                "Rewrite the section as a fuller version. Integrate missing aligned facts, examples, quotes, "
                "timestamps, and transitions. Avoid generic padding.\n\n"
                f"Node JSON:\n{node_json}\n\n"
                f"Aligned facts JSON:\n{aligned_facts_json}\n\n"
                f"Raw transcript slice:\n{raw_transcript_slice}\n\n"
                f"Current section draft:\n{section_draft}"
            ),
        ),
    ]


def build_moc_overview_messages(
    *,
    moc_json: str,
    structured_result_json: str,
    section_summaries_json: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(role="system", content="You write an executive overview for a MoC-guided report."),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Write a concise executive overview. Frame the report without rewriting its sections.\n\n"
                f"MoC JSON:\n{moc_json}\n\n"
                f"Section summaries JSON:\n{section_summaries_json}\n\n"
                f"Structured result JSON:\n{structured_result_json}"
            ),
        ),
    ]


def build_moc_conclusion_messages(
    *,
    moc_json: str,
    structured_result_json: str,
    section_summaries_json: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(role="system", content="You write the final synthesis for a MoC-guided report."),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Write a final synthesis connecting the main claims, evidence, takeaways, and open questions. "
                "Do not rewrite the detailed sections.\n\n"
                f"MoC JSON:\n{moc_json}\n\n"
                f"Section summaries JSON:\n{section_summaries_json}\n\n"
                f"Structured result JSON:\n{structured_result_json}"
            ),
        ),
    ]
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_prompts -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add research/youtube_pipeline/prompts.py research/youtube_pipeline/tests/test_prompts.py
git commit -m "feat: add moc prompt builders"
```

---

### Task 7: Add `moc_guided_map_reduce` Strategy Orchestration

**Files:**
- Modify: `research/youtube_pipeline/strategies.py`
- Test: `research/youtube_pipeline/tests/test_strategies.py`

- [ ] **Step 1: Write failing orchestration test**

Add imports to `research/youtube_pipeline/tests/test_strategies.py`:

```python
    run_moc_guided_map_reduce,
```

Update `test_all_research_strategies_are_registered` expected list to include:

```python
                "moc_guided_map_reduce",
```

Add this test:

```python
    def test_moc_guided_map_reduce_maps_aligns_generates_and_assembles(self):
        transcript = "\n".join(
            [
                "[00:00:00] Media power opening",
                "[00:00:10] Media serves state power",
                "[00:00:20] Family and technology closing",
                "[00:00:30] Family matters",
            ]
        )
        client = SequenceClient(
            [
                (
                    '{"video_id":"video1","report_thesis":"Thesis","global_key_terms":["media","family"],'
                    '"nodes":['
                    '{"node_id":"node_001","title":"Media","time_span":{"start_ms":0,"end_ms":20000},"importance":"high","target_word_count":20,"description_outline":"Media topic","essential_key_terms":["media"],"required_questions":[],"expected_fact_types":["claims"]},'
                    '{"node_id":"node_002","title":"Family","time_span":{"start_ms":20000,"end_ms":40000},"importance":"medium","target_word_count":20,"description_outline":"Family topic","essential_key_terms":["family"],"required_questions":[],"expected_fact_types":["claims"]}'
                    ']}'
                ),
                (
                    '{"chunk_index":1,"chunk_time_span":{"start_ms":0,"end_ms":40000},'
                    '"facts":['
                    '{"fact_id":"f1","kind":"claim","text":"Media serves state power","importance":"high","time_span":{"start_ms":10000,"end_ms":11000},"verbatim_quote":"Media serves state power","speaker":null,"entities":["media"],"topic_tags":["state"],"moc_node_hint":null},'
                    '{"fact_id":"f2","kind":"claim","text":"Family matters","importance":"medium","time_span":{"start_ms":30000,"end_ms":31000},"verbatim_quote":"Family matters","speaker":null,"entities":["family"],"topic_tags":["family"],"moc_node_hint":null}'
                    '],"action_items":[],"open_questions":[]}'
                ),
                "## Section 1: Media\n\nMedia section has enough words with [00:00:10] timestamp " * 3,
                "## Section 2: Family\n\nFamily section has enough words with [00:00:30] timestamp " * 3,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=40,
                max_report_words=40,
                chapter_target_words=20,
            ),
        )

        self.assertIn("Generated via `moc_guided_map_reduce`", outcome.result.summary_text)
        self.assertIn("Media", outcome.result.summary_text)
        self.assertIn("Family", outcome.result.summary_text)
        self.assertEqual(outcome.request_count, 6)
        self.assertEqual(outcome.result.timeline[0].title, "Media")
        self.assertEqual(outcome.result.claims[0].text, "Media serves state power")
        self.assertIn("moc.json", outcome.extra_artifacts)
        self.assertIn("mapped_facts.jsonl", outcome.extra_artifacts)
        self.assertEqual(outcome.extra_metrics["moc_node_count"], 2)
        self.assertEqual(outcome.extra_metrics["deduplicated_fact_count"], 2)
        self.assertGreater(outcome.extra_metrics["estimated_transcript_tokens"], 0)
        self.assertEqual(outcome.extra_metrics["parallelism_enabled"], False)
        self.assertEqual(outcome.extra_metrics["parallelizable_map_call_count"], 1)
        self.assertEqual(outcome.extra_metrics["parallelizable_node_call_count"], 2)

    def test_moc_guided_map_reduce_records_invalid_json_when_moc_fallback_is_used(self):
        transcript = "\n".join(
            [
                "[00:00:00] Alpha topic",
                "[00:00:10] Beta topic",
            ]
        )
        client = SequenceClient(
            [
                "not json",
                '{"chunk_index":1,"facts":[],"action_items":[],"open_questions":[]}',
                "Fallback section text with enough words [00:00:00] " * 5,
                "Executive overview",
                "Final conclusion",
            ]
        )

        outcome = run_moc_guided_map_reduce(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                video_id="video1",
                max_tokens=2000,
                chunk_token_limit=100,
                chunk_overlap_tokens=10,
                min_report_words=20,
                max_report_words=20,
                chapter_target_words=20,
            ),
        )

        self.assertFalse(outcome.json_valid)
        self.assertTrue(outcome.extra_metrics["moc_fallback_used"])
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: FAIL because `run_moc_guided_map_reduce` is missing.

- [ ] **Step 3: Add imports and helper wrappers**

In `research/youtube_pipeline/strategies.py`, import MoC helpers:

```python
from research.youtube_pipeline.moc import (
    align_fact_clusters_to_moc,
    assemble_moc_markdown_report,
    approximate_token_count,
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
```

Import prompt builders:

```python
    build_moc_conclusion_messages,
    build_moc_map_extraction_messages,
    build_moc_node_expansion_messages,
    build_moc_node_section_messages,
    build_moc_overview_messages,
    build_moc_plan_messages,
```

- [ ] **Step 4: Implement strategy function**

Add to `research/youtube_pipeline/strategies.py` before `STRATEGIES`:

```python
def run_moc_guided_map_reduce(
    *,
    client: LlmClient,
    transcript: str,
    options: StrategyOptions,
) -> StrategyOutcome:
    if not transcript.strip():
        raise ValueError("transcript is empty")
    video_id = options.video_id

    segments, transcript_warnings = parse_timestamped_transcript(transcript)
    transcript_words = len(transcript.split())
    budget = compute_moc_budget(transcript_words=transcript_words, options=options)

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

    estimated_tokens = approximate_token_count(transcript)
    moc_projection_used = estimated_tokens > options.planner_context_token_limit
    if moc_projection_used:
        projection = build_temporal_projection(segments, source_word_count=transcript_words)
        planner_context = json.dumps(projection, ensure_ascii=False, indent=2)
    else:
        planner_context = format_segments_for_prompt(segments)

    moc_response = call_llm(
        build_moc_plan_messages(
            transcript_context=planner_context,
            video_id=video_id,
            report_min_words=budget.report_min_words,
            report_max_words=budget.report_max_words,
            target_report_words=budget.target_report_words,
            expected_node_min=budget.expected_node_min,
            expected_node_max=budget.expected_node_max,
            output_language=options.output_language,
        ),
        options.max_tokens,
    )
    moc_plan, moc_valid = parse_json_payload(moc_response.text)
    moc_fallback_used = not moc_valid or not isinstance(moc_plan.get("nodes"), list)
    if moc_fallback_used:
        moc_plan = fallback_moc_plan(video_id=video_id, segments=segments, budget=budget)
    json_valid = json_valid and moc_valid

    chunks = chunk_segments_by_approx_tokens(
        segments,
        max_tokens=options.chunk_token_limit,
        overlap_tokens=options.chunk_overlap_tokens,
    )
    mapped_rows: list[dict[str, object]] = []
    all_facts: list[dict[str, object]] = []
    action_items: list[dict[str, object]] = []
    open_questions: list[dict[str, object]] = []
    map_json_warning_count = 0
    for chunk in chunks:
        response = call_llm(
            build_moc_map_extraction_messages(
                chunk_text=chunk.text,
                chunk_index=chunk.chunk_index,
                total_chunks=len(chunks),
                output_language=options.output_language,
            ),
            options.max_tokens,
        )
        payload, valid = parse_json_payload(response.text)
        if not valid:
            map_json_warning_count += 1
            retry_messages = build_moc_map_extraction_messages(
                chunk_text=chunk.text,
                chunk_index=chunk.chunk_index,
                total_chunks=len(chunks),
                output_language=options.output_language,
            )
            retry_messages[-1].content += "\n\nThe previous response was invalid. Return valid JSON only."
            response = call_llm(retry_messages, options.max_tokens)
            payload, valid = parse_json_payload(response.text)
        json_valid = json_valid and valid
        if not valid:
            payload = {"chunk_index": chunk.chunk_index, "facts": [], "action_items": [], "open_questions": []}
        mapped_rows.append(payload)
        all_facts.extend(fact for fact in payload.get("facts", []) if isinstance(fact, dict))
        action_items.extend(item for item in payload.get("action_items", []) if isinstance(item, dict))
        open_questions.extend(item for item in payload.get("open_questions", []) if isinstance(item, dict))

    clusters = deduplicate_facts(all_facts)
    aligned_nodes, unaligned = align_fact_clusters_to_moc(moc_plan, clusters)

    section_rows: list[dict[str, object]] = []
    sections: list[dict[str, str]] = []
    expansion_count = 0
    slice_truncated_count = 0
    global_context_json = json.dumps(
        {
            "report_thesis": moc_plan.get("report_thesis", ""),
            "global_key_terms": moc_plan.get("global_key_terms", []),
            "node_titles": [node.get("title", "") for node in moc_plan.get("nodes", []) if isinstance(node, dict)],
        },
        ensure_ascii=False,
        indent=2,
    )
    for row in aligned_nodes:
        node = row["node"]
        facts = row["aligned_fact_clusters"]
        raw_slice, slice_truncated = build_evidence_slice(
            node=node,
            clusters=facts,
            segments=segments,
            max_slice_tokens=options.max_slice_tokens,
        )
        if slice_truncated:
            slice_truncated_count += 1
        row["raw_transcript_slice"] = raw_slice
        node_json = json.dumps(node, ensure_ascii=False, indent=2)
        facts_json = json.dumps(facts, ensure_ascii=False, indent=2)
        section_response = call_llm(
            build_moc_node_section_messages(
                node_json=node_json,
                aligned_facts_json=facts_json,
                raw_transcript_slice=raw_slice,
                global_context_json=global_context_json,
                output_language=options.output_language,
            ),
            options.max_tokens,
        )
        section_text = section_response.text.strip()
        target_words = int(node.get("target_word_count", options.chapter_target_words) or options.chapter_target_words)
        if len(section_text.split()) < int(0.8 * target_words):
            expansion_count += 1
            expansion_response = call_llm(
                build_moc_node_expansion_messages(
                    section_draft=section_text,
                    node_json=node_json,
                    aligned_facts_json=facts_json,
                    raw_transcript_slice=raw_slice,
                    current_word_count=len(section_text.split()),
                    target_word_count=target_words,
                    output_language=options.output_language,
                ),
                options.max_tokens,
            )
            if expansion_response.text.strip():
                section_text = expansion_response.text.strip()
        section = {"title": str(node.get("title", "")), "content": section_text}
        sections.append(section)
        section_rows.append(
            {
                "node_id": node.get("node_id"),
                "title": node.get("title"),
                "word_count": len(section_text.split()),
                "target_word_count": target_words,
                "slice_truncated": slice_truncated,
                "content": section_text,
            }
        )

    structured = build_structured_result_from_facts(
        moc_plan,
        aligned_nodes,
        action_items=action_items,
        open_questions=open_questions,
    )
    structured_json = json.dumps(structured.to_dict(), ensure_ascii=False, indent=2)
    moc_json = json.dumps(moc_plan, ensure_ascii=False, indent=2)
    section_summaries_json = json.dumps(
        [{"title": row["title"], "word_count": row["word_count"]} for row in section_rows],
        ensure_ascii=False,
        indent=2,
    )

    overview = call_llm(
        build_moc_overview_messages(
            moc_json=moc_json,
            structured_result_json=structured_json,
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
            structured_result_json=structured_json,
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
        "transcript_warnings": transcript_warnings,
        "map_json_warning_count": map_json_warning_count,
        "unaligned_fact_count": len(unaligned),
        "section_count": len(sections),
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
        },
        extra_artifacts={
            "moc.json": moc_plan,
            "mapped_facts.jsonl": "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in mapped_rows),
            "deduplicated_facts.json": clusters,
            "alignment.json": {"aligned_nodes": aligned_nodes, "unaligned_facts": unaligned},
            "node_sections.jsonl": "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in section_rows),
            "quality_checks.json": quality_checks,
        },
    )
```

Register it:

```python
STRATEGIES = {
    "adaptive_book_report": run_adaptive_book_report,
    "antigravity_chunk_map_reduce": run_antigravity_chunk_map_reduce,
    "moc_guided_map_reduce": run_moc_guided_map_reduce,
    ...
}
```

- [ ] **Step 5: Verify runner strategy invocation stays uniform**

No runner branch is needed for `moc_guided_map_reduce`. `build_strategy_options()` stores `args.video_id` in `StrategyOptions.video_id`, and every strategy keeps the same call shape:

```python
    outcome = STRATEGIES[args.strategy](
        client=client,
        transcript=transcript,
        options=options,
    )
```

Existing strategies can ignore `options.video_id`; `run_moc_guided_map_reduce()` uses it for prompts, fallback MoC, markdown, metrics, and artifacts.

- [ ] **Step 6: Run strategy tests and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add research/youtube_pipeline/strategies.py research/youtube_pipeline/runner.py research/youtube_pipeline/tests/test_strategies.py
git commit -m "feat: add moc guided map reduce strategy"
```

---

### Task 8: Add Runner Artifact Tests for MoC Files

**Files:**
- Modify: `research/youtube_pipeline/tests/test_runner.py`

- [ ] **Step 1: Add integration-style artifact test**

Add:

```python
    def test_write_run_artifacts_rejects_nested_extra_artifact_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[],
                raw_responses=[],
                extra_artifacts={"nested/file.json": {}},
            )

            with self.assertRaisesRegex(ValueError, "extra artifact filename"):
                write_run_artifacts(
                    root=Path(tmp),
                    strategy="moc_guided_map_reduce",
                    video_id="video1",
                    outcome=outcome,
                )
```

- [ ] **Step 2: Run test and verify pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: PASS.

- [ ] **Step 3: Commit**

```powershell
git add research/youtube_pipeline/tests/test_runner.py
git commit -m "test: cover moc artifact safeguards"
```

---

### Task 9: Update README

**Files:**
- Modify: `research/youtube_pipeline/README.md`

- [ ] **Step 1: Update strategy list**

Add `moc_guided_map_reduce` to the available strategies list:

```text
adaptive_book_report
antigravity_chunk_map_reduce
moc_guided_map_reduce
one_shot_full_json
one_shot_markdown_plus_json
two_pass_summary_structure
chunk_map_reduce
```

- [ ] **Step 2: Add MoC run example**

Add:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id f_lRdkH_QoY `
  --strategy moc_guided_map_reduce `
  --output-language ru `
  --max-tokens 8192 `
  --chunk-token-limit 3000 `
  --chunk-overlap-tokens 700 `
  --planner-context-token-limit 120000 `
  --max-slice-tokens 8000
```

- [ ] **Step 3: Run docs grep**

Run:

```powershell
Select-String -LiteralPath research\youtube_pipeline\README.md -Pattern "moc_guided_map_reduce"
```

Expected: at least two matches.

- [ ] **Step 4: Commit**

```powershell
git add research/youtube_pipeline/README.md
git commit -m "docs: document moc guided map reduce research strategy"
```

---

### Task 10: Full Verification

**Files:**
- No code edits unless verification reveals a defect.

- [ ] **Step 1: Run all unit tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: all tests PASS.

- [ ] **Step 2: Run a dry CLI parse check without LLM**

Run:

```powershell
python -m research.youtube_pipeline.runner --help
```

Expected: help includes:

```text
moc_guided_map_reduce
--chunk-overlap-tokens
--planner-context-token-limit
--max-slice-tokens
```

- [ ] **Step 3: Inspect git status**

Run:

```powershell
git status --short
```

Expected: only intentionally untracked research artifacts or spec/plan files remain. Do not add files under `research/youtube_pipeline/inputs/` or `research/youtube_pipeline/runs/`.

- [ ] **Step 4: Commit plan/spec if requested**

If the user wants the approved spec and this plan committed, run:

```powershell
git add docs/superpowers/specs/2026-06-18-youtube-moc-guided-map-reduce-design.md docs/superpowers/plans/2026-06-18-youtube-moc-guided-map-reduce-implementation.md
git commit -m "docs: add moc guided map reduce design and plan"
```

Do not commit them automatically unless the user asks.

---

## Manual Live Validation After Implementation

Use this only after unit tests pass and the user has configured `YOUTUBE_RESEARCH_LLM_API_KEY`.

```powershell
$env:YOUTUBE_RESEARCH_LLM_BASE_URL = "http://localhost:20128/v1"
$env:YOUTUBE_RESEARCH_LLM_MODEL = "gemini/gemini-3.1-flash-lite-preview"
$env:YOUTUBE_RESEARCH_LLM_API_KEY = "..."

python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id f_lRdkH_QoY `
  --strategy moc_guided_map_reduce `
  --output-language ru `
  --max-tokens 8192 `
  --chunk-token-limit 3000 `
  --chunk-overlap-tokens 700 `
  --planner-context-token-limit 120000 `
  --max-slice-tokens 8000
```

Inspect:

```powershell
Get-Content research\youtube_pipeline\runs\manual\moc_guided_map_reduce\f_lRdkH_QoY\metrics.json
Get-ChildItem research\youtube_pipeline\runs\manual\moc_guided_map_reduce\f_lRdkH_QoY
```

Expected artifact files:

```text
result.json
result.md
metrics.json
raw_requests.jsonl
raw_responses.jsonl
moc.json
mapped_facts.jsonl
deduplicated_facts.json
alignment.json
node_sections.jsonl
quality_checks.json
```

Success criteria for the Tucker run:

- `summary_words` is closer to the configured target than previous `chunk_map_reduce` runs;
- `moc_node_count` is in the expected `8-12` range unless the planner intentionally chooses fewer;
- `deduplicated_fact_count` and `aligned_fact_count` are non-zero;
- `timeline_segments_count`, `claims_count`, and `evidence_count` are non-zero;
- `moc_fallback_used=false` for normal full-context run;
- `moc_projection_used=false` for the current Tucker input with the default planner limit;
- `estimated_transcript_tokens` is greater than `transcript_words`;
- `parallelism_enabled=false` is paired with non-zero `parallelizable_map_call_count`
  and `parallelizable_node_call_count`;
- if `unaligned_fact_count` is non-zero, `result.md` contains
  `Coverage Appendix: Unaligned Facts`.

---

## Self-Review Notes

Spec coverage:

- MoC planning: Task 3 and Task 7.
- Planner context policy and projection: Task 3 and Task 7.
- Segment parser and overlapping chunker: Task 2.
- Map extraction contract: Task 6 and Task 7.
- Fact deduplication and alignment: Task 4.
- Raw slice cap: Task 5 and Task 7.
- Node reduce and expansion: Task 6 and Task 7.
- Structured outputs: Task 5 and Task 7.
- Overview/conclusion with fallback: Task 7.
- Python assembly: Task 5 and Task 7.
- Extra artifacts and metrics: Task 1, Task 7, Task 8.
- Runner integration: Task 1 and Task 7.
- README: Task 9.
- Verification: Task 10.

Known v1 tradeoff:

- The plan keeps node reducers sequential in implementation while recording `parallelism_enabled=false`. This matches the spec's allowed fallback and avoids adding async client complexity before the first live research run.
