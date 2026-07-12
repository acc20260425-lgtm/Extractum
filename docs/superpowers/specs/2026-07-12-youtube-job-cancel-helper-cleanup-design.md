# YouTube Job Cancellation Helper Cleanup Design

## Goal

Remove the final Rust dead-code warning by deleting the obsolete
`run_source_job_step_with_cancel` helper and keeping cancellation tests aligned
with the production source-job path.

## Current State

Source-job metadata, transcript, and comment steps call
`run_source_job_step_with_cancel_and_processes`. On cancellation, that helper
both returns the source-job cancellation error and calls
`YoutubeProcessRegistry::cancel_all()` so owned yt-dlp operations begin cleanup.

`run_source_job_step_with_cancel` predates process-registry integration. It
implements only the future-versus-token selection and is now called solely by
two in-file tests. Compiling this duplicate helper in production creates the
last `cargo check --all-targets` warning.

This slice follows the committed Apalis warning cleanup in `8cbbfd9c`. The
implementation plan must verify that commit is an ancestor of `HEAD` before
asserting the single-warning baseline, and its RED command must fail unless the
named `run_source_job_step_with_cancel` diagnostic is present.

## Selected Design

Delete `run_source_job_step_with_cancel` and migrate its two tests to
`run_source_job_step_with_cancel_and_processes`.

Each migrated test constructs a fresh `YoutubeProcessRegistry::new()` and
passes it to the production helper. The completed-future test continues
covering the no-token branch. The cancelled-future test continues covering the
already-cancelled token branch. Process-tree cancellation and reap behavior
remain covered by the existing `youtube::process_runtime` tests.

Rename the migrated tests to
`source_job_step_with_process_cancel_allows_completed_future` and
`source_job_step_with_process_cancel_interrupts_pending_future` so their names
describe the production helper they exercise.

In the test module's `use super::{...}` list, replace
`run_source_job_step_with_cancel` with
`run_source_job_step_with_cancel_and_processes` and add
`YoutubeProcessRegistry`. The cancelled-token test must assert both
`AppErrorKind::Validation` and the exact message `Source job cancelled`.

## Rejected Alternatives

- Marking the old helper `#[cfg(test)]` would silence the warning but preserve a
  duplicate cancellation implementation that can drift from production.
- Reusing the old helper inside the production helper would obscure the
  required `registry.cancel_all()` side effect and add indirection without
  removing behavior.
- Returning production call sites to the old helper would regress managed
  yt-dlp cancellation.

## Scope

The implementation changes only `src-tauri/src/youtube/jobs.rs`. It does not
change source-job wire values, status values, error text, Tauri commands,
process-registry behavior, yt-dlp launching, TypeScript code,
`docs/value-registry.md`, or production call sites.

## Error Handling

The production helper remains unchanged. With no cancellation token it awaits
and returns the future result. With an already-cancelled or later-cancelled
token it calls `registry.cancel_all()` and returns the existing validation
error `Source job cancelled`.

## Verification

- Verify that Apalis cleanup commit `8cbbfd9c` is present, then record the
  current single-warning `cargo check --all-targets` baseline with a required
  assertion naming `run_source_job_step_with_cancel`.
- Run focused `youtube::jobs` tests after migrating the two tests.
- Run focused `youtube::process_runtime` tests to preserve managed yt-dlp
  cancellation coverage.
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` and require
  zero Rust warnings.

The migrated cancelled-token test uses a fresh, empty registry. It proves that
the production helper returns the expected cancellation error, but cannot
observe that the helper called `registry.cancel_all()`. Existing
`youtube::process_runtime` tests prove that registry cancellation reaps managed
operations, but they do not directly connect that behavior to this helper.
Adding an observable registry cancellation seam would require changing
`process_runtime.rs`; that is an explicitly accepted coverage limitation for
this warning-only slice and a candidate for a separate lifecycle-test follow-up.

## Acceptance Criteria

- `run_source_job_step_with_cancel` no longer exists.
- Both renamed cancellation tests call
  `run_source_job_step_with_cancel_and_processes` with a fresh registry.
- The cancelled-token test asserts validation kind and exact cancellation text.
- Production source-job call sites and cancellation behavior are unchanged.
- Focused jobs and process-runtime tests pass.
- `cargo check --all-targets` exits successfully without warnings.
