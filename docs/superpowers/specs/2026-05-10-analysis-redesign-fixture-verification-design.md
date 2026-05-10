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

- account labels, source titles, group names, scope labels, prompt/template names, and optional fixture LLM profile ids begin with `__analysis_redesign_fixture__`;
- source `external_id` values use a namespaced fixture prefix;
- JSON metadata stored in compressed metadata columns includes `"analysis_redesign_fixture": true` where practical;
- run result text and trace payloads include fixture-specific labels.

Rows that cannot carry an independent fixture marker must be selected for deletion only through fixture parent ids, such as fixture run ids, fixture source ids, fixture source group ids, or fixture prompt template ids. The clear command must not rely on broad content matching for child rows.

`clear_analysis_redesign_fixtures` deletes only rows that match this marker set or belong to marker-matched fixture parents. It must delete dependent rows in an order that respects foreign keys and existing cascade behavior:

1. analysis chat rows for fixture runs;
2. analysis run snapshot rows for fixture runs;
3. fixture analysis runs;
4. fixture LLM profile settings in `app_settings`;
5. fixture analysis prompt templates;
6. fixture source group memberships and groups;
7. YouTube playlist rows for fixture playlist/video sources;
8. YouTube transcript segments for fixture source items;
9. Telegram forum topics for fixture sources;
10. source items for fixture sources;
11. fixture sources;
12. fixture accounts.

The clear command must be safe when no fixture rows exist. The seed command must be idempotent by clearing the fixture set first, then inserting a fresh deterministic dataset.

Visible fixture labels should be deterministic, human-recognizable, and scenario-oriented so browser verification can select the intended source, group, run, or profile without ambiguity. Examples:

- `__analysis_redesign_fixture__ Telegram Channel`
- `__analysis_redesign_fixture__ Telegram Supergroup`
- `__analysis_redesign_fixture__ YouTube Video`
- `__analysis_redesign_fixture__ YouTube Playlist`
- `__analysis_redesign_fixture__ Telegram Group`
- `__analysis_redesign_fixture__ Completed Snapshot Run`
- `__analysis_redesign_fixture__ Missing Snapshot Run`
- `__analysis_redesign_fixture__ Running Run`
- `__analysis_redesign_fixture__ Failed Run`
- `__analysis_redesign_fixture__ Cancelled Run`
- `__analysis_redesign_fixture__ Group Snapshot Run`
- `__analysis_redesign_fixture__ LLM Profile`

## Fixture Dataset

The fixture set targets the populated browser scenarios that were blocked by missing local data in `docs/superpowers/verification/2026-05-10-analysis-redesign.md`.

This fixture dataset is for populated-state verification. No-source and no-context onboarding states are verified separately by clearing fixtures and using an empty or controlled development database.

### Fixture Accounts

Create one debug-only Telegram account row for fixture Telegram sources. The account uses a fixture label, a harmless placeholder `api_id`, an empty `api_hash`, and no phone number. The seed must not create real credentials, Telegram session files, or secure-store entries.

Fixture Telegram sources reference this account row. This keeps `/analysis` source rendering close to real Telegram source state while avoiding dependence on the user's actual Telegram accounts or authentication state.

The clear command deletes fixture accounts only after fixture sources have been deleted. It must not call account deletion helpers that touch secure storage because the fixture account never writes secrets.

### Prompt Template

Create one fixture `analysis_prompt_templates` row for report runs. Fixture analysis runs reference this template so run metadata can show a deterministic prompt template name through the existing `analysis_runs` to `analysis_prompt_templates` join.

The clear command deletes fixture prompt templates after fixture runs have been deleted. This keeps prompt-template cleanup explicit and avoids leaving fixture labels in the normal template list.

### LLM Profile State

The fixture layer must not create real LLM secrets. It must not write API keys to secure storage and must not create legacy `llm.profile.*.api_key` settings.

Saved fixture runs carry their own provider/profile/model provenance in `analysis_runs`, so saved-run verification does not require a configured active LLM profile. If setup/provenance browser checks need profile metadata, the fixture may create non-secret provider profile settings in `app_settings` using a fixture profile id, provider, default model, and base URL. That metadata must be marked by the fixture profile id and cleared with the rest of the fixture set.

