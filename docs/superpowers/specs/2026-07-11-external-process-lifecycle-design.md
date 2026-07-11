# External Process Lifecycle Design

**Date:** 2026-07-11
**Status:** awaiting written-spec review

## Goal

Guarantee bounded, observable ownership and cleanup of external process trees
started by Extractum: `yt-dlp`, the Gemini Browser sidecar, and the dedicated
CDP Chrome process. On Windows, controlled shutdown/cancellation and an
unexpected Extractum process death must not leave those owned trees running.
Other platforms receive deterministic parent kill/reap during controlled
shutdown; crash-safe tree containment there is not claimed by this slice.

## Scope

Included:

- all real `yt-dlp` invocations through `youtube/ytdlp.rs`;
- Gemini Browser Node and bundled Tauri shell sidecar transports;
- the Chrome process launched by `gemini_bridge_start_cdp_chrome`;
- application exit coordination with a three-second cleanup deadline;
- per-process Windows Job Objects with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`;
- regression tests, live Windows verification, and current-state docs.

Excluded:

- business-state changes for source, prompt-pack, analysis, or Apalis jobs;
- startup reconciliation of internal worker records;
- cancellation of internal analysis/prompt-pack tasks;
- browsers not launched and owned by Extractum.
- non-Windows crash-safe descendant containment.

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

## Windows Process-Tree Containment

Add a small Windows-specific owner backed by `windows-sys` process/threading
and job-object APIs. Immediately after each owned child spawn, create a Job
Object, set `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, open the child process by PID,
and assign it to that job before exposing the operation/handle to callers.
Each `yt-dlp` operation, sidecar, and CDP Chrome launch gets its own job rather
than sharing an application-wide job, so cancelling one operation cannot kill
unrelated owned processes.

The job handle moves with the subsystem process owner and is closed only after
normal termination/reap. On cancellation, timeout, force cleanup, or Extractum
crash, closing the handle terminates the entire assigned tree, including
helpers such as `ffmpeg` and Chrome descendants. If job creation or assignment
fails, the just-spawned child is immediately killed/reaped and the launch
returns a sanitized internal error; an uncontained child is never published.

On non-Windows systems the containment owner is a no-op marker and controlled
cleanup targets the direct child. OS-level crash cleanup for descendants is a
documented platform limitation and future work.

## Application Shutdown State

Add `ExternalProcessShutdownState` managed by Tauri. It has atomic phases:

- `running`: new external operations may start;
- `shutting_down`: new operations fail before spawn;
- `completed`: cleanup finished and the application may exit.

The phase is runtime-only and not persisted or exposed as a domain/API value.
It therefore does not add a value-registry entry.

Replace the terminal `.run(...).expect(...)` usage with the Tauri event-loop
form that receives `RunEvent`. On the first `RunEvent::ExitRequested`, preserve
its `code: Option<i32>` (defaulting to `0` only when absent), then:

1. call `api.prevent_exit()`;
2. atomically transition `running → shutting_down`;
3. spawn exactly one async cleanup task and an independent hard watchdog;
4. run subsystem cleanup under a shared three-second deadline;
5. force-kill and reap any remaining owned processes;
6. set the phase to `completed` and call `AppHandle::exit(original_code)`.

Repeated exit requests during `shutting_down` are prevented but do not create
another cleanup task. Exit requests after `completed` are not prevented. This
guard avoids duplicate cleanup and a programmatic-exit loop.

The cleanup task is not trusted as the sole exit path. An OS watchdog thread,
created outside the async runtime task, waits four seconds from the first exit
request; unless the phase is already `completed`, it records a constant
sanitized warning, marks the phase completed, and calls
`AppHandle::exit(original_code)`. Thus an async-task panic, mutex deadlock, or
force-reap stall cannot leave an invisible application running.
The first three seconds are the shared graceful budget; force cleanup receives
the remaining hard-cap interval of at most one second.

Use a single admission gate shared by all external-process subsystems.
Acquiring a permit atomically checks `running` and increments the count of
in-progress spawn/install transactions. The permit is held from before spawn
until the child is either contained and installed in its owning state/registry,
or killed/reaped after any failure. Shutdown closes admission and waits for all
outstanding permits before taking installed handles. This removes check-then-
act races for `yt-dlp`, sidecar, and Chrome. Rejection uses a typed validation
error with the message `Application is shutting down` and does not spawn.

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

1. acquire an admission permit and reserve/register the operation before spawn;
2. construct and spawn the child with piped stdout/stderr;
3. assign the child to its per-operation Windows Job Object;
4. on spawn/containment failure, kill/reap any new child, remove the reservation,
   and preserve existing spawn-error classification, including
   `ErrorKind::NotFound` to `yt-dlp is not available on PATH`;
5. move the `Child`, its Job Object owner, and any cookie `NamedTempFile` into a
   dedicated async task before releasing the admission permit;
6. start stdout and stderr drain futures immediately and poll them concurrently
   with child exit, deadline, and cancellation (equivalent to the pipe-draining
   semantics of `wait_with_output`);
7. return a caller future guarded by an RAII cancellation guard;
8. on timeout/cancellation, terminate the contained tree and await direct-child
   reap within the hard cleanup cap;
9. join the already-running output drains, produce the current result type,
   remove the registry entry, notify waiters, then drop the cookie temp file.

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

