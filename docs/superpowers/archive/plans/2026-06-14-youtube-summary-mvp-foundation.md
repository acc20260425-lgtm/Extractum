# YouTube Summary Prompt Pack Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Prompt Pack DB/library foundation needed before any YouTube Summary run can be created.

**Architecture:** Introduce a new `prompt_packs` Rust module and a new migration. Keep the foundation read-only at runtime: it defines schema, seeds bundled `youtube_summary` pack assets, and exposes pack/library read commands without executing LLM work.

**Tech Stack:** Rust/Tauri 2, SQLite, `sqlx`, zstd compression helpers in `src-tauri/src/compression.rs`, `serde_json`, bundled assets loaded with `include_str!`.

---

## File Structure

- Create `src-tauri/migrations/0006_prompt_pack_mvp.sql`: all Prompt Pack library, run, stage, result, projection, audit, quarantine, and YouTube projection tables from the approved spec.
- Modify `src-tauri/src/migrations.rs`: register migration version 6 and add tests for the new schema.
- Create `src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json`: bundled pack definition metadata.
- Create `src-tauri/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json`: combined stage template metadata with schema ids.
- Create `src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-input.json`: MVP stage input schema asset.
- Create `src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-output.json`: MVP stage output schema asset.
- Create `src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/canonical-result.json`: canonical Prompt Pack result schema asset pointer for this pack version.
- Create `src-tauri/src/prompt_packs/mod.rs`: module exports and public Tauri commands.
- Create `src-tauri/src/prompt_packs/library.rs`: pack seed and read-only pack-version queries.
- Create `src-tauri/src/prompt_packs/models.rs`: Rust DTOs for pack records and seed assets.
- Create `src-tauri/src/prompt_packs/seed.rs`: bundled asset loader, content hash, and idempotent DB seed.
- Create `src-tauri/src/prompt_packs/store.rs`: low-level SQL helpers for pack library rows.
- Modify `src-tauri/src/lib.rs`: register the module, run seed on startup, and expose read commands.

---

## Task 1: Migration Registration

**Files:**
- Create: `src-tauri/migrations/0006_prompt_pack_mvp.sql`
- Modify: `src-tauri/src/migrations.rs`

- [x] **Step 1: Write the failing migration tests**

Add tests in `src-tauri/src/migrations.rs`:

```rust
#[tokio::test]
async fn prompt_pack_mvp_migration_creates_library_and_run_tables() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    for table in [
        "prompt_packs",
        "prompt_pack_versions",
        "prompt_pack_stage_templates",
        "prompt_pack_schema_assets",
        "prompt_pack_runs",
        "prompt_pack_run_source_snapshots",
        "prompt_pack_run_source_origins",
        "prompt_pack_run_material_snapshots",
        "prompt_pack_stage_runs",
        "prompt_pack_stage_artifacts",
        "prompt_pack_results",
        "prompt_pack_result_ref_edges",
        "prompt_pack_youtube_videos",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("check table");
        assert_eq!(exists, 1, "missing table {table}");
    }
}

#[test]
fn build_migrations_includes_prompt_pack_mvp_version_six() {
    let versions = build_migrations()
        .iter()
        .map(|migration| migration.version)
        .collect::<Vec<_>>();

    assert_eq!(versions, vec![1, 2, 3, 4, 5, 6]);
}

#[tokio::test]
async fn prompt_pack_mvp_migration_declares_required_integrity_constraints() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    let prompt_pack_runs_sql = table_sql(&pool, "prompt_pack_runs").await;
    assert!(prompt_pack_runs_sql.contains(
        "FOREIGN KEY (pack_version_id, pack_id, pack_version, schema_version)"
    ));
    assert!(prompt_pack_runs_sql.contains(
        "REFERENCES prompt_pack_versions(id, pack_id, pack_version, schema_version)"
    ));

    let origins_sql = table_sql(&pool, "prompt_pack_run_source_origins").await;
    assert!(origins_sql.contains("FOREIGN KEY (source_snapshot_id, run_id)"));
    assert!(origins_sql.contains("FOREIGN KEY (origin_scope_id, run_id)"));
    assert!(origins_sql.contains(
        "inclusion_status <> 'included' OR source_snapshot_id IS NOT NULL"
    ));

    let material_sql = table_sql(&pool, "prompt_pack_run_material_snapshots").await;
    assert!(material_sql.contains("FOREIGN KEY (source_snapshot_id, run_id)"));

    let stages_sql = table_sql(&pool, "prompt_pack_stage_runs").await;
    assert!(stages_sql.contains("FOREIGN KEY (source_snapshot_id, run_id)"));
    assert!(stages_sql.contains("UNIQUE(id, run_id)"));

    let active_version_index: Option<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type = 'index' AND tbl_name = 'prompt_pack_versions' AND sql LIKE '%UNIQUE%' AND sql LIKE '%lifecycle_status = ''active''%'",
    )
    .fetch_optional(&pool)
    .await
    .expect("active version index lookup");
    assert!(active_version_index.is_some(), "missing active version unique index");
}

async fn table_sql(pool: &sqlx::SqlitePool, table_name: &str) -> String {
    sqlx::query_scalar::<_, String>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|error| panic!("load table sql for {table_name}: {error}"))
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests::build_migrations_includes_prompt_pack_mvp_version_six
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints
```

