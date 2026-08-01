# Telegram Phase 8C Extraction Design

**Status:** Implemented and retained; [verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)
**Date:** 2026-08-01

**Parent boundary design:**
[`2026-07-26-telegram-crate-boundary-design.md`](2026-07-26-telegram-crate-boundary-design.md)

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

**Retained Phase 8B plan:**
[`2026-07-28-extractum-telegram-8b-preparation.md`](../plans/2026-07-28-extractum-telegram-8b-preparation.md)

**Retained Phase 8B evidence:**
[`2026-07-28-extractum-telegram-8b-preparation.md`](../verification/2026-07-28-extractum-telegram-8b-preparation.md)

## Purpose

Phase 8B completed the Telegram preparation slice inside the application
package. Phase 8C must now create `extractum-telegram`, move the prepared
19-file tree, transfer its low-level dependencies, and leave the application
behind the existing private compatibility facade.

The retained 8B tree exposed one extraction-specific issue that could not be
observed while producer and consumer compiled in the same package: four
app-owned Takeout tests construct two private-field crate-owned values through
`#[cfg(test)] pub(crate)` helpers. A dependency is not compiled with the
consumer's `cfg(test)`, and `pub(crate)` cannot cross a package boundary.
Therefore a literal byte-for-byte move of all 19 files cannot compile those
four retained app tests.

This design resolves that issue inside one atomic extraction checkpoint. The
immutable 8B artifact remains the path-inventory authority, while an exact
two-file patch and Git blob comparisons make the only moved-source adaptation
explicit and auditable.

## Decision

Use one forward-only, atomic Phase 8C implementation checkpoint. Before edits,
record the clean repository HEAD as `BASE_COMMIT` and capture the retained app
test identities. `BASE_COMMIT` is evidence and a rollback anchor, not a new
implementation checkpoint.

The single extraction change creates the package edge, moves the 19-file tree,
applies the exact two-file fixture-boundary patch described below, creates the
private app facade, updates the existing boundary contract once for the final
layout, and transfers dependency ownership. After focused and full deterministic
gates plus the release build pass, retain that state as `EXTRACTION_COMMIT`.

The correction introduces a non-default `app-test-support` feature and exposes
exactly two fixture functions when that feature is enabled. The production
dependency does not enable the feature. The application dev-dependency does,
and the private app facade re-exports the two functions only under
`#[cfg(test)]`.

No production behavior, public wire contract, persistence format, source
ownership, or normal-build API is changed by the fixture-boundary patch.

## Considered Alternatives

### Edit only the parent design

Rejected. The parent is the retained record of the full 8A/8B/8C boundary and
already describes completed 8A/8B decisions. Rewriting it deeply would blur
retained history and make the new 8C exception hard to audit.

### Create an unlinked standalone specification

Rejected. Two active descriptions of 8C without explicit precedence would be
ambiguous.

### Reopen Phase 8B

Rejected. Phase 8B is complete and retained. The cross-package `cfg(test)`
problem is an extraction concern discovered from the retained final state; it
does not invalidate the achieved 8B package-portability boundary.

### Make constructors permanently public

Rejected for boundary consistency, not because the constructors enforce hidden
invariants. Their arguments are already observable through read-only public
accessors, but making them public would add unconditional production
construction names beyond the curated 8B surface. The selected feature gates
the existing fixture seam and leaves the normal-build type surface unchanged.

## Authority and Precedence

This owner-approved document is the normative authority for Phase 8C where it
explicitly differs from the parent design. Approval authorizes implementation
planning only; it does not authorize implementation. This document supersedes
only these parent clauses:

1. 8C may apply the exact two-file fixture-boundary patch below inside the
   atomic extraction checkpoint;
2. the feature-enabled crate surface may contain exactly the two test fixture
   functions specified below;
3. final content authority consists of exact Git blob equality for 17 moved
   files and the exact approved patch for the other two, while the immutable 8B
   artifact remains retained evidence and the exact 19-path inventory;
4. the app may declare the same path package once in normal dependencies and
   once in dev-dependencies solely to enable `app-test-support` for app tests;
