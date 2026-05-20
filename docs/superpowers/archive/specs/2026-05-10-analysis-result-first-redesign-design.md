# Analysis Result-First Redesign Design

## Implementation Status

Status: implemented and merged into `main` on 2026-05-11.

Canonical verification record:
`docs/superpowers/archive/verification/2026-05-10-analysis-redesign.md`.

The staged implementation plans for Parts 1-7 were removed from active docs
after merge because they were execution checklists, not current product
documentation. Git history remains the audit trail for those plans.

## Context

Before this redesign, the `/analysis` route was a dense workspace with three major areas:

- `WorkspaceRail` for sources and source groups;
- `WorkspaceMain` for scope controls, source context, report output, chat, template editing, and source group editing;
- `WorkspaceInspector` for active runs, saved runs, trace refs, and chunk summaries.

That worked functionally, but the opened analysis result competed with setup controls, source browsing, history, trace inspection, and chat. The implemented redesign makes the opened run and source material the primary work surface while keeping source switching, evidence, chat, and saved runs nearby.

## Goals

- Make `/analysis` a result-first research workspace.
- Keep the central report readable and dominant when a run is open.
- Preserve fast access to source context, saved runs, trace evidence, and follow-up chat.
- Add a central `ReportCanvas` mode switch so the same main surface can show either generated report output or source material.
- Show Telegram source material as a TDesktop-like timeline.
- Show YouTube source material as a transcript-oriented reader.
- Reuse existing components where practical instead of replacing the whole route at once.

## Non-Goals

- Do not redesign the global app shell or primary app sidebar.
- Do not implement a full reading-mode drawer system in this redesign.
- Do not move source management, sync, or provider logic into the backend.
- Do not change the saved-run immutability model.
- Do not automatically substitute live source data when a run snapshot is unavailable.

## Approved Direction

The approved layout direction is:

```text
CompactSourceRail | ReportCanvas | RunCompanionTabs
```

The selected interaction model is:

- **Report Primary**: the central canvas is the dominant visual area.
- **Compact Rail**: sources and groups remain visible as a narrow context switcher instead of a full-width rail.
- **Tabbed Companion**: evidence, chat, and runs share one right-side panel.
- **Evidence default**: when a completed run opens, the right companion defaults to `Evidence`.
- **Chat activation**: explicit chat-tab selection or submitting a follow-up question switches the companion to `Chat`.
- **Trace activation**: clicking a trace ref switches the companion to `Evidence`.

## Preparatory Implementation Pass

Before the visible layout redesign, the implementation may start with a small enabling pass that keeps the current `/analysis` UI behavior largely unchanged while creating safer contracts for the larger migration. This pass is part of the approved redesign, not a separate product feature.

Recommended preparatory scope:

- Add a frontend state-contract module for `WorkspaceSelection`, `OpenRunState`, `CanvasMode`, `SourceViewBasis`, `CompanionTab`, and pure transition helpers such as opening a run, switching workspace scope, clearing run-bound state, and normalizing restored workspace state.
- Persist the selected YouTube corpus mode with each analysis run, expose it through backend and frontend run models, and keep existing runs compatible through an explicit default or nullable legacy value.
- Add snapshot-only run source access, either as a paged `list_analysis_run_messages` API or a focused snapshot availability/probe plus first-page API. This path must not use legacy live-source fallback when callers need the frozen run corpus.

This pass should include focused unit tests around state transitions, restored-state normalization, YouTube corpus mode persistence, and snapshot-only availability behavior. It should avoid introducing the new visual layout, companion tabs, compact rail, or source readers until the main implementation plan reaches those phases.

## Workspace And Open Run State Model

The redesign must keep the selected analysis workspace separate from the opened run. These two pieces of state answer different questions:

```ts
type WorkspaceSelection =
  | { kind: "source"; sourceId: number }
  | { kind: "source_group"; sourceGroupId: number }
  | { kind: "none" };

type OpenRunState =
  | { kind: "none" }
  | { kind: "active"; runId: number }
  | { kind: "saved"; runId: number };
```

`OpenRunState.saved` means a persisted non-active run, including completed, failed, and cancelled runs. The run's terminal status still controls `Report` and `Source` behavior.

`WorkspaceSelection` owns:

- the compact rail selection;
- live source browsing;
- the target scope for starting a new report;
- the current-scope filter in `Runs`.

`OpenRunState` owns:

- report output;
- run-bound source snapshot browsing;
- trace evidence;
- follow-up chat;
- run metadata and active-run status.

The MVP redesign should avoid a stable mismatch where the rail shows one source or group while the center shows a report for another source or group. The invariant is:

```text
If OpenRunState is active or saved and the run's live scope exists, WorkspaceSelection matches that run's scope.
```

Opening an active or saved run aligns `WorkspaceSelection` to the run scope, sets `canvasMode = "report"`, and applies the normal companion default for that run state.

Deleted live scope is the exception to this alignment rule. If a saved run's original source or source group no longer exists, the saved run remains openable as an artifact. `Report` mode shows the saved run with a deleted or missing scope label, preferably using `scope_label_snapshot` when available. `Source` mode can show the run snapshot if snapshot data exists. `CompactSourceRail` must not pretend the deleted source or group is an active live workspace selection; it should show a clearly marked missing run context or no live selection for that opened run.

