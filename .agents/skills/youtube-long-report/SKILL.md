---
name: youtube-long-report
description: Use for lower-level/manual orchestration of a long file-backed YouTube report when the public youtube-summary wrapper is not being used.
---

# YouTube Long Report

## Overview

Use this as the lower-level/manual orchestrator contract for the agentic MoC
workflow. For normal user-facing runs, prefer the public `youtube-summary`
wrapper, because it creates or resumes runs and owns `workflow_state.json`.

Python owns deterministic mechanics; agent and sub-agent reasoning owns
extraction, planning, section prose, and review notes.

Direct LLM API calls are forbidden in this workflow. Do not replace map extraction, MoC planning, section writing, or QA judgment with Python API calls.

## Inputs

- transcript path
- run directory
- output language
- target report words
- optional chunk/token settings

## Relationship To `youtube-summary`

`youtube-summary` is the public wrapper skill. It starts or resumes a run with:

```powershell
python -m research.youtube_pipeline.tools.start_youtube_summary --transcript <path> --language <language> --target-words <words>
```

It then advances `workflow_state.json` after deterministic gates with:

```powershell
python -m research.youtube_pipeline.tools.advance_youtube_summary_state --run-dir <run-dir> --after <step>
```

Use this `youtube-long-report` skill only for manual research runs, legacy notes,
or child-skill contract work where the wrapper state machine is not the entry
point.

## Workflow

1. Run transcript prep:
   `python -m research.youtube_pipeline.tools.prep_all --transcript <path> --out <run-dir> --language ru --target-tokens 1600 --overlap-tokens 200`
2. Run map assignment prep:
   `python -m research.youtube_pipeline.tools.prepare_map_assignments --run-dir <run-dir> --output-language ru`
3. Dispatch map extractor sub-agents with `youtube-map-extract`.
4. Validate and assemble map artifacts:
   `python -m research.youtube_pipeline.tools.validate_map_outputs --run-dir <run-dir>`
   `python -m research.youtube_pipeline.tools.assemble_map_artifacts --run-dir <run-dir>`
5. Build planner context:
   `python -m research.youtube_pipeline.tools.build_planner_context --run-dir <run-dir> --max-tokens 24000 --language ru`
6. Use `youtube-moc-planning` to write `planning/moc.raw.json`.
7. Validate MoC:
   `python -m research.youtube_pipeline.tools.validate_moc --run-dir <run-dir> --target-words <words>`
8. Deduplicate and align facts:
   `python -m research.youtube_pipeline.tools.dedupe_facts --run-dir <run-dir>`
   `python -m research.youtube_pipeline.tools.align_facts --run-dir <run-dir>`
   `python -m research.youtube_pipeline.tools.prepare_section_assignments --run-dir <run-dir>`
9. Dispatch section writers with `youtube-section-reduce`.
10. Write `sections/000-overview.md` and `sections/999-synthesis.md`.
11. Use `youtube-report-qa`, then assemble:
    `python -m research.youtube_pipeline.tools.build_structured_analysis --run-dir <run-dir>`
    `python -m research.youtube_pipeline.tools.assemble_report --run-dir <run-dir>`

## Output Contract

The final report path is `final/report.md`. Metrics are in `final/metrics.json`; run metadata is in `final/result.json`.
