# NotebookLM Render Crate Boundary Design

**Status:** Approved in conversation; awaiting written-spec review
**Date:** 2026-07-17

## Summary

Extract the pure NotebookLM rendering pipeline from the Tauri application into
a second domain crate, `extractum-notebooklm-render`, only if a controlled
before/after experiment demonstrates a material improvement in incremental
`cargo check` time.

The candidate boundary contains seven existing modules:

- `model`;
- `filename`;
- `links`;
- `media`;
- `renderer`;
- `glossary`;
- `chunker`.

The movement is mechanical. Serialized request/result shapes, Markdown output,
filenames, validation behavior, progress events, database reads, and filesystem
orchestration must not change.

## Context and Goal

The Rust workspace currently contains the application package and
`extractum-core`. The previous core slices established a shared target
directory, workspace-wide gates, explicit dependency boundaries, and a process
for preserving the complete test inventory when module paths change.

`src-tauri/src/notebooklm_export` currently contains ten Rust files and about
5,900 lines. Its responsibilities are mixed:

- pure filtering, rendering, grouping, chunking, and filename logic;
- SQL loading and row mapping;
- Tauri command/progress orchestration;
- filesystem and manifest lifecycle.

The goal of this slice is not merely to reduce the size of the application
module. The goal is to prove that placing the pure rendering pipeline behind a
crate boundary produces a meaningful improvement in the daily incremental
compile loop without slowing edits to the remaining application shell.

## Investigation Evidence

The proposed pure subgraph is approximately 1,593 lines:

| Module | Lines | Tests | Direct role |
| --- | ---: | ---: | --- |
| `chunker.rs` | 527 | 7 | filtering, grouping, splitting, chunks |
| `filename.rs` | 170 | 5 | safe path components and child paths |
| `glossary.rs` | 132 | 1 | participant aggregation and glossary rendering |
| `links.rs` | 34 | 1 | URL detection |
| `media.rs` | 95 | 2 | textual media placeholders |
| `model.rs` | 151 | 0 | request/result and rendering data models |
| `renderer.rs` | 484 | 6 | message/document Markdown rendering |
| **Total** | **1,593** | **22** | |

These seven modules use only the standard library plus `serde`, `serde_json`,
`time`, and the already-extracted media metadata API from `extractum-core`.
They do not use Tauri, SQLx, grammers, database pools, readiness state, or
application process state.

The remaining modules have application-facing dependencies:

- `message_mapping.rs` uses SQLx row types, decompression, application errors,
  media decoding, and pure render helpers;
- `query.rs` uses SQLx, readiness selection, application errors, row mapping,
  and pure models;
- `mod.rs` owns the Tauri command, progress events, DB admission, filesystem
  writes, manifests, and task orchestration.

## Selected Architecture

Create this workspace member:

```text
src-tauri/crates/extractum-notebooklm-render/
  Cargo.toml
  src/
    lib.rs
    model.rs
    filename.rs
    links.rs
    media.rs
    renderer.rs
    glossary.rs
    chunker.rs
```

The dependency direction is one-way:

```text
extractum
  ├── extractum-notebooklm-render
  └── extractum-core

extractum-notebooklm-render
  └── extractum-core
```

The new crate has exactly four direct dependency roots:

- `extractum-core`;
- `serde`;
- `serde_json`;
- `time`.

It must not depend on Tauri, SQLx, grammers, Tokio, filesystem-specific helper
crates, or the application package.

The root workspace manifest should define path dependencies for both local
crates under `[workspace.dependencies]`. The application and render-crate
manifests consume them with `workspace = true`. Existing external dependency
versions must not change.

## Application Boundary

The application retains:

- `export_source_to_notebooklm` and its Tauri command signature;
- progress-event DTOs and event emission;
- `query` and `message_mapping`;
- source identity readiness checks;
- SQL and archive-read-model selection;
- `spawn_blocking` orchestration;
- output directory, manifest, symlink, and filesystem safety logic;
- conversion of DB, filesystem, task, and validation failures into `AppError`.

The new crate receives ready Rust values and performs pure transformations:

