# Extractum Session Context

> Last updated: 2026-05-08
> Purpose: restore the current Codex session context after compaction, restart, or handoff.

## Environment

- Repository: `G:\Develop\Extractum`
- Branch: `main`
- Shell: PowerShell
- User language: Russian. Prefer Russian for chat responses unless the user switches language.
- Current user intent: preserve the current session context in this file and provide a commit message.
- Current IDE active file: `docs/superpowers/specs/2026-05-08-secure-secret-storage-design.md`
- Open tabs visible in IDE:
  - `docs/superpowers/specs/2026-05-08-secure-secret-storage-design.md`
  - `docs/superpowers/plans/2026-05-08-secure-secret-storage.md`
  - `docs/superpowers/plans/2026-05-07-source-provider-readiness.md`

Note: `docs/superpowers/plans/2026-05-07-source-provider-readiness.md` was deleted earlier as a completed temporary plan artifact. The IDE tab may still be stale.

## Active Instructions And Working Style

- Use relevant Superpowers skills before acting. At minimum, start turns with `superpowers:using-superpowers`.
- For implementation or bugfix work, use the relevant Superpowers workflow:
  - `superpowers:test-driven-development` for code changes.
  - `superpowers:systematic-debugging` for bugs or failing behavior.
  - `superpowers:executing-plans` or `superpowers:subagent-driven-development` when executing a written plan.
  - `superpowers:verification-before-completion` before claiming success.
- Use `rg` or `rg --files` for search.
- Use `apply_patch` for manual file edits.
- Do not revert user changes.
- Git commands that write the index can fail under sandbox permissions. If a necessary `git add` or `git commit` fails due to `.git/index.lock` or permissions, rerun the same command with escalation and a concise Russian justification.

## Recent Completed Work

Source provider readiness has been completed and merged into `main`.

Important commits now on `main`:

- `b026dc4 docs(security): plan secure secret storage`
- `fddd817 docs: refresh provider readiness state`
- `3fd0e63 fix(db): register source subtype migration`
- `b8728c1 docs(sources): complete provider readiness verification`
- `eca5676 refactor(analysis): use provider-neutral refs`
- `2371fd0 refactor(sources): dispatch sync by provider`
- `2e43070 docs(session): refresh source provider readiness context`
- `1a4dcb7 refactor(sources): expose provider subtype`
- `677d561 refactor(analysis): gate source UI by capabilities`

Previously fixed bug:

- User saw `Error loading workspace sources: error returned from database: (code: 1) no such column: source_subtype`.
- Root cause: `src-tauri/migrations/15.sql` existed but migration v15 was not registered in `src-tauri/src/migrations.rs`.
- Fix committed as `3fd0e63`.

Provider-readiness verification that passed after merge:

- `npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts src/lib/api/sources.test.ts src/lib/analysis-state.test.ts`
- `npm.cmd run check`
- `cargo test sources::`
- `cargo test analysis::`
- `cargo test migrations::`
- `git diff --check`

## Current Working Tree State

After secure-storage planning was saved, the plan/spec files were committed as:

```text
b026dc4 docs(security): plan secure secret storage
```

Current `git status --short --branch --untracked-files=all` before staging this file shows:

```text
## main
?? docs/session-context-2026-05-03.md
```

That means the only current working-tree change is this recreated handoff file. No implementation code has been changed for secure storage yet.

## Secure Secret Storage Planning

User asked to discuss and plan Secure Secret Storage. The planning outcome is saved in:

- `docs/superpowers/specs/2026-05-08-secure-secret-storage-design.md`
- `docs/superpowers/plans/2026-05-08-secure-secret-storage.md`

Locked decisions:

- Scope: LLM API keys plus Telegram `api_hash`.
- Out of scope for this implementation: Telegram session JSON migration/encryption.
- Backend: Rust `keyring` crate using OS credential storage.
- Service name: `org.ai.extractum`.
- Secret IDs:
  - `llm.profile.<profile_id>.api_key`
  - `telegram.account.<account_id>.api_hash`
- Migration policy:
  - lazy automatic migration from old plaintext SQLite values;
  - write to keyring first;
  - only after successful keyring write, delete or blank plaintext SQLite value;
  - if keyring fails, fail closed and leave legacy plaintext untouched.
- LLM UI semantics:
  - frontend no longer receives saved key values;
  - `LlmProfile` exposes `api_key_configured: boolean`;
  - empty API key field on save preserves the existing key;
  - typing a new key replaces it;
  - add `Clear API key` only for LLM profiles.
- Telegram semantics:
  - `create_account` writes `api_hash` to keyring and stores `accounts.api_hash = ""`;
  - restore/init/sign-in read `api_hash` through backend secure storage;
  - deleting an account removes its Telegram keyring secret.
- Documentation updates are included in the same implementation task.

Planned dependency:

```toml
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
```

Planned new backend module:

- `src-tauri/src/secret_store.rs`

Planned backend trait shape:

```rust
pub(crate) trait SecretStore: Send + Sync {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>>;
    fn set_secret(&self, key: &str, value: &str) -> AppResult<()>;
    fn delete_secret(&self, key: &str) -> AppResult<()>;
}
```

Important implementation note:

- `keyring` operations are synchronous, so async command paths should call them through `tauri::async_runtime::spawn_blocking`.
- Rust tests must use a mock/in-memory secret store and must not depend on real OS credential storage.

## Planned Secure Storage Files To Touch

Likely implementation files:

- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/src/secret_store.rs`
- `src-tauri/src/llm/types.rs`
- `src-tauri/src/llm/profiles.rs`
- `src-tauri/src/llm/mod.rs`
- `src-tauri/src/accounts.rs`
- `src-tauri/src/telegram.rs`
- `src/lib/types/llm.ts`
- `src/lib/api/llm.ts`
- `src/lib/api/llm.test.ts`
- `src/routes/settings/+page.svelte`
- `README.md`
- `docs/project.md`
- `docs/database-schema.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`
- `docs/backlog.md`

Implementation should follow the saved plan rather than this context file if details differ.

## Planned Verification For Secure Storage Implementation

Use these commands after implementation:

```powershell
npm.cmd test
npm.cmd run check
cargo test llm::
cargo test accounts::
cargo test telegram::
cargo test migrations::
git diff --check
```

For the current documentation-only planning state, previous verification already run:

- placeholder scan on the secure storage plan/spec found no unresolved placeholders;
- trailing-whitespace scan on the secure storage plan/spec found no matches;
- `git diff --check` returned exit code 0.

## Suggested Commit Messages

The current working tree contains only this session context file, so use:

```text
docs(session): capture secure storage planning context
```

## Next Recommended Step

Commit the documentation artifacts, then start implementation from:

```text
docs/superpowers/plans/2026-05-08-secure-secret-storage.md
```

Recommended execution skill when implementation starts:

- `superpowers:executing-plans` for inline execution; or
- `superpowers:subagent-driven-development` if the user explicitly asks for subagents.
