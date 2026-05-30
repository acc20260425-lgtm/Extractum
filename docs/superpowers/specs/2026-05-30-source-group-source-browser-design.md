# Source Group Source Browser Design

> Date: 2026-05-30
> Status: approved design, pending implementation plan
> Scope: live source group browsing in the `/analysis` Source canvas.

## Summary

Move live source group browsing into the shared `SourceBrowserShell` without
changing saved run snapshot behavior. Source groups should use the same browser
model as live single sources while keeping group-specific membership browsing as
the primary view.

This slice migrates only live source groups. Saved run snapshots and saved group
snapshots remain on their existing snapshot readers.

## Current Context

The shipped live Source Browser supports single live sources:

- Telegram: `Timeline | Items | Metadata | Activity`;
- YouTube video: `Transcript | Comments | Items | Metadata | Activity`;
- YouTube playlist: `Videos | Items | Metadata | Activity`.

Live source groups still bypass `SourceBrowserShell` and render
`SourceGroupReader` directly from `ReportSourceSurface`. The route owns group
member data, loaded item windows, per-source cursors, selected member source,
and paging callbacks.

`SourceGroupReader` already has the right group leaf behavior: it groups loaded
rows by member source, renders Telegram rows with `TelegramTimelineReader`,
renders YouTube transcript rows with `YoutubeTranscriptReader`, and supports
per-source load-more callbacks.

## Goals

- Route live source groups into `SourceBrowserShell`.
- Add a group-only canonical tab id `sources`.
- Use live source group tabs: `Sources | Items | Metadata | Activity`.
- Make `Sources` the smart default for live source groups.
- Keep `SourceGroupReader` or an equivalent group leaf as the `Sources` tab
  body.
- Keep `/analysis` as the owner of group data, loaded windows, paging callbacks,
  source selection, and evidence navigation.
- Add group-aware metadata and loaded-window item browsing inside the shared
  shell.
- Keep source group Activity lightweight until there is an explicit
  group-scoped activity or job contract.

## Non-Goals

- Do not migrate saved run snapshots or saved group snapshots into
  `SourceBrowserShell` in this slice.
- Do not add a group-scoped job model.
- Do not show source-scoped job CTAs as if they were group-scoped activity.
- Do not add backend-global search/filter/sort across every item in a group.
- Do not change group membership, report corpus, or snapshot persistence
  semantics.
- Do not create a second shell-like wrapper for groups.

## Subject Model

The browser model should become subject-aware:

```ts
export type SourceBrowserSubject =
  | { kind: "source"; source: Source }
  | { kind: "source_group"; group: AnalysisSourceGroup };
```

New subject-aware functions are the primary contract:

```ts
sourceBrowserTabsForSubject(subject)
smartDefaultSourceBrowserTab(subject)
reconcileSourceBrowserTab(previousTab, nextSubject)
sourceBrowserShellAppliesToSubject(subject)
```

Existing source-only helpers should remain as compatibility wrappers:

```ts
sourceBrowserTabsForSource(source)
smartDefaultSourceBrowserTab(source)
sourceBrowserShellAppliesToSource(source)
```

The wrappers should delegate to the subject-aware model instead of carrying a
parallel source-only tab implementation.

## UX Contract

Live source groups use:

```text
Sources | Items | Metadata | Activity
```

`Sources` is the smart default. It is the group-aware primary view and displays
loaded rows grouped by member source. Each member section keeps the existing
per-source pagination behavior.

`Items` is a combined loaded-window view across rows that are already loaded for
the live group. It is not a backend-global search over the entire group. Empty
and help copy should say:

```text
Group items are limited to the source rows loaded in this browser session. Use Sources to load more rows for each member source.
```

Group `Items` must preserve member source attribution for each row, either
through an existing source title field or through an optional member-source
label derived from route-owned group metadata. If `UniversalItemsView` needs a
new display hook for this, add it as optional group-mode rendering rather than
changing the semantics of single-source item browsing.

`Metadata` shows group-level structure from route-owned data:

- group name;
- group provider/source type;
- member count;
- member list with loaded item counts from existing group member metadata;
- `created_at` and `updated_at` when they are already exposed on the frontend
  group DTO; otherwise show the available group/member fields without backend
  or DTO expansion in this slice.