Selecting a different source or source group from `CompactSourceRail` is an explicit workspace switch. It clears `OpenRunState`, clears run-bound evidence/chat/trace selection, sets `canvasMode = "source"`, sets `sourceViewBasis = "live_source"`, and moves the companion to `Runs` so the right panel does not show orphaned run tools.

Switching `Report | Source` does not change `WorkspaceSelection`. It only changes the central canvas view for the current state. Local navigation inside a run snapshot, such as filtering a source group snapshot down to one source, is not a rail selection and must not close the opened run.

During async transitions, the UI can briefly show loading or pending state, but it should not settle into:

```text
Rail: Source B
ReportCanvas: report for Source A
```

If future versions intentionally allow that mismatch, the header must make it explicit, for example `Viewing report for Source A` and `Current workspace Source B`. That is outside this MVP redesign.

## Component Architecture

Introduce three explicit page zones.

### CompactSourceRail

`CompactSourceRail` is the left analysis-level rail. It is separate from the global app sidebar.

Responsibilities:

- show the current source or group identity in compact form;
- show source/group status indicators;
- allow quick source/group switching;
- expose a way to open the full source manager or expanded source list;
- avoid taking full persistent width while a report is open.

Existing `WorkspaceRail` behavior should be reused or extracted where useful, but the new rail should not keep the current full card-heavy source list visible by default.

The rail must not look or behave like a second application menu. The global `AppSidebar` remains the only place for app-level destinations such as Workspace, Accounts, and Settings. `CompactSourceRail` lives inside `/analysis` and should be visually scoped to research context: source avatars, provider/source-type marks, group marks, source status, and an explicit source-switcher affordance. It should not repeat the global brand, route navigation, theme toggle, or app settings actions.

When both rails are visible on desktop, the visual hierarchy should be:

```text
AppSidebar: app navigation
CompactSourceRail: analysis source context
ReportCanvas: primary work surface
RunCompanionTabs: run tools
```

The compact source rail can expose the full source list through a popover, drawer, or expanded temporary panel, but the collapsed/default state should remain clearly source-oriented rather than route-oriented.

### CompactSourceRail Access And Status Details

The compact rail must stay quiet enough that `ReportCanvas` remains visually dominant. It should split source access into two layers instead of making every source action always visible.

Collapsed rail contract:

- a current source/group context button or avatar that opens the source switcher;
- a compact provider/source type mark, such as Telegram, YouTube video, YouTube playlist, or source group;
- selected state through rail item highlight or active ring;
- one contextual primary action slot, used for the most relevant immediate action such as `New source` when no source exists, `Sync`/`Retry` for actionable stale or failed state, or compact running progress while work is active;
- critical warning/running state through a visible dot, spinner, or progress ring.

The collapsed rail should not render separate always-visible controls for source switching, source expansion, source management, new-source creation, detailed provider status, and YouTube transcript/comment availability at the same time. `Manage sources`, `New source`, detailed statuses, sync/takeout actions, and YouTube availability details belong in the expanded layer unless they are the current contextual primary action.

Popover or expanded panel contract:

- full source and source-group list;
- source/group search and filtering;
- manage sources;
- new source;
- detailed statuses such as `Syncing`, `Sync failed`, `Sign in required`, `Transcript unavailable`, or `Takeout import running`;
- provider-specific details, including YouTube transcript/comment availability;
- source sync, retry, and takeout actions when supported.

The full source list should appear as an overlay, popover, or temporary side panel anchored to the compact rail. It should support the source/group search and selection behavior that the old `WorkspaceRail` owned before the redesign. Closing the list without changing selection must return the user to the same `ReportCanvas` mode, scroll position, and companion tab. Selecting a different source or group from the expanded list follows the same workspace-switch rule as selecting directly from `CompactSourceRail`.

Status indicators must have accessible labels. Hover-only information is not sufficient; keyboard users need the same status through titles, aria labels, or the expanded source list.

### ReportCanvas

`ReportCanvas` owns the main center surface. It has two modes:

```text
Report | Source
```

`Report` mode:

- shows the current run header and run metadata;
- shows live output while a run is running;
- shows saved report output when a run is completed;
- represents report setup rather than report output when no run is open;
- keeps trace refs clickable;
- exposes cancel action for a cancellable active run.

When no run is open, the `Report` side of the canvas represents report setup rather than report output. `Source` remains the default mode after a source or group selection, but if the user switches to `Report`, the canvas should show the pre-run setup surface instead of an empty report. The visible mode label can remain `Report` for consistency or become contextual such as `Setup`; either way the state represents preparation for a new report.

Run header metadata should include at least:

- target source or source group label, including missing/deleted labeling when the live scope is gone;
- run status;
- created time and completed time when available;
- prompt template name and version;
- provider profile;
- provider and model;
- source basis status, such as run snapshot available, live source, pending snapshot, or unavailable snapshot;
- YouTube corpus mode when applicable, such as transcript-only, transcript plus comments, playlist scope, or description-derived corpus.

Saved run records persist the selected YouTube corpus mode as durable run metadata. A saved run header should not reconstruct the corpus mode from current source defaults because that can drift from the corpus used for the report.