Expected: fail because migration 6 is not registered, the tables do not exist,
and the required FK/UNIQUE/CHECK contracts are not declared.

- [x] **Step 3: Add migration registration**

In `src-tauri/src/migrations.rs`, add:

```rust
const PROMPT_PACK_MVP_VERSION: i64 = 6;
const PROMPT_PACK_MVP_DESCRIPTION: &str = "prompt pack mvp schema";
const PROMPT_PACK_MVP_SQL: &str = include_str!("../migrations/0006_prompt_pack_mvp.sql");

fn prompt_pack_mvp_migration() -> Migration {
    Migration {
        version: PROMPT_PACK_MVP_VERSION,
        description: PROMPT_PACK_MVP_DESCRIPTION,
        sql: PROMPT_PACK_MVP_SQL,
        kind: MigrationKind::Up,
    }
}
```

Append `prompt_pack_mvp_migration()` to `build_migrations()`.

- [x] **Step 4: Add schema SQL**

Create `src-tauri/migrations/0006_prompt_pack_mvp.sql` with tables and constraints from the approved spec:

- library tables: `prompt_packs`, `prompt_pack_versions`, `prompt_pack_stage_templates`, `prompt_pack_schema_assets`;
- run tables: `prompt_pack_runs`, `prompt_pack_run_scopes`, `prompt_pack_run_source_snapshots`, `prompt_pack_run_source_origins`, `prompt_pack_run_material_snapshots`, `prompt_pack_stage_runs`, `prompt_pack_stage_artifacts`;
- result tables: `prompt_pack_results`, generic projections, YouTube projections, validation findings, audit events, quarantine artifacts;
- required composite ownership constraints:
  - `UNIQUE(id, run_id)` on run-owned parents;
  - composite FKs `(source_snapshot_id, run_id)`;
  - composite FKs `(origin_scope_id, run_id)`;
  - composite FK `(pack_version_id, pack_id, pack_version, schema_version)` from `prompt_pack_runs` to `prompt_pack_versions`;
  - composite FKs `(result_row_id, run_id)` from projection tables to `prompt_pack_results`.
- required CHECK/partial unique constraints:
  - included `prompt_pack_run_source_origins` rows require
    `source_snapshot_id IS NOT NULL`;
  - skipped and blocking origin rows may have `source_snapshot_id IS NULL`;
  - at most one `prompt_pack_versions.lifecycle_status = 'active'` row per
    `pack_id`;
  - `prompt_pack_schema_assets.schema_kind` accepts only canonical result,
    stage input, stage output, and pack data schema kinds.

