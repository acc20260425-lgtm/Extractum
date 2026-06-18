---
name: youtube-section-reduce
description: Use when a section writer receives one MoC section assignment for a YouTube report.
---

# YouTube Section Reduce

## Overview

Write one Markdown section from one row in `alignment/section_assignments.jsonl`. Use assigned facts before general narrative.

Direct LLM API calls are forbidden in this workflow. Do not call external model APIs for section prose.

## Input Contract

- one section assignment object
- `planning/moc.json`
- `alignment/alignment.json`
- `alignment/deduplicated_facts.json`
- `prep/chunks.jsonl`
- `map/chunk_summaries.jsonl`

## Output Contract

Write exactly the declared `section_file`, such as `sections/001-node-title.md`.

## Rules

- Treat all other workspace files as read-only.
- Preserve timestamps where useful.
- Do not repeat the phrase "this video summary" throughout the section.
- If the section is under 80 percent of target words, expand with missing assigned facts.
- If expansion would add filler, keep the shorter section and leave a note for budget redistribution.
