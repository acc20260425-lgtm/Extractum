# Grammers crates.io 0.10.0 Source Migration Design

**Date:** 2026-08-11

**Status:** Approved for implementation planning

## Goal

Move Extractum's complete direct `grammers-*` dependency line from the
Codeberg Git revision
`5c6d44ff30e02d6c9295bcf1fcb51403ad77c981` to the crates.io releases at the
exact version `0.10.0`, while preserving the feature selection, resolved
behavior, and `extractum-telegram` public interface established by the
previous `grammers 0.10` upgrade.

The crates.io releases of `grammers-client`, `grammers-mtsender`,
`grammers-session`, and `grammers-tl-types` at `0.10.0` are available and
publish the feature flags currently requested by Extractum.

## Scope

The slice owns all active project state that declares or enforces the current
Grammers dependency source:

- the four workspace dependency declarations and `Cargo.lock`;
- the Grammers feature-baseline generator and generated JSON artifact;
- the repository rule and its current-state fixtures;
- the active dependency-policy description in `docs/project.md`;
- focused Rust and repository-rule verification plus the final workspace gate.

Historical specifications, plans, verification records, archived migration
notes, and diagnostic snapshots remain unchanged. They continue to describe
the source and evidence that applied when they were recorded.

No product behavior, Telegram schema handling, session format, persistence
format, public Rust interface, or frontend interface changes in this slice.
If the registry packages expose an API or resolved-feature difference from the
previous Git source, stop and re-plan instead of adapting product behavior in
this source-only migration.

## Dependency Declaration and Resolution

Declare each direct dependency with the exact Cargo requirement `=0.10.0`:

- `grammers-client`, preserving `default-features = false`;
- `grammers-mtsender`, preserving its current default-feature behavior;
- `grammers-session`, preserving `default-features = false` and `serde`;
- `grammers-tl-types`, preserving `deserializable-functions` and its current
  default-feature behavior.

Regenerate `Cargo.lock` through Cargo so all resolved `grammers-*` packages use
the crates.io registry source and registry checksums. Audit the lockfile diff;
unrelated dependency upgrades are excluded where Cargo permits. The completed
graph must contain no Git-sourced `grammers-*` package.

The exact manifest requirement is part of repository policy, not merely the
version currently selected by the lockfile. This prevents a future lockfile
refresh from silently moving the dependency line beyond `0.10.0`.

## Feature Baseline and Repository Policy

Migrate the generated baseline schema from Git identity to release identity:

- replace the top-level `revision` field with `version: "0.10.0"`;
- make the generator require the canonical crates.io registry source and the
  exact selected version for every direct Grammers package;
- make the repository rule verify the direct manifest requirements, resolved
  versions, registry sources, package inventory, and enabled feature closure;
- update positive and negative repository-rule fixtures to represent crates.io
  packages and reject a wrong version, a Git source, a path source, or another
  registry source.

The baseline JSON remains generated output: edit the generator, then regenerate
the artifact with its `--write` mode. A diff in `universe` or `forbidden` is an
upstream publication fact and may be accepted after review. Any diff in
`required` is a feature-policy change and must be investigated before the
slice proceeds.

## Documentation Policy

Rewrite only the current-state dependency-policy text in `docs/project.md` so
it identifies crates.io and exact version `0.10.0` as the active source. Keep
the surrounding historical Codeberg/GitHub migration narrative and the prior
sanitized verification record intact, clearly marked as historical evidence.

## Error Handling and Rollback

No new runtime error path is introduced. Cargo resolution, baseline generation,
or repository-rule failure closes the migration before product code changes.
Rollback is a revert of the isolated source-migration commits; there is no data
or configuration migration.

## Testing Strategy

The initial source switch supplies the RED signal: the existing baseline and
repository rule still require the Codeberg revision and must reject registry
metadata. After updating the policy, focused repository-rule tests must include
non-empty negative coverage for source and version drift.

Compare the regenerated baseline to the previous artifact. The `required`
arrays must remain policy-equivalent. Cargo metadata provides the authoritative
completion audit: all four direct packages must resolve at `0.10.0` from the
canonical crates.io registry source, and no Git-sourced Grammers package may
remain.

Because the selected semantic version remains `0.10.0` and this slice does not
change product code, the previous live Telegram and Takeout smoke record remains
the behavioral evidence. A new live smoke run is not required unless automated
verification reveals a runtime-relevant difference.

## Rust Verification Loops

Affected package: `extractum-telegram`. No Rust source file is expected to
change, but its full target graph proves that the crates.io artifacts remain
source-compatible with the existing adapters and fixtures.

Focused policy loop:

- run the repository-rule test that owns the Grammers dependency policy;
- run `node scripts/telegram-grammers-feature-baseline.mjs --check`;
- inspect locked `cargo metadata` for exact versions, canonical registry
  sources, direct ownership, and the absence of Git-sourced Grammers packages.

Package gates:

- `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`.

End-of-slice workspace gate:

- `npm.cmd run verify`.

## Completion Criteria

- The four direct dependencies use exact Cargo requirements `=0.10.0` and the
  locked Grammers graph contains only crates.io registry sources.
- The generated baseline and repository rule enforce version `0.10.0`, the
  crates.io source, the existing direct package inventory, and the unchanged
  required-feature policy.
- Current project documentation names crates.io as active without rewriting
  historical migration evidence.
- Focused package checks and `npm.cmd run verify` pass.
