# YouTube Agentic MoC Skills Design

Date: 2026-06-18

Status: draft for user review.

## Goal

Create a full agentic research workflow for long YouTube transcript reports
where Codex skills are the primary user-facing workflow layer and Python tools
are the deterministic execution layer.

The workflow should produce long, file-backed reports without relying on one
large LLM response. The agent writes and revises section files on disk, while
Python scripts perform repeatable operations such as transcript normalization,
chunking, word counting, token estimation, artifact validation, report assembly,
and metrics.

The first target is long YouTube transcript summarization using MoC-guided
Map-Reduce:

```text
transcript
  -> Python prep tools
  -> Python fact and chunk-summary extraction tool
  -> Python planner-context builder from map artifacts
  -> MoC planning skill
  -> deterministic fact-to-MoC alignment by chunk id
  -> section writing skill with sub-agents
  -> QA skill
  -> Python assembly and metrics
```

## Non-Goals

- No production integration into the Tauri app in v1.
- No replacement of `adaptive_book_report`; it remains a baseline.
- No requirement that every stage be fully automated in one command in v1.
  The first version may be an agent-guided workflow.
- No hidden dependency on model-specific tools. The skills should work with
  Codex-style agents first, and remain portable enough for other CLI agents
  that can read files, run commands, and write artifacts.
- No final whole-report LLM rewrite. Final rewrite passes tend to compress
  section detail and undo the benefit of file-backed generation.
- No manual word counting, chunking, or filename convention enforcement by the
  LLM. Deterministic scripts own those mechanics.
- No guarantee that arbitrary-length transcripts fit directly into the agent
  context. V1 uses a map-first flow: `extract_facts.py` summarizes and extracts
  evidence per chunk, then `build_planner_context.py` builds a bounded planner
  context from those dense map artifacts instead of handing the raw transcript
  to the planner.
- No conversational-agent fact extraction in v1. Chunk-level fact extraction is
  an API-like Python tool with fixed prompts and JSONL output, supervised by
  the orchestrator skill.

## Core Design Decision

Skills are the control plane. Python is the deterministic toolbox.

```text
skills
  - know the workflow
  - call the right scripts
  - dispatch sub-agents
  - write and revise prose files
  - enforce qualitative rules

Python scripts
  - normalize inputs
  - count words and estimate tokens
  - chunk transcripts
  - validate JSON artifacts
  - dedupe and align facts
  - assemble report.md
  - compute metrics.json
```

This avoids the two main failure modes:

1. Pure API pipelines hit response-token limits and often compress final
   reports.
2. Pure agent workflows are flexible but can be inconsistent unless scripts
   enforce the mechanical contract.

## Relationship To Existing Research

This spec is a sibling to the Python-first MoC design:

- `docs/superpowers/specs/2026-06-18-youtube-moc-guided-map-reduce-design.md`

That draft describes a Python strategy named `moc_guided_map_reduce`.
This spec describes a skill-first agentic workflow that can reuse the same
MoC concepts, but does not make Python the only orchestrator.

Useful pieces from `adaptive_book_report`:

- adaptive target word budgeting;
- language-aware output token budgeting;
- no final full-report rewrite;
- section expansion guards;
- metrics and artifact discipline.

Useful pieces from the MoC draft:

- global Map of Content planning;
- timestamped facts;
- fact-to-MoC alignment;
- node-level section generation;
- quality and coverage checks.

## Skill Set

The v1 workflow should use a small set of skills only where agent reasoning
adds value. Mechanical stages remain Python scripts called by the top-level
orchestrator.

### 1. `youtube-long-report`

Top-level orchestrator skill.

Responsibilities:

- create or validate the agent workspace;
- read the input transcript path and target output directory;
- call transcript prep tools;
- call Python fact and chunk-summary extraction;
- call Python planner-context builder;
- trigger MoC planning from map artifacts;
- call deterministic fact-to-MoC alignment by chunk id;
- dispatch section-writing sub-agents;
- run QA checks;
- call final assembly;
- report final artifact paths and metrics.

