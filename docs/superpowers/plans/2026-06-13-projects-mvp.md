# Projects MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the real Projects MVP: durable `projects`, direct `project_sources`, `/projects` workspace backed by the new model, and single-provider project analysis runs.

**Architecture:** Add a new backend `projects` module beside legacy `analysis_source_groups`. Extend the existing analysis run pipeline with optional `project_id` instead of creating a second project-run pipeline. Replace the current `/projects` source-group facade with project API/types/view-models while reusing extractum-ui wrappers and the existing Library source read model.

**Tech Stack:** Rust/Tauri, SQLite via sqlx and Tauri SQL migrations, Svelte 5, TypeScript, Vitest, cargo tests, extractum-ui wrappers.

---

## File Structure

Backend:

- Create `src-tauri/migrations/0005_projects_mvp.sql`: schema migration for `projects`, `project_sources`, and `analysis_runs.project_id`.
- Modify `src-tauri/src/migrations.rs`: register migration 5 and add migration tests.
- Create `src-tauri/src/projects.rs`: project CRUD, membership commands, read models, project-run command wrapper.
- Modify `src-tauri/src/lib.rs`: register the `projects` module and Tauri commands.
- Modify `src-tauri/src/analysis/models.rs`: add project fields to run rows/summaries/details.
- Modify `src-tauri/src/analysis/store.rs`: include project joins, project filters, duplicate detection, insert fields, and run deletion support.
- Modify `src-tauri/src/analysis/corpus.rs`: support `project_id` scope resolution.
- Modify `src-tauri/src/analysis/report.rs`: accept `project_id` in `StartAnalysisReportRequest`.
- Modify `src-tauri/src/analysis/report_commands.rs`: keep legacy `start_analysis_report`; project run starts through `projects::start_project_analysis`.
- Modify `src-tauri/src/analysis/mod.rs`: expose `ANALYSIS_SCOPE_TYPE_PROJECT`, `models`, `report`, and `store` to sibling modules with `pub(crate)`.
- Modify `src-tauri/src/library_sources/mod.rs`: count real `project_sources`.
- Modify `src-tauri/src/sources/store.rs`: block Library source deletion when the source is used by projects.
- Modify `docs/database-schema.md`: document new schema and `analysis_runs.project_id`.

Frontend:

- Create `src/lib/types/projects.ts`: project API and UI types.
- Create `src/lib/api/projects.ts`: Tauri invoke wrappers for project commands.
- Modify `src/lib/types/analysis.ts`: add `project_id`, `project_name`, and optional project run inputs.
- Modify `src/lib/api/analysis-runs.ts`: add `startProjectAnalysis`.
- Modify `src/lib/ui/research-projects-model.ts`: rebuild view model from real projects/project sources.
- Modify `src/lib/ui/research-projects-workflow.ts`: use project APIs instead of source-group APIs.
- Modify `src/routes/projects/+page.svelte`: wire new workflow dependencies.
- Modify `src/lib/components/research-projects/ProjectsShell.svelte`: add inspector column and project CRUD callbacks.
- Modify `src/lib/components/research-projects/ProjectRail.svelte`: create/select/search real projects.
- Modify `src/lib/components/research-projects/ProjectWorkspace.svelte`: focus on source table and selected source.
- Modify `src/lib/components/research-projects/SourcesTab.svelte`: render project source rows and remove action.
- Modify `src/lib/components/research-projects/ConnectFromLibrary.svelte`: reuse modal/sheet for project membership.
- Create `src/lib/components/research-projects/ProjectEditorDialog.svelte`: create/edit modal.
- Create `src/lib/components/research-projects/ProjectInspector.svelte`: summary, selected source context, run eligibility, recent runs.
- Create `src/lib/components/research-projects/ProjectRunDialog.svelte`: project report-run settings using current report-run fields.

---

### Task 1: Schema Migration

**Files:**
- Create: `src-tauri/migrations/0005_projects_mvp.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `docs/database-schema.md`

- [x] **Step 1: Write failing migration tests**

Add tests to `src-tauri/src/migrations.rs`:

```rust
#[test]
fn projects_mvp_migration_is_registered() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 5)
        .expect("projects MVP migration is registered");

    assert_eq!(migration.description, "projects mvp schema");
    assert!(migration.sql.contains("CREATE TABLE IF NOT EXISTS projects"));
    assert!(migration.sql.contains("CREATE TABLE IF NOT EXISTS project_sources"));
    assert!(migration.sql.contains("ALTER TABLE analysis_runs ADD COLUMN project_id"));
}

#[tokio::test]
async fn projects_mvp_schema_applies_to_memory_pool() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    let project_table: String = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'projects'",
    )
    .fetch_one(&pool)
    .await
    .expect("projects table exists");
    assert_eq!(project_table, "projects");

    let project_sources_table: String = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'project_sources'",
    )
    .fetch_one(&pool)
    .await
    .expect("project_sources table exists");
    assert_eq!(project_sources_table, "project_sources");

    let project_id_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('analysis_runs') WHERE name = 'project_id'",
    )
    .fetch_one(&pool)
    .await
    .expect("read analysis_runs columns");
    assert_eq!(project_id_columns, 1);
}
```

- [x] **Step 2: Run tests and verify failure**

Run:

```powershell
cargo test projects_mvp --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL because migration version 5 is not registered.

- [x] **Step 3: Create migration SQL**

Create `src-tauri/migrations/0005_projects_mvp.sql`:

```sql
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL COLLATE NOCASE,
    description TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_name_unique
ON projects(name COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS project_sources (
    project_id INTEGER NOT NULL
        REFERENCES projects(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL
        REFERENCES sources(id) ON DELETE RESTRICT,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (project_id, source_id)
);

CREATE INDEX IF NOT EXISTS idx_project_sources_source_id
ON project_sources(source_id);

CREATE INDEX IF NOT EXISTS idx_project_sources_project_id_added_at
ON project_sources(project_id, added_at DESC, source_id DESC);

ALTER TABLE analysis_runs
ADD COLUMN project_id INTEGER
    REFERENCES projects(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_analysis_runs_project_id_created_at
ON analysis_runs(project_id, created_at DESC);
```

- [x] **Step 4: Register migration**

Modify `src-tauri/src/migrations.rs`:

```rust
const PROJECTS_MVP_VERSION: i64 = 5;
const PROJECTS_MVP_DESCRIPTION: &str = "projects mvp schema";
const PROJECTS_MVP_SQL: &str = include_str!("../migrations/0005_projects_mvp.sql");
```

Add:

```rust
fn projects_mvp_migration() -> Migration {
    Migration {
        version: PROJECTS_MVP_VERSION,
        description: PROJECTS_MVP_DESCRIPTION,
        sql: PROJECTS_MVP_SQL,
        kind: MigrationKind::Up,
    }
}
```

Append it to `build_migrations()` after `source_delete_cascade_indexes_migration()`:

```rust
pub fn build_migrations() -> Vec<Migration> {
    vec![
        current_schema_baseline_migration(),
        migrated_history_opt_in_migration(),
        analysis_telegram_history_scope_migration(),
        source_delete_cascade_indexes_migration(),
        projects_mvp_migration(),
    ]
}
```

Update the existing `build_migrations_starts_at_current_schema_baseline` test so the
version list includes the new migration:

```rust
assert_eq!(versions, vec![1, 2, 3, 4, 5]);
assert_eq!(migrations[4].description, "projects mvp schema");
assert!(migrations[4].sql.contains("CREATE TABLE IF NOT EXISTS projects"));
```

- [x] **Step 5: Update database schema docs**

In `docs/database-schema.md`, add a section after `analysis_source_group_members`:

```markdown
### 2.5 `projects`

Durable research projects. Projects are first-class analysis workspaces and are
not aliases for `analysis_source_groups`.

Important fields:

- `id`
- `name`
- `description`
- `created_at`
- `updated_at`

Notes:

- `name` is unique in the MVP after trimming. This is a product policy, not a
  stable identity rule.
- Timestamps are integer Unix seconds.

### 2.6 `project_sources`

Join table between projects and canonical Library sources.

Important fields:

- `project_id`
- `source_id`
- `added_at`

Notes:

- `UNIQUE(project_id, source_id)` prevents duplicate membership.
- A source can belong to multiple projects.
- Deleting a project deletes its project-source links.
- Deleting a Library source is blocked while it belongs to a project.
```

Update the `analysis_runs` section with:

```markdown
- `project_id`
```

and note:

```markdown
- Project-scoped runs use `scope_type = 'project'` and `project_id`.
- Project run history is stored in `analysis_runs`; there is no separate
  `project_runs` table in the MVP.
```

Renumber following sections if the document uses consecutive numbering.

- [x] **Step 6: Run tests**

Run:

```powershell
cargo test projects_mvp --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri\src\migrations.rs src-tauri\migrations\0005_projects_mvp.sql docs\database-schema.md
git commit -m "feat: add projects mvp schema"
```

---

### Task 2: Backend Project CRUD And Membership

**Files:**
- Create: `src-tauri/src/projects.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/projects.rs`

- [x] **Step 1: Write failing backend tests**

