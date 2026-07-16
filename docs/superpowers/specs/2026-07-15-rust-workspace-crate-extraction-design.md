# Rust Workspace and Incremental Crate Extraction Design

## Status

Approved for implementation planning on 2026-07-15 after iterative written-spec
review.

## Context

The Extractum Rust backend is currently one `extractum` package rooted at
`src-tauri/Cargo.toml`. Application commands, Tauri state, storage helpers,
shared models, process integration, and large domains all compile as one
library crate.

The existing Cargo timing evidence shows that the root `extractum` unit can
dominate an incremental or cold check. A source change inside the package may
therefore require Rust to revisit a much larger compilation unit than the
changed behavior logically owns. Splitting stable code and selected domains
into crates may improve the normal edit-check-test loop by allowing Cargo to
reuse unchanged crate artifacts.

The current source directories are not automatically safe crate boundaries.
Several domains depend on internal modules of other domains, Tauri handles,
managed state, shared SQL fixtures, or application-owned process lifecycle.
The design therefore combines one deliberately mechanical core extraction
with later just-in-time preparation of one domain at a time. It does not
perform a repository-wide architectural rewrite before the first extraction.

## Goal

Establish a working Cargo workspace and a small reusable `extractum-core`
crate, then define a measured, incremental path for extracting additional
Rust domains without changing application behavior.

The design must:

- preserve the existing Tauri development and production workflows;
- retain the current Cargo development profiles;
- prevent Tauri and Telegram-specific dependencies from leaking into the
  foundational core;
- avoid a big-bang commands/service/store refactor across all domains;
- make every cross-crate public API expansion explicit;
- use predeclared timing criteria to decide whether further domain splitting
  is justified as a performance optimization.

## Non-Goals

- No immediate extraction of every large Rust domain.
- No global conversion of `crate::...` imports to `super::...` imports.
- No up-front commands/service/store split across all modules.
- No domain-specific error hierarchy before a concrete consumer requires it.
- No migration of Tauri commands, managed state, plugin registration, window
  events, or application startup into `extractum-core`.
- No move of the Tauri-aware database-pool adapter into `extractum-core`.
- No mandatory `cargo-public-api` installation or public-API snapshot tool.
- No claim that workspace creation alone must improve production build time.
- No portable timing threshold across different machines.

## Current Dependency Evidence

The first foundational candidates have no dependencies on other application
domains:

- `error.rs` depends on `serde` and the standard library;
- `time.rs` depends on the external `time` crate and the standard library;
- `compression.rs` depends on `zstd` and the standard library.

Other apparent foundation modules have additional constraints:

- `sql_helpers.rs` depends on `sqlx`;
- `tx.rs` depends on `sqlx` and the shared error API;
- `db.rs` depends on `tauri::AppHandle`, `tauri::Manager`, and
  `tauri-plugin-sql`, so it remains an application adapter;
- `media.rs` mixes pure metadata behavior with `grammers-client` Telegram
  media extraction and must be split before its pure portion can move.

The four grammers crates are direct git dependencies pinned to a Codeberg
revision. Moving the current `media.rs` wholesale into core would make the
core inherit those heavyweight git dependencies. Separating pure media
metadata keeps core independent of grammers and lets a future pure
`notebooklm_export` crate compile and test without a transitive Telegram
client dependency.

The apparent `sources`/`youtube` cycle is asymmetric. `sources` consumes only
the pure `youtube::dto` types, while `youtube` consumes behavior owned by
`sources`. The cycle does not need an abstract neutral service layer. Before
the two domains become separate crates, the shared YouTube DTO module can move
down into `sources`, producing a one-way `youtube -> sources` dependency.

## Selected Architecture

### Workspace Root

`src-tauri/Cargo.toml` remains both:

- the `extractum` application package manifest;
- the Cargo workspace root;
- the owner of all `[profile.*]` sections;
- the owner of `[workspace.dependencies]`.

The initial workspace shape is:

```text
src-tauri/
  Cargo.toml                 # package + workspace root
  src/                       # Tauri application crate
  crates/
    extractum-core/
      Cargo.toml
      src/
        lib.rs
        error.rs
        time.rs
        compression.rs
```

The workspace includes the root package explicitly and uses resolver version
2. Shared dependency versions are declared in `[workspace.dependencies]`, and
member crates opt into them with `workspace = true`.

The existing development profile remains effective because the current
manifest remains the workspace root:

```toml
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false
```

Member manifests must not contain profile sections. Cargo must not emit an
ignored-profile warning. If a future change introduces a repository-root
workspace manifest above `src-tauri`, moving the profiles and workspace
dependencies is a separate, explicit migration.

### Minimal Core

The first `extractum-core` slice contains only:

- `error`;
- `time`;
- `compression`.

This is primarily an infrastructure and boundary-validation slice. It is not
required by itself to satisfy the later domain-extraction performance gate.

The application crate preserves its existing internal paths through curated
re-exports. Existing consumers may continue to use:

```rust
crate::error::AppResult
crate::time::now_secs
crate::compression::compress_text
```

Code moved into a separate domain crate uses the real dependency path, such
as `extractum_core::error::AppResult`. This keeps the first diff mechanical
and defers import changes to the domain whose boundary is being established.

### Public API Policy

Items that cross a crate boundary must become `pub`; `pub(crate)` cannot span
packages. Every visibility expansion is reviewed explicitly rather than
performed as an unconstrained textual replacement.

The expected foundational surface at design time is:

| Module | Cross-crate items |
| --- | ---: |
| `error` | 2 |
| `time` | 3 |
| `compression` | 4 |
| `sql_helpers` when triggered | 1 |
| `tx` when triggered | 7 |

The exact names and current consumers are rechecked during implementation
because the source can evolve after this design. Test-only helpers are not
exported merely to preserve old unit-test placement.

The API contract is the curated `extractum-core/src/lib.rs` plus compilation
of real consumers:

- modules and re-exports are explicit;
- glob re-exports are forbidden;
- only demonstrated consumers justify public items;
- no external public-API snapshot tool is required for this workspace size.

### Tauri Application Shell

The root `extractum` crate continues to own:

- `#[tauri::command]` adapters;
- `AppHandle`, `State`, `Manager`, and `Emitter` integration;
- plugin registration and application startup;
- managed state and process lifecycle;
- the Tauri-plugin database pool lookup in `db.rs`;
- integration tests that intentionally span multiple domains.

When preparing a particular domain, a Tauri command may resolve application
dependencies and delegate to a pool-level or service-level function. That
pattern is introduced only where the selected extraction requires it, not
globally in advance.

## Media Metadata Boundary

Before extracting the pure part of `notebooklm_export`, split `media.rs` into
two responsibilities.

The core-owned `media_metadata` module contains:

- `ItemMediaMetadata`;
- the seven metadata fields needed by existing structure literals;
- `encode_media_metadata`;
- `decode_media_metadata`;
- `media_label`;
- only genuinely shared, pure constants required by those operations.

The application-owned media adapter retains:

- `grammers_client::media::Media` and Telegram TL handling;
- extraction from Telegram media values;
- `ExtractedMediaPayload`;
- `ExtractedItemPayload`;
- `DocumentSignals` and other grammers-specific behavior.

The application `media` module re-exports the core metadata API so current
`crate::media::...` users do not require a mass import rewrite. When
`notebooklm_export` later becomes another crate, it imports the metadata API
directly from `extractum-core`.

The media split is not only a NotebookLM preparation. Its verification covers
all current production consumers, including `notebooklm_export`, `sources`,
and `takeout_import`, followed by the full Rust suite and Tauri build.

The current `media.rs` test module is self-contained. Current
`notebooklm_export` production code consumes only `ItemMediaMetadata` and
`decode_media_metadata`; no cross-crate test-util feature is required by this
boundary.

## Just-in-Time Domain Preparation

No domain-wide preparation happens before a domain is selected for extraction.
For one candidate at a time:

1. Recompute its incoming and outgoing dependency map.
2. Identify the pure or storage-level portion that can form a coherent crate.
3. Separate Tauri commands, state lookup, or process adapters only where the
   candidate requires it.
4. Replace access to another domain's internals with the smallest deliberate
   facade needed by this candidate.
5. Review test fixtures, `test_support`, dev-dependencies, and integration-test
   ownership before adding any cross-crate test dependency.
6. Move the selected code and its local unit tests.
7. Verify correctness and measure the representative incremental loop.
8. Apply the stop/go decision before selecting the next extraction.

Changing `crate::...` to `super::...` is optional cleanup during a touched
slice, not a prerequisite. The compiler and a path-scoped dependency search
define the actual boundary at extraction time.

