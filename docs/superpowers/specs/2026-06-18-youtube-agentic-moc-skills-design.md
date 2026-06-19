# YouTube Agentic MoC Skills Design

Date: 2026-06-18

Status: implemented and verified on `main`.

Implementation notes:

- `youtube-summary` is the public wrapper skill for normal user-facing runs.
- `youtube-long-report` remains as a lower-level/manual orchestrator contract
  for older research notes and child-skill documentation.
- The agentic workflow remains separate from `runner.py --strategy`.

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
  -> Python map-assignment builder
  -> Map Extractor Sub-Agents
  -> Python map-output validation and assembly
  -> Python planner-context builder from map artifacts
  -> MoC planning skill
  -> Python fact dedupe
  -> deterministic fact-to-MoC alignment by chunk id
  -> Python section-assignment builder
  -> section writing skill with sub-agents
  -> overview and synthesis sections
  -> QA skill
  -> Python structured-analysis builder
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
  context. V1 uses a map-first flow: map extractor sub-agents summarize and
  extract evidence per chunk, then `build_planner_context.py` builds a bounded
  planner context from those dense map artifacts instead of handing the raw
  transcript to the planner.
- No direct LLM API calls in v1. Every LLM reasoning step is performed by the
  main Codex-style agent or by sub-agents. Python tools prepare assignments,
  validate files, repair JSON, assemble artifacts, and compute metrics.

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

### 1. `youtube-summary`

Public wrapper skill.

Responsibilities:

- accept the transcript path, output language, target words, and optional run
  directory;
- create or resume a run through `start_youtube_summary.py`;
- read and update `workflow_state.json`;
- call deterministic Python gates in the documented order;
- advance state with `advance_youtube_summary_state.py`;
- dispatch `youtube-map-extract`, `youtube-moc-planning`,
  `youtube-section-reduce`, and `youtube-report-qa`;
- pause before map extraction when sub-agents are unavailable;
- return `final/report.md`, `final/metrics.json`, and any validation warnings.

This is the skill users should ask for directly.

### 2. `youtube-long-report`

Lower-level/manual orchestrator skill.

Responsibilities:

- create or validate the agent workspace;
- read the input transcript path and target output directory;
- call transcript prep tools;
- call Python map-assignment builder;
- dispatch map extractor sub-agents;
- call Python map-output validation and assembly;
- call Python planner-context builder;
- trigger MoC planning from map artifacts;
- call deterministic fact-to-MoC alignment by chunk id;
- dispatch section-writing sub-agents;
- write overview and synthesis boundary sections;
- run QA checks;
- call Python structured-analysis builder;
- call final assembly;
- report final artifact paths and metrics.

This skill should not contain the full writing prompts for every stage. It
links to child skills and keeps the run lifecycle, artifact paths, failure
policy, and script invocation order. In the current implementation it is
retained for manual research runs and legacy documentation; `youtube-summary`
is the preferred public wrapper because it owns resume state.

### 3. `youtube-map-extract`

Map extraction skill used by map extractor sub-agents.

Responsibilities:

- read assigned chunk work packets from `map/assignments/`;
- extract chunk summary, claims, examples, quotes, entities, open questions,
  and facts;
- write JSON only to assigned `map/agent_outputs/*.json` files;
- avoid outside knowledge and unsupported inference;
- keep local fact ids only.

This skill contains the prompt contract used by map extractor sub-agents. The
top-level orchestrator dispatches it with one or more assignment files.

### 4. `youtube-moc-planning`

Global Map of Content planning skill.

Responsibilities:

- build planner input using Python tools;
- use the current agent reasoning context to write a MoC JSON plan;
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
3. Write a structure plan, not long prose.
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

### 5. `youtube-section-reduce`

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
- `alignment/section_assignments.jsonl`
- `prep/chunks.jsonl`
- `map/chunk_summaries.jsonl`
- `map/mapped_facts.jsonl`
- one section assignment row

