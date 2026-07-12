# YouTube Summary User-Facing Skill Design

Date: 2026-06-18

Status: draft for user review.

## Goal

Add a user-facing `youtube-summary` skill that lets the user ask for a long
summary from a transcript file in one natural request:

```text
Use youtube-summary.
File: research/youtube_pipeline/inputs/a9_k-meLQaYP5Y_en_orig.txt
Language: ru
Length: about 10000 words
```

The skill should hide the existing mechanical Python commands from the user.
Python tools remain the deterministic execution layer, but the user should not
manually run `prep_all`, `prepare_map_assignments`, validation, dedupe,
alignment, QA, or assembly commands.

## Problem

The current agentic MoC workflow works end-to-end, but it feels like a lab
bench:

- the user must know the exact command order;
- the user must inspect intermediate directories manually;
- the user must know when to call map extraction, MoC planning, section writing,
  QA, and assembly;
- validation failures are useful, but the recovery prompts are still manual;
- small path mistakes break the flow.

This is acceptable for building the mechanism, but it is not the intended
product experience. The product experience should be one skill invocation.

## Non-Goals

- No direct LLM API calls from Python.
- No Tauri app integration in this slice.
- No replacement of the existing deterministic tools.
- No final whole-report LLM rewrite.
- No hidden prompt-pack integration with the existing production
  `youtube_summary` pack.
- No attempt to make arbitrary transcript sizes finish without agent work. The
  workflow still requires map extraction by sub-agents, and section writing by
  sub-agents or by the main agent only after valid map outputs exist.

## Relationship To Existing Work

This design builds on:

- `.agents/skills/youtube-long-report/SKILL.md`
- `.agents/skills/youtube-map-extract/SKILL.md`
- `.agents/skills/youtube-moc-planning/SKILL.md`
- `.agents/skills/youtube-section-reduce/SKILL.md`
- `.agents/skills/youtube-report-qa/SKILL.md`
- `research/youtube_pipeline/tools/*.py`
- `docs/superpowers/specs/2026-06-18-youtube-agentic-moc-skills-design.md`

The new `youtube-summary` skill is a public wrapper. The existing
`youtube-long-report` skill becomes an internal workflow reference that
`youtube-summary` may reuse.

## User Experience

The user asks:

```text
Use skill youtube-summary.
Here is the transcript file: <path>
Write a long Russian summary, about 10000 words.
```

The skill responds with progress updates such as:

```text
Created run:
research/youtube_pipeline/runs/manual/youtube_summary_agentic/<slug>/<run_id>

Prepared 8 map assignments. Starting map extraction.
```

The skill owns the rest of the workflow:

1. Create or resume a run.
2. Run transcript prep and map assignment creation.
3. Dispatch map extraction.
4. Validate and repair or request corrections.
5. Build planner context.
6. Create and validate MoC.
7. Deduplicate and align facts.
8. Prepare section assignments.
9. Dispatch section writing.
10. Write overview and synthesis.
11. Run QA, structured analysis, and final assembly.
12. Return `final/report.md`.

The user should see artifact paths and high-level progress, not command
bookkeeping.

## Components

### 1. `youtube-summary` Skill

Path:

```text
.agents/skills/youtube-summary/SKILL.md
```

Responsibilities:

- parse the user request for transcript path, output language, target words,
  and optional run directory;
- run the bootstrap helper;
- read `workflow_state.json`;
- call Python tools for deterministic stages;
- dispatch child skills for LLM reasoning stages;
- recover from validation failures by asking for corrected files, not by
  guessing;
- resume from existing `workflow_state.json` when possible;
- update `workflow_state.json` after every validated stage transition;
- write orchestrator-owned overview and synthesis files after section
  validation;
- return final report path and metrics.

The skill should allow the user to say `youtube_summary` or `youtube-summary`,
but the local skill folder should use the hyphenated name.

### 2. `start_youtube_summary.py`

Path:

```text
research/youtube_pipeline/tools/start_youtube_summary.py
```

Responsibilities:

- validate the transcript file exists;
- create a run directory when the user did not provide one;
- compute the transcript hash and options hash used for resume lookup;
- reuse the latest matching run unless `--force` is passed;
- choose defaults for chunk size and overlap;
- call the same underlying Python functions used by `prep_all.py` and
  `prepare_map_assignments.py`;
- write `workflow_state.json`;
- update the run index under the configured run root;
- print the run directory and next action.

This helper does not call an LLM API. It is a deterministic bootstrapper.

Example command used internally by the skill:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary `
  --transcript research/youtube_pipeline/inputs/a9_k-meLQaYP5Y_en_orig.txt `
  --language ru `
  --target-words 10000
```

Optional flags:

```text
--run-dir <path>
--target-tokens <int>
--overlap-tokens <int>
--planner-context-tokens <int>
--force
```

Resume lookup rules:

- if `--run-dir` is supplied, use that exact run and fail if its
  `workflow_state.json` is missing or incompatible;
- otherwise compute `transcript_sha256` from the transcript bytes and
  `options_hash` from normalized workflow options;
- search `run_root/run_index.json` for the latest run with matching
  `transcript_sha256` and `options_hash`;
- resume that run when found;
- create a new run when no match exists or when `--force` is supplied.

`options_hash` must include only stable workflow-affecting fields:

- `schema`;
- `output_language`;
- `target_words`;
- `target_tokens`;
- `overlap_tokens`;
- `planner_context_tokens`;
- workflow skill version when available.

It must exclude volatile paths and timestamps such as `run_dir` and run
creation time.

`run_index.json` updates must be atomic: write a temporary file in the same
directory, then replace the index file. If the index is missing or unreadable,
the helper may rebuild it by scanning child run directories for valid
`workflow_state.json` files.

### 3. `workflow_state.json`

Path:

```text
<run-dir>/workflow_state.json
```

Shape:

```json
{
  "schema": "youtube-summary-workflow-state-v1",
  "run_dir": "research/youtube_pipeline/runs/manual/youtube_summary_agentic/a9/20260618-161500",
  "transcript_path": "research/youtube_pipeline/inputs/a9_k-meLQaYP5Y_en_orig.txt",
  "transcript_sha256": "4f8c...",
  "output_language": "ru",
  "target_words": 10000,
  "options_hash": "9a21...",
  "current_stage": "map_assignments_ready",
  "next_action": "dispatch_map_extractors",
  "artifacts": {
    "chunks": "prep/chunks.jsonl",
    "assignment_manifest": "map/assignment_manifest.json"
  },
  "counts": {
    "chunk_count": 8,
    "map_assignment_count": 8
  },
  "commands": {
    "validate_map_outputs": "python -m research.youtube_pipeline.tools.validate_map_outputs --run-dir <run-dir>",
    "assemble_map_artifacts": "python -m research.youtube_pipeline.tools.assemble_map_artifacts --run-dir <run-dir>"
  }
}
```

The state file is for the skill, not for the user. It exists so the skill can
resume and can explain what it is doing.

`start_youtube_summary.py` creates the initial state. After that, the
`youtube-summary` skill owns state updates. It must update
`current_stage`, `next_action`, `artifacts`, `counts`, and validation warnings
after each successful deterministic gate. If the implementation uses a helper,
it should be a deterministic state updater such as:

```text
research/youtube_pipeline/tools/update_youtube_summary_state.py
```

That helper may inspect existing manifests and validation files, but it must not
perform LLM reasoning.

## Workflow State Machine

```text
created
  -> map_assignments_ready
  -> map_outputs_ready
  -> map_assembled
  -> planner_context_ready
  -> moc_ready
  -> alignment_ready
  -> sections_ready
  -> qa_ready
  -> final_ready
```

Each transition is gated by deterministic validation:

