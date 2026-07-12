# Apalis Prune Warning Cleanup Design

## Goal

Remove the production dead-code warning for
`apalis_jobs_prune_terminal_from_pool` without changing the Apalis terminal-job
pruning command, its retention rules, SQL behavior, or serialized API.

## Current State

The production Tauri command `apalis_jobs_prune_terminal` normalizes the
optional retention request and calls
`apalis_jobs_prune_terminal_from_pool_with_hours`. That function contains the
actual pruning behavior.

`apalis_jobs_prune_terminal_from_pool` is a test convenience wrapper. It passes
the default `TERMINAL_PRUNE_OLDER_THAN_HOURS` value and is used only by two
in-file Rust tests. Because it is compiled in production but has no production
caller, `cargo check --all-targets` reports it as dead code.

## Selected Design

Add `#[cfg(test)]` to `apalis_jobs_prune_terminal_from_pool`.

This preserves the existing test API and keeps the default-retention tests
readable. The production command continues calling
`apalis_jobs_prune_terminal_from_pool_with_hours` with the normalized request,
so custom retention values and all production behavior remain unchanged.

## Rejected Alternatives

- Removing the wrapper would make its tests duplicate the lower-level call and
  explicitly pass the default constant without improving production code.
- Calling the wrapper from production would discard support for a caller's
  normalized `older_than_hours` value and therefore change behavior.
- Adding `allow(dead_code)` would suppress rather than express the real
  test-only boundary.

## Scope

The implementation changes only `src-tauri/src/apalis_jobs.rs`. It does not
change Tauri commands, DTOs, SQL, retention values, TypeScript code,
`docs/value-registry.md`, or the remaining warning in `youtube/jobs.rs`.

## Verification

- Record the current `cargo check --all-targets` baseline with two warnings.
- Run the focused `apalis_jobs` Rust tests after adding the test-only gate.
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` and require
  no warning from `apalis_jobs.rs`.
- The only expected remaining Rust warning is
  `youtube::jobs::run_source_job_step_with_cancel`.

## Acceptance Criteria

- `apalis_jobs_prune_terminal_from_pool` is compiled only for tests.
- Existing default-retention and missing-table pruning tests pass unchanged.
- Production pruning behavior and API contracts are unchanged.
- `cargo check --all-targets` emits no warning from `apalis_jobs.rs`.
