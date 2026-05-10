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
- Do not automatically substitute live source data when a saved run snapshot is unavailable.

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
- **Chat activation**: focusing or submitting the follow-up question switches the companion to `Chat`.
- **Trace activation**: clicking a trace ref switches the companion to `Evidence`.

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
- shows saved-run snapshot material when a saved run is open;
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
- becomes active when the user focuses or submits the follow-up question;
- stays disabled or explanatory until chat is available.

`Runs`:

- combines active runs and saved run history entry points;
- replaces the current always-visible inspector role;
- can keep filter controls for saved run status and scope.

Existing `TracePanel`, `ChatPanel`, `ActiveRunList`, `RunHistory`, and `ChunkSummaries` should be reused inside the tab body where practical.

## Canvas Source Mode

### Saved Run Basis

When a saved or completed run is open, `Source` mode shows the frozen corpus or snapshot behind that run. This preserves the trust contract: the source material visible beside the report is the material the report was based on.

If snapshot material is unavailable, the UI must show an explicit unavailable state and offer `View live source`. It must not silently fall back to live source data.

Suggested state:

```ts
type CanvasMode = "report" | "source";
type SourceViewBasis = "run_snapshot" | "live_source";
type CompanionTab = "evidence" | "chat" | "runs";
```

### Telegram Source View

Telegram `Source` mode should become a TDesktop-like reading timeline:

- chronological message groups;
- clear author and timestamp metadata;
- forum topic badges when available;
- reply/thread hints when available;
- reaction and media metadata where available;
- media placeholders for metadata-only media;
- no binary media preview unless a later media feature explicitly implements it.

### YouTube Source View

YouTube `Source` mode should be transcript-oriented:

- video title and metadata;
- transcript segments with timestamps;
- transcript sync status;
- evidence ref alignment when trace refs point to transcript segments;
- explicit empty state when transcript is unavailable.

For YouTube playlists, `Source` mode shows the playlist item list first. Transcript reading happens at the individual video/source level.

### Source Groups

For source groups, `Source` mode should not merge unrelated messages into one pseudo-chat by default.

It should show grouped corpus or snapshot material by source, with source headings and per-source counts. This keeps mixed Telegram and YouTube groups understandable.

## Interaction Rules

- Opening a completed run sets `canvasMode = "report"` and `companionTab = "evidence"`.
- Opening an active run sets `canvasMode = "report"` and shows live run status in the canvas.
- Selecting a source without an open run sets `canvasMode = "source"` and `sourceViewBasis = "live_source"`.
- Clicking a trace ref sets `companionTab = "evidence"`.
- Focusing or submitting a follow-up question sets `companionTab = "chat"`.
- Choosing `View live source` in saved-run source mode sets `sourceViewBasis = "live_source"` without pretending the live source is the run snapshot.
- Switching back to `Report` from `Source` must not lose the selected run or companion tab state.

## Empty And Error States

- If a saved run snapshot is unavailable, show a clear unavailable state and a `View live source` action.
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

- opening a completed run defaults to report canvas and evidence companion;
- source selection without an open run defaults to source canvas;
- focusing chat switches companion tab to chat;
- clicking a trace ref switches companion tab to evidence;
- saved-run source mode prefers run snapshot over live source;
- snapshot unavailable state does not silently fall back to live data;
- raw-source or component tests confirm the new `CompactSourceRail`, `ReportCanvas`, and `RunCompanionTabs` zones exist.

Browser verification should check desktop and narrow widths for:

- no incoherent text overlap;
- canvas dominance;
- usable right companion tabs;
- compact rail labels or tooltips;
- source mode visibility for Telegram and YouTube states.
