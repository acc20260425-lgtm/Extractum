# Telegram Phase 8C Extraction Design

**Status:** Draft for owner review; implementation not authorized
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

This design resolves that issue before the mechanical move, derives a new
hash authority for the corrected tree, and keeps the actual extraction free of
source adaptation.

## Decision

Use a two-step, forward-only Phase 8C:

1. make one bounded preparation correction in the still app-owned staged tree;
2. mechanically extract the resulting prepared tree into a new workspace
   package.

The correction introduces a non-default `app-test-support` feature and exposes
exactly two fixture functions when that feature is enabled. The production
dependency does not enable the feature. The application dev-dependency does,
and the private app facade re-exports the two functions only under
`#[cfg(test)]`.

No production behavior, public wire contract, persistence format, source
ownership, or normal-build API is changed by the correction.

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

Rejected. The app needs the constructors only in tests. Permanently expanding
the production API would weaken the curated crate boundary.

## Authority and Precedence

Until explicit owner approval, this document is a draft and the parent design
remains the active 8C authority. Approval authorizes design only; it does not
authorize implementation.

After explicit approval, this document becomes the normative authority for
Phase 8C where it explicitly differs from the parent design. It supersedes
only these parent clauses:

1. 8C may perform a bounded two-file staged-tree correction before the move;
2. the feature-enabled crate surface may contain exactly the two test fixture
   functions specified below;
3. final move equality is measured against a new derived 8C prepared-tree
   artifact, while the immutable 8B artifact remains retained evidence;
4. the app may declare the same path package once in normal dependencies and
   once in dev-dependencies solely to enable `app-test-support` for app tests;
5. after the preparation correction, the two corrected source files move
   byte-for-byte with the other 17 files.

Every parent requirement not named above remains in force. In particular,
this document does not reopen 8A or 8B, expand the production API, alter the
19-path ownership map, or authorize application consumer rewrites.

## Frozen Starting Identity

Phase 8C begins from retained 8B terminal commit
`c4c446b1169733d8623f84bbda5e028c2e7fa365`.

The implementation plan must verify these files before its first source edit:

| Authority | SHA-256 |
| --- | --- |
| `src/lib/telegram-grammers-feature-baseline.json` | `774e2b979d5cdc8185a85488c965548cf09cdf1ef0ab4b9ecad58246283cf5b3` |
| `src/lib/telegram-8b-test-identities.json` | `507f09f4fab76bee4360185eca3fbef17fb1563e784f7654bebb430cf7f08a95` |
| `src/lib/telegram-8b-symbol-map.json` | `f978e80cd58303fd9cd6402ba17deef1df817d22fdbf821830e9e0a5968c13b3` |
| `src/lib/telegram-8b-staging-sha256.json` | `12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9` |
| `src-tauri/Cargo.toml` | `ee323d7b613573918d4ad3777b238bc7e107d049588ddcfa0959dacfd1e2cf69` |
| `src-tauri/Cargo.lock` | `720e38ea632d7b932b2a23d1481528845ec9304376035b1c851c546ea402e43c` |

Unexpected drift stops the slice. It is not normalized inside Phase 8C.

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

When either the package's own tests or that feature are enabled, the crate root
may expose exactly these two free functions:

```rust
#[cfg(any(test, feature = "app-test-support"))]
pub fn takeout_attempt_fixture(
    home_dc_id: i32,
    export_dc_id: i32,
) -> TakeoutAttempt;

#[cfg(any(test, feature = "app-test-support"))]
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

During the preparation correction, the still app-owned package temporarily
declares `app-test-support = []` so Rust's checked-cfg recognizes the feature.
The two definitions and root exports use
`#[cfg(any(test, feature = "app-test-support"))]`; their visibility becomes
`pub`. No app test or production call site changes.

After extraction:

```toml
[dependencies]
extractum-telegram = { path = "crates/extractum-telegram" }

[dev-dependencies]
extractum-telegram = {
  path = "crates/extractum-telegram",
  features = ["app-test-support"],
}
```

The temporary app-owned `app-test-support` feature declaration is removed at
this point; the feature belongs only to `extractum-telegram`.

The private compatibility facade contains the normal explicit production
allowlist plus only this test-gated addition:

```rust
#[cfg(test)]
pub(crate) use extractum_telegram::{
    takeout_attempt_fixture,
    takeout_fallback_fixture,
};
```

