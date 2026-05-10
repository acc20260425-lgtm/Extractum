# Analysis Result-First Redesign Part 1 Preparatory Pass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the enabling contracts for the `/analysis` result-first redesign without changing the visible workspace layout.

**Architecture:** This part adds a small frontend state-contract module, persists the YouTube corpus mode on analysis runs, and exposes snapshot-only paged run-message access. The current `WorkspaceRail`, `WorkspaceMain`, `WorkspaceInspector`, and `/analysis` route layout stay visually unchanged in this part so later UI phases can migrate onto safer contracts.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, Vitest, Tauri 2 commands, Rust 2021, sqlx SQLite migrations, existing Extractum analysis modules.

---

## Part Boundary

This is **Part 1 of 7** for the approved result-first redesign.

Part 1 may:

- add pure frontend workspace state helpers;
- add focused unit tests for those helpers;
- add a migration and model fields for `analysis_runs.youtube_corpus_mode`;
- keep legacy runs compatible by defaulting old rows to `transcript_description`;
- add a snapshot-only paged API over `analysis_run_messages`;
- add frontend API wrappers and tests for that API.

Part 1 must not:

- introduce `CompactSourceRail`;
- introduce `ReportCanvas`;
- introduce `RunCompanionTabs`;
- redesign `/analysis/+page.svelte`;
- move existing source readers into the central canvas;
- change saved-run immutability;
- make completed-run evidence/chat fall back to live source data.

Stop after this part is implemented, verified, and committed. Continue to Part 2 only after explicit user approval.

## File Structure

- Create: `src/lib/analysis-workspace-state.ts`
  - Responsibility: define `WorkspaceSelection`, `OpenRunState`, `CanvasMode`, `SourceViewBasis`, `CompanionTab`, and pure state-transition helpers for the redesign.
- Create: `src/lib/analysis-workspace-state.test.ts`
  - Responsibility: lock the run-opening, workspace-switch, deleted-scope, and restored-state normalization rules.
- Modify: `src/lib/types/analysis.ts`
  - Responsibility: expose `youtube_corpus_mode` on run summaries/details and define frontend snapshot-message page types.
- Modify: `src/lib/api/analysis-runs.ts`
  - Responsibility: wrap the new `list_analysis_run_messages` Tauri command.
- Modify: `src/lib/api/analysis-runs.test.ts`
  - Responsibility: verify frontend IPC command names and argument shapes.
- Create: `src-tauri/migrations/17.sql`
  - Responsibility: add durable `youtube_corpus_mode` metadata to `analysis_runs`.
- Modify: `src-tauri/src/migrations.rs`
  - Responsibility: register migration 17 and test that the migration is included.
- Modify: `docs/database-schema.md`
  - Responsibility: document the new analysis run field and migration entry.
- Modify: `src-tauri/src/analysis/models.rs`
  - Responsibility: add run metadata fields and serializable snapshot-message page DTOs.
- Modify: `src-tauri/src/analysis/store.rs`
  - Responsibility: read, map, duplicate-check, and insert `youtube_corpus_mode`.
- Modify: `src-tauri/src/analysis/report.rs`
  - Responsibility: pass the parsed YouTube corpus mode into persisted run creation.
- Modify: `src-tauri/src/analysis/corpus.rs`
  - Responsibility: add a wire serializer for `YoutubeCorpusMode` and a snapshot-only paged loader that never resolves live source data.
- Modify: `src-tauri/src/analysis/mod.rs`
  - Responsibility: expose the `list_analysis_run_messages` Tauri command.
- Modify: `src-tauri/src/lib.rs`
  - Responsibility: register the new command in `tauri::generate_handler!`.

## Task 1: Add Frontend Workspace State Contract

**Files:**
- Create: `src/lib/analysis-workspace-state.test.ts`
- Create: `src/lib/analysis-workspace-state.ts`

- [ ] **Step 1: Write the failing state-contract tests**

