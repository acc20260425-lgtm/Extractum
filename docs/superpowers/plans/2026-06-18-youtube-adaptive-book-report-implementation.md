# YouTube Adaptive Book Report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the `adaptive_book_report` research strategy that produces adaptive long-form YouTube transcript reports through dense chunk notes, substance-aware chapter planning, chapter-by-chapter generation, expansion guards, structured reductions, and Python-side Markdown assembly.

**Architecture:** Keep the existing research prototype standard-library-only. Add a focused `research/youtube_pipeline/adaptive.py` module for deterministic planning, partitioning, token budgeting, descriptors, bridges, and assembly. Keep LLM-facing orchestration in `strategies.py`, prompt text in `prompts.py`, and CLI/artifact behavior in `runner.py`.

**Tech Stack:** Python 3 standard library, `dataclasses`, `json`, `math`, `unittest`, OpenAI-compatible chat completions through the existing `llm_client.py`.

---

## File Structure

- Create: `research/youtube_pipeline/adaptive.py`
  - Deterministic helpers for adaptive budgets, substance scores, dynamic programming chapter partitioning, outline descriptors, bridge extraction, token budgeting, and Markdown assembly.
- Create: `research/youtube_pipeline/tests/test_adaptive.py`
  - Unit tests for deterministic helper behavior without LLM calls.
- Modify: `research/youtube_pipeline/strategies.py`
  - Add `StrategyOptions`, extend `StrategyOutcome` with `extra_metrics`, migrate strategy signatures to the shared options interface, and implement `run_adaptive_book_report`.
- Modify: `research/youtube_pipeline/prompts.py`
  - Add adaptive prompt builders for chunk analysis, chapter outline, chapter generation, chapter expansion, overview, and conclusion.
- Modify: `research/youtube_pipeline/runner.py`
  - Expose adaptive CLI flags, build `StrategyOptions`, call strategies through the unified interface, fail fast on invalid report word overrides, and merge `extra_metrics` into `metrics.json`.
- Modify: `research/youtube_pipeline/README.md`
  - Document `adaptive_book_report`, adaptive flags, and ready-to-run examples.
- Modify tests:
  - `research/youtube_pipeline/tests/test_strategies.py`
  - `research/youtube_pipeline/tests/test_prompts.py`
  - `research/youtube_pipeline/tests/test_runner.py`

Verification command for every task:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

---

### Task 1: Shared Strategy Options And Extra Metrics

**Files:**
- Modify: `research/youtube_pipeline/strategies.py`
- Modify: `research/youtube_pipeline/tests/test_strategies.py`

- [x] **Step 1: Write failing tests for `StrategyOptions` and `extra_metrics`**

Update imports in `research/youtube_pipeline/tests/test_strategies.py`:

```python
from research.youtube_pipeline.strategies import (
    StrategyOptions,
    run_antigravity_chunk_map_reduce,
    run_chunk_map_reduce,
    run_one_shot_full_json,
)
```

Update the first one-shot test to use the shared options interface:

```python
    def test_one_shot_full_json_returns_result_and_usage(self):
        client = FakeClient()
        outcome = run_one_shot_full_json(
            client=client,
            transcript="Transcript",
            options=StrategyOptions(output_language="ru", max_tokens=1000),
        )

        self.assertEqual(outcome.result.summary_text, "Summary text")
        self.assertEqual(outcome.request_count, 1)
        self.assertEqual(outcome.input_tokens, 10)
        self.assertEqual(outcome.output_tokens, 20)
        self.assertTrue(outcome.json_valid)
        self.assertEqual(outcome.extra_metrics, {})
        self.assertEqual(client.calls[0][1], 1000)
```

Update existing `run_chunk_map_reduce` calls in the same file to pass `options`:

```python
        outcome = run_chunk_map_reduce(
            client=client,
            transcript="one two three four five six",
            options=StrategyOptions(output_language="ru", max_tokens=1000, chunk_token_limit=3),
        )
```

Update the `run_antigravity_chunk_map_reduce` call similarly:

```python
        outcome = run_antigravity_chunk_map_reduce(
            client=client,
            transcript="one two three four five six",
            options=StrategyOptions(output_language="ru", max_tokens=1000, chunk_token_limit=3),
        )
```

- [x] **Step 2: Run the strategy tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: FAIL because `StrategyOptions` does not exist and existing strategy signatures do not accept `options`.

- [x] **Step 3: Add `StrategyOptions` and `extra_metrics`**

In `research/youtube_pipeline/strategies.py`, update the dataclass imports:

```python
from dataclasses import dataclass, field
```

Add this dataclass above `StrategyOutcome`:

```python
@dataclass
class StrategyOptions:
    output_language: str = "ru"
    max_tokens: int = 8192
    chunk_token_limit: int = 3000
    target_depth: str = "auto"
    min_report_words: int | None = None
    max_report_words: int | None = None
    chapter_target_words: int = 900
```

Add `extra_metrics` to `StrategyOutcome`:

```python
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
```

- [x] **Step 4: Migrate `run_one_shot_full_json` and wrappers to `StrategyOptions`**

Replace `run_one_shot_full_json` signature and internal option usage:

```python
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
        raw_responses=[
            {"text": response.text, "input_tokens": response.input_tokens, "output_tokens": response.output_tokens}
        ],
    )
```

Replace wrapper calls so they pass `options=options`. For example:

```python
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
```

- [x] **Step 5: Migrate chunked strategies to `StrategyOptions`**

Update `run_chunk_map_reduce`, `run_timeline_segment_reduce`, and `run_antigravity_chunk_map_reduce` signatures to accept only `options`. Replace:

```python
output_language
max_tokens
chunk_token_limit
```

with:

```python
options.output_language
options.max_tokens
options.chunk_token_limit
```

Every raw request entry should continue recording the exact max token value used:

```python
raw_requests.append({"messages": [message.__dict__ for message in messages], "max_tokens": options.max_tokens})
```

- [x] **Step 6: Run strategy tests to verify they pass**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: PASS.

- [x] **Step 7: Commit**

Run:

```powershell
git add research/youtube_pipeline/strategies.py research/youtube_pipeline/tests/test_strategies.py
git commit -m "refactor: add strategy options interface"
```

Expected: commit succeeds.

---

### Task 2: Runner CLI Options And Extra Metrics Artifacts

**Files:**
- Modify: `research/youtube_pipeline/runner.py`
- Modify: `research/youtube_pipeline/tests/test_runner.py`

