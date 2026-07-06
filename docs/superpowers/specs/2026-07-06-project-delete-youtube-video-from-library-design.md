# Project YouTube Video Delete From Library Design

Status: ready for user review
Date: 2026-07-06

## Objective

Add a project Sources action that deletes one selected YouTube video source from the current project and from Library in one safe operation.

The action is intentionally narrower than the existing `Remove` action:

- `Remove` deletes only the project-source membership.
- `Delete from Library` deletes the project-source membership and the Library source record, including the source's stored materials such as transcript, comments, and analysis documents.

## Reviewed Code Reality

- The project Sources screen already has selected-source toolbar actions in `SourcesTab.svelte`.
- Existing frontend workflow has `removeProjectSources`, which removes rows from `project_sources` only.
- Existing frontend API has `deleteSource(sourceId)`, backed by the Tauri command `delete_source`.
- `delete_source` already refuses to delete a Library source while it is referenced by any project, but the current error reports only a count and does not list project names.
- `delete_source_from_pool` currently deletes only the `sources` row. Cleanup of transcripts, comments, source-specific YouTube metadata, analysis documents, archive read rows, prompt-pack snapshots, and other dependent rows relies on SQLite foreign-key actions such as `ON DELETE CASCADE`.
- SQLite enforces those foreign-key actions only when `PRAGMA foreign_keys = ON` is enabled on the active connection. The new project-scoped delete path must not assume that the pooled connection already has this enabled.
- `project_sources.source_id` is `ON DELETE RESTRICT`, so the current project membership must be deleted before the source row. Other project memberships must block the operation.
- `youtube_playlist_items.video_source_id` uses `ON DELETE SET NULL`. A YouTube video source may have been materialized from a playlist; deleting the video source is allowed, and linked playlist item rows should remain as playlist snapshots with `video_source_id = NULL`.
- Calling `removeProjectSources` first and `deleteSource` second is unsafe for this feature: if the source is linked to another project, the current project membership would already be gone while Library deletion fails.
- Project archive state does not remove `project_sources` membership. Archived projects still count as usage and must block deletion.

## UX Contract

The selected-source toolbar shows a `Delete from Library` button near `Remove`.

The button is always visible, but it is enabled only when:

- exactly one source is selected;
- the selected source provider is `youtube`;
- the selected source subtype is `video`;
- the project is selected and the page is not already saving.

Disabled states use a concise `title`/tooltip:

- `Select one YouTube video source`
- `Only YouTube videos can be deleted from Library here`
- existing saving/project-unavailable wording where appropriate.

On click, the UI asks for confirmation before any deletion:

```text
Delete this YouTube video from the project and Library? The app will cancel the deletion if another project still uses it. This will remove its transcript, comments, and stored materials.
```

If the user cancels, no command is called.

The confirm appears before the authoritative backend check. This is intentional: adding a separate pre-check would add another round-trip and still require the backend check to avoid a time-of-check/time-of-use race.

If deletion succeeds, the UI shows:

```text
Source deleted from project and Library.
```

Then it refreshes both:

- the current project sources/state;
- the Library catalog, so `Connect from Library` and other panels no longer show the deleted source.

## Blocking Behavior

If the source is used by any project other than the current project, the operation is fully cancelled.

No partial deletion is allowed:

- do not remove the source from the current project;
- do not delete the Library source;
- do not mutate source materials.

The status message lists up to three blocking project names and summarizes the rest:

```text
Cannot delete from Library: source is used by other projects: Project A, Project B, Project C, and 2 more.
```

Archived projects are included in this check and can appear in the blocking list.

The blocking list excludes the current project. If the current project is the only project using the source, deletion is allowed.

The backend returns at most three blocking projects. Blocking projects are returned in a stable order, preferably by project title and then project id, so tests and status text are deterministic. `remaining_blocking_project_count` is the number of blocking projects beyond those returned.

## Backend Design

Add a project-scoped Tauri command:

```rust
delete_project_youtube_video_source_from_library(project_id: i64, source_id: i64)
```

The command returns a structured outcome instead of using an error for the expected "used by other projects" branch. `blocking_projects` is payload-capped by the backend at length `<= 3`.

```ts
type DeleteProjectYoutubeVideoSourceOutcome =
  | {
      status: "deleted";
      blocking_projects: [];
      remaining_blocking_project_count: 0;
    }
  | {
      status: "blocked_by_other_projects";
      blocking_projects: Array<{ project_id: number; title: string; archived: boolean }>;
      remaining_blocking_project_count: number;
    };
```

Validation errors remain errors:

- source not found;
- project not found;
- source is not linked to the current project;
- source is not `youtube / video`;
- source identity repair or active ingest/delete lock prevents deletion.

The command runs the preflight and deletion inside one write transaction. The implementation must acquire a SQLite write lock before the "other projects" check, for example with an immediate transaction or an equivalent first write, so a concurrent project-source insert cannot appear between the check and source deletion.

1. Acquire the same `SourceIngestLocks` state with `SourceIngestKind::Delete` that `delete_source` uses for `source_id`.
2. Start the write transaction.
3. Enable and verify `PRAGMA foreign_keys = ON` on the transaction connection before deleting rows.
4. Confirm the source exists and is `youtube / video`.
5. Confirm `project_sources(project_id, source_id)` exists.
6. Query all other projects using the source, including archived projects.
7. If any other project exists, explicitly roll back the transaction and return `blocked_by_other_projects` without mutations.
8. Delete the current `project_sources` row.
9. Delete the `sources` row for `source_id`.

The deletion must factor the existing source-delete behavior; reuse-as-is is not valid because `delete_source_from_pool` acquires its own connection and performs its own `project_count > 0` guard. That separate connection would not see the uncommitted deletion of the current `project_sources` row.

Implementation should split source deletion into explicit helpers, for example:

- standalone path: acquire a connection, enable `foreign_keys`, set the busy timeout, check `project_sources` count is zero, then call a low-level row-delete helper;
- project-scoped path: use the already-open write transaction, enable `foreign_keys`, perform the "other projects" structured check, delete the current membership, then call the same low-level row-delete helper without the standalone `project_count` guard.

The standalone `delete_source` command keeps its existing semantics: any project membership blocks deletion with a validation error. The project-scoped command owns the more specific "other projects" branch and must not rely on the standalone count guard for correctness.

The user-facing promise that transcripts, comments, and stored materials are removed is backed by SQLite FK cascade and SET NULL behavior, not by ad hoc manual deletion. Backend tests must prove that the relevant dependent rows disappear or detach as intended.

## Frontend Design

Add API wrapper:

```ts
deleteProjectYoutubeVideoSourceFromLibrary({
  projectId,
  sourceId,
})
```

Add workflow helper in `research-projects-workflow.ts`:

```ts
deleteProjectYoutubeVideoSourceFromLibrary(sourceId: number)
```

Responsibilities:

- derive the selected/current project id from workflow state;
- call `confirm`;
- invoke the new API command;
- map `blocked_by_other_projects` into the status message;
- refresh project workspace and Library catalog after `deleted`;
- preserve the current "saving" guard and error formatting style.

Add model helper:

```ts
selectedProjectSourceLibraryDeleteDisabledReason(rows: ProjectSourceRecord[]): string | null
```

The helper is the single source of truth for the button enabled state in `SourcesTab.svelte` and route-level contract tests.

## Testing

Backend tests:

- deletes a YouTube video only when the source is linked only to the current project;
- enables FK enforcement on the delete connection and removes dependent rows for the deleted source, including at least `items`/YouTube comments, `youtube_transcript_segments`, and `analysis_documents`;
- detaches playlist item rows that pointed at the deleted video source by setting `youtube_playlist_items.video_source_id` to `NULL`;
- returns `blocked_by_other_projects` without deleting anything when another active project uses the source;
- returns `blocked_by_other_projects` without deleting anything when another archived project uses the source;
- returns at most three `blocking_projects` with a correct `remaining_blocking_project_count`;
- rejects a YouTube playlist;
- rejects a Telegram or other non-YouTube source;
- rejects a source not linked to the current project;
- preserves the existing `delete_source` behavior for standalone Library deletion, including the existing project-membership validation;
- covers the refactored low-level source-row delete path from both standalone and project-scoped commands.

Frontend unit and contract tests:

- disabled reason requires exactly one selected YouTube video;
- button is visible in `SourcesTab.svelte` and wired to the new handler;
- no command is called when confirmation is cancelled;
- successful outcome refreshes project data and Library catalog;
- blocked outcome displays up to three project names plus `and N more`;
- existing `Remove` behavior remains membership-only.

Validation:

- focused Vitest tests for changed frontend helpers/contracts;
- `npm.cmd run check` after Svelte/TypeScript changes;
- `cargo check` and focused Rust tests after backend command changes.

## Out Of Scope

- Bulk deletion.
- Deleting playlists, Telegram sources, channels, or other source types from the project toolbar.
- A modal for blocked-project details.
- Listing every blocking project in the status message.
- Changing the standalone Library deletion UI.
- Force-delete behavior.