Normal `cargo check -p extractum --lib` must compile the dependency without
the feature. App tests enable it through the dev-dependency. A contract must
prove that the normal dependency has no feature, the dev-dependency enables
exactly `app-test-support`, the app retains no same-named package feature, and
no third feature-gated public item exists.

## Prepared-Tree Hash Authority

The immutable 8B staging artifact remains unchanged. It continues to prove
the retained 8B result and must never be regenerated for Phase 8C.

The preparation correction creates:

- `src/lib/telegram-8c-prepared-sha256.json`;
- `scripts/telegram-8c-extraction-sha256.mjs`.

The new artifact has the same 19 sorted relative paths as the 8B artifact. Its
records differ from 8B for exactly:

- `lib.rs`;
- `takeout/mod.rs`.

The other 17 path/hash records must be byte-identical to the retained 8B
artifact. The script supports `--write` only while the source root is
`src-tauri/src/telegram_impl`, and `--check` against either that prepared root
or `src-tauri/crates/extractum-telegram/src` as selected by an explicit
argument. It rejects missing or extra paths, order drift, root drift, malformed
hashes, and a changed two-file delta.

The final extracted destination must match all 19 prepared records exactly.
The newly created app facade is outside this comparison.

## Target Package and Dependency Ownership

Phase 8C adds workspace member:

```text
src-tauri/crates/extractum-telegram
```

The workspace member count becomes seven. The package name is
`extractum-telegram`, its library crate is `extractum_telegram`, and it
inherits workspace edition and version.

The new package owns the four exact Grammers declarations and their retained
revision/feature policy:

- `grammers-client`;
- `grammers-session`;
- `grammers-mtsender`;
- `grammers-tl-types`.

It also owns direct dependencies actually used by the moved tree, including
`chacha20poly1305` and `rand_core`. The app removes those six direct roots.
`base64` and `secrecy` remain app dependencies because app-owned code uses
them independently; duplicate direct ownership is allowed only where metadata
and source search prove both packages consume the dependency.

The implementation plan must freeze the exact new package manifest from
current source imports before the extraction RED contract. It may not infer a
new capability from transitive resolution. The retained Grammers feature
baseline must remain GREEN after ownership moves.

`Cargo.lock` changes only as Cargo requires for the new path package and moved
direct ownership. Versions, Git revisions, checksums, and effective Grammers
features must not drift.

## Physical Move and Compatibility Facade

After the preparation checkpoint is retained, Phase 8C atomically:

1. creates the new package manifest;
2. moves all 19 prepared source paths from
   `src-tauri/src/telegram_impl/**` to
   `src-tauri/crates/extractum-telegram/src/**`;
3. creates a new private compatibility facade at the vacated
   `src-tauri/src/telegram_impl/lib.rs`;
4. adds the workspace member and normal/dev path dependency declarations;
5. removes moved direct dependencies from the app package;
6. updates the lockfile and boundary contracts.

Every destination source file, including corrected `lib.rs` and
`takeout/mod.rs`, equals the prepared artifact byte-for-byte. There is no
source fix during or after the move. A required moved-tree edit means the
preparation is incomplete and the extraction checkpoint stops.

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
are excluded from production API accounting. Contract checks must evaluate
both normal and feature-enabled surfaces.

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
assertion. The 8B identity artifact remains immutable. An 8C ownership
contract derives the expected package split from its 71 staged identities and
proves the union against the retained 736-test set.

## Execution Checkpoints

### Checkpoint 0: bounded test-support preparation

The first implementation checkpoint changes only:

- `src-tauri/src/telegram_impl/lib.rs`;
- `src-tauri/src/telegram_impl/takeout/mod.rs`;
- `src-tauri/Cargo.toml` for temporary checked-cfg feature recognition;
- the new 8C hash script/artifact and their focused contract tests;
- documentation/evidence required by the plan.

It must prove all 736 app library tests remain GREEN and that the new artifact
differs from the immutable 8B artifact at exactly two source paths. The
roadmap status remains `8B preparation retained; 8C pending` because no crate
or dependency edge exists yet.

### Checkpoint 1: final boundary RED

Before moving files, contracts must fail for the intended missing final state:

- no `extractum-telegram` workspace member;
- no new package manifest;
- staged files still app-owned;
- Grammers roots still app-owned;
- compatibility facade still points to local modules.

Every RED must be a real executed failure with the exact expected assertion;
syntax errors, zero-test filters, or unrelated failures do not qualify.

