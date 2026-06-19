# YouTube Summary Python Research Design

Date: 2026-06-17

Status: draft approved for research planning.

## Goal

Build a simplified Python equivalent of the current `youtube_summary` LLM
pipeline for research only. The prototype will not use the Extractum database,
Tauri runtime, prompt-pack validation, migrations, UI, or projection tables.

The purpose is to compare LLM pipeline shapes and learn which one produces the
best full analytical result for long YouTube transcripts:

- detailed readable summary;
- timeline;
- claims;
- evidence;
- action items;
- open questions.

The research should answer:

- how many LLM requests are needed for useful long-video output;
- when a one-shot request is enough;
- when chunking or segment-based MapReduce becomes necessary;
- how much each strategy costs in tokens, latency, and request count;
- which strategy should become the candidate for the next production pipeline.

## Baseline Context

The current production pipeline already supports a `youtube_summary` prompt pack
with a transcript-analysis stage and a synthesis stage. It also has a
`detailed_report` control preset that asks for a full Markdown report in
`summary_text` and uses a larger transcript-analysis output budget.

The Python research prototype should compare new strategies against that
behavioral baseline, but it should not attempt to reproduce production storage
or validation details.

## Inputs

The prototype reads local transcript files from:

```text
research/youtube_pipeline/inputs/
```

Initial input format is plain text. Timestamped transcript text is preferred
when available:

```text
[00:01:12] Speaker: Transcript text...
[00:03:40] Transcript text...
```

The first experiment set should include at least three videos:

- short: 10-20 minutes;
- long: 60-90 minutes;
- very long: 120+ minutes.

Use mixed content types when possible:

- interview or discussion;
- lecture, tutorial, or presentation;
- news, analysis, or opinion content.

## Output Contract

Every strategy writes a normalized result shape:

```json
{
  "summary_text": "",
  "timeline": [],
  "claims": [],
  "evidence": [],
  "action_items": [],
  "open_questions": []
}
```

The prototype does not need strict production schemas. It only needs enough
consistency to compare strategies.

Suggested item shapes:

```json
{
  "timeline": [
    {
      "start": "00:00:00",
      "end": "00:05:00",
      "title": "",
      "summary": ""
    }
  ],
  "claims": [
    {
      "text": "",
      "importance": "high",
      "evidence_refs": []
    }
  ],
  "evidence": [
    {
      "text": "",
      "timestamp": "00:00:00",
      "supports_claims": []
    }
  ],
  "action_items": [
    {
      "text": "",
      "target_audience": "",
      "priority": "medium"
    }
  ],
  "open_questions": [
    {
      "text": "",
      "why_it_matters": ""
    }
  ]
}
```

## Research Strategies

### Strategy 1: `one_shot_full_json`

One request:

```text
transcript -> summary + timeline + claims + evidence + action_items + open_questions
```

Purpose:

- establish the simplest baseline;
- measure whether a strong prompt can fill all fields in one pass;
- reveal whether JSON structure suppresses summary length.

Expected trade-off:

- cheapest and simplest;
- likely unstable for long videos;
- fields may be sparse, especially evidence and timeline.

### Strategy 2: `one_shot_markdown_plus_json`

One request:

```text
transcript -> detailed Markdown report + compact structured appendix
```

Purpose:

- test whether putting the readable report first improves summary quality;
- keep enough structured data to compare claims, evidence, action items, and
  timeline.

Expected trade-off:

- may produce better narrative text than strict JSON;
- structured appendix may still be incomplete;
- parsing can be loose in the research prototype.

### Strategy 3: `two_pass_summary_structure`

Two requests:

```text
1. transcript -> detailed summary + timeline
2. transcript + summary + timeline -> claims + evidence + action_items + open_questions
```

Purpose:

- separate narrative generation from structured extraction;
- test the likely best ROI candidate for production;
- keep request count low while improving field coverage.

Expected trade-off:

- more expensive than one-shot;
- much simpler than full MapReduce;
- likely default candidate for normal-length videos.

### Strategy 4: `chunk_map_reduce`

Multiple requests:

```text
1..N. transcript chunks -> chunk summary + timeline fragment + claims + evidence + action items
N+1. chunk outputs -> final summary + merged timeline + deduplicated claims/evidence/action items
```

Purpose:

- test long-video coverage;
- verify whether chunking preserves middle and end details better than one-shot;
- estimate cost growth as transcript length increases.

Chunking variants to test:

