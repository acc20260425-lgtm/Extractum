# YouTube MoC-Guided Map-Reduce Design

Date: 2026-06-18

Status: draft for user review.

## Table of Contents

- [Goal](#goal)
- [Non-Goals](#non-goals)
- [Concept Summary](#concept-summary)
- [Relationship To Adaptive Book Report](#relationship-to-adaptive-book-report)
- [Target Length Policy](#target-length-policy)
- [Transcript Representation](#transcript-representation)
- [Global Map of Content](#global-map-of-content)
- [Planner Context Policy](#planner-context-policy)
- [Parallel Map Extraction](#parallel-map-extraction)
- [Fact Deduplication and Entity Resolution](#fact-deduplication-and-entity-resolution)
- [MoC Alignment](#moc-alignment)
- [Node Reduce Generation](#node-reduce-generation)
- [Node Expansion Guard](#node-expansion-guard)
- [Structured Outputs](#structured-outputs)
- [Overview and Final Synthesis](#overview-and-final-synthesis)
- [Python Markdown Assembly](#python-markdown-assembly)
- [Quality and Self-Correction Loop](#quality-and-self-correction-loop)
- [Research Artifacts](#research-artifacts)
- [Cost and Parallelism](#cost-and-parallelism)
- [Strategy Options and Runner Integration](#strategy-options-and-runner-integration)
- [Metrics](#metrics)
- [Edge Cases](#edge-cases)
- [Testing Strategy](#testing-strategy)
- [Expected Outcome](#expected-outcome)

## Goal

Add a new Python research strategy named `moc_guided_map_reduce` for long
YouTube transcripts. The strategy should keep the core concept from the
provided architecture note: a hybrid MoC-guided Map-Reduce pipeline.

The goal is to produce a long-form, timestamp-grounded analytical report by
separating global planning from local detail generation:

- a long-context planning pass builds a global Map of Content, or `MoC`;
- chunk-level map passes extract dense facts, claims, quotes, and evidence
  with timestamps;
- Python aligns extracted facts to MoC nodes;
- local reduce calls generate detailed report sections from aligned facts and
  the relevant raw transcript slice;
- Python assembles the final Markdown report without a final rewrite pass.

The immediate research target is the Tucker Carlson transcript
`f_lRdkH_QoY`, roughly 41,000 words. For dense long transcripts in this range,
the strategy should make a 7,000-10,000 word report more reliable than the
current `antigravity_chunk_map_reduce` result.

## Non-Goals

- No production integration with the Rust/Tauri YouTube pipeline.
- No database reads or writes.
- No UI changes.
- No replacement or removal of `adaptive_book_report` or
  `antigravity_chunk_map_reduce`; they remain baselines.
- No requirement to use multiple real LLM providers in the first
  implementation. The architecture has "planner" and "mapper/reducer" roles,
  but both may be served by the same OpenAI-compatible endpoint during local
  research.
- No large embedding/vector database dependency in v1. Fact alignment may use
  deterministic time overlap and lightweight lexical scoring first.
- No final LLM rewrite of the whole report. Full-report rewrites are likely to
  compress the detailed sections and undo the strategy's main benefit.
- No guarantee that every 3-hour transcript must produce exactly 10,000 words.
  The target should depend on transcript length and density, with explicit CLI
  overrides for research runs.
- No high-quality MoC guarantee from a lossy projection. V1 is optimized for
  the current research target where the full transcript should fit the planner
  context. Projection is a fallback that must be recorded, not a quality
  equivalent path.
- No claim that source-document accuracy numbers such as "98.7% recall" are
  established for this prototype. Those are hypotheses to measure, not facts
  to repeat as achieved metrics.

## Concept Summary

The strategy is a hybrid of long-context planning and Map-Reduce detail
generation:

```text
timestamped transcript
  -> normalized transcript segments
  -> long-context MoC planning pass
  -> overlapping chunk map extraction
  -> fact deduplication and entity resolution
  -> align facts to MoC nodes
  -> local node reduce section generation
  -> per-node expansion when too short
  -> structured field extraction from aligned facts
  -> optional judge/coverage checks
  -> Python Markdown assembly
```

The MoC is the central difference from ordinary Map-Reduce. Standard chunk
Map-Reduce tends to inherit chunk boundaries and can produce a dry,
fragmented, or duplicated report. MoC-guided Map-Reduce first asks a
long-context model to identify the source's global structure: major topics,
time ranges, recurring themes, key terms, and section word budgets. Local
reducers then write against that global map rather than against arbitrary
chunk order.

## Relationship To Adaptive Book Report

The existing `adaptive_book_report` design is useful but conceptually
different:

- `adaptive_book_report` partitions chunks into chapters with Python budget
  logic and dynamic programming.
- `moc_guided_map_reduce` lets the MoC planning pass define the major report
  nodes and their time ranges.

Useful ideas to reuse:

- adaptive target length policy;
- per-section word budgets;
- expansion guard when generated prose is too short;
- Python assembly instead of final rewrite;
- `StrategyOptions` and `extra_metrics`;
- language-aware response token budgeting.

Ideas to avoid copying directly:

- DP chapter partitioning as the primary structure. In this design, MoC nodes
  are the primary structure.
- chunk notes as the only source for section generation. In this design, each
  node reducer receives aligned facts plus the relevant transcript slice.

## Target Length Policy

The strategy should compute a report budget from transcript word count. V1
does not apply a MoC density multiplier to the total report budget. MoC density
is used only to distribute words across nodes after the global target is
computed. This keeps v1 comparable with `adaptive_book_report` and avoids
letting a noisy MoC planning pass change the total target too aggressively.

Base table:

| Transcript words | Base report words | Expected MoC nodes |
| --- | ---: | ---: |
| `< 5,000` | 1,000-1,800 | 1-2 |
| `5,000-15,000` | 2,000-3,500 | 2-4 |
| `15,000-35,000` | 4,000-6,500 | 4-7 |
| `35,000-70,000` | 7,000-10,000 | 8-12 |
| `>= 70,000` | 10,000-14,000 | 12-18 |

Depth modes:

| Mode | Multiplier | Intended use |
| --- | ---: | --- |
| `brief` | `0.5x` | cheap comparison runs |
| `standard` | `1.0x` | default research depth |
| `deep` | `1.5x` | more complete long-video review |
| `book` | `2.0x` | exploratory maximum-depth run |

Hard caps for v1:

```text
max_report_words = 20,000
max_moc_nodes = 20
min_node_words = 500
```

Budget calculation:

```text
base_min_words, base_max_words = lookup transcript word count range
scaled_min_words = base_min_words * depth_multiplier
scaled_max_words = base_max_words * depth_multiplier
report_min_words = explicit min override or scaled_min_words
report_max_words = explicit max override or scaled_max_words
clamp both to hard caps
target_report_words = round((report_min_words + report_max_words) / 2)
```

If explicit overrides produce `report_min_words > report_max_words`, the
runner should fail fast with a clear error.

The MoC prompt receives the target range and asks for node-level
`target_word_count` values whose sum lands near `target_report_words`. Python
must validate and normalize those node budgets:

- clamp each node to at least `min_node_words` when possible;
- clamp total node budget to `report_min_words..report_max_words`;
- if the model omits budgets, distribute target words by node time span and
  node importance.

If the model provides node budgets, Python should normalize them while
preserving relative proportions. If it does not, distribute words with this
simple weight:

```text
node_weight = time_span_words * importance_multiplier
importance_multiplier = high: 1.3, medium: 1.0, low: 0.75
```

Then scale all node budgets so their sum is close to `target_report_words`.

## Transcript Representation

The source architecture note emphasizes timestamp fidelity. The prototype
should normalize transcript input into a simple timestamped transcript format
before LLM calls.

Suggested internal shape:

```json
{
  "segment_id": "seg_000123",
  "start_ms": 75200,
  "end_ms": 82100,
  "speaker": null,
  "text": "First, we need to partition our dataset..."
}
```

The first implementation can parse existing timestamped text files with
best-effort timestamps. If no timestamp exists for a line, keep `start_ms` and
`end_ms` as `null` and record a warning in `extra_metrics`.

Speaker diarization is optional. If speaker labels exist, preserve them. If
not, the pipeline must work as a monologue pipeline.

The implementation should add an explicit segment parser rather than relying
on plain-text chunking alone. Accepted v1 timestamp formats:

```text
[HH:MM:SS] text
[MM:SS] text
HH:MM:SS text
MM:SS text
```

VTT files are not a required direct input for this strategy, but the parser
should work with the timestamped `.txt` files produced by the current research
workflow. If timestamps are missing, the strategy may still run, but time
slicing, timestamp quality checks, and time-based MoC alignment must emit
warnings.

## Global Map of Content

The MoC pass is a planning call, not a writing call. Planner input selection
follows the [Planner Context Policy](#planner-context-policy).

MoC goals:

- identify major logical sections;
- assign time ranges;
- capture recurring themes and key terms;
- assign target word budgets;
- specify what each section must cover;
- create a stable structure for local reducers.

Expected JSON shape:

```json
{
  "video_id": "f_lRdkH_QoY",
  "report_thesis": "One-sentence throughline for the whole report.",
  "global_key_terms": ["term or entity"],
  "nodes": [
    {
      "node_id": "node_001",
      "title": "Descriptive section title",
      "time_span": {
        "start_ms": 0,
        "end_ms": 900000
      },
      "importance": "high",
      "target_word_count": 900,
      "description_outline": "What this section must explain.",
      "essential_key_terms": ["term"],
      "required_questions": [
        "Question this section should answer."
      ],
      "expected_fact_types": [
        "claims",
        "examples",
        "quotes",
        "counterarguments"
      ]
    }
  ]
}
```

V1 uses a flat ordered list of MoC nodes. Hierarchical `parent_id` nodes are
intentionally out of scope for the first implementation. If later research
needs nested sections, hierarchy should be added together with deterministic
assembly rules for `##` and `###` headings.

If MoC JSON is invalid, v1 should fall back to deterministic time-window
nodes. This keeps the research run alive but must record
`moc_fallback_used=true` in `extra_metrics`.

Fallback node generation:

```text
node_count = clamp(round(target_report_words / 900), 1, max_moc_nodes)
split transcript duration or segment sequence into node_count contiguous ranges
title nodes as "Section 1", "Section 2", ...
distribute target_report_words evenly
```

## Planner Context Policy

The MoC planner should prefer the full normalized transcript. This keeps the
global structure grounded and avoids asking the planner to invent unseen
sections.

V1 policy:

```text
planner_context_token_limit = options.planner_context_token_limit or 120000
estimated_transcript_tokens = approximate_token_count(transcript)
if estimated_transcript_tokens <= planner_context_token_limit:
    use full transcript for MoC planning
else:
    use deterministic temporal projection and record moc_projection_used=true
```

The default `120000` limit is a conservative local research setting, not a
claim about every provider. For the Tucker transcript at roughly 41,000 words,
the expected path is the full-transcript planner input.

`approximate_token_count()` should be deterministic and conservative enough for
provider-agnostic local research. V1 can use a lightweight heuristic based on
word count plus punctuation/CJK/non-ASCII adjustment; it must not treat words
and tokens as identical in the planner context decision.

Projection format, when required:

```json
{
  "projection_kind": "temporal_skeleton",
  "source_segment_count": 2840,
  "source_word_count": 95000,
  "windows": [
    {
      "window_id": "window_001",
      "start_ms": 0,
      "end_ms": 300000,
      "word_count": 1850,
      "first_words": "first 80 words from this time window",
      "last_words": "last 80 words from this time window",
      "sampled_timestamped_lines": [
        "[00:01:12] representative line"
      ]
    }
  ]
}
```

Projection windows should be contiguous and timestamp-preserving. Use five
minute windows by default, or smaller windows if a five minute span exceeds
about 2,000 words. The projection must preserve the first and last text of
each window and a small deterministic sample of timestamped lines. It is only
used to create coarse MoC nodes; local Map extraction still processes the full
transcript chunks.

## Parallel Map Extraction

The map stage extracts facts from overlapping chunks. It should favor recall
over prose quality.

Chunking policy:

```text
chunk_token_limit = options.chunk_token_limit or 3000
chunk_overlap_tokens = 500-800
```

The existing plain-text chunk helper does not support overlap or segment
metadata. V1 should add a segment-aware chunker, for example
`chunk_segments_by_approx_tokens(segments, max_tokens, overlap_tokens)`, that
preserves segment ids and timestamp ranges.

Map extraction output:

```json
{
  "chunk_index": 3,
  "chunk_time_span": {
    "start_ms": 1200000,
    "end_ms": 1440000
  },
  "facts": [
    {
      "fact_id": "chunk_003_fact_001",
      "kind": "claim",
      "text": "Atomic factual or argumentative statement.",
      "importance": "high",
      "time_span": {
        "start_ms": 1234000,
        "end_ms": 1250000
      },
      "verbatim_quote": "Short source quote when available.",
      "speaker": null,
      "entities": ["entity"],
      "topic_tags": ["topic"],
      "moc_node_hint": null
    }
  ],
  "action_items": [],
  "open_questions": []
}
```

V1 enum values:

```text
kind =
  claim | evidence | quote | example | definition |
  warning | action_item | open_question

importance =
  high | medium | low
```

If the model returns an unknown `kind`, normalize it to `evidence`. If it
returns an unknown `importance`, normalize it to `medium` and record a warning
count in `extra_metrics`.

The prompt should explicitly ask the model not to summarize the chunk into a
single paragraph. It should extract all material that could support a detailed
later section:

- claims and counterclaims;
- concrete examples;
- named entities;
- numbers and dates;
- definitions;
- direct quotes;
- warnings, tensions, and unresolved questions;
- practical takeaways.

## Fact Deduplication and Entity Resolution

Long talks often return to the same theme repeatedly. The fact set must be
deduplicated before local reduce generation, but not so aggressively that it
loses repeated emphasis.

V1 deduplication can be lightweight:

1. Normalize fact text:
   - lowercase;
   - trim punctuation;
   - remove repeated whitespace;
   - keep named entities and numbers.
2. Group exact or near-exact normalized duplicates.
3. Merge facts that have high lexical overlap and overlapping or nearby time
   ranges.
4. Preserve all timestamps as `mentions`.

V1 deterministic merge rule:

```text
normalized_jaccard = |tokens_a ∩ tokens_b| / |tokens_a ∪ tokens_b|
time_near = intervals overlap OR nearest timestamp distance <= 60 seconds
merge if normalized strings are equal
merge if normalized_jaccard >= 0.60 AND time_near
```

Tokens shorter than three characters should be ignored for the Jaccard score
unless they are numeric. Named entities and numbers should be retained because
they are often the evidence-bearing part of a fact.

Merged fact shape:

```json
{
  "cluster_id": "cluster_00042",
  "canonical_text": "Canonical statement.",
  "kind": "claim",
  "importance": "high",
  "mentions": [
    {
      "fact_id": "chunk_003_fact_001",
      "time_span": {
        "start_ms": 1234000,
        "end_ms": 1250000
      },
      "verbatim_quote": "Short quote."
    }
  ],
  "entities": ["entity"],
  "topic_tags": ["topic"]
}
```

Future versions may add embeddings or HDBSCAN-style clustering, but this is
not required for v1.

## MoC Alignment

Alignment assigns each fact cluster to one or more MoC nodes.

Primary signal:

- temporal intersection between fact `mentions[].time_span` and node
  `time_span`.

Secondary signals:

- lexical overlap between fact tags/entities and node title/key terms;
- `moc_node_hint` from the mapper, if present;
- nearby timestamps when a fact falls just outside a node boundary.

Alignment score:

```text
score =
  0.65 * time_overlap_score
  + 0.25 * lexical_overlap_score
  + 0.10 * mapper_hint_score
```

V1 score definitions:

```text
time_overlap_score =
  max over fact mentions:
    if mention interval intersects node interval:
      intersection_ms / max(mention_duration_ms, 1)
    else if nearest distance to node boundary <= 120 seconds:
      0.5 * (1 - distance_ms / 120000)
    else:
      0

lexical_overlap_score =
  |fact_terms ∩ node_terms| / max(1, |fact_terms|)

mapper_hint_score =
  1.0 if fact.moc_node_hint == node.node_id else 0.0

assignment_threshold = 0.30
secondary_assignment_threshold = 0.45
```

`fact_terms` are the union of `entities`, `topic_tags`, and normalized
keywords from `canonical_text`. `node_terms` are the union of node title words,
`essential_key_terms`, and global key terms.

V1 should assign each fact to the highest-scoring node when the score is at
least `assignment_threshold`. It may assign a recurring fact to additional
nodes when those nodes score at least `secondary_assignment_threshold`. Reused
clusters must be marked so local reducers can avoid duplicating the same prose
across sections.

Aligned node shape:

```json
{
  "node": {
    "node_id": "node_001",
    "title": "Section title",
    "target_word_count": 900
  },
  "aligned_fact_clusters": [],
  "raw_transcript_slice": "[00:00:00] ...",
  "coverage_warnings": []
}
```

## Node Reduce Generation

Each MoC node gets a local reduce call. The reducer writes one detailed
Markdown section using only:

- node metadata;
- aligned fact clusters;
- the raw transcript slice for the node time span;
- a compact global context: report thesis, global key terms, and neighboring
  node titles.

It must not receive all chunks or all facts from the full transcript.

Raw transcript slice cap:

```text
max_slice_tokens = options.max_slice_tokens or 8000
```

If the full raw slice for a MoC node exceeds `max_slice_tokens`, build a
compressed evidence slice:

- include transcript windows around aligned fact mentions with `±30s` context;
- merge overlapping windows;
- include the first and last 300 words of the node time range;
- preserve timestamps on every included line;
- record `slice_truncated=true` for that node in artifacts and metrics.

This keeps quote verification available without letting a broad MoC node
overload the node reducer input.

Expected section output:

```markdown
## Section 4: Title From MoC

Detailed analytical prose with timestamps such as [00:42:15].
```

Prompt constraints:

- write Markdown prose only;
- hit the node `target_word_count`;
- cite timestamps for important claims;
- use raw transcript slice only to verify wording and quotes;
- avoid generic filler;
- avoid duplicating facts marked as already covered in previous nodes;
- preserve the section's role in the global MoC.

This stage is the heart of the design. Long output is achieved by generating
many bounded sections rather than one huge response.

## Node Expansion Guard

After each node section is generated, Python counts words.

Default expansion rule:

```text
expand if generated_words < 0.8 * node.target_word_count
```

Expansion input:

- current section draft;
- node metadata;
- target word count and current word count;
- aligned fact clusters;
- raw transcript slice;
- coverage hints for facts that appear unused or thinly covered.

The expansion prompt should ask for a fuller revised section, not an appendix.
It should integrate missing details into the existing flow and avoid generic
padding.

V1 cap:

```text
max_expansions_per_node = 1
```

If the section remains short after expansion, keep the best available section
and record the shortfall in `extra_metrics`.

## Structured Outputs

Structured outputs should come from the fact clusters and aligned nodes, not
from the final prose alone.

`result.timeline`:

- derived from MoC nodes and important fact mentions;
- sorted chronologically;
- include node title and concise summary.

`result.claims`:

- top claim clusters by importance;
- preserve evidence refs from fact mentions.

`result.evidence`:

- timestamped fact mentions and quotes;
- linked to claims.

`result.action_items` and `result.open_questions`:

- merge mapper outputs and aligned fact metadata;
- deduplicate with the same lightweight dedupe rules.

This keeps JSON generation smaller and more reliable than asking one final
model response to invent the full structure after writing the report.

V1 should not parse structured data back out of Markdown node sections. If a
future reducer returns sidecar JSON, that should be a separate contract and
test path.

## Overview and Final Synthesis

V1 should include executive overview and final synthesis sections. The
preferred path uses two small LLM calls:

- executive overview;
- final synthesis/conclusion.

These calls should receive:

- MoC thesis and key terms;
- node titles and one-line descriptions;
- short section summaries;
- structured result JSON.

They must not receive the full long report and must not rewrite generated
sections. Their purpose is framing, not compression.

If either call fails or returns empty text, Python should generate a
deterministic fallback from MoC title/thesis and section titles. This keeps the
assembled report shape stable.

## Python Markdown Assembly

The final Markdown report is assembled deterministically in Python.

Suggested shape:

```markdown
# <video_id> MoC-Guided Deep Digest

Generated via `moc_guided_map_reduce`.

## Table of Contents

## Executive Overview

# Part I: Detailed MoC-Guided Narrative

## Section 1: ...
## Section 2: ...

# Part II: Structured Analysis

## Timeline and Development of Ideas
## Major Claims and Evidence
## Actionable Takeaways
## Open Questions

## Conclusion and Synthesis
```

`result.summary_text` should contain the assembled Markdown report.
`result.timeline`, `result.claims`, `result.evidence`,
`result.action_items`, and `result.open_questions` should be populated from
structured reductions over facts and MoC nodes.

If fact alignment leaves unassigned clusters, the final Markdown should include
a short "Coverage Appendix: Unaligned Facts" section or an equivalent visible
coverage warning. Saving unaligned facts only in `alignment.json` is not enough
because the research report would hide possible coverage loss from the reader.

## Quality and Self-Correction Loop

V1 should include deterministic checks and leave full LLM-as-a-Judge as an
optional extension.

Deterministic checks:

- every generated section has a word count;
- every section has at least one timestamp when source timestamps exist;
- every high-importance fact cluster is assigned to at least one MoC node;
- every MoC node has at least one aligned fact or a coverage warning;
- final report word count lands near the target range;
- structured arrays are not empty for long transcripts unless the map stage
  found no facts.

Run deterministic checks at two levels:

- after each node generation or expansion, for node word count, timestamps,
  aligned-fact coverage, and slice truncation warnings;
- after final Python assembly, for total word count, structured field counts,
  unaligned facts, and report-level timestamp coverage.

Optional judge checks:

1. Node coverage judge:
   - input: node metadata, aligned facts, generated section;
   - output: missing facts, unsupported statements, duplicate coverage.

2. Final report judge:
   - input: MoC summary, structured facts, section summaries;
   - output: coverage score, hallucination risk, timestamp quality score.

Judge output shape:

```json
{
  "coverage_score": 0.86,
  "timestamp_quality_score": 0.92,
  "unsupported_claims": [],
  "missing_high_importance_facts": [],
  "repair_recommendations": []
}
```

V1 does not need automatic multi-round repair. If the judge is implemented,
it should record findings in `extra_metrics` and artifacts. Targeted repair
can be a later reliability improvement.

Judge granularity:

- node coverage judge runs after node expansion and before assembly;
- final report judge runs after assembly using MoC, structured facts, and
  compact section summaries, not the full transcript.

## Research Artifacts

This strategy needs explicit intermediate artifacts for debugging and manual
evaluation. The standard `raw_requests.jsonl`, `raw_responses.jsonl`,
`result.json`, `result.md`, and `metrics.json` are not enough to understand
where coverage was lost.

`write_run_artifacts()` should support strategy-provided extra artifacts.
For `moc_guided_map_reduce`, write:

```text
moc.json                  # parsed or fallback MoC plan
mapped_facts.jsonl        # one map extraction result per chunk
deduplicated_facts.json   # merged fact clusters
alignment.json            # node -> assigned fact cluster ids and warnings
node_sections.jsonl       # generated/expanded section text and node metrics
quality_checks.json       # deterministic checks and optional judge findings
```

Extend `StrategyOutcome` with:

```python
extra_artifacts: dict[str, object | str] = field(default_factory=dict)
```

`write_run_artifacts()` should serialize each entry by filename:

- strings are written as UTF-8 text;
- dicts/lists are written as pretty JSON with `ensure_ascii=False`;
- filenames must be relative names without path separators.

Existing strategies can leave `extra_artifacts` empty.

## Cost and Parallelism

For the Tucker transcript, approximate v1 call count:

| Stage | Approx calls |
| --- | ---: |
| MoC planning | 1 |
| Map extraction | 14-18 |
| Node reduce | 8-12 |
| Node expansion | 0-4 |
| Overview | 1 |
| Conclusion | 1 |
| Optional node judges | 8-12 |
| Optional final judge | 1 |

Expected total without judges: roughly `25-36` calls. Expected total with
judges: roughly `34-49` calls.

Map extraction calls are independent. Node reduce calls should also be
independent because repeated coverage is marked before generation rather than
derived from previous prose. Therefore the strategy should support bounded
parallelism:

```text
max_parallel_map_calls = options.max_parallel_map_calls or 4
max_parallel_node_calls = options.max_parallel_node_calls or 3
```

If the first implementation keeps the current synchronous client and runs
sequentially, it should still record `parallelism_enabled=false` and the
estimated parallelizable call counts in `extra_metrics`.

## Strategy Options and Runner Integration

Use the shared options direction from the adaptive spec.

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
```

The option name `chapter_target_words` can still be reused as the desired
section/node target size, or renamed later to `section_target_words` if the
runner grows beyond research mode.

V1 CLI should expose the options most useful for manual research comparison.
The remaining fields may stay internal defaults until live runs show they need
frequent adjustment.

Runner options:

```text
--video-id <id>
--target-depth auto|brief|standard|deep|book
--min-report-words <int>
--max-report-words <int>
--chapter-target-words <int>
--chunk-token-limit <int>
--chunk-overlap-tokens <int>
--planner-context-token-limit <int>
--max-slice-tokens <int>
```

Internal defaults in v1:

```text
max_parallel_map_calls = 4
max_parallel_node_calls = 3
```

If parallel execution is not implemented in the first pass, these fields
should still exist in `StrategyOptions` but should be recorded as disabled in
`extra_metrics`.

Register:

```text
moc_guided_map_reduce
```

Per-call token budgets should be language-aware:

| Output language | Multiplier |
| --- | ---: |
| `en` | `1.8` |
| `ru` | `2.8` |
| other or unknown | `3.0` |

```text
response_token_budget = ceil(target_words * language_multiplier * 1.15)
```

These multipliers are conservative heuristics for local research. The runner
should record the chosen multiplier and estimated response token budget in
artifacts so later runs can tune them per provider/model.

Call caps:

- MoC planning: `options.max_tokens`;
- map extraction: `options.max_tokens`;
- node section generation: `min(options.max_tokens, response_token_budget)`;
- node expansion: `min(options.max_tokens, response_token_budget)`;
- overview/conclusion: around `2,000` tokens each.

## Metrics

Keep existing metrics:

- request count;
- input tokens;
- output tokens;
- latency;
- summary word count;
- timeline segment count;
- claims count;
- evidence count;
- action item count;
- open question count;
- JSON validity.

Add `extra_metrics`:

```json
{
  "transcript_words": 41384,
  "report_min_words": 7000,
  "report_max_words": 10000,
  "target_report_words": 8500,
  "actual_report_words": 8120,
  "estimated_transcript_tokens": 68000,
  "moc_node_count": 10,
  "moc_fallback_used": false,
  "moc_projection_used": false,
  "map_chunk_count": 16,
  "extracted_fact_count": 420,
  "deduplicated_fact_count": 260,
  "aligned_fact_count": 248,
  "unaligned_fact_count": 12,
  "node_expansion_count": 3,
  "nodes_below_target_after_expansion": 1,
  "timestamped_section_count": 10,
  "slice_truncated_node_count": 2,
  "parallelism_enabled": false,
  "parallelizable_map_call_count": 16,
  "parallelizable_node_call_count": 10,
  "max_parallel_map_calls": 4,
  "max_parallel_node_calls": 3,
  "coverage_warnings": []
}
```

`write_run_artifacts()` should merge `extra_metrics` into `metrics.json`.

## Edge Cases

| Scenario | Behavior |
| --- | --- |
| Empty transcript | Fail fast before any LLM call. |
| Transcript has fewer than 1,000 words | Use `one_shot_full_json` or a single MoC node. |
| Transcript has no timestamps | Continue, but disable timestamp quality checks and record warning. |
| Full transcript exceeds planner context limit | Use deterministic temporal projection, record `moc_projection_used=true`, and keep full transcript for map extraction. |
| MoC JSON invalid | Use deterministic time-window fallback, record `moc_fallback_used=true`, and keep aggregate `json_valid=false`. |
| MoC node budgets missing | Distribute target words by time span and importance. |
| Map JSON invalid for a chunk | Retry once with the same prompt and a stricter "return valid JSON only" reminder; if still invalid, keep an empty fact set for that chunk and record warning. |
| Facts do not align to any node | Store them as unaligned facts and add an appendix or warning. |
| Node has no aligned facts | Generate from raw transcript slice and record low-confidence warning. |
| Node raw slice exceeds cap | Build fact-centered evidence windows and record `slice_truncated=true`. |
| Node section too short after expansion | Use best available text and record shortfall. |
| Repeated facts appear in several nodes | Allow repeated assignment but mark repeated coverage to prevent duplicate prose. |

## Testing Strategy

Use mocked LLM clients. Tests should cover:

- strategy registration under `moc_guided_map_reduce`;
- base report budget selection from transcript word count;
- explicit min/max override validation;
- MoC prompt asks for global map only, not long prose;
- planner context token estimate does not treat word count as token count;
- planner context policy uses full transcript under the limit and temporal
  projection over the limit;
- MoC JSON parsing and fallback when invalid;
- aggregate JSON validity remains false when MoC fallback recovered from
  invalid planner JSON;
- fallback MoC nodes preserve order and distribute target words;
- timestamped transcript parser handles `HH:MM:SS` and `MM:SS` formats;
- map extraction prompt requests atomic facts, quotes, timestamps, entities,
  action items, and open questions;
- overlapping chunk generation preserves text coverage;
- overlapping chunker preserves segment ids and timestamp ranges;
- fact deduplication merges repeated mentions while preserving timestamps;
- fact deduplication uses the configured Jaccard/time-near thresholds;
- alignment assigns facts by time overlap;
- alignment uses key terms as secondary signal;
- alignment score uses the defined component formulas and threshold;
- unaligned facts are recorded;
- node reducer receives only node metadata, aligned facts, raw slice, and
  compact global context;
- node reducer does not receive all facts from the full transcript;
- node raw slice is capped and replaced with fact-centered evidence windows
  when necessary;
- expansion call occurs when node text is shorter than `0.8 * target`;
- expansion call is skipped when node text is long enough;
- final report is assembled in Python, not rewritten by a final LLM call;
- structured outputs are populated from facts and MoC nodes;
- intermediate artifacts are written for MoC, facts, alignment, node sections,
  and quality checks;
- unaligned facts are visible in the final Markdown report as an appendix or
  warning, not only in `alignment.json`;
- `extra_metrics` are written to `metrics.json`;
- timestamp quality warnings are emitted when timestamps are missing;
- language-aware token budget uses the Russian multiplier for `ru`;
- bounded parallelism options are passed through or recorded as disabled;
- disabled parallelism still records parallelizable map/node call counts;
- runner passes `StrategyOptions` to all registered strategies.

Live LLM runs remain manual research validation, not unit tests.

## Expected Outcome

For short transcripts, the strategy should avoid inflated output. For long,
dense transcripts like the Tucker Carlson interview at roughly 41,000 words,
`standard` or `auto` should target a report in the 7,000-10,000 word range.

Compared with ordinary `chunk_map_reduce`, the expected improvement is better
global coherence because section boundaries come from the MoC rather than from
chunk boundaries.

Compared with `adaptive_book_report`, the expected improvement is stronger
timestamp and fact grounding because local sections are generated from aligned
fact clusters and the relevant raw transcript slice, not only from dense chunk
notes.

The core hypothesis to validate manually:

```text
MoC-guided Map-Reduce can preserve the global structure of a long video while
keeping local generation factual, timestamped, and long enough to satisfy a
deep-digest target.
```