5. 8C observes its final fixture-boundary RED inside the uncommitted extraction
   change and retains neither a separate RED-contract commit nor a fixture
   preparation commit; the sole implementation commit is
   `EXTRACTION_COMMIT`. This explicitly replaces the parent's separate
   committed-RED and mechanical-extraction commits;
6. the single 8C fixture-boundary RED is a bounded exception to the parent's
   runtime-only RED rule: compilation of the `extractum` library test target
   fails with the exact `E0432` fixture-import diagnostic before any test binary
   exists. No other compiler failure or zero-test execution is RED evidence.

Every parent requirement not named above remains in force. In particular,
this document does not reopen 8A or 8B, expand the production API, alter the
19-path ownership map, or authorize application consumer rewrites.

## Frozen Starting Identity

The frozen source identity for Phase 8C is retained 8B terminal commit
`c4c446b1169733d8623f84bbda5e028c2e7fa365`. Later documentation-only commits
do not replace that authority.

The implementation plan must verify these files before its first source edit:

| Authority | SHA-256 |
| --- | --- |
| `src/lib/telegram-grammers-feature-baseline.json` | `774e2b979d5cdc8185a85488c965548cf09cdf1ef0ab4b9ecad58246283cf5b3` |
| `src/lib/telegram-8b-test-identities.json` | `507f09f4fab76bee4360185eca3fbef17fb1563e784f7654bebb430cf7f08a95` |
| `src/lib/telegram-8b-symbol-map.json` | `f978e80cd58303fd9cd6402ba17deef1df817d22fdbf821830e9e0a5968c13b3` |
| `src/lib/telegram-8b-staging-sha256.json` | `12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9` |
| `src-tauri/Cargo.toml` | `ee323d7b613573918d4ad3777b238bc7e107d049588ddcfa0959dacfd1e2cf69` |
| `src-tauri/Cargo.lock` | `720e38ea632d7b932b2a23d1481528845ec9304376035b1c851c546ea402e43c` |

Immediately before implementation, record the clean current HEAD as
`BASE_COMMIT`. Verify that the 19 staged source paths, `src-tauri/Cargo.toml`,
and `src-tauri/Cargo.lock` at `BASE_COMMIT` still match the frozen 8B source
identity and the hashes above. `BASE_COMMIT` may include later documentation
commits, but no Telegram source, manifest, or lockfile drift. Unexpected drift
stops the slice; it is not normalized inside Phase 8C.

## Cross-Package Test-Support Problem

The retained staged root currently re-exports:

```rust
#[cfg(test)]
pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;
#[cfg(test)]
pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;
```

The functions are defined in `telegram_impl/takeout/mod.rs` with the same
`#[cfg(test)] pub(crate)` boundary. These four app-owned tests consume them:

- `takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future`;
- `takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation`;
- `takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize`;
- `takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues`.

After extraction, `cfg(test)` applies to `extractum`, not automatically to its
dependency, and crate-private visibility cannot cross from
`extractum-telegram` to `extractum`. The four tests would therefore fail to
compile even though production code remained valid.

## Exact Test-Support Boundary

The new package owns one empty, non-default feature:

```toml
[features]
app-test-support = []
```

When that feature is enabled, the crate root may expose exactly these two free
functions:

```rust
#[cfg(feature = "app-test-support")]
pub fn takeout_attempt_fixture(
    home_dc_id: i32,
    export_dc_id: i32,
) -> TakeoutAttempt;

#[cfg(feature = "app-test-support")]
pub fn takeout_fallback_fixture(
    kind: TakeoutFallbackKind,
    warning: &str,
    provenance_message: Option<&str>,
) -> TakeoutFallback;
```

The feature exposes no module, field, raw Grammers type, secret, mutable
global, or additional constructor. The functions preserve their current
implementation and construct values only from already public owned boundary
types.

`app-test-support` is a development-isolation mechanism, not a security
boundary: any external consumer can explicitly enable a public Cargo feature.
It is authorized here only because it changes no production behavior, is
needed solely by consumer tests, exposes a two-function API, and is safe when
Cargo activates the consumer's dev-dependency. It must never expose secrets,
bypass authentication, or enable an administrative operation.

