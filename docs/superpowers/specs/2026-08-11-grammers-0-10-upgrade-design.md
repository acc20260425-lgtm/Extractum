# Grammers 0.10 Upgrade Design

**Date:** 2026-08-11

**Status:** Approved for implementation planning

## Goal

Upgrade the complete Extractum `grammers-*` dependency line from Codeberg
revision `1f901ce6e973fdcf0e74267f3d8efad5c729daaa` (`0.9.0`) to revision
`5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` (`0.10.0`) while preserving the
existing Telegram live-sync, Takeout, session-persistence, source-identity,
and feature-selection behavior.

## Scope

The slice owns:

- the four workspace dependencies `grammers-client`, `grammers-mtsender`,
  `grammers-session`, and `grammers-tl-types`;
- the `extractum-telegram` compatibility changes required by the new fallible
  session interface and Telegram TL layer 227;
- test fixtures that construct raw TL values;
- the grammers feature baseline, repository-rule fixtures, lockfile, and
  dependency-policy documentation;
- focused Rust verification, the workspace verification gate, and a separate
  live Telegram/Takeout smoke record.

The slice does not add product support for rich messages, guard bots, or other
new layer-227 capabilities. New optional TL fields are accepted and otherwise
ignored by existing extraction behavior.

## Architecture

Keep `grammers-*` isolated in the existing `extractum-telegram` package. Do not
add a second session wrapper or expose grammers types across package boundaries.
Adapt the existing `TelegramSession`, live, avatar, and Takeout adapters in
place, preserving their current responsibilities and public interfaces unless
fallible session access requires an internal `AppResult` return.

## Error Policy

Session access is fail-closed: convert `MemorySessionError` to `AppError` and
use `?` at every owning operation. Avatar loading retains its existing
best-effort outer boundary.

| Location | Before | After |
| --- | --- | --- |
| `memory_session_to_saved` | `async fn(...) -> SavedSession` | `async fn(...) -> AppResult<SavedSession>` |
| `TelegramSession::cache_peer_infos` | `async fn(...)` | `async fn(...) -> AppResult<()>`; its single caller uses `?` |
| `peer_photo_bytes` | `Peer::photo(...).await -> Option<_>` | unwrap `Result` with `?`; the timeout wrapper still suppresses the resulting `AppError` |
| `prepare_export_dc_alias` | session reads/writes are infallible | propagate `home_dc_id`, `dc_option`, and `set_dc_option` failures with `?` |

`InvocationError::Session` is not an export-DC fallback condition.

## Telegram TL Layer 227

Set every new required layer-227 field in raw TL test fixtures to its neutral
absent/false value so fixtures continue to describe the same messages and
peers. The compiler supplies the authoritative field list.

Production parsing continues to extract the existing text, media, reaction,
reply, peer, and topic fields. It must compile against layer 227 without
claiming support for new rich-message semantics.

## Dependency and Repository Metadata

Update:

- the four workspace Git pins and `Cargo.lock`;
- `src/lib/telegram-grammers-feature-baseline.json`;
- `scripts/telegram-grammers-feature-baseline.mjs`;
- the grammers revision fixture in `scripts/testing/repository-rules.test.ts`;
- the current dependency-policy revision and validation note in
  `docs/project.md`.

Preserve the current requested feature policy: client default features remain
disabled, session keeps only `serde`, mtsender keeps its existing default
selection, and TL types retain `deserializable-functions` plus their required
generated-code features. Review lockfile churn and keep unrelated dependency
updates out of the slice where Cargo permits.

## Testing Strategy

Use test-driven development for behavior changes. Tests must first demonstrate
that session failures propagate and that avatar failures remain best-effort.
Raw TL fixture compilation supplies the RED signal for schema additions, then
the fixtures receive only the new neutral field values.

Use exact tests and
`cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`
inside the RED/GREEN loop. The slice gates are:

- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`;
- `npm.cmd run verify`.

After automated verification, run live Telegram sync and Takeout smoke checks
against available test sources and record only sanitized identifiers, counts,
states, and warnings. Credentials, session material, API hashes, and raw
private payloads must not be committed.

## Compatibility and Rollback

No migration or API change is introduced; rollback is a revert of the isolated
dependency-slice commit.

## Completion Criteria

- All four grammers pins resolve revision `5c6d44f` at version `0.10.0`.
- `npm.cmd run verify` passes.
- Sanitized live Telegram sync and Takeout smoke evidence is recorded.