Create `src/lib/analysis-workspace-state.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  defaultAnalysisWorkspaceUiState,
  legacyScopeFromWorkspaceSelection,
  normalizeRestoredWorkspaceState,
  openRunWorkspaceState,
  selectSourceGroupWorkspace,
  selectSourceWorkspace,
  workspaceSelectionFromLegacy,
  workspaceSelectionFromRunScope,
  type AnalysisWorkspaceUiState,
} from "./analysis-workspace-state";

function baseState(overrides: Partial<AnalysisWorkspaceUiState> = {}): AnalysisWorkspaceUiState {
  return {
    ...defaultAnalysisWorkspaceUiState(),
    ...overrides,
  };
}

describe("analysis-workspace-state", () => {
  it("maps legacy route scope ids into explicit workspace selections", () => {
    expect(workspaceSelectionFromLegacy("single_source", "7", "")).toEqual({
      kind: "source",
      sourceId: 7,
    });
    expect(workspaceSelectionFromLegacy("source_group", "", "9")).toEqual({
      kind: "source_group",
      sourceGroupId: 9,
    });
    expect(workspaceSelectionFromLegacy("single_source", "", "9")).toEqual({ kind: "none" });
    expect(workspaceSelectionFromLegacy("source_group", "7", "")).toEqual({ kind: "none" });
    expect(workspaceSelectionFromLegacy("single_source", "not-a-number", "")).toEqual({
      kind: "none",
    });
  });

  it("maps explicit workspace selection back to legacy route scope ids", () => {
    expect(legacyScopeFromWorkspaceSelection({ kind: "source", sourceId: 7 })).toEqual({
      analysisScope: "single_source",
      selectedSourceId: "7",
      selectedGroupId: "",
    });
    expect(legacyScopeFromWorkspaceSelection({ kind: "source_group", sourceGroupId: 9 }))
      .toEqual({
        analysisScope: "source_group",
        selectedSourceId: "",
        selectedGroupId: "9",
      });
    expect(legacyScopeFromWorkspaceSelection({ kind: "none" })).toEqual({
      analysisScope: "single_source",
      selectedSourceId: "",
      selectedGroupId: "",
    });
  });

  it("opens a completed run as a saved report and defaults companion to evidence", () => {
    const next = openRunWorkspaceState(baseState(), {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    });

    expect(next).toMatchObject({
      workspaceSelection: { kind: "source", sourceId: 7 },
      openRunState: { kind: "saved", runId: 42 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: null,
    });
  });

  it("opens queued and running runs as active reports", () => {
    expect(openRunWorkspaceState(baseState(), {
      runId: 43,
      status: "queued",
      sourceId: null,
      sourceGroupId: 9,
      liveScopeExists: true,
    })).toMatchObject({
      workspaceSelection: { kind: "source_group", sourceGroupId: 9 },
      openRunState: { kind: "active", runId: 43 },
      canvasMode: "report",
      companionTab: "runs",
    });

    expect(openRunWorkspaceState(baseState(), {
      runId: 44,
      status: "running",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    }).openRunState).toEqual({ kind: "active", runId: 44 });
  });

  it("does not fake a live workspace selection for a run with deleted scope", () => {
    const next = openRunWorkspaceState(baseState({
      workspaceSelection: { kind: "source", sourceId: 99 },
    }), {
      runId: 45,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: false,
    });

    expect(workspaceSelectionFromRunScope({
      runId: 45,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: false,
    })).toEqual({ kind: "none" });
    expect(next.workspaceSelection).toEqual({ kind: "none" });
    expect(next.openRunState).toEqual({ kind: "saved", runId: 45 });
  });

  it("selecting a source clears run-bound state and returns to live source mode", () => {
    const next = selectSourceWorkspace(baseState({
      openRunState: { kind: "saved", runId: 42 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: "s7-i1",
    }), 8);

    expect(next).toEqual({
      workspaceSelection: { kind: "source", sourceId: 8 },
      openRunState: { kind: "none" },
      canvasMode: "source",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
  });

  it("selecting a source group clears run-bound state and returns to live source mode", () => {
    const next = selectSourceGroupWorkspace(baseState({
      openRunState: { kind: "active", runId: 42 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      selectedTraceRef: "s7-i1",
    }), 10);

    expect(next).toEqual({
      workspaceSelection: { kind: "source_group", sourceGroupId: 10 },
      openRunState: { kind: "none" },
      canvasMode: "source",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
  });

  it("normalizes restored run-bound UI state when no run is open", () => {
    expect(normalizeRestoredWorkspaceState(baseState({
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      selectedTraceRef: "s7-i1",
    }))).toEqual({
      workspaceSelection: { kind: "none" },
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });

    expect(normalizeRestoredWorkspaceState(baseState({
      openRunState: { kind: "saved", runId: 42 },
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: "s7-i1",
    }))).toMatchObject({
      openRunState: { kind: "saved", runId: 42 },
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: "s7-i1",
    });
  });
});
```

- [ ] **Step 2: Run the tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-state.test.ts
```

Expected: FAIL because `src/lib/analysis-workspace-state.ts` does not exist.

- [ ] **Step 3: Add the state-contract module**

Create `src/lib/analysis-workspace-state.ts`:

```ts
import type { AnalysisRunSummary } from "$lib/types/analysis";

export type WorkspaceSelection =
  | { kind: "source"; sourceId: number }
  | { kind: "source_group"; sourceGroupId: number }
  | { kind: "none" };

export type OpenRunState =
  | { kind: "none" }
  | { kind: "active"; runId: number }
  | { kind: "saved"; runId: number };

export type CanvasMode = "report" | "source";
export type SourceViewBasis = "live_source" | "run_snapshot";
export type CompanionTab = "evidence" | "chat" | "runs";
export type LegacyAnalysisScope = "single_source" | "source_group";