This skill should not contain the full writing prompts for every stage. It
links to child skills and keeps the run lifecycle, artifact paths, failure
policy, and script invocation order.

### 2. `youtube-moc-planning`

Global Map of Content planning skill.

Responsibilities:

- build planner input using Python tools;
- ask a long-context model for a MoC JSON plan;
- require each MoC node to claim contiguous or thematically related
  `chunk_ids`;
- validate the MoC shape;
- normalize node budgets;
- fall back to deterministic time-window nodes when MoC JSON is invalid;
- record fallback and planner-context truncation flags.

This skill should emphasize that the MoC call is planning, not long prose
generation.

Draft `SKILL.md` contract:

```text
Use when a long YouTube transcript needs a global Map of Content plan before
section writing.

Inputs:
- `planning/planner_context.md`
- `map/chunk_summaries.jsonl`
- `map/map_manifest.json`
- `prep/transcript_stats.json`
- requested output language
- target report word count
- optional section count range

Required action:
1. Read the planner context, map manifest, and transcript stats.
2. Produce only JSON for `planning/moc.raw.json`.
3. Ask for a structure plan, not long prose.
4. Assign every planned node explicit `chunk_ids`.
5. Run `validate_moc.py`.
6. If validation fails, ask once for corrected JSON or use deterministic
   fallback nodes when correction is unavailable.

Expected JSON shape:
{
  "report_title": "...",
  "source_kind": "youtube_video_transcript",
  "report_thesis": "...",
  "target_words": 9000,
  "nodes": [
    {
      "node_id": "moc_001",
      "title": "...",
      "purpose": "...",
      "target_words": 800,
      "time_range": {"start_ms": 0, "end_ms": 600000},
      "chunk_ids": ["chunk_001", "chunk_002"],
      "key_questions": ["..."],
      "required_fact_types": ["claim", "example", "quote"]
    }
  ]
}

Rules:
- Keep node order coherent and mostly chronological unless the transcript has
  a clearly thematic structure.
- Keep total node target words within 5 percent of the requested target.
- Prefer 6-14 nodes for normal long reports unless the requested target implies
  a smaller or larger outline.
- Cover every important chunk in at least one node, and avoid assigning the
  same chunk to many nodes unless it genuinely bridges topics.
- Do not invent facts. Factual claims must come from the map artifacts and
  aligned facts, not from planner inference.
```

### 3. `youtube-section-reduce`

File-backed section writing skill.

Responsibilities:

- write one Markdown section per MoC node into `sections/`;
- use aligned facts and relevant transcript slices;
- keep each section near its target word budget;
- expand only the section file when short;
- avoid whole-report rewrites;
- preserve timestamps when they help verification.

This is the main place where file-backed generation removes output-token
pressure.

Draft `SKILL.md` contract:

```text
Use when MoC nodes have been planned and aligned facts are available.

Inputs:
- `planning/moc.json`
- `alignment/alignment.json`
- `prep/chunks.jsonl`
- `map/chunk_summaries.jsonl`
- `map/mapped_facts.jsonl`
- one node assignment JSON

Writable output:
- only the assigned Markdown section file under `sections/`

Read-only context:
- all planning, prep, map, and alignment artifacts
- other section files

Required action:
1. Read the assigned node and aligned fact IDs.
2. Write the assigned section as substantive Markdown prose.
3. Use assigned facts before general narrative.
4. Include timestamps when they help verification.
5. Expand the same section file if it is under 80 percent of target words.
6. Stop after writing the assigned file. Do not edit other sections.

Style rules:
- Mention at most once, if needed, that this is based on a YouTube transcript.
- Avoid repeating "author" or "speaker" as a sentence crutch.
- Avoid generic filler and unsupported claims.
- Preserve nuance, disagreement, examples, and caveats from the evidence.
```

### 4. `youtube-report-qa`

Coverage and final review skill.

Responsibilities:

- run deterministic `quality_check.py`;
- optionally dispatch a qualitative review sub-agent;
- check coherence, repeated prose, unsupported claims, missing high-importance
  facts, and overused framing words;