### Checkpoint 2: atomic extraction GREEN

Perform the physical move, manifest/workspace/dependency transfer, facade
creation, lockfile refresh, and final contract updates as one extraction
checkpoint. Do not edit moved source content.

The checkpoint is GREEN only when package tests, app tests, hash comparison,
dependency ownership, feature closure, API allowlists, and workspace checks
all pass.

### Checkpoint 3: completion evidence

Record final hashes, exact file disposition, test ownership, Cargo metadata,
normal and feature-enabled API evidence, release build, startup smoke, MCP
smoke, and full repository verification. Then update roadmap/design status to
the exact terminal disposition required by the parent design.

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

Completion requires the parent-design startup and live MCP evidence. The MCP
smoke invokes the real webview command:

```text
tg_get_account_statuses { accountIds: [] }
```

The exact result is `[]`. A plugin-side command executor is not a substitute
for webview-to-Tauri IPC evidence.

Environment-sensitive full verification, release build, startup, and MCP
smoke use the repository-approved outside-sandbox protocol. Ports must be free
before self-managed smoke checks, and live MCP smoke runs first.

## Rust Verification Loops

Affected packages are initially `extractum`, then both `extractum-telegram`
and its immediate consumer `extractum`.

### Preparation loop

- RED: focused contract proving the two helpers are not feature-addressable;
- GREEN: exact four app tests plus the test-support/hash contract;
- focused check:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`;
- package checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.

### Extraction loop

- RED: exact final package-boundary assertions described in Checkpoint 1;
- crate GREEN:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- crate checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-telegram --all-targets`;
- consumer GREEN:
  `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`;
- consumer checkpoint:
  `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`.

A filtered run reporting zero tests is not evidence. List test names first when
an exact identity is uncertain.

### End-of-slice gates

Run all of:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

The final package must also pass normal and `app-test-support` feature checks
separately so accidental production feature enablement cannot hide.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-telegram --lib --no-default-features --features app-test-support
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --lib
```

## Rollback

Checkpoint 0 is independently retained only if its app package checkpoint and
prepared-hash proof are GREEN. If extraction fails later, return to that clean
checkpoint; do not partially copy files back or regenerate the 8B artifact.

Checkpoint 2 is atomic. A failed move is not retained. A rollback restores the
single app-owned prepared tree, removes the unfinished workspace/package edge,
and restores the pre-extraction lockfile without changing Checkpoint 0 source
content.

## Non-Goals

Phase 8C does not:

- redesign Telegram behavior or public wire values;
- change frontend flows, commands, events, permissions, or SQLite schema;
- reorganize the 19-file module tree;
- rename consumer paths away from `crate::telegram_impl`;
- refactor or optimize moved code;
- add general test builders or public modules;
- reopen retained 8A/8B checkpoints;
- upgrade dependencies or alter Grammers revision/features;
- resolve unrelated security debt.

## Acceptance Criteria

Phase 8C is complete only when:

1. retained 8B authority files and their starting hashes were verified before
   edits;
2. the immutable 8B staging artifact is unchanged;
3. the prepared artifact has the same 19 paths and exactly the authorized
   two-file hash delta;
4. all 19 destination files equal the prepared artifact byte-for-byte;
5. `extractum-telegram` is the seventh workspace member;
6. the app has no direct Grammers, `chacha20poly1305`, or `rand_core`
   dependency;
7. Cargo metadata proves exact dependency ownership, revision, and features;
8. the production dependency does not enable `app-test-support`;
9. the dev-dependency enables exactly `app-test-support` and the feature
   exposes exactly two approved functions;
10. the compatibility facade is explicit, private to the app, and contains no
    behavior;
11. existing app consumer paths and all production behavior are unchanged;
12. test ownership is exactly 665 app / 71 crate / 736 union;
13. focused package checks and all end-of-slice gates pass;
14. release, startup, and real webview MCP evidence pass;
15. final documentation records the exact retained commits, hashes, commands,
    results, and terminal Phase 8 disposition.

## Implementation-Plan Requirements

After owner approval, write a separate Phase 8C implementation plan using the
project's planning workflow. The plan must include the Rust verification loops
above, exact RED/GREEN assertions, checkpoint commits, path and dependency
allowlists, hash-script schema, API checks for both feature states, test
identity commands, rollback commands, and final evidence template.

Approval of this specification alone does not start implementation.
