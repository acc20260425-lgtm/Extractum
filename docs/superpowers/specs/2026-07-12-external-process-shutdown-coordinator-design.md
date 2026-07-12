# External Process Shutdown Coordinator Design

**Date:** 2026-07-12
**Status:** Approved for specification review

## Purpose

Refactor application-exit orchestration so the production Tauri exit hook uses
the same framework-neutral shutdown coordinator exercised by Rust tests. The
change also corrects the graceful deadline: the existing three-second budget
starts with the first exit request and includes the admission barrier rather
than starting only after outstanding spawn/install transactions finish.

This is a focused lifecycle refactor. It does not change subsystem ownership,
Windows Job Object behavior, sidecar protocol semantics, or the four-second
hard watchdog established by the external-process lifecycle design.

## Current Problem

`src-tauri/src/external_process.rs` contains testable abstractions for cleanup,
watchdog scheduling, and final exit. Production code in `src-tauri/src/lib.rs`
does not use several of them. Instead, the Tauri `RunEvent::ExitRequested`
branch independently:

- creates the watchdog thread;
- waits for admission permits;
- starts three subsystem cleanup futures;
- applies the graceful timeout;
- completes the phase and exits.

This duplication produces dead-code warnings and has already caused at least
three production/test divergences:

- production waits for admission outside the three-second graceful timeout,
  although the approved lifecycle design defines one shared three-second
  budget from the first exit request;
- production calls `prevent_exit()` only when `begin_shutdown` succeeds, so a
  repeated `ExitRequested` during `ShuttingDown` can currently terminate the
  event loop immediately and interrupt cleanup;
- production polls all three cleanup futures through one `tokio::join!`, so a
  panic in one future drops the entire coordinator task and defers exit to the
  watchdog, while the tested `JoinSet` path isolates task panics.

The latter two behaviors violate the approved lifecycle design and are bugs,
not compatibility requirements to preserve during the refactor.

## Selected Architecture

Keep `src-tauri/src/lib.rs` as a thin Tauri adapter over a framework-neutral
coordinator in `src-tauri/src/external_process.rs`.

The coordinator owns:

- the atomic transition from `Running` to `ShuttingDown`;
- preservation of the first requested exit code;
- the absolute graceful deadline;
- watchdog scheduling;
- the admission barrier;
- concurrent cleanup execution and failure isolation;
- the single transition to `Completed` and final exit callback.

The Tauri adapter owns only:

- receiving `RunEvent::ExitRequested`;
- calling or withholding `api.prevent_exit()` according to the coordinator's
  start result;
- cloning Tauri managed state and constructing the cleanup factory;
- submitting the returned shutdown run to `tauri::async_runtime`;
- adapting `AppHandle::exit` to the coordinator's exit callback.

Tauri types do not enter the coordinator API.

## Coordinator Interface

The synchronous start operation has the conceptual form:

```rust
let result = shutdown.start(code, timing, watchdog_scheduler, exit_callback);
```

It returns one of three explicit outcomes:

- `Started(run)` when this request atomically begins shutdown;
- `AlreadyShuttingDown` when another request already owns cleanup;
- `Completed` when programmatic exit is completing the event loop.

For `Started`, the operation synchronously:

1. closes admission;
2. stores the first exit code, defaulting an absent code to zero;
3. captures an absolute graceful deadline equal to the current monotonic time
   plus `ShutdownTiming::graceful`;
4. schedules the independent watchdog immediately;
5. returns a single-use `ShutdownRun` value.

The adapter submits that value to the async runtime:

```rust
run.coordinate(cleanup_factory).await;
```

The exact Rust names may be adjusted during implementation for clarity, but
the start result, synchronous deadline capture, one-shot run ownership, and
framework-neutral dependency direction are required contracts.

`cleanup_factory` is a `Send + 'static` one-shot factory. It owns the cloned
Tauri handles needed to obtain managed state and returns the three boxed
cleanup futures for YouTube operations, the Gemini sidecar, and owned CDP
Chrome. It is invoked only after the admission barrier, so cleanup cannot take
installed handles while a permitted spawn/install transaction is publishing
one.

## Shutdown Data Flow

On the first exit request:

1. `start` atomically transitions to `ShuttingDown`, captures the deadline,
   and schedules the four-second watchdog.
2. The adapter calls `prevent_exit()` and spawns exactly one coordinator task.
3. The task waits for outstanding admission permits, bounded by the already
   running absolute graceful deadline.
4. If time remains after the barrier, the cleanup factory creates all three
   subsystem futures and the coordinator starts them concurrently.
5. The coordinator waits until all cleanup futures settle or the absolute
   deadline expires.
6. The coordinator atomically claims completion and invokes the exit callback
   with the first requested exit code.

If admission consumes the full graceful budget, the cleanup factory is not
invoked. The coordinator proceeds to final exit; existing process-owner Drop
and Job Object kill-on-close behavior remains the degradation path. The
independent watchdog still enforces the approximately four-second hard cap if
the coordinator task stalls or panics.

During `ShuttingDown`, repeated exit requests call `prevent_exit()` but do not
start another task or watchdog. After `Completed`, the adapter does not prevent
the event, avoiding a programmatic-exit loop.

## Timing Semantics