`Source` mode:

- shows source material in a large readable canvas;
- shows run snapshot material when an opened run has a saved snapshot;
- shows a pending or unavailable state when an opened run has no saved snapshot yet;
- shows live source material only when the user explicitly switches to it;
- becomes the default mode when no run is open and a source/group is selected;
- owns the source-specific action header for the selected live source context.

The existing `ReportViewer`, `SourceMessagesPanel`, `YoutubeSourceDetail`, and `YoutubePlaylistDetail` can be reused internally, but they should be adapted toward the new canvas model rather than rendered as independent stacked panels.

### Report Setup / Template Management

Report setup is a pre-run concern. It should stay available when the user is preparing a new analysis, but it must not compete with reading an opened report.

When no run is open and a source or source group is selected, `ReportCanvas` may show a focused run setup surface near the report/source header or as a compact setup card. This setup surface owns:

- report template picker;
- selected provider/profile/model summary when needed for launch confidence;
- date/topic/scope controls that are required before starting a run;
- primary `Run report` action;
- access to advanced run settings through a collapsed `Run setup` panel or drawer.

Template editing and template creation should be reachable from the pre-run setup surface, preferably as a modal, dialog, or temporary drawer launched from the template picker or setup header. Editing a template should not replace the report reader, source reader, evidence tab, chat tab, or run history tab as a primary workspace mode.

When a completed, failed, cancelled, queued, or running run is opened, report setup controls leave the primary canvas. The opened run can still show the prompt template name/version as metadata, and may expose a secondary `Use this template for new run` or `Open run setup` action, but the default surface remains the report or run status.

`RunCompanionTabs.Runs` can filter by prompt template and display template metadata for saved runs, but it should not become the main template editor. Global app settings remain responsible for LLM profiles, provider keys, and smoke tests; report template editing remains part of analysis run setup unless a later product decision moves all report-template management to `/settings`.

### Source Ingest Controls And Activity

The redesign must preserve source ingest control after the old full-width `WorkspaceRail` is collapsed. Source sync and source-job controls should remain close to the source material, but they must not compete with report reading or mix with analysis runs.

Placement rules:

- `CompactSourceRail` shows only compact ingest status, running, warning, or error indicators, plus the single contextual primary action slot described above.
- `ReportCanvas` `Source` mode header is the primary location for source-specific ingest actions for the selected live source context.
- `Source` mode header actions can include Telegram `Sync source`, Telegram `Takeout import`, YouTube metadata sync, YouTube transcript sync, YouTube comments sync, playlist sync, playlist retry, and per-video playlist sync entry points when relevant.
- A compact `Source activity` area in `Source` mode, or the expanded source panel, should show active and recent source jobs with progress, phase, warnings, errors, retry, and cancel actions.
- The expanded source panel can show detailed source status, source job history, first-sync policy, and secondary ingest actions for sources that are not currently open in the canvas.
- When a report is open, source ingest activity may remain visible as compact status in the rail, but should not interrupt or visually compete with report reading unless it affects the opened run's availability or integrity.
- `RunCompanionTabs.Runs` is only for analysis report runs. Source ingest jobs must not appear there unless the product explicitly renames and redesigns the tab as a broader `Activity` surface.

Snapshot trust rules:

- When `Source` mode is showing `run_snapshot`, ingest controls must not imply that sync, Takeout, or YouTube jobs can change the snapshot behind the opened report.
- In `run_snapshot` mode, live-source ingest actions should be hidden, secondary, or labeled as `Live source actions`.
- Full source ingest controls become primary only when the user is browsing live source context, such as no open run or explicit `View live source`.

Provider-specific rules:

- Telegram first sync policy should be visible before first sync for an unsynced Telegram source, for example `First sync will import the last N messages/days`, with a path to Settings when the user needs to change it.
- After a Telegram sync completes, the applied first-sync policy can remain in the sync result/status message.
- Takeout import progress should show phase, count/progress when available, warnings, terminal error, and cancel state.
- YouTube source activity should show metadata, transcript, comments, playlist, and playlist-video jobs separately enough that users can tell which corpus part is stale or running.
- YouTube transcript/comments/playlist actions should live in `Source` mode near the relevant reader/list, not only in the compact rail.

### RunCompanionTabs

`RunCompanionTabs` is the right-side panel. It owns one visible tool at a time:

```text
Evidence | Chat | Runs
```

`Evidence`:

- shows trace refs and selected evidence;
- becomes active when a trace ref is clicked;
- is the default tab for completed runs;
- exposes `Show in source` when the referenced item, message, or transcript segment can be located.

`Show in source` is the explicit report to evidence to source bridge. It switches `ReportCanvas` to `Source`, uses `sourceViewBasis = "run_snapshot"` when snapshot data is available, and highlights the matching message or transcript segment. If only a live source path is available for an active, failed, or cancelled run state, the UI must label `sourceViewBasis = "live_source"` explicitly and must not present live data as the frozen report corpus. For a completed run with missing snapshot rows, `Show in source` should degrade or become unavailable instead of resolving evidence against live source data; any separate `View live source` action must be clearly labeled as live browsing, not evidence resolution.

`Chat`:

- shows follow-up chat for the opened run;
- becomes active when the user explicitly selects the tab or submits the follow-up question;
- stays disabled or explanatory until chat is available;
- for MVP, is available only for completed runs with usable saved run context.

Chat availability:

- `queued` or `running`: disabled, with an explanation that chat becomes available after completion.
- `completed` with snapshot: enabled against the saved run context.
- `completed` with missing snapshot: disabled or warning-bound; it must not use live source as replacement context.
- `failed` or `cancelled` with snapshot: disabled for MVP, with an explanation that the run is terminal and chat is only available for completed reports.
- `failed` or `cancelled` without snapshot: disabled; it may show saved error or result context only as read-only run information.

When `OpenRunState` is not `none` and `sourceViewBasis = "live_source"`, `Evidence` and `Chat` remain bound to the opened run. The UI must not imply that follow-up chat or evidence refs are based on newly synced live source material.

`Runs`:

- may show queued/running analysis report runs and saved analysis runs;
- replaces the current always-visible inspector role;
- includes search and filtering for saved runs, because run history is expected to grow;
- does not show source ingest jobs, Takeout jobs, or YouTube source jobs.

When `Runs` opens because of a workspace switch, such as selecting a source or source group from `CompactSourceRail`, it defaults to current-scope runs. The user can switch to all runs. When `Runs` opens from a global history entry point, it may default to all runs.

Existing `TracePanel`, `ChatPanel`, `ActiveRunList`, `RunHistory`, and `ChunkSummaries` should be reused inside the tab body where practical.

The chat input should not steal the right companion from a user who is only navigating through the report or source view. Do not switch to `Chat` merely because an input receives incidental focus through tab navigation or restored focus. If the user explicitly clicks the `Chat` tab, uses a dedicated `Ask`/compose action, or submits a question, then `Chat` becomes active.

### Privacy And Provenance

The result-first UI should keep provenance visible without turning every action into a warning modal.

When launching a report, the setup surface and run header should make the selected LLM profile, provider, and model visible. If the profile is missing or unusable, report launch remains disabled as described in onboarding states.

Follow-up chat should make clear that the question is answered against the opened run context, normally the saved snapshot for completed runs, and may be sent to the configured provider. If the user is browsing live source material while a run is open, the chat surface still refers to the opened run context rather than the live source currently visible in `ReportCanvas`.

### Runs Search And Filtering

The `Runs` tab should scale beyond the current short saved-run list.

It should include:

- text search across target label, source title/group name, template name, provider, model, and error text when available;
- status filter, including at least all, completed, failed, cancelled, queued/running;
- scope filter for all runs vs current source/group;
- date range filtering by created or completed date;
- optional filters for provider profile, provider/model, and prompt template when those lists are available;
- clear empty states for no runs, no current-scope runs, and no filter matches.

`RunHistory` already had basic all/current scope and all/completed/failed filters before the redesign. The redesigned `Runs` tab can extend that path with richer filtering. If the list can grow large, prefer backend-backed filtering and cursor/limit pagination over loading all saved runs and filtering only in the frontend.

## Canvas Source Mode

### Active And Terminal Run Source Availability

`Source` mode must distinguish the run's frozen snapshot from live source browsing for every run status.

Current backend behavior persists `analysis_run_messages` near the final persist step after provider work has succeeded. That means an active run usually has no browsable run snapshot until the report is almost complete. The UI must not imply that a future snapshot already exists.

Suggested state:

```ts
type RunSnapshotAvailability =
  | "unknown"
  | "capturing"
  | "available"
  | "unavailable";
```

The final implementation can derive this from an explicit backend field such as `has_run_snapshot`, from a focused snapshot list/probe API, or from the first page of `list_analysis_run_messages`. Do not infer availability from run status alone: a failed or cancelled run can still have a snapshot if failure happened after snapshot persistence. Completed runs are expected to have saved snapshot rows; a completed run without snapshot data should be treated as a source-basis storage or integrity problem, not as a supported legacy source-view path.

For `queued` or `running` runs:

- `Report` remains the default mode.
- `Source` can be selected, but if no snapshot exists yet it shows a clear pending state such as `Source snapshot will be available after the run's corpus snapshot is saved`.
- The pending state may expose an explicit `View live source` action, but this must switch to `sourceViewBasis = "live_source"` and show the persistent `Live source` indicator.
- If a snapshot becomes available while the run is still active, `Source` mode may show `run_snapshot`; if the user is already viewing live source, do not switch automatically. Show an action such as `View run snapshot`.

For `completed` runs:

- `Report` mode should still open and display the completed report when saved report output is available.
- `Source` mode uses `run_snapshot`.
- Missing snapshot rows are a source-basis integrity problem, not necessarily a report-output problem.
- If snapshot rows are missing, the run header should show a clear warning that the frozen source snapshot is missing. The warning should communicate degraded source/evidence availability, not imply that the saved report output itself is unavailable.
- If snapshot rows are missing, `Source` mode shows a storage or integrity error state and does not offer live-source fallback as an equivalent source view. The error should explain that the report can still be read, but Extractum cannot show the exact source material used for that completed run.
- Evidence views may still show saved trace refs when available, but any evidence that requires snapshot resolution should show a degraded or unavailable state rather than silently resolving against live source data.
- Follow-up chat should be unavailable or warning-bound if it requires the missing snapshot context. It must not use live source data as a replacement for the completed run's frozen corpus.