export interface AnalysisWorkspaceUiState {
  workspaceSelection: WorkspaceSelection;
  openRunState: OpenRunState;
  canvasMode: CanvasMode;
  sourceViewBasis: SourceViewBasis;
  companionTab: CompanionTab;
  selectedTraceRef: string | null;
}

export interface LegacyAnalysisScopeState {
  analysisScope: LegacyAnalysisScope;
  selectedSourceId: string;
  selectedGroupId: string;
}

export interface RunWorkspaceInput {
  runId: number;
  status: string;
  sourceId: number | null;
  sourceGroupId: number | null;
  liveScopeExists?: boolean;
}

function numericId(value: string) {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

export function defaultAnalysisWorkspaceUiState(): AnalysisWorkspaceUiState {
  return {
    workspaceSelection: { kind: "none" },
    openRunState: { kind: "none" },
    canvasMode: "source",
    sourceViewBasis: "live_source",
    companionTab: "runs",
    selectedTraceRef: null,
  };
}

export function workspaceSelectionFromLegacy(
  analysisScope: LegacyAnalysisScope,
  selectedSourceId: string,
  selectedGroupId: string,
): WorkspaceSelection {
  if (analysisScope === "single_source") {
    const sourceId = numericId(selectedSourceId);
    return sourceId === null ? { kind: "none" } : { kind: "source", sourceId };
  }

  const sourceGroupId = numericId(selectedGroupId);
  return sourceGroupId === null ? { kind: "none" } : { kind: "source_group", sourceGroupId };
}

export function legacyScopeFromWorkspaceSelection(
  selection: WorkspaceSelection,
): LegacyAnalysisScopeState {
  if (selection.kind === "source") {
    return {
      analysisScope: "single_source",
      selectedSourceId: String(selection.sourceId),
      selectedGroupId: "",
    };
  }

  if (selection.kind === "source_group") {
    return {
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: String(selection.sourceGroupId),
    };
  }

  return {
    analysisScope: "single_source",
    selectedSourceId: "",
    selectedGroupId: "",
  };
}

export function runWorkspaceInputFromSummary(
  run: Pick<AnalysisRunSummary, "id" | "status" | "source_id" | "source_group_id">,
  liveScopeExists = true,
): RunWorkspaceInput {
  return {
    runId: run.id,
    status: run.status,
    sourceId: run.source_id,
    sourceGroupId: run.source_group_id,
    liveScopeExists,
  };
}

export function workspaceSelectionFromRunScope(run: RunWorkspaceInput): WorkspaceSelection {
  if (run.liveScopeExists === false) {
    return { kind: "none" };
  }

  if (run.sourceId !== null) {
    return { kind: "source", sourceId: run.sourceId };
  }

  if (run.sourceGroupId !== null) {
    return { kind: "source_group", sourceGroupId: run.sourceGroupId };
  }

  return { kind: "none" };
}

export function openRunStateForStatus(status: string, runId: number): OpenRunState {
  if (status === "queued" || status === "running") {
    return { kind: "active", runId };
  }

  return { kind: "saved", runId };
}

export function defaultCompanionTabForRun(status: string): CompanionTab {
  return status === "completed" ? "evidence" : "runs";
}

export function openRunWorkspaceState(
  current: AnalysisWorkspaceUiState,
  run: RunWorkspaceInput,
): AnalysisWorkspaceUiState {
  return {
    ...current,
    workspaceSelection: workspaceSelectionFromRunScope(run),
    openRunState: openRunStateForStatus(run.status, run.runId),
    canvasMode: "report",
    sourceViewBasis: "run_snapshot",
    companionTab: defaultCompanionTabForRun(run.status),
    selectedTraceRef: null,
  };
}

export function clearRunBoundWorkspaceState(
  current: AnalysisWorkspaceUiState,
): AnalysisWorkspaceUiState {
  return {
    ...current,
    openRunState: { kind: "none" },
    sourceViewBasis: "live_source",
    companionTab: "runs",
    selectedTraceRef: null,
  };
}

export function selectSourceWorkspace(
  current: AnalysisWorkspaceUiState,
  sourceId: number,
): AnalysisWorkspaceUiState {
  return {
    ...clearRunBoundWorkspaceState(current),
    workspaceSelection: { kind: "source", sourceId },
    canvasMode: "source",
  };
}

export function selectSourceGroupWorkspace(
  current: AnalysisWorkspaceUiState,
  sourceGroupId: number,
): AnalysisWorkspaceUiState {
  return {
    ...clearRunBoundWorkspaceState(current),
    workspaceSelection: { kind: "source_group", sourceGroupId },
    canvasMode: "source",
  };
}

export function normalizeRestoredWorkspaceState(
  state: AnalysisWorkspaceUiState,
): AnalysisWorkspaceUiState {
  if (state.openRunState.kind !== "none") {
    return state;
  }

  return {
    ...state,
    sourceViewBasis: "live_source",
    companionTab:
      state.companionTab === "evidence" || state.companionTab === "chat"
        ? "runs"
        : state.companionTab,
    selectedTraceRef: null,
  };
}
```

- [ ] **Step 4: Run the state-contract tests and verify they pass**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-state.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the frontend contract**

Run:

```powershell
git add src/lib/analysis-workspace-state.ts src/lib/analysis-workspace-state.test.ts
git commit -m "feat: add analysis workspace state contract"
```

## Task 2: Persist YouTube Corpus Mode On Analysis Runs

**Files:**
- Create: `src-tauri/migrations/17.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `docs/database-schema.md`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/analysis-state.test.ts`

- [ ] **Step 1: Write failing migration registration test**

In `src-tauri/src/migrations.rs`, add this test to the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn includes_analysis_run_youtube_corpus_mode_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 17)
        .expect("version 17 migration is registered");

    for fragment in [
        "ALTER TABLE analysis_runs ADD COLUMN youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description'",
        "CHECK (youtube_corpus_mode IN",
        "'transcript_only'",
        "'transcript_description'",
        "'transcript_description_comments'",
    ] {
        assert!(
            migration.sql.contains(fragment),
            "missing migration fragment {fragment}"
        );
    }
}
```

- [ ] **Step 2: Run the Rust migration test and verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_analysis_run_youtube_corpus_mode_migration
```

Expected: FAIL because migration 17 is not registered.

- [ ] **Step 3: Add migration 17**

Create `src-tauri/migrations/17.sql`:

```sql
ALTER TABLE analysis_runs
ADD COLUMN youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description'
CHECK (
    youtube_corpus_mode IN (
        'transcript_only',
        'transcript_description',
        'transcript_description_comments'
    )
);
```

- [ ] **Step 4: Register migration 17**

In `src-tauri/src/migrations.rs`, add this entry after version 16 in `build_migrations()`:

```rust
Migration {
    version: 17,
    description: "add youtube corpus mode to analysis runs",
    sql: include_str!("../migrations/17.sql"),
    kind: MigrationKind::Up,
},
```

- [ ] **Step 5: Run the migration registration test and verify it passes**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_analysis_run_youtube_corpus_mode_migration
```

Expected: PASS.

- [ ] **Step 6: Add a wire serializer for the existing backend enum**

In `src-tauri/src/analysis/corpus.rs`, extend `impl YoutubeCorpusMode`:

```rust
pub(crate) fn as_wire(self) -> &'static str {
    match self {
        Self::TranscriptOnly => "transcript_only",
        Self::TranscriptDescription => "transcript_description",
        Self::TranscriptDescriptionComments => "transcript_description_comments",
    }
}
```

Also extend the existing `youtube_corpus_mode_parses_wire_values_and_defaults` test:

```rust
assert_eq!(
    YoutubeCorpusMode::TranscriptOnly.as_wire(),
    "transcript_only"
);
assert_eq!(
    YoutubeCorpusMode::TranscriptDescription.as_wire(),
    "transcript_description"
);
assert_eq!(
    YoutubeCorpusMode::TranscriptDescriptionComments.as_wire(),
    "transcript_description_comments"
);
```

- [ ] **Step 7: Add run model fields**

In `src-tauri/src/analysis/models.rs`, add this field to `AnalysisRunSummary`, `AnalysisRunDetail`, and `AnalysisRunRow` immediately after `model`:

```rust
pub youtube_corpus_mode: String,
```

For `AnalysisRunRow`, keep crate visibility:

```rust
pub(crate) youtube_corpus_mode: String,
```

- [ ] **Step 8: Select and map the persisted mode**

In `src-tauri/src/analysis/store.rs`, update `map_run_summary` and `map_run_detail` to copy `row.youtube_corpus_mode` into the returned DTOs:

```rust
youtube_corpus_mode: row.youtube_corpus_mode,
```

In `fetch_run_row`, add this selected column immediately after `runs.model`:

```sql
runs.youtube_corpus_mode,
```

In all three `list_analysis_runs` SELECT statements in `src-tauri/src/analysis/mod.rs`, add the same column immediately after `runs.model`:

```sql
runs.youtube_corpus_mode,
```

- [ ] **Step 9: Persist the selected mode when inserting a run**

In `src-tauri/src/analysis/store.rs`, import the enum:

```rust
use super::corpus::YoutubeCorpusMode;
```

Add this field to `AnalysisRunInsert<'a>`:

```rust
pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
```

In `insert_analysis_run`, add the column after `model`:

```sql
youtube_corpus_mode,
```

Add the matching value placeholder after the model placeholder:

```sql
?,
```

Bind the wire value after binding `insert.model`:

```rust
.bind(insert.youtube_corpus_mode.as_wire())
```

The resulting insert column block should include:

```sql
provider,
model,
youtube_corpus_mode,
status,
scope_label_snapshot,
created_at
```

The resulting values block should include:

```sql
?, ?, ?, ?, NULL, ?
```

- [ ] **Step 10: Include the mode in active duplicate detection**

In `src-tauri/src/analysis/store.rs`, add this field to `DuplicateRunLookup<'a>`:

```rust
pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
```

In `find_active_duplicate_run`, add this predicate after `AND model = ?`:

```sql
AND youtube_corpus_mode = ?
```

Bind the wire value immediately after `lookup.model`:

```rust
.bind(lookup.youtube_corpus_mode.as_wire())
```

In `src-tauri/src/analysis/report.rs`, pass the parsed mode to both structs:

```rust
youtube_corpus_mode,
```

Use it in `DuplicateRunLookup` and `AnalysisRunInsert`.

- [ ] **Step 11: Update Rust test fixtures**

Every in-memory `analysis_runs` table used by tests must include the new column:

```sql
youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
```

Update these known table definitions:

- `src-tauri/src/analysis/mod.rs` test helper `memory_pool()`;
- `src-tauri/src/analysis/corpus.rs` test helper `snapshot_pool()`;
- any new `analysis_runs` table created in `src-tauri/src/analysis/store.rs` tests.

Update every direct `AnalysisRunRow` construction in `src-tauri/src/analysis/store.rs` tests:

```rust
youtube_corpus_mode: "transcript_description_comments".to_string(),
```

Update every direct `AnalysisRunDetail` construction in `src-tauri/src/analysis/corpus.rs` tests:

```rust
youtube_corpus_mode: "transcript_description".to_string(),
```

- [ ] **Step 12: Add persistence-focused store tests**

In `src-tauri/src/analysis/store.rs` tests, add:

```rust
#[test]
fn map_run_summary_exposes_youtube_corpus_mode() {
    let summary = map_run_summary(sample_run_row());
    assert_eq!(
        summary.youtube_corpus_mode,
        "transcript_description_comments"
    );
}

#[test]
fn map_run_detail_exposes_youtube_corpus_mode() {
    let detail = map_run_detail(sample_run_row());
    assert_eq!(
        detail.youtube_corpus_mode,
        "transcript_description_comments"
    );
}
```

Then add an async insert test:

```rust
#[tokio::test]
async fn insert_analysis_run_persists_youtube_corpus_mode() {
    use super::{insert_analysis_run, AnalysisRunInsert};
    use crate::analysis::corpus::YoutubeCorpusMode;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        r#"
        CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_type TEXT NOT NULL,
            scope_type TEXT NOT NULL,
            source_id INTEGER,
            source_group_id INTEGER,
            period_from INTEGER NOT NULL,
            period_to INTEGER NOT NULL,
            output_language TEXT NOT NULL,
            prompt_template_id INTEGER,
            prompt_template_version INTEGER NOT NULL,
            provider_profile TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
            status TEXT NOT NULL,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            scope_label_snapshot TEXT,
            error TEXT,
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create runs");

    let template = AnalysisPromptTemplate {
        id: 5,
        name: "Report".to_string(),
        template_kind: "report".to_string(),
        body: "Body".to_string(),
        version: 3,
        is_builtin: false,
        created_at: 1,
        updated_at: 1,
    };

    let run_id = insert_analysis_run(
        &pool,
        &AnalysisRunInsert {
            scope_type: "single_source",
            source_id: Some(7),
            source_group_id: None,
            period_from: 10,
            period_to: 20,
            output_language: "English",
            prompt_template: &template,
            provider_profile: "default",
            provider: "gemini",
            model: "gemini-2.5-flash",
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescriptionComments,
        },
    )
    .await
    .expect("insert run");

    let mode = sqlx::query_scalar::<_, String>(
        "SELECT youtube_corpus_mode FROM analysis_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("load mode");

    assert_eq!(mode, "transcript_description_comments");
}
```

- [ ] **Step 13: Update frontend run types**

In `src/lib/types/analysis.ts`, add this property to `AnalysisRunSummary` immediately after `model`:

```ts
youtube_corpus_mode: YoutubeCorpusMode;
```

`AnalysisRunDetail` extends `AnalysisRunSummary`, so no separate property is needed there.

Update `runSummary()` in `src/lib/analysis-state.test.ts` with:

```ts
youtube_corpus_mode: "transcript_description",
```

- [ ] **Step 14: Update database docs**

In `docs/database-schema.md`, add `youtube_corpus_mode` to the `analysis_runs` important fields after `model`:

```markdown
- `youtube_corpus_mode`
```

Add this migration row after version 16:

```markdown
| 17 | `17.sql` | Add durable YouTube corpus mode metadata to `analysis_runs` |
```

Add this current behavior implication:

```markdown
- `analysis_runs.youtube_corpus_mode` preserves the selected YouTube corpus scope used by the run, rather than reconstructing it from current source defaults.
```

- [ ] **Step 15: Run focused metadata tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary_exposes_youtube_corpus_mode analysis::store::tests::map_run_detail_exposes_youtube_corpus_mode analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode analysis::corpus::tests::youtube_corpus_mode_parses_wire_values_and_defaults migrations::tests::includes_analysis_run_youtube_corpus_mode_migration
```

Expected: PASS.

Then run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: PASS.

- [ ] **Step 16: Commit the run metadata changes**

Run:

```powershell
git add src-tauri/migrations/17.sql src-tauri/src/migrations.rs docs/database-schema.md src-tauri/src/analysis/models.rs src-tauri/src/analysis/store.rs src-tauri/src/analysis/report.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/mod.rs src/lib/types/analysis.ts src/lib/analysis-state.test.ts
git commit -m "feat: persist analysis youtube corpus mode"
```

## Task 3: Add Snapshot-Only Paged Run Message Access

**Files:**
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Modify: `src/lib/api/analysis-runs.test.ts`

- [ ] **Step 1: Add failing frontend API wrapper test**

In `src/lib/api/analysis-runs.test.ts`, add `listAnalysisRunMessages` to the import list:

```ts
  listAnalysisRunMessages,
```

Then add this test:

```ts
it("wraps snapshot-only run message paging", async () => {
  invokeMock.mockResolvedValueOnce({
    messages: [],
    next_cursor: null,
    has_more: false,
  });

  await expect(listAnalysisRunMessages({
    runId: 42,
    after: { published_at: 1_710_000_000, ref: "s7-i1" },
    limit: 25,
  })).resolves.toEqual({
    messages: [],
    next_cursor: null,
    has_more: false,
  });

  expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_run_messages", {
    runId: 42,
    after: { published_at: 1_710_000_000, ref: "s7-i1" },
    limit: 25,
  });
});
```

- [ ] **Step 2: Run the frontend API test and verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts
```

