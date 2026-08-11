# Grammers crates.io 0.10.0 Source Migration Design

**Date:** 2026-08-11

**Status:** Approved for implementation planning

## Goal

Move Extractum's four direct `grammers-*` dependencies from Codeberg revision
`5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` to their crates.io releases at the
exact version `0.10.0`, preserving the feature selection, resolved behavior,
and `extractum-telegram` public interface established by the previous upgrade.

## Scope

The slice owns the active project state that declares or enforces the current
Grammers dependency source:

- the four workspace declarations and `Cargo.lock`;
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

The reproducible provenance record confirms that the downloaded `0.10.0`
artifacts for all four direct crates identify Git SHA
`5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` and that every file shared with the
cached checkout is byte-identical. It records the commands, cache paths,
excluded packaging files, the two unpublished test-support files, and observed
output.

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

Regenerate `Cargo.lock` through Cargo. All resolved `grammers-*` packages must
use the canonical crates.io registry source and registry checksums, with no Git-
sourced Grammers package left in the graph. Keep unrelated lockfile updates out
where Cargo permits.

The exact requirement makes the intent visible in the manifest and prevents a
lock refresh from silently moving beyond `0.10.0`. If a future valid Grammers
resolution cannot satisfy exact mutually dependent pins, the policy may be
relaxed to a caret requirement only while retaining exact resolved-version
enforcement in the repository rule.

## Feature Baseline and Repository Policy

The existing generator and repository rule already enforce the four-package
inventory, canonical source, feature universe, and enabled-feature closure.
Responsibility for the migration delta is split explicitly:

- The generator replaces `revision` with `version: "0.10.0"`, replaces its
  Git `exactSource` with the canonical crates.io registry source, and requires
  every selected direct metadata package to have version `0.10.0`.
- The repository rule requires every direct Grammers dependency in
  `extractum-telegram` metadata to have manifest requirement `=0.10.0`. It
  relies on canonical baseline generation for resolved version and source, and
  removes its redundant second source comparison.
- Repository-rule fixtures move from Git package identities to registry
  identities.

Negative coverage needs two policy classes: non-canonical source and release
drift. Release-drift coverage must exercise both the selected package version
and the direct manifest requirement; separate Git/path/alternate-registry cases
are unnecessary because they share the same canonical-source comparison.

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
  versions, canonical registry sources, unchanged required features, and no
  Git-sourced Grammers packages.

Package gates:

- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.

End-of-slice gate: `npm.cmd run verify`.

No migrations are introduced; rollback is a revert of the source-migration
commit.

## Completion Criteria

- All four manifests require `=0.10.0`; the locked Grammers graph resolves
  `0.10.0` only from crates.io and contains no Grammers Git source.
- Repository-rule fixtures fail for a non-canonical source and for release
  drift in both the resolved package version and direct manifest requirement.
- `npm.cmd run verify` passes.
