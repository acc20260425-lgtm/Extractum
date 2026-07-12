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

- Record the current single-warning `cargo check --all-targets` baseline.
- Run focused `youtube::jobs` tests after migrating the two tests.
- Run focused `youtube::process_runtime` tests to preserve managed yt-dlp
  cancellation coverage.
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` and require
  zero Rust warnings.

## Acceptance Criteria

- `run_source_job_step_with_cancel` no longer exists.
- Both renamed cancellation tests call
  `run_source_job_step_with_cancel_and_processes` with a fresh registry.
- Production source-job call sites and cancellation behavior are unchanged.
- Focused jobs and process-runtime tests pass.
- `cargo check --all-targets` exits successfully without warnings.