Create `src-tauri/src/projects.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::apply_all_migrations_for_test_pool;

    async fn pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    async fn seed_account(pool: &sqlx::SqlitePool, id: i64) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (?, ?, 1, 'hash', 100)",
        )
        .bind(id)
        .bind(format!("Account {id}"))
        .execute(pool)
        .await
        .expect("seed account");
    }

    async fn seed_source(pool: &sqlx::SqlitePool, id: i64, provider: &str, subtype: &str) {
        let account_id = if provider == "telegram" { Some(1_i64) } else { None };
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at, account_id
            )
            VALUES (?, ?, ?, ?, ?, 1, 0, 100, ?)
            "#,
        )
        .bind(id)
        .bind(provider)
        .bind(subtype)
        .bind(format!("{provider}-{id}"))
        .bind(format!("Source {id}"))
        .bind(account_id)
        .execute(pool)
        .await
        .expect("seed source");
    }

    #[tokio::test]
    async fn create_project_trims_and_rejects_duplicate_names_case_insensitively() {
        let pool = pool().await;

        let created = create_project_in_pool(&pool, "  Alpha  ", Some("Desc".to_string()))
            .await
            .expect("create project");
        assert_eq!(created.name, "Alpha");
        assert_eq!(created.description.as_deref(), Some("Desc"));

        let duplicate = create_project_in_pool(&pool, "alpha", None)
            .await
            .expect_err("duplicate rejected");
        assert_eq!(duplicate.kind, crate::error::AppErrorKind::Validation);
    }

    #[tokio::test]
    async fn add_project_sources_is_idempotent_and_lists_ui_ready_rows() {
        let pool = pool().await;
        seed_account(&pool, 1).await;
        seed_source(&pool, 10, "youtube", "video").await;
        seed_source(&pool, 11, "telegram", "supergroup").await;
        let project = create_project_in_pool(&pool, "Mixed", None)
            .await
            .expect("create project");

        let first = add_project_sources_in_pool(&pool, project.id, vec![10, 11])
            .await
            .expect("add sources");
        assert_eq!(first.added_count, 2);
        assert_eq!(first.already_present_count, 0);

        let second = add_project_sources_in_pool(&pool, project.id, vec![10, 11])
            .await
            .expect("add sources again");
        assert_eq!(second.added_count, 0);
        assert_eq!(second.already_present_count, 2);

        let sources = list_project_sources_in_pool(&pool, project.id)
            .await
            .expect("list project sources");
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_id, 11);
        assert_eq!(sources[0].provider, "telegram");
        assert_eq!(sources[1].source_id, 10);
        assert_eq!(sources[1].provider, "youtube");
    }

    #[tokio::test]
    async fn delete_project_removes_membership_and_project_runs_but_keeps_sources() {
        let pool = pool().await;
        seed_source(&pool, 10, "youtube", "video").await;
        let project = create_project_in_pool(&pool, "Delete me", None)
            .await
            .expect("create project");
        add_project_sources_in_pool(&pool, project.id, vec![10])
            .await
            .expect("add source");
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id, run_type, scope_type, project_id, period_from, period_to,
                output_language, prompt_template_id, prompt_template_version,
                provider_profile, provider, model, status, created_at
            )
            VALUES (500, 'report', 'project', ?, 1, 2, 'en', 1, 1, 'default', 'openai', 'gpt', 'completed', 100)
            "#,
        )
        .bind(project.id)
        .execute(&pool)
        .await
        .expect("seed project run");

        delete_project_in_pool(&pool, project.id)
            .await
            .expect("delete project");

        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await
            .expect("count projects");
        let membership_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_sources")
            .fetch_one(&pool)
            .await
            .expect("count project sources");
        let run_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_runs")
            .fetch_one(&pool)
            .await
            .expect("count runs");
        let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources")
            .fetch_one(&pool)
            .await
            .expect("count sources");

        assert_eq!(project_count, 0);
        assert_eq!(membership_count, 0);
        assert_eq!(run_count, 0);
        assert_eq!(source_count, 1);
    }
}
```

- [x] **Step 2: Run tests and verify failure**

Run:

```powershell
cargo test projects::tests --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL because helper functions and structs are not implemented.

- [x] **Step 3: Implement project models and helpers**

Add to `src-tauri/src/projects.rs` above tests:

```rust
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, serde::Serialize, sqlx::FromRow, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, sqlx::FromRow, PartialEq, Eq)]
pub struct ProjectSourceRecord {
    pub project_id: i64,
    pub source_id: i64,
    pub provider: String,
    pub source_subtype: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub item_count: i64,
    pub added_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct AddProjectSourcesOutcome {
    pub added_count: i64,
    pub already_present_count: i64,
}

fn normalize_project_name(name: &str) -> AppResult<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::validation("Project name cannot be empty"));
    }
    Ok(name)
}

fn normalize_description(description: Option<String>) -> Option<String> {
    description
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn ensure_project_exists(pool: &sqlx::SqlitePool, project_id: i64) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if exists == 0 {
        return Err(AppError::not_found(format!("Project {project_id} not found")));
    }
    Ok(())
}

async fn ensure_sources_exist(pool: &sqlx::SqlitePool, source_ids: &[i64]) -> AppResult<()> {
    if source_ids.is_empty() {
        return Ok(());
    }

    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT COUNT(*) FROM sources WHERE id IN (",
    );
    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");

    let found: i64 = query
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if found != source_ids.len() as i64 {
        return Err(AppError::validation("One or more selected sources do not exist"));
    }
    Ok(())
}
```

- [x] **Step 4: Implement CRUD functions**

Add:

```rust
pub(crate) async fn list_projects_in_pool(
    pool: &sqlx::SqlitePool,
) -> AppResult<Vec<ProjectRecord>> {
    sqlx::query_as(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM projects
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn create_project_in_pool(
    pool: &sqlx::SqlitePool,
    name: &str,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let name = normalize_project_name(name)?;
    let description = normalize_description(description);
    let now = crate::time::now_secs();

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO projects (name, description, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|error| {
        if error.to_string().to_lowercase().contains("unique") {
            AppError::validation("A project with this name already exists")
        } else {
            AppError::database(error)
        }
    })?;

    get_project_in_pool(pool, id).await?.ok_or_else(|| {
        AppError::not_found(format!("Project {id} not found after creation"))
    })
}

pub(crate) async fn get_project_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
) -> AppResult<Option<ProjectRecord>> {
    sqlx::query_as(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM projects
        WHERE id = ?
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn update_project_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    name: &str,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let name = normalize_project_name(name)?;
    let description = normalize_description(description);
    let now = crate::time::now_secs();

    let result = sqlx::query(
        r#"
        UPDATE projects
        SET name = ?, description = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(now)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(|error| {
        if error.to_string().to_lowercase().contains("unique") {
            AppError::validation("A project with this name already exists")
        } else {
            AppError::database(error)
        }
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Project {project_id} not found")));
    }

    get_project_in_pool(pool, project_id).await?.ok_or_else(|| {
        AppError::not_found(format!("Project {project_id} not found after update"))
    })
}
```

- [x] **Step 5: Implement membership and delete functions**

Add:

```rust
pub(crate) async fn add_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    mut source_ids: Vec<i64>,
) -> AppResult<AddProjectSourcesOutcome> {
    ensure_project_exists(pool, project_id).await?;
    source_ids.retain(|source_id| *source_id > 0);
    source_ids.sort_unstable();
    source_ids.dedup();
    ensure_sources_exist(pool, &source_ids).await?;

    let now = crate::time::now_secs();
    let mut added_count = 0;
    let mut already_present_count = 0;
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    for source_id in source_ids {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO project_sources (project_id, source_id, added_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(project_id)
        .bind(source_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

        if result.rows_affected() == 0 {
            already_present_count += 1;
        } else {
            added_count += 1;
        }
    }

    sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;
    Ok(AddProjectSourcesOutcome {
        added_count,
        already_present_count,
    })
}

pub(crate) async fn remove_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
    mut source_ids: Vec<i64>,
) -> AppResult<()> {
    ensure_project_exists(pool, project_id).await?;
    source_ids.retain(|source_id| *source_id > 0);
    source_ids.sort_unstable();
    source_ids.dedup();

    if source_ids.is_empty() {
        return Ok(());
    }

    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "DELETE FROM project_sources WHERE project_id = ",
    );
    query.push_bind(project_id);
    query.push(" AND source_id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in &source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");
    query
        .build()
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
        .bind(crate::time::now_secs())
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    Ok(())
}

pub(crate) async fn list_project_sources_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
) -> AppResult<Vec<ProjectSourceRecord>> {
    ensure_project_exists(pool, project_id).await?;
    sqlx::query_as(
        r#"
        SELECT
            ps.project_id,
            s.id AS source_id,
            s.source_type AS provider,
            s.source_subtype,
            s.title,
            CASE
                WHEN s.account_id IS NOT NULL THEN 'Account #' || s.account_id
                ELSE NULL
            END AS subtitle,
            COUNT(items.content_zstd) AS item_count,
            ps.added_at
        FROM project_sources ps
        JOIN sources s ON s.id = ps.source_id
        LEFT JOIN items ON items.source_id = s.id
        WHERE ps.project_id = ?
        GROUP BY ps.project_id, s.id, s.source_type, s.source_subtype, s.title, s.account_id, ps.added_at
        ORDER BY ps.added_at DESC, s.id DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn delete_project_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: i64,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    sqlx::query("DELETE FROM analysis_runs WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    sqlx::query("DELETE FROM project_sources WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    tx.commit().await.map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Project {project_id} not found")));
    }
    Ok(())
}
```

- [x] **Step 6: Implement Tauri commands**

Add command wrappers:

```rust
#[tauri::command]
pub async fn list_projects(handle: AppHandle) -> AppResult<Vec<ProjectRecord>> {
    let pool = get_pool(&handle).await?;
    list_projects_in_pool(&pool).await
}

#[tauri::command]
pub async fn create_project(
    handle: AppHandle,
    name: String,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let pool = get_pool(&handle).await?;
    create_project_in_pool(&pool, &name, description).await
}

#[tauri::command]
pub async fn update_project(
    handle: AppHandle,
    project_id: i64,
    name: String,
    description: Option<String>,
) -> AppResult<ProjectRecord> {
    let pool = get_pool(&handle).await?;
    update_project_in_pool(&pool, project_id, &name, description).await
}

#[tauri::command]
pub async fn delete_project(handle: AppHandle, project_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_project_in_pool(&pool, project_id).await
}

#[tauri::command]
pub async fn list_project_sources(
    handle: AppHandle,
    project_id: i64,
) -> AppResult<Vec<ProjectSourceRecord>> {
    let pool = get_pool(&handle).await?;
    list_project_sources_in_pool(&pool, project_id).await
}

#[tauri::command]
pub async fn add_project_sources(
    handle: AppHandle,
    project_id: i64,
    source_ids: Vec<i64>,
) -> AppResult<AddProjectSourcesOutcome> {
    let pool = get_pool(&handle).await?;
    add_project_sources_in_pool(&pool, project_id, source_ids).await
}

#[tauri::command]
pub async fn remove_project_sources(
    handle: AppHandle,
    project_id: i64,
    source_ids: Vec<i64>,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    remove_project_sources_in_pool(&pool, project_id, source_ids).await
}
```

- [x] **Step 7: Register module and commands**

Modify `src-tauri/src/lib.rs`:

```rust
mod projects;
use projects::{
    add_project_sources, create_project, delete_project, list_project_sources, list_projects,
    remove_project_sources, update_project,
};
```

Add these functions to `tauri::generate_handler!` near existing analysis/project commands:

```rust
list_projects,
create_project,
update_project,
delete_project,
list_project_sources,
add_project_sources,
remove_project_sources,
```

- [x] **Step 8: Run tests**

Run:

```powershell
cargo test projects::tests --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 9: Commit**

```powershell
git add src-tauri\src\projects.rs src-tauri\src\lib.rs
git commit -m "feat: add project crud commands"
```

---

### Task 3: Library Integration And Source Deletion Guard

**Files:**
- Modify: `src-tauri/src/library_sources/mod.rs`
- Modify: `src-tauri/src/sources/store.rs`

- [ ] **Step 1: Write failing Library project count test**

In `src-tauri/src/library_sources/mod.rs`, update `create_schema` in tests to create `projects` and `project_sources`:

```rust
r#"
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)
"#,
r#"
CREATE TABLE project_sources (
    project_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    added_at INTEGER NOT NULL
)
"#,
```

Change the existing seed from source groups to real projects:

```rust
sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (10, 'Project A', 1, 1), (11, 'Project B', 1, 1)")
    .execute(&pool)
    .await
    .expect("insert projects");
sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (10, 1, 1), (11, 1, 1), (10, 3, 1)")
    .execute(&pool)
    .await
    .expect("insert project sources");
```

Keep the assertion:

```rust
assert_eq!(video.project_count, 2);
```

- [ ] **Step 2: Run test and verify failure**

Run:

```powershell
cargo test library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL because `LIBRARY_SOURCES_SQL` still reads `analysis_source_group_members`.

- [ ] **Step 3: Update Library source query**

In `src-tauri/src/library_sources/mod.rs`, change the CTE:

```rust
project_counts AS (
    SELECT source_id, COUNT(DISTINCT project_id) AS project_count
    FROM project_sources
    GROUP BY source_id
)
```

- [ ] **Step 4: Write source deletion guard test**

In `src-tauri/src/sources/store.rs` tests, add a focused unit test using an in-memory schema:

```rust
#[tokio::test]
async fn delete_source_is_blocked_when_source_is_used_by_project() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    for statement in [
        "CREATE TABLE sources (id INTEGER PRIMARY KEY, source_type TEXT NOT NULL, created_at INTEGER NOT NULL)",
        "CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)",
        "CREATE TABLE project_sources (project_id INTEGER NOT NULL, source_id INTEGER NOT NULL, added_at INTEGER NOT NULL)",
    ] {
        sqlx::query(statement).execute(&pool).await.expect("create schema");
    }
    sqlx::query("INSERT INTO sources (id, source_type, created_at) VALUES (7, 'youtube', 1)")
        .execute(&pool)
        .await
        .expect("insert source");
    sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (3, 'Project', 1, 1)")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (3, 7, 1)")
        .execute(&pool)
        .await
        .expect("insert membership");

    let error = delete_source_from_pool(&pool, 7)
        .await
        .expect_err("source delete blocked");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
}
```

- [ ] **Step 5: Run test and verify failure**

Run:

```powershell
cargo test delete_source_is_blocked_when_source_is_used_by_project --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL because delete currently removes the source directly.

- [ ] **Step 6: Implement guard**

In `delete_source_from_pool`, before `DELETE FROM sources`, add:

```rust
let project_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM project_sources WHERE source_id = ?",
)
.bind(source_id)
.fetch_one(&mut *conn)
.await
.map_err(AppError::database)?;

if project_count > 0 {
    return Err(AppError::validation(format!(
        "Source {source_id} is used by {project_count} project(s). Remove it from projects first."
    )));
}
```

- [ ] **Step 7: Run tests**

Run:

```powershell
cargo test library_sources::tests --manifest-path src-tauri/Cargo.toml
cargo test delete_source_is_blocked_when_source_is_used_by_project --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri\src\library_sources\mod.rs src-tauri\src\sources\store.rs
git commit -m "feat: connect library sources to projects"
```

---

### Task 4: Project Scope In Analysis Runs

**Files:**
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/models.rs`
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/report_commands.rs`
- Modify: `src-tauri/src/projects.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing corpus tests for project scope**

In `src-tauri/src/analysis/corpus.rs` tests, add:

```rust
#[tokio::test]
async fn resolve_analysis_sources_rejects_mixed_provider_project() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    create_project_scope_schema(&pool).await;
    sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (9, 'Mixed', 1, 1)")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (1, 'youtube', 'video', 'v1', 'Video', 1, 0, 1), (2, 'telegram', 'supergroup', 'tg2', 'Telegram', 1, 0, 1)")
        .execute(&pool)
        .await
        .expect("insert sources");
    sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (9, 1, 1), (9, 2, 1)")
        .execute(&pool)
        .await
        .expect("insert project sources");

    let error = resolve_analysis_sources(&pool, None, None, Some(9))
        .await
        .expect_err("mixed project rejected");
    assert!(error.to_string().contains("mixed_provider_project_runs_not_supported"));
}

