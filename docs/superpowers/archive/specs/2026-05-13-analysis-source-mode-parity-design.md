# Analysis Source Mode Parity Design

## Context

The result-first `/analysis` redesign moved source browsing into `ReportCanvas`
`Source` mode. The route still owns most of the source runtime state and actions:
YouTube detail loading, YouTube source jobs, comments sync, playlist retry, source
job cancellation, Telegram forum topics, and topic-filtered message loading.

The current central source readers are cleaner than the older transitional
`WorkspaceMain` detail panels, but they do not yet expose the full live source
capability set. This pass restores parity without bringing back the older
stacked detail cards or changing the run snapshot trust model.

## Goal

Restore live `Source` mode parity for YouTube source actions/activity and
Telegram topic filtering while preserving the result-first canvas architecture.

## Non-Goals

- Do not implement `Show in source` report/evidence navigation.
- Do not implement chunk summaries in the source canvas.
- Do not add new backend APIs.
- Do not reintroduce `YoutubeSourceDetail` or `YoutubePlaylistDetail` as the
  primary source canvas surface.
- Do not show live sync, comments, job, or topic controls in run snapshot
  read-only views.
- Do not redesign source groups beyond preserving their current grouped-by-source
  reading behavior.

## Approved Approach

Extend the new source readers instead of reverting to the old detail panels.

YouTube video live source views remain transcript-first. The transcript reader
gains the live actions and status that users need near the transcript: metadata
sync, transcript sync, comments sync, comments status, and source job activity.

YouTube playlist live source views remain playlist-first. The playlist reader
keeps playlist and per-video sync/retry actions, surfaces comments status, and
shows source job activity for the playlist.

Telegram live single-source views regain the topic/forum selector in `Source`
mode. Existing route state and APIs already load topics and reload source items
for a selected topic; this pass makes that capability visible in the new canvas.

If job rendering becomes duplicated between YouTube video and playlist readers,
create a small `YoutubeSourceActivity` component responsible only for rendering
source jobs and cancel controls. Keep it YouTube-specific for this pass.

## Source Basis Rules

The snapshot/live split from the previous stabilization pass remains binding.

- `live_source` may show sync, comments, playlist retry, per-video sync, job
  activity, cancellation, and Telegram topic filters.
- `run_snapshot` must not show controls that imply live jobs can mutate the
  opened run snapshot.
- A user viewing a live source while a run is open still sees the explicit
  `Live source` basis indicator and can return to the run snapshot when one is
  available.
- Completed-run snapshot views do not fall back to live source data for parity
  features.

## YouTube Video Source Mode

For a single YouTube video source in live `Source` mode:

- `YoutubeTranscriptReader` remains the primary reader.
- The header shows `Sync metadata`, `Sync transcript`, and `Sync comments`.
- The transcript status stays visible through caption state, segment count, and
  last synced time.
- Comments status is visible near the transcript metadata: state label, item
  count, and last synced time.
- Source job activity is visible below the header or below the transcript search
  area, with job type, status, message or error, progress when available,
  warnings when present, and cancel for active jobs.
- Runtime diagnostics such as missing `yt-dlp` continue to appear in the central
  live source canvas before the reader.

Comments are metadata/status and corpus readiness in this pass. The source
canvas does not need to render a full comments timeline.

## YouTube Playlist Source Mode

For a single YouTube playlist source in live `Source` mode:

- `YoutubePlaylistReader` remains the primary reader.
- The header keeps `Sync all` and `Retry failed`.
- Playlist status shows captions, comments, availability, linked count, and
  unavailable count where available.
- Each video row keeps `Open video source`, `Sync this video`, and `Retry this
  video` actions.
- Source job activity is visible for playlist jobs and cancellable active jobs.
- Per-video comments status remains visible through each playlist item's badges.

This pass does not add a separate comments sync button per playlist item unless
the existing playlist-video sync path already syncs comments. The plan should
verify the current route options before deciding whether the per-video sync call
should continue to request only transcripts or also comments.

## Telegram Topic Source Mode