- map outputs: `validate_map_outputs.py`;
- MoC: `validate_moc.py`;
- generated files: `validate_generated_files.py`;
- section coverage: `quality_check.py`;
- final assembly: `assemble_report.py`.

## Agent Work Policy

Python never performs LLM reasoning.

The skill uses:

- `youtube-map-extract` for map outputs;
- `youtube-moc-planning` for `planning/moc.raw.json`;
- `youtube-section-reduce` for node sections;
- `youtube-report-qa` for qualitative review notes.

When sub-agents are available, the skill should use them for map extraction and
section writing.

Map extraction keeps the approved agentic boundary: if sub-agents are
unavailable before map extraction, the skill must pause and explain that the
workflow cannot proceed until map extractor sub-agents are available. It must
not replace map extraction with Python LLM API calls or hidden main-agent
reasoning.

After valid map outputs exist, section writing may fall back to sequential
main-agent execution using the same `youtube-section-reduce` contract when
section-writer sub-agents are unavailable. The skill must record this in
metrics or review notes when the artifact format supports it. Python API
fallback remains forbidden.

## Overview And Synthesis Ownership

The `youtube-summary` orchestrator skill owns:

- `sections/000-overview.md`;
- `sections/999-synthesis.md`.

It writes these files after section files pass generated-file validation and
before final assembly. Inputs should be limited to the validated MoC thesis,
node titles, section opening paragraphs, section conclusions, and repeated
high-importance facts so the overview and synthesis do not become a
whole-report rewrite stage.

## Error Handling

Validation failure should produce exact recovery instructions.

Examples:

- Missing map output: dispatch the relevant `youtube-map-extract` assignment.
- Invalid map output schema: ask the map extractor to rewrite the exact output
  file with the missing fields.
- Invalid MoC: request one corrected `moc.raw.json`, then use deterministic
  fallback if correction fails.
- Missing section file: rerun the exact section assignment.
- QA source framing overuse: ask for targeted section rewrite, not whole-report
  rewrite.

The skill should not continue silently past invalid artifacts.

## Defaults

Recommended defaults:

```text
output_language = ru
target_words = 10000
target_tokens = 1600
overlap_tokens = 200
planner_context_tokens = 24000
run_root = research/youtube_pipeline/runs/manual/youtube_summary_agentic
```

If the transcript is small, `start_youtube_summary.py` may emit only one or two
map assignments. If it is long, it should keep assignments chunked so map
extractors stay manageable.

## Acceptance Criteria

- A user can start the workflow with one skill request and a transcript path.
- The user does not need to manually run the deterministic Python commands.
- `start_youtube_summary.py` creates prep artifacts, map assignments, and
  `workflow_state.json`.
- `youtube-summary` can resume from an existing run directory.
- `youtube-summary` can discover the latest matching run from transcript and
  options hashes when no run directory is supplied.
- `workflow_state.json` is updated after each successful validated transition.
- Map extraction pauses when sub-agents are unavailable instead of falling back
  to main-agent or Python LLM reasoning.
- Overview and synthesis ownership is explicit and belongs to the orchestrator
  skill.
- The skill preserves the existing no-direct-LLM-API rule.
- The final response includes the path to `final/report.md`, `final/metrics.json`,
  and any validation warnings.
- Unit tests cover `workflow_state.json` creation.
- Skill contract tests verify that `youtube-summary` references existing tools
  and forbids direct LLM API calls.

## Implementation Notes

This is a UX wrapper, not a new summarization algorithm. Keep the lower-level
tools and child skills unchanged unless a pilot reveals a contract bug.

The first implementation should be intentionally small:

1. add `start_youtube_summary.py`;
2. add `.agents/skills/youtube-summary/SKILL.md`;
3. add deterministic state update or run inspection support;
4. add tests for state creation, resume lookup, state transitions, and skill
   contract;
5. update README with the one-request workflow;
6. run a small pilot with existing fixture or a short transcript.