The first domain candidate is the pure portion of `notebooklm_export`, such as
models, chunking, filename handling, links, mapping, and rendering. The exact
set remains conditional on the media split and the fresh dependency map.
Database lookup, Tauri state, and dependencies on application-owned source
behavior remain in the app shell until a clean interface is established.

## Deferred Backlog and Triggers

The following work remains explicit even though it is not part of the minimal
core commit:

### SQL Helpers and Transactions

Move `sql_helpers` and `tx` in a separate commit after the minimal workspace is
stable and before the first extracted domain that needs those helpers. Recheck
their actual APIs and dependencies at that point. This commit deliberately
adds the shared `sqlx` dependency to core.

### YouTube DTO Placement

Immediately before `sources` and `youtube` are separated into distinct crates,
move the pure `youtube::dto` types into a lower `sources` module, for example
`sources::youtube_types`. Do not perform this churn merely to remove a legal
same-crate module cycle earlier.

### Test Support

Before every domain extraction, inventory production dependencies separately
from test-only dependencies. Keep cross-domain integration fixtures in the
root application crate unless a small, stable fixture crate is independently
justified. Do not create cyclic dev-dependencies between domain crates.

## Measurement Protocol

### Baseline

Record the baseline before workspace changes. Capture the commit, toolchain,
Cargo profile, target path, relevant environment state, and two exact source
probes:

- a domain probe in the pure portion of `notebooklm_export`, with
  `notebooklm_export/chunker.rs` as the default candidate;
- an application-shell probe in code that remains in the root package, with
  `src-tauri/src/lib.rs` as the default candidate.

Freeze the selected logical files and inert edit before collecting baseline
data. If source drift makes a default candidate unsuitable, record the
replacement before any workspace or extraction change; do not choose a new
probe after seeing post-change results.

Measure:

- five warmed no-op `cargo check` runs;
- five warmed incremental `cargo check` runs after the identical inert domain
  edit;
- five warmed incremental `cargo check` runs after the identical inert
  application-shell edit;
- three `cargo test --no-run` runs;
- Rust test execution separately from compilation;
- two production `tauri build` runs;
- a `cargo build --timings` report that records the root `extractum` unit.

Temporary probe edits are restored byte-for-byte, and the worktree is checked
after restoration. Cold-cache evidence is labeled separately and is not mixed
with warmed medians.

### Re-measurement

Repeat both probes after the minimal core extraction and after each domain
extraction. When the selected domain file moves into `crates/.../src/...`, the
domain probe follows that same logical file and applies the same inert edit at
its new path. The shell probe remains in the root application package. The
same source behavior, profile, target directory, and machine conditions are
used as far as practical.

For fast metrics, compare five-run medians. For production build, two runs per
side are sufficient; run a third only when the first two disagree enough to
change the decision. Production-build correctness is a separate mandatory
gate and does not depend on timing precision.

### Domain Stop/Go Gate

The minimal core extraction is enabling infrastructure and does not have to
pass this performance threshold.

A domain extraction qualifies as a performance success when the representative
incremental `cargo check` median improves by both:

- at least 25%; and
- at least 2 seconds.

When the baseline is below 8 seconds, the absolute two-second threshold is not
applied because it would make a useful percentage result structurally
unreachable; the report must call out the smaller absolute scale.

A no-op regression is recorded only when it exceeds both 5% and 0.5 seconds.
This avoids treating normal sub-second Windows, filesystem, or Defender noise
as a failure. The application-shell incremental probe fails its regression
gate only when it becomes slower by both more than 5% and more than 1 second.
The 25%/two-second improvement gate applies only to the domain probe. Longer
commands use an explicit absolute tolerance appropriate to their duration;
production build and full test behavior must not show a material regression.

If a domain does not meet the performance gate, do not automatically continue
splitting other domains for performance. The extracted boundary may be retained
only when it has separately documented architectural value and no correctness
or workflow regression.

## Verification Strategy

The minimal workspace/core slice verifies:

- `cargo fmt --check` for the complete workspace;
- `cargo check --workspace --all-targets`;
- `cargo test --workspace --all-targets` for complete workspace tests;
- unchanged or increased executed-test counts relative to the pre-workspace
  inventory, with every moved `error`, `time`, and `compression` test present;