- [x] **Step 1: Write failing runner tests for `extra_metrics`**

In `research/youtube_pipeline/tests/test_runner.py`, update the `StrategyOutcome` in `test_write_run_artifacts_creates_expected_files`:

```python
            outcome = StrategyOutcome(
                result=NormalizedResult(summary_text="Summary text"),
                request_count=1,
                input_tokens=10,
                output_tokens=20,
                latency_seconds=1.25,
                json_valid=True,
                raw_requests=[{"messages": []}],
                raw_responses=[{"text": "{}"}],
                extra_metrics={"chapter_count": 3, "target_report_words": 2700},
            )
```

Add assertions after reading metrics:

```python
            self.assertEqual(metrics["chapter_count"], 3)
            self.assertEqual(metrics["target_report_words"], 2700)
```

- [x] **Step 2: Write failing runner tests for CLI option parsing**

Add this import:

```python
from research.youtube_pipeline.runner import build_parser, build_strategy_options, write_run_artifacts
```

Add this test:

```python
    def test_build_strategy_options_reads_adaptive_cli_flags(self):
        parser = build_parser()
        args = parser.parse_args(
            [
                "--input",
                "input.txt",
                "--video-id",
                "video1",
                "--strategy",
                "chunk_map_reduce",
                "--output-language",
                "ru",
                "--max-tokens",
                "9000",
                "--chunk-token-limit",
                "2500",
                "--target-depth",
                "deep",
                "--min-report-words",
                "5000",
                "--max-report-words",
                "8000",
                "--chapter-target-words",
                "1000",
            ]
        )

        options = build_strategy_options(args)

        self.assertEqual(options.output_language, "ru")
        self.assertEqual(options.max_tokens, 9000)
        self.assertEqual(options.chunk_token_limit, 2500)
        self.assertEqual(options.target_depth, "deep")
        self.assertEqual(options.min_report_words, 5000)
        self.assertEqual(options.max_report_words, 8000)
        self.assertEqual(options.chapter_target_words, 1000)
```

Add invalid override test:

```python
    def test_build_strategy_options_rejects_min_greater_than_max(self):
        parser = build_parser()
        args = parser.parse_args(
            [
                "--input",
                "input.txt",
                "--video-id",
                "video1",
                "--strategy",
                "chunk_map_reduce",
                "--min-report-words",
                "9000",
                "--max-report-words",
                "8000",
            ]
        )

        with self.assertRaisesRegex(ValueError, "min-report-words cannot be greater than max-report-words"):
            build_strategy_options(args)
```

- [x] **Step 3: Run runner tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: FAIL because `build_parser`, `build_strategy_options`, and metrics merging are missing.

- [x] **Step 4: Implement parser and strategy option builder**

In `runner.py`, update imports:

```python
from research.youtube_pipeline.strategies import STRATEGIES, StrategyOptions, StrategyOutcome
```

Add:

```python
def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run YouTube summary pipeline research strategies.")
    parser.add_argument("--input", required=True, help="Path to transcript text file")
    parser.add_argument("--strategy", required=True, choices=sorted(STRATEGIES))
    parser.add_argument("--video-id", required=True)
    parser.add_argument("--output-root", default="research/youtube_pipeline/runs/manual")
    parser.add_argument("--output-language", default="ru")
    parser.add_argument("--max-tokens", type=int, default=8192)
    parser.add_argument("--chunk-token-limit", type=int, default=3000)
    parser.add_argument("--target-depth", choices=["auto", "brief", "standard", "deep", "book"], default="auto")
    parser.add_argument("--min-report-words", type=int, default=None)
    parser.add_argument("--max-report-words", type=int, default=None)
    parser.add_argument("--chapter-target-words", type=int, default=900)
    return parser


def build_strategy_options(args: argparse.Namespace) -> StrategyOptions:
    if args.min_report_words is not None and args.max_report_words is not None:
        if args.min_report_words > args.max_report_words:
            raise ValueError("min-report-words cannot be greater than max-report-words")
    return StrategyOptions(
        output_language=args.output_language,
        max_tokens=args.max_tokens,
        chunk_token_limit=args.chunk_token_limit,
        target_depth=args.target_depth,
        min_report_words=args.min_report_words,
        max_report_words=args.max_report_words,
        chapter_target_words=args.chapter_target_words,
    )
```

- [x] **Step 5: Merge `extra_metrics` into artifacts**

In `write_run_artifacts()`, replace the metrics write with:

```python
    metrics = build_metrics(
        strategy=strategy,
        video_id=video_id,
        result=outcome.result,
        request_count=outcome.request_count,
        input_tokens=outcome.input_tokens,
        output_tokens=outcome.output_tokens,
        latency_seconds=outcome.latency_seconds,
        json_valid=outcome.json_valid,
    )
    metrics.update(outcome.extra_metrics)
    write_json(output_dir / "metrics.json", metrics)
```

- [x] **Step 6: Update `main()` to use `StrategyOptions`**

Replace parser construction and strategy call in `main()`:

```python
def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    transcript = Path(args.input).read_text(encoding="utf-8")
    client = build_client_from_env()
    options = build_strategy_options(args)
    outcome = STRATEGIES[args.strategy](
        client=client,
        transcript=transcript,
        options=options,
    )
    output_dir = write_run_artifacts(
        root=Path(args.output_root),
        strategy=args.strategy,
        video_id=args.video_id,
        outcome=outcome,
    )
    print(output_dir)
    return 0
```

- [x] **Step 7: Run runner tests**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_runner -v
```

Expected: PASS.

- [x] **Step 8: Run all tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: PASS.

- [x] **Step 9: Commit**

Run:

```powershell
git add research/youtube_pipeline/runner.py research/youtube_pipeline/tests/test_runner.py
git commit -m "feat: add adaptive runner options"
```

Expected: commit succeeds.

---

### Task 3: Deterministic Adaptive Planning Helpers

**Files:**
- Create: `research/youtube_pipeline/adaptive.py`
- Create: `research/youtube_pipeline/tests/test_adaptive.py`

- [x] **Step 1: Write failing helper tests**

Create `research/youtube_pipeline/tests/test_adaptive.py`:

```python
import math
import unittest

from research.youtube_pipeline.adaptive import (
    BudgetPlan,
    build_outline_chunk_descriptors,
    compute_budget_plan,
    compute_chapter_word_target,
    compute_substance_multiplier,
    extract_previous_chapter_bridge,
    normalize_substance_score,
    partition_weighted_chunks,
    response_token_budget,
)
from research.youtube_pipeline.strategies import StrategyOptions


