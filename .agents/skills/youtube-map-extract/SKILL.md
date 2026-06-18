---
name: youtube-map-extract
description: Use when a map extractor sub-agent receives YouTube transcript chunk assignment JSON.
---

# YouTube Map Extract

## Overview

Extract evidence from assigned transcript chunks only. Write JSON only to the assignment's declared `output_file`.

Direct LLM API calls are forbidden in this workflow. Use the current agent reasoning context and the assignment file; do not call external model APIs.

## Input Contract

Read one assignment from `map/assignments/*.assignment.json`. Treat every file outside the declared `output_file` as read-only.

## Output Contract

Write exactly one JSON object to `map/agent_outputs/<chunk_id>.json` with:

- `chunk_id`
- `time_range`
- `chunk_summary`
- `claims`
- `examples`
- `quotes`
- `entities`
- `open_questions`
- `facts`

Facts use local ids only. `assemble_map_artifacts.py` creates canonical global fact ids.

## Rules

- Extract from assigned transcript text only.
- Prefer over-extraction to missed evidence.
- Preserve timestamps when present.
- Do not add Markdown wrappers or commentary.