#[tokio::test]
async fn resolve_analysis_sources_loads_single_provider_project() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    create_project_scope_schema(&pool).await;
    sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (9, 'YouTube', 1, 1)")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (1, 'youtube', 'video', 'v1', 'Video 1', 1, 0, 1), (2, 'youtube', 'video', 'v2', 'Video 2', 1, 0, 1)")
        .execute(&pool)
        .await
        .expect("insert sources");
    sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (9, 1, 1), (9, 2, 1)")
        .execute(&pool)
        .await
        .expect("insert project sources");

    let resolved = resolve_analysis_sources(&pool, None, None, Some(9))
        .await
        .expect("resolve project");
    assert_eq!(resolved.source_type, "youtube");
    assert_eq!(resolved.source_ids, vec![1, 2]);
}
```

Add this helper in the `src-tauri/src/analysis/corpus.rs` test module:

```rust
async fn create_project_scope_schema(pool: &sqlx::SqlitePool) {
    for statement in [
        r#"
        CREATE TABLE sources (
            id INTEGER PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            external_id TEXT,
            title TEXT,
            is_active INTEGER NOT NULL,
            is_member INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
        r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
        r#"
        CREATE TABLE project_sources (
            project_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            added_at INTEGER NOT NULL
        )
        "#,
        r#"
        CREATE TABLE youtube_playlist_items (
            playlist_source_id INTEGER NOT NULL,
            video_source_id INTEGER,
            video_id TEXT NOT NULL,
            position INTEGER,
            is_removed_from_playlist INTEGER NOT NULL DEFAULT 0
        )
        "#,
    ] {
        sqlx::query(statement)
            .execute(pool)
            .await
            .expect("create project scope test schema");
    }
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
cargo test resolve_analysis_sources_ --manifest-path src-tauri/Cargo.toml
```

Expected: FAIL because `resolve_analysis_sources` does not accept `project_id`.

- [ ] **Step 3: Extend analysis constants and model types**

In `src-tauri/src/analysis/mod.rs`, add:

```rust
pub(crate) const ANALYSIS_SCOPE_TYPE_PROJECT: &str = "project";
```

In `src-tauri/src/analysis/models.rs`, add these fields to `AnalysisRunSummary`, `AnalysisRunDetail`, and `AnalysisRunRow`:

```rust
pub project_id: Option<i64>,
pub project_name: Option<String>,
```

For `AnalysisRunRow`, use `pub(crate)`.

Update all Rust tests that manually construct `AnalysisRunSummary`,
`AnalysisRunDetail`, or `AnalysisRunRow` with:

```rust
project_id: None,
project_name: None,
```

- [ ] **Step 4: Update store queries**

In `src-tauri/src/analysis/store.rs`, update `ANALYSIS_RUN_LIST_SELECT` and `fetch_run_row`:

```sql
runs.project_id,
projects.name AS project_name,
```

Add join:

```sql
LEFT JOIN projects ON projects.id = runs.project_id
```

Add `project_id` to `AnalysisRunListFilters`:

```rust
pub(crate) project_id: Option<i64>,
```

Update `src-tauri/src/analysis/mod.rs::list_analysis_runs` to set this field to
`None` when constructing `AnalysisRunListFilters`, because the legacy global
Runs API does not filter by project:

```rust
project_id: None,
```

Replace the current two-scope validation in `list_analysis_run_summaries` with
the same "at most one scope filter" rule:

```rust
let scope_filter_count = [
    filters.source_id.is_some(),
    filters.source_group_id.is_some(),
    filters.project_id.is_some(),
]
.into_iter()
.filter(|selected| *selected)
.count();
if scope_filter_count > 1 {
    return Err(AppError::validation(
        "Pass only one of source_id, source_group_id, or project_id",
    ));
}
```

Add filtering:

```rust
if let Some(project_id) = filters.project_id {
    query.push(" AND runs.project_id = ");
    query.push_bind(project_id);
}
```

Add `"lower(coalesce(projects.name, ''))"` to `RUN_QUERY_FIELDS`.

Update mapping functions so `scope_label` uses project name/snapshot for project scope:

```rust
fn resolve_run_scope_label_parts(
    scope_type: &str,
    source_id: Option<i64>,
    source_title: Option<&str>,
    source_group_id: Option<i64>,
    source_group_name: Option<&str>,
    project_id: Option<i64>,
    project_name: Option<&str>,
    scope_label_snapshot: Option<&str>,
) -> String {
    if let Some(snapshot) = scope_label_snapshot.filter(|value| !value.trim().is_empty()) {
        return snapshot.to_string();
    }
    if scope_type == crate::analysis::ANALYSIS_SCOPE_TYPE_PROJECT {
        return project_name
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("Project {}", project_id.unwrap_or_default()));
    }
    // keep existing source/source_group behavior below
}
```

Keep existing source and source-group fallback logic intact.

Update store tests and fixtures that create minimal `analysis_runs` schemas:

- add `project_id INTEGER` to every `CREATE TABLE analysis_runs` schema used by
  `insert_analysis_run`, duplicate lookup, and run-list tests;
- add a minimal `projects` table to the run-list test schema before using the
  `LEFT JOIN projects`;
- update `RunListFixture` and `insert_run_list_fixture` with `project_id:
  Option<i64>`;
- update `insert_run_list_fixture` scope type selection to use `"project"` when
  `fixture.project_id.is_some()`, `"source_group"` when
  `fixture.source_group_id.is_some()`, and `"single_source"` otherwise;
- update existing `AnalysisRunListFilters { ... }` literals without
  `..Default::default()` to include `project_id: None`.
- update `sample_run_row()` / `sample_run()` fixtures with `project_id: None`
  and `project_name: None`.

Add a focused store test:

```rust
#[tokio::test]
async fn list_analysis_run_summaries_filters_project_runs() {
    let pool = run_list_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (7, 'Alpha Project'), (8, 'Beta Project')")
        .execute(&pool)
        .await
        .expect("insert projects");
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 1,
            source_id: None,
            source_group_id: None,
            project_id: Some(7),
            scope_label_snapshot: "Alpha Project",
            created_at: 300,
            ..RunListFixture::completed(1, 300, "Alpha Project")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 2,
            source_id: None,
            source_group_id: None,
            project_id: Some(8),
            scope_label_snapshot: "Beta Project",
            created_at: 200,
            ..RunListFixture::completed(2, 200, "Beta Project")
        },
    )
    .await;

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            project_id: Some(7),
            limit: 50,
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list project runs");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    assert_eq!(runs[0].project_name.as_deref(), Some("Alpha Project"));
}
```

- [ ] **Step 5: Add `project_id` to duplicate lookup and inserts**

In `DuplicateRunLookup`:

```rust
pub(crate) project_id: Option<i64>,
```

In the duplicate SQL:

```sql
AND (project_id = ? OR (project_id IS NULL AND ? IS NULL))
```

Bind `lookup.project_id` twice.

In `AnalysisRunInsert`:

```rust
pub(crate) project_id: Option<i64>,
pub(crate) scope_label_snapshot: Option<&'a str>,
```

Add `project_id` to the insert column list and bind it. Change `scope_label_snapshot` from hardcoded `NULL` to a bound value.

Update existing tests in `src-tauri/src/analysis/store.rs` that construct
`AnalysisRunInsert` and `DuplicateRunLookup` directly:

```rust
project_id: None,
scope_label_snapshot: None,
```

For `DuplicateRunLookup`, add only `project_id: None`. For project-specific
duplicate coverage, add a focused test proving a project run and a source-group
run with the same period/template/model do not collide when their scope ids
differ.

- [ ] **Step 6: Extend corpus source resolution**

Change signature in `src-tauri/src/analysis/corpus.rs`:

```rust
pub(crate) async fn resolve_analysis_sources(
    pool: &Pool<Sqlite>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
) -> AppResult<ResolvedAnalysisSources>
```

Replace the current exactly-one validation with:

```rust
let selected_count = [source_id.is_some(), source_group_id.is_some(), project_id.is_some()]
    .into_iter()
    .filter(|selected| *selected)
    .count();