`Activity` is intentionally lightweight for this slice. It may show a muted
group-level status or empty state, but it must not render source-scoped job CTAs
or detailed source job cards. `SourceActivityView` must not render for source
groups until there is an explicit group-scoped activity/job contract.

## Tab Reconciliation

Shared tabs preserve across subject changes:

| From | To | Active before | Expected after |
| --- | --- | ---: | ---: |
| source | source_group | `items` | `items` |
| source | source_group | `metadata` | `metadata` |
| source | source_group | `activity` | `activity` |
| source | source_group | `timeline` | `sources` |
| source | source_group | `transcript` | `sources` |
| source | source_group | `comments` | `sources` |
| source | source_group | `videos` | `sources` |
| source_group | Telegram source | `sources` | `timeline` |
| source_group | YouTube video source | `sources` | `transcript` |
| source_group | YouTube playlist source | `sources` | `videos` |
| source_group | source | `items` | `items` |
| source_group | source | `metadata` | `metadata` |
| source_group | source | `activity` | `activity` |

Unsupported tab ids always fall back to the target subject's smart default.

## Component Responsibilities

`SourceBrowserShell` supports live browser subjects:

- `{ kind: "source"; source }` for existing live single-source browsing;
- `{ kind: "source_group"; group }` for live source groups.

The shell owns only local tab state and tab reconciliation. It receives
route-owned data and callbacks through props. It does not import `$lib/api/*`
and does not call `invoke`.

The group `Sources` leaf reuses the existing grouped reader behavior. It does
not own tab state, route selection, source data loading, or evidence navigation.

`ReportSourceSurface` routes live source groups through `SourceBrowserShell`.
Saved run snapshots and saved group snapshots do not enter `SourceBrowserShell`
in this slice.

## Data Flow

1. The user selects a live source group in `/analysis`.
2. The route keeps the current group, member list, selected group member source,
   per-source loaded item windows, cursors, loading flags, and has-more flags.
3. `ReportSourceSurface` passes a `source_group` browser subject and group props
   into `SourceBrowserShell`.
4. `SourceBrowserShell` derives group tabs and opens `Sources`.
5. `Sources` renders grouped loaded rows and calls route-owned paging callbacks.
6. `Items` renders the already-loaded group item window across member sources
   and preserves source attribution on each row.
7. `Metadata` renders route-owned group/member fields.
8. `Activity` renders only the group-level lightweight empty/status state.

## Error And Empty States

- No group selected: keep the existing source material empty state.
- No rows loaded for a live group: show a compact empty state in `Sources`.
- No rows loaded for `Items`: show the loaded-window explanation from the UX
  contract.
- Per-source paging errors remain route-owned and continue to surface through
  the route status channel used today.
- Activity without a group-scoped job contract shows a muted "No group activity
  is available for this source group" state.

## Testing

Frontend contract and model tests should assert:

- `sourceBrowserTabsForSubject({ kind: "source_group", group })` returns
  `sources`, `items`, `metadata`, `activity`;
- `smartDefaultSourceBrowserTab(groupSubject)` returns `sources`;
- source-only wrappers still return the existing source tab sets;
- reconciliation follows the table above;
- `sourceBrowserShellAppliesToSubject(groupSubject)` is true;
- live source groups route into `SourceBrowserShell`;
- saved run snapshots and saved group snapshots do not enter
  `SourceBrowserShell` in this slice;
- `SourceBrowserShell` renders the group `Sources` leaf for `sources`;
- the group leaf and shell import no `$lib/api/*` modules and call no `invoke`;
- group `Items` uses loaded-window copy and does not claim global group search;
- group `Items` preserves member source attribution for each row;
- group `Activity` does not render `SourceActivityView`;
- existing Telegram, YouTube video, and YouTube playlist live source browser
  contracts still pass.

Manual smoke should verify:

- selecting a fixture source group opens the Source Browser on `Sources`;
- `Sources` shows member sections and per-source load-more controls;
- `Items` shows only loaded group rows, its loaded-window copy, and source
  attribution for each row;
- `Metadata` shows group name, type, member count, and member list;
- `Activity` shows no source-job CTAs or detailed source job cards;
- saved run snapshots still render through the existing snapshot path.

## Rollout Notes

This slice prepares the browser model for live source and live source group
subjects without changing frozen snapshot semantics. A later snapshot-specific
slice can decide whether frozen run material should get its own browser subject
or remain on specialized snapshot readers.
