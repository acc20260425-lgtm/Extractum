# Projects MVP Design

Date: 2026-06-13

## Goal

Introduce `Project` as a real product and backend concept.

The MVP creates a durable project model, lets users manage project source membership from Library, and lets single-provider projects start analysis runs under project scope. It deliberately avoids project chat, audit log, legacy group migration, and mixed-provider project analysis.

## Confirmed Brief

- Implement a new backend entity, not only a UI rename of `analysis_source_groups`.
- Use direct project-source membership via `project_sources`.
- `/projects` becomes the main workspace for the new Projects model.
- Keep `analysis_source_groups` as legacy. Do not migrate or delete them in this MVP.
- A project can contain sources from multiple providers.
- Project analysis runs are supported only when all project sources are from one provider.
- Project runs are stored in the existing `analysis_runs` table with `project_id` and `scope_type = "project"`.
- Project analysis runs analyze all sources connected to the project at the moment of start.
- A project can be created empty.
- Sources are added to projects only from existing Library sources.
- One Library source can belong to multiple projects.
- A source cannot be connected to the same project twice.
- Project-specific source notes, aliases, priorities, and include rules are out of scope.
- Project chat and audit log are out of scope.

## Chosen Approach

Use a clean Project MVP beside the legacy source-group model.

Add new `projects` and `project_sources` tables, add `project_id` support to `analysis_runs`, and replace the current `/projects` UI with a project workspace. Keep `analysis_source_groups` untouched as a legacy analysis-scope mechanism for existing flows.

This keeps the domain model honest without forcing a risky migration of existing source groups.

## Data Model

### `projects`

Fields:

- `id`
- `name`
- `description` nullable
- `created_at`
- `updated_at`

Rules:

- `name` is trimmed.
- Empty names are rejected.
- Project names are unique in the MVP.
- The unique-name rule is a temporary product policy, not project identity.
- All durable relationships must use `project_id`, not project name.
- Future versions may allow duplicate project names by removing the unique validation/index.

### `project_sources`

Fields:

- `project_id`
- `source_id`
- `added_at`

Rules:

- `UNIQUE(project_id, source_id)`.
- The same Library source can belong to multiple projects.
- The same Library source cannot be connected twice to one project.
- `project_sources` stores only membership and add time in the MVP.
- No `enabled`, `note`, `display_alias`, `priority`, or include rules.

### `analysis_runs`

Add:

- `project_id` nullable
- `scope_type = "project"` as a supported scope

Project runs remain normal `analysis_runs`. Do not add a separate `project_runs` table in the MVP.

For project runs, persist snapshots at start time:

- project label snapshot;
- source id snapshot;
- source title snapshot;
- existing prompt/model/profile/period snapshots already required by the run model.

If project membership changes after a run starts, historical run context stays based on the snapshots.

### Delete Semantics

Project deletion is hard delete in the MVP:

- delete the project;
- delete its `project_sources`;
- delete associated project `analysis_runs`;
- keep Library sources.

This behavior can change in a later version when archive, restore, audit log, or long-term history requirements are introduced.

Library source deletion should be blocked if the source is used by any project. The user should remove the source from projects first.

## Backend API

Add Tauri/backend commands:

- `list_projects()`
- `create_project(name, description)`
- `update_project(project_id, name, description)`
- `delete_project(project_id)`
- `list_project_sources(project_id)`
- `add_project_sources(project_id, source_ids[])`
- `remove_project_sources(project_id, source_ids[])`
- `start_project_analysis(project_id, analysis_options)`
- `list_project_runs(project_id)`

### Project CRUD

`create_project` and `update_project` should:

- trim `name`;
- reject empty names;
- reject duplicate names in the MVP;
- accept nullable/empty `description`;
- return clear validation errors that the frontend can display directly.

`delete_project` should:

- delete the project;
- delete project-source links;
- delete project analysis runs;
- not delete Library sources.

### Source Membership

`add_project_sources` accepts an array of source ids so the Library selector can add multiple sources at once.

It should be idempotent:

- create missing links;
- skip existing links;
- not fail only because some selected sources are already present.

Return an outcome useful for UI feedback:

- `added_count`;
- `already_present_count`;
- optionally affected ids if that matches existing command style.

`remove_project_sources` removes only links from the project. It never deletes Library sources.

`list_project_sources` should return a UI-ready read model:

- source id;
- title;
- provider;
- subtype when available;
- compact metadata/details;
- `added_at`.

### Project Analysis

`start_project_analysis`:

