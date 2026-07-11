# External Process Lifecycle Design

**Date:** 2026-07-11
**Status:** awaiting written-spec review

## Goal

Guarantee bounded, observable ownership and cleanup of external processes
started by Extractum: `yt-dlp`, the Gemini Browser sidecar, and the dedicated
CDP Chrome process. Closing the application, cancelling the owning operation,
or reaching a timeout must not leave orphan processes.

## Scope

Included:

- all real `yt-dlp` invocations through `youtube/ytdlp.rs`;
- Gemini Browser Node and bundled Tauri shell sidecar transports;
- the Chrome process launched by `gemini_bridge_start_cdp_chrome`;
- application exit coordination with a three-second cleanup deadline;
- regression tests, live Windows verification, and current-state docs.

Excluded:

- business-state changes for source, prompt-pack, analysis, or Apalis jobs;
- startup reconciliation of internal worker records;
- cancellation of internal analysis/prompt-pack tasks;
- browsers not launched and owned by Extractum.

## Current Risks

`run_ytdlp_with_options` waits on `command.output()` inside `timeout`. When the
timeout or the existing source-job cancellation `select!` drops that future,
the Tokio child handle is dropped without an explicit kill/reap contract. The
process can continue after the caller has reported timeout or cancellation.

Gemini Browser state owns sidecar and CDP Chrome handles. Their Drop
implementations attempt cleanup, but there is no application-level shutdown
barrier, shared deadline, admission gate, or verification that all transports
have terminated before the Tauri process exits.

## Selected Architecture

Keep process ownership inside each subsystem and add a thin application exit
coordinator. This avoids a type-erased global registry spanning incompatible
Tokio `Child`, synchronous `std::process::Child`, and Tauri shell
`CommandChild` types.

Rejected alternatives:

- A single heterogeneous process registry centralizes counting but requires a
  complex erased process interface and moves transport knowledge out of its
  owning subsystem.
- Drop-only cleanup is useful as a last resort but cannot provide graceful
  stop, a shared deadline, deterministic reap, or shutdown admission control.

## Application Shutdown State

Add `ExternalProcessShutdownState` managed by Tauri. It has atomic phases:

- `running`: new external operations may start;
- `shutting_down`: new operations fail before spawn;
- `complete`: cleanup finished and the application may exit.

The phase is runtime-only and not persisted or exposed as a domain/API value.
It therefore does not add a value-registry entry.

Replace the terminal `.run(...).expect(...)` usage with the Tauri event-loop
form that receives `RunEvent`. On the first `RunEvent::ExitRequested`:

1. call `api.prevent_exit()`;
2. atomically transition `running → shutting_down`;
3. spawn exactly one async cleanup task;
4. run subsystem cleanup under a shared three-second deadline;
5. force-kill and reap any remaining owned processes;
6. set the phase to `complete` and call `AppHandle::exit(0)`.

Repeated exit requests during `shutting_down` are prevented but do not create
another cleanup task. Exit requests after `complete` are not prevented. This
guard avoids duplicate cleanup and a programmatic-exit loop.

Every external-process entry point checks the admission state immediately
before spawning. Rejection uses a typed validation error with the message
`Application is shutting down` and does not start a process.

## Managed yt-dlp Operations

Add a YouTube-owned runtime registry managed by Tauri. Each active operation
has an internal monotonically increasing ID, a `CancellationToken`, and a
completion notification. The registry accepts registrations only in the
application `running` phase.

Public Tauri commands and frontend DTOs remain unchanged. Internal backend
metadata/comments/captions/preview functions explicitly receive
`&YoutubeProcessRegistry` (or an `Arc` clone for spawned work), threaded from
their existing `AppHandle`/managed-state command boundary. No global singleton
is introduced. `run_ytdlp_with_options` performs these steps:

1. construct and spawn the configured child with piped stdout/stderr;
2. register the managed operation;
3. move the `Child` into a dedicated async task;
4. return a caller future guarded by an RAII cancellation guard;
5. have the task select between child exit, the existing operation timeout,
   and cancellation;
6. on timeout/cancellation, call `child.kill().await`, which kills and waits;
7. finish stdout/stderr readers, produce the current result type, then remove
   the registry entry and notify waiters.

If the caller future is dropped by the existing source-job cancellation
`select!`, the guard cancels the managed task. This closes the current gap
without threading source-job tokens through every metadata/comments/captions
function.

Normal non-zero exits keep current `yt-dlp` error classification. Timeout keeps
the existing timeout messages. Source-job cancellation keeps its existing
`Source job cancelled` result at the outer job boundary. Application shutdown
uses `Application is shutting down` for newly rejected work; in-flight caller
futures are cancelled while their managed tasks perform kill/reap.