- timestamp windows, such as 10-20 minutes;
- token windows, such as 8k-12k input tokens;
- overlap windows if important details are lost near chunk boundaries.

Expected trade-off:

- best candidate for 60+ minute videos;
- more requests and more merge complexity;
- final result quality depends heavily on deduplication prompt quality.

### Removed Timeline-First Candidate

An earlier candidate explored model-created semantic segments before reduction.
It was removed from the active runner because it remained a stub alias for
`two_pass_summary_structure` and did not provide a distinct implementation.
Future timeline-first experiments should be introduced under a new design only
when they include real segment extraction, segment-detail prompts, and
deduplication behavior.

## Metrics

Each run writes:

```text
research/youtube_pipeline/runs/<timestamp>/<strategy>/<video_id>/
  result.json
  result.md
  metrics.json
  raw_requests.jsonl
  raw_responses.jsonl
```

`metrics.json`:

```json
{
  "strategy": "two_pass_summary_structure",
  "video_id": "video_long",
  "request_count": 2,
  "input_tokens": 0,
  "output_tokens": 0,
  "latency_seconds": 0,
  "summary_words": 0,
  "timeline_segments_count": 0,
  "claims_count": 0,
  "evidence_count": 0,
  "action_items_count": 0,
  "open_questions_count": 0,
  "json_valid": true,
  "notes": ""
}
```

Manual scoring should be recorded after reviewing the outputs:

```json
{
  "coverage_score": 1,
  "summary_quality_score": 1,
  "structure_quality_score": 1,
  "evidence_quality_score": 1,
  "hallucination_risk_score": 1,
  "review_notes": ""
}
```

Use a 1-5 scale where 5 is best, except `hallucination_risk_score`, where 5
means highest risk.

## Success Criteria

For long and very long videos, a promising strategy should usually produce:

- 800-1500+ words of useful summary when the source has enough substance;
- 6-15 timeline segments;
- 5-12 major claims;
- at least one evidence item for most important claims;
- action items when the video contains recommendations or instructions;
- explicit empty action items when the video does not contain actionable advice;
- coverage of the beginning, middle, and end of the transcript;
- no obvious claims unsupported by the transcript.

The research winner does not need to be the most complete strategy. It should
have the best quality-to-cost ratio and a clear rule for when to switch to a
more expensive strategy.

## Initial Experiment Matrix

Run four strategies against three transcript sizes:

```text
short       x one_shot_full_json
short       x one_shot_markdown_plus_json
short       x two_pass_summary_structure
short       x chunk_map_reduce

long        x one_shot_full_json
long        x one_shot_markdown_plus_json
long        x two_pass_summary_structure
long        x chunk_map_reduce

very_long   x one_shot_full_json
very_long   x one_shot_markdown_plus_json
very_long   x two_pass_summary_structure
very_long   x chunk_map_reduce
```

This gives 12 runs. If cost needs to be lower, start with:

```text
long x one_shot_full_json
long x two_pass_summary_structure
long x chunk_map_reduce
```

Then add the other combinations only if the early results are inconclusive.

## Recommended Decision Rule To Test

The likely production rule is:

```text
if transcript_tokens <= threshold:
    use two_pass_summary_structure
else:
    use chunk_map_reduce
```

The research should estimate the threshold. Candidate thresholds:

- 20k transcript tokens;
- 30k transcript tokens;
- 40k transcript tokens.

Timeline-first reduction was removed from the active runner because the
prototype never implemented it as a distinct strategy.

## Non-Goals

- No database reads or writes.
- No Tauri commands.
- No UI.
- No production prompt-pack schema validation.
- No canonical result builder.
- No projection tables.
- No automatic integration into the existing Rust pipeline.

## Implementation Notes For Later

The later Python prototype can stay intentionally small:

- `runner.py` for CLI entry point;
- `strategies.py` for strategy implementations;
- `llm_client.py` for provider calls;
- `chunking.py` for token or timestamp chunking;
- `metrics.py` for counters and output statistics;
- `prompts/` for prompt text files.

Provider credentials should come from environment variables. The prototype
should never store API keys in run artifacts.

## Open Questions

- Which LLM provider and model should be the default for the research run?
- Should the first prototype support both OpenAI-compatible APIs and the app's
  existing configured provider, or only one provider?
- What transcript token threshold should trigger chunked strategies in the
  first pass?
- Should manual scoring be done in plain JSON files or in a generated Markdown
  comparison table?