- loads all sources connected to the project;
- rejects projects with zero sources;
- rejects mixed-provider projects with `mixed_provider_project_runs_not_supported`;
- starts the existing analysis flow when all sources are from one provider;
- stores `analysis_runs.project_id`;
- stores `scope_type = "project"`;
- stores project and source snapshots at start time.

The command should reuse the current report-run options where possible:

- period;
- prompt template;
- model/profile;
- output language;
- provider-specific corpus options.

## Frontend UX

`/projects` becomes a three-zone workspace:

```text
ProjectRail | Project Sources Table | Project Inspector
```

### ProjectRail

Purpose: navigation between projects.

Contents:

- project list;
- `Create project`;
- selected-project state;
- empty state when there are no projects.

The rail should represent the new `projects` model only. It should not show legacy `analysis_source_groups` as projects.

### Project Sources Table

Purpose: manage the selected project's Library sources.

Minimum columns:

- `Title`
- `Provider`
- `Subtype`
- `Details`
- `Added to project at`

Actions:

- `Add from Library`;
- `Remove from project`;
- row selection.

Selecting a row updates the Project Inspector with source context. Removing a source removes only the project membership link.

### Add From Library Modal

`Add from Library` opens a modal selector with:

- Library source table;
- search;
- provider/type filters;
- multi-select;
- `Add selected`.

Sources already connected to the selected project remain visible but disabled with status `Already in project`. This avoids silent no-op behavior and explains idempotency to the user.

The modal only connects existing Library sources. It does not create new Library sources. New source creation remains in the Library add-source flow.

### Create/Edit Project Modal

Create and edit use modal dialogs with:

- `name`;
- `description`.

The UI must show backend validation errors clearly, including duplicate-name errors.

The UI should not treat project name as a permanent identity. Navigation and mutations use `project_id`.

### Project Inspector

When no source row is selected, show project context:

- project name;
- description;
- total source count;
- provider breakdown;
- run eligibility;
- actions:
  - `Run project analysis`;
  - `Edit project`;
  - `Delete project`;
- recent project runs.

When a source row is selected, show selected-source context:

- source title;
- provider/subtype;
- compact metadata/details;
- `added_at`;
- `Remove from project`.

Keep project summary visible in compact form so selection does not make the project context disappear entirely.

### Run Project Analysis

The action should reuse the existing report-run modal/form with project scope.

Disable the action and show a clear reason when:

- no project is selected;
- the project has zero sources;
- the project contains mixed providers;
- required report-run dependencies are unavailable;
- the backend reports project scope as invalid.

For mixed-provider projects, use an explicit message such as:

```text
Mixed-provider project runs are not supported yet.
```

### Global Reports/Runs

Project runs should appear both:

- in the selected project's recent runs;
- in global Reports/Runs views.

Global views must support `scope_type = "project"` and display the project label from the run snapshot/read model.

## Compatibility

- Existing source and source-group analysis flows continue to work.
- Existing `analysis_source_groups` remain in the backend as legacy.
- No automatic conversion from legacy groups to projects.
- No `/projects-v2` route; `/projects` becomes the new Projects workspace.
- Library remains the owner of source creation and source metadata.
- Projects own only membership, project metadata, and project-scoped analysis context.

## Non-Goals

- Project chat.
- Audit log.
- Migrating `analysis_source_groups` into projects.
- Mixed-provider project analysis.
- Separate `project_runs` table.
- Project-specific default LLM settings.
- Project-specific source notes, aliases, priority, or include rules.
- Soft delete, archive, or restore.
- Creating new Library sources inside the Project add-source modal.

## Backlog After MVP

- Allow duplicate project names if a real workflow requires it.
- Mixed-provider project analysis.
- Project-specific default analysis settings.
- Convert legacy source group to project.
- Archive/restore instead of hard delete.
- Project chat.
- Audit log.
- Project-specific source notes and aliases.
- Separate orchestration model if project-level jobs need more than one `analysis_run`.

## Testing Notes

Backend tests should cover:

- project create/update validation;
- duplicate project names;
- hard delete behavior;
- source membership idempotency;
- source membership uniqueness;
- blocking Library source deletion when used by projects;
- project run rejection for empty projects;
- project run rejection for mixed-provider projects;
- project run creation for single-provider projects;
- `analysis_runs.project_id` and snapshots.

Frontend tests should cover:

- `/projects` empty state;
- create/edit/delete project flows;
- Add from Library modal;
- already-connected disabled rows;
- project source removal;
- run disabled reasons;
- recent project runs rendering;
- global Reports/Runs rendering project-scoped runs.
