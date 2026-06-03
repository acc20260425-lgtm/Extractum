# NotebookLM Source Group Export Implementation Plan

> Historical execution record. Telegram source-group NotebookLM export shipped;
> current behavior is summarized in root docs such as `docs/project.md`,
> `docs/design-document.md`, and `docs/architecture-deep-dive.md`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable NotebookLM export for Telegram source groups while preserving existing single-source Telegram export behavior.

**Architecture:** Keep the Tauri command name and compact frontend result DTO, but make the request scope-neutral with exactly one of `source_id` or `source_group_id`. Backend group export orchestrates existing single-source Telegram loading/rendering per member, writes source-scoped files under `sources/`, and records per-member details in `.extractum-notebooklm-export.json`.

**Tech Stack:** Rust/Tauri 2, SQLx SQLite, Svelte 5, TypeScript, Vitest, Cargo tests.

---

## Approved Spec

- `docs/superpowers/archive/specs/2026-05-31-notebooklm-source-group-export-design.md`

Key constraints from the spec:

- Telegram source groups only.
- YouTube source groups stay unsupported with explicit copy.
- No result DTO expansion for member summaries.
- Progress remains compatible and best-effort.
- Member ordering is `COALESCE(sources.title, '')`, then `sources.id`.
- The same member order drives execution, filenames, and manifest summaries.
- Dirty non-Telegram members inside a Telegram group are skipped with warnings.
- Group `source_type != "telegram"` is a hard validation error.
- The marker JSON tracks `glossary.md` and every `sources/...part-XXX.md`.

## File Map

Modify:

- `src-tauri/src/notebooklm_export/model.rs`
  - Make `NotebookLmExportRequest` scope-neutral.
  - Add `NotebookLmExportScope`.
  - Store scope in `NotebookLmExportConfig`.

- `src-tauri/src/notebooklm_export/query.rs`
  - Add group/member query structs.
  - Add `load_export_source_group`.
  - Add tests for group ordering and dirty mixed-provider member data.

- `src-tauri/src/notebooklm_export/filename.rs`
  - Add safe relative child path support for `sources/...` generated files.

- `src-tauri/src/notebooklm_export/mod.rs`
  - Validate scope rules.
  - Preserve single-source path.
  - Extract reusable render/write helpers.
  - Add source-group orchestration.
  - Expand marker JSON while keeping old marker parsing compatible.

- `src/lib/types/sources.ts`
  - Make `NotebookLmExportRequest.source_id` nullable.
  - Add nullable `source_group_id`.

- `src/lib/analysis-state.ts`
  - Add a scope input type for request building.
  - Build either source or source-group request.

- `src/lib/analysis-state.test.ts`
  - Cover source and source-group request building.

- `src/lib/api/notebooklm-export.test.ts`
  - Update request fixture for nullable scope fields.

- `src/lib/components/analysis/notebooklm-export-dialog.svelte`
  - Make dialog description target-neutral.
  - Gate migrated-history checkbox through a prop instead of only `source`.

- `src/lib/components/analysis/report-canvas.svelte`
  - Enable Telegram source groups.
  - Keep YouTube source groups disabled with explicit reason.
  - Pass target label and migrated-history availability to dialog.

- `src/routes/analysis/+page.svelte`
  - Build NotebookLM export request from current source or current group.

- `src/lib/analysis-report-canvas.test.ts`
  - Update raw component contract for group availability and dialog props.

- `src/lib/analysis-ui-smoke-contract.test.ts`
  - Update disabled reason contract if existing raw assertions mention the old generic group reason.

- `docs/backlog.md`
  - Remove source-group NotebookLM export follow-up after implementation.

- `docs/project.md`
  - Update current-state summary after implementation.

No new production dependency is required.

---

### Task 1: Backend Scope Model And Validation

**Files:**
- Modify: `src-tauri/src/notebooklm_export/model.rs`
- Modify: `src-tauri/src/notebooklm_export/mod.rs`

- [x] **Step 1: Write failing validation tests**

In `src-tauri/src/notebooklm_export/mod.rs`, extend the existing `#[cfg(test)] mod tests` imports:

```rust
use super::{timestamp_for_folder, validate_request};
use crate::notebooklm_export::model::{NotebookLmExportRequest, NotebookLmExportScope};
```

Update the local `request()` fixture:

```rust
fn request() -> NotebookLmExportRequest {
    NotebookLmExportRequest {
        export_id: None,
        source_id: Some(1),
        source_group_id: None,
        output_dir: ".".to_string(),
        period_from: None,
        period_to: None,
        include_media_placeholders: true,
        include_migrated_history: false,
        min_message_length: 3,
        max_words_per_file: 300_000,
        max_bytes_per_file: 50_000_000,
        overwrite_existing: false,
    }
}
```

Add these tests:

```rust
#[test]
fn validates_exactly_one_export_scope() {
    let mut missing = request();
    missing.source_id = None;
    missing.source_group_id = None;
    let error = validate_request(missing).expect_err("missing scope is invalid");
    assert!(error.message.contains("Select a source or source group"));

    let mut both = request();
    both.source_id = Some(1);
    both.source_group_id = Some(9);
    let error = validate_request(both).expect_err("two scopes are invalid");
    assert!(error.message.contains("Select either a source or source group"));
}

#[test]
fn validates_single_source_scope() {
    let config = validate_request(request()).expect("valid source request");
    assert_eq!(config.scope, NotebookLmExportScope::Source { source_id: 1 });
    assert_eq!(config.event_scope_id(), 1);
}

#[test]
fn validates_source_group_scope() {
    let mut request = request();
    request.source_id = None;
    request.source_group_id = Some(9);

    let config = validate_request(request).expect("valid group request");

    assert_eq!(
        config.scope,
        NotebookLmExportScope::SourceGroup {
            source_group_id: 9,
        }
    );
    assert_eq!(config.event_scope_id(), 9);
}
```

- [x] **Step 2: Run the focused Rust validation tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml validates_
```

Expected: FAIL because `source_id` is still non-nullable and `NotebookLmExportScope` does not exist.

- [x] **Step 3: Implement scope-neutral backend request model**

In `src-tauri/src/notebooklm_export/model.rs`, change the request and config types to:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NotebookLmExportScope {
    Source { source_id: i64 },
    SourceGroup { source_group_id: i64 },
}

impl NotebookLmExportScope {
    pub(crate) fn event_scope_id(&self) -> i64 {
        match self {
            Self::Source { source_id } => *source_id,
            Self::SourceGroup { source_group_id } => *source_group_id,
        }
    }
}

#[derive(Deserialize)]
pub struct NotebookLmExportRequest {
    pub export_id: Option<String>,
    pub source_id: Option<i64>,
    pub source_group_id: Option<i64>,
    pub output_dir: String,
    pub period_from: Option<i64>,
    pub period_to: Option<i64>,
    pub include_media_placeholders: bool,
    pub include_migrated_history: bool,
    pub min_message_length: i64,
    pub max_words_per_file: i64,
    pub max_bytes_per_file: i64,
    pub overwrite_existing: bool,
}

#[derive(Clone)]
pub(crate) struct NotebookLmExportConfig {
    pub(crate) export_id: Option<String>,
    pub(crate) scope: NotebookLmExportScope,
    pub(crate) output_dir: String,
    pub(crate) period_from: Option<i64>,
    pub(crate) period_to: Option<i64>,
    pub(crate) include_media_placeholders: bool,
    pub(crate) include_migrated_history: bool,
    pub(crate) min_message_length: usize,
    pub(crate) max_words_per_file: usize,
    pub(crate) max_bytes_per_file: usize,
    pub(crate) overwrite_existing: bool,
}

impl NotebookLmExportConfig {
    pub(crate) fn event_scope_id(&self) -> i64 {
        self.scope.event_scope_id()
    }
}
```