- [x] **Step 5: Run migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations
```

Expected: pass.

- [x] **Step 6: Commit**

```powershell
git add src-tauri/migrations/0006_prompt_pack_mvp.sql src-tauri/src/migrations.rs
git commit -m "feat: add prompt pack mvp schema"
```

---

## Task 2: Bundled Pack Asset Files

**Files:**
- Create: `src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json`
- Create: `src-tauri/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json`
- Create: `src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-input.json`
- Create: `src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-output.json`
- Create: `src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/canonical-result.json`

- [x] **Step 1: Create pack metadata**

Write `pack.json`:

```json
{
  "pack_id": "youtube_summary",
  "pack_version": "1.0.0",
  "schema_version": "1.0",
  "display_name": "YouTube Summary",
  "origin_kind": "bundled",
  "lifecycle_status": "active",
  "default_control_preset": "standard",
  "default_evidence_mode": "standard",
  "default_include_comments": false
}
```

- [x] **Step 2: Create combined stage template metadata**

Write `stages/transcript_analysis.json`:

```json
{
  "stage_name": "youtube_summary/transcript_analysis",
  "stage_order": 20,
  "provider_family": "generic_chat",
  "input_schema_id": "stage-io/youtube_summary_transcript_analysis_input",
  "output_schema_id": "stage-io/youtube_summary_transcript_analysis_output",
  "validator_mode": "stage_output",
  "prompt_template": {
    "system": "Return strict JSON for the YouTube Summary transcript analysis stage. Use only refs from the provided registries.",
    "user": "Analyze the frozen transcript and return video_candidate, claim_candidates, evidence_fragment_candidates, and warning_candidates."
  }
}
```

- [x] **Step 3: Create minimal input schema asset**

Write `schemas/stage-io-youtube-summary-transcript-analysis-input.json` with required fields: `stage_io_version`, `schema_version`, `stage`, `pack_id`, `pack_version`, `run_id`, `source_ref_id`, `allowed_source_ref_ids`, `allowed_material_refs`, `transcript_segment_registry`, `comment_selection_policy`, `control_preset`, `evidence_mode`, and `output_language`.

- [x] **Step 4: Create minimal output schema asset**

Write `schemas/stage-io-youtube-summary-transcript-analysis-output.json` requiring:

- `stage_io_version = "1.0"`;
- `schema_version = "1.0"`;
- `stage = "youtube_summary/transcript_analysis"`;
- `video_candidate`;
- `claim_candidates`;
- `evidence_fragment_candidates`;
- `warning_candidates`.

- [x] **Step 5: Create canonical result schema asset**

Write `schemas/canonical-result.json` with a `$comment` pointing to
`docs/prompt-packs/prompt_pack_json_contract_v1_draft.md`.

Required top-level keys must match the contract/spec names:

- `schema_version`
- `result_id`
- `run_id`
- `pack_id`
- `pack_version`
- `stage`
- `created_at`
- `output_language`
- `metadata`
- `run_context`
- `outputs`
- `source_refs`
- `claims`
- `evidence`
- `warnings`
- `limitations`
- `quality_flags`
- `audit_refs`

The YouTube-specific objects live under
`outputs.pack_data.youtube_summary`, not a top-level `pack_data` key. The source
list is `source_refs`, not `sources`. Audit references are stored in
`audit_refs`, not `audit`.

- [x] **Step 6: Run asset parse and manifest check**

Run:

```powershell
$root = "src-tauri/prompt-packs/youtube_summary/1.0.0"
Get-ChildItem $root -Recurse -Filter *.json | ForEach-Object {
  Get-Content -Raw $_.FullName | ConvertFrom-Json | Out-Null
}
$pack = Get-Content -Raw "$root/pack.json" | ConvertFrom-Json
if ($pack.pack_id -ne "youtube_summary") { throw "pack_id mismatch" }
if ($pack.default_control_preset -ne "standard") { throw "default_control_preset mismatch" }
if ($pack.default_evidence_mode -ne "standard") { throw "default_evidence_mode mismatch" }
$stage = Get-Content -Raw "$root/stages/transcript_analysis.json" | ConvertFrom-Json
if ($stage.input_schema_id -ne "stage-io/youtube_summary_transcript_analysis_input") { throw "input_schema_id mismatch" }
if ($stage.output_schema_id -ne "stage-io/youtube_summary_transcript_analysis_output") { throw "output_schema_id mismatch" }
$canonical = Get-Content -Raw "$root/schemas/canonical-result.json" | ConvertFrom-Json
$required = @($canonical.required)
foreach ($key in @("source_refs", "outputs", "audit_refs")) {
  if ($required -notcontains $key) { throw "missing canonical key $key" }
}
foreach ($wrongKey in @("sources", "pack_data", "audit")) {
  if ($required -contains $wrongKey) { throw "wrong canonical key $wrongKey" }
}
```

Expected: no output and exit code 0 after every JSON asset parses and the
canonical schema uses contract-compatible top-level keys.

- [x] **Step 7: Commit**

```powershell
git add src-tauri/prompt-packs/youtube_summary/1.0.0
git commit -m "feat: add bundled youtube summary pack assets"
```

---

## Task 3: Pack Library Seed

**Files:**
- Create: `src-tauri/src/prompt_packs/mod.rs`
- Create: `src-tauri/src/prompt_packs/models.rs`
- Create: `src-tauri/src/prompt_packs/seed.rs`
- Create: `src-tauri/src/prompt_packs/store.rs`
- Create: `src-tauri/src/prompt_packs/library.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Write seed behavior tests**