class AdaptiveHelperTests(unittest.TestCase):
    def test_normalize_substance_score_defaults_and_clamps(self):
        self.assertEqual(normalize_substance_score(None), 3)
        self.assertEqual(normalize_substance_score("bad"), 3)
        self.assertEqual(normalize_substance_score(0), 1)
        self.assertEqual(normalize_substance_score(6), 5)
        self.assertEqual(normalize_substance_score("4"), 4)

    def test_compute_substance_multiplier_uses_narrow_range(self):
        self.assertAlmostEqual(compute_substance_multiplier([1, 1]), 0.7)
        self.assertAlmostEqual(compute_substance_multiplier([3, 3]), 1.0)
        self.assertAlmostEqual(compute_substance_multiplier([5, 5]), 1.3)
        self.assertAlmostEqual(compute_substance_multiplier([]), 1.0)

    def test_compute_budget_plan_records_range_and_midpoint_target(self):
        options = StrategyOptions(output_language="ru", target_depth="auto", chapter_target_words=900)
        plan = compute_budget_plan(transcript_words=41000, substance_scores=[3, 3], options=options)

        self.assertEqual(plan.report_min_words, 7000)
        self.assertEqual(plan.report_max_words, 10000)
        self.assertEqual(plan.target_report_words, 8500)
        self.assertEqual(plan.chapter_count, 9)
        self.assertEqual(plan.chapter_word_target, 944)

    def test_compute_budget_plan_applies_overrides_and_caps(self):
        options = StrategyOptions(
            output_language="ru",
            target_depth="book",
            min_report_words=12000,
            max_report_words=50000,
            chapter_target_words=900,
        )
        plan = compute_budget_plan(transcript_words=80000, substance_scores=[5], options=options)

        self.assertEqual(plan.report_min_words, 12000)
        self.assertEqual(plan.report_max_words, 20000)
        self.assertEqual(plan.target_report_words, 16000)
        self.assertLessEqual(plan.chapter_count, 20)

    def test_compute_budget_plan_rejects_invalid_overrides(self):
        options = StrategyOptions(min_report_words=9000, max_report_words=8000)

        with self.assertRaisesRegex(ValueError, "min_report_words cannot be greater than max_report_words"):
            compute_budget_plan(transcript_words=41000, substance_scores=[3], options=options)

    def test_compute_chapter_word_target_rounds_from_total_and_count(self):
        self.assertEqual(compute_chapter_word_target(8500, 9), 944)

    def test_partition_weighted_chunks_uses_contiguous_dp_groups(self):
        groups = partition_weighted_chunks([1, 1, 10, 1, 1], chapter_count=3)

        self.assertEqual(groups, [(0, 2), (2, 3), (3, 5)])

    def test_response_token_budget_is_language_aware(self):
        self.assertEqual(response_token_budget(900, "ru", max_tokens=8192), math.ceil(900 * 2.8 * 1.15))
        self.assertEqual(response_token_budget(900, "en", max_tokens=8192), math.ceil(900 * 1.8 * 1.15))
        self.assertEqual(response_token_budget(900, "ja", max_tokens=8192), math.ceil(900 * 3.0 * 1.15))
        self.assertEqual(response_token_budget(900, "ru", max_tokens=1000), 1000)

    def test_build_outline_chunk_descriptors_caps_summary_words(self):
        summary = " ".join(f"word{i}" for i in range(120))
        descriptors = build_outline_chunk_descriptors(
            [
                {
                    "chunk_index": 1,
                    "substance_score": 4,
                    "result": {
                        "summary_text": summary,
                        "timeline": [{"title": "Timeline A"}, {"title": "Timeline B"}],
                        "claims": [{"text": "Claim A"}, {"text": "Claim B"}],
                    },
                }
            ]
        )

        self.assertEqual(descriptors[0]["chunk_index"], 1)
        self.assertEqual(descriptors[0]["substance_score"], 4)
        self.assertEqual(len(descriptors[0]["summary_preview"].split()), 100)
        self.assertEqual(descriptors[0]["snippets"], ["Timeline A", "Timeline B", "Claim A"])

    def test_extract_previous_chapter_bridge_uses_tail_and_caps_words(self):
        chapter = "Intro paragraph.\n\n" + " ".join(f"tail{i}" for i in range(250))

        bridge = extract_previous_chapter_bridge(chapter)

        self.assertEqual(len(bridge.split()), 200)
        self.assertTrue(bridge.startswith("tail50"))


if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Run helper tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_adaptive -v
```

Expected: FAIL because `research.youtube_pipeline.adaptive` does not exist.

- [x] **Step 3: Implement `adaptive.py` dataclasses and budget helpers**

Create `research/youtube_pipeline/adaptive.py` with these imports and budget code:

```python
from dataclasses import dataclass
import math
from typing import Any

from research.youtube_pipeline.metrics import count_words


MAX_REPORT_WORDS = 20000
MAX_CHAPTERS = 20


@dataclass
class BudgetPlan:
    transcript_words: int
    report_min_words: int
    report_max_words: int
    target_report_words: int
    chapter_count: int
    chapter_word_target: int
    substance_multiplier: float
    average_substance_score: float


def normalize_substance_score(value: object) -> int:
    try:
        score = int(value)
    except (TypeError, ValueError):
        return 3
    return max(1, min(5, score))


def compute_substance_multiplier(scores: list[int]) -> float:
    if not scores:
        return 1.0
    average = sum(scores) / len(scores)
    return 0.7 + 0.6 * ((average - 1) / 4)


def select_base_word_range(transcript_words: int) -> tuple[int, int]:
    if transcript_words < 5000:
        return 1000, 1800
    if transcript_words < 15000:
        return 2000, 3500
    if transcript_words < 35000:
        return 4000, 6500
    if transcript_words < 70000:
        return 7000, 10000
    return 10000, 14000


def depth_multiplier(target_depth: str) -> float:
    return {
        "auto": 1.0,
        "brief": 0.5,
        "standard": 1.0,
        "deep": 1.5,
        "book": 2.0,
    }.get(target_depth, 1.0)


def compute_chapter_word_target(target_report_words: int, chapter_count: int) -> int:
    return max(1, round(target_report_words / max(1, chapter_count)))