Both budgets start from the first accepted exit request:

- the graceful deadline is three seconds total;
- the watchdog deadline is approximately four seconds total.

The admission barrier and all subsystem cleanup share the same three-second
budget. They do not receive sequential three-second windows. Cleanup futures
start concurrently and share the remaining time rather than receiving a
separate timeout per subsystem.

Deadline capture uses an injected monotonic-clock seam as a required
coordinator dependency. Production supplies the real monotonic clock; tests
supply a deterministic clock coordinated with their timer driver. Paused Tokio
time alone is not the deadline abstraction because synchronous `start()` can
run outside a Tokio runtime context.

The watchdog runs on an independent OS thread through an injected scheduler.
It is not dependent on the Tokio runtime or coordinator task making progress.
The existing four-second value remains below the approximate Windows
logoff/shutdown responsiveness window documented by the lifecycle design.

## Failure Handling

Each subsystem cleanup executes as an independently joined task. An error or
panic in one cleanup must not prevent the other cleanup tasks from running.
Coordinator warnings use a dedicated stage-only helper and do not invent an
operation ID for global lifecycle stages. Subsystem warnings continue using
`operation_id` plus stage. Neither format may contain process arguments,
prompts, cookies, provider output, stdout/stderr, executable paths, or profile
paths.

When the graceful deadline expires, unfinished cleanup tasks are cancelled by
dropping/aborting their join set and the coordinator attempts final exit. The
owned-process Drop implementations and Windows Job Objects remain the fallback
for process-tree termination. If cancellation, Drop, reap, a runtime stall, or
a coordinator panic prevents progress, the watchdog attempts completion.

Completion is an atomic gate. Whichever path first changes `ShuttingDown` to
`Completed`—the coordinator or watchdog—is the only path allowed to invoke the
exit callback. Later contenders do nothing. The original exit code is
preserved on both paths.

## Testing Strategy

Rust tests exercise the public-within-crate coordinator path used by
production, with injected watchdog scheduling and exit callbacks. Tests must
cover:

- the first request returns `Started`, closes admission, and preserves its exit
  code;
- requests during shutdown return `AlreadyShuttingDown` without scheduling a
  second watchdog;
- requests after completion return `Completed`;
- an outstanding admission permit consumes part of the single graceful
  budget;
- admission consuming the full budget does not create a fresh cleanup window
  or invoke the cleanup factory;
- all three cleanup futures start concurrently;
- stalled cleanup is bounded by the absolute graceful deadline;
- an error or panic in one cleanup does not prevent the others from starting
  and settling;
- coordinator/watchdog races invoke the exit callback exactly once;
- both normal completion and watchdog fallback preserve the first exit code;
- the injected scheduler receives the four-second watchdog timing.

Tests use the injected monotonic-clock seam, together with a deterministic
timer driver, to avoid wall-clock sleeps and to keep synchronous `start()`
independent of Tokio runtime context. The production scheduler remains an
OS-thread implementation.

A focused source-level contract verifies that `lib.rs` remains a thin adapter:
it must use the coordinator start/run interface, call `prevent_exit()` for both
`Started` and `AlreadyShuttingDown`, and not call it for `Completed`. It must not
independently sleep for the watchdog duration, apply its own graceful timeout,
wait on the admission barrier, or mark shutdown complete.

The same source-contract suite verifies warning mechanics rather than trying
to intercept `eprintln!` in a Rust unit test. Coordinator code may call only the
stage-only coordinator warning helper; subsystem code retains the existing
operation-ID helper. The helper bodies and their call sites must not contain
secret, output, argument, or path fields. No injectable warning sink or pure
Tauri-outcome mapper is introduced solely for testing.

Validation includes focused coordinator tests, the lifecycle source contract,
the full Rust test suite, and `cargo check`. No new frontend behavior is
introduced.

## Documentation Impact

Update current-state lifecycle documentation to say explicitly that the
three-second budget begins with the first accepted exit request and includes
the admission barrier. Retain the existing four-second watchdog and subsystem
ownership descriptions.

No persisted, API, UI, fixture, or domain string value changes. Therefore
`docs/value-registry.md` requires inspection but no entry or edit.

## Out of Scope

- changing YouTube, sidecar, or Chrome process ownership;
- changing Windows Job Object containment;
- changing sidecar graceful-stop or mid-session-stop semantics;
- adding a force-cleanup protocol beyond existing subsystem and Drop behavior;
- changing the three-second or four-second configured durations;
- frontend or settings changes;
- unrelated cleanup of large lifecycle modules.

## Acceptance Criteria

- Production `ExitRequested` handling delegates timing, admission waiting,
  concurrent cleanup, completion, watchdog gating, and final exit to the tested
  coordinator.
- The graceful deadline starts synchronously with the first accepted exit
  request and includes admission waiting.
- Only one cleanup task, watchdog, and final exit callback can win per shutdown.
- All subsystem cleanup futures start concurrently when admission finishes
  before the deadline.
- The first exit code is preserved on normal and watchdog paths.
- Cleanup failures remain isolated and warnings remain sanitized.
- The Tauri adapter contains no duplicate shutdown orchestration.
- Focused and full Rust verification passes without the current coordinator
  dead-code warnings.
