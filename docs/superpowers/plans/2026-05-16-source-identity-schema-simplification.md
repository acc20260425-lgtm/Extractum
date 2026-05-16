# Source Identity Schema Simplification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move normal Telegram source identity from legacy `sources.telegram_source_kind` and compressed `sources.metadata_zstd` into canonical `sources.source_subtype` plus typed `telegram_sources`, while preserving source ids and current product behavior.

**Architecture:** Add a soft SQL migration for typed tables and safe indexes, then run an idempotent Rust startup repair that validates existing `sources` rows, backfills `telegram_sources`, creates the canonical Telegram unique index only after duplicate preflight, and records non-fatal repair notes. Normal source commands are gated by repair state: the app starts, but source commands return the typed startup repair error until repair succeeds. Runtime Telegram sync, Takeout, topics, list, and add flows read typed identity instead of legacy metadata, while DTOs keep `telegram_source_kind` as a deprecated mirror for one compatibility window.

**Tech Stack:** Rust 2021, Tauri 2, SQLx SQLite, tauri-plugin-sql migrations, grammers Telegram client, Svelte 5, TypeScript, Vitest.

---

## Pre-Implementation Branch Rule

No implementation code changes happen on `main`. The plan itself may live on
`main`; before Task 1 code edits, create an isolated branch/worktree.

Preferred execution setup:

```powershell
git status --short --branch
git worktree add .worktrees/source-identity-schema -b feature/source-identity-schema
Set-Location .worktrees/source-identity-schema
git status --short --branch
```

Expected:

```text
## main
Preparing worktree (new branch 'feature/source-identity-schema')
HEAD is now at <current-sha> <current-subject>
## feature/source-identity-schema
```

If `git worktree add` is blocked by the execution environment, use the current
checkout but still create the branch before code edits:

```powershell
git status --short --branch
git switch -c feature/source-identity-schema
git status --short --branch
```

Expected:

```text
## main
Switched to a new branch 'feature/source-identity-schema'
## feature/source-identity-schema
```

---

## File Structure

Create:

- `src-tauri/migrations/18.sql`: soft schema bridge for `telegram_sources`,
  `source_identity_repair_notes`, safe indexes, and simple subtype backfills.
- `src-tauri/src/sources/identity.rs`: canonical source identity and typed
  Telegram identity structs, enum conversions, normalization helpers, loaders,
  and typed-table write helpers.
- `src-tauri/src/sources/identity_repair.rs`: dry-run/apply repair engine,
  startup repair state, command gate helper, repair report DTOs, duplicate
  diagnostics, and repair tests.

Modify:

- `src-tauri/src/migrations.rs`: register migration 18 and add migration
  coverage tests.
- `src-tauri/src/lib.rs`: manage repair state, run startup repair after SQL
  plugin migrations, register dry-run/status commands, and gate source
  commands through their command handlers.
- `src-tauri/src/sources/mod.rs`: expose identity and repair modules.
- `src-tauri/src/sources/types.rs`: make Telegram subtype parsing explicit via
  `TelegramSourceKind::from_source_subtype`; keep legacy `parse` as a
  compatibility wrapper during the transition.
- `src-tauri/src/sources/test_support.rs`: add `telegram_sources`,
  `source_identity_repair_notes`, and canonical Telegram index helpers to
  in-memory fixtures.
- `src-tauri/src/sources/store.rs`: switch add/list/load DTO paths to canonical
  subtype and typed identity; keep `telegram_source_kind` only as DTO mirror.
- `src-tauri/src/sources/peer_resolution.rs`: normal peer resolution reads
  `TelegramSourceIdentity`; legacy metadata decode remains available to repair.
- `src-tauri/src/sources/sync.rs`: use typed peer resolution and update
  `telegram_sources.avatar_cache_key`, not `sources.metadata_zstd`.
- `src-tauri/src/sources/topics.rs`: use typed Telegram subtype for supergroup
  behavior.
- `src-tauri/src/takeout_import/mod.rs`,
  `src-tauri/src/takeout_import/pagination.rs`,
  `src-tauri/src/takeout_import/export_dc.rs`: pass Telegram subtype from typed
  identity instead of reading the legacy column.
- `src-tauri/src/youtube/preview.rs`,
  `src-tauri/src/youtube/jobs.rs`,
  `src-tauri/src/youtube/detail.rs`: ensure source reads use canonical subtype
  and source-command gate where they touch persisted sources.
- `src-tauri/src/notebooklm_export/query.rs`: stop selecting or trusting
  legacy Telegram kind as source identity during export source loading.
- `src/lib/api/sources.ts`: remove persisted-source fallback from
  `source_subtype` to `telegram_source_kind`.
- `src/lib/types/sources.ts`: document `telegramSourceKind` as deprecated
  compatibility data for persisted `Source`.
- `src/lib/source-capabilities.ts`: derive Telegram behavior from
  `sourceSubtype`.
- `src/lib/components/analysis/source-management-dialog.svelte`: use
  canonical persisted subtype for existing-source keys; live dialog rows may
  keep `telegramSourceKind`.
- `docs/database-schema.md`, `docs/architecture-deep-dive.md`,
  `docs/backlog.md`: update after code is verified.

---

### Task 0: Baseline And Branch Guard

**Files:**
- No source edits.

- [x] **Step 1: Confirm clean main before branch creation**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## main
```

- [x] **Step 2: Create the implementation branch/worktree**

Run the preferred setup from "Pre-Implementation Branch Rule":

```powershell
git worktree add .worktrees/source-identity-schema -b feature/source-identity-schema
Set-Location .worktrees/source-identity-schema
git status --short --branch
```

Expected:

```text
## feature/source-identity-schema
```

- [x] **Step 3: Run baseline frontend tests**

Run:

```powershell
npm test
```

Expected:

```text
Test Files  <all passed count> passed
Tests       <all passed count> passed
```

If this fails, stop and record the failing test names before implementation.

- [x] **Step 4: Run baseline frontend type check**

Run:

```powershell
npm run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

If existing warnings appear, record them before implementation.

- [x] **Step 5: Run baseline Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

If dependency download is blocked by the sandbox, rerun with approval according
to the active escalation rules.

- [x] **Step 6: Commit no changes**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## feature/source-identity-schema
```

No commit is created for this task.

---

### Task 1: SQL Schema Bridge

**Files:**
- Create: `src-tauri/migrations/18.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Write failing migration registration tests**

Add these tests to the existing `#[cfg(test)] mod tests` in
`src-tauri/src/migrations.rs`:

```rust
#[test]
fn includes_source_identity_schema_bridge_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 18)
        .expect("version 18 migration is registered");

    for fragment in [
        "CREATE TABLE IF NOT EXISTS telegram_sources",
        "source_identity_repair_notes",
        "idx_telegram_sources_account_peer",
        "idx_telegram_sources_account_subtype",
        "idx_telegram_sources_account_username",
        "SET source_subtype = telegram_source_kind",
    ] {
        assert!(
            migration.sql.contains(fragment),
            "missing migration fragment {fragment}"
        );
    }
}

#[test]
fn source_identity_schema_bridge_does_not_sql_backfill_typed_identity() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 18)
        .expect("version 18 migration is registered");

    let forbidden_fragments = [
        "INSERT INTO telegram_sources",
        "INSERT OR IGNORE INTO telegram_sources",
        "CAST(external_id",
        "GLOB",
        "idx_sources_unique_telegram_identity",
    ];

    for fragment in forbidden_fragments {
        assert!(
            !migration.sql.contains(fragment),
            "migration 18 must not contain {fragment}"
        );
    }
}
```

- [x] **Step 2: Run the failing migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_source_identity_schema_bridge_migration migrations::tests::source_identity_schema_bridge_does_not_sql_backfill_typed_identity
```

Expected:

```text
includes_source_identity_schema_bridge_migration FAILED
```

- [x] **Step 3: Add migration 18 SQL**

Create `src-tauri/migrations/18.sql`:

```sql
UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype IS NULL
  AND telegram_source_kind IN ('channel', 'supergroup', 'group');

UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype NOT IN ('channel', 'supergroup', 'group')
  AND telegram_source_kind IN ('channel', 'supergroup', 'group');

CREATE TABLE IF NOT EXISTS source_identity_repair_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    issue_code TEXT NOT NULL,
    detail TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    UNIQUE(source_id, issue_code)
);