def compute_budget_plan(
    *,
    transcript_words: int,
    substance_scores: list[int],
    options: Any,
    chunk_count: int | None = None,
) -> BudgetPlan:
    if options.min_report_words is not None and options.max_report_words is not None:
        if options.min_report_words > options.max_report_words:
            raise ValueError("min_report_words cannot be greater than max_report_words")
    base_min, base_max = select_base_word_range(transcript_words)
    average_score = sum(substance_scores) / len(substance_scores) if substance_scores else 3.0
    substance_multiplier = compute_substance_multiplier(substance_scores)
    multiplier = depth_multiplier(options.target_depth) * substance_multiplier
    scaled_min = round(base_min * multiplier)
    scaled_max = round(base_max * multiplier)
    report_min_words = options.min_report_words if options.min_report_words is not None else scaled_min
    report_max_words = options.max_report_words if options.max_report_words is not None else scaled_max
    report_min_words = min(report_min_words, MAX_REPORT_WORDS)
    report_max_words = min(report_max_words, MAX_REPORT_WORDS)
    if report_min_words > report_max_words:
        raise ValueError("min_report_words cannot be greater than max_report_words")
    target_report_words = round((report_min_words + report_max_words) / 2)
    chapter_count = max(1, round(target_report_words / max(1, options.chapter_target_words)))
    if chunk_count is not None:
        chapter_count = min(chapter_count, max(1, chunk_count))
    chapter_count = min(chapter_count, MAX_CHAPTERS)
    return BudgetPlan(
        transcript_words=transcript_words,
        report_min_words=report_min_words,
        report_max_words=report_max_words,
        target_report_words=target_report_words,
        chapter_count=chapter_count,
        chapter_word_target=compute_chapter_word_target(target_report_words, chapter_count),
        substance_multiplier=substance_multiplier,
        average_substance_score=average_score,
    )
```

- [x] **Step 4: Implement DP partitioning**

Append:

```python
def partition_weighted_chunks(weights: list[int | float], chapter_count: int) -> list[tuple[int, int]]:
    n = len(weights)
    if n == 0:
        return []
    k = max(1, min(chapter_count, n))
    prefix = [0.0]
    for weight in weights:
        prefix.append(prefix[-1] + float(weight))
    target = prefix[-1] / k
    dp = [[float("inf")] * (k + 1) for _ in range(n + 1)]
    cut = [[0] * (k + 1) for _ in range(n + 1)]
    dp[0][0] = 0.0
    for end in range(1, n + 1):
        for groups in range(1, min(k, end) + 1):
            for start in range(groups - 1, end):
                chapter_weight = prefix[end] - prefix[start]
                cost = dp[start][groups - 1] + (chapter_weight - target) ** 2
                if cost < dp[end][groups]:
                    dp[end][groups] = cost
                    cut[end][groups] = start
    groups_out: list[tuple[int, int]] = []
    end = n
    groups = k
    while groups > 0:
        start = cut[end][groups]
        groups_out.append((start, end))
        end = start
        groups -= 1
    groups_out.reverse()
    return groups_out
```

- [x] **Step 5: Implement descriptors, bridge, token budgets, and markdown helpers**

Append:

```python
def first_words(text: str, limit: int) -> str:
    return " ".join(text.split()[:limit])


