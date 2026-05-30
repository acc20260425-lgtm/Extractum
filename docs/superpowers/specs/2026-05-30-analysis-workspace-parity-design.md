# Analysis Workspace Parity Design

> Date: 2026-05-30
> Status: active design
> Scope: `/analysis` report canvas workspace tools for setup and opened runs.

## Summary

Make core analysis workspace tools available from the same `ReportCanvas`
surface whether the user is preparing a new report or reviewing an opened run.

The workspace tools are:

- `Export for NotebookLM`;
- `Edit templates`;
- `Edit groups`.

Today those actions exist in two places: setup-only actions inside
`ReportSetupPanel`, and opened-run actions inside `ReportCanvas`. This creates
two ownership paths for the same workspace-level actions. The implementation
should consolidate them into one canvas-level tools area and leave
`ReportSetupPanel` focused on configuring and starting a report.

## Current Context

`ReportCanvas` owns the central report/source canvas. It renders:

- canvas mode tabs: `Report` and `Source`;
- opened-run management tools when `currentRun` exists;
- `ReportSetupPanel` when no run is opened and the report canvas is active;
- `ReportSourceSurface` when the source canvas is active;
- `NotebookLmExportDialog` at canvas level.

`ReportSetupPanel` currently owns setup-specific controls and also owns:

- `Export for NotebookLM` for single-source setup;
- local state for opening `TemplateEditor`;
- local state for opening `SourceGroupEditor`.

`ReportCanvas` separately owns opened-run local state for opening the same
template and group editor drawers. The user-facing behavior is close, but the
contract is split across setup and opened-run paths.

## Goals

- Render workspace-level tools through one canvas-level path in `ReportCanvas`.
- Keep `Edit templates` and `Edit groups` available in setup, opened current
  runs, opened saved runs, report mode, and source mode.
- Keep `Export for NotebookLM` visible as a workspace action when the selected
  analysis scope is eligible or intentionally not yet supported.
- Keep single-source NotebookLM export working exactly as it does today.
- Show a disabled source-group NotebookLM export affordance so the future
  capability is visible without implying it is implemented.
- Remove editor drawer ownership from `ReportSetupPanel`.
- Keep report launch controls inside `ReportSetupPanel`.

## Non-Goals

- Do not implement source-group NotebookLM export in this slice.
- Do not redesign the NotebookLM export dialog or export backend.
- Do not add saved-run snapshot export.
- Do not add command palette, keyboard shortcuts, or a global action registry.
- Do not change report generation, run loading, source browsing, or follow-up
  chat behavior.
- Do not move setup form state out of the route in this slice.

## UX Contract

`ReportCanvas` renders a compact `Workspace tools` area below the canvas
header and `Report` / `Source` mode tabs, and above the setup, report, or
source body. The tools area is shown for both setup and opened-run states.
Editor drawers render immediately below `Workspace tools` so setup and
opened-run states do not get different visual placement.

`Edit templates`:

- is always available;
- opens the existing compact `TemplateEditor` drawer below the tools area;
- uses the existing route-owned template state and callbacks;
- closes only by the same toggle interaction used to open it.

`Edit groups`:

- is always available;
- opens the existing compact `SourceGroupEditor` drawer below the tools area;
- uses the existing route-owned group state and callbacks;
- closes only by the same toggle interaction used to open it.

`Export for NotebookLM`:

- is enabled for a selected single source when the existing export dialog can
  receive a non-null `currentSource`;
- is enabled only when `currentSource` is non-null at the canvas level;
- does not become enabled from opened saved-run metadata alone;
- uses the existing `onOpenNotebookLmExport` callback and dialog state;
- reflects the existing `exportingNotebookLm` pending state;
- is visible but disabled for a selected source group;
- uses a clear disabled reason for source groups:
  `Source-group NotebookLM export is not implemented yet.`

When an opened saved run does not restore a live `currentSource`, such as after
source deletion or missing source context, the export action stays hidden unless
`currentGroup` is present. If `currentGroup` is present, the source-group
disabled affordance is shown instead. When there is no eligible source or
source-group workspace selection, the export button is hidden to avoid inactive
noise.

## Component Design

Add a small canvas-owned component named `ReportWorkspaceTools.svelte`.

Responsibilities:

- render the three workspace-level buttons;
- receive explicit availability state and disabled reasons;
- emit callbacks for opening export, templates, and groups;
- stay presentational and avoid importing API modules.

The component contract should be explicit and route-state agnostic:

```ts
type ReportWorkspaceToolsProps = {
  showNotebookLmExport: boolean;
  canExportNotebookLm: boolean;
  exportDisabledReason: string | null;
  exportingNotebookLm: boolean;
  templateEditorOpen: boolean;
  groupEditorOpen: boolean;
  onOpenNotebookLmExport: () => void;
  onToggleTemplateEditor: () => void;
  onToggleGroupEditor: () => void;
};
```