Implementation note: existing backend helpers that load a run corpus may include legacy live-source fallback when snapshot rows are empty. Snapshot-bound UI, evidence resolution, and follow-up chat availability for completed runs must use a snapshot-only path, a dedicated availability probe, or an explicit fallback mode so a completed run with missing snapshot rows cannot silently resolve against live source data.

For `failed` and `cancelled` runs:

- `Source` mode shows `run_snapshot` when snapshot data exists.
- If no snapshot exists, show an unavailable state that explains the run ended before a frozen source snapshot was saved, with an explicit `View live source` action.
- Failed or cancelled report output should remain clearly terminal in `Report` mode; `Source` mode must not make the run look completed.

This keeps edge cases honest without blocking future optimization. If a later backend change persists the snapshot immediately after corpus capture, the UI can surface `Source` earlier through the same availability state without changing the user-facing model.

### Run Snapshot Basis

When an opened run has a saved snapshot, `Source` mode shows the frozen corpus or snapshot behind that run. This preserves the trust contract: the source material visible beside the report is the material the report was based on.

When snapshot material is legitimately unavailable for an active, failed, or cancelled run, the UI must show an explicit unavailable state and offer `View live source`. It must not silently fall back to live source data. Explicit live source browsing is not a replacement for a completed run's missing snapshot.

When the user chooses live data from a run source view, the canvas must show a persistent `Live source` indicator near the `Report | Source` mode control or source header. The indicator should include a clear way back to the run snapshot, such as `Back to run snapshot`, whenever snapshot data exists. This prevents live source data from being mistaken for the frozen material behind the opened report.

Suggested state:

```ts
type CanvasMode = "report" | "source";
type SourceViewBasis = "run_snapshot" | "live_source";
type CompanionTab = "evidence" | "chat" | "runs";
```

The implementation should preserve view state independently for `Report` and `Source` modes. Switching modes should not reset:

- the report scroll position for the opened run;
- the source scroll position for the active source view basis;
- the selected trace ref;
- the selected source item or transcript segment;
- the active companion tab, except for explicit rules such as trace ref activation, explicit chat tab selection, or question submission.

Suggested keys:

```ts
type ReportCanvasViewKey =
  | `report:${number}`
  | `source:snapshot:${number}`
  | `source:live:source:${number}:${string}`
  | `source:live:group:${number}:${string}`;
```

The final key shape can differ, but it should distinguish run report scroll, saved-run snapshot source scroll, live single-source scroll, live source-group scroll, and topic/filter-specific live source scroll. If component remounting is needed, restore scroll after the next render tick rather than treating mode switches as fresh navigation.

### Source Data Loading

`Source` mode must not load an entire large archive into the DOM.

Current live source loading already uses `listSourceItems` with a bounded limit, and the backend clamps source item requests to at most 200 rows with a `beforePublishedAt` cursor. The redesign should preserve that bounded model and extend it for the larger center canvas:

- load an initial page of source items or transcript segments;
- load older/newer pages on explicit action or near-scroll boundary;
- keep DOM size bounded with pagination, incremental rendering, or virtualization;
- keep topic filters and source-group filters compatible with paging;
- avoid refetching already loaded pages when toggling `Report | Source`.

Run snapshot source view must use the frozen `analysis_run_messages` data through paged access. For run snapshots larger than a small threshold, `ReportCanvas` `Source` mode must load an initial page and then request additional pages by cursor, source filter, and limit. The route must not require loading the entire `analysis_run_messages` table for a run into memory just to enter `Source` mode.

If the current frontend API is insufficient for this, add a focused command/API such as `list_analysis_run_messages` with `runId`, optional source filter, optional cursor, and limit. Snapshot availability probes should use a cheap count/existence check or the first page, not full snapshot hydration.

The paged snapshot API should be distinct from any legacy helper that falls back to live source data when `analysis_run_messages` is empty. UI code that needs the frozen corpus must be able to ask for snapshot-only data and receive an explicit empty, pending, or unavailable result.

For YouTube transcript source view, prefer segment-aware loading and rendering. The UI should be able to show a transcript page without loading every comment or playlist item unless the selected source mode requires it.

### Telegram Source View

Telegram `Source` mode should become a TDesktop-like reading timeline:

- chronological message groups;
- clear author and timestamp metadata;
- forum topic badges when available;
- reply/thread hints when available;
- reaction and media metadata where available;
- media placeholders for metadata-only media;
- no binary media preview unless a later media feature explicitly implements it.

Telegram media handling in this redesign is metadata-first only. The timeline should reserve stable slots for future preview, but the current design should render placeholders rather than real binary previews:

- images: image placeholder, media summary, file name when available, mime type when available;
- videos: video placeholder, duration or summary metadata when available, file name/mime type when available;
- documents: document icon, file name, mime type, and summary metadata;
- media-only posts: a clear media-only message body instead of an empty bubble;
- mixed text and media posts: text remains primary, media metadata appears as an attached block.