Expected: FAIL because `listAnalysisRunMessages` is not exported.

- [ ] **Step 3: Add frontend snapshot-message types**

In `src/lib/types/analysis.ts`, add:

```ts
export interface AnalysisRunMessageCursor {
  published_at: number;
  ref: string;
}

export interface AnalysisRunMessage {
  item_id: number;
  source_id: number;
  external_id: string;
  author: string | null;
  published_at: number;
  ref: string;
  content: string;
  item_kind: string | null;
  source_type: string | null;
  source_subtype: string | null;
  metadata_json: unknown | null;
}

export interface AnalysisRunMessagesPage {
  messages: AnalysisRunMessage[];
  next_cursor: AnalysisRunMessageCursor | null;
  has_more: boolean;
}

export interface ListAnalysisRunMessagesInput {
  runId: number;
  after: AnalysisRunMessageCursor | null;
  limit: number;
}
```

- [ ] **Step 4: Add the frontend API wrapper**

In `src/lib/api/analysis-runs.ts`, import the new types:

```ts
  AnalysisRunMessagesPage,
  ListAnalysisRunMessagesInput,
```

Then add:

```ts
export function listAnalysisRunMessages(input: ListAnalysisRunMessagesInput) {
  return invoke<AnalysisRunMessagesPage>("list_analysis_run_messages", { ...input });
}
```