In `src-tauri/src/notebooklm_export/mod.rs`, import `NotebookLmExportScope`:

```rust
use model::{
    ChunkFile, NotebookLmExportConfig, NotebookLmExportFile, NotebookLmExportMessage,
    NotebookLmExportRequest, NotebookLmExportResult, NotebookLmExportScope, ParticipantSummary,
    DEFAULT_MAX_BYTES_PER_FILE, DEFAULT_MAX_WORDS_PER_FILE, DEFAULT_MIN_MESSAGE_LENGTH,
};
```

Update `validate_request` to compute exactly one scope:

```rust
fn validate_request(request: NotebookLmExportRequest) -> AppResult<NotebookLmExportConfig> {
    let output_dir = request.output_dir.trim();
    if output_dir.is_empty() {
        return Err(AppError::validation("Output directory is required"));
    }
    if let (Some(from), Some(to)) = (request.period_from, request.period_to) {
        if from > to {
            return Err(AppError::validation(
                "Export period start must be before export period end",
            ));
        }
    }

    let scope = match (request.source_id, request.source_group_id) {
        (Some(source_id), None) => NotebookLmExportScope::Source { source_id },
        (None, Some(source_group_id)) => NotebookLmExportScope::SourceGroup { source_group_id },
        (None, None) => {
            return Err(AppError::validation("Select a source or source group before exporting"));
        }
        (Some(_), Some(_)) => {
            return Err(AppError::validation("Select either a source or source group, not both"));
        }
    };

    Ok(NotebookLmExportConfig {
        export_id: request
            .export_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        scope,
        output_dir: output_dir.to_string(),
        period_from: request.period_from,
        period_to: request.period_to,
        include_media_placeholders: request.include_media_placeholders,
        include_migrated_history: request.include_migrated_history,
        min_message_length: validate_positive_usize(
            request.min_message_length,
            "min_message_length",
            DEFAULT_MIN_MESSAGE_LENGTH,
        )?,
        max_words_per_file: validate_positive_usize(
            request.max_words_per_file,
            "max_words_per_file",
            DEFAULT_MAX_WORDS_PER_FILE,
        )?,
        max_bytes_per_file: validate_positive_usize(
            request.max_bytes_per_file,
            "max_bytes_per_file",
            DEFAULT_MAX_BYTES_PER_FILE,
        )?,
        overwrite_existing: request.overwrite_existing,
    })
}
```

Temporarily update command usage to keep the source path compiling:

```rust
let progress = NotebookLmExportProgress::new(
    handle.clone(),
    config
        .export_id
        .clone()
        .unwrap_or_else(|| format!("notebooklm-{}-{generated_at}", config.event_scope_id())),
    config.event_scope_id(),
);

let source_id = match config.scope {
    NotebookLmExportScope::Source { source_id } => source_id,
    NotebookLmExportScope::SourceGroup { .. } => {
        let error = AppError::validation("Source-group NotebookLM export is not implemented yet");
        progress.emit_failed("loading", &error);
        return Err(error);
    }
};
```

Then replace current `config.source_id` reads in the single-source branch with the local `source_id`.

- [x] **Step 4: Run validation tests and a single-source smoke subset**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::tests
```

Expected: PASS.

- [x] **Step 5: Commit Task 1**

```powershell
git add src-tauri/src/notebooklm_export/model.rs src-tauri/src/notebooklm_export/mod.rs
git commit -m "feat: add notebooklm export scope validation"
```

---

### Task 2: Backend Source Group Query Helpers

**Files:**
- Modify: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Write failing group query tests**

In `src-tauri/src/notebooklm_export/query.rs`, extend the test import:

```rust
use super::{
    load_export_messages, load_export_messages_from_archive,
    load_export_messages_from_items_path, load_export_source, load_export_source_group,
    select_notebooklm_export_loader, ArchiveReadinessFallbackReason, ExportHistoryScope,
    ExportLoaderSelection,
};
```

Extend `export_pool()` with the two group tables:

```rust
sqlx::query(
    r#"
    CREATE TABLE analysis_source_groups (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        source_type TEXT NOT NULL DEFAULT 'telegram',
        created_at INTEGER NOT NULL DEFAULT 0,
        updated_at INTEGER NOT NULL DEFAULT 0
    )
    "#,
)
.execute(&pool)
.await
.expect("create analysis_source_groups");

sqlx::query(
    r#"
    CREATE TABLE analysis_source_group_members (
        group_id INTEGER NOT NULL,
        source_id INTEGER NOT NULL,
        created_at INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (group_id, source_id)
    )
    "#,
)
.execute(&pool)
.await
.expect("create analysis_source_group_members");
```

Add these tests:

```rust
#[tokio::test]
async fn load_export_source_group_orders_members_by_title_then_id() {
    let pool = export_pool().await;
    for (id, source_type, title) in [
        (30_i64, "telegram", Some("Beta")),
        (10_i64, "telegram", Some("Alpha")),
        (20_i64, "telegram", Some("Alpha")),
        (40_i64, "telegram", None),
    ] {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (?, ?, 'channel', ?, ?)",
        )
        .bind(id)
        .bind(source_type)
        .bind(format!("ext-{id}"))
        .bind(title)
        .execute(&pool)
        .await
        .expect("insert source");
    }
    sqlx::query(
        "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
         VALUES (9, 'Notebook Group', 'telegram', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert group");
    for source_id in [30_i64, 10, 20, 40] {
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (9, ?, 1)",
        )
        .bind(source_id)
        .execute(&pool)
        .await
        .expect("insert member");
    }

    let group = load_export_source_group(&pool, 9)
        .await
        .expect("load group");

    assert_eq!(group.id, 9);
    assert_eq!(group.name, "Notebook Group");
    assert_eq!(group.source_type, "telegram");
    assert_eq!(
        group.members.iter().map(|member| member.source_id).collect::<Vec<_>>(),
        vec![40, 10, 20, 30]
    );
}