No automatic media download, thumbnail generation, inline video playback, or document preview is included in this redesign. If future media download support adds local file references, these placeholder slots can become preview slots without changing the overall `Source` mode structure.

### YouTube Source View

YouTube `Source` mode should be transcript-oriented:

- video title and metadata;
- transcript segments with timestamps;
- transcript sync status;
- evidence ref alignment when trace refs point to transcript segments;
- transcript search and timestamp navigation;
- explicit empty state when transcript is unavailable.

Transcript navigation should support:

- clicking a timestamp to open the canonical YouTube URL at that time when available;
- selecting a trace ref and scrolling/highlighting the matching transcript segment when the ref resolves to a timestamped segment;
- copying or exposing the timestamp link from the segment action area;
- text search within loaded transcript segments, with paging or additional loading when the transcript is large.

The redesign does not require an embedded YouTube player or playback synchronization. Extractum remains an analysis workspace; timestamp navigation should help users verify evidence and jump to YouTube when they need the original video context.

For YouTube playlists, `Source` mode shows the playlist item list first. Transcript reading happens at the individual video/source level.

### Source Groups

For source groups, `Source` mode should not merge unrelated messages into one pseudo-chat by default.

It should show grouped corpus or snapshot material by source, with source headings and per-source counts. This keeps mixed Telegram and YouTube groups understandable.

## Workspace State Persistence

Persist enough state to restore the user's analysis workspace context between app sessions, but avoid restoring transient UI moments that can feel surprising.

Persist:

- last `WorkspaceSelection`, meaning the last selected source or source group;
- analysis scope, such as single source vs source group;
- `canvasMode`;
- `companionTab`;
- `sourceViewBasis`, with the same visible `Live source` indicator when it restores to live data;
- durable filter/search preferences for the `Runs` tab when they are useful across sessions;
- lightweight layout preferences that do not open overlays automatically.

Do not persist:

- `OpenRunState`, so the route must not automatically reopen the last run;
- transient selected trace ref;
- partially typed chat question;
- open popovers, drawers, or temporary expanded source panels;
- scroll positions across app restarts unless a later usability pass proves this is helpful.

On app restart, `/analysis` should restore the last selected source or group and its working view, but the user chooses a saved run explicitly from `Runs`. If the persisted source or group no longer exists, fall back to the first available source/group or an empty state without error noise.

Missing or deleted run scope context is run-bound and should not be persisted as `WorkspaceSelection` after restart. If the user had an opened saved run whose original source or group was deleted, restart should not restore that deleted run context as a live workspace selection because `OpenRunState` itself is not persisted.

Because `OpenRunState` is not persisted, restored UI state must be normalized before rendering. If restart restores no opened run, `sourceViewBasis = "run_snapshot"` is invalid and should become `live_source` for the restored workspace selection. Persisted `Evidence` or `Chat` companion tabs are also invalid without an opened run and should restore to `Runs` or another non-run-bound default. A restored `canvasMode = "report"` without an opened run shows report setup, not a stale report.

State keys should be versioned or namespaced so future layout changes can discard incompatible saved UI state without breaking the route.

Implementation note: `/analysis` workspace state is now intentionally modeled
as a small typed state machine. `src/lib/analysis-workspace-state.ts` owns the
`AnalysisWorkspaceUiState`, `AnalysisWorkspaceEvent`, and pure
`transitionAnalysisWorkspaceState(current, event)` function. The route applies
workspace changes through a single `dispatchWorkspaceEvent(event)` boundary and
keeps side effects such as loading runs, loading source pages, persistence, and
legacy scope synchronization outside the pure transition. This gives the
result-first UI FSA-style invariants today while keeping a future migration to
XState or another state-machine library mechanical: the route already sends
events, and the transition module is the only place that computes next state.

## Interaction Rules

- Opening a completed run sets `OpenRunState = { kind: "saved", runId }`, aligns `WorkspaceSelection` to the run scope when that live scope still exists, sets `canvasMode = "report"`, and sets `companionTab = "evidence"`.
- Opening an active run sets `OpenRunState = { kind: "active", runId }`, aligns `WorkspaceSelection` to the run scope, sets `canvasMode = "report"`, and shows live run status in the canvas.
- Opening a saved run whose original source or source group has been deleted keeps the run open, shows the missing/deleted scope label in the run header, and keeps the compact rail from selecting a fallback live source as if it were the run's source.
- Selecting `Source` for an active run shows the run snapshot only if snapshot data exists; otherwise it shows the pending snapshot state and an explicit live source option.
- Selecting `Source` for a failed or cancelled run shows the run snapshot when available, or an unavailable state with an explicit live source option when it is not available.
- Selecting a source or source group from `CompactSourceRail` sets `WorkspaceSelection` to that scope, clears `OpenRunState`, clears run-bound evidence/chat state, sets `canvasMode = "source"`, sets `sourceViewBasis = "live_source"`, sets `companionTab = "runs"`, and defaults `Runs` scope filtering to current scope.
- When no run is open and a source or group is selected, report setup and template selection can appear in the primary canvas as pre-run setup.
- When no run is open and the user switches the canvas to `Report`, show report setup instead of report output.
- Opening any run removes report setup and template editing controls from the primary canvas, leaving template name/version as run metadata and any setup entry point as secondary.
- Clicking a trace ref sets `companionTab = "evidence"`.
- Choosing `Show in source` from an evidence item sets `canvasMode = "source"`, prefers `sourceViewBasis = "run_snapshot"` when available, and highlights the referenced message or transcript segment. For completed runs with missing snapshot rows, it must degrade or explain that exact source resolution is unavailable rather than resolving against live source data.
- Explicitly selecting the chat tab or submitting a follow-up question sets `companionTab = "chat"`.
- Choosing `View live source` in run source mode sets `sourceViewBasis = "live_source"` without pretending the live source is the run snapshot.
- `sourceViewBasis = "live_source"` shows a persistent `Live source` indicator and a return action when run snapshot data is available.
- When `OpenRunState` is not `none` and `sourceViewBasis = "live_source"`, `Evidence` and `Chat` remain bound to the opened run and must not imply newly synced live source context.
- When `OpenRunState` is not `none`, switching back to `Report` from `Source` must not lose the selected run or companion tab state.
- Local filtering or source focus inside a run snapshot does not count as `CompactSourceRail` selection and must not close the opened run.

