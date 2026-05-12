# YouTube Transcript Reading Mode Design

Date: 2026-05-12

## Goal

Make YouTube transcript reading feel like a continuous text transcript instead of a stack of repeated panels.

The reader should group nearby caption segments into paragraph-like rows. Each row keeps a clickable start timestamp, but the text becomes the primary surface. Segment metadata remains available without dominating the layout.

## Current Problem

The current YouTube transcript reader renders every caption segment as its own bordered card:

- one border and background per segment;
- repeated badges and copy controls on every row;
- short caption fragments that force the page to look like a grid of panels.

This is useful for debugging raw caption data, but it is visually noisy for reading and analysis.

## Proposed Behavior

Introduce a local presentation model for transcript groups.

Each group contains:

- `id`: stable id derived from the first segment item id/ref;
- `startSeconds`: timestamp for the first segment in the group;
- `content`: joined text from all segments in the group;
- `items`: source reader items included in the group;
- `selected`: true when any included item is selected;
- `captionLabel`: shared label when all included items use the same label;
- `sourceId`: shared source id;
- `refs`: included trace refs for testing and future interactions.

Grouping rules:

- start a new group when there is a large pause between segments;
- start a new group when the current group is already long enough and the previous text ends with sentence punctuation;
- start a new group when adding another segment would make the paragraph too long;
- keep a fallback that still renders every item when only snapshot reader items are available and timing is missing.

Initial thresholds:

- pause threshold: `2000ms`;
- preferred paragraph length: `360` characters;
- hard paragraph length: `560` characters.

These values keep paragraphs readable without turning the transcript into long walls of text.

## UI Design

Render the transcript as one continuous list:

- no border, radius, or panel background on ordinary rows;
- a thin divider between groups;
- timestamp column on the left with tabular blue links;
- main text column in the center;
- compact actions/metadata on the right, visually muted;
- selected group gets a subtle background and a left accent line;
- hover can reveal or strengthen actions, but actions must remain keyboard accessible.

The transcript should feel closer to a reading pane than a table of cards.

## Search And Selection

Search continues to work through the existing filtered `readerItems` input. Grouping happens after filtering, so matching segments naturally appear inside grouped paragraphs.

When `selectedTraceRef` matches any segment inside a group, the whole group is selected. This keeps trace navigation reliable while avoiding complex inline highlight work in the first pass.

## Non-Goals

This pass does not add:

- inline highlighting of a selected segment inside a grouped paragraph;
- a raw segment/debug mode switch;
- semantic NLP paragraph detection;
- changes to storage or backend transcript sync.

## Tests

Add source-level tests for:

- transcript grouping helper and thresholds;
- grouped rows replacing per-segment panel styling;
- selected state applying at group level;
- compact continuous transcript styling without ordinary row cards.

Run the existing Svelte checks and unit tests before merging.