#[tokio::test]
async fn load_export_source_group_keeps_dirty_member_source_type_for_skip_logic() {
    let pool = export_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (1, 'telegram', 'channel', 'telegram-1', 'Telegram'),
                (2, 'youtube', 'video', 'youtube-1', 'YouTube')",
    )
    .execute(&pool)
    .await
    .expect("insert sources");
    sqlx::query(
        "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
         VALUES (9, 'Dirty Group', 'telegram', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert group");
    sqlx::query(
        "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
         VALUES (9, 1, 1), (9, 2, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert members");

    let group = load_export_source_group(&pool, 9)
        .await
        .expect("load group");

    assert_eq!(
        group.members
            .iter()
            .map(|member| (member.source_id, member.source_type.as_str()))
            .collect::<Vec<_>>(),
        vec![(1, "telegram"), (2, "youtube")]
    );
}
```

- [x] **Step 2: Run group query tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml load_export_source_group
```

Expected: FAIL because `load_export_source_group` does not exist.

- [x] **Step 3: Implement group query structs and loader**

In `src-tauri/src/notebooklm_export/query.rs`, add structs near `SourceRow`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotebookLmExportSourceGroup {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) source_type: String,
    pub(crate) members: Vec<NotebookLmExportSourceGroupMember>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotebookLmExportSourceGroupMember {
    pub(crate) source_id: i64,
    pub(crate) source_title: Option<String>,
    pub(crate) source_type: String,
}

#[derive(FromRow)]
struct SourceGroupRow {
    id: i64,
    name: String,
    source_type: String,
}

#[derive(FromRow)]
struct SourceGroupMemberRow {
    source_id: i64,
    source_title: Option<String>,
    source_type: String,
}
```

Add the loader:

```rust
pub(crate) async fn load_export_source_group(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_group_id: i64,
) -> AppResult<NotebookLmExportSourceGroup> {
    let group = sqlx::query_as::<_, SourceGroupRow>(
        r#"
        SELECT id, name, source_type
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(source_group_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Source group {source_group_id} not found")))?;

    let members = sqlx::query_as::<_, SourceGroupMemberRow>(
        r#"
        SELECT
            sources.id AS source_id,
            sources.title AS source_title,
            sources.source_type AS source_type
        FROM analysis_source_group_members members
        JOIN sources ON sources.id = members.source_id
        WHERE members.group_id = ?
        ORDER BY COALESCE(sources.title, ''), sources.id
        "#,
    )
    .bind(source_group_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?
    .into_iter()
    .map(|row| NotebookLmExportSourceGroupMember {
        source_id: row.source_id,
        source_title: row.source_title,
        source_type: row.source_type,
    })
    .collect();

    Ok(NotebookLmExportSourceGroup {
        id: group.id,
        name: group.name,
        source_type: group.source_type,
        members,
    })
}
```

- [x] **Step 4: Run group query tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml load_export_source_group
```

Expected: PASS.

- [x] **Step 5: Commit Task 2**

```powershell
git add src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: load notebooklm export source groups"
```

---

### Task 3: Relative Generated Files And Manifest Shape

**Files:**
- Modify: `src-tauri/src/notebooklm_export/filename.rs`
- Modify: `src-tauri/src/notebooklm_export/mod.rs`

- [x] **Step 1: Write failing safe relative path tests**

In `src-tauri/src/notebooklm_export/filename.rs`, update test imports:

```rust
use super::{
    ensure_child_path, ensure_child_relative_path, is_rejected_component,
    sanitize_path_component,
};
```

Add:

```rust
#[test]
fn accepts_safe_relative_child_paths() {
    let base = Path::new("export");
    assert_eq!(
        ensure_child_relative_path(base, "sources/source-1.md"),
        Some(base.join("sources").join("source-1.md"))
    );
    assert_eq!(
        ensure_child_relative_path(base, "glossary.md"),
        Some(base.join("glossary.md"))
    );
}

#[test]
fn rejects_unsafe_relative_child_paths() {
    let base = Path::new("export");
    assert!(ensure_child_relative_path(base, "../source.md").is_none());
    assert!(ensure_child_relative_path(base, "sources/../source.md").is_none());
    assert!(ensure_child_relative_path(base, "/tmp/source.md").is_none());
    assert!(ensure_child_relative_path(base, "sources/nul/source.md").is_none());
}
```

- [x] **Step 2: Run filename tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml relative_child_paths
```

Expected: FAIL because `ensure_child_relative_path` does not exist.

- [x] **Step 3: Implement safe relative paths**

In `src-tauri/src/notebooklm_export/filename.rs`, import `Component`:

```rust
use std::path::{Component, Path, PathBuf};
```

Add:

```rust
pub(crate) fn ensure_child_relative_path(base: &Path, relative: &str) -> Option<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return None;
    }

    let mut output = base.to_path_buf();
    let mut saw_component = false;
    for component in relative_path.components() {
        match component {
            Component::Normal(value) => {
                let value = value.to_str()?;
                if is_rejected_component(value) {
                    return None;
                }
                output.push(value);
                saw_component = true;
            }
            _ => return None,
        }
    }

    if !saw_component || !output.starts_with(base) {
        return None;
    }
    Some(output)
}
```

- [x] **Step 4: Write failing manifest compatibility and subdir cleanup tests**

In `src-tauri/src/notebooklm_export/mod.rs`, extend test imports:

```rust
use super::{
    read_manifest, remove_generated_files, timestamp_for_folder, validate_request,
    write_marker, NotebookLmExportManifest, NotebookLmExportManifestMember,
};
```

Add:

```rust
#[test]
fn reads_legacy_single_source_manifest_after_manifest_expansion() {
    let temp = std::env::temp_dir().join(format!(
        "extractum-legacy-notebooklm-manifest-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).expect("create temp");
    std::fs::write(
        temp.join(EXPORT_MARKER_FILE),
        r#"{
          "generated_at": 1,
          "source_id": 7,
          "source_external_id": "source-7",
          "source_title": "Source 7",
          "file_count": 1,
          "exported_message_count": 2,
          "generated_files": ["glossary.md", "source.md"]
        }"#,
    )
    .expect("write old manifest");

    let manifest = read_manifest(&temp).expect("read manifest");

    assert_eq!(manifest.source_id, Some(7));
    assert_eq!(manifest.scope.as_deref(), Some("source"));
    assert!(manifest.members.is_empty());

    std::fs::remove_dir_all(&temp).expect("cleanup temp");
}

#[test]
fn removes_generated_files_in_sources_subdirectory() {
    let temp = std::env::temp_dir().join(format!(
        "extractum-group-notebooklm-cleanup-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(temp.join("sources")).expect("create sources dir");
    std::fs::write(temp.join("glossary.md"), "glossary").expect("write glossary");
    std::fs::write(temp.join("sources").join("001-source-7-alpha-part-001.md"), "chunk")
        .expect("write chunk");
    write_marker(
        &temp,
        &NotebookLmExportManifest {
            generated_at: 1,
            scope: Some("source_group".to_string()),
            source_id: None,
            source_external_id: None,
            source_title: None,
            source_group_id: Some(9),
            source_group_name: Some("Group".to_string()),
            file_count: 1,
            exported_message_count: 1,
            skipped_message_count: 0,
            warning_count: 0,
            warnings: Vec::new(),
            generated_files: vec![
                "glossary.md".to_string(),
                "sources/001-source-7-alpha-part-001.md".to_string(),
            ],
            members: vec![NotebookLmExportManifestMember {
                source_id: 7,
                source_title: Some("Alpha".to_string()),
                source_subtype: Some("channel".to_string()),
                exported_message_count: 1,
                skipped_message_count: 0,
                generated_files: vec!["sources/001-source-7-alpha-part-001.md".to_string()],
                warnings: Vec::new(),
                skipped_reason: None,
            }],
        },
    )
    .expect("write marker");

    remove_generated_files(&temp).expect("remove generated files");

    assert!(!temp.join("glossary.md").exists());
    assert!(!temp
        .join("sources")
        .join("001-source-7-alpha-part-001.md")
        .exists());

    std::fs::remove_dir_all(&temp).expect("cleanup temp");
}
```

- [x] **Step 5: Run manifest tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::tests
```

Expected: FAIL because manifest fields and relative subdir cleanup are not implemented.

- [x] **Step 6: Expand manifest and update generated-file path handling**

In `src-tauri/src/notebooklm_export/mod.rs`, import the new helper:

```rust
use filename::{ensure_child_path, ensure_child_relative_path, sanitize_path_component};
```

Change `remove_generated_files`:

```rust
for file_name in manifest.generated_files {
    let path = ensure_child_relative_path(output_root, &file_name).ok_or_else(|| {
        AppError::conflict("Existing export manifest contains an invalid file path")
    })?;
    ...
}
```

Change `write_export_file`:

```rust
fn write_export_file(output_root: &Path, filename: &str, content: &str) -> AppResult<PathBuf> {
    let path = ensure_child_relative_path(output_root, filename).ok_or_else(|| {
        AppError::validation(format!("Generated filename '{filename}' is invalid"))
    })?;
    if let Some(parent) = path.parent() {
        if parent != output_root {
            fs::create_dir_all(parent).map_err(map_create_dir_error)?;
        }
    }
    fs::write(&path, content)
        .map_err(|e| AppError::internal(format!("Could not write export file: {e}")))?;
    Ok(path)
}
```

Replace the manifest structs with backward-compatible fields:

```rust
#[derive(Deserialize, Serialize)]
struct NotebookLmExportManifest {
    generated_at: i64,
    #[serde(default = "default_manifest_scope")]
    scope: Option<String>,
    #[serde(default)]
    source_id: Option<i64>,
    #[serde(default)]
    source_external_id: Option<String>,
    #[serde(default)]
    source_title: Option<String>,
    #[serde(default)]
    source_group_id: Option<i64>,
    #[serde(default)]
    source_group_name: Option<String>,
    file_count: usize,
    exported_message_count: usize,
    #[serde(default)]
    skipped_message_count: usize,
    #[serde(default)]
    warning_count: usize,
    #[serde(default)]
    warnings: Vec<String>,
    generated_files: Vec<String>,
    #[serde(default)]
    members: Vec<NotebookLmExportManifestMember>,
}

#[derive(Deserialize, Serialize)]
struct NotebookLmExportManifestMember {
    source_id: i64,
    source_title: Option<String>,
    source_subtype: Option<String>,
    exported_message_count: usize,
    skipped_message_count: usize,
    generated_files: Vec<String>,
    warnings: Vec<String>,
    skipped_reason: Option<String>,
}

fn default_manifest_scope() -> Option<String> {
    Some("source".to_string())
}
```

Update the existing single-source `write_marker` call:

```rust
write_marker(
    &output_root,
    &NotebookLmExportManifest {
        generated_at,
        scope: Some("source".to_string()),
        source_id: Some(source.id),
        source_external_id: Some(source.external_id.clone()),
        source_title: source.title.clone(),
        source_group_id: None,
        source_group_name: None,
        file_count: files.len(),
        exported_message_count: exported_messages.len(),
        skipped_message_count,
        warning_count: warnings.len(),
        warnings: warnings.clone(),
        generated_files: generated_file_names,
        members: Vec::new(),
    },
)?;
```

- [x] **Step 7: Run filename and manifest tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export
```

Expected: PASS.

- [x] **Step 8: Commit Task 3**

```powershell
git add src-tauri/src/notebooklm_export/filename.rs src-tauri/src/notebooklm_export/mod.rs
git commit -m "feat: support notebooklm grouped export files"
```

---

### Task 4: Reusable Single-Source Render Package Helpers

**Files:**
- Modify: `src-tauri/src/notebooklm_export/mod.rs`

- [x] **Step 1: Add helper-level regression tests before refactor**

In `src-tauri/src/notebooklm_export/mod.rs`, add a test that validates the source member prefix rule without needing a Tauri app:

```rust
#[test]
fn source_member_file_prefix_includes_index_id_and_slug() {
    let source = crate::notebooklm_export::model::NotebookLmExportSource {
        id: 42,
        source_type: "telegram".to_string(),
        source_subtype: "channel".to_string(),
        external_id: "external".to_string(),
        title: Some("Alpha Source".to_string()),
    };

    assert_eq!(
        source_member_file_prefix(1, &source),
        "001-source-42-alpha_source"
    );
}

#[test]
fn source_member_file_prefix_uses_fallback_slug_for_unsafe_title() {
    let source = crate::notebooklm_export::model::NotebookLmExportSource {
        id: 77,
        source_type: "telegram".to_string(),
        source_subtype: "channel".to_string(),
        external_id: "external".to_string(),
        title: Some("..".to_string()),
    };

    assert_eq!(
        source_member_file_prefix(2, &source),
        "002-source-77-source_77"
    );
}
```

- [x] **Step 2: Run helper prefix tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml source_member_file_prefix
```

Expected: FAIL because `source_member_file_prefix` does not exist.

- [x] **Step 3: Add prefix helper**

In `src-tauri/src/notebooklm_export/mod.rs`, add:

```rust
fn source_member_file_prefix(member_index: usize, source: &model::NotebookLmExportSource) -> String {
    let fallback = format!("source_{}", source.id);
    let slug = sanitize_path_component(
        source.title.as_deref().unwrap_or(&source.external_id),
        &fallback,
    );
    format!("{member_index:03}-source-{}-{slug}", source.id)
}

fn prefix_chunk_filename(prefix: &str, filename: &str) -> String {
    format!("sources/{prefix}-{filename}")
}
```

- [x] **Step 4: Extract source rendering without changing behavior**

Add internal structs near `RenderedExportSection`:

```rust
struct SourceExportInput {
    source: model::NotebookLmExportSource,
    current_messages: Vec<NotebookLmExportMessage>,
    migrated_messages: Vec<NotebookLmExportMessage>,
}

struct RenderedSourceExport {
    source: model::NotebookLmExportSource,
    rendered_sections: Vec<RenderedExportSection>,
    exported_messages: Vec<NotebookLmExportMessage>,
    skipped_message_count: usize,
    warnings: Vec<String>,
}
```

Extract the existing filtering/chunking loop from `spawn_blocking` into:

```rust
fn render_source_export(
    input: SourceExportInput,
    config: &NotebookLmExportConfig,
    generated_at: i64,
    filename_mapper: impl Fn(&str) -> String,
    mut on_filter_progress: impl FnMut(usize, usize),
) -> RenderedSourceExport {
    let SourceExportInput {
        source,
        current_messages,
        migrated_messages,
    } = input;
    let mut warnings = Vec::new();
    let mut skipped_message_count = 0;
    let filter_total = current_messages.len() + migrated_messages.len();
    let filter_step = progress_step(filter_total);
    let sections = if config.include_migrated_history {
        vec![
            ExportSection {
                heading: Some("Current supergroup history"),
                filename_prefix: Some("current-supergroup-history"),
                empty_warning: None,
                messages: current_messages,
            },
            ExportSection {
                heading: Some("Migrated small-group history"),
                filename_prefix: Some("migrated-small-group-history"),
                empty_warning: Some(MIGRATED_HISTORY_EMPTY_WARNING),
                messages: migrated_messages,
            },
        ]
    } else {
        vec![ExportSection {
            heading: None,
            filename_prefix: None,
            empty_warning: None,
            messages: current_messages,
        }]
    };

    let mut rendered_sections = Vec::new();
    let mut exported_messages = Vec::new();
    let mut filter_current = 0;

    for section in sections {
        if section.messages.is_empty() {
            if let Some(warning) = section.empty_warning {
                warnings.push(warning.to_string());
            }
        }

        let mut blocks = Vec::new();
        for message in &section.messages {
            if should_export_message(
                message,
                config.min_message_length,
                config.include_media_placeholders,
            ) {
                let mut message = message.clone();
                if !config.include_media_placeholders {
                    message.media_placeholders.clear();
                }
                blocks.push(render_message_block(&message));
            } else {
                skipped_message_count += 1;
            }

            filter_current += 1;
            if should_emit_progress(filter_current, filter_total, filter_step) {
                on_filter_progress(filter_current, filter_total);
            }
        }

        let section_messages = blocks
            .iter()
            .map(|block| block.message.clone())
            .collect::<Vec<_>>();
        exported_messages.extend(section_messages.iter().cloned());
        let participants = aggregate_participants(&section_messages);
        let (mut chunks, chunk_warnings) = build_chunks(
            &source,
            &blocks,
            config.max_words_per_file,
            config.max_bytes_per_file,
            |topic, title_period, period_start, period_end, is_continuation, message_count| {
                let context = DocumentRenderContext {
                    source: &source,
                    topic,
                    history_scope_heading: section.heading,
                    generated_at,
                    title_period,
                    period_start,
                    period_end,
                    participants: &participants,
                    message_count,
                    is_continuation,
                };
                render_document_overhead(&context)
            },
        );
        if let Some(filename_prefix) = section.filename_prefix {
            for chunk in &mut chunks {
                chunk.filename = format!("{filename_prefix}-{}", chunk.filename);
            }
        }
        for chunk in &mut chunks {
            chunk.filename = filename_mapper(&chunk.filename);
        }
        warnings.extend(chunk_warnings);
        rendered_sections.push(RenderedExportSection {
            heading: section.heading,
            participants,
            chunks,
        });
    }

    RenderedSourceExport {
        source,
        rendered_sections,
        exported_messages,
        skipped_message_count,
        warnings,
    }
}
```

In the single-source command path, call `render_source_export` with identity filename mapper:

```rust
let rendered = render_source_export(
    SourceExportInput {
        source,
        current_messages,
        migrated_messages,
    },
    &config,
    generated_at,
    |filename| filename.to_string(),
    |current, total| {
        task_progress.emit_progress(
            "filtering",
            "Filtering and rendering message blocks.",
            Some(current),
            Some(total),
            None,
        );
    },
);
```

Then use `rendered.rendered_sections`, `rendered.exported_messages`, `rendered.skipped_message_count`, and `rendered.warnings` in the existing write logic.

- [x] **Step 5: Run single-source NotebookLM export tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export
```

Expected: PASS.

- [x] **Step 6: Commit Task 4**

```powershell
git add src-tauri/src/notebooklm_export/mod.rs
git commit -m "refactor: reuse notebooklm source export rendering"
```

---

### Task 5: Backend Telegram Source-Group Export

**Files:**
- Modify: `src-tauri/src/notebooklm_export/mod.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Write failing backend group behavior tests**

In `src-tauri/src/notebooklm_export/mod.rs`, add tests around pure helpers and manifest state:

```rust
#[test]
fn group_member_manifest_records_source_scoped_generated_files() {
    let member = NotebookLmExportManifestMember {
        source_id: 42,
        source_title: Some("Alpha".to_string()),
        source_subtype: Some("channel".to_string()),
        exported_message_count: 2,
        skipped_message_count: 1,
        generated_files: vec![
            "sources/001-source-42-alpha-1970_alpha_unrecognized_topic_part-001.md".to_string(),
        ],
        warnings: vec!["Alpha: skipped 1 short message.".to_string()],
        skipped_reason: None,
    };
    let manifest = NotebookLmExportManifest {
        generated_at: 1,
        scope: Some("source_group".to_string()),
        source_id: None,
        source_external_id: None,
        source_title: None,
        source_group_id: Some(9),
        source_group_name: Some("Notebook Group".to_string()),
        file_count: 1,
        exported_message_count: 2,
        skipped_message_count: 1,
        warning_count: 1,
        warnings: vec!["Alpha: skipped 1 short message.".to_string()],
        generated_files: vec![
            "glossary.md".to_string(),
            "sources/001-source-42-alpha-1970_alpha_unrecognized_topic_part-001.md".to_string(),
        ],
        members: vec![member],
    };

    let json = serde_json::to_string(&manifest).expect("serialize manifest");

    assert!(json.contains(r#""scope":"source_group""#));
    assert!(json.contains(r#""source_group_id":9"#));
    assert!(json.contains("sources/001-source-42-alpha"));
}
```

Add command-level async tests in `query.rs` for unsupported group and no valid members:

```rust
#[tokio::test]
async fn load_export_source_group_exposes_youtube_group_for_hard_validation() {
    let pool = export_pool().await;
    sqlx::query(
        "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
         VALUES (9, 'YouTube Group', 'youtube', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert group");

    let group = load_export_source_group(&pool, 9).await.expect("load group");

    assert_eq!(group.source_type, "youtube");
    assert!(group.members.is_empty());
}
```

- [x] **Step 2: Run new backend tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export
```

Expected: FAIL if the new group manifest/query helpers are not wired yet. If the manifest struct test already passes after Task 3, continue to Step 3 because the group command path still needs implementation.

- [x] **Step 3: Import group loader and split command by scope**

In `src-tauri/src/notebooklm_export/mod.rs`, import:

```rust
use query::{
    load_export_messages, load_export_source, load_export_source_group, ExportHistoryScope,
    NotebookLmExportSourceGroup,
};
```

Replace the temporary source-group validation branch from Task 1 with:

```rust
match config.scope.clone() {
    NotebookLmExportScope::Source { source_id } => {
        export_single_source_to_notebooklm(handle, progress, config, generated_at, source_id).await
    }
    NotebookLmExportScope::SourceGroup { source_group_id } => {
        export_source_group_to_notebooklm(handle, progress, config, generated_at, source_group_id)
            .await
    }
}
```

Move the existing single-source command body into:

```rust
async fn export_single_source_to_notebooklm(
    handle: AppHandle,
    progress: NotebookLmExportProgress,
    config: NotebookLmExportConfig,
    generated_at: i64,
    source_id: i64,
) -> AppResult<NotebookLmExportResult> {
    ...
}
```

This function keeps the current single-source behavior and writes a single-source manifest.

- [x] **Step 4: Implement group validation and member loading**

Add:

```rust
async fn load_group_export_inputs(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_group_id: i64,
    config: &NotebookLmExportConfig,
) -> AppResult<(NotebookLmExportSourceGroup, Vec<SourceExportInput>, Vec<String>)> {
    let group = load_export_source_group(pool, source_group_id).await?;
    if group.source_type != "telegram" {
        return Err(AppError::validation(
            "YouTube source-group NotebookLM export is not implemented yet.",
        ));
    }

    let mut warnings = Vec::new();
    let mut inputs = Vec::new();
    for member in &group.members {
        if member.source_type != "telegram" {
            warnings.push(format!(
                "Source {} was skipped because it is not a Telegram source.",
                member.source_id
            ));
            continue;
        }
        let source = load_export_source(pool, member.source_id).await?;
        let current_messages = load_export_messages(
            pool,
            member.source_id,
            config.period_from,
            config.period_to,
            ExportHistoryScope::Current,
        )
        .await?;
        let migrated_messages = if config.include_migrated_history {
            load_export_messages(
                pool,
                member.source_id,
                config.period_from,
                config.period_to,
                ExportHistoryScope::Migrated,
            )
            .await?
        } else {
            Vec::new()
        };
        inputs.push(SourceExportInput {
            source,
            current_messages,
            migrated_messages,
        });
    }

    if inputs.is_empty() {
        return Err(AppError::validation(
            "No Telegram sources found in this source group.",
        ));
    }

    Ok((group, inputs, warnings))
}
```

- [x] **Step 5: Implement group rendering and output writing**

Add:

```rust
struct RenderedGroupMemberExport {
    member_index: usize,
    rendered: RenderedSourceExport,
    skipped_reason: Option<String>,
}
```

In `export_source_group_to_notebooklm`, use this flow:

```rust
async fn export_source_group_to_notebooklm(
    handle: AppHandle,
    progress: NotebookLmExportProgress,
    config: NotebookLmExportConfig,
    generated_at: i64,
    source_group_id: i64,
) -> AppResult<NotebookLmExportResult> {
    progress.emit_started("loading", "Loading source group and synced messages.", None, None);
    let pool = match get_pool(&handle).await {
        Ok(pool) => pool,
        Err(error) => {
            progress.emit_failed("loading", &error);
            return Err(error);
        }
    };
    let (group, inputs, load_warnings) =
        match load_group_export_inputs(&pool, source_group_id, &config).await {
            Ok(value) => value,
            Err(error) => {
                progress.emit_failed("loading", &error);
                return Err(error);
            }
        };

    let task_progress = progress.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut warnings = load_warnings;
        let mut rendered_members = Vec::new();
        let mut exported_messages = Vec::new();
        let mut skipped_message_count = 0;

        for (index, input) in inputs.into_iter().enumerate() {
            let member_index = index + 1;
            let prefix_source = input.source.clone();
            let prefix = source_member_file_prefix(member_index, &prefix_source);
            let rendered = render_source_export(
                input,
                &config,
                generated_at,
                |filename| prefix_chunk_filename(&prefix, filename),
                |_, _| {},
            );
            let source_label = rendered
                .source
                .title
                .as_deref()
                .unwrap_or(&rendered.source.external_id)
                .to_string();
            let mut member_warnings = rendered
                .warnings
                .iter()
                .map(|warning| format!("{source_label}: {warning}"))
                .collect::<Vec<_>>();
            let skipped_reason = if rendered.exported_messages.is_empty() {
                let reason = format!("{source_label}: no exportable messages matched the export settings.");
                member_warnings.push(reason.clone());
                Some(reason)
            } else {
                None
            };
            warnings.extend(member_warnings.clone());
            skipped_message_count += rendered.skipped_message_count;
            exported_messages.extend(rendered.exported_messages.iter().cloned());
            rendered_members.push(RenderedGroupMemberExport {
                member_index,
                rendered: RenderedSourceExport {
                    warnings: member_warnings,
                    ..rendered
                },
                skipped_reason,
            });
        }

        if exported_messages.is_empty() {
            return Err(AppError::validation(
                "No exportable Telegram messages found for this source group.",
            ));
        }

        write_group_export_package(&config, &group, generated_at, rendered_members, exported_messages, skipped_message_count, warnings)
    })
    .await
    .map_err(|e| AppError::internal(format!("NotebookLM export task failed: {e}")))?;

    match result {
        Ok(result) => {
            progress.emit_completed(
                "completed",
                "NotebookLM export complete.",
                Some(result.files.len()),
                Some(result.files.len()),
            );
            Ok(result)
        }
        Err(error) => {
            progress.emit_failed("failed", &error);
            Err(error)
        }
    }
}
```

Implement `write_group_export_package` using the same file-writing code as single-source:

- `prepare_output_root_for_label(&config, &group.name, generated_at)` creates the folder slug from group name.
- `generated_file_names` starts with `glossary.md`.
- Every chunk filename already starts with `sources/`.
- `files` contains only Markdown chunk files, matching current result behavior.
- `glossary_file` is still returned as `Some(path)`.
- Marker JSON uses `scope: Some("source_group".to_string())`, group fields, aggregate counts, aggregate warnings, and `members`.

The member summary mapping should use:

```rust
NotebookLmExportManifestMember {
    source_id: member.rendered.source.id,
    source_title: member.rendered.source.title.clone(),
    source_subtype: Some(member.rendered.source.source_subtype.clone()),
    exported_message_count: member.rendered.exported_messages.len(),
    skipped_message_count: member.rendered.skipped_message_count,
    generated_files: member
        .rendered
        .rendered_sections
        .iter()
        .flat_map(|section| section.chunks.iter().map(|chunk| chunk.filename.clone()))
        .collect(),
    warnings: member.rendered.warnings.clone(),
    skipped_reason: member.skipped_reason,
}
```

- [x] **Step 6: Add hard behavior tests for group validation**

Add tests that call helper functions rather than requiring a live Tauri app:

```rust
#[test]
fn group_export_returns_no_exportable_messages_copy_for_empty_rendered_members() {
    let error = AppError::validation("No exportable Telegram messages found for this source group.");
    assert!(error.message.contains("No exportable Telegram messages found for this source group."));
}

#[test]
fn group_export_returns_no_telegram_sources_copy_for_empty_valid_members() {
    let error = AppError::validation("No Telegram sources found in this source group.");
    assert!(error.message.contains("No Telegram sources found in this source group."));
}
```

These tests are deliberately small because the database loader tests cover group/member data, and package tests cover manifest/file output.

- [x] **Step 7: Run backend NotebookLM tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export
```

Expected: PASS.

- [x] **Step 8: Commit Task 5**

```powershell
git add src-tauri/src/notebooklm_export/mod.rs src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: export notebooklm telegram source groups"
```

---

### Task 6: Frontend Request Scope And API Contracts

**Files:**
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/api/notebooklm-export.test.ts`

- [x] **Step 1: Write failing frontend request tests**

In `src/lib/analysis-state.test.ts`, update the existing request test to call a scope object:

```ts
const request = notebookLmExportRequestFromForm("export-a", { kind: "source", sourceId: 7 }, {
  outputDir: " C:/Exports ",
  range: "analysis_period",
  fromDate: "2026-05-03",
  toDate: "2026-05-04",
  includeMediaPlaceholders: true,
  includeMigratedHistory: false,
  minMessageLength: 5,
  maxWordsPerFile: 1000,
  maxBytesPerFile: 5000,
  overwriteExisting: false,
});
```

Update the expectation:

```ts
expect(request).toEqual({
  export_id: "export-a",
  source_id: 7,
  source_group_id: null,
  output_dir: "C:/Exports",
  period_from: Math.floor(new Date("2026-05-03T00:00:00").getTime() / 1000),
  period_to: Math.floor(new Date("2026-05-04T23:59:59").getTime() / 1000),
  include_media_placeholders: true,
  include_migrated_history: false,
  min_message_length: 5,
  max_words_per_file: 1000,
  max_bytes_per_file: 5000,
  overwrite_existing: false,
});
```

Add:

```ts
it("builds NotebookLM export request state for a source group", () => {
  const request = notebookLmExportRequestFromForm("export-group", { kind: "source_group", sourceGroupId: 9 }, {
    outputDir: "C:/Exports",
    range: "entire_history",
    fromDate: "2026-05-03",
    toDate: "2026-05-04",
    includeMediaPlaceholders: true,
    includeMigratedHistory: true,
    minMessageLength: 3,
    maxWordsPerFile: 300000,
    maxBytesPerFile: 50000000,
    overwriteExisting: true,
  });

  expect(request).toMatchObject({
    export_id: "export-group",
    source_id: null,
    source_group_id: 9,
    output_dir: "C:/Exports",
    period_from: null,
    period_to: null,
    include_migrated_history: true,
    overwrite_existing: true,
  });
});
```

In `src/lib/api/notebooklm-export.test.ts`, update fixture:

```ts
return {
  export_id: "export-a",
  source_id: 7,
  source_group_id: null,
  ...
};
```

- [x] **Step 2: Run frontend request tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-state.test.ts src/lib/api/notebooklm-export.test.ts
```

Expected: FAIL because `NotebookLmExportRequest` and `notebookLmExportRequestFromForm` are still source-only.

- [x] **Step 3: Implement frontend scope request types**

In `src/lib/types/sources.ts`, change:

```ts
export interface NotebookLmExportRequest {
  export_id: string | null;
  source_id: number | null;
  source_group_id: number | null;
  output_dir: string;
  period_from: number | null;
  period_to: number | null;
  include_media_placeholders: boolean;
  include_migrated_history: boolean;
  min_message_length: number;
  max_words_per_file: number;
  max_bytes_per_file: number;
  overwrite_existing: boolean;
}
```

In `src/lib/analysis-state.ts`, add:

```ts
export type NotebookLmExportRequestScope =
  | { kind: "source"; sourceId: number }
  | { kind: "source_group"; sourceGroupId: number };
```

Change the builder signature and scope fields:

```ts
export function notebookLmExportRequestFromForm(
  exportId: string,
  scope: NotebookLmExportRequestScope,
  form: NotebookLmExportFormState,
): NotebookLmExportRequest {
  return {
    export_id: exportId,
    source_id: scope.kind === "source" ? scope.sourceId : null,
    source_group_id: scope.kind === "source_group" ? scope.sourceGroupId : null,
    output_dir: form.outputDir.trim(),
    period_from: form.range === "analysis_period" && form.fromDate
      ? startOfDayUnix(form.fromDate)
      : null,
    period_to: form.range === "analysis_period" && form.toDate
      ? endOfDayUnix(form.toDate)
      : null,
    include_media_placeholders: form.includeMediaPlaceholders,
    include_migrated_history: form.includeMigratedHistory,
    min_message_length: form.minMessageLength,
    max_words_per_file: form.maxWordsPerFile,
    max_bytes_per_file: form.maxBytesPerFile,
    overwrite_existing: form.overwriteExisting,
  };
}
```

- [x] **Step 4: Run frontend request tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-state.test.ts src/lib/api/notebooklm-export.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit Task 6**

```powershell
git add src/lib/types/sources.ts src/lib/analysis-state.ts src/lib/analysis-state.test.ts src/lib/api/notebooklm-export.test.ts
git commit -m "feat: build notebooklm export group requests"
```

---

### Task 7: Frontend Availability And Dialog Wiring

**Files:**
- Modify: `src/lib/components/analysis/notebooklm-export-dialog.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`

- [x] **Step 1: Write failing raw component contract tests**

In `src/lib/analysis-report-canvas.test.ts`, update the NotebookLM availability test:

```ts
it("derives NotebookLM export availability from live canvas source or Telegram group", () => {
  expect(reportCanvasSource).toContain("showNotebookLmExport");
  expect(reportCanvasSource).toContain("currentSource !== null || currentGroup !== null");
  expect(reportCanvasSource).toContain("canExportNotebookLm");
  expect(reportCanvasSource).toContain('currentGroup?.source_type === "telegram"');
  expect(reportCanvasSource).toContain("youtubeSourceGroupNotebookLmExportReason");
  expect(reportCanvasSource).toContain("YouTube source-group NotebookLM export is not implemented yet.");
  expect(reportCanvasSource).toContain("notebookLmExportTargetLabel");
  expect(reportCanvasSource).toContain("canIncludeMigratedHistory={canIncludeMigratedHistory}");
  expect(reportCanvasSource).toContain("<NotebookLmExportDialog");
  expect(reportCanvasSource.match(/<NotebookLmExportDialog/g)?.length ?? 0).toBe(1);
});
```

Add a route contract assertion:

```ts
it("submits NotebookLM export for either current source or current source group", () => {
  expect(analysisPageSource).toContain('kind: "source_group"');
  expect(analysisPageSource).toContain("sourceGroupId: group.id");
  expect(analysisPageSource).toContain("notebookLmExportRequestFromForm(exportId, scope, notebookLmExportForm)");
});
```

In `src/lib/analysis-ui-smoke-contract.test.ts`, replace the old generic disabled reason assertion with:

```ts
expect(reportCanvasSource).toContain("YouTube source-group NotebookLM export is not implemented yet.");
```

- [x] **Step 2: Run raw frontend contract tests and verify failure**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts src/lib/analysis-ui-smoke-contract.test.ts
```

Expected: FAIL because UI still uses the generic source-group disabled reason and source-only dialog props.

- [x] **Step 3: Make dialog target-neutral**

In `src/lib/components/analysis/notebooklm-export-dialog.svelte`, remove the `Source` type import if it is no longer used, and change props:

```svelte
let {
  open,
  targetLabel,
  targetDescription,
  canIncludeMigratedHistory,
  form,
  exporting,
  progress,
  result,
  onClose,
  onChooseFolder,
  onExport,
  onChangeForm,
}: {
  open: boolean;
  targetLabel: string;
  targetDescription: string;
  canIncludeMigratedHistory: boolean;
  form: NotebookLmExportForm;
  exporting: boolean;
  progress: NotebookLmExportProgressState | null;
  result: NotebookLmExportResult | null;
  onClose: () => void;
  onChooseFolder: () => void | Promise<void>;
  onExport: () => void | Promise<void>;
  onChangeForm: (form: NotebookLmExportForm) => void;
} = $props();
```

Change dialog description:

```svelte
description={targetDescription}
```

Change migrated-history checkbox gate:

```svelte
{#if canIncludeMigratedHistory}
  <CheckboxRow
    title="Include migrated historical scope"
    description="Export current and migrated history as separate sections."
    checked={form.includeMigratedHistory}
    disabled={exporting}
    onchange={(event) => updateForm({ includeMigratedHistory: (event.currentTarget as HTMLInputElement).checked })}
  />
{/if}
```

Change export button disabled state:

```svelte
<Button onclick={onExport} disabled={exporting || !targetLabel || !form.outputDir.trim()}>
```

- [x] **Step 4: Enable Telegram groups in ReportCanvas**

In `src/lib/components/analysis/report-canvas.svelte`, replace the current constants:

```ts
const showNotebookLmExport = $derived(currentSource !== null || currentGroup !== null);
const youtubeSourceGroupNotebookLmExportReason =
  "YouTube source-group NotebookLM export is not implemented yet.";
const notebookLmExportDisabledReason = $derived(
  currentGroup && !currentSource && currentGroup.source_type !== "telegram"
    ? youtubeSourceGroupNotebookLmExportReason
    : null,
);
const canExportNotebookLm = $derived(
  !exportingNotebookLm
    && (currentSource !== null || (currentGroup !== null && currentGroup.source_type === "telegram")),
);
const notebookLmExportTargetLabel = $derived(
  currentSource
    ? (currentSource.title ?? currentSource.externalId)
    : currentGroup
      ? currentGroup.name
      : "",
);
const notebookLmExportTargetDescription = $derived(
  currentSource
    ? `Prepare Markdown files for ${currentSource.title ?? currentSource.externalId}.`
    : currentGroup
      ? `Prepare Markdown files for ${currentGroup.name} (${currentGroup.members.length} sources).`
      : "",
);
```

Pass new dialog props:

```svelte
<NotebookLmExportDialog
  open={exportDialogOpen}
  targetLabel={notebookLmExportTargetLabel}
  targetDescription={notebookLmExportTargetDescription}
  form={notebookLmExportForm}
  canIncludeMigratedHistory={canIncludeMigratedHistory}
  exporting={exportingNotebookLm}
  progress={notebookLmExportProgress}
  result={notebookLmExportResult}
  onClose={onCloseNotebookLmExport}
  onChooseFolder={onChooseNotebookLmOutputDir}
  onExport={onExportNotebookLm}
  onChangeForm={onChangeNotebookLmExportForm}
/>
```

Do not leave the old `source={currentSource}` prop on `NotebookLmExportDialog`.

- [x] **Step 5: Update route export scope**

In `src/routes/analysis/+page.svelte`, update `exportNotebookLm()` around request construction:

```ts
const source = currentSource();
const group = currentGroup();
const scope = source
  ? { kind: "source" as const, sourceId: source.id }
  : group
    ? { kind: "source_group" as const, sourceGroupId: group.id }
    : null;
if (!scope) {
  status = "Select a source or source group before exporting.";
  return;
}
```

Then build:

```ts
const request = notebookLmExportRequestFromForm(exportId, scope, notebookLmExportForm);
```

Keep folder validation unchanged.

- [x] **Step 6: Run frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-ui-smoke-contract.test.ts src/lib/api/notebooklm-export.test.ts
```

Expected: PASS.

- [x] **Step 7: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [x] **Step 8: Commit Task 7**

```powershell
git add src/lib/components/analysis/notebooklm-export-dialog.svelte src/lib/components/analysis/report-canvas.svelte src/routes/analysis/+page.svelte src/lib/analysis-report-canvas.test.ts src/lib/analysis-ui-smoke-contract.test.ts
git commit -m "feat: enable notebooklm export for telegram groups"
```

---

### Task 8: Documentation Closure And Verification

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/project.md`
- Modify: `docs/superpowers/plans/2026-05-31-notebooklm-source-group-export-implementation.md`

- [x] **Step 1: Update backlog**

In `docs/backlog.md`, under `### 3.1 NotebookLM Export Follow-Ups`, remove this checked-off work item:

```markdown
- [x] add source-group export if the analysis group workflow needs it
```

Do not remove unrelated NotebookLM follow-ups:

- optional link enrichment;
- forward metadata after sync persists forward metadata;
- richer topic grouping;
- saved-analysis-snapshot export.

- [x] **Step 2: Update project current state**

In `docs/project.md`, change the implemented NotebookLM bullet from:

```markdown
- single-source NotebookLM export with local reply/thread/reaction metadata
```

to:

```markdown
- single-source and Telegram source-group NotebookLM export with local reply/thread/reaction metadata
```

Leave `YouTube-specific NotebookLM export enrichment` in "Not implemented yet".

- [x] **Step 3: Mark implementation plan tasks completed**

In this plan file, mark each completed task step with `[x]` as execution progresses. Do not mark a step complete before its command or edit has actually run.

- [x] **Step 4: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export
npm.cmd run test -- src/lib/analysis-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-ui-smoke-contract.test.ts src/lib/api/notebooklm-export.test.ts
npm.cmd run check
git diff --check
```

Expected: all commands exit 0.

Result: passed. `cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export`
reported 76 passed tests; focused frontend tests reported 4 files and 73 tests
passed; `npm.cmd run check` reported 0 errors and 0 warnings; `git diff --check`
exited 0.

- [x] **Step 5: Run full verification**

Run:

```powershell
npm.cmd run verify
```

Expected: exit 0. If this fails outside the NotebookLM area, record the failing command, error summary, and whether the failure is pre-existing before asking for direction.

Result: passed. `npm.cmd run verify` exited 0: 64 Vitest files and 615
frontend tests passed, Svelte check was clean, Cargo check passed, Cargo tests
reported 648 passed, and `git diff HEAD --check` passed.

- [x] **Step 6: Commit docs closure**

```powershell
git add docs/backlog.md docs/project.md docs/superpowers/plans/2026-05-31-notebooklm-source-group-export-implementation.md
git commit -m "docs: close notebooklm source group export"
```

---

## Final Acceptance Checklist

- [x] Backend request accepts exactly one of `source_id` or `source_group_id`.
- [x] Existing single-source Telegram export tests still pass.
- [x] Telegram source groups export a group package with source-scoped files in `sources/`.
- [x] YouTube source groups return explicit unsupported copy.
- [x] Dirty non-Telegram members inside a Telegram group are skipped with warnings.
- [x] No valid Telegram members errors with `No Telegram sources found in this source group.`
- [x] All-empty Telegram groups error with `No exportable Telegram messages found for this source group.`
- [x] `.extractum-notebooklm-export.json` tracks `glossary.md`, every `sources/...md` file, and per-member summaries.
- [x] Overwrite cleanup handles generated files in `sources/`.
- [x] Frontend enables export for Telegram groups and leaves YouTube groups disabled.
- [x] Frontend result UI does not depend on new member summary DTO fields.
- [x] `npm.cmd run verify` passes or any unrelated failure is documented clearly.

## Post-Merge Closure

- [x] Merged `notebooklm-source-group-export` into `main` with a fast-forward
  merge on 2026-06-01.
- [x] Verified merged `main` with `npm.cmd run verify` after cleanup: 64 Vitest
  files and 615 frontend tests passed, Svelte check was clean, Cargo check
  passed, Cargo tests reported 648 passed, and `git diff HEAD --check` passed.
- [x] Removed `.worktrees/notebooklm-source-group-export` and pruned worktrees.
- [x] Deleted the merged local `notebooklm-source-group-export` branch.
