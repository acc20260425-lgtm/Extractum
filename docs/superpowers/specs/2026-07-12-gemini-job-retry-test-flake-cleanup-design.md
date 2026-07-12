# Gemini Job Retry Test Flake Cleanup Design

## Goal

Remove a redundant high-contention trigger for a reproducible parallel-test
flake and reduce its observed probability without changing Gemini Browser job
behavior, application migrations, Apalis storage setup, or production code.

## Observed Failure

During a full Rust test run,
`failed_gemini_browser_job_retry_is_not_attempted` failed while enqueueing an
Apalis job:

```text
table Jobs has 13 columns but 14 values were supplied
```

The same test passed in isolation and the next full test run passed.

## Investigation Evidence

`failed_gemini_browser_job_is_not_retried` and
`failed_gemini_browser_job_retry_is_not_attempted` are exact duplicate test
adapters. Both call the same
`assert_failed_gemini_browser_job_is_not_retried()` helper with no different
setup or assertion.

Both adapters were introduced together in commit `8414a198` with these same
bodies. No separate historical intent or distinct contract was found for the
second name.

The shared helper creates a fresh `tempfile::tempdir()`, opens
`extractum.db` inside that directory, applies application migrations, runs
Apalis storage setup, enqueues the same test job, executes a failing worker,
and asserts one execution with terminal status and one total attempt.

Twenty repeated parallel runs of the two-test filter produced one schema
failure. Twenty repeated runs with `--test-threads=1` produced no failures.
Each invocation uses its own temporary database path, so a fixed shared SQLite
URL or reused test database was not found.

The evidence identifies concurrent execution of the duplicate scenario as the
only observed trigger. However, the full suite contains many other concurrent
migration, Apalis setup, and enqueue paths: `apply_all_migrations_for_test_pool`
is used across 19 source files, including 10 call sites in
`gemini_browser/jobs.rs`. Removing the identical pair reduces one especially
synchronized source of contention; it does not prove that a process-wide race
cannot recur between different tests. The exact internal mechanism inside
SQLx, SQLite, or Apalis migration/enqueue handling across separate database
files remains unknown.

## Selected Design

Delete only the redundant
`failed_gemini_browser_job_retry_is_not_attempted` test adapter. Keep
`failed_gemini_browser_job_is_not_retried` and
`assert_failed_gemini_browser_job_is_not_retried()` unchanged.

The retained test continues covering the complete contract:

- enqueue uses a job configured for one total attempt;
- the failing worker executes exactly once;
- the stored row becomes `Failed` or `Killed`;
- `attempts` equals 1;
- `max_attempts` equals 1.

Removing an identical second caller eliminates redundant coverage and reduces
the probability of this particular synchronized collision without changing
the tested behavior. It is not presented as a complete fix for every possible
suite-wide migration race.

## Rejected Alternatives

- A global test mutex would serialize redundant tests, add shared test state,
  and slow the suite without adding coverage.
- Changing temporary database naming is unsupported by the evidence because
  each invocation already uses a distinct `tempdir`.
- Modifying application migrations or Apalis storage setup would expand a
  test-only cleanup into production-sensitive code without a proven production
  defect.
- Investigating third-party migration internals becomes required if any test
  later reproduces a `Jobs` schema error of the form `N columns but M values
  were supplied`; recurrence is keyed to the error signature, not to the name
  of the removed test.

## Scope

The implementation changes only the test module in
`src-tauri/src/gemini_browser/jobs.rs`. It removes one test function and does
not change helpers, imports, production functions, schemas, migrations,
dependencies, serialized values, TypeScript code, or `docs/value-registry.md`.

## Verification

- Mechanically assert that the current source contains both duplicate adapter
  names before editing and exactly one no-retry adapter afterward.
- Remove only `failed_gemini_browser_job_retry_is_not_attempted`.
- Run the retained test 20 times under the normal test harness.
- Run the complete `gemini_browser::jobs` test group.
- Run the full Rust test suite 3 times to check for recurrence.
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` and require
  a successful zero-warning result.

Because the original failure is probabilistic, repeated passing runs reduce
but cannot mathematically prove the absence of every third-party concurrency
defect. A recurrence after removing the duplicate test reopens the deeper
migration investigation rather than justifying additional serialization.

The first hypotheses for that follow-up are:

- inspect which application migration or Apalis setup owns the visible `Jobs`
  schema at the failing enqueue boundary;
- inspect apalis-sqlite and SQLx for process-global once, lock, or cached
  migration state that could cause setup for one database to affect or skip
  setup for another database under concurrency.

These are investigation starting points, not established causes.

## Acceptance Criteria

- Exactly one no-retry behavior test remains.
- The retained assertion helper and production code are unchanged.
- All 20 retained-test runs pass.
- All `gemini_browser::jobs` tests pass.
- Three full Rust test runs pass without any `Jobs` `N columns but M values
  were supplied` failure.
- `cargo check --all-targets` succeeds with zero warnings.