The producer definitions and crate-root re-exports use only
`#[cfg(feature = "app-test-support")]`; they do not use `cfg(test)` or
`cfg(any(...))`. No `extractum-telegram` test consumes the fixtures. This is a
required ownership rule: isolated `-p extractum-telegram` checks/tests compile
the producer feature-off, while workspace `--all-targets` gates compile it
feature-on through the app dev-dependency. A producer test that used either
fixture would fail in the isolated package checkpoint while workspace feature
unification could mask that invalid reliance. Both configurations must remain
GREEN.

The app does not temporarily declare the feature, and no app test or production
call site changes. The two `pub fn` definitions remain inside the private
`takeout` module; their two crate-root `pub use` declarations expose exactly two
external fixture names, not four.

The extraction adds these dependency declarations to the app package:

```toml
[dependencies]
extractum-telegram = { path = "crates/extractum-telegram" }

[dev-dependencies]
extractum-telegram = {
  path = "crates/extractum-telegram",
  features = ["app-test-support"],
}
```

The feature is declared only by `extractum-telegram` and is non-default. The
normal app dependency remains feature-free; only the app dev-dependency enables
it.

The private compatibility facade contains the normal explicit production
allowlist plus only this test-gated addition:

```rust
#[cfg(test)]
pub(crate) use extractum_telegram::{
    takeout_attempt_fixture,
    takeout_fallback_fixture,
};
```

### Resolver-v2 semantics and exact evidence

The workspace uses Cargo resolver version 2. In a normal library build,
features requested only by a dev-dependency are not unified into the normal
dependency. Consumer tests, examples, benches, and consumer `--all-targets`
commands require dev-dependencies, so both
`cargo check -p extractum --all-targets` and
`cargo test -p extractum --all-targets` enable and unify
`app-test-support`. Neither command is feature-off evidence.

Cargo activation has exactly two verification invariants:

1. `extractum-telegram` compiles as a library without
   `app-test-support`:
   `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`;
2. the consumer package tests compile and pass with the feature enabled through
   its dev-dependency:
   `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.

No separate feature-unification contract or explicit feature-on crate check is
required. The exact normal/dev manifest declarations above remain part of the
dependency-ownership evidence. The standing curated source/API allowlist still
allows exactly those two feature-gated definitions in the private `takeout`
module and their two corresponding crate-root re-exports. It rejects an
unconditional fixture, any additional feature-gated definition or root export,
and any third externally reachable fixture name. This is API-boundary evidence,
not a second Cargo feature model.
`cargo tree -e features` is a diagnostic when the resolved graph is surprising,
not an additional completion gate.

## Extraction Content Authority

The immutable 8B staging artifact remains unchanged. It continues to prove the
retained 8B result, supplies the exact sorted 19-path inventory, and must never
be regenerated for Phase 8C.

`BASE_COMMIT` records the clean pre-implementation HEAD. Before edits, evidence
proves that all 19 app-owned source blobs at `BASE_COMMIT` equal their blobs at
retained 8B terminal commit
`c4c446b1169733d8623f84bbda5e028c2e7fa365`. This lets the plan use
`BASE_COMMIT` as its operational rollback and test-list anchor without
redefining frozen 8B authority.

The single retained implementation checkpoint is `EXTRACTION_COMMIT`. For 17
relative paths other than `lib.rs` and `takeout/mod.rs`, completion evidence
requires:

```text
git rev-parse c4c446b1169733d8623f84bbda5e028c2e7fa365:src-tauri/src/telegram_impl/<path>
==
git rev-parse EXTRACTION_COMMIT:src-tauri/crates/extractum-telegram/src/<path>
```

The remaining two destination files may differ from their frozen source only
by this exact patch:

| File | Exact allowed substitutions |
| --- | --- |
| `lib.rs` | On both fixture re-exports, replace `#[cfg(test)]` with `#[cfg(feature = "app-test-support")]` and `pub(crate) use` with `pub use`. |
| `takeout/mod.rs` | On both fixture definitions, replace `#[cfg(test)]` with `#[cfg(feature = "app-test-support")]` and `pub(crate) fn` with `pub fn`. |

For each corrected path, evidence resolves the frozen source blob and retained
destination blob with `git rev-parse`, then runs:

```text
git diff --no-ext-diff --unified=0 <source-blob-id> <destination-blob-id>
```

Record both blob IDs and the complete diff. Excluding diff headers, the two
diffs together may contain only the eight line substitutions in the table:
function bodies, signatures, names, order, imports, and all other bytes remain
unchanged. No generated 8C hash artifact or comparison script is added.

Git rename output may supplement the evidence but is not the authority: the
old `telegram_impl/lib.rs` path remains present as the new app facade, so
ordinary rename detection is not sufficient for that file. The facade is new
app-owned content and is outside the 19-file source comparison.

## Target Package and Dependency Ownership

Phase 8C adds workspace member:

```text
src-tauri/crates/extractum-telegram
```

The workspace member count becomes seven. The package name is
`extractum-telegram`, its library crate is `extractum_telegram`, and it
inherits workspace edition and version.

`[workspace.dependencies]` remains the sole declaration authority for versions,
Git revisions, default-feature policy, and explicit features. Phase 8C does not
copy or alter those declarations. The new package owns direct
`{ workspace = true }` inheritance edges for:

- `grammers-client`;
- `grammers-session`;
- `grammers-mtsender`;
- `grammers-tl-types`.

It also owns direct inheritance edges for dependencies actually used by the
moved tree, including `chacha20poly1305` and `rand_core`. The app removes those
six direct roots.
`base64` and `secrecy` remain app dependencies because app-owned code uses
them independently; duplicate direct ownership is allowed only where metadata
and source search prove both packages consume the dependency.

The implementation plan must freeze the exact new package manifest from
current source imports before the extraction RED assertion. It may not infer a
new capability from transitive resolution. Cargo metadata must prove the
package-edge move while the retained Grammers feature baseline proves that the
unchanged workspace declarations resolve to the same effective features.

`Cargo.lock` changes only as Cargo requires for the new path package and moved
direct ownership. Versions, Git revisions, checksums, and effective Grammers
features must not drift.

## Physical Move and Compatibility Facade

Starting from recorded `BASE_COMMIT`, Phase 8C atomically:

1. creates the new package manifest;
2. moves all 19 prepared source paths from
   `src-tauri/src/telegram_impl/**` to
   `src-tauri/crates/extractum-telegram/src/**`;
3. applies the exact two-file fixture-boundary patch and no other moved-source
   edit;
4. creates a new private compatibility facade at the vacated
   `src-tauri/src/telegram_impl/lib.rs`;
5. adds the workspace member and normal/dev path dependency declarations;
6. removes moved direct dependencies from the app package;
7. updates the lockfile and the existing boundary contract once for the final
   physical layout.

The 17 unchanged destination files retain their frozen Git blobs. The other
two match the exact approved patch above. Any additional moved-tree edit stops
the extraction checkpoint.

The app retains:

```rust
#[path = "telegram_impl/lib.rs"]
mod telegram_impl;
```

All existing `crate::telegram_impl::...` production and test consumer paths
remain unchanged. The facade uses an explicit `pub(crate) use` allowlist and
never a glob. It contains no behavior, wrapper, adapter, owned type, or
dependency-specific logic.

## Public API

The production API is exactly the curated parent-design API already prepared
by 8B. Phase 8C widens no production item and changes no signature.

The only additional externally reachable names are the two functions under
`app-test-support`. They are not reachable in a normal dependency build and
are excluded from production API accounting. The feature-off crate check and
feature-on consumer tests exercise the two relevant Cargo configurations. The
standing source/API allowlist proves that fixture exports remain conditional;
no second production API accounting model is introduced.

The package does not publicly re-export `AppError`, `AppResult`, Grammers
types, internal modules, or raw protocol values.

## Test Ownership

The retained terminal inventory contains 736 unique `extractum` library tests:

- 71 staged future-owner tests move to `extractum-telegram` and are renamed
  only by their package/module prefix as required by the move;
- 665 tests remain in `extractum`, including the four app-owned tests that use
  the feature-gated fixtures;
- the workspace union remains exactly 736 unique logical identities.