Writable output:
- only the assigned Markdown section file under `sections/`

Read-only context:
- all planning, prep, map, and alignment artifacts
- other section files

Required action:
1. Read the assigned section assignment row and aligned fact IDs.
2. Write the assigned section as substantive Markdown prose.
3. Use assigned facts before general narrative.
4. Include timestamps when they help verification.
5. Expand the same section file if it is under 80 percent of target words.
6. If the section is shorter than target but already covers all assigned facts
   without filler, mark the remaining word deficit for redistribution instead
   of padding prose.
7. Stop after writing the assigned file. Do not edit other sections.

Style rules:
- Mention at most once, if needed, that this is based on a YouTube transcript.
- Avoid repeating "author" or "speaker" as a sentence crutch.
- Avoid generic filler and unsupported claims.
- Preserve nuance, disagreement, examples, and caveats from the evidence.
```

### 6. `youtube-report-qa`

Coverage and final review skill.

Responsibilities:

- run deterministic `quality_check.py`;
- optionally dispatch a qualitative review sub-agent;
- check coherence, repeated prose, unsupported claims, missing high-importance
  facts, and overused framing words;
- check cross-section reuse of the same facts, so repeated evidence is framed
  in each section's local argument instead of duplicated as near-identical
  prose;
- write `review/coverage.json`, `review/coverage.md`, and
  `review/reviewer_notes.md`;
- leave final report assembly to the orchestrator after QA completes.

### Later Skill Candidates

The following stages can become separate skills after the workflow stabilizes,
but remain deterministic Python helper stages in v1:

- transcript prep;
- map assignment preparation;
- map output validation and assembly;
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
    prepare_map_assignments.py
    validate_map_outputs.py
    assemble_map_artifacts.py
    build_planner_context.py
    validate_moc.py
    dedupe_facts.py
    align_facts.py
    prepare_section_assignments.py
    validate_generated_files.py
    build_structured_analysis.py
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
- cap output at `planner_context_target_tokens`. The default is the smaller of
  40000 estimated tokens and 40 percent of the configured planner model context
  window, leaving room for skill text, conversation state, and MoC JSON output;
- write planner context metadata including source transcript tokens, map artifact
  tokens, planner context tokens, compression ratio, omitted low-priority
  facts, and whether truncation was needed.

`prepare_map_assignments.py` creates sub-agent work packets from
`prep/chunks.jsonl`. It writes:

```text
map/assignments/chunk_001.assignment.json
map/assignments/chunk_002.assignment.json
map/assignment_manifest.json
```

Each assignment includes the chunk id, time range, transcript text, output
language, expected output schema, target density, and the exact output file the
map extractor sub-agent may write. When one map extractor receives multiple
assignments, the tool also writes an expected-files manifest for that agent:

```text
map/expected_files/mapper_batch_001.json
```

Map extractor sub-agents read one or more assignment files and write only their
assigned outputs:

```text
map/agent_outputs/chunk_001.json
map/agent_outputs/chunk_002.json
```

Each chunk output should include `chunk_id`, time range, chunk summary,
important claims, examples, quotes, entities, open questions, and extracted
facts tagged with their source `chunk_id`. The orchestrator may dispatch many
map extractor sub-agents in parallel, but Python does not call an LLM API for
this stage.

`validate_map_outputs.py` validates and normalizes sub-agent JSON files. Before
marking an output invalid, it should attempt lightweight JSON repair for common
formatting defects such as trailing text, missing closing braces, or unescaped
newlines. Repair attempts must be recorded in `map/validation_manifest.json`
so the run remains auditable. The tool reports invalid outputs and repair
failures, but it does not dispatch sub-agents. The orchestrator decides whether
to request one corrected sub-agent output or list the chunk in
`map/quarantine.jsonl`.

`assemble_map_artifacts.py` combines valid sub-agent outputs into:

```text
map/chunk_summaries.jsonl
map/mapped_facts.raw.jsonl
map/mapped_facts.jsonl
map/map_manifest.json
map/assembly_manifest.json
map/quarantine.jsonl
```

It must preserve stable output ordering by chunk index and fail if required
chunks are missing unless the run policy allows quarantine. It also assigns
canonical global fact ids deterministically, so sub-agent outputs may use local
fact ids without risking cross-chunk collisions.

`map/map_manifest.json` is the public consolidated manifest assembled from the
assignment, validation, and assembly manifests. Individual tools should write
their own stage manifest first, then let `assemble_map_artifacts.py` produce the
consolidated manifest.

`dedupe_facts.py` may merge repeated facts across chunks, but it must preserve
all contributing chunk ids as `source_chunk_ids` and all original timestamps as
`source_timestamps` on each deduplicated fact. Section writers can then cite
the most relevant timestamp for their chapter while QA can still see every
place where the fact appeared.

`align_facts.py` is deterministic in v1. It joins facts to MoC nodes by
chunk membership: a node receives every fact whose `chunk_id` or
`source_chunk_ids` intersects the node's `chunk_ids`. The tool may still emit
`unaligned_facts.json` for facts from chunks not covered by any node, but it
should not use semantic similarity as the default alignment mechanism.

`prepare_section_assignments.py` creates `alignment/section_assignments.jsonl`
after alignment. Each row gives a section writer exactly one MoC node, target
word budget, aligned fact ids, chunk ids, time range, and output section file.
Section writer agents must receive assignments from this file rather than
hand-authored prompts.

`build_structured_analysis.py` builds deterministic structured analysis
sections from deduplicated facts, repeated entities, fact clusters, and coverage
artifacts. It should not call an LLM or rewrite narrative sections. It writes
`review/structured_analysis.md` for final assembly.

`validate_moc.py` should also own deterministic fallback planning. If the MoC
JSON cannot be corrected, it creates time-window nodes with:

```text
fallback_node_count = max(1, round(target_words / chapter_target_words))
```

The fallback divides the transcript into contiguous chunk ranges, or by video
duration when reliable timestamps are available.

MoC validation should perform deterministic coverage checks before section
writing starts:

- gap detection: warn when important `chunk_ids` are not assigned to any node;
- overlap limit: warn when a chunk is assigned to more than 2-3 nodes;
- chronological sanity: warn when node chunk ranges are strongly non-ascending
  without an explicit thematic reason;
- budget sanity: verify total target words and node target words stay within
  configured tolerance.

`validate_generated_files.py` checks file ownership after map extractor and
section writer sub-agents finish. Parallel sub-agent execution must use
isolated workspaces or branch-backed sub-agent workspaces so each agent can
only merge its assigned outputs. A shared workspace is allowed only for
sequential or debug runs, and validation must be agent-specific:

```powershell
python -m research.youtube_pipeline.tools.validate_generated_files `
  --agent-id section_writer_moc_003 `
  --expected-file workspace/sections/003-introduction.md
