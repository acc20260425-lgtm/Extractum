# YouTube Summary Transcript Artifact Atomicity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make successful YouTube Summary transcript attempts write `metrics`, `intermediate_entities`, `parsed_output`, and the terminal stage-status update in one SQL transaction.

**Architecture:** Keep `prompt_input` and `raw_output` outside the success transaction because they are diagnostic artifacts for the attempt itself. After parsing, validation, and graph construction succeed in memory, open one SQL transaction and write `metrics`, `intermediate_entities`, `parsed_output`, and the `stage_status = 'succeeded'`/repaired update together. Add transaction-aware artifact insert helpers so existing callers can keep using the pool helper while transcript success paths can use a transaction.

**Tech Stack:** Rust, SQLx, SQLite, Tauri backend, existing Prompt Pack artifact tables.

---

## Scope

This plan covers transcript-analysis success artifacts only:

- first-attempt `execute_transcript_analysis_stage_with_completion`;
- repaired-attempt `execute_transcript_analysis_stage_repair_completion`.

This plan does not change:

- synthesis-stage artifact persistence;
- quarantine artifact persistence;
- final result persistence;
- database schema.

## Files

- Modify `src-tauri/src/prompt_packs/stage_io.rs`
  - Add a transaction-aware artifact insert helper.
  - Keep `insert_stage_artifact_in_pool(...)` as the public pool wrapper for existing callers.
- Modify `src-tauri/src/prompt_packs/youtube_summary/entities.rs`
  - Add a transaction-aware `insert_intermediate_entities_artifact_in_transaction(...)`.
  - Keep the pool helper for existing tests and call sites.
- Modify `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
  - Use one transaction for transcript first-attempt success artifact writes and `stage_status = 'succeeded'`.
- Modify `src-tauri/src/prompt_packs/json_repair.rs`
  - Use one transaction for transcript repaired-attempt success artifact writes and `mark_stage_repaired`.
- Modify `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`
  - Add focused rollback tests for first-attempt and repaired-attempt duplicate artifact insertion failures.

---

### Task 1: Add Transaction-Aware Artifact Insert Helpers

**Files:**
- Modify: `src-tauri/src/prompt_packs/stage_io.rs`

- [ ] **Step 1: Add SQLx transaction imports**

Change the imports at the top of `stage_io.rs` from:

```rust
use sqlx::SqlitePool;
```

to:

```rust
use sqlx::{Sqlite, SqlitePool, Transaction};
```

- [ ] **Step 2: Extract artifact insert body into a transaction helper**

Replace the current `insert_stage_artifact_in_pool(...)` function with this pair:

```rust
pub(crate) async fn insert_stage_artifact_in_pool(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    artifact_kind: &str,
    attempt_number: i64,
    artifact_index: i64,
    content: &str,
) -> AppResult<()> {
    insert_stage_artifact_with_executor(
        pool,
        run_id,
        stage_run_id,
        artifact_kind,
        attempt_number,
        artifact_index,
        content,
    )
    .await
}

pub(crate) async fn insert_stage_artifact_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
    stage_run_id: i64,
    artifact_kind: &str,
    attempt_number: i64,
    artifact_index: i64,
    content: &str,
) -> AppResult<()> {
    insert_stage_artifact_with_executor(
        &mut **tx,
        run_id,
        stage_run_id,
        artifact_kind,
        attempt_number,
        artifact_index,
        content,
    )
    .await
}

async fn insert_stage_artifact_with_executor<'e, E>(
    executor: E,
    run_id: i64,
    stage_run_id: i64,
    artifact_kind: &str,
    attempt_number: i64,
    artifact_index: i64,
    content: &str,
) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let content_hash = format!("sha384-{}", sha384_hex(content.as_bytes()));
    let content_zstd = compress_text(content).map_err(AppError::internal)?;
    let created_at = crate::time::now_rfc3339_utc();
    sqlx::query(
        "INSERT INTO prompt_pack_stage_artifacts (
            run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
            content_type, content_hash, content_zstd, redaction_state, created_at
         )
         VALUES (?, ?, ?, ?, ?, 'application/json', ?, ?, 'none', ?)",
    )
    .bind(run_id)
    .bind(stage_run_id)
    .bind(artifact_kind)
    .bind(attempt_number)
    .bind(artifact_index)
    .bind(content_hash)
    .bind(content_zstd)
    .bind(created_at)
    .execute(executor)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [ ] **Step 3: Run the existing stage_io test**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib insert_stage_artifact_uses_current_time -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Commit**

