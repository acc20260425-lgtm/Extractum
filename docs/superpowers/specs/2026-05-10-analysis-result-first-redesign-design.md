# Analysis Result-First Redesign Design

## Context

The `/analysis` route is currently a dense workspace with three major areas:

- `WorkspaceRail` for sources and source groups;
- `WorkspaceMain` for scope controls, source context, report output, chat, template editing, and source group editing;
- `WorkspaceInspector` for active runs, saved runs, trace refs, and chunk summaries.

This works functionally, but the opened analysis result competes with setup controls, source browsing, history, trace inspection, and chat. The requested redesign makes the opened run and source material the primary work surface while keeping source switching, evidence, chat, and saved runs nearby.

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
If OpenRunState is active or saved, WorkspaceSelection matches that run's scope.
```

Opening an active or saved run aligns `WorkspaceSelection` to the run scope, sets `canvasMode = "report"`, and applies the normal companion default for that run state.

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

The full source list should appear as an overlay, popover, or temporary side panel anchored to the compact rail. It should support the same source/group search and selection behavior currently owned by `WorkspaceRail`. Closing the list without changing selection must return the user to the same `ReportCanvas` mode, scroll position, and companion tab. Selecting a different source or group from the expanded list follows the same workspace-switch rule as selecting directly from `CompactSourceRail`.

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
- keeps trace refs clickable;
- exposes cancel action for a cancellable active run.

`Source` mode:

- shows source material in a large readable canvas;
- shows run snapshot material when an opened run has a saved snapshot;
- shows a pending or unavailable state when an opened run has no saved snapshot yet;
- shows live source material only when the user explicitly switches to it;
- becomes the default mode when no run is open and a source/group is selected.

The existing `ReportViewer`, `SourceMessagesPanel`, `YoutubeSourceDetail`, and `YoutubePlaylistDetail` can be reused internally, but they should be adapted toward the new canvas model rather than rendered as independent stacked panels.

### RunCompanionTabs

`RunCompanionTabs` is the right-side panel. It owns one visible tool at a time:

```text
Evidence | Chat | Runs
```

`Evidence`:

- shows trace refs and selected evidence;
- becomes active when a trace ref is clicked;
- is the default tab for completed runs.

`Chat`:

- shows follow-up chat for the opened run;
- becomes active when the user explicitly selects the tab or submits the follow-up question;
- stays disabled or explanatory until chat is available.

`Runs`:

- combines active runs and saved run history entry points;
- replaces the current always-visible inspector role;
- includes search and filtering for saved runs, because run history is expected to grow.

Existing `TracePanel`, `ChatPanel`, `ActiveRunList`, `RunHistory`, and `ChunkSummaries` should be reused inside the tab body where practical.

The chat input should not steal the right companion from a user who is only navigating through the report or source view. Do not switch to `Chat` merely because an input receives incidental focus through tab navigation or restored focus. If the user explicitly clicks the `Chat` tab, uses a dedicated `Ask`/compose action, or submits a question, then `Chat` becomes active.

### Runs Search And Filtering

The `Runs` tab should scale beyond the current short saved-run list.

It should include:

- text search across target label, source title/group name, template name, provider, model, and error text when available;
- status filter, including at least all, completed, failed, cancelled, queued/running;
- scope filter for all runs vs current source/group;
- date range filtering by created or completed date;
- optional filters for provider profile, provider/model, and prompt template when those lists are available;
- clear empty states for no runs, no current-scope runs, and no filter matches.

Current `RunHistory` already has basic all/current scope and all/completed/failed filters, and `list_analysis_runs` currently accepts only source id, source group id, and limit. The redesign should treat richer filtering as part of the `Runs` tab work. If the list can grow large, prefer backend-backed filtering and cursor/limit pagination over loading all saved runs and filtering only in the frontend.

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

The final implementation can derive this from an explicit backend field such as `has_run_snapshot`, from a focused snapshot list/probe API, or from the first page of `list_analysis_run_messages`. Do not infer availability from run status alone: a failed or cancelled run can still have a snapshot if failure happened after snapshot persistence. Completed runs are expected to have saved snapshot rows; a completed run without snapshot data should be treated as a storage or integrity problem, not as a supported legacy source-view path.

For `queued` or `running` runs:

- `Report` remains the default mode.
- `Source` can be selected, but if no snapshot exists yet it shows a clear pending state such as `Source snapshot will be available after the run's corpus snapshot is saved`.
- The pending state may expose an explicit `View live source` action, but this must switch to `sourceViewBasis = "live_source"` and show the persistent `Live source` indicator.
- If a snapshot becomes available while the run is still active, `Source` mode may show `run_snapshot`; if the user is already viewing live source, do not switch automatically. Show an action such as `View run snapshot`.

For `completed` runs:

- `Source` mode uses `run_snapshot`.
- If snapshot rows are missing, show a storage or integrity error state and do not offer live-source fallback as an equivalent source view.

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
  | `source:live:${number}:${string}`;
```

The final key shape can differ, but it should distinguish run report scroll, saved-run snapshot source scroll, and live source/topic scroll. If component remounting is needed, restore scroll after the next render tick rather than treating mode switches as fresh navigation.

### Source Data Loading

`Source` mode must not load an entire large archive into the DOM.

Current live source loading already uses `listSourceItems` with a bounded limit, and the backend clamps source item requests to at most 200 rows with a `beforePublishedAt` cursor. The redesign should preserve that bounded model and extend it for the larger center canvas:

- load an initial page of source items or transcript segments;
- load older/newer pages on explicit action or near-scroll boundary;
- keep DOM size bounded with pagination, incremental rendering, or virtualization;
- keep topic filters and source-group filters compatible with paging;
- avoid refetching already loaded pages when toggling `Report | Source`.

Run snapshot source view should use the frozen `analysis_run_messages` data. If the current frontend API is insufficient for paged snapshot browsing, add a focused command/API such as `list_analysis_run_messages` with `runId`, optional source filter, optional cursor, and limit. Do not force the whole snapshot into route state just to render source mode.

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

State keys should be versioned or namespaced so future layout changes can discard incompatible saved UI state without breaking the route.

## Interaction Rules

- Opening a completed run sets `OpenRunState = { kind: "saved", runId }`, aligns `WorkspaceSelection` to the run scope, sets `canvasMode = "report"`, and sets `companionTab = "evidence"`.
- Opening an active run sets `OpenRunState = { kind: "active", runId }`, aligns `WorkspaceSelection` to the run scope, sets `canvasMode = "report"`, and shows live run status in the canvas.
- Selecting `Source` for an active run shows the run snapshot only if snapshot data exists; otherwise it shows the pending snapshot state and an explicit live source option.
- Selecting `Source` for a failed or cancelled run shows the run snapshot when available, or an unavailable state with an explicit live source option when it is not available.
- Selecting a source or source group from `CompactSourceRail` sets `WorkspaceSelection` to that scope, clears `OpenRunState`, clears run-bound evidence/chat state, sets `canvasMode = "source"`, sets `sourceViewBasis = "live_source"`, and sets `companionTab = "runs"`.
- Clicking a trace ref sets `companionTab = "evidence"`.
- Explicitly selecting the chat tab or submitting a follow-up question sets `companionTab = "chat"`.
- Choosing `View live source` in run source mode sets `sourceViewBasis = "live_source"` without pretending the live source is the run snapshot.
- `sourceViewBasis = "live_source"` shows a persistent `Live source` indicator and a return action when run snapshot data is available.
- When `OpenRunState` is not `none`, switching back to `Report` from `Source` must not lose the selected run or companion tab state.
- Local filtering or source focus inside a run snapshot does not count as `CompactSourceRail` selection and must not close the opened run.

## Empty And Error States

- If a completed run has no snapshot rows, show a storage or integrity error instead of a legacy live-source fallback.
- If an active run snapshot is not available yet, show a pending state rather than an empty timeline or transcript.
- If a failed or cancelled run has no snapshot, explain that the run ended before a frozen source snapshot was saved and offer `View live source`.
- If a Telegram source has no synced items, show an empty state with `Sync source`.
- If a YouTube video has no transcript, show transcript status and actions such as `Sync transcript` and `Sync metadata`.
- If a YouTube playlist has no linked videos, show playlist status and sync actions.
- If chat is unavailable because the run is not completed, the `Chat` tab should explain when it becomes available.
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
- source or source group selection from `CompactSourceRail` clears `OpenRunState`, clears run-bound evidence/chat state, defaults to source canvas, uses live source basis, and switches the companion to `Runs`;
- local source filtering inside a run snapshot does not close the opened run;
- focusing the chat input alone does not unexpectedly switch companion tab to chat;
- explicit chat tab selection or question submission switches companion tab to chat;
- clicking a trace ref switches companion tab to evidence;
- runs search and filters narrow saved runs without losing current-scope behavior;
- run source mode prefers run snapshot over live source when snapshot data is available;
- completed run source mode requires snapshot data and does not expose a legacy live-source fallback;
- active-run source mode shows pending snapshot state until snapshot data exists;
- failed and cancelled run source mode shows snapshot when available and unavailable state plus live source action when not;
- source snapshot availability is not inferred from run status alone;
- live source does not auto-switch to run snapshot when a running snapshot becomes available;
- live source mode shows an explicit indicator and return path to the run snapshot;
- Telegram media renders metadata placeholders without requiring binary preview support;
- YouTube transcript timestamp actions expose jump/copy behavior without requiring an embedded player;
- snapshot unavailable state does not silently fall back to live data;
- workspace persistence restores last source/group and UI context without auto-opening the last run;
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