CREATE TABLE IF NOT EXISTS telegram_sources (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    account_id INTEGER NOT NULL,
    source_subtype TEXT NOT NULL,
    peer_kind TEXT NOT NULL,
    peer_id INTEGER NOT NULL,
    resolution_strategy TEXT NOT NULL,
    username TEXT,
    access_hash INTEGER,
    avatar_cache_key TEXT,
    identity_refreshed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
    CHECK (peer_kind IN ('channel', 'chat')),
    CHECK (
        (source_subtype IN ('channel', 'supergroup') AND peer_kind = 'channel')
        OR
        (source_subtype = 'group' AND peer_kind = 'chat')
    ),
    CHECK (resolution_strategy IN ('username', 'dialog', 'legacy_metadata', 'unknown'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_telegram_sources_account_peer
    ON telegram_sources(account_id, peer_kind, peer_id);

CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_subtype
    ON telegram_sources(account_id, source_subtype);

CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_username
    ON telegram_sources(account_id, username)
    WHERE username IS NOT NULL;
```

- [x] **Step 4: Register migration 18**

In `src-tauri/src/migrations.rs`, append to `build_migrations()` after version
17:

```rust
Migration {
    version: 18,
    description: "add source identity bridge schema",
    sql: include_str!("../migrations/18.sql"),
    kind: MigrationKind::Up,
},
```

- [x] **Step 5: Extend in-memory source fixture schema**

In `src-tauri/src/sources/test_support.rs`, after creating `sources`, create
the two new tables and safe indexes by executing the same schema fragments used
in migration 18.

Add helper functions:

```rust
pub(crate) async fn create_source_identity_tables(pool: &sqlx::SqlitePool) {
    sqlx::query(include_str!("../../../migrations/18.sql"))
        .execute(pool)
        .await
        .expect("create source identity bridge schema");
}

pub(crate) async fn create_canonical_telegram_identity_index(pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
            ON sources(account_id, source_type, source_subtype, external_id)
            WHERE source_type = 'telegram'
        "#,
    )
    .execute(pool)
    .await
    .expect("create canonical telegram identity index");
}
```

- [x] **Step 6: Run migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_source_identity_schema_bridge_migration migrations::tests::source_identity_schema_bridge_does_not_sql_backfill_typed_identity
```

Expected:

```text
test result: ok. 2 passed; 0 failed
```

- [x] **Step 7: Commit schema bridge**

Run:

```powershell
git add src-tauri/migrations/18.sql src-tauri/src/migrations.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: add source identity bridge schema"
```

---

### Task 2: Typed Identity Types And Helpers

**Files:**
- Create: `src-tauri/src/sources/identity.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [x] **Step 1: Write failing enum/helper tests**

Add tests in `src-tauri/src/sources/types.rs`:

```rust
#[test]
fn telegram_source_kind_parses_from_canonical_source_subtype() {
    assert_eq!(
        TelegramSourceKind::from_source_subtype("channel").unwrap(),
        TelegramSourceKind::Channel
    );
    assert_eq!(
        TelegramSourceKind::from_source_subtype("supergroup").unwrap(),
        TelegramSourceKind::Supergroup
    );
    assert_eq!(
        TelegramSourceKind::from_source_subtype("group").unwrap(),
        TelegramSourceKind::Group
    );
}

#[test]
fn telegram_source_kind_rejects_unsupported_source_subtype() {
    let error = TelegramSourceKind::from_source_subtype("video").expect_err("unsupported subtype");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
}
```

- [x] **Step 2: Run failing helper tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::types::tests::telegram_source_kind_parses_from_canonical_source_subtype sources::types::tests::telegram_source_kind_rejects_unsupported_source_subtype
```

Expected:

```text
no function or associated item named `from_source_subtype`
```

- [x] **Step 3: Add canonical subtype parser**

In `src-tauri/src/sources/types.rs`, add:

```rust
pub(crate) fn from_source_subtype(value: &str) -> crate::error::AppResult<Self> {
    match value {
        TELEGRAM_KIND_CHANNEL => Ok(Self::Channel),
        TELEGRAM_KIND_SUPERGROUP => Ok(Self::Supergroup),
        TELEGRAM_KIND_GROUP => Ok(Self::Group),
        other => Err(crate::error::AppError::validation(format!(
            "Unsupported Telegram source_subtype '{other}'"
        ))),
    }
}
```

Change `parse` to delegate:

```rust
pub(crate) fn parse(value: &str) -> crate::error::AppResult<Self> {
    Self::from_source_subtype(value)
}
```

- [x] **Step 4: Create typed identity module**

Create `src-tauri/src/sources/identity.rs`:

```rust
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::Serialize;

use crate::error::{AppError, AppResult};

use super::types::{
    now_secs, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
    TELEGRAM_KIND_SUPERGROUP, TELEGRAM_SOURCE_TYPE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceIdentity {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) source_subtype: String,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelegramPeerKind {
    Channel,
    Chat,
}

impl TelegramPeerKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Channel => "channel",
            Self::Chat => "chat",
        }
    }

    pub(crate) fn parse(value: &str) -> AppResult<Self> {
        match value {
            "channel" => Ok(Self::Channel),
            "chat" => Ok(Self::Chat),
            other => Err(AppError::validation(format!(
                "Unsupported Telegram peer_kind '{other}'"
            ))),
        }
    }

    pub(crate) fn from_source_subtype(subtype: TelegramSourceKind) -> Self {
        match subtype {
            TelegramSourceKind::Channel | TelegramSourceKind::Supergroup => Self::Channel,
            TelegramSourceKind::Group => Self::Chat,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelegramResolutionStrategy {
    Username,
    Dialog,
    LegacyMetadata,
    Unknown,
}

impl TelegramResolutionStrategy {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Username => "username",
            Self::Dialog => "dialog",
            Self::LegacyMetadata => "legacy_metadata",
            Self::Unknown => "unknown",
        }
    }

    pub(crate) fn parse(value: &str) -> AppResult<Self> {
        match value {
            "username" => Ok(Self::Username),
            "dialog" => Ok(Self::Dialog),
            "legacy_metadata" => Ok(Self::LegacyMetadata),
            "unknown" => Ok(Self::Unknown),
            other => Err(AppError::validation(format!(
                "Unsupported Telegram resolution_strategy '{other}'"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramSourceIdentity {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) source_subtype: TelegramSourceKind,
    pub(crate) peer_kind: TelegramPeerKind,
    pub(crate) peer_id: i64,
    pub(crate) resolution_strategy: TelegramResolutionStrategy,
    pub(crate) username: Option<String>,
    pub(crate) access_hash: Option<i64>,
    pub(crate) avatar_cache_key: Option<String>,
}

impl TelegramSourceIdentity {
    pub(crate) fn peer_ref(&self) -> AppResult<Option<PeerRef>> {
        match (self.peer_kind, self.source_subtype, self.access_hash) {
            (
                TelegramPeerKind::Channel,
                TelegramSourceKind::Channel | TelegramSourceKind::Supergroup,
                Some(access_hash),
            ) => Ok(Some(PeerRef {
                id: PeerId::channel(self.peer_id),
                auth: PeerAuth::from_hash(access_hash),
            })),
            (TelegramPeerKind::Chat, TelegramSourceKind::Group, _) => Ok(None),
            _ => Err(AppError::validation(format!(
                "Source {} has inconsistent Telegram typed identity",
                self.source_id
            ))),
        }
    }
}

pub(crate) fn canonical_telegram_external_id(value: &str) -> AppResult<i64> {
    let parsed = value.parse::<i64>().map_err(|_| {
        AppError::validation(format!("Malformed Telegram external_id for source identity"))
    })?;
    if parsed < 0 || parsed.to_string() != value {
        return Err(AppError::validation(format!(
            "Malformed Telegram external_id for source identity"
        )));
    }
    Ok(parsed)
}

pub(crate) fn normalize_telegram_username(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    let stripped = raw
        .strip_prefix("https://t.me/")
        .or_else(|| raw.strip_prefix("http://t.me/"))
        .or_else(|| raw.strip_prefix("t.me/"))
        .unwrap_or(raw)
        .trim_start_matches('@')
        .split(['/', '?'])
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

pub(crate) fn ensure_telegram_source_type(identity: &SourceIdentity) -> AppResult<()> {
    if identity.source_type == TELEGRAM_SOURCE_TYPE {
        Ok(())
    } else {
        Err(AppError::validation(format!(
            "Source {} is not a Telegram source",
            identity.id
        )))
    }
}

pub(crate) fn identity_updated_at() -> i64 {
    now_secs()
}
```

- [x] **Step 5: Export module**

In `src-tauri/src/sources/mod.rs`, add:

```rust
pub(crate) mod identity;
```

- [x] **Step 6: Add unit tests for identity helpers**

In `src-tauri/src/sources/identity.rs`, add:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_kind_matches_telegram_subtype() {
        assert_eq!(
            TelegramPeerKind::from_source_subtype(TelegramSourceKind::Channel),
            TelegramPeerKind::Channel
        );
        assert_eq!(
            TelegramPeerKind::from_source_subtype(TelegramSourceKind::Supergroup),
            TelegramPeerKind::Channel
        );
        assert_eq!(
            TelegramPeerKind::from_source_subtype(TelegramSourceKind::Group),
            TelegramPeerKind::Chat
        );
    }

    #[test]
    fn canonical_external_id_rejects_malformed_values() {
        for value in ["+123", "-123", "00123", "123 ", "12a3", ""] {
            assert!(
                canonical_telegram_external_id(value).is_err(),
                "{value} should be rejected"
            );
        }
        assert_eq!(canonical_telegram_external_id("123").unwrap(), 123);
    }

    #[test]
    fn username_normalization_removes_url_and_at_syntax() {
        assert_eq!(
            normalize_telegram_username(Some("https://t.me/Example_User?x=1")).as_deref(),
            Some("example_user")
        );
        assert_eq!(
            normalize_telegram_username(Some("@MixedCase")).as_deref(),
            Some("mixedcase")
        );
        assert_eq!(normalize_telegram_username(Some("  ")), None);
    }
}
```

- [x] **Step 7: Run helper tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::types::tests::telegram_source_kind_parses_from_canonical_source_subtype sources::types::tests::telegram_source_kind_rejects_unsupported_source_subtype sources::identity::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 8: Commit typed helpers**

Run:

```powershell
git add src-tauri/src/sources/types.rs src-tauri/src/sources/identity.rs src-tauri/src/sources/mod.rs
git commit -m "feat: add typed source identity helpers"
```

---

### Task 3: Repair Engine With Dry-Run Mode

**Files:**
- Create: `src-tauri/src/sources/identity_repair.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/peer_resolution.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Expose legacy metadata fields to repair only**

In `src-tauri/src/sources/peer_resolution.rs`, make `SourceMetadata` fields
needed by repair visible inside `sources`:

```rust
#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(super) struct SourceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) peer_identity: Option<SourcePeerIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) avatar_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) added_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) access_hash: Option<i64>,
}
```

Also make `SourcePeerIdentity`, its fields, and
`SourcePeerResolutionStrategy` visible to the repair module:

```rust
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum SourcePeerResolutionStrategy {
    Username,
    Dialog,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(super) struct SourcePeerIdentity {
    pub(super) strategy: SourcePeerResolutionStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) access_hash: Option<i64>,
}
```

- [x] **Step 2: Write failing dry-run tests**

Create `src-tauri/src/sources/identity_repair.rs` with an empty test module
first, then add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;
    use crate::sources::test_support::memory_pool_with_sources;

    async fn insert_telegram_source(
        pool: &sqlx::SqlitePool,
        id: i64,
        subtype: Option<&str>,
        legacy_kind: Option<&str>,
        account_id: Option<i64>,
        external_id: &str,
        metadata_json: Option<&[u8]>,
    ) {
        let metadata_zstd = metadata_json
            .map(compress_json_bytes)
            .transpose()
            .expect("compress metadata");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, telegram_source_kind, account_id,
                external_id, title, metadata_zstd, is_active, is_member, created_at
            )
            VALUES (?, 'telegram', ?, ?, ?, ?, 'source', ?, 1, 1, 100)
            "#,
        )
        .bind(id)
        .bind(subtype)
        .bind(legacy_kind)
        .bind(account_id)
        .bind(external_id)
        .bind(metadata_zstd)
        .execute(pool)
        .await
        .expect("insert source");
    }

    #[tokio::test]
    async fn dry_run_reports_repair_without_writing_typed_rows() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77},"avatar_cache_key":"1_channel_12345.jpg"}"#),
        )
        .await;

        let report = repair_source_identity(&pool, SourceIdentityRepairMode::DryRun)
            .await
            .expect("dry run succeeds");

        assert_eq!(report.repaired_sources, vec![101]);
        assert!(report.fatal_errors.is_empty());

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
            .fetch_one(&pool)
            .await
            .expect("count typed rows");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn apply_repair_creates_typed_identity_and_keeps_source_id() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77},"avatar_cache_key":"1_channel_12345.jpg"}"#),
        )
        .await;

        let report = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("repair succeeds");

        assert_eq!(report.repaired_sources, vec![101]);

        let row: (i64, String, String, i64, String, Option<i64>, Option<String>) = sqlx::query_as(
            "SELECT source_id, source_subtype, peer_kind, peer_id, username, access_hash, avatar_cache_key FROM telegram_sources WHERE source_id = 101",
        )
        .fetch_one(&pool)
        .await
        .expect("typed row");

        assert_eq!(row.0, 101);
        assert_eq!(row.1, "channel");
        assert_eq!(row.2, "channel");
        assert_eq!(row.3, 12345);
        assert_eq!(row.4, "example");
        assert_eq!(row.5, Some(77));
        assert_eq!(row.6.as_deref(), Some("1_channel_12345.jpg"));
    }
}
```

- [x] **Step 3: Run failing repair tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id
```

