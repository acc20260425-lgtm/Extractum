# YouTube Adaptive Book Report Design

Date: 2026-06-18

Status: draft for user review.

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

## Strategy Summary

`adaptive_book_report` is an extension of the current antigravity direction:

```text
transcript
  -> token chunks
  -> dense chunk notes with substance scores
  -> Python budget and chapter plan
  -> chapter-by-chapter report generation
  -> per-chapter expansion when too short
  -> separate structured reductions
  -> short overview and conclusion generation
  -> Python Markdown assembly
```

The core change is that the target length is enforced through smaller,
measured chapter outputs instead of a single prompt asking for a long report.

## Adaptive Length Policy

The strategy computes a base report budget from transcript word count:

| Transcript words | Base report words | Base chapter count |
| --- | ---: | ---: |
| `< 5,000` | 1,000-1,800 | 1-2 |
| `5,000-15,000` | 2,000-3,500 | 2-4 |
| `15,000-35,000` | 4,000-6,500 | 4-7 |
| `35,000-70,000` | 7,000-10,000 | 8-11 |
| `>= 70,000` | 10,000-14,000 | 11-15 |

The default `auto` mode uses this base table. Explicit depth modes apply a
multiplier:

| Mode | Multiplier | Intended use |
| --- | ---: | --- |
| `brief` | `0.5x` | Quick review and cheap runs |
| `standard` | `1.0x` | Default depth for most research |
| `deep` | `1.5x` | More complete long-video review |
| `book` | `2.0x` | Maximum-depth exploratory run |

User-provided `--min-report-words` and `--max-report-words` override the
computed bounds. `--chapter-target-words` defaults to `900`.

For the Tucker transcript at roughly 41,000 words, `auto` should target roughly
7,000-10,000 report words and about 8-11 chapters before substance adjustment.

## Substance-Aware Planning

Each chunk analysis returns dense notes plus a `substance_score`:

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
report budget; sparse or repetitive transcripts get less.

## Chapter Partitioning

After computing the adjusted target word count, Python computes chapter count:

```text
chapter_count = max(1, round(target_report_words / chapter_target_words))
```

Chunks are partitioned into contiguous chapter groups. The target is to balance
chapter weight rather than raw chunk count:

```text
chunk_weight = chunk_word_count * substance_score
```

This keeps dense regions in smaller, more focused chapters while merging less
dense transcript regions into broader chapters.

The first implementation can use a greedy contiguous partitioner:

1. Compute total chunk weight.
2. Compute target weight per chapter.
3. Add chunks to the current chapter until the target is reached.
4. Start the next chapter unless doing so would leave too few chunks for the
   remaining chapters.
5. Always preserve original transcript order.

## Chapter Generation

Each chapter is generated in its own LLM request. The prompt receives:

- chapter index and total chapter count;
- target word count for the chapter;
- assigned chunk indexes;
- dense chunk notes for the assigned chunks;
- a compact overall context built from all chunk summaries or the generated
  chapter plan.

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

The strategy should cap expansion attempts with
`max_expansions_per_chapter = 1` for the first implementation. This keeps cost
bounded and makes run behavior easy to compare.

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

## Overview And Conclusion

The strategy may generate a short executive overview and final synthesis in
separate LLM calls. These calls should not receive the entire long report when
the report is large.

Preferred inputs:

- adaptive chapter plan;
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

## CLI Changes

Extend `research/youtube_pipeline/runner.py` with:

```text
--target-depth auto|brief|standard|deep|book
--min-report-words <int>
--max-report-words <int>
--chapter-target-words <int>
--chunk-token-limit <int>
```

The runner should pass these options only to strategies that accept them, or
strategy functions should accept a shared options object. The first
implementation can use keyword parameters with conservative defaults.

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

These can initially live in `metrics["notes"]` as compact JSON text if changing
the metrics schema would slow the prototype.

## Testing Strategy

Use mocked LLM clients. Tests should cover:

- adaptive budget selection from transcript word count;
- depth multipliers and explicit min/max overrides;
- substance multiplier calculation;
- contiguous weighted chunk partitioning;
- strategy registration under `adaptive_book_report`;
- chapter expansion call occurs when a chapter is too short;
- chapter expansion is skipped when the chapter is long enough;
- final result summary is assembled from generated chapters, not a final
  rewrite response;
- runner passes the new CLI options.

Live LLM runs remain manual research validation, not unit tests.

## Expected Outcome

For short transcripts, the strategy should avoid bloated output. For very long
transcripts like `f_lRdkH_QoY`, `auto` should produce a report in the
7,000-10,000 word range when the source is sufficiently dense. The strategy
should make long output more reliable by distributing the target across
chapters and measuring each chapter before assembly.