Add tests under `src-tauri/src/prompt_packs/seed.rs`:

```rust
#[tokio::test]
async fn seed_youtube_summary_pack_is_idempotent() {
    let pool = test_pool_with_migrations().await;

    seed_builtin_prompt_packs_in_pool(&pool).await.expect("first seed");
    seed_builtin_prompt_packs_in_pool(&pool).await.expect("second seed");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_versions WHERE pack_id = 'youtube_summary'",
    )
    .fetch_one(&pool)
    .await
    .expect("count pack versions");

    assert_eq!(count, 1);
}

#[tokio::test]
async fn seed_youtube_summary_pack_writes_required_schema_assets() {
    let pool = test_pool_with_migrations().await;

    seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed");

    let schema_ids = sqlx::query_scalar::<_, String>(
        "SELECT schema_id FROM prompt_pack_schema_assets WHERE schema_id LIKE 'stage-io/%' ORDER BY schema_id",
    )
    .fetch_all(&pool)
    .await
    .expect("schema ids");

    assert_eq!(
        schema_ids,
        vec![
            "stage-io/youtube_summary_transcript_analysis_input".to_string(),
            "stage-io/youtube_summary_transcript_analysis_output".to_string(),
        ],
    );
}

#[tokio::test]
async fn seed_youtube_summary_pack_rejects_bundled_hash_conflict() {
    let pool = test_pool_with_migrations().await;

    seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed");

    sqlx::query(
        "UPDATE prompt_pack_versions SET content_hash = 'sha384-conflict' WHERE pack_id = 'youtube_summary' AND pack_version = '1.0.0'",
    )
    .execute(&pool)
    .await
    .expect("mutate content hash");

    let error = seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect_err("hash conflict rejected");

    assert!(error.to_string().contains("hash conflict"));
}

#[tokio::test]
async fn seed_youtube_summary_pack_rejects_user_collision() {
    let pool = test_pool_with_migrations().await;

    sqlx::query(
        r#"
        INSERT INTO prompt_packs (pack_id, display_name, is_builtin, created_at, updated_at)
        VALUES ('youtube_summary', 'User YouTube Summary', 0, 1, 1)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert colliding pack");

    sqlx::query(
        r#"
        INSERT INTO prompt_pack_versions (
            pack_id, pack_version, schema_version, origin_kind, lifecycle_status,
            content_hash, bundled_source_path, created_at, updated_at
        )
        VALUES (
            'youtube_summary', '1.0.0', '1.0', 'user', 'draft',
            'sha384-user-draft', NULL, 1, 1
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert colliding user draft");

    let error = seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect_err("user collision rejected");

    assert!(error.to_string().contains("collision"));
}

#[tokio::test]
async fn seed_youtube_summary_pack_preserves_unknown_newer_bundled_version() {
    let pool = test_pool_with_migrations().await;

    sqlx::query(
        r#"
        INSERT INTO prompt_packs (pack_id, display_name, is_builtin, created_at, updated_at)
        VALUES ('youtube_summary', 'YouTube Summary', 1, 1, 1)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert pack");

    sqlx::query(
        r#"
        INSERT INTO prompt_pack_versions (
            pack_id, pack_version, schema_version, origin_kind, lifecycle_status,
            content_hash, bundled_source_path, created_at, updated_at
        )
        VALUES (
            'youtube_summary', '9.9.9', '1.0', 'bundled', 'archived',
            'sha384-future', 'future-bundle', 1, 1
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert future bundled version");

    seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed current bundle");

    let future_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_versions WHERE pack_id = 'youtube_summary' AND pack_version = '9.9.9'",
    )
    .fetch_one(&pool)
    .await
    .expect("future version count");

    assert_eq!(future_count, 1);
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::seed
```