Shutdown closes registry admission, cancels every active token, and waits for
the registry to become empty within the shared deadline. Cleanup failures are
sanitized warnings: they include operation ID and stage only, never command
arguments, cookies, output payloads, or local paths.

## Gemini Sidecar Lifecycle

`GeminiBrowserState` remains the owner of its optional sidecar. Shutdown takes
the handle out of the state first, so concurrent commands cannot continue using
it.

Graceful shutdown sends the existing sidecar protocol `Stop` command and waits
for acknowledgement/termination within the remaining shared budget. If the
sidecar does not terminate:

- Tokio Node transport calls kill and awaits reap;
- Tauri shell transport calls `CommandChild::kill()` and consumes receiver
  events until `CommandEvent::Terminated` or the deadline expires.

The existing Drop implementation remains a best-effort fallback, not the
normal shutdown path. Its behavior must stay non-panicking.

## CDP Chrome Lifecycle

`GeminiBrowserState` remains the owner of `ChromeCdpProcess`. Shutdown takes the
handle out of state and calls an explicit `shutdown()` method that performs
`kill` followed by `wait`. Chrome has no Extractum protocol for graceful exit,
so this immediate owned-process termination is the normal path.

Drop retains the current kill/wait fallback. Extractum never enumerates or
terminates Chrome instances it did not launch.

## Failure Isolation

Shutdown attempts all three subsystem cleanups even if one reports an error.
The coordinator records sanitized warnings and proceeds to the remaining
subsystems. The three-second deadline bounds total graceful cleanup, not three
seconds per subsystem. After the deadline, force cleanup runs for all remaining
handles before programmatic exit.

No shutdown log may contain process arguments, cookies, prompts, provider
output, command stdout/stderr, profile directories, or executable paths.

## Testing Strategy

Implementation follows red-green-refactor cycles.

Automated tests cover:

- normal `yt-dlp` exit removes its registry entry and returns captured output;
- timeout kills/reaps the fake child and preserves the timeout error;
- dropping the caller future triggers cancellation and reap;
- global shutdown closes admission, cancels active operations, and waits for an
  empty registry;
- registration after shutdown returns the typed error without spawning;
- sidecar graceful Stop precedes force-kill;
- stalled Node and Tauri shell transports use their transport-specific
  kill/reap behavior;
- explicit Chrome shutdown and Drop fallback both kill/wait once;
- repeated `ExitRequested` events create one cleanup task and a completed phase
  permits final exit;
- one subsystem failure does not skip later cleanup;
- source-level contracts keep the Tauri lifecycle hook and forbid secret/path
  data in shutdown warnings.

Process tests use injected fake launch/transport adapters; they do not depend
on installed `yt-dlp`, Node, or Chrome binaries. Existing real-process smoke
tests remain separate integration evidence.

Final automated verification includes:

```powershell
npm.cmd run test
npm.cmd run check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

## Live Verification

On Windows, using a release GUI build:

- start a deliberately long `yt-dlp` operation, close Extractum, and verify no
  owned `yt-dlp.exe` remains;
- start the Gemini sidecar and Extractum-owned CDP Chrome, close Extractum, and
  verify both owned processes terminate;
- close Extractum with no active external processes;
- confirm total exit delay is at most approximately three seconds plus small
  scheduling/process-reap overhead.

Process identity is captured before exit from the owning handles/PIDs; tests do
not infer ownership by killing every process with a matching executable name.

## Documentation

Update:

- `docs/project.md`: external-process ownership and bounded shutdown;
- `docs/browser-providers-llm-troubleshooting.md`: sidecar/Chrome cleanup and
  recovery diagnostics;
- `AGENTS.md`: require explicit kill/reap ownership for new child processes and
  prohibit dropping process futures as cancellation;
- `docs/value-registry.md` only if implementation introduces a persisted or API
  string value beyond this design (none is planned).

## Acceptance Criteria

- Timeout, caller cancellation, and app shutdown cannot orphan `yt-dlp`.
- Gemini sidecar receives graceful Stop before bounded force cleanup.
- Extractum-owned CDP Chrome is killed and reaped; unrelated Chrome is untouched.
- New external operations are rejected after shutdown begins.
- Repeated exit requests do not duplicate cleanup or loop programmatic exit.
- Total graceful shutdown uses one three-second budget.
- Cleanup failures do not prevent other subsystem cleanup and do not leak
  sensitive process data.
- Automated tests, full Rust/TypeScript checks, current-state docs, and Windows
  release GUI verification describe and confirm the implemented behavior.