- call `assemble_report.py`;
- verify final `report.md` and `metrics.json`.

### Later Skill Candidates

The following stages can become separate skills after the workflow stabilizes,
but are Python-driven in v1:

- transcript prep;
- fact extraction;
- fact deduplication;
- fact-to-MoC alignment.

## Python Tooling Layout

Use the existing `research/youtube_pipeline` package. Add deterministic tools
under a `tools` package, with importable helper modules for testability.

Suggested layout:

```text
research/youtube_pipeline/
  moc_models.py
  moc_budget.py
  moc_transcript.py
  moc_artifacts.py
  moc_alignment.py
  tools/
    __init__.py
    normalize_transcript.py
    count_words.py
    estimate_tokens.py
    chunk_transcript.py
    prep_all.py
    build_planner_context.py
    validate_moc.py
    extract_facts.py
    dedupe_facts.py
    align_facts.py
    validate_section_files.py
    assemble_report.py
    quality_check.py
```

Each tool should support command-line execution and emit machine-readable JSON
or JSONL where possible.

Example:

```powershell
python -m research.youtube_pipeline.tools.chunk_transcript `
  --input workspace/transcript.normalized.jsonl `
  --output workspace/chunks.jsonl `
  --chunk-token-limit 3000 `
  --overlap-tokens 250
```

Scripts must fail with a non-zero exit code on invalid input and write clear
error messages. Skills should stop on script failure instead of improvising.

### Key Tool Contracts

`prep_all.py` is an orchestration convenience wrapper for transcript
normalization, word counting, token estimation, and chunking. The individual
tools remain available for tests and debugging.

`build_planner_context.py` creates `planning/planner_context.md`. It should:

- read `map/chunk_summaries.jsonl`, `map/mapped_facts.jsonl`, and
  `prep/transcript_stats.json`;
- include one compact entry per chunk with chunk id, time range, summary,
  high-importance facts, entities, and open questions;
- avoid raw transcript excerpts unless needed to disambiguate a short or
  low-confidence chunk summary;
- cap output at `planner_context_target_tokens`, defaulting to 40000 estimated
  tokens for the map-first v1 flow;
- write planner context metadata including source transcript tokens, map artifact
  tokens, planner context tokens, compression ratio, omitted low-priority
  facts, and whether truncation was needed.

`extract_facts.py` performs chunk-level LLM API calls outside the agent
conversation context. It writes:

```text
map/chunk_summaries.jsonl
map/mapped_facts.raw.jsonl
map/mapped_facts.jsonl
map/map_manifest.json
map/quarantine.jsonl
```

Each chunk result should include `chunk_id`, time range, chunk summary,
important claims, examples, quotes, entities, open questions, and extracted
facts tagged with their source `chunk_id`. The tool should support concurrent
API calls with `--concurrency` defaulting to 5, retry invalid JSON once when
configured, preserve stable output ordering by chunk index, and quarantine
failed chunks with structured warnings instead of letting the agent improvise
fact extraction.

`dedupe_facts.py` may merge repeated facts across chunks, but it must preserve
all contributing chunk ids as `source_chunk_ids` on each deduplicated fact.

`align_facts.py` is deterministic in v1. It joins facts to MoC nodes by
chunk membership: a node receives every fact whose `chunk_id` or
`source_chunk_ids` intersects the node's `chunk_ids`. The tool may still emit
`unaligned_facts.json` for facts from chunks not covered by any node, but it
should not use semantic similarity as the default alignment mechanism.

`validate_moc.py` should also own deterministic fallback planning. If the MoC
JSON cannot be corrected, it creates time-window nodes with:

```text
fallback_node_count = max(1, round(target_words / chapter_target_words))
```

The fallback divides the transcript into contiguous chunk ranges, or by video
duration when reliable timestamps are available.

`validate_section_files.py` checks file ownership after section writers finish.
Preferred execution uses isolated writer workspaces or branch-backed sub-agent
workspaces so each writer can only merge its assigned section. When writers
share the same workspace, validation must be agent-specific:

