# YouTube Adaptive Book Report Design

Date: 2026-06-18

Status: draft for user review.

## Table of Contents

- [Goal](#goal)
- [Non-Goals](#non-goals)
- [Strategy Summary](#strategy-summary)
- [Adaptive Length Policy](#adaptive-length-policy)
- [Substance-Aware Planning](#substance-aware-planning)
- [Chapter Partitioning](#chapter-partitioning)
- [Chapter Outline](#chapter-outline)
- [Chapter Generation](#chapter-generation)
- [Chapter Expansion Guard](#chapter-expansion-guard)
- [Structured Reductions](#structured-reductions)
- [Overview and Conclusion](#overview-and-conclusion)
- [Python Markdown Assembly](#python-markdown-assembly)
- [Strategy Options and Runner Integration](#strategy-options-and-runner-integration)
- [Metrics](#metrics)
- [Edge Cases](#edge-cases)
- [Testing Strategy](#testing-strategy)
- [Expected Outcome](#expected-outcome)

## Goal

Add a new Python research strategy named `adaptive_book_report` for long
YouTube transcripts. The strategy should produce a long-form analytical report
whose length adapts to the transcript size and information density.

The immediate goal is to improve on the current `antigravity_chunk_map_reduce`
result. That strategy successfully produced a valid structured result and a
much longer report than the earlier `chunk_map_reduce` runs, but the Tucker
Carlson transcript still produced about 3,656 words rather than the desired
8,000-10,000 word report for a very long, dense source.

The design should avoid relying on one final large LLM response to write or
rewrite the whole report. Long prose should be generated chapter by chapter and
assembled programmatically in Python.

## Non-Goals

- No production integration with the Rust/Tauri pipeline.
- No database reads or writes.
- No UI changes.
- No automatic provider selection.
- No attempt to force every transcript to a fixed 10,000-word report.
- No final LLM rewrite pass that can compress the generated chapters.
- No replacement or removal of `antigravity_chunk_map_reduce`; it remains a
  baseline for comparison.
- No substance-based chunk skipping in the first implementation. Low-substance
  chunks may receive less budget, but they should not be dropped.
- No unbounded `book` mode. Very deep reports should still respect default hard
  caps unless the code is intentionally changed for a special research run.
- No heavy multi-agent long-form writing system in the first implementation.
  The strategy may keep a lightweight context ledger, but should avoid adding
  broad state-management machinery before manual runs prove it is needed.

## Strategy Summary

`adaptive_book_report` is an extension of the current antigravity direction:

```text
transcript
  -> token chunks
  -> dense chunk notes with substance scores
  -> Python budget and weighted chapter partition
  -> lightweight chapter outline
  -> chapter-by-chapter report generation with outline context
  -> per-chapter expansion when too short
  -> separate structured reductions
  -> short overview and conclusion generation
  -> Python Markdown assembly
```

The core change is that the target length is enforced through smaller,
measured chapter outputs instead of a single prompt asking for a long report.
The existing `antigravity_chunk_map_reduce` strategy should stay unchanged as a
baseline; `adaptive_book_report` is a separate strategy.

## Adaptive Length Policy

The strategy computes a base report budget from transcript word count:

| Transcript words | Base report words | Base chapter count |
| --- | ---: | ---: |
| `< 5,000` | 1,000-1,800 | 1-2 |
| `5,000-15,000` | 2,000-3,500 | 2-4 |
| `15,000-35,000` | 4,000-6,500 | 4-7 |
| `35,000-70,000` | 7,000-10,000 | 8-11 |
| `>= 70,000` | 10,000-14,000 | 11-15 |

The default `auto` mode uses this base table directly. Explicit depth modes
apply a multiplier to the base budget before hard caps:

| Mode | Multiplier | Intended use |
| --- | ---: | --- |
| `brief` | `0.5x` | Quick review and cheap runs |
| `standard` | `1.0x` | Default depth for most research |
| `deep` | `1.5x` | More complete long-video review |
| `book` | `2.0x` | Maximum-depth exploratory run |

User-provided `--min-report-words` and `--max-report-words` override the
computed bounds within the default hard caps. `--chapter-target-words` defaults
to `900`.

The first implementation should apply these hard caps after depth and substance
adjustments:

```text
max_report_words = 20,000
max_chapters = 20
```

These caps keep `book` mode from accidentally producing 30+ chapter runs with
50-60 LLM calls. If a later experiment needs more than 20,000 words, that
should be an explicit research decision rather than an accidental CLI setting.

For the Tucker transcript at roughly 41,000 words, `auto` should target roughly
7,000-10,000 report words and about 8-11 chapters before substance adjustment.

## Substance-Aware Planning

Each adaptive chunk analysis uses a dedicated JSON contract that extends the
normal normalized result with a `substance_score`. This should be implemented
as a new prompt builder rather than by silently relying on the existing
`RESULT_CONTRACT`, because the current shared contract does not contain
`substance_score`.

The expected chunk analysis shape is:

```json
{
  "substance_score": 4,
  "summary_text": "Dense narrative notes for this chunk...",
  "timeline": [],
  "claims": [],
  "evidence": [],
  "action_items": [],
  "open_questions": []
}
```

`substance_score` uses a 1-5 scale:

- `1`: repetitive, low-information, housekeeping, or filler.
- `3`: normal conversational value with useful but moderate density.
- `5`: dense expert analysis, important claims, evidence, examples, or
  argument transitions.

Python computes the average substance score:

```text
substance_multiplier = 0.7 + 0.6 * ((average_score - 1) / 4)
```

This creates a multiplier from `0.7x` to `1.3x`. Dense transcripts get more
report budget; sparse or repetitive transcripts get less. The formula is
empirical and may need per-model tuning after manual research runs.

If `substance_score` is absent, non-numeric, or outside the 1-5 range, Python
should default it to `3` and clamp parsed numeric values into `1..5`. If more
than 80% of chunks receive the same score, the strategy should add a warning to
its extra metrics or notes because that may indicate weak calibration.

## Chapter Partitioning

After computing the adjusted target word count, Python computes chapter count:

```text
chapter_count = max(1, round(target_report_words / chapter_target_words))
```

Then it clamps the chapter count:

```text
chapter_count = min(chapter_count, max_chapters, chunk_count)
```

Chunks are partitioned into contiguous chapter groups. The target is to balance
chapter weight rather than raw chunk count:

```text
chunk_weight = chunk_word_count * substance_score
```

This keeps dense regions in smaller, more focused chapters while merging less
dense transcript regions into broader chapters.

The first implementation should prefer a dynamic programming contiguous
partitioner because chapter balance matters for readability. The objective is
to split ordered chunks into `chapter_count` groups while minimizing squared
deviation from the target chapter weight:

```text
minimize sum((chapter_weight - target_weight) ** 2)
```

The partitioner must preserve transcript order and keep at least one chunk per
chapter. This is inexpensive for research transcripts because chunk counts are
small. A greedy contiguous partitioner can remain as a fallback if the DP
implementation is postponed, but the implementation plan should treat DP as
the target.

If a greedy fallback is used, it should:

1. Compute total chunk weight.
2. Compute target weight per chapter.
3. Add chunks to the current chapter until the target is reached.
4. Start the next chapter unless doing so would violate the explicit constraint
   that each remaining chapter must receive at least one chunk.
5. Always preserve original transcript order.

## Chapter Outline

After Python computes chapter groups, the strategy should make one lightweight
LLM call to create a chapter outline. This call is for coherence, titles, and a
compact context shared by chapter prompts. It should not generate long prose.

Input:

- chapter index and assigned chunk indexes for every chapter;
- short chunk descriptors derived from chunk analysis results;
- the target word range and output language.

The outline response should be JSON:

```json
{
  "report_thesis": "One-sentence throughline for the whole report.",
  "key_terms": ["important recurring term or entity"],
  "chapters": [
    {
      "chapter_index": 1,
      "title": "Descriptive chapter title",
      "one_liner": "What this chapter covers and how it connects to the whole report.",
      "assigned_chunk_indexes": [1, 2]
    }
  ]
}
```

If the outline JSON is invalid, the strategy should fall back to deterministic
chapter titles such as `Chapter 1`, `Chapter 2`, and one-liners derived from
the assigned chunk summaries. This failure should be recorded in extra metrics
or notes.

The outline also seeds a lightweight context ledger. For v1, this ledger should
stay small and deterministic:

- `report_thesis` from the outline;
- `key_terms` from the outline;
- generated chapter titles and one-liners;
- the final paragraph of the previous generated chapter.

The ledger is not a separate long memory system. It exists only to keep chapter
prompts aligned without sending all chunk notes to every chapter call.

## Chapter Generation

Each chapter is generated in its own LLM request. The prompt receives:

- chapter index and total chapter count;
- target word count for the chapter;
- assigned chunk indexes;
- dense chunk notes for the assigned chunks;
- the full chapter outline from the lightweight outline call;
- the lightweight context ledger;
- the previous chapter bridge, when available.

The chapter prompt must not receive all chunk summaries or all dense chunk
notes for the entire transcript. For the first implementation, the context
policy is:

```text
chapter prompt context =
  assigned chapter chunk notes
  + full chapter outline
  + report thesis and key terms
  + previous chapter title and final paragraph, if any
```

This keeps input cost bounded while preserving a shared structure across
chapters. The strategy should also keep a maximum approximate input budget for
each chapter call. If assigned notes are too large, it should prefer compact
fields from chunk analysis results over raw transcript text.

The model writes Markdown prose only. It must not return JSON.

Expected chapter shape:

```markdown
## Chapter 3: Descriptive Chapter Title

...
```

This stage should aim for coverage-preserving analytical prose rather than a
short abstract. It should explain argument flow, examples, claims, tensions,
and transitions visible in the assigned notes.

## Chapter Expansion Guard

After every chapter generation call, Python counts words in the chapter. If the
chapter is shorter than the minimum acceptable length, the strategy performs an
expansion call for that chapter only.

Default rule:

```text
expand if generated_words < 0.8 * chapter_target_words
```

The expansion prompt receives the current chapter draft, the target length, and
the original source notes. It asks the model to produce a fuller revised chapter
by expanding underdeveloped topics, examples, and transitions from the notes.
The expansion must be factual rather than stylistic. The prompt should ask the
model to identify claims, examples, evidence, timeline moments, or unresolved
questions from the assigned notes that were missing or thinly covered in the
draft, then integrate those anchors into the revised chapter. It should
explicitly avoid generic filler, repeated phrasing, and abstract restatement
that does not add source-grounded detail.

The strategy should cap expansion attempts with
`max_expansions_per_chapter = 1` for the first implementation. This keeps cost
bounded and makes run behavior easy to compare. If the expansion response is
still shorter than the target, the strategy should use the best available
chapter text and record the shortfall in extra metrics or notes.

## Structured Reductions

Long prose and structured JSON should stay separate. After chunk analysis, the
strategy performs independent structured reductions:

- timeline reduction;
- claims and evidence reduction;
- action items and open questions reduction.

The first implementation should reuse the existing antigravity reduce prompt
builders where practical. This keeps the new strategy focused on adaptive
budgeting, chapter generation, and Python assembly.

JSON repair/retry remains a separate reliability improvement. The new strategy
should be compatible with it later, but does not need to implement repair in
the first pass.

## Overview and Conclusion

The strategy may generate a short executive overview and final synthesis in
separate LLM calls. These calls should not receive the entire long report when
the report is large.

Preferred inputs:

- chapter outline;
- chapter titles;
- short chapter summaries or the dense chunk summaries;
- structured result JSON.

The overview and conclusion should frame the report, not rewrite or compress
the generated chapters.

## Python Markdown Assembly

The final Markdown report is assembled in Python. The LLM should not receive
one final instruction to rewrite the whole report.

Suggested report shape:

```markdown
# <video_id> Research Report

Generated via `adaptive_book_report`.

## Table of Contents

## Executive Overview

# Part I: Detailed Narrative

## Chapter 1: ...
## Chapter 2: ...

# Part II: Structured Analysis

## Timeline and Development of Ideas
## Major Claims and Evidence
## Actionable Takeaways
## Open Questions

## Conclusion and Synthesis
```

`result.summary_text` should contain this assembled Markdown report.
`result.timeline`, `result.claims`, `result.evidence`,
`result.action_items`, and `result.open_questions` should come from the
structured reductions.

## Strategy Options and Runner Integration

Extend `research/youtube_pipeline/runner.py` with:

```text
--target-depth auto|brief|standard|deep|book
--min-report-words <int>
--max-report-words <int>
--chapter-target-words <int>
--chunk-token-limit <int>
```

`--chunk-token-limit` should become a real CLI option for both the current
chunked strategies and the new strategy. It already exists as a strategy
parameter but is not exposed by the runner.

To avoid fragile strategy-name branching or `inspect.signature()` logic, add a
shared options dataclass in `strategies.py`:

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

The runner should construct `StrategyOptions` once and pass it to every
strategy. Existing strategies can ignore fields they do not use. If converting
all strategies at once creates too much churn, the first implementation may use
a compatibility wrapper, but the end state should be one shared options object.

## Metrics

Existing metrics stay useful:

- request count;
- input tokens;
- output tokens;
- latency;
- summary word count;
- structured field counts;
- JSON validity.

Add strategy-specific notes when available:

- computed transcript word count;
- target report word range;
- actual report word count;
- chapter count;
- expansion call count;
- average substance score.

Extend `StrategyOutcome` with an optional `extra_metrics` field:

```python
extra_metrics: dict[str, object] = field(default_factory=dict)
```

`write_run_artifacts()` should merge `extra_metrics` into `metrics.json` after
the standard metrics are built. This is cleaner than encoding structured
details inside `metrics["notes"]`.

## Edge Cases

The first implementation should handle these cases explicitly:

| Scenario | Behavior |
| --- | --- |
| Transcript has fewer than 1,000 words | Use `one_shot_full_json` or a single compact chapter to avoid inflating thin material. |
| `substance_score` is missing or invalid | Default to `3`, clamp numeric values into `1..5`, and record a warning. |
| More than 80% of chunks have identical score | Continue, but record a calibration warning in `extra_metrics`. |
| All chunks have low substance | Reduce budget through the substance multiplier, but do not skip chunks in v1. |
| Expansion still misses target | Use the best available text and record the shortfall. |
| Chapter notes exceed the approximate input budget | Prefer compact chunk analysis fields over raw transcript text. |
| Chapter prose repeats earlier chapters | Use the outline, key terms, and previous final paragraph bridge to redirect the next chapter, but do not run a full-report rewrite. |

## Testing Strategy

Use mocked LLM clients. Tests should cover:

- adaptive budget selection from transcript word count;
- depth multipliers and explicit min/max overrides;
- hard caps for report words and chapter count;
- substance multiplier calculation;
- invalid substance score fallback and clamping;
- dynamic programming contiguous weighted chunk partitioning;
- one chunk per remaining chapter partition constraint;
- chapter outline fallback when outline JSON is invalid;
- context ledger uses outline thesis, key terms, and previous final paragraph;
- strategy registration under `adaptive_book_report`;
- chapter expansion call occurs when a chapter is too short;
- expansion prompt receives source-grounded missing-detail anchors;
- chapter expansion is skipped when the chapter is long enough;
- final result summary is assembled from generated chapters, not a final
  rewrite response;
- `extra_metrics` are written into `metrics.json`;
- runner passes the new CLI options.

Live LLM runs remain manual research validation, not unit tests.

## Expected Outcome

For short transcripts, the strategy should avoid bloated output. For very long
transcripts like the Tucker Carlson interview transcript
`f_lRdkH_QoY` at roughly 41,000 words, `auto` should produce a report in the
7,000-10,000 word range when the source is sufficiently dense. The strategy
should make long output more reliable by distributing the target across
chapters and measuring each chapter before assembly.