No test is deleted, duplicated, ignored, or converted into a contract-only
assertion. The 8B identity artifact remains immutable and identifies the exact
71 future-owner tests. Before edits, evidence captures the exact 736-test
`extractum` library list at `BASE_COMMIT` from:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list
```

Expected final crate identities are the 71 staged identities with the
`telegram_impl::` prefix removed; expected final app identities are the
captured 736 identities minus those 71 staged identities.

`EXTRACTION_COMMIT` evidence captures the exact library test lists from both
packages using the same command shape with `-p extractum` and
`-p extractum-telegram`, then compares the sets, not only their counts, with
those expectations. The counts must still be 665 app / 71 crate / 736 union.
The captured lists belong in completion evidence; no persistent full-736
contract or generator is introduced.

## Execution Checkpoints

Phase 8C has exactly one retained implementation checkpoint.

### Final-boundary RED and atomic extraction GREEN

Inside the same uncommitted extraction change, first create the package edge,
move the frozen source, and create the app facade without yet applying the
fixture patch or enabling `app-test-support`. In this transient state, the app
manifest contains only the normal feature-free path dependency on
`extractum-telegram`; neither the app dev-dependency nor the producer
`app-test-support` feature declaration exists. Add those two declarations
together with the exact source patch after the RED. Compile the `extractum`
library test target with:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib --no-run
```

The exact expected RED is compiler error `E0432` at the facade re-export,
naming both unresolved imports
`extractum_telegram::takeout_attempt_fixture` and
`extractum_telegram::takeout_fallback_fixture`. No test binary is produced and
no test starts. This directly reproduces the cross-package seam and is the
single meaningful RED. The transient state is never committed. A Cargo
invocation that fails merely because an undeclared feature name was supplied
is not an accepted RED.

The exact oracle constrains that primary facade diagnostic, not the total
diagnostic count. Current rustc suppresses a downstream cascade; if another
supported compiler also emits `E0432` at the `takeout_import` import, record it
as expected secondary evidence, but do not require it or accept it as a
substitute for the primary facade error.

Do not run the TypeScript boundary contract in this transient state. It still
describes the app-owned topology and would add an unrelated structural failure.
Its first 8C execution occurs only after the contract is updated in the final
GREEN state; the later full verifier sees that same final state.

Then apply the exact two-file patch and final normal/dev dependency declarations.
Update the existing Telegram crate boundary contract once so its focused 8C
assertion describes only the final physical layout and final API boundary. The
contract must assert exactly two public fixture exports gated by
`feature = "app-test-support"`, reject either fixture when unconditional, and
allow exactly the two feature-gated private-module definitions plus their two
corresponding crate-root re-exports. Any additional gated definition or root
export fails. These four declarations represent exactly two externally
reachable fixture names. The contract is not rewritten through preparation and
extraction variants and does not embed the full 736-test identity list.

The contract derives the extracted state from exact package, path, and facade
preconditions and must pass before terminal roadmap/design statuses are
updated. Partial layouts fail closed; terminal status is not a prerequisite for
final GREEN.

Run all four fixture-consuming app tests GREEN with a non-empty selection, then
finish the lockfile and every tracked file intended for `EXTRACTION_COMMIT`.
On that final, still-uncommitted tree, run `npm.cmd run verify`, then run the
exact release build specified below on the same unchanged tree. Any tracked
checkpoint-file change after either gate invalidates both results and requires
both commands to be rerun. The checkpoint is GREEN only when focused package
and consumer tests, the 17 exact blob pairs and two exact diffs, exact test
identity sets, dependency ownership, feature closure, API allowlists, full
verification, and release build all pass. Only then retain the clean state as
`EXTRACTION_COMMIT`.

After `EXTRACTION_COMMIT`, run the live MCP and startup evidence once and record
completion in a docs-only terminal commit. That evidence commit records
`BASE_COMMIT`, `EXTRACTION_COMMIT`, content comparison, file disposition, test
identity sets, Cargo metadata, feature-off and feature-on evidence, full
verification, release build, MCP smoke, and startup smoke. It changes no Rust
source, manifest, or lockfile and is not an implementation checkpoint. It also
updates roadmap/design status to the exact terminal disposition required by
the parent design.

## Error and Behavior Compatibility