```powershell
python -m research.youtube_pipeline.tools.validate_section_files `
  --agent-id section_writer_moc_003 `
  --expected-file workspace/sections/003-introduction.md
```

The script fails only when that writer's tracked changes include generated
paths other than the expected section file. Automatic reverts should only be
allowed for generated workspace files and only when explicitly enabled.

## Skill Storage

Use project-local skills:

```text
.agents/skills/youtube-long-report/SKILL.md
.agents/skills/youtube-moc-planning/SKILL.md
.agents/skills/youtube-moc-planning/examples/moc_sample.json
.agents/skills/youtube-section-reduce/SKILL.md
.agents/skills/youtube-section-reduce/examples/section_assignment_sample.json
.agents/skills/youtube-section-reduce/examples/alignment_sample.json
.agents/skills/youtube-report-qa/SKILL.md
```

Reasoning:

- this repository already has project-local skills under `.agents/skills`;
- the workflow is project-specific and should travel with Extractum research
  tooling;
- global user skills under `C:\Users\Dima\.codex\skills` would make the repo
  harder to reproduce.

Implementation note: `.agents/` is currently ignored by `.gitignore`. If these
skills should be committed, the implementation plan must either:

- add a targeted `.gitignore` exception for the new skill paths; or
- place the skill source under a tracked directory and copy/install it into
  `.agents/skills` during local setup.

The recommended option is a targeted `.gitignore` exception for
`.agents/skills/youtube-*/`.

## Agent Workspace

Each run should use a workspace directory distinct from final run artifacts.
The orchestrator skill creates the workspace. Use:

```text
run_id = <video_id>_<YYYYMMDD_HHMMSS>
```

Suggested scratch workspace:

```text
research/youtube_pipeline/work/<run_id>/
  input/
    transcript.txt
  prep/
    transcript.normalized.jsonl
    transcript_stats.json
    token_estimate.json
    chunks.jsonl
    chunk_manifest.json
  planning/
    planner_context.md
    planner_context_metadata.json
    moc.raw.json
    moc.json
  map/
    chunk_summaries.jsonl
    mapped_facts.raw.jsonl
    mapped_facts.jsonl
    map_manifest.json
    quarantine.jsonl
  alignment/
    deduplicated_facts.json
    alignment.json
    unaligned_facts.json
  sections/
    001-introduction.md
    002-...
  review/
    coverage.json
    coverage.md
    reviewer_notes.md
  final/
    report.md
    result.json
    metrics.json
```

Durable final artifacts should still be copied to the existing run directory:

```text
research/youtube_pipeline/runs/manual/moc_agentic_writer/<video_id>/
  <run_id>/
    report.md
    result.json
    metrics.json
    moc.json
    chunk_summaries.jsonl
    mapped_facts.jsonl
    map_manifest.json
    deduplicated_facts.json
    alignment.json
    quarantine.jsonl
    coverage.json
    sections/
```

## Sub-Agent Model

Sub-agents are first-class in this workflow. The orchestrator skill should
dispatch them when the environment supports sub-agents, and fall back to local
sequential execution when it does not.

### Required V1 Sub-Agent Roles

1. Section Writer Agents
   - write disjoint section files.
   - each agent owns one or more MoC nodes and must not edit other sections.

2. Section QA Agent
   - performs qualitative review after all sections are written.
   - checks coherence, repeated prose, missing high-importance facts,
     unsupported claims, and source framing overuse.

### Optional Auditor Roles

Auditor sub-agents can be added when qualitative judgment is useful, but v1
should not use LLM agents for checks that Python can perform deterministically.

Examples:

- MoC Planner Reviewer for plan coherence and chapter balance;
- Map Extraction Auditor for fact density and grounding on a small sample;
- Alignment Auditor for suspicious clusters and important unaligned facts;
- Final Report Auditor for narrative coherence and repetitive phrasing.

Quantitative checks such as word count, timestamp coverage, chunk coverage,
token estimates, JSON validity, and section file ownership belong to Python
tools.

