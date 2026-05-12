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
- `startSeconds`: `youtubeStartSeconds` from the first item in the group, or `null` when unavailable;
- `content`: joined text from all segments in the group;
- `items`: source reader items included in the group;
- `selected`: true when any included item is selected;
- `captionLabel`: shared label when all included items use the same non-empty label, otherwise `null`;
- `sourceId`: shared numeric source id when all included items use the same source id, otherwise `null`;
- `refs`: included trace refs for testing and future interactions.

## Grouping Contract

Grouping is a pure presentation-layer helper:

```ts
type TranscriptGroup = {
  id: string;
  startSeconds: number | null;
  content: string;
  items: SourceReaderItem[];
  selected: boolean;
  captionLabel: string | null;
  sourceId: number | null;
  refs: string[];
};
```

For each next item, decide whether it starts a new group before appending it to the current group.

Rule order:

1. If there is no current group, create one from the next item.
2. If the next item has no `youtubeStartSeconds`, render it as a single-item group.
3. If the current group's last item has `youtubeEndSeconds` and the next item has `youtubeStartSeconds`, start a new group when the gap is at least the pause threshold.
4. Start a new group when appending the next item would make the joined content exceed the hard paragraph length.
5. Start a new group when the current content is at least the preferred paragraph length and ends with sentence punctuation.
6. Otherwise append the next item.

The hard length rule is checked before appending. A single segment may exceed the hard length when the source caption text is already longer than the threshold.

Initial thresholds:

- pause threshold: `2` seconds;
- preferred paragraph length: `360` characters;
- hard paragraph length: `560` characters.

These values keep paragraphs readable without turning the transcript into long walls of text.

Sentence punctuation is limited to `.`, `!`, `?`, and `...` in the first pass:

```ts
/(?:[.!?]|\.\.\.)["')\]]?$/
```

The regex allows one trailing closing quote, parenthesis, or bracket.

Joined content is normalized for reading:

```ts
content = items
  .map((item) => item.content.trim())
  .filter(Boolean)
  .join(" ")
  .replace(/\s+/g, " ")
  .trim();
```

This pass does not preserve line breaks inside captions.

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

In search mode, groups are built only from filtered `readerItems`. The first pass does not pull hidden neighboring segments back into the visible paragraph for context.

When `selectedTraceRef` matches any segment inside a group, the whole group is selected. This keeps trace navigation reliable while avoiding complex inline highlight work in the first pass.

Selection is group-level only:

- if `selectedTraceRef` matches any value in `group.refs`, `group.selected` is true;
- selected groups use a subtle background and a left accent line;
- inline segment-level highlighting remains out of scope.

## Non-Goals

This pass does not add:

- inline highlighting of a selected segment inside a grouped paragraph;
- a raw segment/debug mode switch;
- semantic NLP paragraph detection;
- changes to storage or backend transcript sync.

## Tests

Add helper and source-level tests for:

- nearby short caption segments with small timing gaps join into one group;
- a gap of at least `2` seconds starts a new group when both timestamps are available;
- appending a segment that would make content longer than `560` characters starts a new group;
- content at least `360` characters long starts a new group before the next segment when it ends with `.`, `!`, `?`, or `...`;
- preferred length alone does not split a sentence when sentence punctuation is missing;
- selected state applies to the whole group when any included ref matches `selectedTraceRef`;
- mixed caption labels produce `captionLabel: null`;
- mixed source ids produce `sourceId: null`;
- grouped rows replace per-segment panel styling;
- ordinary groups do not render card borders, rounded panel backgrounds, or per-row panel shadows;
- groups are separated by thin dividers and keep keyboard-reachable actions.

Run the existing Svelte checks and unit tests before merging.