`GeminiBrowserState` remains the owner of its optional sidecar. Existing
`request_sidecar` holds the sidecar mutex across the entire protocol request,
including potentially long Gemini generation. Shutdown therefore first cancels
a state-level in-flight-request token. Every sidecar request selects between
that token and the protocol future; cancellation drops the borrowed protocol
future, returns a typed shutdown/cancellation error, and releases the mutex.
Only then does shutdown take the handle out of state. The bounded force path
must never wait indefinitely for the request mutex.

Spawn uses the shared admission permit across process creation, Job Object
assignment, and installation into `GeminiBrowserState`. Shutdown closes
admission and waits for outstanding spawn/install permits before cancelling
requests and taking the installed handle.

Graceful shutdown sends the existing sidecar protocol `Stop` command and waits
for acknowledgement/termination within the remaining shared budget. If the
sidecar does not terminate:

- Tokio Node transport calls kill and awaits reap;
- Tauri shell transport calls `CommandChild::kill()` and consumes receiver
  events until `CommandEvent::Terminated` or the hard cleanup cap expires.

The existing Drop implementation remains a best-effort fallback, not the
normal shutdown path. Its behavior must stay non-panicking.

## CDP Chrome Lifecycle

`GeminiBrowserState` remains the owner of `ChromeCdpProcess`. Shutdown takes the
handle out of state and calls an explicit `shutdown()` method that performs
contained-tree termination followed by direct-child `wait`. Chrome has no
Extractum protocol for graceful exit, so this immediate owned-process
termination is the normal path. Spawn, Job Object assignment, and installation
into state occur under one shared admission permit. Synchronous Chrome
kill/wait runs through `spawn_blocking` so it cannot stall the Tokio worker that
drives other cleanup futures.

Drop retains the current kill/wait fallback. Extractum never enumerates or
terminates Chrome instances it did not launch.

Known limitation: if Chrome finds an already-running browser with the same
`--user-data-dir`, the newly spawned process may delegate to it and exit. That
existing browser was not launched by the current operation, cannot be assigned
retroactively to its Job Object, and remains intentionally unmanaged. The CDP
startup path must detect that the owned child exited and must not claim that an
unrelated delegated browser is owned or will be terminated on shutdown.

## Failure Isolation

Shutdown attempts all three subsystem cleanups even if one reports an error.
The coordinator records sanitized warnings and proceeds to the remaining
subsystems. The three-second deadline bounds total graceful cleanup, not three
seconds per subsystem. After that deadline, force cleanup runs concurrently for
all remaining handles under the independent four-second hard exit cap. The
watchdog exits even if force cleanup or its task panics/stalls.

No shutdown log may contain process arguments, cookies, prompts, provider
output, command stdout/stderr, profile directories, or executable paths.

## Testing Strategy

Implementation follows red-green-refactor cycles.

Automated tests cover:

- normal `yt-dlp` exit removes its registry entry and returns captured output;
- output larger than the platform pipe buffer is drained concurrently and
  completes successfully without a false timeout;
- timeout kills/reaps the fake child and preserves the timeout error;
- dropping the caller future triggers cancellation and reap;
- global shutdown closes admission, cancels active operations, and waits for an
  empty registry;
- registration after shutdown returns the typed error without spawning;
- admission shutdown racing each spawn either rejects before spawn or publishes
  a contained, registered handle that shutdown subsequently cleans;
- spawn NotFound and other spawn errors preserve current classification and
  remove the pre-spawn reservation;
- the cookie temp file remains alive until managed task termination and is
  removed only after kill/reap;
- sidecar graceful Stop precedes force-kill;
- shutdown cancellation releases a mutex held by an in-flight long sidecar
  request before the graceful/force path takes its handle;
- stalled Node and Tauri shell transports use their transport-specific
  kill/reap behavior;
- explicit Chrome shutdown and Drop fallback both kill/wait once;
- repeated `ExitRequested` events create one cleanup task, preserve the first
  requested exit code, and a `completed` phase permits final exit;
- a panicking or stalled cleanup task still exits through the four-second
  watchdog with the preserved code;
- one subsystem failure does not skip later cleanup;
- Windows Job Object closure terminates fake descendant processes as well as
  the direct child;
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
  owned `yt-dlp.exe` or helper descendant remains;
- start the Gemini sidecar and Extractum-owned CDP Chrome, close Extractum, and
  verify both owned processes terminate;
- close Extractum with no active external processes;
- confirm total exit delay is at most approximately three seconds plus small
- force-terminate Extractum during an owned test process tree and confirm the
  Windows Job Object removes that tree.

The normal target is three seconds plus small scheduling overhead; the hard
watchdog requires process exit by approximately four seconds even when cleanup
stalls.

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

- On Windows, timeout, caller cancellation, app shutdown, and Extractum crash
  cannot orphan the owned `yt-dlp` process tree.
- Gemini sidecar receives graceful Stop before bounded force cleanup.
- Extractum-owned CDP Chrome is killed and reaped; unrelated Chrome is untouched.
- New external operations are rejected after shutdown begins.
- Repeated exit requests do not duplicate cleanup or loop programmatic exit.
- The first requested exit code is preserved.
- Total graceful shutdown uses one three-second budget and the independent
  watchdog enforces an approximately four-second hard cap.
- Cleanup failures do not prevent other subsystem cleanup and do not leak
  sensitive process data.
- Automated tests, full Rust/TypeScript checks, current-state docs, and Windows
  release GUI verification describe and confirm the implemented behavior.