## Onboarding / No Context States

`/analysis` needs clear first-run and no-context states before a useful report or source canvas exists. These states should live in the central canvas instead of being hidden in the compact rail, because the result-first layout still needs to guide the user when there is no result yet.

- If there are no sources, `ReportCanvas` should offer source creation entry points for Telegram and YouTube. Telegram source creation may point to account setup when needed; YouTube source creation should not be blocked just because no Telegram account exists.
- If there are sources but no selected source or source group, `ReportCanvas` should prompt the user to choose a source/group from `CompactSourceRail` or its expanded source panel.
- If no Telegram accounts exist, Telegram-specific source creation and sync guidance should point to `/accounts`, but the rest of `/analysis` should remain usable for source types that do not require Telegram auth.
- If a Telegram source is selected but its account is disconnected, restoring, or unauthenticated, source sync and report generation for that source should be disabled with sign-in or restore guidance.
- If a YouTube source is selected but `yt-dlp` is unavailable, the canvas should show the runtime diagnostic and a settings/help action instead of relying only on a small rail badge.
- If a selected source is synced but has no text corpus, the canvas should explain the text-first limitation, such as media-only Telegram posts or YouTube content without transcript/comments, and offer the relevant sync or source-detail action when one exists.
- If no LLM profile is configured, or the active profile lacks a usable key/configuration, report generation should be disabled with a link or action to `/settings`. The UI can suggest running the provider smoke test before starting reports.

## Empty And Error States

- If a completed run has no snapshot rows, `Report` mode can still show saved report output when available. `Source` mode shows a storage or integrity error instead of a legacy live-source fallback.
- If saved evidence refs exist for a completed run with missing snapshot rows, `Evidence` can still show the refs, but source resolution should be degraded or unavailable rather than resolved against live source data.
- If follow-up chat depends on the missing snapshot context, `Chat` should be unavailable or warning-bound rather than sent against live source data.
- If a saved run's original source or source group has been deleted, keep the run readable with a missing/deleted scope label. Do not replace it with a fallback live source or group.
- If an active run snapshot is not available yet, show a pending state rather than an empty timeline or transcript.
- If a failed or cancelled run has no snapshot, explain that the run ended before a frozen source snapshot was saved and offer `View live source`.
- If a Telegram source has no synced items, show an empty state with `Sync source`.
- If a YouTube video has no transcript, show transcript status and actions such as `Sync transcript` and `Sync metadata`.
- If a YouTube playlist has no linked videos, show playlist status and sync actions.
- If chat is unavailable because the run is not completed or the completed run context is unusable, the `Chat` tab should explain why and when chat becomes available.
- If evidence refs are empty, the `Evidence` tab should say that no trace refs were captured for this run.

## Responsive Behavior

Desktop:

- use the full three-zone layout;
- keep `CompactSourceRail` narrow;
- keep `ReportCanvas` as the largest column;
- keep `RunCompanionTabs` wide enough for evidence snippets and chat controls.

Medium widths:

- keep the compact rail if possible;
- allow the companion to move below the canvas if width is insufficient.

Mobile:

- stack the canvas and companion;
- expose source switching through a compact top control or drawer;
- keep `Report | Source` mode switching visible near the canvas title.

## Accessibility

- The `Report | Source` control should use buttons or tabs with clear selected state.
- `RunCompanionTabs` should use proper tab semantics.
- Icon-only compact rail items need accessible labels and titles.
- Source timeline items and transcript segments should be keyboard reachable when they can be selected or linked from evidence.
- Trace ref focus should move or announce context without trapping keyboard users.

## Testing

Add focused tests around state and structure:

- opening a completed run defaults to report canvas and evidence companion, and aligns `WorkspaceSelection` to the run scope;
- opening an active run aligns `WorkspaceSelection` to the run scope and shows live run status in the canvas;
- source or source group selection from `CompactSourceRail` clears `OpenRunState`, clears run-bound evidence/chat state, defaults to source canvas, uses live source basis, switches the companion to `Runs`, and defaults `Runs` filtering to current scope;
- no-run source context exposes pre-run report setup with template selection and launch controls;
- no-run report/setup canvas mode shows report setup rather than empty report output;
- opening a run removes template editing and report setup from the primary report-reading surface;
- template editor opens as a modal, dialog, or temporary drawer without replacing report/source reading or companion tabs;
- run header exposes the required minimum metadata: scope label, status, timestamps, template version, provider profile, provider/model, source basis status, and YouTube corpus mode when applicable;
- local source filtering inside a run snapshot does not close the opened run;
- focusing the chat input alone does not unexpectedly switch companion tab to chat;
- explicit chat tab selection or question submission switches companion tab to chat;
- chat availability follows the MVP matrix: enabled for completed runs with usable saved context, disabled/explanatory for queued/running/failed/cancelled runs, and disabled or warning-bound for completed runs with missing snapshot context;
- clicking a trace ref switches companion tab to evidence;
- `Show in source` from evidence switches the canvas to `Source`, prefers run snapshot basis, highlights the referenced message or transcript segment, and labels live source basis explicitly when snapshot basis is unavailable;
- completed-run `Show in source` with missing snapshot rows degrades honestly and does not resolve evidence against live source data;
- runs search and filters narrow saved runs without losing current-scope behavior;
- `Runs` defaults to current-scope filtering after a workspace switch and can default to all runs from a global history entry point;
- `Runs` shows queued/running analysis report runs and saved analysis runs, not source ingest jobs;
- run source mode prefers run snapshot over live source when snapshot data is available;
- completed runs with missing snapshot rows keep saved report output readable when available, while source mode shows a source-basis integrity error without legacy live-source fallback;
- evidence and chat for a completed run with missing snapshot rows degrade honestly when they require snapshot resolution or snapshot context;
- snapshot-bound evidence and chat availability paths do not use legacy live-source fallback for completed runs with missing snapshot rows;
- YouTube corpus mode is persisted with the run and appears in saved run header metadata when applicable;
- saved runs with deleted source or source group remain openable, show missing scope labeling, and do not select a fallback live source in `CompactSourceRail`;
- active-run source mode shows pending snapshot state until snapshot data exists;
- failed and cancelled run source mode shows snapshot when available and unavailable state plus live source action when not;
- source snapshot availability is not inferred from run status alone;
- large run snapshots are entered through paged `analysis_run_messages` access without hydrating the full run snapshot into route state;
- live source does not auto-switch to run snapshot when a running snapshot becomes available;
- live source mode shows an explicit indicator and return path to the run snapshot;
- live source browsing inside an opened run keeps `Evidence` and `Chat` bound to the opened run context, not newly synced live source material;
- Telegram media renders metadata placeholders without requiring binary preview support;
- YouTube transcript timestamp actions expose jump/copy behavior without requiring an embedded player;
- snapshot unavailable state does not silently fall back to live data;
- no-source state shows central Telegram and YouTube add-source onboarding;
- missing Telegram account or disconnected Telegram account disables affected Telegram sync/report actions with `/accounts` guidance;
- missing `yt-dlp` for a YouTube source surfaces the runtime diagnostic in the central canvas;
- synced sources with no text corpus explain the text-first limitation;
- missing or unusable LLM profile disables report generation with `/settings` guidance;
- report launch and follow-up chat make the selected LLM profile/provider/model and run-context provenance visible;
- source ingest jobs are visible in `Source activity` or the expanded source panel, not in `Runs`;
- report mode keeps source ingest activity compact and non-dominant unless it affects the opened run's availability or integrity;
- active Takeout and YouTube source jobs expose progress plus cancel/retry where supported;
- Telegram first-sync policy is visible before first sync for an unsynced Telegram source;
- `run_snapshot` source view does not present live sync/takeout controls as if they change the opened run snapshot;
- YouTube metadata, transcript, comments, playlist, and playlist-video sync actions are reachable from `Source` mode;
- workspace persistence restores last source/group and UI context without auto-opening the last run;
- workspace persistence normalizes run-bound persisted UI state after restart: no restored run snapshot basis, Evidence tab, or Chat tab without an opened run;
- workspace persistence does not save missing/deleted run scope context as `WorkspaceSelection`;
- persistence ignores stale source/group ids gracefully;
- persistence does not restore transient trace selection, draft chat text, or open popovers;
- persisted workspace state does not restore `OpenRunState`;
- run opening from `Runs` updates the rail scope instead of allowing a stable rail/report mismatch;
- `Report | Source` switching is covered for Telegram, YouTube video, YouTube playlist, active run, completed run, failed run, and no-run states;
- collapsed `CompactSourceRail` does not render manage-source, new-source, detailed provider status, and YouTube transcript/comment availability as separate always-visible controls;
- collapsed `CompactSourceRail` keeps critical warning/running state visible and accessible;
- expanded source panel exposes full source management, source search, detailed statuses, provider-specific availability, and supported sync/takeout actions;
- raw-source or component tests confirm the new `CompactSourceRail`, `ReportCanvas`, and `RunCompanionTabs` zones exist.

Browser verification should check desktop and narrow widths for:

- no incoherent text overlap;
- canvas dominance;
- usable right companion tabs;
- compact rail labels or tooltips;
- source mode visibility for Telegram and YouTube states.