The fixture dataset must not claim successful new report launch without a real usable developer-configured profile. If the current UI allows attempting a launch without an API key, that behavior is outside fixture success criteria and should be recorded as a setup limitation, not masked by fake secrets.

### Sources

Create these sources:

- Telegram channel source with timeline messages.
- Telegram supergroup source with topic metadata, replies, reaction counts, and media-placeholder metadata.
- YouTube video source with transcript detail and timestamp segments.
- YouTube playlist source with playlist membership rows that link to the video source.

Source rows use realistic titles, provider types, subtypes, sync timestamps, and item counts so `/analysis` can render the compact source rail, setup state, source readers, and source group reader without network access.

Fixture sources that are intended to exercise source readers must be seeded in a post-sync-looking state: non-empty source items, plausible `last_synced_at` values, plausible `last_sync_state` values where the provider uses them, and source metadata sufficient for the `/analysis` reader to treat the material as locally available. Do not use these populated fixture sources to verify unsynced or first-sync onboarding prompts.

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

Trace refs must be shaped so the redesigned Evidence panel can resolve one Telegram snapshot message and one YouTube timestamped segment through the same path used by real runs. The snapshot-backed fixture should enable Evidence refs, report inline refs, `Show in source` for a Telegram message, and `Show in source` for a YouTube transcript timestamp. The missing-snapshot run should include at least one trace ref in its trace payload that cannot resolve to saved snapshot rows, so degraded evidence/source behavior can be verified.

### Source Items

Create source `items` rows that exercise:

- Telegram text messages;
- Telegram forum topic mapping through `telegram_forum_topics`;
- reply metadata;
- reaction metadata;
- media placeholder metadata without binary previews;
- YouTube transcript item;
- YouTube comment item for `transcript_description_comments` corpus-mode labels;
- YouTube transcript segments in `youtube_transcript_segments`;
- playlist membership in `youtube_playlist_items`.

Compressed fields must use the same zstd helpers as production code, including `content_zstd`, `raw_data_zstd`, `media_metadata_zstd`, `metadata_zstd`, `analysis_run_messages.content_zstd`, and `analysis_runs.trace_data_zstd`.

### Source Activity And Jobs

This fixture layer seeds persistent database-backed source state. It does not seed in-memory source job or Takeout import job state unless a separate debug-only runtime job fixture is explicitly added. Browser scenarios that require active Takeout progress or active YouTube source job progress may still require manual triggering or remain residual risks.

## Command Contract

`seed_analysis_redesign_fixtures` returns a serializable summary:

```ts
interface AnalysisRedesignFixtureSummary {
  accounts: number;
  llmProfiles: number;
  sources: number;
  sourceGroups: number;
  promptTemplates: number;
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
- clearing deletes child rows through fixture parent ids instead of broad content matching;
- fixture Telegram sources reference fixture account rows, and fixture accounts contain no credentials or secure-store state;
- fixture runs reference a fixture report prompt template, and clearing removes that template;
- fixture LLM profile metadata, if created, stores no API key and clearing removes only fixture profile settings;
- fixture sources intended for reader verification have non-empty items, non-null sync timestamps, and provider metadata that renders as locally available;
- the completed snapshot-backed run has saved `analysis_run_messages` rows;
- the completed missing-snapshot run has no saved snapshot rows;
- fixture trace refs resolve a Telegram snapshot message and a YouTube timestamped segment, while the missing-snapshot run leaves evidence resolution explicitly unavailable;
- fixture statuses include completed, running, failed, and cancelled runs;
- fixture data includes Telegram topic/media metadata, YouTube transcript segments, playlist membership, and source group membership.

Tests should use in-memory SQLite where possible. If the fixture code depends on the full migration schema, tests may apply the existing migration SQL to an in-memory pool before seeding.

## Non-Goals

- No release-build seed commands.
- No visible product UI for fixtures.
- No new `/analysis` product behavior.
- No fake LLM secrets and no successful report-launch guarantee without a real configured profile.
- No source ingest jobs in `RunCompanionTabs.Runs`.
- No live-source fallback for completed-run evidence or chat.
- No Playwright dependency.
- No edits to the completed Part 7 verification results until the fixture-backed browser scenarios have actually been exercised.

## Open Operational Note

The existing development environment may require elevated process control for starting or stopping the dev server, as recorded during Part 7. That operational constraint should be recorded honestly during verification but does not change the fixture design.