- existing focused Rust tests;
- existing frontend and source-contract tests;
- the repository verification command;
- normal Tauri development startup;
- production Tauri build and startup smoke;
- absence of Cargo warnings about ignored profiles;
- effective use of the canonical shared `src-tauri/target` directory;
- continued effectiveness of the existing dev profile;
- no accidental grammers dependency in `extractum-core`.

Before accepting the workspace, audit every active repository invocation of
`cargo test` and `cargo check`, including `scripts/verify.mjs`, `package.json`,
CI configuration, developer guidance, and other scripts. Canonical complete
gates must use workspace-aware commands. Focused commands may use `-p` or an
equivalent explicit package selection, but must not accidentally present a
root-package-only run as complete verification.

Record the pre-workspace Rust test inventory before moving tests. After the
move, enumerate tests for all workspace members and compare the total executed
count and the identities of the moved core tests. A successful exit code alone
is insufficient because a root-package-only `cargo test` can pass while
silently omitting member-crate tests.

The canonical target assertion means that Cargo metadata resolves the workspace
target directory to the existing `src-tauri/target`, no command introduces a
slice-specific `CARGO_TARGET_DIR`, and member crates share that directory. The
workspace must not create a second target tree elsewhere in the repository.

The media slice additionally verifies focused tests for:

- media metadata encoding and decoding;
- `notebooklm_export`;
- `sources`;
- `takeout_import`;
- the full Rust workspace.

Every later domain extraction records its dependency map, public surface,
focused tests, complete workspace tests, Tauri build evidence, timing samples,
and stop/go decision.

## Failure Handling

- If Cargo ignores a profile, stop and correct workspace-root ownership before
  collecting timing or correctness evidence.
- If Tauri CLI, build scripts, capabilities, bundled binaries, or production
  packaging behave differently, restore the previous package shape and treat
  the workspace infrastructure as incomplete.
- If a core module requires an unexpected application or Tauri dependency,
  keep that responsibility in the app crate rather than pulling the dependency
  into core without design review.
- If media metadata cannot be separated without grammers-specific types in its
  API, stop the NotebookLM extraction and redesign that boundary.
- If a domain extraction introduces cyclic normal or dev-dependencies, keep
  the integration behavior in the root crate and narrow the candidate.
- Timing failures do not justify weakening correctness gates or changing test
  inventory.

## Risks and Mitigations

### More crates can increase clean-build overhead

Crate boundaries improve reuse but add crate metadata and linking work. Measure
incremental and full workflows separately; do not infer one from the other.

### Core can become a dumping ground

Only stable, demonstrated shared behavior enters core. Domain DTOs remain with
the lowest owning domain when possible. Deferred work is trigger-based rather
than automatically accumulated in core.

### Public APIs can expand accidentally

Cross-crate visibility is reviewed item by item, `lib.rs` is curated, glob
re-exports are forbidden, and tests compile against real consumer paths.

### Test dependencies can recreate domain coupling

Inventory `test_support` before every extraction and keep multi-domain tests in
the application crate by default.

### Performance measurements can be noisy

Use warmed repeated runs, medians, absolute and relative thresholds, identical
probes, and separately labeled cold evidence. A structural refactor is not
declared successful from one timing sample.

## Acceptance Criteria

1. `src-tauri/Cargo.toml` is a functioning package/workspace root and retains
   the current effective development profiles.
2. `extractum-core` initially contains only `error`, `time`, and `compression`,
   with no Tauri, plugin, grammers, or application-domain dependency.
3. Existing `crate::error`, `crate::time`, and `crate::compression` application
   paths remain compatible through curated re-exports.
4. Every cross-crate `pub` item is enumerated and justified; glob re-exports
   are absent.
5. The canonical Cargo, Tauri development, production build, and repository
   verification workflows pass with unchanged behavior.
6. Baseline and post-change timing evidence use the same documented domain and
   application-shell probes and distinguish no-op, incremental,
   test-execution, and production-build costs.
7. `media_metadata` is separated from grammers-specific extraction before the
   pure NotebookLM domain is extracted, and all known consumers pass.
8. Domain preparation is just-in-time and limited to one selected candidate.
9. Every domain extraction applies the predeclared stop/go rule before further
   performance-motivated splitting.
10. The deferred `sql_helpers`/`tx`, YouTube DTO, and test-support work remains
    documented with explicit trigger conditions.
11. Every canonical complete Cargo gate is workspace-aware, and before/after
    inventory evidence proves that moved member-crate tests are still executed.