```text
SQLite/archive rows
  -> application message_mapping
  -> render crate filter/render/chunk/glossary pipeline
  -> application filesystem and manifest writer
```

The render crate performs no I/O and emits no Tauri events.

## API Surface and Compatibility Facades

`extractum-notebooklm-render/src/lib.rs` uses explicit module declarations and
explicit re-exports. Glob re-exports are forbidden.

Only items required across the crate boundary become `pub`. Fields become
public only when the application constructs a value with a struct literal or
reads the returned value. Internal helper functions and implementation-only
types remain private to their module or crate.

The current request/result types move with `model.rs`, including:

- `NotebookLmExportScope`;
- `NotebookLmExportRequest`;
- `NotebookLmExportConfig`;
- `NotebookLmExportResult`;
- `NotebookLmExportFile`;
- the source, message, participant, rendered-block, topic, and chunk models.

Their derives, field order, serde attributes, defaults, and serialized names
must remain byte-for-byte equivalent.

The application keeps private inline compatibility modules under
`notebooklm_export` for the seven old module paths. Each compatibility module
uses an explicit `pub(crate) use extractum_notebooklm_render::<module>::{...}`
list. This preserves existing paths used by `query.rs`, `message_mapping.rs`,
`mod.rs`, and their tests without mass import churn. The old seven `.rs` files
are removed after their contents and tests move; compatibility modules contain
no copied implementation.

The public Tauri command signature and frontend wire contract remain
unchanged.

## Error Handling

The render crate introduces no new error enum.

- Safe path helpers retain their current `Option` contracts.
- Rendering, aggregation, filtering, and chunking retain their current value
  return types.
- Database, readiness, decompression, filesystem, join, and validation errors
  remain in the application and continue to use `AppError`.

No application error strings, classifications, or failure ordering change as
part of this slice.

## Performance Measurement Protocol

### Environment

Record before measurement:

- commit hash and dirty-state check;
- `rustc -Vv` and `cargo -V`;
- CPU and logical-core count;
- active Windows power profile;
- whether Microsoft Defender real-time protection is enabled;
- canonical Cargo target path.

No Cargo, rustc, rust-analyzer, Tauri, or Extractum process may be active when a
measurement sequence begins. Use the existing `src-tauri/target`; do not create
slice-specific target directories and do not run `cargo clean`.

### Probes

Capture two baseline probes before changing workspace structure:

1. **Domain probe:** an inert, reversible comment edit in
   `notebooklm_export/renderer.rs`.
2. **Shell probe:** an inert, reversible comment edit in the surviving
   `notebooklm_export/mod.rs`.

For each probe:

1. run one discarded warm-up cycle;
2. perform five recorded cycles;
3. in each cycle, insert the exact probe comment, run
   `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`,
   record wall time and exit status, restore the file byte-for-byte, and run a
   no-op check before the next cycle;
4. store raw logs and JSON summaries in a temporary directory outside the
   repository.

After extraction, repeat the domain probe in the same logical `renderer.rs` at
its new crate path. Repeat the shell probe in the unchanged application
`notebooklm_export/mod.rs`.

Also capture one `cargo build --timings` report and repeated no-op check values
before and after. These are diagnostic evidence, not hard retention gates.

### Predeclared Retention Gates

Retain the extraction only when all of these are true:

1. the median domain-probe time improves by at least 25%;
2. the same median improves by at least 2.0 seconds absolutely;
3. the shell-probe median does not regress by more than 5%;
4. the shell-probe median does not regress by more than 0.5 seconds
   absolutely;
5. the complete Rust test inventory and all correctness gates pass.

Both shell limits must be satisfied. The thresholds are fixed before any
candidate measurement. Individual runs may not be discarded after results are
visible, except for the single predeclared warm-up.

If the candidate fails a performance or correctness gate, restore the complete
pre-extraction source and manifest state byte-for-byte. Preserve the negative
experiment and raw measurements in the verification record; do not retain a
partial crate split.

## Test Inventory

