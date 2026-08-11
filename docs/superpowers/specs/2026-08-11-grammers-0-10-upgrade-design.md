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

Update all four direct grammers dependencies to the same upstream revision.
Do not mix registry and Git variants or split the dependency line across
revisions.

## Error Policy

Session state is fail-closed. Every `MemorySessionError` from reading or
writing the home DC, DC options, update state, or peer cache is converted to an
`AppError` and propagated to the owning operation. Production code must not use
`expect`, `unwrap`, fabricated defaults, or silent logging to hide these
errors.

`TelegramSession::cache_peer_infos` becomes fallible. Its live-message caller
must stop and return the error when the peer cache cannot be updated, because
continuing could make peer identity and access-hash behavior inconsistent.

Avatar loading remains the sole explicit best-effort boundary. A failure from
`Peer::photo` is converted to `AppError`, then the existing timeout/best-effort
wrapper suppresses it and returns no avatar.

The new `InvocationError::Session` variant is not a local transport error and
must not trigger export-DC fallback. It follows the ordinary non-fallback error
path. Existing fallback behavior remains limited to invalid DC, I/O,
transport, authentication, and dropped-connection failures.

## Telegram TL Layer 227

Update raw TL test fixtures for fields introduced by layer 227, including
`Message::rich_message`, `User::bot_guard`, and
`MessageReplyHeader::reply_to_ephemeral`. Use absent/false values so fixtures
continue to describe the same messages and peers.

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

Focused verification covers the `extractum-telegram` package and its exact
affected tests. The package checkpoint is
`cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
The focused check is
`cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`.
The producer production-surface check is
`cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.

The slice ends with `npm.cmd run verify`, as required for every Rust slice.
After automated verification, run live Telegram sync and Takeout smoke checks
against available test sources and record only sanitized identifiers, counts,
states, and warnings. Credentials, session material, API hashes, and raw
private payloads must not be committed.

## Compatibility and Rollback

No database migration, frontend API change, or new persisted value is
introduced. Existing encrypted session JSON retains the same Extractum schema;
only access to its in-memory grammers representation becomes fallible.

Rollback consists of reverting the isolated dependency slice: code adapters,
fixtures, four Git pins, lockfile, baseline, repository rules, and dependency
documentation move together back to the `0.9.0` revision.

## Completion Criteria

- All four grammers crates resolve from revision
  `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` at version `0.10.0`.
- No production session error is unwrapped, ignored, or replaced with a
  default.
- Avatar lookup remains best-effort and export-DC fallback excludes session
  failures.
- `extractum-telegram` compiles and all of its targets pass.
- The feature baseline and repository rules pass with the new revision.
- `npm.cmd run verify` passes.
- Live Telegram sync and Takeout smoke evidence is recorded separately without
  sensitive data.