def build_outline_chunk_descriptors(chunk_results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    descriptors: list[dict[str, Any]] = []
    for row in chunk_results:
        result = row.get("result") if isinstance(row.get("result"), dict) else {}
        timeline = result.get("timeline") if isinstance(result.get("timeline"), list) else []
        claims = result.get("claims") if isinstance(result.get("claims"), list) else []
        snippets: list[str] = []
        for item in timeline:
            if isinstance(item, dict) and item.get("title"):
                snippets.append(str(item["title"]))
            if len(snippets) >= 3:
                break
        for item in claims:
            if len(snippets) >= 3:
                break
            if isinstance(item, dict) and item.get("text"):
                snippets.append(str(item["text"]))
        descriptors.append(
            {
                "chunk_index": row.get("chunk_index"),
                "substance_score": row.get("substance_score", 3),
                "summary_preview": first_words(str(result.get("summary_text", "")), 100),
                "snippets": snippets[:3],
            }
        )
    return descriptors


def extract_previous_chapter_bridge(chapter_text: str, max_words: int = 200) -> str:
    stripped = chapter_text.strip()
    if not stripped:
        return ""
    paragraphs = [part.strip() for part in stripped.split("\n\n") if part.strip()]
    candidate = paragraphs[-1] if paragraphs else stripped
    words = candidate.split()
    if len(words) < 30 or candidate.lstrip().startswith(("-", "*", "1.")):
        words = stripped.split()[-max_words:]
    else:
        words = words[-max_words:]
    return " ".join(words)


def language_token_multiplier(output_language: str) -> float:
    normalized = output_language.lower()
    if normalized.startswith("en"):
        return 1.8
    if normalized.startswith("ru"):
        return 2.8
    return 3.0


def response_token_budget(target_words: int, output_language: str, max_tokens: int) -> int:
    estimate = math.ceil(target_words * language_token_multiplier(output_language) * 1.15)
    return min(max_tokens, estimate)


def build_table_of_contents(chapter_titles: list[str]) -> str:
    lines = ["## Table of Contents", "", "- Executive Overview", "- Part I: Detailed Narrative"]
    for index, title in enumerate(chapter_titles, start=1):
        lines.append(f"  - Chapter {index}: {title}")
    lines.extend(
        [
            "- Part II: Structured Analysis",
            "  - Timeline and Development of Ideas",
            "  - Major Claims and Evidence",
            "  - Actionable Takeaways",
            "  - Open Questions",
            "- Conclusion and Synthesis",
        ]
    )
    return "\n".join(lines)


def assemble_adaptive_markdown_report(
    *,
    overview: str,
    chapters: list[str],
    chapter_titles: list[str],
    timeline_markdown: str,
    claims_markdown: str,
    action_items_markdown: str,
    open_questions_markdown: str,
    conclusion: str,
) -> str:
    parts = [
        "# YouTube Research Report",
        "",
        "Generated via `adaptive_book_report`.",
        "",
        build_table_of_contents(chapter_titles),
        "",
        "## Executive Overview",
        "",
        overview.strip(),
        "",
        "# Part I: Detailed Narrative",
        "",
        "\n\n".join(chapter.strip() for chapter in chapters if chapter.strip()),
        "",
        "# Part II: Structured Analysis",
        "",
        "## Timeline and Development of Ideas",
        "",
        timeline_markdown.strip(),
        "",
        "## Major Claims and Evidence",
        "",
        claims_markdown.strip(),
        "",
        "## Actionable Takeaways",
        "",
        action_items_markdown.strip(),
        "",
        "## Open Questions",
        "",
        open_questions_markdown.strip(),
        "",
        "## Conclusion and Synthesis",
        "",
        conclusion.strip(),
    ]
    return "\n".join(parts).strip() + "\n"
```

- [x] **Step 6: Run helper tests**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_adaptive -v
```

Expected: PASS.

- [x] **Step 7: Run all tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: PASS.

- [x] **Step 8: Commit**

Run:

```powershell
git add research/youtube_pipeline/adaptive.py research/youtube_pipeline/tests/test_adaptive.py
git commit -m "feat: add adaptive report planning helpers"
```

Expected: commit succeeds.

---

### Task 4: Adaptive Prompt Builders

**Files:**
- Modify: `research/youtube_pipeline/prompts.py`
- Modify: `research/youtube_pipeline/tests/test_prompts.py`

- [x] **Step 1: Write failing prompt tests**

Update imports in `test_prompts.py`:

```python
from research.youtube_pipeline.prompts import (
    build_adaptive_chapter_expansion_messages,
    build_adaptive_chapter_generation_messages,
    build_adaptive_chapter_outline_messages,
    build_adaptive_chunk_analysis_messages,
    build_adaptive_conclusion_messages,
    build_adaptive_overview_messages,
    build_chunk_analysis_messages,
    build_chunk_reduce_messages,
    build_one_shot_full_json_messages,
)
```

Add tests:

```python
    def test_adaptive_chunk_analysis_prompt_includes_substance_rubric(self):
        messages = build_adaptive_chunk_analysis_messages(
            "Chunk text",
            chunk_index=1,
            total_chunks=2,
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("substance_score", joined)
        self.assertIn("greetings, ads, sponsor reads", joined)
        self.assertIn("Use 1 and 2 when appropriate", joined)
        self.assertIn("600-1000 words", joined)
        self.assertIn("Chunk 1 of 2", joined)

    def test_adaptive_outline_prompt_uses_compact_descriptors(self):
        messages = build_adaptive_chapter_outline_messages(
            chunk_descriptors_json='[{"chunk_index":1,"summary_preview":"Short"}]',
            chapter_groups_json='[{"chapter_index":1,"assigned_chunk_indexes":[1]}]',
            report_min_words=7000,
            report_max_words=10000,
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("report_thesis", joined)
        self.assertIn("key_terms", joined)
        self.assertIn("one_liner", joined)
        self.assertIn("Do not write chapter prose", joined)
        self.assertIn("7000-10000", joined)

    def test_adaptive_chapter_generation_prompt_includes_ledger_and_target(self):
        messages = build_adaptive_chapter_generation_messages(
            chapter_index=1,
            total_chapters=2,
            chapter_word_target=900,
            assigned_notes_json='[{"chunk_index":1}]',
            outline_json='{"report_thesis":"Thesis","key_terms":["Term"],"chapters":[]}',
            previous_bridge="Previous ending",
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("900 words", joined)
        self.assertIn("Thesis", joined)
        self.assertIn("Previous ending", joined)
        self.assertIn("Markdown prose only", joined)

    def test_adaptive_expansion_prompt_requests_source_grounded_expansion(self):
        messages = build_adaptive_chapter_expansion_messages(
            chapter_index=1,
            chapter_word_target=900,
            current_word_count=300,
            chapter_draft="Short draft",
            assigned_notes_json='[{"substance_score":5}]',
            outline_entry_json='{"title":"Chapter title"}',
            report_thesis="Thesis",
            key_terms=["Term"],
            previous_bridge="Bridge",
            output_language="ru",
        )
        joined = "\n".join(message.content for message in messages)

        self.assertIn("source-grounded detail", joined)
        self.assertIn("claims, examples, evidence, timeline moments", joined)
        self.assertIn("avoid generic filler", joined)
        self.assertIn("300 words", joined)

    def test_adaptive_overview_and_conclusion_prompts_do_not_rewrite_report(self):
        overview = build_adaptive_overview_messages(
            outline_json='{"report_thesis":"Thesis"}',
            structured_result_json='{"timeline":[]}',
            output_language="ru",
        )
        conclusion = build_adaptive_conclusion_messages(
            outline_json='{"report_thesis":"Thesis"}',
            structured_result_json='{"claims":[]}',
            output_language="ru",
        )
        joined = "\n".join(message.content for message in overview + conclusion)

        self.assertIn("Do not rewrite the chapters", joined)
        self.assertIn("executive overview", joined)
        self.assertIn("final synthesis", joined)
```

- [x] **Step 2: Run prompt tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_prompts -v
```

Expected: FAIL because adaptive prompt builders do not exist.

- [x] **Step 3: Add adaptive chunk contract and prompt builders**

Append to `prompts.py`:

```python
ADAPTIVE_CHUNK_CONTRACT = """Return JSON with this shape:
{
  "substance_score": 4,
  "summary_text": "dense narrative notes for this chunk",
  "timeline": [{"start": "00:00:00", "end": "00:05:00", "title": "", "summary": ""}],
  "claims": [{"text": "", "importance": "high", "evidence_refs": []}],
  "evidence": [{"text": "", "timestamp": "00:00:00", "supports_claims": []}],
  "action_items": [{"text": "", "target_audience": "", "priority": "medium"}],
  "open_questions": [{"text": "", "why_it_matters": ""}]
}
"""


SUBSTANCE_RUBRIC = """substance_score must be an integer from 1 to 5:
1 = greetings, ads, sponsor reads, like-and-subscribe requests, technical issues, repeated intro/outro, or other filler with little analytical value.
2 = casual small talk, low-value anecdotes, logistical setup, or repetition of already covered points.
3 = coherent narrative with moderate informational density and normal interview or lecture flow, but no major new thesis or evidence cluster.
4 = specific claims backed by examples or data, novel frameworks, clear argument transitions, or meaningful practical implications.
5 = pivotal thesis statements, dense expert analysis, evidence-backed counterarguments, or sections with three or more concrete facts, figures, citations, or high-impact examples.
Use 1 and 2 when appropriate. Do not default filler or repeated material to 3.
"""
```

Add:

```python
def build_adaptive_chunk_analysis_messages(
    chunk_text: str,
    *,
    chunk_index: int,
    total_chunks: int,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You analyze one chunk of a longer YouTube transcript for an adaptive long-form report. "
                "Use only this chunk. Return one JSON object and no Markdown wrapper."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Chunk {chunk_index} of {total_chunks}\n\n"
                "Create dense chunk notes, not a short abstract. In summary_text write 600-1000 words "
                "when the chunk has enough substance. Preserve concrete details, argument flow, examples, "
                "named concepts, transitions, evidence, tensions, and practical implications.\n\n"
                f"{SUBSTANCE_RUBRIC}\n\n"
                f"{ADAPTIVE_CHUNK_CONTRACT}\n\nTranscript chunk:\n{chunk_text}"
            ),
        ),
    ]
