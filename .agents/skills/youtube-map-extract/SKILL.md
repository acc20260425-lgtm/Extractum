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
Use the assignment's `allowed_fact_types` as the complete enum for `facts[].fact_type`.

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

Each `facts[]` item must contain exactly these fields:

- `local_fact_id`: local id within this chunk, such as `fact_001`;
- `text`: evidence statement extracted from the assigned transcript text;
- `fact_type`: one of the assignment's `allowed_fact_types` values;
- `timestamp`: best available timestamp string from the chunk, or `null` when no
  timestamp exists;
- `importance`: integer from 1 to 5;
- `chunk_id`: the assignment `chunk_id`.

Facts use local ids only. `assemble_map_artifacts.py` creates canonical global
fact ids. Do not rename these fields, omit them, or replace `facts` with another
shape.

## Rules

- Extract from assigned transcript text only.
- Prefer over-extraction to missed evidence.
- Preserve timestamps when present.
- Do not add Markdown wrappers or commentary.