All fallible public operations continue to return
`extractum_core::error::AppResult<T>`. Error variants, sanitized messages,
Telegram status strings, Tauri command names and payloads, event names and
payloads, SQLite behavior, session encryption, cancellation, Takeout fallback
ordering, provenance writes, pagination, and source synchronization remain
unchanged.

The extraction adds no IPC command and no frontend permission. Telegram,
SQLite, compression, and session persistence remain backend-owned.

## Runtime and MCP Evidence

Terminal Phase 8C completion requires the parent-design startup and live MCP
evidence. Run the live MCP smoke once, after the clean `EXTRACTION_COMMIT` is
retained and before any self-managed startup smoke. It invokes the real webview
command:

```text
tg_get_account_statuses { accountIds: [] }
```

The exact result is `[]`. A plugin-side command executor is not a substitute
for webview-to-Tauri IPC evidence. The empty-ID call is intentionally bounded:
it proves command registration and the real webview-to-Tauri invoke path while
deterministic tests prove extracted Telegram behavior.

The release build is a deterministic pre-retention gate; its execution may
still require the repository-approved outside-sandbox protocol. The plan derives
`$hostTarget` from the `host:` line of `rustc -vV`, proves that
`CARGO_TARGET_DIR` and `CARGO_BUILD_TARGET` are unset, and runs exactly:

```powershell
npm.cmd run tauri -- build --no-bundle --target $hostTarget
```

This intentionally continues the retained 8B release protocol. The subsequent
startup smoke resolves only
`src-tauri/target/$hostTarget/release/extractum.exe`, records its hash and actual
PID path, and never substitutes `src-tauri/target/release/extractum.exe`.

MCP and startup are environment-sensitive post-commit terminal evidence. Their
failure blocks the terminal 8C/Phase 8 disposition but does not erase or by
itself invalidate the clean extraction commit. App ports must be free before
the self-managed startup smoke. The required order is final uncommitted
`npm.cmd run verify` -> release command above -> `EXTRACTION_COMMIT` -> live MCP
-> self-managed startup smoke -> docs-only terminal commit.

## Rust Verification Loops

Affected packages are `extractum-telegram` and its immediate consumer
`extractum`.

### Extraction loop

- compile-time RED: after the frozen move and package edge exist in the
  uncommitted working change but before the feature patch,
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib --no-run`
  fails with `E0432` for both fixture imports at the facade re-export; no test
  starts;
- narrow consumer GREEN: the four named app tests that consume the fixture seam
  run and pass with a non-empty selection;
- feature-off crate GREEN:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features`;
- producer-isolated focused check; selecting only the producer factually leaves
  the feature off, but this is not the canonical feature-off evidence:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- producer-isolated checkpoint, with the same evidence qualification:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- normal consumer GREEN:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --lib`;
- feature-on consumer focused check; resolver v2 activates the app
  dev-dependency for its development targets:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`;
- feature-on consumer checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.

A filtered GREEN run intended to execute tests is not evidence when it reports
zero tests; list names first when an exact identity is uncertain. The
compile-time RED above is different: it intentionally starts no tests and is
proved by the exact `E0432` diagnostic at the facade.

### End-of-slice gate

Run one final time on the complete, still-uncommitted checkpoint tree:

```powershell
npm.cmd run verify
```

The repository verifier already includes rustfmt plus workspace-wide Cargo
check and test gates, so those workspace commands are not duplicated in this
section. Its final `git diff HEAD --check` must inspect the complete extraction
diff, which is why verification precedes retention. The explicit feature-off
crate library check and feature-on consumer checkpoint remain the two primary
feature-specific invariants.

The package-level focused checks and checkpoints intentionally remain even
though the workspace verifier later recompiles their targets: the repository
workflow requires both directly affected packages and the immediate consumer
to have isolated package evidence before the full gate.

## Rollback

Before `EXTRACTION_COMMIT` is retained, any failed extraction returns the
workflow-owned working change to recorded `BASE_COMMIT` using the safe rollback
procedure named in the implementation plan. Do not partially copy files back,
reconstruct individual paths, or regenerate the immutable 8B artifact.

After `EXTRACTION_COMMIT` is retained, a failed environment-sensitive MCP or
startup check leaves that clean commit intact and Phase 8C pending. No terminal
status or docs-only evidence commit is written until the required evidence
passes.