```

For agents that own multiple output files, use an expected-files manifest:

```powershell
python -m research.youtube_pipeline.tools.validate_generated_files `
  --agent-id mapper_batch_001 `
  --expected-files-manifest workspace/map/expected_files/mapper_batch_001.json
```

The script fails when that agent's tracked changes include generated paths
outside the expected file or expected-files manifest. Automatic reverts should
only be allowed for generated workspace files and only when explicitly enabled.

## Skill Storage

Use project-local skills:

```text
.agents/skills/youtube-summary/SKILL.md
.agents/skills/youtube-long-report/SKILL.md
.agents/skills/youtube-long-report/examples/map_assignment_sample.json
.agents/skills/youtube-long-report/examples/map_output_sample.json
.agents/skills/youtube-map-extract/SKILL.md
.agents/skills/youtube-map-extract/examples/map_assignment_sample.json
.agents/skills/youtube-map-extract/examples/map_output_sample.json
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
    assignments/
      chunk_001.assignment.json
      chunk_002.assignment.json
    expected_files/
      mapper_batch_001.json
    agent_outputs/
      chunk_001.json
      chunk_002.json
    assignment_manifest.json
    validation_manifest.json
    assembly_manifest.json
    chunk_summaries.jsonl
    mapped_facts.raw.jsonl
    mapped_facts.jsonl
    map_manifest.json
    quarantine.jsonl
  alignment/
    deduplicated_facts.json
    alignment.json
    section_assignments.jsonl
    unaligned_facts.json
  sections/
    000-overview.md
    001-introduction.md
    002-...
    999-synthesis.md
  review/
    coverage.json
    coverage.md
    reviewer_notes.md
    structured_analysis.md
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
    assignment_manifest.json
    validation_manifest.json
    assembly_manifest.json
    deduplicated_facts.json
    alignment.json
    section_assignments.jsonl
    quarantine.jsonl
    coverage.json
    structured_analysis.md
    sections/
