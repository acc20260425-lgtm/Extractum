# YouTube Source Specialization Design

Date: 2026-06-04

## Goal

Make YouTube sources in Analysis workspace feel provider-aware, trustworthy, and report-ready without forking the overall Source Browser architecture.

This spec covers a separate workstream after the completed `2026-06-03 UX/UI follow-up` plan. It focuses on YouTube video and playlist source states, report corpus selection, evidence inventory, and sync activity.

## Reference Sketches

- `reference/ux-panel-sketches-2026-06-04/youtube-source-overview.html`
- `reference/ux-panel-sketches-2026-06-04/youtube-report-corpus.html`
- `reference/ux-panel-sketches-2026-06-04/youtube-playlist-problem.html`
- `reference/ux-panel-sketches-2026-06-04/youtube-evidence-activity.html`

These sketches are visual references only. The implementation should keep using the existing Analysis route, Source Browser shell, and UI primitives.

## Observed Problems

The live app was reviewed against several YouTube sources:

- a normal video with transcript and comments;
- a longer video with transcript and a stale playlist error still visible;
- a playlist whose typed metadata failed validation;
- report setup for a YouTube single-source scope;
- YouTube Items, Metadata, Activity, Transcript, and Comments tabs.

Findings:

1. A playlist validation error remained visible after switching to a different YouTube video. The alert was stored as global route status instead of a source-scoped detail error.
2. Source switcher and source detail can disagree about YouTube state. Example: the switcher showed `live ended transcript pending`, while the opened video detail showed synced captions and transcript rows.
3. A broken playlist detail looked like an empty playlist: `0 videos`, `0 linked`, `Sync all`, and `Retry failed` were visible even though the root problem was invalid typed metadata.
4. Report setup has the right `YouTube corpus` control, but it is visually equal to dates and model settings even though it is the primary YouTube analysis decision.
5. Transcript and comments readers repeat the title and show noisy status labels such as `Comments Comments synced`.
6. Items view exposes raw evidence as database-flavored rows with `Source #...` and raw comment IDs as primary text.
7. Activity does not explain provider steps. It can show `synced` and no recent jobs while the user still needs to know whether metadata, transcript, and comments are current.

## Product Principles

- YouTube-specific UI should clarify source truth, not add a second visual language.
- Every error must belong to the source that produced it.
- YouTube report setup must make the selected evidence corpus explicit before the user runs a model.
- Comments are audience evidence. They must be visually and semantically distinct from transcript and description evidence.
- Playlist validation failures are problem states, not empty states.
- Source Browser remains the canonical navigation shell for source material.

## In Scope

### Correctness And State

- Add source-scoped YouTube detail error state in `/analysis`.
- Clear or replace YouTube detail errors on source switch, group switch, and successful detail load.
- After successful YouTube detail load, refresh the matching `youtubeSummaries[source.id]` entry from `detail.summary` so the source switcher and opened detail share the same current status.
- Pass the scoped detail error into the source browser and report setup.
- Make report disabled reasons use the same root cause as the source problem panel when YouTube detail is invalid.
- Keep stale global status messages from appearing as current-source errors.

### YouTube Video Source UI

- Replace repeated title/status clusters with a compact provider header inside YouTube readers.
- Format duration as `m:ss` or `h:mm:ss`.
- Show channel, published date, duration, transcript segment count, comments count, and last sync in a compact status strip.
- Remove duplicated labels such as `Comments Comments synced`.
- Keep sync actions available, but secondary to the reader content.

### Report Corpus Selection

- Promote `YouTube corpus` from a normal select to a provider-specific decision block.
- Show the three corpus modes with availability and counts:
  - `Transcript`;
  - `Transcript + description`;
  - `Transcript + description + comments`.
- Mark comments as audience-generated evidence.
- Disable or explain corpus options when required evidence is unavailable.

### Playlist Problem State

- Render invalid playlist metadata as a problem-first state.
- Do not show `0 videos` as the primary message when playlist detail validation failed.
- Show one root cause, one disabled report reason, and one next action set.
- Keep ordinary empty playlists distinct from invalid playlist detail.

### Evidence Inventory And Activity

- Present YouTube Items as evidence groups rather than raw storage rows:
  - Transcript;
  - Description;
  - Comments;
  - Other archived items.
- Preserve trace refs, author, timestamp, and source category for report evidence.
- Add provider-step activity summary:
  - Metadata;
  - Transcript;
  - Comments;
  - Warnings and unavailable states.

## Out Of Scope

- Backend schema changes.
- New YouTube ingestion jobs.
- YouTube source-group NotebookLM export.
- Transcript virtualization.
- A full comments moderation system.
- Changing report prompt templates.
- Replacing the Source Browser shell.

## Architecture

Keep route-owned loading and Tauri API calls in `src/routes/analysis/+page.svelte`.

Add a small pure view-model module:

- `src/lib/youtube-source-view-model.ts`

This module should format YouTube provider display data and derive UI states without importing Svelte components or Tauri APIs.

Route-level state should pass through existing component ownership:

- `/analysis` route owns `youtubeVideoDetail`, `youtubePlaylistDetail`, loading flags, the new source-scoped detail error, and the update that reconciles loaded YouTube detail summaries back into `youtubeSummaries`.
- `ReportCanvas` passes YouTube detail and error state to `ReportSetupPanel` and `ReportSourceSurface`.
- `ReportSourceSurface` passes source-browser data into `SourceBrowserShell`.
- `SourceBrowserShell` routes provider data into YouTube leaf views.
- YouTube leaf views stay display-only and receive all callbacks as props.

## Proposed Data Shapes

### Source-Scoped Detail Error

```ts
export type YoutubeDetailErrorState = {
  sourceId: number;
  sourceSubtype: string | null;
  message: string;
} | null;
```

Use `sourceId` as the authority. A YouTube detail error should render only when the selected source id matches the error source id.

### Provider Header Summary

```ts
export type YoutubeProviderHeaderSummary = {
  title: string;
  sourceKind: "video" | "playlist";
  channelLabel: string;
  durationLabel: string | null;
  publishedLabel: string | null;
  canonicalUrl: string | null;
  thumbnailUrl: string | null;
  availabilityLabel: string;
  captionsLabel: string;
  captionsCountLabel: string;
  commentsLabel: string;
  commentsCountLabel: string;
};
```

### Corpus Option

```ts
export type YoutubeCorpusOptionView = {
  value: "transcript_only" | "transcript_description" | "transcript_description_comments";
  label: string;
  description: string;
  countLabel: string;
  available: boolean;
  disabledReason: string | null;
  evidenceWarning: string | null;
};
```

## Error Handling

- `loadYoutubeDetail(source)` clears stale YouTube detail error before starting a new request for the selected source.
- When a request fails and its request key still matches, it sets `youtubeDetailError` with the failing source id and does not overwrite unrelated current-source state.
- `resetYoutubeDetailState()` clears `youtubeDetailError`.
- The global `status` can still show transient operation messages, but it must not carry source-specific detail errors across source switches.
- Playlist detail views must distinguish:
  - loading;
  - detail unavailable because it was not requested yet;
  - typed metadata invalid;
  - valid detail with zero playlist items.

## Component Design

### YouTube Header

Use a compact source identity block above transcript/comments/videos. It should show:

- thumbnail or provider avatar;
- source kind;
- title clamped to avoid forcing a tall first viewport;
- channel;
- duration;
- published date;
- availability;
- transcript and comments counts.

### Transcript

Keep the existing grouped transcript model. Tighten the header:

- show `YouTube transcript` and compact provider status;
- avoid repeating the full workspace title when the surrounding source header already has it;
- keep search visible and compact;
- keep timestamp links and copy actions.

### Comments

Rename `Search loaded comments` to `Search comments`.

Keep `Threaded`, `Flat`, and `Most liked`, but show the selected mode more clearly. Add concise copy for audience-generated content. Constrain long replies or long thread bodies with a bounded scroll region in the first pass.

### Items

The Items tab should read as evidence inventory:

- evidence role label first;
- preview second;
- raw source id and external id as muted metadata;
- comment IDs not primary copy.

### Activity

For YouTube sources, the Activity tab should summarize provider steps even when there are no recent jobs. Recent jobs remain below the summary.

## Acceptance Criteria

- Switching from a playlist with invalid detail metadata to a normal YouTube video clears the playlist error from the visible UI.
- Source switcher summary and opened source detail no longer communicate contradictory YouTube status for the same source.
- Invalid playlist detail renders as a problem-first panel and does not look like a normal empty playlist.
- Report setup for YouTube shows corpus options with counts and evidence warnings before model settings.
- `Run report` disabled copy matches the root source problem when YouTube detail is invalid.
- Transcript and comments status labels do not duplicate words.
- YouTube Items read as evidence groups, not storage rows.
- Activity shows metadata, transcript, comments, warnings, and recent jobs for YouTube sources.
- Existing Telegram source browsing, source groups, run snapshots, and Accounts YouTube access behavior stay unchanged.

## Testing Strategy

Use existing Vitest raw-source and pure helper tests:

- Add `src/lib/youtube-source-view-model.test.ts` for view-model helpers.
- Add `src/lib/analysis-youtube-source-specialization.test.ts` for route/component contracts.
- Extend `src/lib/source-browser-model.test.ts` only when helper behavior belongs to the shared source browser model.
- Extend `src/lib/analysis-report-canvas.test.ts` and `src/lib/analysis-report-setup-props.test.ts` for prop threading and corpus setup.
- Extend `src/lib/analysis-source-readers.test.ts` for YouTube reader contracts.

Verification commands:

```powershell
npm.cmd run test -- src/lib/youtube-source-view-model.test.ts src/lib/analysis-youtube-source-specialization.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-setup-props.test.ts src/lib/analysis-source-readers.test.ts
npm.cmd run test
npm.cmd run check
npm.cmd run smoke:analysis
```

Manual Tauri checks:

- normal YouTube video with synced transcript and comments;
- YouTube video after switching away from invalid playlist;
- invalid playlist metadata source;
- Report setup with each corpus mode;
- Items and Activity tabs.

## Self-Review

- No backend schema work is required for the requested UX and correctness pass.
- The spec keeps Source Browser as the shared shell and limits provider-specific behavior to route state, view-model helpers, and YouTube leaf components.
- The plan can be implemented in separate tasks that each produce testable behavior.
- The reference sketches are linked, but the implementation is not required to match them pixel-for-pixel.
