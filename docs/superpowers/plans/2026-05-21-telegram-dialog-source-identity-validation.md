# Telegram Dialog Source Identity Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add backend-only validation prep for dialog-picked Telegram source identity without live Telegram access.

**Architecture:** Cover pure resolver order in `peer_resolution.rs` and SQLite storage invariants in `store.rs`, then document the future manual live-validation matrix. Keep changes minimal and let RED tests drive any production code changes.

**Tech Stack:** Rust, Tokio, SQLx SQLite tests, Cargo test filters, Markdown verification docs.

---

### Task 1: Resolver Order And Usable Stored Identity

**Files:**
- Modify: `src-tauri/src/sources/peer_resolution.rs`

- [x] **Step 1: Write resolver tests**

Add focused tests for dialog channel/supergroup stored-peer-first behavior, group dialog-dependence, username-without-hash fallback, and unusable typed identity not starting with `StoredPeerIdentity`.

- [x] **Step 2: Run resolver tests**

Run: `cargo test sources::peer_resolution::tests`

Expected: a failure if current behavior has a mismatch; otherwise the new tests become green characterization coverage.

- [x] **Step 3: Implement minimal resolver change if RED exposes a mismatch**

Only change resolver logic if a RED test proves current behavior is wrong. Expected target order:

```text
usable stored peer -> username when available -> dialog scan
```

- [x] **Step 4: Run resolver tests for GREEN**

Run: `cargo test sources::peer_resolution::tests`

Expected: all resolver tests pass.

### Task 2: Typed Identity Storage Invariants

**Files:**
- Modify: `src-tauri/src/sources/store.rs`

- [x] **Step 1: Write failing storage tests**

Add SQLite tests for dialog-picked channel, supergroup, and group typed identity writes; cross-account same peer allowance; and same-account peer conflict at `telegram_sources(account_id, peer_kind, peer_id)`.

Note: regular small groups are dialog-dependent; do not require `access_hash` for `group`.

Ensure the same-account conflict fixture avoids hitting generic `sources` uniqueness first; the conflict should prove the typed `telegram_sources(account_id, peer_kind, peer_id)` boundary.

- [x] **Step 2: Run storage tests for RED**

Run: `cargo test sources::store::tests`

Expected: at least one new storage test fails for missing coverage or behavior mismatch.

- [x] **Step 3: Implement minimal storage change if RED exposes a mismatch**

Only adjust upsert/storage behavior if tests show current writes or constraints are wrong.

- [x] **Step 4: Run storage tests for GREEN**

Run: `cargo test sources::store::tests`

Expected: all storage tests pass.

### Task 3: Manual Runtime Validation Matrix

**Files:**
- Create: `docs/superpowers/verification/telegram-runtime-private-source-validation.md`

- [x] **Step 1: Add future manual validation matrix**

Document cases for public/private channel and supergroup, regular small group, migrated group, lost access, and same source on two accounts. Mark the file as a plan for future live validation, not evidence that validation has already run.

- [x] **Step 2: Verify docs diff**

Run: `git diff --check`

Expected: no whitespace errors.

### Task 4: Slice Verification

**Files:**
- Verify all changed files.

- [x] **Step 1: Run targeted and sources verification**

Run:

```text
cargo test sources::peer_resolution::tests
cargo test sources::store::tests
cargo test sources::
cargo fmt --check
git diff --check
```

Expected: all commands exit 0, with only known line-ending warnings from `git diff --check` if they appear.