if selected_count != 1 {
    return Err(AppError::validation(
        "Select exactly one analysis scope",
    ));
}
```

For project branch, load sources:

```rust
let rows: Vec<AnalysisSourceScopeRow> = sqlx::query_as(
    r#"
    SELECT s.id, s.source_type, s.source_subtype
    FROM project_sources ps
    JOIN sources s ON s.id = ps.source_id
    WHERE ps.project_id = ?
    ORDER BY ps.added_at ASC, s.id ASC
    "#,
)
.bind(project_id)
.fetch_all(pool)
.await
.map_err(AppError::database)?;

if rows.is_empty() {
    return Err(AppError::validation("Project does not contain any sources"));
}

let first_type = rows[0].source_type.clone();
if rows.iter().any(|row| row.source_type != first_type) {
    return Err(AppError::validation(
        "mixed_provider_project_runs_not_supported",
    ));
}
source_type = first_type;
```

Then reuse the same playlist-expansion logic for each row that source groups use. Extract this helper:

```rust
async fn push_scope_source(
    pool: &Pool<Sqlite>,
    source: AnalysisSourceScopeRow,
    source_ids: &mut Vec<i64>,
    seen_source_ids: &mut std::collections::HashSet<i64>,
    skipped_unlinked_playlist_items: &mut usize,
) -> AppResult<()> {
    if source.source_type == "youtube" && source.source_subtype.as_deref() == Some("playlist") {
        *skipped_unlinked_playlist_items += count_skipped_unlinked_playlist_items(pool, source.id).await?;
        for video_source_id in linked_playlist_video_source_ids(pool, source.id).await? {
            if seen_source_ids.insert(video_source_id) {
                source_ids.push(video_source_id);
            }
        }
    } else if seen_source_ids.insert(source.id) {
        source_ids.push(source.id);
    }
    Ok(())
}
```

Update the test-only `resolve_run_source_ids` helper in the same module so
project-scoped runs can be resolved consistently in tests and future fallback
paths:

```rust
if run.scope_type == crate::analysis::ANALYSIS_SCOPE_TYPE_PROJECT {
    let project_id = run
        .project_id
        .ok_or_else(|| format!("Analysis run {} is missing project_id", run.id))?;
    return resolve_analysis_sources(pool, None, None, Some(project_id))
        .await
        .map(|resolved| resolved.source_ids)
        .map_err(|error| error.to_string());
}
```

Add a corpus test proving `resolve_run_source_ids` returns project source ids
for a project-scoped run with no captured snapshot rows.

- [ ] **Step 7: Extend report request**

In `src-tauri/src/analysis/report.rs`, add to `StartAnalysisReportRequest`:

```rust
pub(crate) project_id: Option<i64>,
```

Update destructuring, validation, and scope resolution:

```rust
let selected_count = [source_id.is_some(), source_group_id.is_some(), project_id.is_some()]
    .into_iter()
    .filter(|selected| *selected)
    .count();