### Dispatch Rules

- Use sub-agents only for disjoint work or independent review.
- Do not give two writing agents the same section file.
- Do not ask sub-agents to run destructive cleanup.
- Give each sub-agent exact file ownership and expected output.
- Tell section writers that all inputs except their assigned section file are
  read-only.
- Prefer isolated writer workspaces or branch-backed sub-agent workspaces.
- In shared workspaces, run `validate_section_files.py` with `--agent-id` and
  `--expected-file` for each writer.
- Review sub-agent outputs before final assembly.
- If sub-agents are unavailable, run the same stages sequentially in the main
  agent and record `subagents_used=false`.

## Section Writing Contract

Section writers read:

```text
planning/moc.json
alignment/alignment.json
prep/chunks.jsonl
map/chunk_summaries.jsonl
map/mapped_facts.jsonl
```

Each writer receives a node assignment:

```json
{
  "node_id": "moc_003",
  "section_file": "sections/003-title.md",
  "target_words": 900,
  "aligned_fact_ids": ["fact_001", "fact_041"],
  "time_range": {"start_ms": 1200000, "end_ms": 1800000}
}
```

Writer rules:

- write only the assigned section file;
- treat all other workspace files as read-only;
- use assigned facts before general narrative;
- include timestamps where useful;
- avoid generic filler;
- do not repeat the phrase "this video summary" throughout the section;
- do not rewrite other sections;
- if under 80 percent of target words, expand the same file using missing
  assigned facts.

After a writer finishes, the orchestrator runs `validate_section_files.py` for
that writer and expected section file. If an isolated writer workspace is used,
the orchestrator merges only the assigned section file. If a shared workspace
is used and the writer changed unassigned generated files, the run records a
warning and fails the stage for manual review unless explicit generated-file
rollback is enabled.

## Report Assembly Contract

Python owns final assembly.

`assemble_report.py` reads:

```text
planning/moc.json
sections/*.md
review/coverage.json
```

It writes:

```text
final/report.md
final/result.json
final/metrics.json
```

The final report should include:

- title;
- one short source note saying this is a summary and analysis of a YouTube
  video transcript;
- table of contents;
- executive overview;
- MoC-guided narrative sections;
- structured analysis sections;
- final synthesis;
- optional coverage appendix for research runs.

No LLM should rewrite `final/report.md` after assembly.

## Metrics

The workflow should emit:

```json
{
  "strategy": "moc_agentic_writer",
  "entry_point": "agentic_skill",
  "video_id": "...",
  "run_id": "...",
  "transcript_words": 41000,
  "target_report_words": 8500,
  "summary_words": 9000,
  "planner_context_tokens_estimated": 36000,
  "planner_context_truncated": true,
  "chunk_summary_count": 42,
  "moc_node_count": 10,
  "fallback_node_count": null,
  "section_count": 10,
  "mapped_fact_count": 420,
  "deduplicated_fact_count": 260,
  "aligned_fact_count": 248,
  "unaligned_fact_count": 12,
  "quarantined_chunk_count": 1,
  "fact_extraction_concurrency": 5,
  "subagents_used": true,
  "section_expansion_count": 3,
  "coverage_warnings": 2,
  "json_valid": true
}
```

Metrics should make agentic runs comparable with `adaptive_book_report` and
`moc_guided_map_reduce`.

## Error Handling

- Missing transcript: fail fast.
- Transcript normalization failure: fail fast.
- No timestamps: continue, but record degraded timestamp quality.
- Fact extraction invalid JSON: retry once when possible, otherwise quarantine
  the chunk and continue with a warning.
- Fact extraction rate limits: reduce concurrency and retry with backoff when
  the provider signals a recoverable limit.
- Planner context exceeds configured budget: truncate low-priority facts first,
  preserve every chunk summary when possible, and record truncation metadata.
- MoC invalid JSON: deterministic fallback nodes.
- MoC deterministic fallback: create
  `max(1, round(target_words / chapter_target_words))` contiguous fallback
  nodes.