- [ ] **Step 5: Run the frontend API test and verify it passes**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts
```

Expected: PASS.

- [ ] **Step 6: Add backend DTOs for snapshot pages**

In `src-tauri/src/analysis/models.rs`, add these serializable DTOs near the other analysis run DTOs:

```rust
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct AnalysisRunMessageCursor {
    pub published_at: i64,
    pub r#ref: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisRunMessage {
    pub item_id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub author: Option<String>,
    pub published_at: i64,
    pub r#ref: String,
    pub content: String,
    pub item_kind: Option<String>,
    pub source_type: Option<String>,
    pub source_subtype: Option<String>,
    pub metadata_json: Option<serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AnalysisRunMessagesPage {
    pub messages: Vec<AnalysisRunMessage>,
    pub next_cursor: Option<AnalysisRunMessageCursor>,
    pub has_more: bool,
}
```

- [ ] **Step 7: Write failing backend snapshot-only loader tests**

In `src-tauri/src/analysis/corpus.rs`, extend the test imports:

```rust
use crate::analysis::models::AnalysisRunMessageCursor;
```

Add `list_run_snapshot_messages_page` and `ListRunSnapshotMessagesRequest` to the `use super::{ ... }` list.

Then add these tests:

```rust
#[tokio::test]
async fn list_run_snapshot_messages_page_reads_saved_snapshot_only() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_group_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, status, created_at
         )
         VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)",
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");

    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");

    let page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 1,
        },
    )
    .await
    .expect("load first page");

    assert_eq!(page.messages.len(), 1);
    assert_eq!(page.messages[0].content, "First frozen message");
    assert_eq!(page.messages[0].source_type.as_deref(), Some("youtube"));
    assert_eq!(page.messages[0].metadata_json.as_ref().and_then(|value| value.get("video_id")).and_then(|value| value.as_str()), Some("video2"));
    assert!(page.has_more);

    let second_page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: page.next_cursor,
            limit: 1,
        },
    )
    .await
    .expect("load second page");

    assert_eq!(second_page.messages.len(), 1);
    assert_eq!(second_page.messages[0].content, "Second frozen message");
    assert!(!second_page.has_more);
    assert_eq!(second_page.next_cursor, None);
}