The component must not inspect `currentSource`, `currentGroup`, `currentRun`,
`workspaceSelection`, or export dialog state directly.

`ReportCanvas` remains the owner of drawer open state:

- `templateEditorOpen`;
- `groupEditorOpen`.

`ReportCanvas` renders the drawers exactly once, using the existing
`TemplateEditor` and `SourceGroupEditor` compact modes. The drawers appear
under the workspace tools area, regardless of whether the canvas is currently
showing setup, an opened report, or source material.

`ReportSetupPanel` no longer imports or renders:

- `TemplateEditor`;
- `SourceGroupEditor`;
- setup secondary action buttons for editor drawers.

Remove only editor drawer state and editor-only callbacks from
`ReportSetupPanel`. Keep setup controls, `Run report`, `Sync source`, and all
report configuration props. `selectedTemplate` remains because setup preflight
copy uses it.

Props and callbacks that should leave `ReportSetupPanel`:

- `templateName`, `templateBody`, `savingTemplate`, `deletingTemplate`;
- `selectedGroupEditorId`, `groups`, `groupName`, `groupSourceType`,
  `groupMemberSourceIds`, `selectedGroup`, `savingGroup`, `deletingGroup`,
  `sourceMetricsList`;
- `isGroupSourceSelected`;
- `onSaveTemplateCopy`, `onSaveTemplateChanges`, `onDeleteTemplate`;
- `onChangeSelectedGroupId`, `onChangeGroupName`, `onChangeGroupSourceType`,
  `onToggleGroupSource`, `onStartNewGroup`, `onSaveGroupCopy`,
  `onSaveGroupChanges`, `onDeleteGroup`.

`ReportSetupPanel` keeps:

- report period controls;
- prompt template selection;
- output language and provider/model controls;
- YouTube corpus mode controls;
- migrated-history inclusion controls;
- `Run report`;
- single-source `Sync source`, because that is setup/context preparation rather
  than a workspace-wide editor.

## Data Flow

The route continues to own all state and callbacks. `ReportCanvas` receives the
same props it receives today.

`ReportCanvas` derives workspace-tool availability from existing props:

- single-source export enabled only when `currentSource` is non-null;
- source-group export disabled when `currentSource` is null and `currentGroup`
  is non-null;
- export hidden when neither a source nor a group is selected;
- template/group tools always enabled.

`NotebookLmExportDialog` stays in `ReportCanvas` and continues receiving
`source={currentSource}`. Because source-group export is disabled, the dialog is
not opened for groups and does not need a new source-group contract.
`NotebookLmExportDialog` remains rendered once at canvas level.

## Disabled And Error States

- While `exportingNotebookLm` is true, the single-source export button is
  disabled and shows the existing exporting label.
- For source groups, the export button is disabled with the source-group
  not-implemented reason.
- The source-group disabled reason is visible as muted helper text in
  `ReportWorkspaceTools` and connected to the disabled export button with
  `aria-describedby`. A `title` attribute may duplicate the same reason, but
  visible text is the testable and accessible source of truth.
- Template and group editor save/delete errors continue to flow through the
  existing route status behavior.
- If no valid workspace selection exists, template and group tools remain
  usable but export is hidden.

## Testing

Frontend contract tests should cover:

- `ReportCanvas` renders workspace tools when `currentRun` is null;
- `ReportCanvas` renders the same workspace tools when `currentRun` exists;
- single-source workspace enables `Export for NotebookLM`;
- source-group workspace renders disabled `Export for NotebookLM` with the
  source-group not-implemented reason;
- clicking the disabled source-group export action does not open
  `NotebookLmExportDialog`;
- `Edit templates` opens the shared canvas-level template drawer in setup and
  opened-run states;
- `Edit groups` opens the shared canvas-level group drawer in setup and
  opened-run states;
- `ReportSetupPanel` no longer imports `TemplateEditor` or
  `SourceGroupEditor`;
- `ReportWorkspaceTools` imports no API modules and does not call Tauri
  `invoke`;
- `NotebookLmExportDialog` remains rendered once at `ReportCanvas` level;
- `ReportSetupPanel` still renders `Run report` and setup controls;
- `ReportSetupPanel` still exposes `Sync source` for single-source setup.

Manual smoke should cover:

- single-source setup: export opens, template editor opens, group editor opens;
- source-group setup: export is visible but disabled, template/group editors
  open;
- opened single-source run: export opens, template/group editors open;
- opened source-group run: export is visible but disabled, template/group
  editors open;
- source canvas mode for the same selections keeps the same workspace tools.

## Rollout Notes

This slice deliberately makes source-group NotebookLM export visible as an
unsupported capability. A later export-profile or NotebookLM follow-up can
replace the disabled affordance with a real source-group export contract
without moving the tool again.

After this slice ships, active docs should summarize the parity behavior in
current-state documentation, and this spec should move to
`docs/superpowers/archive/specs/`.