For a live single Telegram source with real forum topics:

- `Source` mode shows a `Topic view` selector near the source reader controls.
- The selector includes `All topics`, loading state, real topics with message
  counts, and the uncategorized bucket when provided by the backend.
- Changing the selector uses the existing `onChangeSelectedTopicKey` route
  callback and reloads `sourceItems` with the selected `topicFilter`.
- The timeline continues to render topic badges on messages.
- If a Telegram source has no real topics, the selector stays hidden.
- Source groups and run snapshots do not gain a topic selector in this pass.

## Component Boundaries

`ReportCanvas` should continue to pass source state through to
`ReportSourceSurface`. The central source surface remains the only owner of
which reader appears for live source versus run snapshot source.

`ReportSourceSurface` should:

- pass YouTube comments sync and source job props to live YouTube readers;
- pass topic selector props to the live Telegram single-source reader path;
- avoid passing live source action props into run snapshot readers;
- keep source group rendering grouped by source.

`YoutubeTranscriptReader` should:

- keep `showSyncActions` as the guard for all live YouTube sync actions;
- accept optional comments sync and job activity props;
- render comments/job controls only when live sync actions are enabled.

`YoutubePlaylistReader` should:

- accept optional source job activity props;
- render job activity only in live mode;
- keep playlist-first reading and per-video actions.

`TelegramTimelineReader` or `ReportSourceSurface` may own the topic selector.
Prefer the smallest change that keeps topic filtering visibly tied to the live
Telegram source reader without duplicating the workspace title.

If a `YoutubeSourceActivity` component is introduced, it should accept only:

- `jobs: SourceJobRecord[]`;
- `formatTimestamp`;
- `onCancelJob`;
- optional labels for empty state and compact layout.

It should not know about transcript segments, playlist items, current runs, or
workspace selection.

## Error And Empty States

- If YouTube detail is still loading, readers can show loading text while keeping
  source actions stable enough to avoid layout jumps.
- If YouTube detail is missing, live sync actions remain available when runtime
  diagnostics do not block source sync.
- If source jobs are empty, show a compact muted empty state or no activity block;
  do not create a large blank panel.
- Failed source jobs show their error and failed status.
- Active source jobs show progress when `progress_current` and `progress_total`
  are present.
- Topic loading failures continue to use the existing route-level status path.

## Testing

Add focused tests before implementation:

- raw-source contract tests proving live YouTube video readers receive and render
  `onSyncComments`, comments status, `sourceJobs`, and `onCancelSourceJob`;
- raw-source contract tests proving live YouTube playlist readers receive and
  render source jobs and cancel controls;
- raw-source contract tests proving run snapshot readers do not receive live
  comments/job controls;
- raw-source contract tests proving Telegram live single-source mode renders a
  topic selector when `showTopicSelector` is true and calls
  `onChangeSelectedTopicKey`;
- route wiring tests proving existing `sourceJobs`, topic state, and callbacks
  flow from `+page.svelte` through `ReportCanvas` into `ReportSourceSurface`;
- Svelte autofixer runs for every changed Svelte component;
- targeted Vitest runs for source readers, report canvas, route wiring, and
  source safety contracts;
- full `npm.cmd run check`, full `npm.cmd test -- --run`, and `git diff --check`
  before implementation completion.

Runtime smoke should inspect `/analysis` in the running Tauri app when available:

- live YouTube video source shows transcript reader, comments sync, comments
  status, and source job activity;
- live YouTube playlist source shows playlist reader, job activity, and playlist
  actions;
- live Telegram forum-capable source shows topic selector and reloads the
  timeline after topic changes;
- run snapshot source mode stays read-only and does not show live source job
  controls.

## Historical Implementation Plan Boundary

This shipped slice was implemented from a completed plan that has since been
removed from the working tree. Git history retains the original plan. It broke
the work into small TDD tasks:

1. YouTube video comments/action parity.
2. YouTube source job activity rendering and cancellation.
3. YouTube playlist job/activity parity.
4. Telegram topic selector in live source mode.
5. Verification and documentation.