```powershell
git add src-tauri/src/prompt_packs/stage_io.rs
git commit -m "refactor: add transactional stage artifact insert"
```

---

### Task 2: Add Transaction-Aware Intermediate Graph Artifact Insert

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/entities.rs`

- [ ] **Step 1: Add SQLx transaction imports**

Change the SQLx import in `entities.rs` from:

```rust
use sqlx::SqlitePool;
```

to:

```rust
use sqlx::{Sqlite, SqlitePool, Transaction};
```

Change the stage IO import from:

```rust
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, insert_stage_artifact_in_pool,
};
```

to:

```rust
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, insert_stage_artifact_in_pool,
    insert_stage_artifact_in_transaction,
};
```

- [ ] **Step 2: Add transaction helper and keep pool wrapper**

Replace the current `insert_intermediate_entities_artifact(...)` with:

```rust
pub(crate) async fn insert_intermediate_entities_artifact(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    graph: &serde_json::Value,
    attempt_number: i64,
) -> AppResult<()> {
    let content = serde_json::to_string(graph)
        .map_err(|error| AppError::internal(format!("serialize intermediate entities: {error}")))?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        INTERMEDIATE_ENTITIES_ARTIFACT_KIND,
        attempt_number,
        5,
        &content,
    )
    .await
}

pub(crate) async fn insert_intermediate_entities_artifact_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
    stage_run_id: i64,
    graph: &serde_json::Value,
    attempt_number: i64,
) -> AppResult<()> {
    let content = serde_json::to_string(graph)
        .map_err(|error| AppError::internal(format!("serialize intermediate entities: {error}")))?;
    insert_stage_artifact_in_transaction(
        tx,
        run_id,
        stage_run_id,
        INTERMEDIATE_ENTITIES_ARTIFACT_KIND,
        attempt_number,
        5,
        &content,
    )
    .await
}
```

- [ ] **Step 3: Run focused entity/output tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib intermediate_entities -- --nocapture
```

Expected: PASS. If this filter matches more tests than expected, all matched tests must pass.

- [ ] **Step 4: Commit**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/entities.rs
git commit -m "refactor: add transactional intermediate graph insert"
```

---

### Task 3: Add Failing Rollback Tests For Transcript Success Artifacts

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`

- [ ] **Step 1: Add first-attempt rollback test**

Add this test near the existing transcript artifact tests:

```rust
#[tokio::test]
async fn transcript_success_artifacts_roll_back_when_parsed_insert_fails() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
        &pool,
        1,
        stage_id,
        "parsed_output",
        1,
        3,
        r#"{"preexisting":true}"#,
    )
    .await
    .expect("seed duplicate parsed artifact");

    let error = execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
    )
    .await
    .expect_err("duplicate parsed artifact should fail success transaction");

    assert!(error.message.contains("UNIQUE") || error.message.contains("unique"));

    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(artifacts.contains(&("prompt_input".to_string(), 1, 1)));
    assert!(artifacts.contains(&("raw_output".to_string(), 1, 2)));
    assert!(artifacts.contains(&("parsed_output".to_string(), 1, 3)));
    assert!(!artifacts.contains(&("metrics".to_string(), 1, 4)));
    assert!(!artifacts.contains(&("intermediate_entities".to_string(), 1, 5)));

    let status: String =
        sqlx::query_scalar("SELECT stage_status FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_id)
            .fetch_one(&pool)
            .await
            .expect("stage status");
    assert_eq!(status, "running");
}
```

- [ ] **Step 2: Add repaired-attempt rollback test**

Add this test near `repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt`:

```rust
#[tokio::test]
async fn repaired_transcript_success_artifacts_roll_back_when_parsed_insert_fails() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
        &pool,
        1,
        stage_id,
        "parsed_output",
        2,
        3,
        r#"{"preexisting":true}"#,
    )
    .await
    .expect("seed duplicate repaired parsed artifact");

    let error = crate::prompt_packs::json_repair::execute_transcript_analysis_stage_repair_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
        2,
    )
    .await
    .expect_err("duplicate repaired parsed artifact should fail success transaction");

    assert!(error.message.contains("UNIQUE") || error.message.contains("unique"));

    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(artifacts.contains(&("raw_output".to_string(), 2, 2)));
    assert!(artifacts.contains(&("parsed_output".to_string(), 2, 3)));
    assert!(!artifacts.contains(&("metrics".to_string(), 2, 4)));
    assert!(!artifacts.contains(&("intermediate_entities".to_string(), 2, 5)));

    let status: String =
        sqlx::query_scalar("SELECT stage_status FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_id)
            .fetch_one(&pool)
            .await
            .expect("stage status");
    assert_eq!(status, "running");
}
```

- [ ] **Step 3: Run first rollback test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib transcript_success_artifacts_roll_back_when_parsed_insert_fails -- --nocapture
```

Expected before implementation: FAIL because `metrics #4` and `intermediate_entities #5` are still present after the duplicate `parsed_output #3` insert fails.

- [ ] **Step 4: Run repaired rollback test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib repaired_transcript_success_artifacts_roll_back_when_parsed_insert_fails -- --nocapture
```

Expected before implementation: FAIL for the same reason on attempt `2`.

- [ ] **Step 5: Commit failing tests**

Only do this if the repository accepts red TDD commits. If not, skip this commit and include tests in Task 4's green commit.

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs
git commit -m "test: cover transcript artifact rollback"
```

---

### Task 4: Make Transcript Success Artifact Writes Atomic

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
- Modify: `src-tauri/src/prompt_packs/json_repair.rs`

- [ ] **Step 1: Update imports in `outputs.rs`**

Change:

```rust
use super::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact, load_required_allowed_refs_for_live_synthesis,
};
```

to:

```rust
use super::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact, insert_intermediate_entities_artifact_in_transaction,
    load_required_allowed_refs_for_live_synthesis,
};
```

Change:

```rust
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
    SYNTHESIS_OUTPUT_SCHEMA_ID, TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
};
```

to:

```rust
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
    insert_stage_artifact_in_transaction, SYNTHESIS_OUTPUT_SCHEMA_ID,
    TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
};
```

- [ ] **Step 2: Replace first-attempt transcript success writes in `outputs.rs`**

In `execute_transcript_analysis_stage_with_completion(...)`, replace the block that inserts `metrics`, `intermediate_entities`, `parsed_output`, and then runs the `UPDATE prompt_pack_stage_runs SET stage_status = 'succeeded'` query with:

```rust
let parsed_json = serde_json::to_string(&parsed)
    .map_err(|error| AppError::internal(format!("serialize parsed output: {error}")))?;
let mut tx = pool.begin().await.map_err(AppError::database)?;
insert_stage_artifact_in_transaction(
    &mut tx,
    run_id,
    stage_run_id,
    "metrics",
    1,
    4,
    &metrics.to_string(),
)
.await?;
insert_intermediate_entities_artifact_in_transaction(
    &mut tx,
    run_id,
    stage_run_id,
    &intermediate_graph,
    1,
)
.await?;
insert_stage_artifact_in_transaction(
    &mut tx,
    run_id,
    stage_run_id,
    "parsed_output",
    1,
    3,
    &parsed_json,
)
.await?;
sqlx::query(
    "UPDATE prompt_pack_stage_runs
     SET stage_status = 'succeeded', completed_at = ?, updated_at = ?
     WHERE id = ?",
)
.bind(now_string())
.bind(now_string())
.bind(stage_run_id)
.execute(&mut **tx)
.await
.map_err(AppError::database)?;
tx.commit().await.map_err(AppError::database)?;
```

- [ ] **Step 3: Update imports in `json_repair.rs`**

Change:

```rust
use super::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
    SYNTHESIS_OUTPUT_SCHEMA_ID, TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
};
```

to:

```rust
use super::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
    insert_stage_artifact_in_transaction, SYNTHESIS_OUTPUT_SCHEMA_ID,
    TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
};
```

Change:

```rust
use super::youtube_summary::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact, load_required_allowed_refs_for_live_synthesis,
};
```

to:

```rust
use super::youtube_summary::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact_in_transaction,
    load_required_allowed_refs_for_live_synthesis,
};
```

- [ ] **Step 4: Make repaired transcript success writes transactional**

