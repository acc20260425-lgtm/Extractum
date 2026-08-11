# Grammers crates.io 0.10.0 Source Migration Design

**Date:** 2026-08-11

**Status:** Approved for implementation planning

## Goal

Move Extractum's complete eight-package `grammers-*` graph from Codeberg
revision `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` to the corresponding crates.io
releases, preserving the feature selection, resolved behavior, and
`extractum-telegram` public interface established by the previous upgrade.

## Scope

The slice owns the active project state that declares or enforces the current
Grammers dependency source:

- the four direct workspace declarations and the complete eight-package graph
  in `Cargo.lock`;
- the feature-baseline generator and generated JSON artifact;
- the repository rule and its current-state fixtures;
- the reproducible provenance record in
  `docs/superpowers/verification/2026-08-11-grammers-crates-io-provenance.md`;
- the active dependency-policy description in `docs/project.md`;
- focused Rust, policy, and workspace verification.

Historical specifications, plans, verification records, migration notes, and
diagnostic snapshots remain unchanged. No product behavior, Telegram schema,
session or persistence format, public Rust interface, or frontend interface
changes in this slice. If registry resolution exposes an API or required-feature
difference, stop and re-plan instead of adapting product behavior.

## Published Artifact Equivalence

The reproducible provenance record confirms that all eight downloaded artifacts
identify Git SHA `5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` and that every file shared with
the cached checkout is byte-identical. It covers the seven `0.10.0` packages
and `grammers-tl-parser 1.2.2`, recording the commands, cache paths, excluded
packaging files, two unpublished test-support files, and observed output.

The crates.io artifacts therefore originate from the existing pin and retain
its runtime source. The previous live Telegram and Takeout smoke record remains
applicable; no repeated live smoke is required. If this provenance check cannot
be reproduced or later reveals runtime-source drift, repeat the smoke checks or
record explicit risk acceptance before completing the slice.

## Dependency Declaration and Resolution

Declare all four direct dependencies with exact Cargo requirement `=0.10.0`,
preserving their current default-feature settings and requested features:

- `grammers-client`: defaults disabled;
- `grammers-mtsender`: current defaults;
- `grammers-session`: defaults disabled, `serde` enabled;
- `grammers-tl-types`: current defaults plus `deserializable-functions`.

Regenerate `Cargo.lock` through Cargo. The resolved graph must contain seven
Grammers packages at `0.10.0` and `grammers-tl-parser` at `1.2.2`, all from the
canonical crates.io registry source, with no Git-sourced Grammers package.
Cargo enforces the registry checksums while resolving and building. Keep
unrelated lockfile updates out where Cargo permits.

The exact direct requirements make intent visible in the manifest. The
generated baseline additionally records the inventory and individual versions
of all eight resolved packages, preventing a lock refresh from silently moving
transitive Grammers crates. If a future valid resolution cannot satisfy exact
mutually dependent direct pins, those requirements may be relaxed to caret only
while retaining the eight-package resolved-version policy.

## Feature Baseline and Repository Policy

The existing generator and repository rule enforce the direct four-package
inventory, canonical source, feature universe, and enabled-feature closure.
This slice extends resolved-graph ownership to all eight packages while keeping
direct feature ownership unchanged:

- The generator replaces `revision` with `version: "0.10.0"`, replaces its
  Git `exactSource` with the canonical crates.io registry source, and keeps the
  existing direct-package feature records.
- The generator adds a canonical `resolvedPackages` inventory derived from all
  `grammers-*` entries in Cargo metadata. It records seven packages at `0.10.0`
  plus `grammers-tl-parser` at `1.2.2`, rejects any non-canonical source, and
  makes inventory or version drift change the generated baseline.
- The repository rule requires every direct Grammers dependency in
  `extractum-telegram` metadata to have manifest requirement `=0.10.0`. It
  relies on canonical baseline generation plus `sameJson` for the complete
  resolved inventory, versions, and sources, and removes its redundant second
  source comparison.
- Repository-rule fixtures move from Git package identities to registry
  identities.

Negative coverage needs two policy classes: non-canonical source and release
drift. Source coverage must exercise a transitive package; release-drift
coverage must exercise a transitive resolved version and a direct manifest
requirement. Separate Git/path/alternate-registry cases are unnecessary because
they share the same canonical-source comparison.

The JSON remains generated output. Update the generator, then regenerate the
artifact with `--write`. A diff in `universe` or `forbidden` is an upstream
publication fact and may be accepted after review. Any diff in `required` is a
feature-policy change and stops the slice for investigation.

## Documentation Policy

Update only the current-state dependency-policy text in `docs/project.md` to
name crates.io and exact version `0.10.0`; retain the surrounding historical
narrative and verification evidence.

## Rust Verification Loops

Affected package: `extractum-telegram`. No Rust source is expected to change,
but its test graph proves that the published artifacts remain compatible with
the existing adapters and fixtures.

RED/GREEN policy loop:

- after switching the manifest but before updating the generator, confirm that
  the pre-migration baseline rejects registry metadata;
- run the focused repository-rule test with non-empty source and release-drift
  cases;
- run `node scripts/telegram-grammers-feature-baseline.mjs --check`;
- inspect locked `cargo metadata` for exact manifest requirements, selected
  eight-package inventory and versions, canonical registry sources, unchanged
  required features, and no Git-sourced Grammers packages.

Package gates:

- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.

End-of-slice gate: `npm.cmd run verify`.

## Compatibility and Rollback

No migrations are introduced; rollback is a revert of the source-migration
commit.

## Completion Criteria

- All four direct declarations require `=0.10.0`; the locked graph contains
  seven Grammers packages at `0.10.0` and `grammers-tl-parser` at `1.2.2`, all
  from crates.io and with no Grammers Git source.
- Repository-rule fixtures fail for a transitive non-canonical source and for
  release drift in both a transitive resolved version and direct manifest
  requirement.
- `npm.cmd run verify` passes.