#[tokio::test]
async fn list_run_snapshot_messages_page_does_not_fall_back_to_live_source() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, status, created_at
         )
         VALUES (1, 'report', 'single_source', 2, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)",
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");

    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("telegram_message")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(compress_text("Live source message").expect("compress live message"))
    .execute(&pool)
    .await
    .expect("insert live item");

    let page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 25,
        },
    )
    .await
    .expect("load snapshot-only page");

    assert_eq!(page.messages, Vec::new());
    assert_eq!(page.next_cursor, None);
    assert!(!page.has_more);
}

#[test]
fn run_message_cursor_uses_ref_and_published_at() {
    let cursor = AnalysisRunMessageCursor {
        published_at: 1_710_000_000,
        r#ref: "s7-i1".to_string(),
    };

    assert_eq!(cursor.published_at, 1_710_000_000);
    assert_eq!(cursor.r#ref, "s7-i1");
}
```

- [ ] **Step 8: Run the backend loader tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only analysis::corpus::tests::list_run_snapshot_messages_page_does_not_fall_back_to_live_source analysis::corpus::tests::run_message_cursor_uses_ref_and_published_at
```

Expected: FAIL because `list_run_snapshot_messages_page` and `ListRunSnapshotMessagesRequest` do not exist.

- [ ] **Step 9: Add the snapshot-only paged loader**

In `src-tauri/src/analysis/corpus.rs`, update the models import:

```rust
use super::models::{
    AnalysisRunDetail, AnalysisRunMessage, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    CorpusMessage, StoredAnalysisItemRow, StoredRunSnapshotRow,
};
```

Add the request struct near `load_run_snapshot_messages`:

```rust
pub(crate) struct ListRunSnapshotMessagesRequest {
    pub(crate) run_id: i64,
    pub(crate) after: Option<AnalysisRunMessageCursor>,
    pub(crate) limit: usize,
}
```

Add these helpers before `load_run_snapshot_messages`:

```rust
fn decode_optional_metadata_json(
    metadata_zstd: Option<&[u8]>,
) -> Result<Option<serde_json::Value>, String> {
    let Some(bytes) = metadata_zstd else {
        return Ok(None);
    };

    let decompressed = decompress_bytes(bytes)?;
    serde_json::from_slice(&decompressed)
        .map(Some)
        .map_err(|e| format!("Failed to decode run message metadata JSON: {e}"))
}

fn run_message_from_snapshot_row(row: StoredRunSnapshotRow) -> Result<AnalysisRunMessage, String> {
    Ok(AnalysisRunMessage {
        item_id: row.item_id,
        source_id: row.source_id,
        external_id: row.external_id,
        author: row.author,
        published_at: row.published_at,
        r#ref: row.r#ref,
        content: decompress_text(&row.content_zstd)?,
        item_kind: row.item_kind,
        source_type: row.source_type,
        source_subtype: row.source_subtype,
        metadata_json: decode_optional_metadata_json(row.metadata_zstd.as_deref())?,
    })
}
```

Add the paged loader:

```rust
pub(crate) async fn list_run_snapshot_messages_page(
    pool: &Pool<Sqlite>,
    request: ListRunSnapshotMessagesRequest,
) -> Result<AnalysisRunMessagesPage, String> {
    let limit = request.limit.clamp(1, 500);
    let fetch_limit = (limit + 1) as i64;

    let rows: Vec<StoredRunSnapshotRow> = if let Some(after) = request.after {
        sqlx::query_as(
            r#"
            SELECT
                item_id,
                source_id,
                external_id,
                author,
                published_at,
                ref,
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            FROM analysis_run_messages
            WHERE run_id = ?
              AND (
                published_at > ?
                OR (published_at = ? AND ref > ?)
              )
            ORDER BY published_at ASC, ref ASC
            LIMIT ?
            "#,
        )
        .bind(request.run_id)
        .bind(after.published_at)
        .bind(after.published_at)
        .bind(after.r#ref)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            r#"
            SELECT
                item_id,
                source_id,
                external_id,
                author,
                published_at,
                ref,
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            FROM analysis_run_messages
            WHERE run_id = ?
            ORDER BY published_at ASC, ref ASC
            LIMIT ?
            "#,
        )
        .bind(request.run_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let has_more = rows.len() > limit;
    let page_rows = rows.into_iter().take(limit).collect::<Vec<_>>();
    let mut messages = Vec::with_capacity(page_rows.len());
    for row in page_rows {
        messages.push(run_message_from_snapshot_row(row)?);
    }

    let next_cursor = if has_more {
        messages.last().map(|message| AnalysisRunMessageCursor {
            published_at: message.published_at,
            r#ref: message.r#ref.clone(),
        })
    } else {
        None
    };

    Ok(AnalysisRunMessagesPage {
        messages,
        next_cursor,
        has_more,
    })
}
```

- [ ] **Step 10: Run the backend loader tests and verify they pass**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only analysis::corpus::tests::list_run_snapshot_messages_page_does_not_fall_back_to_live_source analysis::corpus::tests::run_message_cursor_uses_ref_and_published_at
```

Expected: PASS.

- [ ] **Step 11: Add the Tauri command**

In `src-tauri/src/analysis/mod.rs`, update imports:

```rust
use self::corpus::{list_run_snapshot_messages_page, load_run_corpus_messages, ListRunSnapshotMessagesRequest};
use self::models::{
    AnalysisChatEvent, AnalysisChatTurn, AnalysisRunDetail, AnalysisRunEvent,
    AnalysisRunMessageCursor, AnalysisRunMessagesPage, AnalysisRunRow, AnalysisRunSummary,
    AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
```

Add the command after `get_analysis_run`:

```rust
#[tauri::command]
pub async fn list_analysis_run_messages(
    handle: AppHandle,
    run_id: i64,
    after: Option<AnalysisRunMessageCursor>,
    limit: Option<i64>,
) -> AppResult<AnalysisRunMessagesPage> {
    let pool = get_pool(&handle).await?;
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM analysis_runs WHERE id = ?)",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::database)?;

    if exists == 0 {
        return Err(AppError::not_found(format!("Analysis run {run_id} not found")));
    }

    let limit = limit.unwrap_or(100).clamp(1, 500) as usize;
    list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id,
            after,
            limit,
        },
    )
    .await
    .map_err(AppError::database)
}
```

- [ ] **Step 12: Register the command with Tauri**

In `src-tauri/src/lib.rs`, add `list_analysis_run_messages` to the `use analysis::{ ... }` list:

```rust
list_analysis_run_messages,
```

Add it to `tauri::generate_handler![ ... ]` immediately after `get_analysis_run`:

```rust
list_analysis_run_messages,
```

- [ ] **Step 13: Run command-level compilation tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only
```

