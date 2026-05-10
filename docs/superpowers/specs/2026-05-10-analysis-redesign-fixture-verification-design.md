# Analysis Redesign Fixture Verification Design

Date: 2026-05-10
Scope: fixture-backed browser verification for `/analysis` result-first redesign

## Purpose

The `/analysis` result-first redesign Part 7 verification recorded many browser scenarios as `BLOCKED` because the local development database had no representative source, run, snapshot, and chat data. This design adds a debug-only fixture layer that can seed those scenarios into a development database, verify the UI through the real Tauri and SQLite paths, and then remove only the seeded records.

This is verification infrastructure, not a product feature. It must not change release behavior, redesign `/analysis`, add a new visible workspace surface, or add a Playwright dependency.

## Recommended Approach

Use a debug-only backend seed and clear command:

- `seed_analysis_redesign_fixtures`
- `clear_analysis_redesign_fixtures`

Both commands are registered only under `debug_assertions`. They operate on the existing `sqlite:extractum.db` connection so browser verification exercises the same command handlers and route data loading used by real development data.

This approach is preferred over a separate fixture database because it requires less change to the existing Tauri SQL preload flow. It is preferred over frontend mocks because it verifies the real backend, compression, database rows, route loading, and component rendering path.

## Safety Model

Fixture rows must be identifiable and removable without touching user-created data. The seed set uses a stable marker:

- source titles, group names, scope labels, and prompt/template names begin with `__analysis_redesign_fixture__`;
- source `external_id` values use a namespaced fixture prefix;
- JSON metadata stored in compressed metadata columns includes `"analysis_redesign_fixture": true` where practical;
- run result text and trace payloads include fixture-specific labels.

`clear_analysis_redesign_fixtures` deletes only rows that match this marker set. It must delete dependent rows in an order that respects foreign keys and existing cascade behavior:

1. analysis chat rows for fixture runs;
2. analysis run snapshot rows for fixture runs;
3. fixture analysis runs;
4. fixture source group memberships and groups;
5. YouTube playlist rows for fixture playlist/video sources;
6. YouTube transcript segments and source items for fixture sources;
7. fixture sources.

The clear command must be safe when no fixture rows exist. The seed command must be idempotent by clearing the fixture set first, then inserting a fresh deterministic dataset.

## Fixture Dataset

The fixture set covers every browser scenario that was blocked by missing local data in `docs/superpowers/verification/2026-05-10-analysis-redesign.md`.

### Sources

Create these sources:

- Telegram channel source with timeline messages.
- Telegram supergroup source with topic metadata, replies, reaction counts, and media-placeholder metadata.
- YouTube video source with transcript detail and timestamp segments.
- YouTube playlist source with playlist membership rows that link to the video source.

Source rows use realistic titles, provider types, subtypes, sync timestamps, and item counts so `/analysis` can render the compact source rail, setup state, source readers, and source group reader without network access.

### Source Group

Create a fixture source group with multiple same-provider members. The primary group should be Telegram so the grouped timeline scenario can verify that messages remain grouped by source instead of being merged into a pseudo-chat.

### Runs

Create these analysis runs:

- completed single-source run with result markdown, trace data, snapshot rows, and chat history;
- completed single-source run with result markdown and trace data but no snapshot rows, to verify explicit missing-snapshot degradation;
- running run with no snapshot rows, to verify pending snapshot state and disabled chat;
- failed run with error text;
- cancelled run with cancellation-style error text;
- completed group-scoped run with snapshot rows.

The completed snapshot-backed run must include trace refs that point at `analysis_run_messages` rows. At least one trace ref should exercise Telegram content and one should exercise YouTube timestamp metadata.

### Source Items

Create source `items` rows that exercise:

- Telegram text messages;
- Telegram forum topic mapping through `telegram_forum_topics`;
- reply metadata;
- reaction metadata;
- media placeholder metadata without binary previews;
- YouTube transcript item;
- YouTube comment item if needed for corpus-mode labels;
- YouTube transcript segments in `youtube_transcript_segments`;
- playlist membership in `youtube_playlist_items`.

Compressed fields must use the same zstd helpers as production code, including `content_zstd`, `raw_data_zstd`, `media_metadata_zstd`, `metadata_zstd`, `analysis_run_messages.content_zstd`, and `analysis_runs.trace_data_zstd`.

## Command Contract

`seed_analysis_redesign_fixtures` returns a serializable summary:

```ts
interface AnalysisRedesignFixtureSummary {
  sources: number;
  sourceGroups: number;
  runs: number;
  snapshotMessages: number;
  chatMessages: number;
  youtubeTranscriptSegments: number;
  youtubePlaylistItems: number;
}
```

`clear_analysis_redesign_fixtures` returns the same shape with counts for deleted rows.

In development browser automation, the command can be invoked through the Tauri global, for example:

```js
await window.__TAURI__.core.invoke("clear_analysis_redesign_fixtures");
await window.__TAURI__.core.invoke("seed_analysis_redesign_fixtures");
```

The implementation plan may add a tiny typed frontend wrapper only if it helps browser verification. It must not add a visible `/analysis` control unless explicitly requested later.

## Verification Workflow

The fixture-backed browser verification flow is:

1. Start the Tauri dev app or development browser setup used for `/analysis` verification.
2. Invoke `clear_analysis_redesign_fixtures`.
3. Invoke `seed_analysis_redesign_fixtures`.
4. Open `/analysis`.
5. Exercise the browser scenarios previously marked `BLOCKED`.
6. Update `docs/superpowers/verification/2026-05-10-analysis-redesign.md` with real `PASS` or `FAIL` results.
7. Invoke `clear_analysis_redesign_fixtures` again.
8. Record any remaining fixture or standalone-browser limitations as residual risks.

Browser verification remains documented rather than checked in as Playwright tests. The redesign verification contract already avoided adding Playwright as a dependency, and this fixture layer should preserve that boundary.

## Automated Tests

Backend tests should cover the fixture infrastructure before any browser verification is trusted:

- seeding twice leaves one deterministic fixture dataset rather than duplicates;
- clearing removes fixture rows and leaves non-fixture rows untouched;
- the completed snapshot-backed run has saved `analysis_run_messages` rows;
- the completed missing-snapshot run has no saved snapshot rows;
- fixture statuses include completed, running, failed, and cancelled runs;
- fixture data includes Telegram topic/media metadata, YouTube transcript segments, playlist membership, and source group membership.

Tests should use in-memory SQLite where possible. If the fixture code depends on the full migration schema, tests may apply the existing migration SQL to an in-memory pool before seeding.

## Non-Goals

- No release-build seed commands.
- No visible product UI for fixtures.
- No new `/analysis` product behavior.
- No source ingest jobs in `RunCompanionTabs.Runs`.
- No live-source fallback for completed-run evidence or chat.
- No Playwright dependency.
- No edits to the completed Part 7 verification results until the fixture-backed browser scenarios have actually been exercised.

## Open Operational Note

The existing development environment may require elevated process control for starting or stopping the dev server, as recorded during Part 7. That operational constraint should be recorded honestly during verification but does not change the fixture design.