```

Add outline, chapter, expansion, overview, and conclusion builders:

```python
def build_adaptive_chapter_outline_messages(
    *,
    chunk_descriptors_json: str,
    chapter_groups_json: str,
    report_min_words: int,
    report_max_words: int,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You create a compact chapter outline for a long-form YouTube research report. "
                "Return JSON only. Do not write chapter prose."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Target report length range: {report_min_words}-{report_max_words} words\n\n"
                "Create a report_thesis, key_terms, and one chapter entry per provided chapter group. "
                "Each chapter needs chapter_index, title, one_liner, and assigned_chunk_indexes.\n\n"
                "Return exactly this JSON shape:\n"
                "{\n"
                '  "report_thesis": "One-sentence throughline",\n'
                '  "key_terms": ["term"],\n'
                '  "chapters": [{"chapter_index": 1, "title": "Title", "one_liner": "Purpose", "assigned_chunk_indexes": [1]}]\n'
                "}\n\n"
                f"Chapter groups JSON:\n{chapter_groups_json}\n\n"
                f"Chunk descriptors JSON:\n{chunk_descriptors_json}"
            ),
        ),
    ]


def build_adaptive_chapter_generation_messages(
    *,
    chapter_index: int,
    total_chapters: int,
    chapter_word_target: int,
    assigned_notes_json: str,
    outline_json: str,
    previous_bridge: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You write one chapter of a long-form YouTube research report. "
                "Markdown prose only. Do not return JSON."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Chapter {chapter_index} of {total_chapters}\n"
                f"Target length: about {chapter_word_target} words\n\n"
                "Use the report thesis, key terms, full chapter outline, assigned chunk notes, "
                "and previous chapter bridge to write a coherent source-grounded chapter. "
                "Do not summarize into a short abstract. Preserve concrete claims, examples, evidence, and transitions.\n\n"
                f"Chapter outline JSON:\n{outline_json}\n\n"
                f"Previous chapter bridge:\n{previous_bridge}\n\n"
                f"Assigned chunk notes JSON:\n{assigned_notes_json}"
            ),
        ),
    ]


def build_adaptive_chapter_expansion_messages(
    *,
    chapter_index: int,
    chapter_word_target: int,
    current_word_count: int,
    chapter_draft: str,
    assigned_notes_json: str,
    outline_entry_json: str,
    report_thesis: str,
    key_terms: list[str],
    previous_bridge: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content=(
                "You expand one chapter of a long-form research report using only source-grounded detail. "
                "Return Markdown prose only."
            ),
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n"
                f"Chapter {chapter_index}\n"
                f"Current length: {current_word_count} words\n"
                f"Target length: about {chapter_word_target} words\n\n"
                "Revise the chapter by adding source-grounded detail that was missing or thinly covered. "
                "Look for claims, examples, evidence, timeline moments, unresolved questions, and high-substance notes. "
                "Avoid generic filler, repeated phrasing, and abstract restatement that does not add concrete detail.\n\n"
                f"Report thesis:\n{report_thesis}\n\n"
                f"Key terms:\n{', '.join(key_terms)}\n\n"
                f"Previous chapter bridge:\n{previous_bridge}\n\n"
                f"Chapter outline entry JSON:\n{outline_entry_json}\n\n"
                f"Current chapter draft:\n{chapter_draft}\n\n"
                f"Assigned chunk notes JSON:\n{assigned_notes_json}"
            ),
        ),
    ]


def build_adaptive_overview_messages(
    *,
    outline_json: str,
    structured_result_json: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content="You write an executive overview for a long-form YouTube research report.",
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Write a concise executive overview. Do not rewrite the chapters. "
                "Use the outline and structured result to frame the report.\n\n"
                f"Outline JSON:\n{outline_json}\n\n"
                f"Structured result JSON:\n{structured_result_json}"
            ),
        ),
    ]


def build_adaptive_conclusion_messages(
    *,
    outline_json: str,
    structured_result_json: str,
    output_language: str,
) -> list[ChatMessage]:
    return [
        ChatMessage(
            role="system",
            content="You write the final synthesis for a long-form YouTube research report.",
        ),
        ChatMessage(
            role="user",
            content=(
                f"Output language: {output_language}\n\n"
                "Write a final synthesis. Do not rewrite the chapters. "
                "Connect the main claims, evidence, takeaways, and open questions.\n\n"
                f"Outline JSON:\n{outline_json}\n\n"
                f"Structured result JSON:\n{structured_result_json}"
            ),
        ),
    ]
```

- [x] **Step 4: Run prompt tests**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_prompts -v
```

Expected: PASS.

- [x] **Step 5: Run all tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: PASS.

- [x] **Step 6: Commit**

Run:

```powershell
git add research/youtube_pipeline/prompts.py research/youtube_pipeline/tests/test_prompts.py
git commit -m "feat: add adaptive report prompt builders"
```

Expected: commit succeeds.

---

### Task 5: Adaptive Strategy Orchestration

**Files:**
- Modify: `research/youtube_pipeline/strategies.py`
- Modify: `research/youtube_pipeline/tests/test_strategies.py`

- [x] **Step 1: Write failing adaptive strategy test**

Update imports in `test_strategies.py`:

```python
from research.youtube_pipeline.strategies import (
    StrategyOptions,
    run_adaptive_book_report,
    run_antigravity_chunk_map_reduce,
    run_chunk_map_reduce,
    run_one_shot_full_json,
)
```

Add a helper response generator near `SequenceClient`:

