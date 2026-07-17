# NotebookLM Render Crate Boundary Design

**Status:** Approved in conversation
**Date:** 2026-07-17

## Summary

First run a cheap surrogate experiment against the already-extracted
`extractum-core`. Extract the pure NotebookLM rendering pipeline into a second
domain crate, `extractum-notebooklm-render`, only if that preflight demonstrates
that an extracted dependency can materially improve the full incremental
workspace-check loop on this repository and machine.

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
module. The goal is to test, before implementation, whether a crate boundary
can improve the full incremental workspace-check loop without slowing edits to
the remaining application shell. Focused package checks are a different daily
workflow and are not allowed to substitute for the selected workspace metric.

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

The earlier workspace/core experiment already provides a warning against the
original measurement model. Its application-domain comment probe measured
7,608 ms before and 7,632 ms after the first core extraction (+0.32%). Its
application-shell probe measured 7,592 ms before and 7,634 ms after (+0.55%).
Both probe files remained in the application crate, so this evidence shows that
the first split did not accelerate app-file edits; it does not directly measure
an edit inside an extracted dependency. Stage 0 below supplies that missing
comparison before another extraction is attempted. The older 39.7-second
incremental value from the 2026-07-14 profiling session was captured under a
colder cache state and is not the baseline for this design. Current thresholds
therefore apply only to fresh, same-session paired measurements.

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

### Stage 0: Surrogate Go/No-Go

Before changing a manifest or moving a source file, run a paired experiment
using boundaries that already exist:

1. **Application-domain probe:** an inert, reversible comment edit in
   `notebooklm_export/renderer.rs`.
2. **Extracted-dependency surrogate:** the same inert comment edit in
   `extractum-core/src/media_metadata.rs`.

This surrogate is intentionally conservative: under `--all-targets`, a core
edit also checks core's own test targets and touches a dependency shared more
broadly than the proposed render crate, so it may slightly overstate the cost
of editing the future render crate. Record that asymmetry when interpreting a
result close to either threshold; it does not relax the predeclared gate.

Both probes use the full command:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
```

Alternate the two variants after one discarded warm-up per variant, recording
five successful cycles of each. In every cycle, restore the file byte-for-byte
and run a no-op check before the next edit. Store raw logs, hashes, and JSON
summaries outside the repository.

The implementation plan must preserve the established measurement-failure
classification. Missing run metadata, failure before Cargo starts, or an
unconfirmed runner result is an infrastructure failure: invalidate the whole
measurement session and restart from both warm-ups. A recorded Cargo invocation
with complete metadata and a nonzero exit is a confirmed probe failure: stop
and investigate it rather than treating it as a timing sample or silently
continuing. No failed run may be replaced piecemeal inside an otherwise valid
sequence.

The surrogate is a go only when its median is at least 25% and at least 2.0
seconds faster than the fresh application-domain median. These thresholds
express a user-visible minimum improvement; they are not calibrated from the
obsolete 39.7-second cold-cache sample.

If the surrogate fails either threshold, stop the slice before implementation.
Write a verification record for the negative preflight, retain no production
or workspace-structure change, and do not spend time on the seven-module move,
post-extraction measurements, release build, or test-rename migration.

For diagnostic context only, also measure the focused surrogate loop with
`cargo check -p extractum-core --all-targets`. A fast focused result does not
override a failed workspace go/no-go decision. Choosing focused package work as
the product goal requires a revised specification and user approval.

### Stage 1: Candidate Measurement

Stage 1 exists only after the surrogate passes.

Capture fresh baselines for:

1. **Domain probe:** the same comment edit in
   `notebooklm_export/renderer.rs`.
2. **Shell probe:** the same comment edit in the surviving
   `notebooklm_export/mod.rs`.

After extraction, repeat the domain probe in the same logical `renderer.rs` at
its new crate path and repeat the shell probe in the unchanged application
file. Use one discarded warm-up and five recorded cycles per variant, preserve
byte-for-byte restoration, and keep all raw artifacts outside the repository.

Also capture one `cargo build --timings` report and repeated no-op check values
before and after. These are diagnostic evidence, not hard retention gates.

### Predeclared Candidate Retention Gates

Retain the extraction only when all of these are true:

1. the median domain-probe time improves by at least 25%;
2. the same median improves by at least 2.0 seconds absolutely;
3. the shell-probe median does not regress by more than 5%;
4. the shell-probe median does not regress by more than 0.5 seconds
   absolutely;
5. the complete Rust test inventory and all correctness gates pass.

Both shell limits must be satisfied. Individual runs may not be discarded
after results are visible, except for each predeclared warm-up. If the candidate
fails, restore the complete pre-extraction state byte-for-byte and preserve the
negative result without retaining a partial crate split.

## Test Inventory

Save the complete baseline produced by:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list
```

The 22 tests in the seven candidate modules move with their implementations:

- `model`: 0, so it has no rename-map entries;
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

The measurement slice is accepted when Stage 0 produces five valid paired
samples per variant, restores both probe files byte-for-byte, and records the
predeclared go/no-go decision without changing thresholds after observation.

The crate extraction is retained only when:

1. the seven pure modules and their 22 tests exist only in
   `extractum-notebooklm-render`;
2. Tauri, SQLx, DB access, filesystem orchestration, and `AppError` remain in
   the application;
3. frontend wire values and Markdown/file output are behaviorally unchanged;
4. the new crate has exactly the approved dependency roots and curated API;
5. the full test inventory is preserved through the explicit rename map;
6. all automated and release gates pass;
7. Stage 0 passed both surrogate thresholds;
8. the predeclared Stage 1 domain and shell thresholds pass.

If Stage 0 fails, the correct accepted outcome is a documented negative
preflight with no implementation attempt. If Stage 1 fails, the correct
accepted outcome is a documented negative candidate with no retained
production-code or workspace-structure change.

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
- Claiming focused package-check speed as an improvement to rust-analyzer or
  the full workspace-check loop.

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

### Use focused package commands as the current acceptance metric

`cargo check -p extractum-notebooklm-render --all-targets` and focused package
tests are the workflow most likely to benefit from a crate split. That is a
valid alternative product goal, but it does not accelerate the selected full
workspace command and should not be introduced silently after a failed
surrogate. It requires a separately approved revision with newly calibrated
focused-loop thresholds.

## Follow-Up

If the candidate is retained, use the measurement evidence to decide whether a
later slice should isolate SQL row mapping from orchestration. If Stage 0
rejects the workspace hypothesis, decide explicitly whether focused package
development is valuable enough to justify a new spec. Do not attempt a larger
NotebookLM crate until profiling identifies a different full-workspace
compile-time boundary.