- Empty aligned facts for a node: write a coverage warning and let the section
  use the transcript slice directly.
- Short section after expansion: keep the section, record warning.
- Section writer modifies unassigned generated files: fail the stage and record
  an ownership warning.
- Sub-agent unavailable: sequential fallback.
- Assembly failure: fail the run.

## Testing Strategy

Tests should cover both deterministic Python and skill contracts.

Python unit tests:

- word counting;
- token estimation;
- transcript normalization;
- chunk coverage;
- concurrent fact extraction ordering and quarantine reporting;
- planner context construction from chunk summaries and facts;
- MoC validation and fallback;
- fact dedupe;
- deterministic fact alignment by chunk id;
- section file discovery;
- section file ownership validation in shared and isolated writer workspaces;
- report assembly;
- metrics generation.

Skill contract tests or fixtures:

- each v1 skill references only scripts that exist;
- orchestrator workflow lists all required stages;
- MoC and section-writing skills include example input/output contracts;
- section writer contract forbids editing unassigned sections;
- QA contract checks source framing overuse;
- sub-agent fallback path is documented.

Integration smoke test:

- use a tiny fixture transcript;
- run deterministic prep tools;
- use mocked `extract_facts.py` output with chunk summaries and facts;
- build planner context from mocked map artifacts;
- use mocked LLM artifacts for MoC;
- assemble a final report;
- verify `report.md`, `result.json`, and `metrics.json`.

Manual research run:

- Tucker transcript;
- target 7,000-10,000 words;
- compare against `adaptive_book_report`;
- inspect coverage warnings, section repetition, and source framing.

## Implementation Phases

### Phase 1: Deterministic Toolbox

Build importable Python helpers and CLI wrappers for prep, chunking, counting,
concurrent fact and chunk-summary extraction, planner context construction from
map artifacts, MoC validation, deterministic chunk-id alignment, section
ownership validation, assembly, and metrics.

### Phase 2: Skill Contracts

Create the four v1 skills with concise `SKILL.md` files, clear script calls,
draft prompt contracts, and example JSON fixtures. Add targeted `.gitignore`
exceptions if skills are committed under `.agents`.

### Phase 3: Agentic Workspace

Implement workspace conventions, `run_id` creation, artifact contracts, and a
minimal orchestrator flow that can be run manually by Codex.

### Phase 4: Sub-Agent Workflow

Add section-writer sub-agent dispatch instructions to the top-level skill and
QA skill. Define file ownership, `validate_section_files.py`, qualitative QA,
and sequential fallback.

### Phase 5: End-To-End Research Run

Run the Tucker transcript through the workflow, collect metrics, and compare
with `adaptive_book_report`.

## Resolved V1 Decisions

- Commit project-local skills under `.agents/skills/youtube-*/` with targeted
  `.gitignore` exceptions.
- Keep Python tools under `research/youtube_pipeline/tools`.
- Use a separate skill-driven entry point instead of registering
  `moc_agentic_writer` in the normal `runner.py --strategy` registry.
- Keep `strategy: "moc_agentic_writer"` in `metrics.json` for comparison with
  Python strategies, and add `entry_point: "agentic_skill"` to make the
  execution path explicit.
- Use the same LLM provider/model profile for planner and writers in v1.
  Role-specific model selection can be added later.
- Keep fact extraction in the first MVP, because it is the main guard against
  long fluent sections that miss concrete evidence.
- Make fact extraction a Python API-call tool, not a conversational agent step.
- Use the map-first sequence: prep, chunk summaries and fact extraction,
  planner context from map artifacts, MoC planning, deterministic alignment.
- Keep transcript prep and fact-to-MoC alignment Python-only in v1.
- Default `planner_context_target_tokens` to 40000 estimated tokens for the
  map-first planner context.
- Require sub-agent support when available, but implement sequential fallback.

## Open Questions

1. After manual research runs, should `planner_context_target_tokens` remain at
   40000 estimated tokens, or should it adapt to transcript length and model
   context?