```python
def normalized_chunk(summary, score=3):
    return (
        '{"substance_score":'
        + str(score)
        + ',"summary_text":"'
        + summary
        + '","timeline":[],"claims":[],"evidence":[],"action_items":[],"open_questions":[]}'
    )
```

Add test:

```python
    def test_adaptive_book_report_generates_chapters_expands_and_assembles(self):
        transcript = " ".join(f"word{i}" for i in range(1200))
        long_chapter = " ".join(f"chapter2_{i}" for i in range(700))
        expanded_chapter = " ".join(f"expanded1_{i}" for i in range(700))
        client = SequenceClient(
            [
                normalized_chunk("Chunk one dense notes", 3),
                normalized_chunk("Chunk two dense notes", 3),
                normalized_chunk("Chunk three dense notes", 3),
                normalized_chunk("Chunk four dense notes", 3),
                (
                    '{"report_thesis":"Main thesis","key_terms":["Term"],'
                    '"chapters":['
                    '{"chapter_index":1,"title":"First arc","one_liner":"Covers first half","assigned_chunk_indexes":[1,2]},'
                    '{"chapter_index":2,"title":"Second arc","one_liner":"Covers second half","assigned_chunk_indexes":[3,4]}'
                    ']}'
                ),
                "Too short",
                expanded_chapter,
                long_chapter,
                '{"timeline":[{"start":"00:00:00","end":"00:05:00","title":"T1","summary":"S1"}]}',
                '{"claims":[{"text":"C1","importance":"high","evidence_refs":[]}],"evidence":[]}',
                '{"action_items":[{"text":"A1","target_audience":"Audience","priority":"medium"}],"open_questions":[]}',
                "Executive overview text",
                "Final synthesis text",
            ]
        )

        outcome = run_adaptive_book_report(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                max_tokens=5000,
                chunk_token_limit=300,
                chapter_target_words=900,
            ),
        )

        self.assertIn("Generated via `adaptive_book_report`", outcome.result.summary_text)
        self.assertIn("First arc", outcome.result.summary_text)
        self.assertIn("Second arc", outcome.result.summary_text)
        self.assertIn("Executive overview text", outcome.result.summary_text)
        self.assertIn("Final synthesis text", outcome.result.summary_text)
        self.assertIn("expanded1_0", outcome.result.summary_text)
        self.assertEqual(outcome.result.timeline[0].title, "T1")
        self.assertEqual(outcome.result.claims[0].text, "C1")
        self.assertEqual(outcome.result.action_items[0].text, "A1")
        self.assertEqual(outcome.request_count, 13)
        self.assertTrue(outcome.json_valid)
        self.assertEqual(outcome.extra_metrics["strategy_variant"], "adaptive_book_report")
        self.assertEqual(outcome.extra_metrics["chapter_count"], 2)
        self.assertEqual(outcome.extra_metrics["expansion_call_count"], 1)
        self.assertEqual(outcome.extra_metrics["target_report_words"], 1400)
        self.assertFalse(outcome.extra_metrics["outline_fallback_used"])
        self.assertFalse(outcome.extra_metrics["chapter_expansion_shortfall"])
        self.assertTrue(outcome.extra_metrics["substance_score_calibration_warning"])
        chapter_generation_call = client.calls[5]
        self.assertEqual(chapter_generation_call[1], 2254)

    def test_adaptive_book_report_records_outline_fallback_and_expansion_shortfall(self):
        transcript = " ".join(f"word{i}" for i in range(1200))
        still_short = " ".join(f"short_{i}" for i in range(100))
        client = SequenceClient(
            [
                normalized_chunk("Chunk one dense notes", 1),
                normalized_chunk("Chunk two dense notes", 2),
                normalized_chunk("Chunk three dense notes", 3),
                normalized_chunk("Chunk four dense notes", 4),
                "{not valid json",
                "Tiny chapter",
                still_short,
                "Second chapter has enough words " * 140,
                '{"timeline":[]}',
                '{"claims":[],"evidence":[]}',
                '{"action_items":[],"open_questions":[]}',
                "Overview",
                "Conclusion",
            ]
        )

        outcome = run_adaptive_book_report(
            client=client,
            transcript=transcript,
            options=StrategyOptions(
                output_language="ru",
                max_tokens=5000,
                chunk_token_limit=300,
                chapter_target_words=900,
            ),
        )

        self.assertTrue(outcome.extra_metrics["outline_fallback_used"])
        self.assertTrue(outcome.extra_metrics["chapter_expansion_shortfall"])
        self.assertFalse(outcome.extra_metrics["substance_score_calibration_warning"])
```

The expected `2254` max token value is `ceil(700 * 2.8 * 1.15)`, using the language-aware Russian output token budget.

Add short-input test:

```python
    def test_adaptive_book_report_rejects_empty_transcript(self):
        client = FakeClient()

        with self.assertRaisesRegex(ValueError, "transcript is empty"):
            run_adaptive_book_report(
                client=client,
                transcript="   ",
                options=StrategyOptions(),
            )

        self.assertEqual(client.calls, [])

    def test_adaptive_book_report_uses_one_shot_for_short_transcript(self):
        client = FakeClient()

        outcome = run_adaptive_book_report(
            client=client,
            transcript="short transcript",
            options=StrategyOptions(output_language="ru", max_tokens=1000),
        )

        self.assertEqual(outcome.result.summary_text, "Summary text")
        self.assertEqual(outcome.request_count, 1)
        self.assertEqual(outcome.extra_metrics["strategy_variant"], "adaptive_book_report_short_fallback")
        self.assertEqual(outcome.extra_metrics["transcript_words"], 2)
```

Update registry test expected list to include `"adaptive_book_report"`.

- [x] **Step 2: Run strategy tests to verify they fail**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: FAIL because `run_adaptive_book_report` is missing.

- [x] **Step 3: Import adaptive helpers and prompts**

In `strategies.py`, add imports:

```python
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
```

Add prompt imports:

```python
    build_adaptive_chapter_expansion_messages,
    build_adaptive_chapter_generation_messages,
    build_adaptive_chapter_outline_messages,
    build_adaptive_chunk_analysis_messages,
    build_adaptive_conclusion_messages,
    build_adaptive_overview_messages,
```

- [x] **Step 4: Add JSON helpers and Markdown section helpers**

Add these helpers above the strategy functions:

```python
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
        lines.append(f"- **{item.start}-{item.end}**: {item.title} — {item.summary}")
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
    return "\n".join(f"- {item.text} — {item.why_it_matters}" for item in result.open_questions)


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
```