In `execute_transcript_analysis_stage_repair_completion(...)`, replace the block that inserts `metrics`, `intermediate_entities`, `parsed_output`, and calls `mark_stage_repaired(...)` with:

```rust
let parsed_json = serde_json::to_string(&parsed)
    .map_err(|error| AppError::internal(format!("serialize parsed output: {error}")))?;
let mut tx = pool.begin().await.map_err(AppError::database)?;
insert_stage_artifact_in_transaction(
    &mut tx,
    run_id,
    stage_run_id,
    "metrics",
    attempt_number,
    4,
    &metrics.to_string(),
)
.await?;
insert_intermediate_entities_artifact_in_transaction(
    &mut tx,
    run_id,
    stage_run_id,
    &intermediate_graph,
    attempt_number,
)
.await?;
insert_stage_artifact_in_transaction(
    &mut tx,
    run_id,
    stage_run_id,
    "parsed_output",
    attempt_number,
    3,
    &parsed_json,
)
.await?;
mark_stage_repaired_in_transaction(&mut tx, stage_run_id).await?;
tx.commit().await.map_err(AppError::database)?;
Ok(())
```

- [ ] **Step 5: Replace `mark_stage_repaired(...)` with pool and transaction helpers**

Replace the existing `mark_stage_repaired(...)` function with:

```rust
async fn mark_stage_repaired(pool: &SqlitePool, stage_run_id: i64) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    mark_stage_repaired_in_transaction(&mut tx, stage_run_id).await?;
    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

async fn mark_stage_repaired_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    stage_run_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'succeeded',
             error_message = NULL,
             latest_message = 'Repaired JSON output',
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

This keeps synthesis repair behavior unchanged because `execute_synthesis_stage_repair_completion(...)` can keep calling the pool wrapper.

- [ ] **Step 6: Run rollback tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib transcript_success_artifacts_roll_back_when_parsed_insert_fails -- --nocapture
cargo test --manifest-path src-tauri\Cargo.toml --lib repaired_transcript_success_artifacts_roll_back_when_parsed_insert_fails -- --nocapture
```

Expected: both PASS.

- [ ] **Step 7: Run existing transcript artifact tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib execute_transcript_analysis_stage_persists_intermediate_entities_artifact -- --nocapture
cargo test --manifest-path src-tauri\Cargo.toml --lib repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt -- --nocapture
cargo test --manifest-path src-tauri\Cargo.toml --lib malformed_intermediate_candidates_are_quarantined_without_graph_artifact -- --nocapture
cargo test --manifest-path src-tauri\Cargo.toml --lib repair_graph_build_failure_does_not_write_repaired_parsed_output -- --nocapture
```

Expected: all PASS.

- [ ] **Step 8: Commit implementation**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/outputs.rs src-tauri/src/prompt_packs/json_repair.rs src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs
git commit -m "fix: make transcript success artifacts atomic"
```

---

### Task 5: Broad Verification

**Files:**
- No code changes expected.

- [ ] **Step 1: Format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml --check
```

Expected: PASS.

- [ ] **Step 2: Focused YouTube Summary tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: PASS.

- [ ] **Step 3: Broad Prompt Pack tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
```

Expected: PASS.

- [ ] **Step 4: Compile check**

Run:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

Expected: PASS, allowing existing dead-code warnings.

- [ ] **Step 5: Git whitespace check**

Run:

```powershell
git diff --check
```

Expected: PASS.

- [ ] **Step 6: Final status**

Run:

```powershell
git status --short
```

Expected: only intentional changes or the pre-existing untracked `tmp/`.

---

## Self-Review

- Spec coverage:
  - First-attempt transcript success artifacts are transactional in Task 4.
  - Repaired-attempt transcript success artifacts are transactional in Task 4.
  - Existing diagnostic `prompt_input`, `raw_output`, and `repair_input` remain outside the success transaction by design.
  - Existing synthesis behavior is intentionally unchanged.
- Placeholder scan:
  - No `TBD`, `TODO`, or undefined behavior placeholders.
- Type consistency:
  - `insert_stage_artifact_in_transaction(...)` and `insert_intermediate_entities_artifact_in_transaction(...)` both accept `&mut Transaction<'_, Sqlite>`.
  - Existing pool helpers remain available for current non-transactional callers.
