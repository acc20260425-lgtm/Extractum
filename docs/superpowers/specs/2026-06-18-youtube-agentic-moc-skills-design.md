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
  -> MoC planning skill
  -> map extraction skill
  -> Python dedupe/alignment tools
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

The full variant should use multiple focused skills. The top-level skill
routes to the others. Each child skill is useful independently and can be
loaded only when needed.

### 1. `youtube-long-report`

Top-level orchestrator skill.

Responsibilities:

- create or validate the agent workspace;
- read the input transcript path and target output directory;
- call transcript prep tools;
- trigger MoC planning;
- trigger fact extraction;
- trigger dedupe/alignment;
- dispatch section-writing sub-agents;
- run QA checks;
- call final assembly;
- report final artifact paths and metrics.

This skill should not contain detailed prompt contracts for every stage.
It links to the child skills and keeps only the run lifecycle.

### 2. `youtube-transcript-prep`

Input normalization and chunking skill.

Responsibilities:

- call `normalize_transcript.py`;
- call `count_words.py`;
- call `estimate_tokens.py`;
- call `chunk_transcript.py`;
- verify chunk coverage;
- record timestamp quality warnings;
- write prep artifacts.

The agent must not hand-roll chunking or word counting.

### 3. `youtube-moc-planning`

Global Map of Content planning skill.

Responsibilities:

- build planner input using Python tools;
- ask a long-context model for a MoC JSON plan;
- validate the MoC shape;
- normalize node budgets;
- fall back to deterministic time-window nodes when MoC JSON is invalid;
- record fallback and projection flags.

This skill should emphasize that the MoC call is planning, not long prose
generation.

### 4. `youtube-fact-map`

Chunk-level evidence extraction skill.

Responsibilities:

- dispatch or perform map extraction over chunks;
- require timestamped facts, claims, quotes, examples, entities, and open
  questions;
- retry or quarantine invalid JSON when the policy allows;
- write `mapped_facts.jsonl`.

This skill should bias toward over-extraction. Later stages can dedupe and
filter, but missing facts cannot be recovered reliably.

### 5. `youtube-fact-align`

Fact deduplication and MoC alignment skill.

Responsibilities:

- call `dedupe_facts.py`;
- call `align_facts.py`;
- inspect unaligned facts;
- preserve high-importance facts even when alignment is uncertain;
- write `deduplicated_facts.json` and `alignment.json`.

This skill should keep the agent away from ad hoc semantic merging unless the
deterministic tools mark a cluster as ambiguous.

### 6. `youtube-section-reduce`

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

### 7. `youtube-report-qa`

Coverage and final review skill.

Responsibilities:

- run deterministic `quality_check.py`;
- optionally dispatch review sub-agents;
- check missing MoC nodes, short sections, repeated prose, unsupported claims,
  and overused framing words;
- call `assemble_report.py`;
- verify final `report.md` and `metrics.json`.

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
    build_planner_context.py
    validate_moc.py
    dedupe_facts.py
    align_facts.py
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

## Skill Storage

Use project-local skills:

```text
.agents/skills/youtube-long-report/SKILL.md
.agents/skills/youtube-transcript-prep/SKILL.md
.agents/skills/youtube-moc-planning/SKILL.md
.agents/skills/youtube-fact-map/SKILL.md
.agents/skills/youtube-fact-align/SKILL.md
.agents/skills/youtube-section-reduce/SKILL.md
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
`.agents/skills/youtube-*`.

## Agent Workspace

Each run should use a workspace directory distinct from final run artifacts.

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
    moc.raw.json
    moc.json
  map/
    mapped_facts.raw.jsonl
    mapped_facts.jsonl
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
  report.md
  result.json
  metrics.json
  moc.json
  mapped_facts.jsonl
  deduplicated_facts.json
  alignment.json
  coverage.json
  sections/
```

## Sub-Agent Model

Sub-agents are first-class in this workflow. The orchestrator skill should
dispatch them when the environment supports sub-agents, and fall back to local
sequential execution when it does not.

### Required Sub-Agent Roles

1. Transcript Auditor
   - checks word count, timestamp coverage, chunk coverage, and token estimate.

2. MoC Planner Reviewer
   - checks node count, time coverage, target word budgets, and fallback flags.

3. Map Extraction Auditor
   - samples mapped facts for density, timestamp grounding, and JSON validity.