Expected:

```text
unresolved import `super::*`
```

or:

```text
cannot find function `repair_source_identity`
```

- [x] **Step 4: Implement repair report and mode**

In `src-tauri/src/sources/identity_repair.rs`, add:

```rust
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::RwLock;

use crate::error::{AppError, AppErrorKind, AppResult};

use super::identity::{
    canonical_telegram_external_id, normalize_telegram_username, TelegramPeerKind,
    TelegramResolutionStrategy,
};
use super::peer_resolution::{decode_source_metadata, SourcePeerResolutionStrategy};
use super::types::{
    TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceIdentityRepairMode {
    DryRun,
    Apply,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub(crate) struct SourceIdentityRepairReport {
    pub repaired_sources: Vec<i64>,
    pub repair_notes: Vec<SourceIdentityRepairNotePreview>,
    pub fatal_errors: Vec<SourceIdentityRepairDiagnostic>,
    pub canonical_index_created: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct SourceIdentityRepairNotePreview {
    pub source_id: i64,
    pub issue_code: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct SourceIdentityRepairDiagnostic {
    pub code: String,
    pub source_ids: Vec<i64>,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceIdentityRepairStatus {
    Pending,
    Running,
    Ready,
    Failed { error: crate::error::AppError },
}

#[derive(Clone)]
pub(crate) struct SourceIdentityRepairState {
    status: Arc<RwLock<SourceIdentityRepairStatus>>,
}

impl SourceIdentityRepairState {
    pub(crate) fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(SourceIdentityRepairStatus::Pending)),
        }
    }

    pub(crate) async fn status(&self) -> SourceIdentityRepairStatus {
        self.status.read().await.clone()
    }

    async fn set_status(&self, status: SourceIdentityRepairStatus) {
        *self.status.write().await = status;
    }
}
```

- [x] **Step 5: Implement canonical derivation and repair loop**

Add row structs and `repair_source_identity`:

```rust
#[derive(sqlx::FromRow)]
struct TelegramSourceRepairRow {
    id: i64,
    source_subtype: Option<String>,
    telegram_source_kind: Option<String>,
    account_id: Option<i64>,
    external_id: String,
    metadata_zstd: Option<Vec<u8>>,
}

#[derive(Clone)]
struct TelegramRepairCandidate {
    source_id: i64,
    account_id: i64,
    source_subtype: TelegramSourceKind,
    source_subtype_text: String,
    peer_kind: TelegramPeerKind,
    peer_id: i64,
    resolution_strategy: TelegramResolutionStrategy,
    username: Option<String>,
    access_hash: Option<i64>,
    avatar_cache_key: Option<String>,
}

pub(crate) async fn repair_source_identity(
    pool: &sqlx::SqlitePool,
    mode: SourceIdentityRepairMode,
) -> AppResult<SourceIdentityRepairReport> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let rows: Vec<TelegramSourceRepairRow> = sqlx::query_as(
        r#"
        SELECT id, source_subtype, telegram_source_kind, account_id, external_id, metadata_zstd
        FROM sources
        WHERE source_type = 'telegram'
        ORDER BY id
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::database)?;

    let mut report = SourceIdentityRepairReport::default();
    let mut candidates = Vec::new();

    for row in rows {
        match candidate_from_row(&row) {
            Ok(candidate) => candidates.push(candidate),
            Err(diagnostic) => report.fatal_errors.push(diagnostic),
        }
    }

    report
        .fatal_errors
        .extend(duplicate_canonical_identity_errors(&candidates));
    report
        .fatal_errors
        .extend(duplicate_typed_peer_identity_errors(&candidates));

    if !report.fatal_errors.is_empty() {
        tx.rollback().await.map_err(AppError::database)?;
        if mode == SourceIdentityRepairMode::DryRun {
            return Ok(report);
        }
        return Err(repair_failed_error(&report));
    }

    for candidate in candidates {
        report.repaired_sources.push(candidate.source_id);
        if mode == SourceIdentityRepairMode::Apply {
            upsert_telegram_source_identity(&mut tx, &candidate).await?;
            sqlx::query(
                "UPDATE sources SET source_subtype = ?, telegram_source_kind = ? WHERE id = ?",
            )
            .bind(&candidate.source_subtype_text)
            .bind(&candidate.source_subtype_text)
            .bind(candidate.source_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        }
    }

    if mode == SourceIdentityRepairMode::Apply {
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
                ON sources(account_id, source_type, source_subtype, external_id)
                WHERE source_type = 'telegram'
            "#,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
        report.canonical_index_created = true;
        tx.commit().await.map_err(AppError::database)?;
    } else {
        tx.rollback().await.map_err(AppError::database)?;
    }

    Ok(report)
}
```