- [x] **Step 5: Implement `run_adaptive_book_report`**

Add the strategy below `run_antigravity_chunk_map_reduce`. The implementation should follow this exact sequence:

1. Reject empty transcript.
2. Use one-shot fallback for transcripts shorter than 1,000 words.
3. Split chunks with `options.chunk_token_limit`.
4. Analyze each chunk with `build_adaptive_chunk_analysis_messages`.
5. Normalize `substance_score` and collect chunk results.
6. Compute `BudgetPlan`.
7. Partition chunks with DP.
8. Build compact outline descriptors and chapter group JSON.
9. Make outline call and fall back when invalid.
10. Generate each chapter, expand if shorter than `0.8 * chapter_word_target`, and update previous bridge from final text.
11. Reuse antigravity timeline, claims/evidence, and takeaways reductions.
12. Generate overview and conclusion.
13. Assemble final Markdown in Python.
14. Return `StrategyOutcome` with `extra_metrics`.

Use this structure:

```python
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
```

Continue inside the function with chunk analysis:

```python
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
```

Then budget and chapter groups:

```python
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
```

Outline and fallback:

```python
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
```

Chapters and expansion:

```python
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
```

Structured reductions and assembly:

```python
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
```

Return:

```python
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
```

- [x] **Step 6: Register strategy**

Add to `STRATEGIES`:

```python
"adaptive_book_report": run_adaptive_book_report,
```

- [x] **Step 7: Run strategy tests**

Run:

```powershell
python -m unittest research.youtube_pipeline.tests.test_strategies -v
```

Expected: PASS.

- [x] **Step 8: Run all tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: PASS.

- [x] **Step 9: Commit**

Run:

```powershell
git add research/youtube_pipeline/strategies.py research/youtube_pipeline/tests/test_strategies.py
git commit -m "feat: implement adaptive book report strategy"
```

Expected: commit succeeds.

---

### Task 6: README And Usage Examples

**Files:**
- Modify: `research/youtube_pipeline/README.md`

- [x] **Step 1: Update available strategy list**

Add `adaptive_book_report` to the available strategies block:

```text
adaptive_book_report
antigravity_chunk_map_reduce
one_shot_full_json
one_shot_markdown_plus_json
two_pass_summary_structure
chunk_map_reduce
timeline_segment_reduce
```

- [x] **Step 2: Add adaptive example**

Add this example after the Tucker Carlson example:

```markdown
Adaptive book report for a very long transcript:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id f_lRdkH_QoY `
  --strategy adaptive_book_report `
  --output-language ru `
  --target-depth auto `
  --chunk-token-limit 3000 `
  --chapter-target-words 900 `
  --max-tokens 8192
```
```

- [x] **Step 3: Document adaptive flags**

Add:

```markdown
## Adaptive Book Report Flags

- `--target-depth auto|brief|standard|deep|book`: controls the report budget multiplier.
- `--min-report-words`: optional lower override for report budget.
- `--max-report-words`: optional upper override for report budget.
- `--chapter-target-words`: target words used to derive chapter count; default is `900`.
- `--chunk-token-limit`: approximate chunk size used by chunked strategies; default is `3000`.

For Russian output, the strategy uses a larger output-token budget for chapter
generation and expansion because Cyrillic text usually takes more tokens per
word than English.
```

- [x] **Step 4: Run all tests**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: PASS.

- [x] **Step 5: Commit**

Run:

```powershell
git add research/youtube_pipeline/README.md
git commit -m "docs: add adaptive book report usage"
```

Expected: commit succeeds.

---

### Task 7: Final Verification And Optional Manual Run

**Files:**
- No required source changes.
- Manual run artifacts under `research/youtube_pipeline/runs/` are gitignored.

- [ ] **Step 1: Run the full unit test suite**

Run:

```powershell
python -m unittest discover -s research/youtube_pipeline/tests -v
```

Expected: all tests pass.

- [ ] **Step 2: Check registered strategy through CLI help**

Run:

```powershell
python -m research.youtube_pipeline.runner --help
```

Expected: help output includes:

```text
adaptive_book_report
--target-depth
--chunk-token-limit
--chapter-target-words
```

- [ ] **Step 3: Optional manual LLM run for Tucker transcript**

Only run this when the local LLM endpoint and environment variables are available:

```powershell
python -m research.youtube_pipeline.runner `
  --input research/youtube_pipeline/inputs/tucker_carlson_f_lRdkH_QoY_en.txt `
  --video-id f_lRdkH_QoY `
  --strategy adaptive_book_report `
  --output-language ru `
  --target-depth auto `
  --chunk-token-limit 3000 `
  --chapter-target-words 900 `
  --max-tokens 8192
```

Expected:

- `metrics.json` has `strategy` set to `adaptive_book_report`.
- `metrics.json` has `summary_words` closer to the computed adaptive range than the old `antigravity_chunk_map_reduce` 3,656-word run.
- `metrics.json` includes `chapter_count`, `chapter_word_target`, `target_report_words`, `expansion_call_count`, `average_substance_score`, and `substance_multiplier`.
- `result.md` contains `Generated via adaptive_book_report`, a table of contents, chapter headings, structured analysis sections, and final synthesis.

- [ ] **Step 4: Check git status**

Run:

```powershell
git status --short
```

Expected: generated run artifacts are not listed. If manual run artifacts appear, update `.gitignore` before finishing.

---

## Self-Review Checklist

- Spec coverage:
  - Adaptive budget ranges, depth modes, hard caps, midpoint target, and chapter targets are covered by Task 3.
  - Anchored `substance_score` rubric is covered by Task 4.
  - DP chapter partitioning is covered by Task 3.
  - Chapter outline, context ledger, previous bridge, chapter generation, and expansion guard are covered by Tasks 3-5.
  - Separate structured reductions and Python assembly are covered by Task 5.
  - Runner flags, `StrategyOptions`, and `extra_metrics` are covered by Tasks 1-2.
  - README usage is covered by Task 6.
- Verification:
  - Every task has a targeted test command and a full-suite checkpoint where appropriate.
  - Manual LLM run is optional and explicitly separated from unit verification.
- Risk notes:
  - `run_adaptive_book_report` is intentionally research-only and may be verbose.
  - Existing `antigravity_chunk_map_reduce` remains registered as a baseline.
  - JSON repair/retry is not included in this plan because the approved spec explicitly leaves it for a later reliability improvement.
