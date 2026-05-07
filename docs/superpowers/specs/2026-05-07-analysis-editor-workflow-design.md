# Analysis Editor Workflow Extraction Design

## Summary

Extract the remaining Analysis template and source group create/update
orchestration from `src/routes/analysis/+page.svelte` into the existing source
groups workflow boundary.

The route should keep local Svelte state, derived selections, UI composition,
and editor input bindings. The workflow should own validation decisions,
Tauri-command orchestration, loading refreshes, selected id updates, editor
rebinding, busy flags, and status/error messages for template and source group
create/update/delete/load actions.

## Goals

- Remove raw `create_analysis_prompt_template`,
  `update_analysis_prompt_template`, `create_analysis_source_group`, and
  `update_analysis_source_group` command usage from
  `src/routes/analysis/+page.svelte`.
- Keep all source group and report-template command access centralized in
  `src/lib/api/analysis-source-groups.ts`.
- Extend `src/lib/analysis-source-groups-workflow.ts` so it owns the async
  create/update flows already described by `analysis-editor-state`.
- Preserve current user-facing behavior, copy, selected template/group fallback
  behavior, and editor field rebinding.
- Add focused unit tests around the extracted workflow methods before changing
  the route.

## Non-Goals

- Do not redesign Analysis UI layout or component APIs.
- Do not extract listener lifecycle in this workstream.
- Do not change backend Tauri commands or Rust DTOs.
- Do not introduce generated Rust-to-TypeScript types.
- Do not remove the existing `analysis-editor-state` helpers; reuse them as
  the command/status decision layer.

## Architecture

Use the existing `analysis-source-groups` boundary as the editor boundary
instead of creating a parallel workflow. That file already coordinates group
loading, template deletion, group deletion, fallback selection, editor binding,
and error formatting. Create/update belongs with the same state surface because
successful saves need the same reload-and-rebind behavior.

`src/lib/api/analysis-source-groups.ts` will become the only frontend module
that invokes these template/group commands:

- `list_analysis_source_groups`
- `list_analysis_prompt_templates`
- `create_analysis_prompt_template`
- `update_analysis_prompt_template`
- `delete_analysis_prompt_template`
- `create_analysis_source_group`
- `update_analysis_source_group`
- `delete_analysis_source_group`

`src/lib/analysis-source-groups-workflow.ts` will expose methods for:

- `loadTemplates`
- `loadGroups`
- `saveTemplateChanges`
- `saveTemplateCopy`
- `deleteTemplate`
- `saveGroupChanges`
- `saveGroupCopy`
- `deleteGroup`

The route will instantiate the workflow with dependencies that read and patch
existing Svelte state. UI components will continue calling route-local wrapper
functions, but those wrappers should delegate to workflow methods.

## State Ownership

Route-owned state remains:

- `templates`
- `groups`
- `selectedTemplateId`
- `selectedGroupId`
- editor fields such as `templateName`, `templateBody`, `groupName`, and
  `groupMemberSourceIds`
- busy flags such as `loadingTemplates`, `loadingGroups`, `savingTemplate`,
  `savingGroup`, `deletingTemplate`, and `deletingGroup`
- `status`

Workflow-owned behavior:

- when a command is valid or rejected;
- which Tauri API function to call;
- when to refresh templates/groups;
- when to update selected ids;
- when to rebind editor fields to the saved/copied/deleted entity;
- how to set busy flags around async work;
- how to format operation-specific errors through the route-provided
  `formatError` dependency.

This keeps Svelte reactivity in the route while moving repeated async
orchestration into testable TypeScript.

## Data Flow

Template save:

1. Route calls `sourceGroupsWorkflow.saveTemplateChanges(name, body)`.
2. Workflow uses `templateUpdateCommand(selectedTemplate, name, body)`.
3. On rejection, workflow patches `status`.
4. On success, workflow sets `savingTemplate`, calls
   `updateAnalysisPromptTemplate`, patches success status, reloads templates,
   selects the updated template id, and rebinds the editor to the updated
   template.

Template copy:

1. Workflow uses `templateCopyCommand(name, body)`.
2. On success, workflow calls `createAnalysisPromptTemplate`, patches success
   status, reloads templates, selects the created template id, and rebinds the
   editor to the created template.

Source group save:

1. Workflow uses `groupUpdateCommand(selectedGroup, name, sourceIds)`.
2. On success, workflow calls `updateAnalysisSourceGroup`, patches success
   status, reloads groups, selects the updated group id, and rebinds the editor
   to the updated group.

Source group copy:

1. Workflow uses `groupCopyCommand(name, sourceIds)`.
2. On success, workflow calls `createAnalysisSourceGroup`, patches success
   status, reloads groups, selects the created group id, and rebinds the editor
   to the created group.

Delete flows keep their existing behavior and remain in the same workflow.

## Error Handling

Validation failures from `analysis-editor-state` should not call Tauri. They
patch the same status messages users see today.

Tauri failures are caught by the workflow and reported with operation-specific
messages:

- `saving the template`
- `creating the template`
- `saving the source group`
- `creating the source group`
- existing delete/load action labels remain unchanged

Busy flags must always reset in `finally` blocks.

## Testing

Extend `src/lib/analysis-source-groups-workflow.test.ts` before route changes.
The tests should cover:

- template update validation rejection does not call API dependencies;
- template update success calls update API, reloads templates, selects the
  updated template, and rebinds editor state;
- template copy success calls create API, reloads templates, selects the
  created template, and rebinds editor state;
- group update validation rejection does not call API dependencies;
- group update success calls update API, reloads groups, selects the updated
  group, and rebinds editor state;
- group copy success calls create API, reloads groups, selects the created
  group, and rebinds editor state;
- API failures patch formatted status and reset busy flags.

Existing `analysis-editor-state` tests should stay focused on pure command and
status decisions. Existing API wrapper tests should be extended to cover the
new create/update/list-template wrappers.

## Implementation Slices

1. Add API wrapper coverage and functions for template list/create/update and
   group create/update.
2. Extend workflow tests and workflow methods for template create/update.
3. Extend workflow tests and workflow methods for source group create/update.
4. Replace route-local raw invoke create/update functions with workflow
   delegates and remove unused imports.
5. Refresh review/session docs after verification.

Each slice should be committed separately, and each implementation turn should
perform exactly one top-level task before stopping for user instruction.
