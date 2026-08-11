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

Any other breaking `grammers 0.10` API changes found by compilation are
adapted in this slice without expanding product behavior.

## Architecture

Keep `grammers-*` isolated in the existing `extractum-telegram` package. Do not
add a second session wrapper or expose grammers types across package boundaries.
Adapt the existing `TelegramSession`, live, avatar, and Takeout adapters in
place, preserving their current responsibilities. Internal return types may
become `AppResult` where fallible session access requires propagation.
All affected functions are private or `pub(super)`; the public interface of
`extractum-telegram` does not change.

## Error Policy

Session access is fail-closed: convert `MemorySessionError` to `AppError` and
use `?` at every owning operation. Avatar loading retains its existing
best-effort outer boundary.

| Location | Signature | Internal adaptation |
| --- | --- | --- |
| `memory_session_to_saved` | change `SavedSession` to `AppResult<SavedSession>` | propagate `home_dc_id`, `updates_state`, and `dc_option` failures |
| `TelegramSession::cache_peer_infos` | change `()` to `AppResult<()>` | propagate `cache_peer` failure; its single caller uses `?` |
| `peer_photo_bytes` | unchanged: `AppResult<Option<Vec<u8>>>` | propagate the new `Peer::photo` result; the timeout wrapper still suppresses the resulting `AppError` |
| `prepare_export_dc_alias` | unchanged: `AppResult<(i32, i32)>` | use `?` for session errors, then preserve `ok_or_else` for a missing DC option before propagating `set_dc_option` |

Replace the export-DC `matches!` allowlist with an exhaustive `match` and make
`InvocationError::Session` an explicit non-fallback branch. Extend the existing
fallback unit test with a `Session` assertion so the policy has both compile-
time exhaustiveness and runtime coverage.

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
- the current dependency-policy version, revision, and validation note in
  `docs/project.md`.

Preserve the current requested feature policy: client default features remain
disabled, session keeps only `serde`, mtsender keeps its existing default
selection, and TL types retain `deserializable-functions` plus their required
generated-code features. Review lockfile churn and keep unrelated dependency
updates out of the slice where Cargo permits.

## Testing Strategy

Use test-driven development for behavior changes. `MemorySessionError` has no
production failure-injection seam, so its propagation is enforced by return
types and compilation rather than a new mock abstraction. Tests cover avatar
best-effort behavior and export-DC fallback; an exhaustive `InvocationError`
match provides a compile-time RED signal for the new `Session` variant. Raw TL
fixture compilation supplies the RED signal for schema additions, then the
fixtures receive only the new neutral field values.

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

- All four grammers pins resolve revision
  `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` at version `0.10.0`.
- `npm.cmd run verify` passes.
- Sanitized live Telegram sync and Takeout smoke evidence is recorded.