Expected: fail because `prompt_packs` module and seed functions do not exist.

- [x] **Step 3: Implement asset loader and content hash**

In `seed.rs`, load bundled JSON with `include_str!`, compute a SHA-384 hex content hash over normalized asset bytes, and compress JSON payloads with `crate::compression::compress_text`.

- [x] **Step 4: Implement idempotent seed SQL**

Seed rules:

- insert `prompt_packs` if missing;
- insert `prompt_pack_versions` if `(pack_id, pack_version)` is missing;
- if bundled `(pack_id, pack_version)` exists with same content hash, update only `last_seeded_at`;
- if bundled `(pack_id, pack_version)` exists with a different content hash, return `AppError::validation`;
- if `(pack_id, pack_version)` exists with `origin_kind = 'user'`, return
  `AppError::validation` and do not overwrite the draft;
- activation is explicit from `pack.json`; seed must preserve unknown newer
  bundled versions and must not delete or archive versions absent from the
  current bundle;
- insert or refresh stage templates and schema assets only for the matching immutable bundled version.

- [x] **Step 5: Register startup seed**

In `src-tauri/src/lib.rs`:

```rust
mod prompt_packs;
use prompt_packs::{get_prompt_pack_library, seed_builtin_prompt_packs};
```

Inside `setup`, run the seed in the existing non-async setup closure style before
runtime cleanup tasks that might read prompt-pack state:

```rust
let prompt_pack_seed_handle = app.handle().clone();
tauri::async_runtime::spawn(async move {
    if let Err(error) = seed_builtin_prompt_packs(prompt_pack_seed_handle).await {
        eprintln!("Prompt Pack seed failed: {error}");
    }
});
```

MVP policy: seed failure is logged and the app continues to start. Prompt Pack
commands must return a validation/internal error if the required active
`youtube_summary` pack version is unavailable. Do not silently create runs
against missing seed assets.

