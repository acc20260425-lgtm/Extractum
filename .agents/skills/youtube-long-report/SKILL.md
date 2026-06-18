---
name: youtube-long-report
description: Use when creating a long file-backed report from a YouTube transcript with agentic MoC map-reduce artifacts.
---

# YouTube Long Report

## Overview

Use this as the orchestrator for the agentic MoC workflow. Python owns deterministic mechanics; agent and sub-agent reasoning owns extraction, planning, section prose, and review notes.

Direct LLM API calls are forbidden in this workflow. Do not replace map extraction, MoC planning, section writing, or QA judgment with Python API calls.

## Inputs

- transcript path
- run directory
- output language
- target report words
- optional chunk/token settings

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