if selected_count != 1 {
    return Err(AppError::validation("Select exactly one analysis scope"));
}
```

Add project branch:

```rust
} else if let Some(project_id) = project_id {
    let project = crate::projects::get_project_in_pool(&pool, project_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found")))?;
    let source_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_sources WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .map_err(AppError::database)?;
    if source_count == 0 {
        return Err(AppError::validation("Project does not contain any sources"));
    }
    (
        ANALYSIS_SCOPE_TYPE_PROJECT,
        None,
        None,
        Some(project.id),
        project.name.clone(),
    )
```

Update tuple names to include `resolved_project_id`, and call:

```rust
let resolved_sources =
    resolve_analysis_sources(&pool, resolved_source_id, resolved_group_id, resolved_project_id).await?;
```

Pass `project_id` and `scope_label_snapshot` into duplicate lookup and insert:

```rust
project_id: resolved_project_id,
scope_label_snapshot: Some(&scope_label),
```

Update all existing `StartAnalysisReportRequest` constructors:

- in `src-tauri/src/analysis/report_commands.rs`, pass `project_id: None` for
  the legacy `start_analysis_report` command;
- in `src-tauri/src/analysis/report.rs` tests, pass `project_id: None`.

- [ ] **Step 8: Add project analysis command wrapper**

In `src-tauri/src/projects.rs`, add:

```rust
#[tauri::command]
#[expect(
    clippy::too_many_arguments,
    reason = "Tauri command signature mirrors start_analysis_report for project scope."
)]
pub async fn start_project_analysis(
    handle: AppHandle,
    state: tauri::State<'_, crate::analysis::AnalysisState>,
    project_id: i64,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<i64> {
    crate::analysis::report::start_analysis_report_run(
        handle,
        state.inner(),
        crate::analysis::report::StartAnalysisReportRequest {
            source_id: None,
            source_group_id: None,
            project_id: Some(project_id),
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            model_override,
            profile_id,
            youtube_corpus_mode,
            include_migrated_history,
        },
    )
    .await
}
```

In `src-tauri/src/analysis/mod.rs`, change `mod report;` to:

```rust
pub(crate) mod report;
```

Register `start_project_analysis` in `src-tauri/src/lib.rs`.

- [ ] **Step 9: Add `list_project_runs` command**

In `src-tauri/src/projects.rs`, add:

```rust
#[tauri::command]
pub async fn list_project_runs(
    handle: AppHandle,
    project_id: i64,
) -> AppResult<Vec<crate::analysis::models::AnalysisRunSummary>> {
    let pool = get_pool(&handle).await?;
    ensure_project_exists(&pool, project_id).await?;
    crate::analysis::store::list_analysis_run_summaries(
        &pool,
        crate::analysis::store::AnalysisRunListFilters {
            source_id: None,
            source_group_id: None,
            project_id: Some(project_id),
            limit: 5,
            query: None,
            status: Some("all".to_string()),
            provider: None,
            model: None,
            template: None,
            date_from: None,
            date_to: None,
        },
    )
    .await
}
```

In `src-tauri/src/analysis/mod.rs`, make `analysis::models` and `analysis::store` visible to sibling modules:

```rust
pub(crate) mod models;
pub(crate) mod store;
```

- [ ] **Step 10: Run backend tests**

Run:

```powershell
cargo test resolve_analysis_sources_ --manifest-path src-tauri/Cargo.toml
cargo test projects::tests --manifest-path src-tauri/Cargo.toml
cargo test analysis::store --manifest-path src-tauri/Cargo.toml
cargo test analysis::report --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 11: Commit**

```powershell
git add src-tauri\src\analysis src-tauri\src\projects.rs src-tauri\src\lib.rs
git commit -m "feat: support project analysis scope"
```

---

### Task 5: Frontend API And Types

**Files:**
- Create: `src/lib/types/projects.ts`
- Create: `src/lib/api/projects.ts`
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Test: `src/lib/api/projects.test.ts`
- Test: `src/lib/api/analysis-runs.test.ts`

- [ ] **Step 1: Write failing API tests**

Create `src/lib/api/projects.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  addProjectSources,
  createProject,
  deleteProject,
  listProjectRuns,
  listProjectSources,
  listProjects,
  removeProjectSources,
  startProjectAnalysis,
  updateProject,
} from "./projects";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

describe("projects api", () => {
  beforeEach(() => invokeMock.mockReset());

  it("maps project crud commands", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listProjects();
    expect(invokeMock).toHaveBeenLastCalledWith("list_projects");

    invokeMock.mockResolvedValueOnce({ id: 1 });
    await createProject({ name: "Alpha", description: "Desc" });
    expect(invokeMock).toHaveBeenLastCalledWith("create_project", {
      name: "Alpha",
      description: "Desc",
    });

    invokeMock.mockResolvedValueOnce({ id: 1 });
    await updateProject({ projectId: 1, name: "Beta", description: null });
    expect(invokeMock).toHaveBeenLastCalledWith("update_project", {
      projectId: 1,
      name: "Beta",
      description: null,
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await deleteProject(1);
    expect(invokeMock).toHaveBeenLastCalledWith("delete_project", { projectId: 1 });
  });

  it("maps source membership and run commands", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listProjectSources(2);
    expect(invokeMock).toHaveBeenLastCalledWith("list_project_sources", { projectId: 2 });

    invokeMock.mockResolvedValueOnce({ added_count: 2, already_present_count: 1 });
    await addProjectSources({ projectId: 2, sourceIds: [5, 6, 5] });
    expect(invokeMock).toHaveBeenLastCalledWith("add_project_sources", {
      projectId: 2,
      sourceIds: [5, 6, 5],
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await removeProjectSources({ projectId: 2, sourceIds: [5] });
    expect(invokeMock).toHaveBeenLastCalledWith("remove_project_sources", {
      projectId: 2,
      sourceIds: [5],
    });

    invokeMock.mockResolvedValueOnce([]);
    await listProjectRuns(2);
    expect(invokeMock).toHaveBeenLastCalledWith("list_project_runs", { projectId: 2 });

    invokeMock.mockResolvedValueOnce(77);
    await startProjectAnalysis({
      projectId: 2,
      periodFrom: 1,
      periodTo: 2,
      outputLanguage: "en",
      promptTemplateId: 3,
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("start_project_analysis", {
      projectId: 2,
      periodFrom: 1,
      periodTo: 2,
      outputLanguage: "en",
      promptTemplateId: 3,
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
  });
});
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/api/projects.test.ts
```

Expected: FAIL because `$lib/api/projects` does not exist.

- [ ] **Step 3: Add project types**

Create `src/lib/types/projects.ts`:

```ts
import type { AnalysisRunSummary, YoutubeCorpusMode } from "$lib/types/analysis";
import type { LibrarySourceProvider, LibrarySourceSubtype } from "$lib/types/library-sources";

export interface ProjectRecord {
  id: number;
  name: string;
  description: string | null;
  created_at: number;
  updated_at: number;
}

export interface ProjectSourceRecord {
  project_id: number;
  source_id: number;
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  title: string | null;
  subtitle: string | null;
  item_count: number;
  added_at: number;
}

export interface AddProjectSourcesOutcome {
  added_count: number;
  already_present_count: number;
}

export interface ProjectEditorInput {
  name: string;
  description: string | null;
}

export interface UpdateProjectInput extends ProjectEditorInput {
  projectId: number;
}

export interface ProjectSourcesInput {
  projectId: number;
  sourceIds: number[];
}

export interface ProjectAnalysisStartCommand {
  projectId: number;
  periodFrom: number;
  periodTo: number;
  outputLanguage: string;
  promptTemplateId: number;
  modelOverride: string | null;
  profileId: string | null;
  youtubeCorpusMode: YoutubeCorpusMode;
  includeMigratedHistory: boolean;
}

export type ProjectRuns = AnalysisRunSummary[];
```

- [ ] **Step 4: Add API wrapper**

Create `src/lib/api/projects.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type {
  AddProjectSourcesOutcome,
  ProjectAnalysisStartCommand,
  ProjectEditorInput,
  ProjectRecord,
  ProjectSourceRecord,
  ProjectSourcesInput,
  ProjectRuns,
  UpdateProjectInput,
} from "$lib/types/projects";

export function listProjects() {
  return invoke<ProjectRecord[]>("list_projects");
}

export function createProject(input: ProjectEditorInput) {
  return invoke<ProjectRecord>("create_project", { ...input });
}

export function updateProject(input: UpdateProjectInput) {
  return invoke<ProjectRecord>("update_project", { ...input });
}

export function deleteProject(projectId: number) {
  return invoke<void>("delete_project", { projectId });
}

export function listProjectSources(projectId: number) {
  return invoke<ProjectSourceRecord[]>("list_project_sources", { projectId });
}

export function addProjectSources(input: ProjectSourcesInput) {
  return invoke<AddProjectSourcesOutcome>("add_project_sources", { ...input });
}

export function removeProjectSources(input: ProjectSourcesInput) {
  return invoke<void>("remove_project_sources", { ...input });
}

export function listProjectRuns(projectId: number) {
  return invoke<ProjectRuns>("list_project_runs", { projectId });
}

export function startProjectAnalysis(command: ProjectAnalysisStartCommand) {
  return invoke<number>("start_project_analysis", { ...command });
}
```

- [ ] **Step 5: Extend analysis run types**

In `src/lib/types/analysis.ts`, add to `AnalysisRunSummary` so
`AnalysisRunDetail extends AnalysisRunSummary` receives the same fields:

```ts
project_id: number | null;
project_name: string | null;
```

Do not add `projectId` to legacy `AnalysisReportStartCommand`; project starts use `ProjectAnalysisStartCommand`.

- [ ] **Step 6: Re-export startProjectAnalysis for callers**

In `src/lib/api/analysis-runs.ts`, add:

```ts
export { startProjectAnalysis } from "$lib/api/projects";
```

- [ ] **Step 7: Run tests**

Run:

```powershell
npm.cmd test -- --run src/lib/api/projects.test.ts src/lib/api/analysis-runs.test.ts
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src\lib\types\projects.ts src\lib\api\projects.ts src\lib\types\analysis.ts src\lib\api\analysis-runs.ts src\lib\api\projects.test.ts src\lib\api\analysis-runs.test.ts
git commit -m "feat: add frontend projects api"
```

---

### Task 6: Project View Model And Workflow

**Files:**
- Modify: `src/lib/ui/research-projects-model.ts`
- Modify: `src/lib/ui/research-projects-workflow.ts`
- Test: `src/lib/ui/research-projects-model.test.ts`
- Test: `src/lib/ui/research-projects-workflow.test.ts`

- [ ] **Step 1: Write failing model tests**

Replace source-group-oriented expectations in `src/lib/ui/research-projects-model.test.ts` with project expectations:

```ts
import { describe, expect, it } from "vitest";
import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  projectRunDisabledReason,
} from "./research-projects-model";
import type { ProjectRecord, ProjectSourceRecord } from "$lib/types/projects";
import type { LibrarySourceRecord } from "$lib/types/library-sources";

const projects: ProjectRecord[] = [
  { id: 1, name: "Alpha", description: "Desc", created_at: 100, updated_at: 200 },
];

const projectSources: ProjectSourceRecord[] = [
  {
    project_id: 1,
    source_id: 10,
    provider: "youtube",
    source_subtype: "video",
    title: "Video",
    subtitle: "Channel",
    item_count: 3,
    added_at: 300,
  },
];

const library: LibrarySourceRecord[] = [
  {
    source_id: 10,
    provider: "youtube",
    source_subtype: "video",
    account_id: null,
    external_id: "v1",
    title: "Video",
    subtitle: "Channel",
    canonical_url: "https://youtu.be/v1",
    created_at: 100,
    last_synced_at: 110,
    item_count: 3,
    project_count: 1,
    youtube: {
      video_form: "video",
      duration_seconds: 120,
      playlist_video_count: null,
      channel_title: "Channel",
      availability_status: "available",
    },
    telegram: null,
  },
];

describe("research projects model", () => {
  it("builds real project cards from project records", () => {
    const rows = buildResearchProjectsView(projects, projectSources, []);
    expect(rows).toMatchObject([
      {
        id: "project:1",
        projectId: 1,
        title: "Alpha",
        description: "Desc",
        sourceCount: 1,
        materialCount: 3,
        status: "ready",
      },
    ]);
  });

  it("marks already connected library sources without hiding them", () => {
    const rows = buildLibrarySourcesView(library, projectSources, "project:1", []);
    expect(rows[0]).toMatchObject({
      sourceId: 10,
      alreadyConnected: true,
      connectable: false,
      disabledReason: "Already in project",
    });
  });

  it("builds project source table rows and disables mixed-provider runs", () => {
    const links = buildProjectSourceLinksView("project:1", projectSources);
    expect(links[0]).toMatchObject({
      projectId: "project:1",
      sourceId: "source:10",
      provider: "youtube",
      title: "Video",
      addedAt: 300,
    });

    expect(projectRunDisabledReason(null, [])).toBe("Select a project");
    expect(projectRunDisabledReason(projects[0], [])).toBe("Add sources to run analysis");
    expect(projectRunDisabledReason(projects[0], projectSources)).toBeNull();
    expect(
      projectRunDisabledReason(projects[0], [
        ...projectSources,
        { ...projectSources[0], source_id: 11, provider: "telegram" },
      ]),
    ).toBe("Mixed-provider project runs are not supported yet.");
  });
});
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/research-projects-model.test.ts
```

Expected: FAIL because model still expects `AnalysisSourceGroup`.

- [ ] **Step 3: Update model types and builders**

In `src/lib/ui/research-projects-model.ts`, replace legacy backing with:

```ts
import type { ProjectRecord, ProjectSourceRecord } from "$lib/types/projects";
import type { LibrarySourceRecord, LibrarySourceProvider } from "$lib/types/library-sources";

export type ResearchProjectView = {
  id: string;
  projectId: number;
  title: string;
  description: string | null;
  sourceCount: number;
  evidenceCount: number;
  materialCount: number;
  lastRunLabel: string | null;
  status: ProjectStatus;
};

export type ProjectSourceLinkView = {
  projectId: string;
  sourceId: string;
  sourceNumericId: number;
  provider: LibrarySourceProvider;
  subtype: string | null;
  title: string;
  subtitle: string | null;
  itemCount: number;
  localCopyLabel: string;
  addedAt: number;
  addedAtLabel: string | null;
  connectionStatus: "connected";
  filterSummary: string;
};

export function projectViewId(projectId: number) {
  return `project:${projectId}`;
}

export function projectIdFromViewId(viewId: string | null) {
  if (!viewId?.startsWith("project:")) return null;
  const value = Number(viewId.slice("project:".length));
  return Number.isFinite(value) ? value : null;
}
```

Implement builders:

```ts
export function buildResearchProjectsView(
  projects: ProjectRecord[],
  projectSources: ProjectSourceRecord[],
  runs: AnalysisRunSummary[] = [],
): ResearchProjectView[] {
  return projects.map((project) => {
    const sources = projectSources.filter((source) => source.project_id === project.id);
    const materialCount = sources.reduce((total, source) => total + source.item_count, 0);
    const latestRun = runs
      .filter((run) => run.project_id === project.id)
      .sort((left, right) => right.created_at - left.created_at)[0];
    const running = runs.some(
      (run) => run.project_id === project.id && (run.status === "queued" || run.status === "running"),
    );
    return {
      id: projectViewId(project.id),
      projectId: project.id,
      title: project.name,
      description: project.description,
      sourceCount: sources.length,
      evidenceCount: materialCount,
      materialCount,
      lastRunLabel: latestRun ? dateLabel(latestRun.created_at) : null,
      status: running ? "running" : sources.length === 0 ? "empty" : "ready",
    };
  });
}
```

Update `buildLibrarySourcesView` to accept `LibrarySourceRecord[]` and `ProjectSourceRecord[]`. Already-connected rows should use exact text:

```ts
const disabledReason = alreadyConnected ? "Already in project" : null;
```

Add run eligibility:

```ts
export function projectRunDisabledReason(
  project: ProjectRecord | ResearchProjectView | null,
  sources: Pick<ProjectSourceRecord, "provider">[],
) {
  if (!project) return "Select a project";
  if (sources.length === 0) return "Add sources to run analysis";
  const providers = new Set(sources.map((source) => source.provider));
  if (providers.size > 1) return "Mixed-provider project runs are not supported yet.";
  return null;
}
```

- [ ] **Step 4: Update workflow tests**

In `src/lib/ui/research-projects-workflow.test.ts`, use deps with `listProjects`, `listProjectSources`, `listLibrarySources`, `listProjectRuns`, `listPromptTemplates`, `addProjectSources`, and `removeProjectSources`. Add this test:

```ts
it("loads projects and connects selected Library sources through project APIs", async () => {
  const state = createInitialState();
  const deps = createDeps(state);
  deps.listProjects.mockResolvedValue([{ id: 1, name: "Alpha", description: null, created_at: 1, updated_at: 1 }]);
  deps.listProjectSources.mockResolvedValue([]);
  deps.listLibrarySources.mockResolvedValue([librarySource({ source_id: 10 })]);
  deps.listProjectRuns.mockResolvedValue([]);
  deps.listPromptTemplates.mockResolvedValue([]);
  deps.listSourceJobs.mockResolvedValue([]);
  deps.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });

  const workflow = createResearchProjectsWorkflow(deps);
  await workflow.loadWorkspace();
  state.selectedLibrarySourceIds = new Set(["source:10"]);
  await workflow.connectSelectedSources();

  expect(deps.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [10] });
  expect(state.status).toContain("Connected sources: 1");
});
```

- [ ] **Step 5: Update workflow implementation**

In `src/lib/ui/research-projects-workflow.ts`, replace state:

```ts
projectsRaw: ProjectRecord[];
projectSources: ProjectSourceRecord[];
runs: AnalysisRunSummary[];
libraryRecords: LibrarySourceRecord[];
promptTemplates: AnalysisPromptTemplate[];
```

Use deps:

```ts
listProjects(): Promise<ProjectRecord[]>;
listProjectSources(projectId: number): Promise<ProjectSourceRecord[]>;
listLibrarySources(): Promise<LibrarySourceRecord[]>;
listProjectRuns(projectId: number): Promise<AnalysisRunSummary[]>;
listPromptTemplates(): Promise<AnalysisPromptTemplate[]>;
addProjectSources(input: ProjectSourcesInput): Promise<AddProjectSourcesOutcome>;
removeProjectSources(input: ProjectSourcesInput): Promise<void>;
createProject(input: ProjectEditorInput): Promise<ProjectRecord>;
updateProject(input: UpdateProjectInput): Promise<ProjectRecord>;
deleteProject(projectId: number): Promise<void>;
startProjectAnalysis(input: ProjectAnalysisStartCommand): Promise<number>;
```

Implement `loadWorkspace()` as:

```ts
const [projectsRaw, libraryRecords, sourceJobs, promptTemplates] = await Promise.all([
  deps.listProjects(),
  deps.listLibrarySources(),
  deps.listSourceJobs(),
  deps.listPromptTemplates(),
]);
const allProjectSources = (
  await Promise.all(projectsRaw.map((project) => deps.listProjectSources(project.id)))
).flat();
const runs = (
  await Promise.all(projectsRaw.map((project) => deps.listProjectRuns(project.id)))
).flat();
deps.patch({ projectsRaw, libraryRecords, projectSources: allProjectSources, runs, sourceJobs, promptTemplates });
```

Implement `connectSelectedSources()` with:

```ts
const projectId = projectIdFromViewId(state.selectedProjectId);
const sourceIds = connectableSelection(state.librarySources, state.selectedLibrarySourceIds)
  .map((source) => source.sourceId);
const outcome = await deps.addProjectSources({ projectId, sourceIds });
deps.patch({
  status: `Connected sources: ${outcome.added_count}. Already in project: ${outcome.already_present_count}.`,
  selectedLibrarySourceIds: new Set<string>(),
});
```

Implement `removeProjectSource(sourceId: number)`:

```ts
const projectId = projectIdFromViewId(deps.getState().selectedProjectId);
if (!projectId) {
  deps.patch({ status: "Select a project" });
  return;
}
await deps.removeProjectSources({ projectId, sourceIds: [sourceId] });
await loadWorkspace();
```

Implement create/edit/delete/run workflow methods:

```ts
async function createProject(input: ProjectEditorInput) {
  deps.patch({ saving: true });
  try {
    const project = await deps.createProject(input);
    deps.patch({ selectedProjectId: projectViewId(project.id), status: "Project created." });
    await loadWorkspace();
  } catch (error) {
    deps.patch({ status: deps.formatError("creating project", error) });
  } finally {
    deps.patch({ saving: false });
  }
}

async function updateProject(input: ProjectEditorInput) {
  const projectId = projectIdFromViewId(deps.getState().selectedProjectId);
  if (!projectId) return deps.patch({ status: "Select a project" });
  deps.patch({ saving: true });
  try {
    await deps.updateProject({ projectId, ...input });
    deps.patch({ status: "Project updated." });
    await loadWorkspace();
  } catch (error) {
    deps.patch({ status: deps.formatError("updating project", error) });
  } finally {
    deps.patch({ saving: false });
  }
}

async function deleteSelectedProject() {
  const projectId = projectIdFromViewId(deps.getState().selectedProjectId);
  if (!projectId) return deps.patch({ status: "Select a project" });
  deps.patch({ saving: true });
  try {
    await deps.deleteProject(projectId);
    deps.patch({ selectedProjectId: null, status: "Project deleted." });
    await loadWorkspace();
  } catch (error) {
    deps.patch({ status: deps.formatError("deleting project", error) });
  } finally {
    deps.patch({ saving: false });
  }
}

async function runProjectAnalysis(input: ProjectAnalysisStartCommand) {
  deps.patch({ saving: true });
  try {
    const runId = await deps.startProjectAnalysis(input);
    deps.patch({ status: `Project analysis queued: ${runId}` });
    await loadWorkspace();
  } catch (error) {
    deps.patch({ status: deps.formatError("starting project analysis", error) });
  } finally {
    deps.patch({ saving: false });
  }
}
```

Expose these methods from `createResearchProjectsWorkflow` together with
`loadWorkspace`, `refreshDerivedState`, `connectSelectedSources`, and
`removeProjectSource`.

- [ ] **Step 6: Run tests**

Run:

```powershell
npm.cmd test -- --run src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src\lib\ui\research-projects-model.ts src\lib\ui\research-projects-model.test.ts src\lib\ui\research-projects-workflow.ts src\lib\ui\research-projects-workflow.test.ts
git commit -m "feat: model projects workspace state"
```

---

### Task 7: Projects Workspace UI

**Files:**
- Modify: `src/routes/projects/+page.svelte`
- Modify: `src/lib/components/research-projects/ProjectsShell.svelte`
- Modify: `src/lib/components/research-projects/ProjectRail.svelte`
- Modify: `src/lib/components/research-projects/ProjectWorkspace.svelte`
- Modify: `src/lib/components/research-projects/SourcesTab.svelte`
- Modify: `src/lib/components/research-projects/ConnectFromLibrary.svelte`
- Create: `src/lib/components/research-projects/ProjectEditorDialog.svelte`
- Create: `src/lib/components/research-projects/ProjectInspector.svelte`
- Create: `src/lib/components/research-projects/ProjectRunDialog.svelte`
- Test: `src/lib/research-projects-route-contract.test.ts`

- [ ] **Step 1: Write failing route contract tests**

In `src/lib/research-projects-route-contract.test.ts`, add:

```ts
import { describe, expect, it } from "vitest";
import pageSource from "../routes/projects/+page.svelte?raw";
import shellSource from "$lib/components/research-projects/ProjectsShell.svelte?raw";
import railSource from "$lib/components/research-projects/ProjectRail.svelte?raw";
import inspectorSource from "$lib/components/research-projects/ProjectInspector.svelte?raw";

describe("projects mvp route contract", () => {
  it("uses real project APIs instead of analysis source group APIs", () => {
    expect(pageSource).toContain("listProjects");
    expect(pageSource).toContain("listProjectSources");
    expect(pageSource).toContain("listLibrarySources");
    expect(pageSource).not.toContain("listAnalysisSourceGroups");
    expect(pageSource).not.toContain("updateAnalysisSourceGroup");
  });

  it("renders three-zone projects workspace", () => {
    expect(shellSource).toContain('data-ui-region="project-rail"');
    expect(shellSource).toContain('data-ui-region="project-workspace"');
    expect(shellSource).toContain('data-ui-region="project-inspector"');
  });

  it("exposes create/edit/delete and run eligibility UI", () => {
    expect(railSource).toContain("Create project");
    expect(inspectorSource).toContain("Run project analysis");
    expect(inspectorSource).toContain("Mixed-provider project runs are not supported yet.");
  });
});
```

- [ ] **Step 2: Run test and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because route still imports source group APIs and shell has no inspector region.

- [ ] **Step 3: Wire route to project APIs**

Modify `src/routes/projects/+page.svelte` imports:

```ts
import {
  addProjectSources,
  createProject,
  deleteProject,
  listProjectRuns,
  listProjectSources,
  listProjects,
  removeProjectSources,
  startProjectAnalysis,
  updateProject,
} from "$lib/api/projects";
import { listLibrarySources } from "$lib/api/library-sources";
import { listAnalysisPromptTemplates } from "$lib/api/analysis-source-groups";
```

Remove `listAnalysisSourceGroups`, `updateAnalysisSourceGroup`, and `listAnalysisSources`.

Initialize state with the new fields from Task 6 and create workflow deps:

```ts
const workflow = createResearchProjectsWorkflow({
  getState: () => state,
  patch: (patch) => Object.assign(state, patch),
  listProjects,
  listProjectSources,
  listLibrarySources,
  listProjectRuns,
  listPromptTemplates: () => listAnalysisPromptTemplates("report"),
  listSourceJobs: () => listSourceJobs({ limit: 50 }),
  addProjectSources,
  removeProjectSources,
  createProject,
  updateProject,
  deleteProject,
  startProjectAnalysis,
  formatError: (action, error) => `Error ${action}: ${String(error)}`,
});
```

Initialize `promptTemplates: []` in route state and pass workflow handlers to
`ProjectsShell`:

```svelte
<ProjectsShell
  {state}
  onSelectProject={selectProject}
  onCreateProject={workflow.createProject}
  onUpdateProject={workflow.updateProject}
  onDeleteProject={workflow.deleteSelectedProject}
  onRemoveProjectSource={workflow.removeProjectSource}
  onRunProject={workflow.runProjectAnalysis}
  onConnectSelectedSources={workflow.connectSelectedSources}
  onSelectedLibrarySourceIdsChange={(ids) => (state.selectedLibrarySourceIds = new Set(ids))}
/>
```

- [ ] **Step 4: Add project editor dialog**

Create `src/lib/components/research-projects/ProjectEditorDialog.svelte`:

```svelte
<script lang="ts">
  import { ExtractumButton, ExtractumDialog, ExtractumTextInput } from "$lib/components/extractum-ui";
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";

  let {
    open = $bindable(false),
    project = null,
    saving = false,
    error = "",
    onSubmit,
  }: {
    open?: boolean;
    project?: ResearchProjectView | null;
    saving?: boolean;
    error?: string;
    onSubmit: (input: { name: string; description: string | null }) => void | Promise<void>;
  } = $props();

  let name = $state(project?.title ?? "");
  let description = $state(project?.description ?? "");

  $effect(() => {
    if (open) {
      name = project?.title ?? "";
      description = project?.description ?? "";
    }
  });

  async function submit() {
    await onSubmit({ name, description: description.trim() || null });
  }
</script>

<ExtractumDialog bind:open title={project ? "Edit project" : "Create project"}>
  <form class="project-editor" onsubmit={(event) => { event.preventDefault(); void submit(); }}>
    <label>
      <span>Name</span>
      <ExtractumTextInput bind:value={name} aria-label="Project name" />
    </label>
    <label>
      <span>Description</span>
      <textarea bind:value={description} aria-label="Project description"></textarea>
    </label>
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => (open = false)}>Cancel</ExtractumButton>
      <ExtractumButton type="submit" disabled={saving || name.trim().length === 0}>
        {project ? "Save" : "Create"}
      </ExtractumButton>
    </footer>
  </form>
</ExtractumDialog>

<style>
  .project-editor {
    display: flex;
    min-width: 420px;
    flex-direction: column;
    gap: 12px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  label span {
    color: var(--extractum-muted);
    font-size: 12px;
  }
  textarea {
    min-height: 96px;
    resize: vertical;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    padding: 8px;
  }
  .error {
    color: var(--extractum-danger);
    font-size: 13px;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
```

`ExtractumDialog` exposes bindable `open`; parent components should use `bind:open` with `ProjectEditorDialog`.

- [ ] **Step 5: Add project inspector**

Create `src/lib/components/research-projects/ProjectInspector.svelte`:

```svelte
<script lang="ts">
  import { Play, Pencil, Trash2 } from "@lucide/svelte";
  import { ExtractumButton, ProviderBadge, StatusBadge } from "$lib/components/extractum-ui";
  import { projectRunDisabledReason } from "$lib/ui/research-projects-model";
  import type { ProjectSourceRecord } from "$lib/types/projects";
  import type { AnalysisRunSummary } from "$lib/types/analysis";
  import type { ResearchProjectView, ProjectSourceLinkView } from "$lib/ui/research-projects-model";

  let {
    project,
    sources,
    selectedSource,
    runs,
    saving = false,
    onEditProject,
    onDeleteProject,
    onRunProject,
    onRemoveSource,
  }: {
    project: ResearchProjectView | null;
    sources: ProjectSourceRecord[];
    selectedSource: ProjectSourceLinkView | null;
    runs: AnalysisRunSummary[];
    saving?: boolean;
    onEditProject: () => void;
    onDeleteProject: () => void | Promise<void>;
    onRunProject: () => void;
    onRemoveSource: (sourceId: number) => void | Promise<void>;
  } = $props();

  const runDisabledReason = $derived(projectRunDisabledReason(project, sources));
  const providerBreakdown = $derived(Array.from(new Set(sources.map((source) => source.provider))));
</script>

<aside class="project-inspector-panel">
  <section>
    <span class="eyebrow">Project</span>
    <h2>{project?.title ?? "No project selected"}</h2>
    <p>{project?.description ?? "Create or select a project."}</p>
    <dl>
      <div><dt>Sources</dt><dd>{sources.length}</dd></div>
      <div><dt>Providers</dt><dd>{providerBreakdown.length}</dd></div>
    </dl>
    <div class="provider-row">
      {#each providerBreakdown as provider (provider)}
        <ProviderBadge {provider} />
      {/each}
    </div>
  </section>

  <section>
    <h3>Actions</h3>
    {#if runDisabledReason}
      <p class="hint">{runDisabledReason}</p>
    {/if}
    <ExtractumButton disabled={saving || runDisabledReason !== null} onclick={onRunProject}>
      <Play size={14} aria-hidden="true" />
      Run project analysis
    </ExtractumButton>
    <ExtractumButton variant="outline" disabled={!project || saving} onclick={onEditProject}>
      <Pencil size={14} aria-hidden="true" />
      Edit project
    </ExtractumButton>
    <ExtractumButton variant="outline" disabled={!project || saving} onclick={onDeleteProject}>
      <Trash2 size={14} aria-hidden="true" />
      Delete project
    </ExtractumButton>
  </section>

  {#if selectedSource}
    <section>
      <h3>Selected source</h3>
      <p><strong>{selectedSource.title}</strong></p>
      <p>{selectedSource.subtitle ?? selectedSource.filterSummary}</p>
      <StatusBadge status="connected" />
      <ExtractumButton variant="outline" disabled={saving} onclick={() => onRemoveSource(selectedSource.sourceNumericId)}>
        Remove from project
      </ExtractumButton>
    </section>
  {/if}

  <section>
    <h3>Recent runs</h3>
    {#each runs.slice(0, 5) as run (run.id)}
      <p>{run.scope_label} - {run.status}</p>
    {:else}
      <p class="hint">No project runs</p>
    {/each}
  </section>
</aside>

<style>
  .project-inspector-panel {
    display: flex;
    min-height: 0;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    border-left: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
    overflow: auto;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 12px;
  }
  h2, h3, p, dl {
    margin: 0;
  }
  .eyebrow, .hint, dt {
    color: var(--extractum-muted);
    font-size: 12px;
  }
  dl {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .provider-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
</style>
```

- [ ] **Step 6: Update shell layout**

Modify `ProjectsShell.svelte`:

- add state for editor dialog, run dialog, selected source id;
- change grid columns to `260px minmax(0, 1fr) 380px`;
- pass `state.promptTemplates` into `ProjectRunDialog`;
- render `<ProjectInspector data-ui-region="project-inspector" ... />`;
- render `<ProjectEditorDialog ... />`;
- render `<ProjectRunDialog ... />`.

Use props:

```ts
onCreateProject: (input: { name: string; description: string | null }) => void | Promise<void>;
onUpdateProject: (input: { name: string; description: string | null }) => void | Promise<void>;
onDeleteProject: () => void | Promise<void>;
onRemoveProjectSource: (sourceId: number) => void | Promise<void>;
onRunProject: (input: ProjectAnalysisStartCommand) => void | Promise<void>;
```

- [ ] **Step 7: Update ProjectRail**

Add a create button in `ProjectRail.svelte`:

```svelte
<ExtractumButton data-ui-action="create-project" onclick={onCreateProject}>
  Create project
</ExtractumButton>
```

Pass `onCreateProject` from shell.

- [ ] **Step 8: Update SourcesTab**

Use `ProjectSourceLinkView.sourceNumericId`, `subtype`, `addedAtLabel`, and add a remove button column:

```ts
const columns = [
  { id: "title", header: "Title", flexgrow: 1, cell: LibrarySourceCell },
  { id: "provider", header: "Provider", width: 120 },
  { id: "subtype", header: "Subtype", width: 120 },
  { id: "localCopyLabel", header: "Details", width: 140 },
  { id: "addedAtLabel", header: "Added to project at", width: 180 },
];
```

Pass selected row ids through the existing `ExtractumDataGrid` wrapper-managed selection:

```svelte
<ExtractumDataGrid
  rows={rows}
  {columns}
  selectedRowIds={selectedSourceId ? [selectedSourceId] : []}
  onSelectedRowIdsChange={(ids) => onSelectedSourceIdChange(ids[0] ?? null)}
  height="100%"
  overlay="No project sources"
/>
```

- [ ] **Step 9: Add ProjectRunDialog**

Create `ProjectRunDialog.svelte` with the current report-run fields:

```svelte
<script lang="ts">
  import { ExtractumButton, ExtractumDialog, ExtractumTextInput } from "$lib/components/extractum-ui";
  import type { AnalysisPromptTemplate, YoutubeCorpusMode } from "$lib/types/analysis";
  import type { ProjectAnalysisStartCommand } from "$lib/types/projects";
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";
  import { endOfDayUnix, startOfDayUnix } from "$lib/analysis-utils";

  let {
    open = $bindable(false),
    project,
    templates,
    saving = false,
    onSubmit,
  }: {
    open?: boolean;
    project: ResearchProjectView | null;
    templates: AnalysisPromptTemplate[];
    saving?: boolean;
    onSubmit: (input: ProjectAnalysisStartCommand) => void | Promise<void>;
  } = $props();

  let periodFrom = $state(new Date().toISOString().slice(0, 10));
  let periodTo = $state(new Date().toISOString().slice(0, 10));
  let outputLanguage = $state("en");
  let selectedTemplateId = $state("");
  let youtubeCorpusMode = $state<YoutubeCorpusMode>("transcript_description");

  $effect(() => {
    if (!selectedTemplateId && templates[0]) selectedTemplateId = String(templates[0].id);
  });

  async function submit() {
    if (!project) return;
    await onSubmit({
      projectId: project.projectId,
      periodFrom: startOfDayUnix(periodFrom),
      periodTo: endOfDayUnix(periodTo),
      outputLanguage,
      promptTemplateId: Number(selectedTemplateId),
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode,
      includeMigratedHistory: false,
    });
  }
</script>

<ExtractumDialog bind:open title="Run project analysis">
  <form class="run-dialog" onsubmit={(event) => { event.preventDefault(); void submit(); }}>
    <p>{project?.title ?? "No project selected"}</p>
    <label><span>From</span><ExtractumTextInput type="date" bind:value={periodFrom} /></label>
    <label><span>To</span><ExtractumTextInput type="date" bind:value={periodTo} /></label>
    <label><span>Output language</span><ExtractumTextInput bind:value={outputLanguage} /></label>
    <label>
      <span>Prompt</span>
      <select bind:value={selectedTemplateId} aria-label="Prompt template">
        {#each templates as template (template.id)}
          <option value={String(template.id)}>{template.name}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>YouTube corpus</span>
      <select bind:value={youtubeCorpusMode} aria-label="YouTube corpus mode">
        <option value="transcript_only">Transcript only</option>
        <option value="transcript_description">Transcript and description</option>
        <option value="transcript_description_comments">Transcript, description and comments</option>
      </select>
    </label>
    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => (open = false)}>Cancel</ExtractumButton>
      <ExtractumButton type="submit" disabled={!project || !selectedTemplateId || saving}>Run</ExtractumButton>
    </footer>
  </form>
</ExtractumDialog>
```

Use local `<select>` elements in this MVP, matching the existing pattern in `TopCommandBar.svelte`.

- [ ] **Step 10: Run frontend tests and Svelte check**

Run:

```powershell
npm.cmd test -- --run src/lib/research-projects-route-contract.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts
npm.cmd run check
```

Expected: PASS and Svelte check reports 0 errors.

- [ ] **Step 11: Commit**

```powershell
git add src\routes\projects\+page.svelte src\lib\components\research-projects src\lib\research-projects-route-contract.test.ts
git commit -m "feat: replace projects workspace with real projects"
```

---

### Task 8: Global Runs Project Scope Support

**Files:**
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-viewer.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
- Modify: `src/lib/components/analysis/run-companion-tabs.svelte`
- Modify: `src/lib/components/analysis/report-run-header.svelte`
- Modify: `src/lib/source-browser-model.ts`
- Test: `src/lib/analysis-utils.test.ts`
- Test: `src/lib/analysis-run-companion-tabs.test.ts`
- Test: `src/lib/source-browser-model.test.ts`
- Test: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Write failing tests**

In `src/lib/analysis-utils.test.ts`, add:

```ts
it("labels project analysis runs from scope label", () => {
  const run = runSummary({
    scope_type: "project",
    project_id: 7,
    project_name: "Alpha",
    scope_label: "Alpha",
    source_id: null,
    source_title: null,
    source_group_id: null,
    source_group_name: null,
  });
  expect(runTargetLabel(run)).toBe("Alpha");
});
```

- [ ] **Step 2: Run test and verify failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-utils.test.ts
```

Expected: FAIL until project scope is handled.

- [ ] **Step 3: Update labels**

In `src/lib/analysis-utils.ts`, update `runTargetLabel` to include project fields in the accepted pick type and add:

```ts
if (run.scope_type === "project") {
  return run.scope_label || run.project_name || `Project #${run.project_id ?? "unknown"}`;
}
```

Do not remove existing single-source/source-group branches.

- [ ] **Step 4: Update run list UI copy**

In run list/header components, make labels scope-neutral:

- replace "Source group" only labels with "Scope" where the run can be project-scoped;
- show `run.scope_label`;
- include `run.provider` and `run.model` as before.
- update every `runTargetLabel` prop type in these files to include `"project_id" | "project_name"` in the `Pick<AnalysisRun...>` key list:
  - `src/lib/components/analysis/report-canvas.svelte`
  - `src/lib/components/analysis/report-viewer.svelte`
  - `src/lib/components/analysis/report-run-header.svelte`
  - `src/lib/components/analysis/run-companion-tabs.svelte`
  - `src/lib/components/analysis/run-companion-runs-tab.svelte`

- [ ] **Step 5: Make run snapshot source browser project-aware**

Project snapshots are multi-source snapshots, so they must not be rendered as
single-source snapshots.

In `src/lib/source-browser-model.ts`:

- extend `RunSnapshotBrowserSubject.scopeType` to `"source" | "source_group" | "project"`;
- update `deriveRunSnapshotBrowserKind` so `scopeType === "project"` returns
  `"source_group"` for the MVP multi-source browser layout.

In `src/lib/components/analysis/report-source-surface.svelte`:

- set run snapshot `scopeType` from `currentRun.scope_type` with explicit
  project handling:

```ts
scopeType: currentRun.scope_type === "project"
  ? "project"
  : currentRun.scope_type === "source_group"
    ? "source_group"
    : "source",
```

- use `"Project sources"` instead of `"Source material"` for project-scoped run
  snapshot headers;
- keep source-group behavior unchanged.

Add tests:

- in `src/lib/source-browser-model.test.ts`, assert that
  `deriveRunSnapshotBrowserKind({ scopeType: "project", ... })` returns
  `"source_group"` and exposes `["sources", "items", "metadata"]` tabs;
- in `src/lib/analysis-source-readers.test.ts`, assert
  `report-source-surface.svelte` contains explicit `currentRun.scope_type ===
  "project"` handling.

- [ ] **Step 6: Run tests**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-utils.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src\lib\types\analysis.ts src\lib\analysis-utils.ts src\lib\source-browser-model.ts src\lib\components\analysis src\lib\analysis-utils.test.ts src\lib\analysis-run-companion-tabs.test.ts src\lib\source-browser-model.test.ts src\lib\analysis-source-readers.test.ts
git commit -m "feat: show project-scoped analysis runs"
```

---

### Task 9: Full Verification And Manual Smoke

**Files:**
- No planned source edits.

- [ ] **Step 1: Run focused backend tests**

Run:

```powershell
cargo test projects::tests --manifest-path src-tauri/Cargo.toml
cargo test resolve_analysis_sources_ --manifest-path src-tauri/Cargo.toml
cargo test analysis::report --manifest-path src-tauri/Cargo.toml
cargo test library_sources::tests --manifest-path src-tauri/Cargo.toml
cargo test delete_source_is_blocked_when_source_is_used_by_project --manifest-path src-tauri/Cargo.toml
```

Expected: all PASS.

- [ ] **Step 2: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- --run src/lib/api/projects.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/research-projects-route-contract.test.ts
npm.cmd test -- --run src/lib/analysis-utils.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: all PASS.

- [ ] **Step 3: Run whole-project checks**

Run:

```powershell
npm.cmd run check
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: Svelte check has 0 errors; cargo tests pass.

- [ ] **Step 4: Manual app smoke with Tauri MCP bridge**

Start the app with the usual development command for this repo. If the app is already running with MCP bridge, connect with:

```text
driver_session start
```

Manual smoke steps:

1. Open `/projects`.
2. Create project `MVP Smoke Project`.
3. Confirm it appears in `ProjectRail`.
4. Open `Add from Library`.
5. Add one YouTube source.
6. Reopen `Add from Library` and confirm that source is visible as disabled `Already in project`.
7. Confirm Project Inspector shows one source and enables `Run project analysis`.
8. Add a Telegram source to the same project.
9. Confirm Project Inspector disables run with `Mixed-provider project runs are not supported yet.`
10. Remove the Telegram source.
11. Open Run Project Analysis dialog and verify period, prompt, output language, and YouTube corpus fields are visible.
12. Delete the project and confirm Library sources remain.

- [ ] **Step 5: Record verification**

Create a short verification note:

```powershell
New-Item -ItemType Directory -Force docs\superpowers\verification
```

Create `docs/superpowers/verification/2026-06-13-projects-mvp.md` with:

```markdown
# Projects MVP Verification

Date: 2026-06-13

## Automated

- `cargo test projects::tests --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test resolve_analysis_sources_ --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test analysis::report --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test library_sources::tests --manifest-path src-tauri/Cargo.toml`: PASS
- `npm.cmd test -- --run src/lib/api/projects.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/research-projects-route-contract.test.ts`: PASS
- `npm.cmd test -- --run src/lib/analysis-utils.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts`: PASS
- `npm.cmd run check`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS

## Manual Smoke

- Created project: PASS
- Added source from Library: PASS
- Already-in-project row disabled: PASS
- Mixed-provider run disabled: PASS
- Single-provider run dialog opens: PASS
- Project delete keeps Library sources: PASS
```

- [ ] **Step 6: Commit verification**

```powershell
git add docs\superpowers\verification\2026-06-13-projects-mvp.md
git commit -m "test: verify projects mvp"
```

---

## Plan Self-Review

- Spec coverage:
  - New `projects` and `project_sources`: Task 1 and Task 2.
  - `analysis_runs.project_id` and `scope_type = "project"`: Task 1 and Task 4.
  - Hard delete project plus project runs, keep sources: Task 2.
  - Block source deletion while used by projects: Task 3.
  - Library project counts from real projects: Task 3.
  - `/projects` as three-zone workspace: Task 7.
  - Add from Library modal with already-connected disabled rows: Task 6 and Task 7.
  - Single-provider project analysis and mixed-provider rejection: Task 4 and Task 7.
  - Global Reports/Runs visibility: Task 8.
  - Project chat, audit log, legacy migration, separate `project_runs`: excluded from tasks.
- Placeholder scan:
  - No placeholder markers or unspecified implementation steps are intentionally left.
  - Any wrapper API mismatch is handled by checking the existing wrapper file in the named step.
- Type consistency:
  - Backend command names match frontend invoke names.
  - UI view ids use `project:<id>` and source row ids use `source:<id>`.
  - Project run starts through `start_project_analysis`, not legacy `start_analysis_report`.