- [x] **Step 6: Run seed tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::seed
```

Expected: pass.

- [x] **Step 7: Commit**

```powershell
git add src-tauri/src/prompt_packs src-tauri/src/lib.rs
git commit -m "feat: seed youtube summary prompt pack"
```

---

## Task 4: Read-Only Library Commands

**Files:**
- Modify: `src-tauri/src/prompt_packs/library.rs`
- Modify: `src-tauri/src/prompt_packs/models.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/types/prompt-packs.ts`
- Create: `src/lib/api/prompt-packs.ts`
- Create: `src/lib/api/prompt-packs.test.ts`

- [x] **Step 1: Add backend command tests**

Add Rust tests proving `get_prompt_pack_library_in_pool` returns
`youtube_summary` with:

- active `1.0.0`;
- `schema_version = "1.0"`;
- `default_control_preset = "standard"`;
- `default_evidence_mode = "standard"`;
- `default_include_comments = false`;
- the combined stage template;
- both stage schema ids and their schema kinds.

- [x] **Step 2: Implement backend command**

Expose:

```rust
#[tauri::command]
pub async fn get_prompt_pack_library(handle: AppHandle) -> AppResult<PromptPackLibraryDto>
```

DTO shape:

```rust
pub struct PromptPackLibraryDto {
    pub packs: Vec<PromptPackDto>,
}

pub struct PromptPackDto {
    pub pack_id: String,
    pub display_name: String,
    pub active_version: Option<PromptPackVersionDto>,
}

pub struct PromptPackVersionDto {
    pub pack_version_id: i64,
    pub pack_version: String,
    pub schema_version: String,
    pub lifecycle_status: String,
    pub default_control_preset: String,
    pub default_evidence_mode: String,
    pub default_include_comments: bool,
    pub stages: Vec<PromptPackStageTemplateDto>,
    pub schema_assets: Vec<PromptPackSchemaAssetDto>,
}

pub struct PromptPackStageTemplateDto {
    pub stage_name: String,
    pub stage_order: i64,
    pub provider_family: String,
    pub input_schema_id: String,
    pub output_schema_id: String,
}

pub struct PromptPackSchemaAssetDto {
    pub schema_id: String,
    pub schema_kind: String,
    pub content_hash: String,
}
```

- [x] **Step 3: Register command**

Add `get_prompt_pack_library` to `tauri::generate_handler!` in `src-tauri/src/lib.rs`.

- [x] **Step 4: Add frontend wrapper test**

In `src/lib/api/prompt-packs.test.ts`, mock `@tauri-apps/api/core` and verify:

```ts
await getPromptPackLibrary();
expect(invoke).toHaveBeenCalledWith("get_prompt_pack_library");
```

- [x] **Step 5: Implement frontend wrapper**

Create `src/lib/types/prompt-packs.ts`:

```ts
export interface PromptPackLibrary {
  packs: PromptPack[];
}

export interface PromptPack {
  packId: string;
  displayName: string;
  activeVersion: PromptPackVersion | null;
}

export interface PromptPackVersion {
  packVersionId: number;
  packVersion: string;
  schemaVersion: string;
  lifecycleStatus: string;
  defaultControlPreset: string;
  defaultEvidenceMode: string;
  defaultIncludeComments: boolean;
  stages: PromptPackStageTemplate[];
  schemaAssets: PromptPackSchemaAsset[];
}

export interface PromptPackStageTemplate {
  stageName: string;
  stageOrder: number;
  providerFamily: string;
  inputSchemaId: string;
  outputSchemaId: string;
}

export interface PromptPackSchemaAsset {
  schemaId: string;
  schemaKind: string;
  contentHash: string;
}
```

Create `src/lib/api/prompt-packs.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { PromptPackLibrary } from "$lib/types/prompt-packs";

export function getPromptPackLibrary() {
  return invoke<PromptPackLibrary>("get_prompt_pack_library");
}
```

- [x] **Step 6: Run tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
npm test -- --run src/lib/api/prompt-packs.test.ts
```

Expected: pass.

- [x] **Step 7: Commit**

```powershell
git add src-tauri/src/prompt_packs src-tauri/src/lib.rs src/lib/types/prompt-packs.ts src/lib/api/prompt-packs.ts src/lib/api/prompt-packs.test.ts
git commit -m "feat: expose prompt pack library"
```

---

## Plan Acceptance

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
npm test -- --run src/lib/api/prompt-packs.test.ts
git status --short
```

Expected:

- migration and seed tests pass;
- frontend API wrapper test passes;
- working tree is clean after the final task commit.