Expected: PASS and compile the new command imports.

- [ ] **Step 14: Commit snapshot-only API changes**

Run:

```powershell
git add src-tauri/src/analysis/models.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/mod.rs src-tauri/src/lib.rs src/lib/types/analysis.ts src/lib/api/analysis-runs.ts src/lib/api/analysis-runs.test.ts
git commit -m "feat: add analysis run snapshot message paging"
```

## Task 4: Run Part 1 Verification

**Files:**
- Verify all Part 1 files changed in Tasks 1-3.

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-state.test.ts src/lib/analysis-state.test.ts src/lib/api/analysis-runs.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 3: Run focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary_exposes_youtube_corpus_mode analysis::store::tests::map_run_detail_exposes_youtube_corpus_mode analysis::store::tests::insert_analysis_run_persists_youtube_corpus_mode analysis::corpus::tests::youtube_corpus_mode_parses_wire_values_and_defaults analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only analysis::corpus::tests::list_run_snapshot_messages_page_does_not_fall_back_to_live_source analysis::corpus::tests::run_message_cursor_uses_ref_and_published_at migrations::tests::includes_analysis_run_youtube_corpus_mode_migration
```

Expected: PASS.

- [ ] **Step 4: Run the full test suites before stopping**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Check formatting and whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 6: Commit any final fixes**

If verification required fixes, commit them:

```powershell
git add src src-tauri docs
git commit -m "test: verify analysis redesign preparatory pass"
```

Skip this commit if there are no additional changes after Tasks 1-3.

- [ ] **Step 7: Stop for review**

Run:

```powershell
git status --short
```

Expected: clean working tree.

Report:

```text
Part 1 preparatory pass is implemented and verified. Stopping before Part 2.
```

Do not begin Part 2 until the user explicitly approves continuing.

## Self-Review

- Spec coverage: this plan covers the approved preparatory implementation pass: frontend state contracts, YouTube corpus mode persistence, and snapshot-only run source access. It intentionally excludes visual layout work because that belongs to later parts.
- Placeholder scan: the plan uses concrete file paths, concrete test names, concrete commands, concrete DTO shapes, and concrete migration SQL.
- Type consistency: frontend uses `youtube_corpus_mode`, `AnalysisRunMessageCursor`, and `AnalysisRunMessagesPage`; backend uses matching serialized field names through serde and Tauri command payloads.