## Non-Goals

Phase 8C does not:

- redesign Telegram behavior or public wire values;
- change frontend flows, commands, events, permissions, or SQLite schema;
- reorganize the 19-file module tree;
- rename consumer paths away from `crate::telegram_impl`;
- refactor or optimize moved code;
- add general test builders or public modules;
- add a second staging-hash script or generated 8C hash artifact;
- add a persistent test-ownership generator or count-only ownership contract;
- reopen retained 8A/8B checkpoints;
- upgrade dependencies or alter Grammers revision/features;
- resolve unrelated security debt.

## Acceptance Criteria

Phase 8C is complete only when:

1. retained 8B authority files and their starting hashes were verified before
   edits, the clean pre-implementation HEAD was recorded as `BASE_COMMIT`, and
   its Telegram source, manifest, and lockfile match frozen 8B identity;
2. the immutable 8B staging artifact is unchanged;
3. Phase 8C has one retained implementation checkpoint,
   `EXTRACTION_COMMIT`;
4. 17 destination blobs equal their frozen 8B sources, while `lib.rs` and
   `takeout/mod.rs` contain only the exact approved eight-line fixture patch;
5. compiling the `extractum` library test target in the uncommitted frozen-move
   state produced the recorded primary `E0432` diagnostic at the facade naming
   exactly the two fixture imports, without producing or running a test binary;
   any compiler-emitted downstream cascade was recorded but was not required;
   after the patch, all four consumers are GREEN and the once-updated final
   contract enforces exactly the two conditional fixture exports without
   depending on terminal status;
6. `extractum-telegram` is the seventh workspace member;
7. the app has no direct Grammers, `chacha20poly1305`, or `rand_core`
   dependency;
8. Cargo metadata proves exact package-edge ownership while the unchanged
   `[workspace.dependencies]` table and retained baseline prove revision and
   feature identity;
9. the exact feature-off `extractum-telegram --lib --no-default-features`
   check passes, without using `--all-targets` as feature-off evidence;
10. only `extractum-telegram` declares the non-default `app-test-support`
    feature; both producer definitions and re-exports use only
    `cfg(feature = "app-test-support")`, and the app dev-dependency enables it;
11. app package tests pass with the feature enabled by resolver v2, including
    the four fixture consumers;
12. the compatibility facade is explicit, `cfg(test)`-gated for the two
    fixtures, private to the app, and behavior-free;
13. existing app consumer paths and all production behavior are unchanged;
14. exact `cargo test -- --list` sets match the `BASE_COMMIT` list and
    71-owner map, with counts 665 app / 71 crate / 736 union;
15. focused package checks and checkpoints pass, and `npm.cmd run verify`
    passes on the final uncommitted extraction tree immediately before
    retention;
16. `npm.cmd run tauri -- build --no-bundle --target $hostTarget` passes on the
    same unchanged tree before `EXTRACTION_COMMIT` is created, and startup
    evidence resolves only
    `src-tauri/target/$hostTarget/release/extractum.exe`;
17. the one-time post-commit real-webview MCP smoke and subsequent startup
    smoke pass;
18. final documentation records the exact retained commits, hashes, commands,
    results, and terminal Phase 8 disposition.

## Implementation-Plan Requirements

Implementation requires a separate Phase 8C plan written with the project's
planning workflow. The plan must include the Rust verification loops
above; `BASE_COMMIT` capture and frozen-authority checks; the pre-edit exact
736-test list; the single compile-time `E0432` fixture-boundary RED inside the
uncommitted move; one atomic `EXTRACTION_COMMIT`;
path and dependency allowlists; 17 exact Git blob comparisons plus the two exact
approved diffs; one final-layout boundary-contract update whose physical-state
logic does not require terminal status; resolver-v2 feature-off/feature-on
evidence; exact test-list set comparisons; package loops; final uncommitted
`npm.cmd run verify`; the exact pre-retention release command and host-target
derivation; the exact host-qualified startup executable path; safe rollback
commands; and the one-time post-commit MCP/startup evidence template. It must
not run the boundary contract during the transient compile-time RED.

Approval of this specification alone does not start implementation.