- [x] **Step 6: Implement candidate derivation**

Add helper functions:

```rust
fn candidate_from_row(
    row: &TelegramSourceRepairRow,
) -> Result<TelegramRepairCandidate, SourceIdentityRepairDiagnostic> {
    let account_id = row.account_id.ok_or_else(|| SourceIdentityRepairDiagnostic {
        code: "telegram_source_missing_account".to_string(),
        source_ids: vec![row.id],
        detail: format!("Telegram source {} has no account_id", row.id),
    })?;

    let source_subtype = derive_source_subtype(row)?;
    let source_subtype_text = source_subtype.as_str().to_string();
    let peer_id = canonical_telegram_external_id(&row.external_id).map_err(|_| {
        SourceIdentityRepairDiagnostic {
            code: "malformed_telegram_external_id".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has malformed external_id", row.id),
        }
    })?;
    let metadata = decode_source_metadata(row.metadata_zstd.as_deref()).map_err(|_| {
        SourceIdentityRepairDiagnostic {
            code: "malformed_telegram_metadata".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has malformed legacy metadata", row.id),
        }
    })?;
    let identity = metadata.peer_identity.as_ref();
    let strategy = match identity.map(|identity| identity.strategy) {
        Some(SourcePeerResolutionStrategy::Username) => TelegramResolutionStrategy::Username,
        Some(SourcePeerResolutionStrategy::Dialog) => TelegramResolutionStrategy::Dialog,
        None => TelegramResolutionStrategy::Unknown,
    };

    Ok(TelegramRepairCandidate {
        source_id: row.id,
        account_id,
        peer_kind: TelegramPeerKind::from_source_subtype(source_subtype),
        source_subtype,
        source_subtype_text,
        peer_id,
        resolution_strategy: strategy,
        username: normalize_telegram_username(
            identity.and_then(|identity| identity.username.as_deref()),
        ),
        access_hash: identity.and_then(|identity| identity.access_hash),
        avatar_cache_key: metadata.avatar_cache_key,
    })
}

fn derive_source_subtype(
    row: &TelegramSourceRepairRow,
) -> Result<TelegramSourceKind, SourceIdentityRepairDiagnostic> {
    let canonical = row
        .source_subtype
        .as_deref()
        .and_then(|value| TelegramSourceKind::from_source_subtype(value).ok());
    let legacy = row
        .telegram_source_kind
        .as_deref()
        .and_then(|value| TelegramSourceKind::from_source_subtype(value).ok());

    match (canonical, legacy) {
        (Some(canonical), Some(legacy)) if canonical != legacy => {
            Err(SourceIdentityRepairDiagnostic {
                code: "telegram_subtype_legacy_kind_conflict".to_string(),
                source_ids: vec![row.id],
                detail: format!(
                    "Telegram source {} has conflicting source_subtype and legacy mirror",
                    row.id
                ),
            })
        }
        (Some(canonical), _) => Ok(canonical),
        (None, Some(legacy)) => Ok(legacy),
        (None, None) => Err(SourceIdentityRepairDiagnostic {
            code: "unsupported_telegram_source_subtype".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has no supported subtype", row.id),
        }),
    }
}
```

- [x] **Step 7: Implement duplicate preflight and upsert**

Add:

```rust
fn duplicate_canonical_identity_errors(
    candidates: &[TelegramRepairCandidate],
) -> Vec<SourceIdentityRepairDiagnostic> {
    let mut groups: BTreeMap<(i64, String, String), Vec<i64>> = BTreeMap::new();
    for candidate in candidates {
        groups
            .entry((
                candidate.account_id,
                candidate.source_subtype_text.clone(),
                candidate.peer_id.to_string(),
            ))
            .or_default()
            .push(candidate.source_id);
    }

    groups
        .into_iter()
        .filter_map(|((account_id, subtype, external_id), source_ids)| {
            if source_ids.len() < 2 {
                return None;
            }
            Some(SourceIdentityRepairDiagnostic {
                code: "duplicate_canonical_telegram_identity".to_string(),
                detail: format!(
                    "Duplicate Telegram identity account_id={account_id}, source_subtype={subtype}, external_id={external_id}"
                ),
                source_ids,
            })
        })
        .collect()
}

fn duplicate_typed_peer_identity_errors(
    candidates: &[TelegramRepairCandidate],
) -> Vec<SourceIdentityRepairDiagnostic> {
    let mut groups: BTreeMap<(i64, String, i64), Vec<i64>> = BTreeMap::new();
    for candidate in candidates {
        groups
            .entry((
                candidate.account_id,
                candidate.peer_kind.as_str().to_string(),
                candidate.peer_id,
            ))
            .or_default()
            .push(candidate.source_id);
    }

    groups
        .into_iter()
        .filter_map(|((account_id, peer_kind, peer_id), source_ids)| {
            if source_ids.len() < 2 {
                return None;
            }
            Some(SourceIdentityRepairDiagnostic {
                code: "duplicate_typed_telegram_peer_identity".to_string(),
                detail: format!(
                    "Duplicate Telegram peer identity account_id={account_id}, peer_kind={peer_kind}, peer_id={peer_id}"
                ),
                source_ids,
            })
        })
        .collect()
}

async fn upsert_telegram_source_identity(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    candidate: &TelegramRepairCandidate,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash, avatar_cache_key, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'))
        ON CONFLICT(source_id) DO UPDATE SET
            account_id = excluded.account_id,
            source_subtype = excluded.source_subtype,
            peer_kind = excluded.peer_kind,
            peer_id = excluded.peer_id,
            resolution_strategy = excluded.resolution_strategy,
            username = excluded.username,
            access_hash = excluded.access_hash,
            avatar_cache_key = excluded.avatar_cache_key,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(candidate.source_id)
    .bind(candidate.account_id)
    .bind(&candidate.source_subtype_text)
    .bind(candidate.peer_kind.as_str())
    .bind(candidate.peer_id)
    .bind(candidate.resolution_strategy.as_str())
    .bind(candidate.username.as_deref())
    .bind(candidate.access_hash)
    .bind(candidate.avatar_cache_key.as_deref())
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn repair_failed_error(report: &SourceIdentityRepairReport) -> AppError {
    let details = report
        .fatal_errors
        .iter()
        .map(|diagnostic| {
            format!(
                "{}: sources {:?}: {}",
                diagnostic.code, diagnostic.source_ids, diagnostic.detail
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    AppError::new(AppErrorKind::Validation, format!("Source identity repair failed: {details}"))
}
```

- [x] **Step 8: Export repair module**

In `src-tauri/src/sources/mod.rs`, add:

```rust
pub(crate) mod identity_repair;
```

- [x] **Step 9: Run repair tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 10: Commit repair engine foundation**

Run:

```powershell
git add src-tauri/src/sources/identity_repair.rs src-tauri/src/sources/mod.rs src-tauri/src/sources/peer_resolution.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: add source identity repair engine"
```

---

### Task 4: Fatal Repair Fixtures And Regression Guards

**Files:**
- Modify: `src-tauri/src/sources/identity_repair.rs`

- [x] **Step 1: Add failing malformed external id test**

Add:

```rust
#[tokio::test]
async fn malformed_external_ids_fail_without_writing_typed_rows() {
    for external_id in ["+123", "-123", "00123", "123 ", "12a3"] {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            external_id,
            None,
        )
        .await;

        let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("malformed id fails repair");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("malformed_telegram_external_id"));

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
            .fetch_one(&pool)
            .await
            .expect("count typed rows");
        assert_eq!(count, 0);
    }
}
```

- [x] **Step 2: Add failing missing account test**

Add:

```rust
#[tokio::test]
async fn missing_account_id_is_fatal() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(
        &pool,
        101,
        Some("channel"),
        Some("channel"),
        None,
        "12345",
        None,
    )
    .await;

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("missing account fails repair");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("telegram_source_missing_account"));
}
```

- [x] **Step 3: Add failing subtype conflict test**

Add:

```rust
#[tokio::test]
async fn source_subtype_and_legacy_kind_conflict_is_fatal() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(
        &pool,
        101,
        Some("channel"),
        Some("supergroup"),
        Some(1),
        "12345",
        None,
    )
    .await;

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("conflict fails repair");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("telegram_subtype_legacy_kind_conflict"));
}
```

- [x] **Step 4: Add failing duplicate canonical and typed peer tests**

Add:

```rust
#[tokio::test]
async fn duplicate_canonical_identity_reports_conflicting_source_ids() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(&pool, 101, Some("channel"), Some("channel"), Some(1), "12345", None)
        .await;
    insert_telegram_source(&pool, 102, Some("channel"), Some("channel"), Some(1), "12345", None)
        .await;

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("duplicate canonical identity fails repair");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("duplicate_canonical_telegram_identity"));
    assert!(error.message.contains("101"));
    assert!(error.message.contains("102"));
}

#[tokio::test]
async fn duplicate_typed_peer_identity_reports_conflicting_source_ids() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(&pool, 101, Some("channel"), Some("channel"), Some(1), "12345", None)
        .await;
    insert_telegram_source(
        &pool,
        102,
        Some("supergroup"),
        Some("supergroup"),
        Some(1),
        "12345",
        None,
    )
    .await;

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("duplicate peer identity fails repair");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("duplicate_typed_telegram_peer_identity"));
    assert!(error.message.contains("101"));
    assert!(error.message.contains("102"));
}
```

- [x] **Step 5: Add failing idempotency and rollback tests**

Add:

```rust
#[tokio::test]
async fn apply_repair_is_idempotent() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(&pool, 101, Some("group"), Some("group"), Some(1), "12345", None)
        .await;

    repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect("first repair");
    repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect("second repair");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
        .fetch_one(&pool)
        .await
        .expect("count typed rows");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn fatal_repair_rolls_back_and_does_not_create_canonical_index() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(&pool, 101, Some("channel"), Some("channel"), Some(1), "12345", None)
        .await;
    insert_telegram_source(&pool, 102, Some("channel"), Some("channel"), None, "67890", None)
        .await;

    repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("repair fails");

    let typed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
        .fetch_one(&pool)
        .await
        .expect("count typed rows");
    assert_eq!(typed_count, 0);

    let index_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_sources_unique_telegram_identity'",
    )
    .fetch_one(&pool)
    .await
    .expect("count canonical index");
    assert_eq!(index_count, 0);
}
```

- [x] **Step 6: Run fatal fixture tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 7: Commit repair regression guards**

Run:

```powershell
git add src-tauri/src/sources/identity_repair.rs
git commit -m "test: cover source identity repair failures"
```

---

### Task 5: Startup Repair State, Dry-Run Command, And Source Command Gate