Save the complete baseline produced by:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list
```

The 22 tests in the seven candidate modules move with their implementations:

- `filename`: 5;
- `links`: 1;
- `media`: 2;
- `renderer`: 6;
- `glossary`: 1;
- `chunker`: 7.

The implementation plan must contain an explicit rename map from every current
`notebooklm_export::<module>::tests::<name>` entry to its new render-crate test
name. Every baseline test must either keep its name or resolve through this
declared map. The post-inventory count must equal the baseline count, names must
be unique, and all 22 old test functions must be absent from the application
source files.

## Contract Protection

Add a source-level Vitest contract that normalizes CRLF/LF and verifies:

- the workspace contains `extractum-notebooklm-render`;
- the new crate's direct dependency roots are exactly the approved four;
- its `lib.rs` has a curated explicit surface and no glob re-export;
- its seven source modules contain no Tauri, SQLx, grammers, DB, readiness, or
  application-crate imports;
- the seven old application `.rs` files no longer exist;
- application compatibility modules re-export explicit item lists and contain
  no copied implementations;
- all 22 pure test functions live only in the render crate.

Update the existing workspace core contract if its exact workspace-member or
local-dependency allowlist becomes stale. A full `npm.cmd run test` RED must
identify every stale contract before the implementation is considered green.

## Verification Strategy

The retained candidate must pass:

1. focused tests for `extractum-notebooklm-render`;
2. focused application `notebooklm_export` tests with a confirmed nonzero
   inventory;
3. the source-boundary contract and all existing workspace contracts;
4. the complete workspace inventory/rename-map comparison;
5. `npm.cmd run check:rustfmt`;
6. `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`;
7. `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`;
8. `npm.cmd run verify`;
9. `npm.cmd run tauri -- build --no-bundle`;
10. release startup and normal-shutdown smoke.

Any filtered Cargo test command that executes zero tests is a failed
verification step even when Cargo exits 0.

The release smoke must report navigation as a limitation unless it is actually
performed by a human or desktop automation. Build duration is not a retention
metric for this slice.

## Acceptance Criteria

The slice is accepted only when:

1. the seven pure modules and their 22 tests exist only in
   `extractum-notebooklm-render`;
2. Tauri, SQLx, DB access, filesystem orchestration, and `AppError` remain in
   the application;
3. frontend wire values and Markdown/file output are behaviorally unchanged;
4. the new crate has exactly the approved dependency roots and curated API;
5. the full test inventory is preserved through the explicit rename map;
6. all automated and release gates pass;
7. the predeclared domain and shell performance thresholds pass.

If criterion 7 fails, the correct accepted outcome is a documented negative
experiment with no retained production-code or workspace-structure change.

## Non-Goals

- Moving `query.rs` or `message_mapping.rs`.
- Moving SQLx, readiness, Tauri commands, progress events, or filesystem code.
- Redesigning NotebookLM export behavior or Markdown formatting.
- Changing DTO fields, serde names, events, phases, warnings, or persisted
  manifests.
- Introducing a general-purpose rendering framework.
- Splitting the seven-module candidate into several tiny crates.
- Adding `cargo-nextest`, sccache, linker changes, or a new target directory.
- Optimizing release build or WiX packaging time.

## Rejected Alternatives

### Extract only `filename`, `links`, and `media`

This has the lowest movement risk but is too small to plausibly meet the
predeclared 25% and 2-second domain threshold. It would add workspace and API
surface cost without moving the main rendering work.

### Include `message_mapping` and `query`

This would increase moved line count but would pull SQLx, readiness contracts,
application errors, decompression, and database-specific row models into the
new crate. It weakens the pure boundary and makes compile-time conclusions
harder to attribute.

### Move the complete NotebookLM export domain at once

The command, progress, DB, filesystem, and rendering responsibilities are too
intertwined for a mechanical first extraction. A full move would be a broader
architecture project rather than a bounded compile-time experiment.

## Follow-Up

If the candidate is retained, use the measurement evidence to decide whether a
later slice should isolate SQL row mapping from orchestration. That follow-up
requires its own dependency map and design. If the candidate is rejected, do
not attempt a larger NotebookLM crate until profiling identifies a different
compile-time boundary.
