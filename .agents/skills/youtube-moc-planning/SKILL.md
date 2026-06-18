---
name: youtube-moc-planning
description: Use when a bounded planner context must become a Map of Content JSON plan.
---

# YouTube MoC Planning

## Overview

Create a global Map of Content from `planning/planner_context.md`. The output is a structure plan, not prose.

Direct LLM API calls are forbidden in this workflow. Write planner JSON with the current agent reasoning context only.

## Input Contract

- `planning/planner_context.md`
- `map/chunk_summaries.jsonl`
- `map/map_manifest.json`
- target words
- output language

## Output Contract

Write raw JSON to `planning/moc.raw.json`. Python validation owns `planning/moc.json`.

Required shape:

- `report_title`
- `source_kind`
- `report_thesis`
- `target_words`
- `nodes`

Each node must include `node_id`, `title`, `purpose`, `target_words`, `time_range`, `chunk_ids`, `key_questions`, and `required_fact_types`.

## Rules

- Cover every important chunk.
- Prefer chronological order unless a thematic structure is explicit.
- Do not invent facts.
- Keep total node budgets close to requested target words.
