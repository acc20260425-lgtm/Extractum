# Analysis Source Groups Cleanup Design

## Purpose

Reduce the remaining raw Tauri command and route orchestration surface in
`src/routes/analysis/+page.svelte` for the next small `/analysis` cleanup slice.
This workstream focuses on analysis source group loading and destructive
template/group deletion flows.

## Scope

In scope:

- replace route-local calls to `list_analysis_source_groups`;
- replace route-local calls to `delete_analysis_prompt_template`;
- replace route-local calls to `delete_analysis_source_group`;
- move the related loading, deleting, fallback selection, editor binding, and
  error-status transitions into framework-independent workflow code;
- add focused tests for the API wrapper and workflow behavior;
- update the review and session handoff documents after implementation.

Out of scope:

- `create_analysis_source_group` and `update_analysis_source_group`;
- `create_analysis_prompt_template`, `update_analysis_prompt_template`, and
  `list_analysis_prompt_templates`;
- report start, cancel, and delete actions;
- backend Rust command changes;
- broad UI restructuring of the Analysis page.

## Architecture

Add `src/lib/api/analysis-source-groups.ts` as the typed frontend boundary for
the compact command surface:

- `listAnalysisSourceGroups()`;
- `deleteAnalysisPromptTemplate(templateId: number)`;
- `deleteAnalysisSourceGroup(groupId: number)`.

Add `src/lib/analysis-source-groups-workflow.ts` as the orchestration boundary.
The workflow receives dependencies for API calls, state access, patching route
state, fallback helpers, editor binding, confirmation, status formatting, and
status text generation. It exposes:

- `loadGroups()`;
- `deleteTemplate()`;
- `deleteGroup()`.

The Svelte route remains responsible for wiring `$state`, derived values,
modal implementation, and UI composition. It should call workflow methods from
the existing event handlers instead of invoking these Tauri commands directly.

## Data Flow

`loadGroups()` sets `loadingGroups`, loads groups through the wrapper, patches
the group list, selects the first group only when no group is selected, and
binds the editor to the selected group when needed. On failure it patches a
formatted status and always clears `loadingGroups`.

`deleteTemplate()` asks route-independent editor-state helpers whether deletion
is valid. If valid, it asks the injected confirmation function. Confirmed
deletion sets `deletingTemplate`, calls the wrapper, patches the success status,
reloads templates through an injected `loadTemplates` dependency, applies
`templateFallbackSelection`, and binds the editor to the fallback template. It
does not own template list loading because template listing remains outside this
slice.

`deleteGroup()` mirrors the template deletion flow: validate through
`groupDeleteDecision`, confirm, set `deletingGroup`, call the wrapper, patch the
success status, reload groups, apply `groupFallbackSelection`, and bind the
editor to the fallback group.

## Error Handling

Workflow methods catch unknown errors and convert them with the injected
`formatError(action, error)` dependency, matching existing workflow patterns.
Confirmation cancellation exits without patching deleting flags or status.
Deletion validation failures patch the validation status and do not show a
confirmation modal.

## Testing

Add API wrapper tests that mock `@tauri-apps/api/core` and assert exact command
names and argument shapes.

Add workflow tests covering:

- group loading selects the first group when no group is selected;
- group loading preserves a current selection and binds the matching editor
  record when needed;
- group loading reports errors and clears loading state;
- template deletion exits on validation failure;
- template deletion exits on confirmation cancellation;
- confirmed template deletion calls the API, reloads templates, applies fallback
  selection, binds the fallback editor record, and clears deleting state;
- confirmed group deletion calls the API, reloads groups, applies fallback
  selection, binds the fallback editor record, and clears deleting state;
- deletion API failures patch formatted status and clear deleting state.

Run focused tests after each implementation task, then run full `npm.cmd test`,
`npm.cmd run check`, and `git diff --check` before declaring the workstream
complete.

## Completion Criteria

- `src/routes/analysis/+page.svelte` no longer contains raw
  `list_analysis_source_groups`, `delete_analysis_prompt_template`, or
  `delete_analysis_source_group` command strings.
- Focused API and workflow tests pass.
- Full frontend verification passes.
- `docs/code-review-results-2026-05-03.md` and
  `docs/session-context-2026-05-03.md` reflect the completed cleanup slice.
