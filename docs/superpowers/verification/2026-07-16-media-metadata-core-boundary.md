# Media Metadata Core Boundary Verification

## Scope and Commits

- Starting commit: `858427500880857ca99e8add2a8956008373d8fb`.
- Characterization commit: `6ef52a38d0d7001d6adba946b0f4d71db9ade8e9`
  (`test: characterize absent media metadata`).
- Implementation commit: `8f98eeff6098e023009d6873eb2030ab0dcc26ec`
  (`refactor: move media metadata into core`).
- The slice moved `ItemMediaMetadata`, its encode/decode functions,
  `media_label`, and their four tests into `extractum-core`.
- `src-tauri/src/media.rs` retains the existing application path through an
  explicit `pub(crate) use` facade. Grammers-backed extraction, payload types,
  constants, and adapter logic remain in the application crate.
- The full repository gate exposed an older exact core-module allowlist in
  `rust-workspace-core-contract.test.ts`; the implementation commit updates
  that curated contract for `media_metadata` and `serde_json`.

## Dependency Boundary

- Cargo metadata reports two workspace members and the canonical target at
  `src-tauri/target`.
- `extractum-core` has exactly four direct dependency roots:
  `serde`, `serde_json`, `time`, and `zstd`.
- No member manifest defines a Cargo profile.
- The core source contains no `grammers`, `tauri`, or `sqlx` dependency.
- `Cargo.lock` changed only by adding `serde_json` to the local
  `extractum-core` dependency list; no external package version changed.

## Test Inventory and Rename Map

- Baseline inventory: 1125 unique Rust tests.
- Post-extraction inventory: 1126 unique Rust tests.
- Missing tests outside the declared rename map: 0.
- Duplicate post-extraction names: 0.

Declared renames:

- `media::tests::media_label_covers_known_and_fallback_kinds` →
  `media_metadata::tests::media_label_covers_known_and_fallback_kinds`.
- `media::tests::media_metadata_roundtrip_through_zstd` →
  `media_metadata::tests::media_metadata_roundtrip_through_zstd`.
- `media::tests::media_metadata_decode_failures_are_typed_internal_errors` →
  `media_metadata::tests::media_metadata_decode_failures_are_typed_internal_errors`.

The application adapter tests remain under their original names:

- `media::tests::derive_content_kind_tracks_text_and_media_presence`.
- `media::tests::derive_document_media_kind_prefers_specific_signals`.

The new test
`media_metadata::tests::absent_media_metadata_decodes_to_default` accounts for
the exact net increase of one test and exists only in core.

## Focused and Complete Verification

- The new source-boundary contract first collected five tests and produced the
  intended RED: four failed on the absent dependency/module/facade/test move,
  while the already-satisfied heavyweight-dependency prohibition passed.
- After extraction, the media metadata contract passed 5/5 tests. Together
  with the updated workspace core contract, the focused contract run passed
  10/10 tests across two files.
- `cargo check -p extractum-core --all-targets` passed.
- `cargo test -p extractum-core --all-targets` passed 22/22 tests.
- The `media::tests` substring filter passed four tests: the two intended app
  adapter tests plus two matching `notebooklm_export::media::tests`. The full
  inventory confirms that only the two adapter tests remain in the root
  `media::tests` namespace.
- Consumer filters passed with nonzero inventories:
  `notebooklm_export` 76 tests, `sources` 178 tests, and `takeout_import` 73
  tests.
- `cargo check --workspace --all-targets` passed without a new warning.
- `cargo test --workspace --all-targets` passed 1104 application-library tests,
  0 application-binary tests, and 22 core tests: 1126 total.
- `npm.cmd run verify` passed after the curated workspace contract was updated:
  161 Vitest files / 1277 tests, Svelte diagnostics with 0 errors and 0
  warnings, rustfmt, full workspace Cargo check/test, and diff hygiene.

## Release Build and Startup Smoke

- `npm.cmd run tauri -- build --no-bundle` exited 0 and produced
  `src-tauri/target/release/extractum.exe`.
- The release build retained two pre-existing, out-of-scope warnings: the
  unused `Manager` import in `takeout_import/state.rs` and the unused
  `PromptPackRunState::track` method.
- The release executable opened a visible main window titled `extractum`.
- A normal window-close request was accepted; the process exited with code 0,
  and no `extractum` process remained.

## Limitations

- Desktop navigation automation was unavailable in this execution session, so
  no navigation action was performed. Startup and normal shutdown were
  observed programmatically; navigation remains a human-observation item and
  is not claimed as passed.
- The broad `media::tests` libtest filter also matches nested module paths; the
  namespace inventory, rather than that substring count alone, proves that
  exactly two adapter tests remain in the app module.
- This architectural slice has no compilation-performance acceptance
  threshold. The release build duration is evidence of buildability, not a
  performance result.

## Result and Follow-Up

The media metadata boundary is accepted: pure metadata behavior and its tests
now live in the minimal core crate, existing application import paths remain
stable through an explicit facade, the complete test inventory is preserved
with the declared renames, and all automated gates pass.

The next proposed slice is a fresh dependency map and design for the smallest
pure `notebooklm_export` crate boundary. It should be evaluated independently
before moving production code.