```

### Resume Contract

The orchestrator skill should be resumable from an existing workspace. Before
running an expensive or long stage, it checks whether the expected artifact
already exists and validates it with the matching Python tool.

Examples:

- valid `prep/chunks.jsonl` skips transcript prep;
- valid `map/mapped_facts.jsonl`, `map/chunk_summaries.jsonl`, and
  `map/map_manifest.json` with matching stage manifests skip map sub-agent
  extraction and map assembly;
- valid `planning/moc.json` skips MoC planning;
- valid section files that pass ownership and word-count checks skip section
  writing for those nodes.

Each stage should write a small completion marker or manifest entry with tool
version, input hashes, options, and validation status. A stage may be reused
only when its input hashes and relevant options match the current run.

Resume hash scope:

| Stage | Reuse only when these inputs match |
|---|---|
| transcript prep | raw transcript hash, normalization options |
| chunking | normalized transcript hash, chunk size, overlap, tokenizer estimate mode |
| map assignments | chunks hash, output language, map schema version |
| map outputs | assignment hash, agent model/profile, map schema version, prompt contract version |
| map artifact assembly | validated map output hashes, schema version |
| planner context | chunk summaries hash, mapped facts hash, target token cap, planner context policy version |
| MoC planning | planner context hash, target words, output language, planner agent model/profile, MoC prompt contract version |
| dedupe | mapped facts hash, dedupe policy version |
| alignment | MoC hash, deduplicated facts hash, alignment policy version |
| section assignments | MoC hash, alignment hash, budget policy version |
| section writing | section assignment hash, aligned fact ids hash, writer agent model/profile, section prompt contract version |
| boundary sections | MoC hash, section file hashes, map manifest hash, boundary prompt contract version |
| QA | section file hashes, boundary section hashes, QA policy version, QA agent model/profile when used |
| structured analysis | deduplicated facts hash, alignment hash, coverage hash, structured analysis policy version |
| assembly | MoC hash, section file hashes, coverage hash, structured analysis hash, report assembly version |

## Sub-Agent Model

Sub-agents are first-class in this workflow. The orchestrator skill should
dispatch them when the environment supports sub-agents. V1 does not use direct
LLM API calls as a fallback for LLM reasoning stages.

### Required V1 Sub-Agent Roles

1. Map Extractor Agents
   - read assigned chunk work packets from `map/assignments/`;
   - write one JSON output per assigned chunk into `map/agent_outputs/`;
   - each agent owns only its assigned output files.

2. Section Writer Agents
   - write disjoint section files.
   - each agent owns one or more MoC nodes and must not edit other sections.

3. Section QA Agent
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
- Do not give two map extractor agents the same output file.
- Do not give two writing agents the same section file.
- Do not ask sub-agents to run destructive cleanup.
- Give each sub-agent exact file ownership and expected output.
- Tell map extractor agents that all files except their assigned
  `map/agent_outputs/*.json` files are read-only.
- Tell section writers that all inputs except their assigned section file are
  read-only.
- Use isolated sub-agent workspaces or branch-backed sub-agent workspaces for
  parallel map extraction and section writing.
- Shared workspaces are only allowed for sequential/debug runs; in that mode,
  run ownership validation with `--agent-id` plus either `--expected-file` or
  `--expected-files-manifest` for each map extractor and section writer.
- Review sub-agent outputs before final assembly.
- If sub-agents are unavailable, fail or pause before map extraction. The
  workflow must not replace map extraction with direct Python LLM API calls.
- If sub-agents are unavailable after map extraction, section writing may fall
  back to sequential main-agent execution using the same
  `youtube-section-reduce` contract. Record `section_writer_subagent_count=0`
  and keep generated-file ownership validation enabled.

## Map Extraction Contract

Map extractor sub-agents read assignment files:

```text
map/assignments/chunk_001.assignment.json
map/assignments/chunk_002.assignment.json
```

Each assignment contains:

```json
{
  "chunk_id": "chunk_001",
  "output_file": "map/agent_outputs/chunk_001.json",
  "time_range": {"start_ms": 0, "end_ms": 600000},
  "output_language": "ru",
  "transcript_text": "...",
  "target_summary_words": 250,
  "max_fact_count": 20
}
```

Each map extractor writes only the assigned output file. Expected output:

```json
{
  "chunk_id": "chunk_001",
  "time_range": {"start_ms": 0, "end_ms": 600000},
  "chunk_summary": "...",
  "claims": [{"text": "...", "timestamp": "00:10:00", "importance": 4}],
  "examples": [{"text": "...", "timestamp": "00:12:30"}],
  "quotes": [{"text": "...", "timestamp": "00:13:10"}],
  "entities": ["..."],
  "open_questions": ["..."],
  "facts": [
    {
      "local_fact_id": "fact_001",
      "text": "...",
      "fact_type": "claim",
      "timestamp": "00:10:00",
      "importance": 4,
      "chunk_id": "chunk_001"
    }
  ]
}
```

Map extractor rules:

- extract evidence from the assigned transcript text only;
- do not infer facts from outside knowledge;
- prefer over-extraction to missed evidence;
- keep timestamps when present in the source;
- use only local fact ids; `assemble_map_artifacts.py` assigns canonical
  global fact ids;
- write JSON only, with no Markdown wrapper or commentary;
- do not edit assignment files, other chunks, or assembled map artifacts.

## Section Writing Contract

Section writers read:

```text
planning/moc.json
alignment/alignment.json
alignment/section_assignments.jsonl
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
  assigned facts;
- if no assigned facts are missing and expansion would add filler, keep the
  shorter section and emit a word-deficit note for orchestrator redistribution.

After a writer finishes, the orchestrator runs `validate_generated_files.py` for
that writer and expected section file. If an isolated sub-agent workspace is used,
the orchestrator merges only the assigned section file. If a shared workspace
is used and the writer changed unassigned generated files, the run records a
warning and fails the stage for manual review unless explicit generated-file
rollback is enabled.

## Budget Redistribution Contract

Section target words are guidance, not permission to add filler. If a section
is complete, evidence-backed, and still below target, the orchestrator records
the deficit instead of forcing expansion. Remaining word budget can be
redistributed to later unwritten nodes with higher evidence density.

Redistribution only works when section writing is dispatched in ordered waves.
The orchestrator should write sections in small batches, validate completed
sections, then update targets for later unassigned section assignments before
dispatching the next batch. If the run dispatches all section writers at once,
redistribution is disabled and short complete sections are recorded as budget
underspend instead of being padded.

Redistribution inputs:

```text
planning/moc.json
alignment/alignment.json
map/chunk_summaries.jsonl
sections/*.md
review/section_word_counts.json
```

Redistribution rules:

- never reduce a section below the words it already contains;
- prefer nodes with more high-importance facts, dense chunk summaries, or
  unresolved questions;
- avoid increasing a node target by more than 30 percent in one pass unless the
  user explicitly asks for a longer report;
- record `redistributed_word_count` and per-node budget changes in metrics.

## Overview And Synthesis Contract

The orchestrator skill writes two boundary sections after node sections are
available and before the QA pass:

```text
sections/000-overview.md
sections/999-synthesis.md
```

Inputs:

```text
planning/moc.json
alignment/section_assignments.jsonl
sections/001-*.md
map/map_manifest.json
map/chunk_summaries.jsonl
```

Rules:

- write Markdown only to the assigned boundary section file;
- target 500-900 words for `000-overview.md` unless the report target is very
  small;
- target 500-900 words for `999-synthesis.md` unless the report target is very
  small;
- `000-overview.md` uses the MoC thesis, node titles, section opening
  paragraphs, and map manifest warnings to orient the reader;
- `999-synthesis.md` uses section conclusions, repeated high-importance facts,
  unresolved questions, and map manifest warnings to close the report;
- both files should mention at most once that the report summarizes a YouTube
  transcript;
- neither file should introduce new factual claims that are absent from map
  artifacts or generated sections.

## Report Assembly Contract

Python owns final assembly.

`assemble_report.py` reads:

```text
planning/moc.json
alignment/section_assignments.jsonl
sections/*.md
review/coverage.json
review/structured_analysis.md
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
- executive overview from `sections/000-overview.md`;
- MoC-guided narrative sections;
- structured analysis sections built deterministically from fact clusters,
  repeated entities, and coverage artifacts;
- final synthesis from `sections/999-synthesis.md`;
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
  "section_assignment_count": 10,
  "quarantined_chunk_count": 1,
  "json_repair_count": 3,
  "resume_reused_stage_count": 2,
  "redistributed_word_count": 600,
  "reused_fact_warning_count": 4,
  "map_subagent_count": 5,
  "section_writer_subagent_count": 10,
  "qa_subagent_used": true,
  "subagents_used": true,
  "section_expansion_count": 3,
  "coverage_warnings": 2,
  "json_valid": true
}
```

Metrics should make agentic runs comparable with `adaptive_book_report` and
`moc_guided_map_reduce`.

`subagents_used` is a derived compatibility metric. It is true when any
sub-agent count or boolean role metric indicates sub-agent execution; role-level
metrics are authoritative for analysis.

## Error Handling

- Missing transcript: fail fast.
- Transcript normalization failure: fail fast.
- No timestamps: continue, but record degraded timestamp quality.
- Map sub-agent output invalid JSON: attempt lightweight repair, ask for one
  corrected sub-agent output when possible, otherwise quarantine the chunk and
  continue with a warning.
- Map sub-agent output malformed JSON: record repair attempts and failures in
  `map_manifest.json`.
- Existing workspace artifacts: reuse only when validation passes and input
  hashes/options match; otherwise rerun the stage.
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
- Sub-agent unavailable for map extraction: fail or pause before the map stage.
- Sub-agent unavailable for section writing: use sequential main-agent section
  writing when the run policy allows it; otherwise pause before section writing.
- Structured analysis generation failure: continue only when the run policy
  allows omitting structured analysis; otherwise fail before final assembly.
- Assembly failure: fail the run.

## Testing Strategy

Tests should cover both deterministic Python and skill contracts.

Python unit tests:

- word counting;
- token estimation;
- transcript normalization;
- chunk coverage;
- map assignment generation;
- map output validation, stable assembly ordering, and quarantine reporting;
- lightweight JSON repair before requesting a map sub-agent rerun;
- planner context construction from chunk summaries and facts;
- MoC validation, coverage checks, and fallback;
- fact dedupe;
- deterministic fact alignment by chunk id;
- section assignment generation;
- section file discovery;
- generated file ownership validation in shared and isolated sub-agent
  workspaces;
- generated file ownership validation with an expected-files manifest;
- resume hash-scope validation per stage;
- resume manifest validation and stage reuse;
- word-budget redistribution after short complete sections;
- deterministic structured analysis assembly from fact clusters;
- report assembly;
- metrics generation.

Skill contract tests or fixtures:

- each v1 skill references only scripts that exist;
- orchestrator workflow lists all required stages;
- map extraction, MoC, and section-writing stages include example
  input/output contracts;
- section writer contract forbids editing unassigned sections;
- QA contract checks source framing overuse;
- QA contract checks near-duplicate prose when the same fact appears in
  multiple sections;
- sub-agent availability policy is documented.

Integration smoke test:

- use a tiny fixture transcript;
- run deterministic prep tools;
- use mocked map sub-agent outputs with chunk summaries and facts;
- build planner context from mocked map artifacts;
- use mocked LLM artifacts for MoC;
- generate section assignments;
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
map assignment generation, map output validation and assembly, planner context
construction from map artifacts, MoC coverage validation, deterministic
chunk-id alignment, section assignment generation, generated-file ownership
validation, lightweight JSON repair, deterministic structured analysis
assembly, report assembly, and metrics.

### Phase 2: Skill Contracts

Create the six v1 skills with concise `SKILL.md` files, clear script calls,
prompt contracts, and example JSON fixtures. Add targeted `.gitignore`
exceptions if skills are committed under `.agents`.

### Phase 3: Agentic Workspace

Implement workspace conventions, `run_id` creation, artifact contracts, and a
minimal orchestrator flow that can be run manually by Codex. Add completion
manifests, per-stage hash scopes, and resume checks before expensive stages.

### Phase 4: Sub-Agent Workflow

Add map-extractor and section-writer sub-agent dispatch instructions to the
top-level skill and QA skill. Define isolated workspace requirements, file
ownership, `validate_generated_files.py`, qualitative QA, word-budget
redistribution, map-stage fail/pause behavior, and section-writing sequential
fallback.

### Phase 4.5: Boundary Sections

Add orchestrator instructions for `sections/000-overview.md` and
`sections/999-synthesis.md`, then verify assembly uses those files without a
whole-report rewrite. The execution order is node sections, boundary sections,
QA, structured analysis, then final assembly.

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
- Use the same agent model/profile for planner, map extractors, and writers in v1.
  Role-specific model selection can be added later.
- Keep fact extraction in the first MVP, because it is the main guard against
  long fluent sections that miss concrete evidence.
- Make fact extraction a sub-agent stage. Python only prepares assignments,
  validates outputs, repairs JSON, and assembles map artifacts.
- Use the map-first sequence: prep, chunk summaries and fact extraction,
  planner context from map artifacts, MoC planning, deterministic alignment.
- Keep transcript prep and fact-to-MoC alignment Python-only in v1.
- Default `planner_context_target_tokens` to the smaller of 40000 estimated
  tokens and 40 percent of the configured planner model context window.
- Validate MoC coverage deterministically before section writing.
- Support resume-on-crash with artifact manifests, input hashes, and
  validation-gated stage reuse.
- Attempt lightweight JSON repair before requesting a map sub-agent rerun.
- Redistribute unused section word budget to later high-density nodes instead
  of forcing filler expansion.
- Require sub-agent support for map extraction in v1; do not replace it with
  direct Python LLM API calls.
- Allow section writing to fall back to sequential main-agent execution after
  map extraction when section-writer sub-agents are unavailable.

## Open Questions

1. After manual research runs, should the adaptive
   `planner_context_target_tokens` policy also account for transcript length
   and observed MoC JSON size?