**Files:**
- Modify: `src-tauri/src/sources/identity_repair.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: source command handlers in `src-tauri/src/sources/*`,
  `src-tauri/src/takeout_import/mod.rs`, and persisted-source YouTube command
  handlers.

- [x] **Step 1: Add startup state helpers**

In `src-tauri/src/sources/identity_repair.rs`, add:

```rust
pub(crate) async fn run_startup_source_identity_repair(handle: tauri::AppHandle) {
    let state = handle.state::<SourceIdentityRepairState>().inner().clone();
    state
        .set_status(SourceIdentityRepairStatus::Running)
        .await;

    let result = async {
        let pool = crate::db::get_pool(&handle).await?;
        repair_source_identity(&pool, SourceIdentityRepairMode::Apply).await
    }
    .await;

    match result {
        Ok(_) => state.set_status(SourceIdentityRepairStatus::Ready).await,
        Err(error) => {
            state
                .set_status(SourceIdentityRepairStatus::Failed { error })
                .await
        }
    }
}

pub(crate) async fn require_source_identity_ready(
    state: &SourceIdentityRepairState,
) -> AppResult<()> {
    match state.status().await {
        SourceIdentityRepairStatus::Ready => Ok(()),
        SourceIdentityRepairStatus::Failed { error } => Err(error),
        SourceIdentityRepairStatus::Pending | SourceIdentityRepairStatus::Running => {
            Err(AppError::conflict("Source identity repair is still running"))
        }
    }
}

#[tauri::command]
pub(crate) async fn get_source_identity_repair_status(
    state: tauri::State<'_, SourceIdentityRepairState>,
) -> SourceIdentityRepairStatus {
    state.status().await
}

#[tauri::command]
pub(crate) async fn preview_source_identity_repair(
    handle: tauri::AppHandle,
) -> AppResult<SourceIdentityRepairReport> {
    let pool = crate::db::get_pool(&handle).await?;
    repair_source_identity(&pool, SourceIdentityRepairMode::DryRun).await
}
```

- [x] **Step 2: Register repair state and commands**

In `src-tauri/src/lib.rs`, import:

```rust
use sources::identity_repair::{
    get_source_identity_repair_status, preview_source_identity_repair,
    run_startup_source_identity_repair, SourceIdentityRepairState,
};
```

Add managed state:

```rust
.manage(SourceIdentityRepairState::new())
```

In setup, start repair after SQL plugin initialization:

```rust
let repair_handle = app.handle().clone();
tauri::async_runtime::spawn(async move {
    run_startup_source_identity_repair(repair_handle).await;
});
```

Register commands:

```rust
get_source_identity_repair_status,
preview_source_identity_repair,
```

- [x] **Step 3: Gate source commands**

At the beginning of every command that reads/writes persisted sources or
source-derived rows, add a `SourceIdentityRepairState` parameter and call:

```rust
require_source_identity_ready(repair_state.inner()).await?;
```

Apply to:

- `delete_source`
- `add_telegram_source`
- `list_sources`
- `sync_source`
- `list_source_items`
- `list_source_forum_topics`
- `start_takeout_source_import`
- `run_takeout_export_dc_spike`
- `add_youtube_source`
- `sync_youtube_source`
- `sync_youtube_playlist_video`
- `retry_failed_youtube_playlist_videos`
- `list_youtube_source_summaries`
- `get_youtube_video_detail`
- `get_youtube_playlist_detail`
- `list_youtube_transcript_segments`
- `export_source_to_notebooklm`
- `list_analysis_sources`

Do not gate `preview_youtube_source` because it does not use persisted source
identity.

Also gate spawned job bodies that call `load_source()` after command return:
YouTube source jobs, playlist-video jobs, Takeout jobs, and NotebookLM export
jobs must check repair state before loading persisted source identity.

- [x] **Step 4: Add gate unit tests**

In `src-tauri/src/sources/identity_repair.rs`, add:

```rust
#[tokio::test]
async fn source_identity_gate_blocks_while_running() {
    let state = SourceIdentityRepairState::new();
    state
        .set_status(SourceIdentityRepairStatus::Running)
        .await;

    let error = require_source_identity_ready(&state)
        .await
        .expect_err("running gate blocks");
    assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
}

#[tokio::test]
async fn source_identity_gate_returns_startup_failure() {
    let state = SourceIdentityRepairState::new();
    state
        .set_status(SourceIdentityRepairStatus::Failed {
            error: AppError::validation("Source identity repair failed: example"),
        })
        .await;

    let error = require_source_identity_ready(&state)
        .await
        .expect_err("failed gate blocks");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("Source identity repair failed"));
}
```

- [x] **Step 5: Run repair gate tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair::tests::source_identity_gate_blocks_while_running sources::identity_repair::tests::source_identity_gate_returns_startup_failure
```

Expected:

```text
test result: ok. 2 passed; 0 failed
```

- [x] **Step 6: Run Rust compile check via tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 7: Commit startup gate**

Run:

```powershell
git add src-tauri/src/lib.rs src-tauri/src/sources src-tauri/src/takeout_import src-tauri/src/youtube src-tauri/src/notebooklm_export src-tauri/src/analysis
git commit -m "feat: gate source commands on identity repair"
```

---

### Task 6: Source Store And DTO Switch

**Files:**
- Modify: `src-tauri/src/sources/identity.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Add typed identity loaders and writer tests**

In `src-tauri/src/sources/identity.rs`, add tests using
`memory_pool_with_sources()`:

```rust
#[tokio::test]
async fn load_telegram_identity_returns_typed_row() {
    let pool = crate::sources::test_support::memory_pool_with_sources().await;
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, telegram_source_kind, account_id,
            external_id, title, is_active, is_member, created_at
        )
        VALUES (101, 'telegram', 'channel', 'channel', 1, '12345', 'source', 1, 1, 100)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert source");
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash
        )
        VALUES (101, 1, 'channel', 'channel', 12345, 'username', 'example', 77)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert typed row");

    let identity = load_telegram_source_identity(&pool, 101)
        .await
        .expect("load typed identity");

    assert_eq!(identity.source_id, 101);
    assert_eq!(identity.source_subtype, TelegramSourceKind::Channel);
    assert_eq!(identity.peer_kind, TelegramPeerKind::Channel);
    assert_eq!(identity.username.as_deref(), Some("example"));
}
```

- [x] **Step 2: Implement typed identity loaders**

Add to `src-tauri/src/sources/identity.rs`:

```rust
#[derive(sqlx::FromRow)]
struct TelegramSourceIdentityRow {
    source_id: i64,
    account_id: i64,
    source_subtype: String,
    peer_kind: String,
    peer_id: i64,
    resolution_strategy: String,
    username: Option<String>,
    access_hash: Option<i64>,
    avatar_cache_key: Option<String>,
}

pub(crate) async fn load_telegram_source_identity(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<TelegramSourceIdentity> {
    let row: TelegramSourceIdentityRow = sqlx::query_as(
        r#"
        SELECT source_id, account_id, source_subtype, peer_kind, peer_id,
               resolution_strategy, username, access_hash, avatar_cache_key
        FROM telegram_sources
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| {
        AppError::internal(format!(
            "Source {source_id} is missing Telegram typed identity after startup repair"
        ))
    })?;

    Ok(TelegramSourceIdentity {
        source_id: row.source_id,
        account_id: row.account_id,
        source_subtype: TelegramSourceKind::from_source_subtype(&row.source_subtype)?,
        peer_kind: TelegramPeerKind::parse(&row.peer_kind)?,
        peer_id: row.peer_id,
        resolution_strategy: TelegramResolutionStrategy::parse(&row.resolution_strategy)?,
        username: row.username,
        access_hash: row.access_hash,
        avatar_cache_key: row.avatar_cache_key,
    })
}
```

- [x] **Step 3: Add targeted Telegram runtime loader**

Do not remove `metadata_zstd` from `SourceSyncTarget` in this task. YouTube
jobs still use `SourceSyncTarget.metadata_zstd` for YouTube metadata, so a
targeted Telegram runtime loader reduces database-schema regression risk.

Add:

```rust
pub(crate) struct TelegramRuntimeSource {
    pub(crate) source: SourceSyncTarget,
    pub(crate) identity: TelegramSourceIdentity,
}

pub(crate) async fn load_telegram_runtime_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<TelegramRuntimeSource> {
    let source = crate::sources::store::load_source(pool, source_id).await?;
    if source.source_type != TELEGRAM_SOURCE_TYPE {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a Telegram source"
        )));
    }
    let identity = load_telegram_source_identity(pool, source_id).await?;
    Ok(TelegramRuntimeSource { source, identity })
}
```

- [x] **Step 4: Make DTO subtype non-optional on backend**

In `src-tauri/src/sources/types.rs`, change:

```rust
pub source_subtype: Option<String>,
```

to:

```rust
pub source_subtype: String,
```

for `SourceRecord`.

Keep `SourceRecordRow.source_subtype: Option<String>` while old DB rows still
exist before repair, then force canonical mapping in `store.rs`.

- [x] **Step 5: Map backend DTO compatibility mirror from canonical subtype**

In `src-tauri/src/sources/store.rs`, update `source_record_from_row_parts`:

```rust
let source_subtype = row.source_subtype.unwrap_or_else(|| "unknown".to_string());
let telegram_source_kind = if row.source_type == TELEGRAM_SOURCE_TYPE {
    Some(source_subtype.clone())
} else {
    None
};

SourceRecord {
    id: row.id,
    source_type: row.source_type,
    source_subtype,
    telegram_source_kind,
    account_id: row.account_id,
    external_id: row.external_id,
    title: row.title,
    last_sync_state: row.last_sync_state,
    last_synced_at: row.last_synced_at,
    is_member: row.is_member,
    is_active: row.is_active,
    created_at: row.created_at,
    telegram_username,
    avatar_data_url,
}
```

The `"unknown"` fallback should only be reachable in tests or repair-blocked
startup windows; after Task 5, normal source commands are gated until repair
has populated implemented-provider subtype.

- [x] **Step 6: Change Telegram add/upsert to canonical conflict**

In `add_telegram_source`, wrap `sources` and `telegram_sources` writes in one
transaction:

```rust
let mut tx = pool.begin().await.map_err(AppError::database)?;
let row: SourceRecordRow = sqlx::query_as(
    r#"
    INSERT INTO sources (
        source_type, source_subtype, telegram_source_kind, external_id, title,
        metadata_zstd, is_active, is_member, account_id, created_at
    )
    VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)
    ON CONFLICT(account_id, source_type, source_subtype, external_id)
    WHERE source_type = 'telegram'
    DO UPDATE SET
        title = excluded.title,
        telegram_source_kind = excluded.telegram_source_kind,
        metadata_zstd = excluded.metadata_zstd,
        is_member = excluded.is_member,
        account_id = excluded.account_id,
        is_active = 1
    RETURNING id, source_type, source_subtype, telegram_source_kind, account_id,
              external_id, title, metadata_zstd, last_sync_state, last_synced_at,
              is_active, is_member, created_at
    "#,
)
.bind(SourceType::Telegram.as_str())
.bind(&resolved.telegram_source_kind)
.bind(&resolved.telegram_source_kind)
.bind(&resolved.external_id)
.bind(&resolved.title)
.bind(metadata_zstd)
.bind(resolved.is_member)
.bind(request.account_id)
.bind(now)
.fetch_one(&mut *tx)
.await
.map_err(AppError::database)?;

upsert_telegram_source_identity_from_resolved(
    &mut tx,
    row.id,
    request.account_id,
    &resolved,
    avatar_cache_key.as_deref(),
)
.await?;

tx.commit().await.map_err(AppError::database)?;
```

Then upsert the matching typed row in the same transaction using canonical
subtype and normalized username.

- [x] **Step 7: Read list display fields from typed table**

Update `list_sources` / `load_source_record` SELECT statements to left join
`telegram_sources`:

```sql
SELECT s.id, s.source_type, s.source_subtype, s.telegram_source_kind,
       s.account_id, s.external_id, s.title, s.metadata_zstd,
       s.last_sync_state, s.last_synced_at, s.is_active, s.is_member, s.created_at,
       ts.username AS telegram_username,
       ts.avatar_cache_key AS telegram_avatar_cache_key
FROM sources s
LEFT JOIN telegram_sources ts ON ts.source_id = s.id
WHERE s.id = ?
```

For `list_sources`, use the same selected columns with either:

```sql
WHERE s.account_id = ?
ORDER BY s.created_at DESC
```

or:

```sql
ORDER BY s.created_at DESC
```

Extend `SourceRecordRow` with:

```rust
pub(super) telegram_username: Option<String>,
pub(super) telegram_avatar_cache_key: Option<String>,
```

Use those fields instead of decoding metadata for normal DTO display.

- [x] **Step 8: Update store tests**

Update or add tests in `src-tauri/src/sources/store.rs`:

```rust
#[test]
fn source_record_parts_mirrors_telegram_kind_from_source_subtype() {
    let record = source_record_from_row_parts(
        SourceRecordRow {
            id: 1,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some("supergroup".to_string()),
            telegram_source_kind: Some("channel".to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("source".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
            last_synced_at: None,
            is_active: true,
            is_member: true,
            created_at: 100,
            telegram_username: Some("example".to_string()),
            telegram_avatar_cache_key: None,
        },
        Some("example".to_string()),
        None,
    );

    assert_eq!(record.source_subtype, "supergroup");
    assert_eq!(record.telegram_source_kind.as_deref(), Some("supergroup"));
}
```

- [x] **Step 9: Run source store tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests sources::identity::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 10: Commit source store switch**

Run:

```powershell
git add src-tauri/src/sources/identity.rs src-tauri/src/sources/types.rs src-tauri/src/sources/store.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: switch source store to canonical identity"
```

---

### Task 7: Runtime Telegram Peer Resolution From Typed Identity

**Files:**
- Modify: `src-tauri/src/sources/peer_resolution.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/identity.rs`

- [x] **Step 1: Add typed peer resolution tests**

In `src-tauri/src/sources/peer_resolution.rs`, add tests that exercise pure
planning helpers without a live Telegram client:

```rust
#[test]
fn typed_identity_builds_channel_peer_ref_when_access_hash_exists() {
    let identity = TelegramSourceIdentity {
        source_id: 101,
        account_id: 1,
        source_subtype: TelegramSourceKind::Channel,
        peer_kind: TelegramPeerKind::Channel,
        peer_id: 12345,
        resolution_strategy: TelegramResolutionStrategy::Username,
        username: Some("example".to_string()),
        access_hash: Some(77),
        avatar_cache_key: None,
    };

    assert!(identity.peer_ref().expect("peer ref check").is_some());
}

#[test]
fn typed_identity_rejects_subtype_peer_kind_mismatch() {
    let identity = TelegramSourceIdentity {
        source_id: 101,
        account_id: 1,
        source_subtype: TelegramSourceKind::Group,
        peer_kind: TelegramPeerKind::Channel,
        peer_id: 12345,
        resolution_strategy: TelegramResolutionStrategy::Dialog,
        username: None,
        access_hash: Some(77),
        avatar_cache_key: None,
    };

    let error = identity.peer_ref().expect_err("mismatch is invalid");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
}
```

- [x] **Step 2: Change `ResolvedSyncPeer` to update typed identity**

In `peer_resolution.rs`, change:

```rust
pub(crate) struct ResolvedSyncPeer {
    pub(crate) peer: PeerRef,
    pub(crate) refreshed_metadata_zstd: Option<Vec<u8>>,
}
```

to:

```rust
pub(crate) struct ResolvedSyncPeer {
    pub(crate) peer: PeerRef,
    pub(crate) refreshed_avatar_cache_key: Option<String>,
}
```

- [x] **Step 3: Load typed identity in runtime resolution**

Change `resolve_and_refresh_peer` to accept the pool or a preloaded
`TelegramSourceIdentity`:

```rust
pub(crate) async fn resolve_and_refresh_peer(
    handle: &AppHandle,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    account_id: i64,
) -> AppResult<ResolvedSyncPeer> {
    let identity = load_telegram_source_identity(pool, source.id).await?;
    let peer = resolve_source_peer_from_typed_identity(client, source.id, &identity).await?;
    let refreshed_avatar_cache_key =
        refresh_source_avatar_cache(handle, client, source, account_id, peer).await;
    Ok(ResolvedSyncPeer {
        peer,
        refreshed_avatar_cache_key,
    })
}
```

`resolve_source_peer_from_typed_identity` may use direct `peer_ref()`, username,
and dialog scan. It must not call `decode_source_metadata`.

- [x] **Step 4: Keep legacy metadata decoder only for repair/add-source metadata**

Leave `decode_source_metadata` available for `identity_repair.rs`, but remove
its use from normal `resolve_source_peer`.

Run:

```powershell
rg -n "decode_source_metadata\\(" src-tauri\\src\\sources
```

Expected matches after this task:

```text
src-tauri\src\sources\identity_repair.rs
src-tauri\src\sources\store.rs
src-tauri\src\sources\peer_resolution.rs
```

The remaining `peer_resolution.rs` matches must be metadata encode/decode tests
or legacy helpers used by repair/add-source metadata only, not normal runtime
peer resolution.

- [x] **Step 5: Update sync finalize to write typed avatar cache key**

In `src-tauri/src/sources/sync.rs`, replace `metadata_zstd` refresh with an
update to `telegram_sources`:

```rust
if let Some(cache_key) = resolved_peer.refreshed_avatar_cache_key {
    sqlx::query(
        "UPDATE telegram_sources SET avatar_cache_key = ?, updated_at = strftime('%s','now'), identity_refreshed_at = strftime('%s','now') WHERE source_id = ?",
    )
    .bind(cache_key)
    .bind(source.id)
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;
}
```

- [x] **Step 6: Run runtime peer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::peer_resolution::tests sources::sync::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 7: Commit runtime peer switch**

Run:

```powershell
git add src-tauri/src/sources/peer_resolution.rs src-tauri/src/sources/sync.rs src-tauri/src/sources/identity.rs
git commit -m "feat: resolve telegram peers from typed identity"
```

---

### Task 8: Takeout, Topics, And Provider Consumers

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/pagination.rs`
- Modify: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/youtube/detail.rs`
- Modify: `src-tauri/src/youtube/preview.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Replace legacy kind reads in Takeout with typed subtype**

In `src-tauri/src/takeout_import/mod.rs`, after loading source, load typed
identity for Telegram source operations:

```rust
let identity = crate::sources::identity::load_telegram_source_identity(&pool, source.id).await?;
let telegram_source_subtype = identity.source_subtype.as_str();
```

Pass `telegram_source_subtype` into existing helpers. Rename local variables
only where it reduces confusion; keep enum values unchanged.

- [x] **Step 2: Update topic refresh supergroup check**

In `src-tauri/src/sources/topics.rs`, replace:

```rust
if source.telegram_source_kind != TELEGRAM_KIND_SUPERGROUP {
```

with:

```rust
let identity = crate::sources::identity::load_telegram_source_identity(pool, source.id).await?;
if identity.source_subtype != TelegramSourceKind::Supergroup {
```

- [x] **Step 3: Keep YouTube behavior canonical**

Ensure YouTube code uses `source.source_subtype.as_deref()` or the new
non-optional string form, never `telegram_source_kind`, for routing. Existing
YouTube insert helpers may still write `telegram_source_kind = ''` only as a
compatibility insert field.

- [x] **Step 4: Update NotebookLM export source loading**

In `src-tauri/src/notebooklm_export/query.rs`, remove direct reliance on
`telegram_source_kind` for source identity. Export source labels and provider
routing should use canonical `source_type` and `source_subtype`.

- [x] **Step 5: Add or update targeted consumer tests**

Run existing targeted tests first:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import sources::topics::tests youtube::jobs::tests youtube::detail::tests youtube::preview::tests notebooklm_export
```

Expected before code settles: compile failures around changed types.

Update tests to build `telegram_sources` rows in fixtures when they exercise
Telegram persisted source behavior.

- [x] **Step 6: Run targeted consumer tests after fixes**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import sources::topics::tests youtube::jobs::tests youtube::detail::tests youtube::preview::tests notebooklm_export
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 7: Commit consumer switch**

Run:

```powershell
git add src-tauri/src/takeout_import src-tauri/src/sources/topics.rs src-tauri/src/youtube src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: use typed identity in source consumers"
```

---

### Task 9: Frontend DTO Compatibility Variant A

**Files:**
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/source-capabilities.ts`
- Modify: `src/lib/components/analysis/source-management-dialog.svelte`
- Modify: `src/lib/api/sources.test.ts`
- Modify: `src/lib/source-capabilities.test.ts`
- Modify affected source/analysis tests that construct `Source`.

- [x] **Step 1: Add failing mapSource no-fallback test**

In `src/lib/api/sources.test.ts`, add:

```ts
it("does not derive persisted sourceSubtype from deprecated telegram_source_kind", async () => {
  mockInvoke.mockResolvedValueOnce([
    {
      id: 1,
      source_type: "telegram",
      source_subtype: null,
      telegram_source_kind: "channel",
      account_id: 7,
      external_id: "12345",
      title: "Legacy row",
      last_sync_state: null,
      last_synced_at: null,
      is_member: true,
      is_active: true,
      created_at: 100,
      telegram_username: null,
      avatar_data_url: null,
    },
  ]);

  await expect(listSources(7)).resolves.toMatchObject([
    {
      sourceSubtype: null,
      telegramSourceKind: "channel",
    },
  ]);
});
```

- [x] **Step 2: Run failing frontend DTO test**

Run:

```powershell
npm test -- src/lib/api/sources.test.ts
```

Expected:

```text
FAIL src/lib/api/sources.test.ts
expected sourceSubtype to be null
```

- [x] **Step 3: Remove persisted fallback**

In `src/lib/api/sources.ts`, change:

```ts
sourceSubtype: source.source_subtype ?? source.telegram_source_kind ?? null,
```

to:

```ts
sourceSubtype: source.source_subtype ?? null,
```

- [x] **Step 4: Derive Telegram behavior from canonical subtype**

In `src/lib/source-capabilities.ts`, change:

```ts
function telegramKind(source: Pick<Source, "telegramSourceKind" | "sourceSubtype">) {
  return source.telegramSourceKind ?? telegramSubtype(source.sourceSubtype);
}
```

to:

```ts
function telegramKind(source: Pick<Source, "sourceSubtype">) {
  return telegramSubtype(source.sourceSubtype);
}
```

- [x] **Step 5: Update persisted source keying in source management dialog**

In `src/lib/components/analysis/source-management-dialog.svelte`, change
existing source keys:

```ts
.map((source) => `${source.telegramSourceKind}:${source.externalId}`)
```

to:

```ts
.map((source) => `${source.sourceSubtype}:${source.externalId}`)
```

Keep live dialog source keying as:

```ts
return `${source.telegramSourceKind}:${source.id}`;
```

- [x] **Step 6: Document deprecated TS field**

In `src/lib/types/sources.ts`, add a comment:

```ts
// Deprecated compatibility mirror for persisted Telegram sources.
// Current UI behavior must derive Telegram subtype from sourceSubtype.
telegramSourceKind: TelegramSourceKind | null;
```

- [x] **Step 7: Update frontend tests**

Run:

```powershell
rg -n "telegramSourceKind" src\lib src\routes
```

For each test fixture that sets both fields, keep `telegramSourceKind` only as
compatibility data and assert behavior through `sourceSubtype`.

- [x] **Step 8: Run targeted frontend tests**

Run:

```powershell
npm test -- src/lib/api/sources.test.ts src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts src/lib/analysis-state.test.ts
```

Expected:

```text
Test Files  4 passed
Tests       <count> passed
```

- [x] **Step 9: Commit frontend compatibility update**

Run:

```powershell
git add src/lib/api/sources.ts src/lib/types/sources.ts src/lib/source-capabilities.ts src/lib/components/analysis/source-management-dialog.svelte src/lib/*.test.ts
git commit -m "feat: prefer canonical source subtype in frontend"
```

---

### Task 10: Database Regression Matrix

**Files:**
- Modify: `src-tauri/src/sources/identity_repair.rs`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Add fresh schema regression test**

Add a test that applies all migrations to an in-memory database by iterating
`build_migrations()` and executing each migration with
`sqlx::raw_sql(migration.sql)`, then asserts:

```rust
for table in ["sources", "telegram_sources", "source_identity_repair_notes"] {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table)
    .fetch_one(&pool)
    .await
    .expect("check table");
    assert_eq!(exists, 1, "missing table {table}");
}
```

- [x] **Step 2: Add v17-style upgrade fixture test**

Create an old-schema fixture that has migrations through 17, insert a Telegram
source with id `101`, run migration 18 SQL, then run repair. Assert:

```rust
assert_eq!(source_id_after_repair, 101);
assert_eq!(typed_source_id, 101);
assert_eq!(canonical_index_count, 1);
```

- [x] **Step 3: Add YouTube unaffected regression test**

Insert a YouTube video and playlist through existing helpers before and after
repair. Assert both keep the same `id` on repeated upsert and no
`telegram_sources` row is created for YouTube source ids.

- [x] **Step 4: Add existing typed projection drift tests**

Seed `telegram_sources` before repair:

- non-conflicting drift: wrong `source_subtype` matching no other peer conflict;
  repair updates from `sources`;
- conflicting drift: wrong `peer_kind` for subtype; repair fails.

Assert conflict diagnostics include source id and do not expose raw metadata.

- [x] **Step 5: Run database regression tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests sources::identity_repair::tests sources::store::tests
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [x] **Step 6: Commit regression matrix**

Run:

```powershell
git add src-tauri/src/migrations.rs src-tauri/src/sources/identity_repair.rs src-tauri/src/sources/test_support.rs src-tauri/src/sources/store.rs
git commit -m "test: cover source identity schema regressions"
```

---

### Task 11: Compatibility Containment And Documentation

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/backlog.md`
- Modify code comments at compatibility boundaries only.

- [x] **Step 1: Add compatibility boundary comments**

Add short comments only where the legacy field is intentionally mirrored:

```rust
// Deprecated DTO/database compatibility mirror. Canonical Telegram subtype is source_subtype.
```

Use this near the `telegram_source_kind` DTO assignment and Telegram insert
mirror, not throughout the codebase.

- [x] **Step 2: Run legacy field scan**

Run:

```powershell
rg -n "telegram_source_kind|telegramSourceKind" src-tauri\src src\lib
```

Expected remaining categories:

- old migration SQL includes;
- repair/backfill code;
- DTO compatibility mirror;
- live dialog classification;
- tests explicitly covering compatibility;
- compatibility insert mirror while old DB column exists.

If normal runtime peer resolution, Takeout, topics, frontend capabilities, or
persisted-source mapping still read the legacy field as authoritative, fix
those call sites before continuing.

- [x] **Step 3: Update database schema docs**

In `docs/database-schema.md`, add sections for:

- `telegram_sources`;
- `source_identity_repair_notes`;
- canonical Telegram uniqueness via
  `(account_id, source_type, source_subtype, external_id)`;
- `telegram_source_kind` marked deprecated compatibility mirror.

- [x] **Step 4: Update architecture docs**

In `docs/architecture-deep-dive.md`, add a concise source identity boundary:

```markdown
Telegram operational identity lives in `telegram_sources`; generic provider
identity lives in `sources`. Runtime source flows use canonical
`source_subtype` and typed Telegram peer identity. Legacy metadata is decoded
only during startup repair.
```

- [x] **Step 5: Update backlog**

In `docs/backlog.md`, add follow-up entries:

- remove `telegram_source_kind` from DTO after compatibility window;
- rebuild fresh current schema without legacy column;
- move YouTube metadata to typed source tables;
- item/document identity cleanup.

- [x] **Step 6: Commit documentation**

Run:

```powershell
git add docs/database-schema.md docs/architecture-deep-dive.md docs/backlog.md src-tauri/src src/lib
git commit -m "docs: document source identity bridge"
```

---

### Task 12: Final Verification

**Files:**
- No new source edits unless verification finds failures.

- [ ] **Step 1: Run Rust full test suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
test result: ok. <count> passed; 0 failed
```

- [ ] **Step 2: Run frontend test suite**

Run:

```powershell
npm test
```

Expected:

```text
Test Files  <count> passed
Tests       <count> passed
```

- [ ] **Step 3: Run frontend type check**

Run:

```powershell
npm run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 4: Run SQL/diff hygiene checks**

Run:

```powershell
git diff --check
rg -n "INSERT OR IGNORE INTO telegram_sources|CAST\(external_id|GLOB" src-tauri\migrations src-tauri\src\sources
rg -n "source_subtype \?\? source\.telegram_source_kind|telegramSourceKind \?\?" src\lib src\routes
```

Expected:

```text
git diff --check: no output
rg forbidden SQL coercion: no matches
rg frontend fallback: no matches
```

- [ ] **Step 5: Run legacy-field containment scan**

Run:

```powershell
rg -n "telegram_source_kind|telegramSourceKind" src-tauri\src src\lib
```

Expected: every remaining match is in one of these categories:

- old migration registration/tests;
- repair/backfill;
- DTO compatibility mirror;
- live Telegram dialog classification;
- compatibility tests;
- temporary insert mirror.

- [ ] **Step 6: Inspect final git status**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## feature/source-identity-schema
```

with no unstaged changes after final commits, or only intentional uncommitted
changes if the user requested review before commit.

---

## Execution Notes

- Use TDD inside each task: write the narrow failing test first, run it, then
  implement the minimum code to pass.
- Keep the old `idx_sources_ext` through this slice unless a targeted test
  proves dropping it is necessary and safe.
- Do not use SQLite `CAST`, `GLOB`, or `INSERT OR IGNORE` to backfill typed
  Telegram identity.
- Do not merge, delete, or silently downgrade duplicate/malformed identity rows
  into repair notes.
- `source_identity_repair_notes` is only for non-fatal enrichment gaps.
- `preview_source_identity_repair` is diagnostic and must not write to the
  database.
- Source commands are blocked by repair state; the app itself still starts.
- `telegram_source_kind` remains as a deprecated DTO/database mirror until a
  later cleanup slice.