4. Alignment Auditor
   - checks dedupe aggressiveness, unaligned facts, and repeated fact clusters.

5. Section Writer Agents
   - write disjoint section files.
   - each agent owns one or more MoC nodes and must not edit other sections.

6. Section QA Agent
   - checks section length, repeated prose, missing facts, unsupported claims,
     and source framing overuse.

7. Final Report Auditor
   - reviews final `report.md`, `metrics.json`, and coverage artifacts.

### Dispatch Rules

- Use sub-agents only for disjoint work or independent review.
- Do not give two writing agents the same section file.
- Do not ask sub-agents to run destructive cleanup.
- Give each sub-agent exact file ownership and expected output.
- Review sub-agent outputs before final assembly.
- If sub-agents are unavailable, run the same stages sequentially in the main
  agent and record `subagents_used=false`.

## Section Writing Contract

Section writers read:

```text
planning/moc.json
alignment/alignment.json
prep/chunks.jsonl
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
- use assigned facts before general narrative;
- include timestamps where useful;
- avoid generic filler;
- do not repeat the phrase "this video summary" throughout the section;
- do not rewrite other sections;
- if under 80 percent of target words, expand the same file using missing
  assigned facts.

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
  "video_id": "...",
  "transcript_words": 41000,
  "target_report_words": 8500,
  "summary_words": 9000,
  "moc_node_count": 10,
  "section_count": 10,
  "mapped_fact_count": 420,
  "deduplicated_fact_count": 260,
  "aligned_fact_count": 248,
  "unaligned_fact_count": 12,
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
- MoC invalid JSON: deterministic fallback nodes.
- Map extraction invalid JSON: retry once when possible, otherwise quarantine
  the chunk and continue with a warning.
- Empty aligned facts for a node: write a coverage warning and let the section
  use the transcript slice directly.
- Short section after expansion: keep the section, record warning.
- Sub-agent unavailable: sequential fallback.
- Assembly failure: fail the run.

## Testing Strategy

Tests should cover both deterministic Python and skill contracts.

Python unit tests:

- word counting;
- token estimation;
- transcript normalization;
- chunk coverage;
- MoC validation and fallback;
- fact dedupe;
- fact alignment;
- section file discovery;
- report assembly;
- metrics generation.

Skill contract tests or fixtures:

- each skill references only scripts that exist;
- orchestrator workflow lists all required stages;
- section writer contract forbids editing unassigned sections;
- QA contract checks source framing overuse;
- sub-agent fallback path is documented.

Integration smoke test:

- use a tiny fixture transcript;
- run deterministic prep tools;
- use mocked LLM artifacts for MoC and facts;
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
MoC validation, assembly, and metrics.

### Phase 2: Skill Skeletons

Create all seven skills with concise `SKILL.md` files and clear script calls.
Add targeted `.gitignore` exceptions if skills are committed under `.agents`.

### Phase 3: Agentic Workspace

Implement workspace conventions and artifact contracts. Add a minimal
orchestrator flow that can be run manually by Codex.

### Phase 4: Sub-Agent Workflow

Add sub-agent dispatch instructions to the top-level skill and QA skill.
Define file ownership, review roles, and sequential fallback.

### Phase 5: End-To-End Research Run

Run the Tucker transcript through the workflow, collect metrics, and compare
with `adaptive_book_report`.

## Open Questions

1. Should the first implementation commit skills directly under `.agents/skills`
   with `.gitignore` exceptions, or keep source skills under a tracked docs or
   research directory and install them into `.agents/skills` locally?

2. Should `moc_agentic_writer` live as a normal `runner.py --strategy`, or as a
   separate skill-driven command that calls Python scripts outside the strategy
   registry?

3. Should section-writing sub-agents use the same LLM provider as the planner,
   or should the workflow support role-specific model selection from day one?

4. Should the first MVP generate facts through LLM calls, or start with MoC
   section writing from transcript slices only and add fact extraction in the
   next phase?

## Recommended Decisions

- Commit project-local skills under `.agents/skills/youtube-*` with targeted
  `.gitignore` exceptions.
- Keep Python tools under `research/youtube_pipeline/tools`.
- Make the first user-facing strategy name `moc_agentic_writer`.
- Require sub-agent support when available, but implement sequential fallback.
- Keep fact extraction in the first MVP, because it is the main guard against
  long fluent sections that miss concrete evidence.
