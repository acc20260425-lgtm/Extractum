---
name: youtube-summary
description: Use when the user wants a long report from a YouTube transcript with one public skill request.
---

# YouTube Summary

## Overview

Use this as the public wrapper for long file-backed YouTube transcript reports.
The user provides a transcript path and target language/length once; this skill
creates or resumes the run, runs deterministic Python tools, and delegates
reasoning work to the existing YouTube child skills.

Direct LLM API calls are forbidden. Do not use Python, HTTP clients, or provider
SDKs for map extraction, MoC planning, section writing, or QA judgment.

## Inputs

- transcript path;
- output language, default `ru`;
- target report words, default `10000`;
- optional existing run directory;
- optional chunk settings.

## Bootstrap

Run:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary --transcript <path> --language <language> --target-words <words>
```

Use `--run-dir <path>` when the user gives an explicit run. Use `--force` only
when the user asks for a fresh run instead of resuming a matching run.

Read `<run-dir>/workflow_state.json` after bootstrap. Use its `next_action`,
`artifacts`, `counts`, and `commands` fields to continue.

## Workflow

1. Bootstrap or resume the run with `start_youtube_summary`.
2. If `next_action` is `dispatch_map_extractors`, dispatch `youtube-map-extract`
   sub-agents for files in `map/assignments`.
3. If sub-agents are unavailable before map extraction, pause before map extraction
   and explain that the workflow needs map extractor sub-agents.
4. Validate map outputs:
   `python -m research.youtube_pipeline.tools.validate_map_outputs --run-dir <run-dir>`
5. On valid map outputs, update state:
   `python -m research.youtube_pipeline.tools.update_youtube_summary_state --run-dir <run-dir> --stage map_outputs_ready --next-action assemble_map_artifacts`
6. Assemble map artifacts, build planner context, and update state after each
   deterministic gate.
7. Use `youtube-moc-planning` to write `planning/moc.raw.json`.
8. Validate MoC, dedupe facts, align facts, prepare section assignments, and
   update state after each successful gate.
9. Use `youtube-section-reduce` for section files. If section-writer sub-agents
   are unavailable after valid map outputs exist, the main agent may write
   sections sequentially using the same skill contract.
10. The orchestrator writes `sections/000-overview.md` and
    `sections/999-synthesis.md` after section validation. Use only the validated
    MoC thesis, node titles, section opening paragraphs, section conclusions,
    and repeated high-importance facts.
11. Use `youtube-report-qa` for review notes.
12. Run structured analysis and final assembly:
    `python -m research.youtube_pipeline.tools.build_structured_analysis --run-dir <run-dir>`
    `python -m research.youtube_pipeline.tools.assemble_report --run-dir <run-dir>`

## Error Handling

- Do not continue past invalid artifacts.
- Missing map output: dispatch the relevant `youtube-map-extract` assignment.
- Invalid map schema: ask the extractor to rewrite the exact output file.
- Invalid MoC: request one corrected `planning/moc.raw.json`.
- Missing section file: rerun that exact section assignment.
- Source framing overuse: request targeted section rewrite, not a whole-report
  rewrite.

## Output Contract

Return:

- `final/report.md`;
- `final/metrics.json`;
- validation warnings from `workflow_state.json`;
- any files that still require user or sub-agent action.
