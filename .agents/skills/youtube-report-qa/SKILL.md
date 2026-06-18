---
name: youtube-report-qa
description: Use when generated YouTube report sections need deterministic coverage checks and qualitative review.
---

# YouTube Report QA

## Overview

Review generated section files after node sections plus overview and synthesis exist. Python owns deterministic checks; the agent records qualitative notes.

Direct LLM API calls are forbidden in this workflow. Do not use direct model API calls as QA fallback.

## Required Commands

Run:

`python -m research.youtube_pipeline.tools.quality_check --run-dir <run-dir>`

Then inspect:

- `review/coverage.json`
- `review/coverage.md`
- generated section files

## Output Contract

Write `review/reviewer_notes.md` when qualitative issues remain. Do not assemble the final report; the orchestrator runs deterministic assembly after QA.

## Review Focus

- unsupported claims;
- repeated prose;
- missing high-importance facts;
- overused source framing words;
- duplicated facts that should be framed differently in each section.
