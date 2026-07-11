# External Process Lifecycle Design

**Date:** 2026-07-11
**Status:** awaiting written-spec review

## Goal

Provide bounded, observable ownership and cleanup of external processes
started by Extractum: `yt-dlp`, the Gemini Browser sidecar, and the dedicated
CDP Chrome process. Controlled shutdown/cancellation deterministically targets
the direct owned child. Windows Job Objects additionally contain descendants
created after assignment and clean those contained trees if Extractum crashes.
Pre-assignment descendants are an explicit limitation; the design does not
claim universal orphan prevention.

## Scope

Included:

- all real `yt-dlp` invocations through `youtube/ytdlp.rs`;
- Gemini Browser Node-script and bundled-binary Tokio sidecar modes;
- the Chrome process launched by `gemini_bridge_start_cdp_chrome`;
- application exit coordination with a three-second cleanup deadline;
- per-process Windows Job Objects with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`;
- regression tests, live Windows verification, and current-state docs.

Excluded:

- business-state changes for source, prompt-pack, analysis, or Apalis jobs;
- startup reconciliation of internal worker records;
- cancellation of internal analysis/prompt-pack tasks;
- browsers not launched and owned by Extractum;
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
coordinator. This avoids a type-erased global registry spanning Tokio `Child`
and synchronous `std::process::Child` types.

Rejected alternatives:

- A single heterogeneous process registry centralizes counting but requires a
  complex erased process interface and moves transport knowledge out of its
  owning subsystem.
- Drop-only cleanup is useful as a last resort but cannot provide graceful
  stop, a shared deadline, deterministic reap, or shutdown admission control.
- Keeping the bundled sidecar on `tauri-plugin-shell::CommandChild` preserves
  the existing wrapper but exposes only a PID, prevents safe raw-handle Job
  assignment, and requires a second JSONL/event transport. It is rejected in
  favor of resolving the packaged binary path and spawning it through the same
  Tokio transport as the Node-script mode.

## Windows Process-Tree Containment

Add a small Windows-specific owner backed by `windows-sys` job-object APIs.
Immediately after each owned child spawn, create a Job Object, set
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, and assign the process using the raw
process handle already owned by `tokio::process::Child` or
`std::process::Child`. The primary path must never reopen these children by PID;
this avoids PID-reuse races that could assign and later terminate an unrelated
process. Each `yt-dlp`, sidecar, and CDP Chrome launch gets its
own job rather than sharing an application-wide job.

The job handle moves with the subsystem process owner and is closed only after
normal termination/reap. On cancellation, timeout, force cleanup, or Extractum
crash, closing the handle terminates the assigned process and descendants
created after assignment. If job creation or assignment fails, the
just-spawned child is immediately killed/reaped and the launch returns a
sanitized internal error; an uncontained raw-handle child is never published.

There is also an unavoidable spawn-to-assignment window with Rust's standard
Command APIs. `CREATE_SUSPENDED` alone is insufficient because those APIs do
not expose the primary thread handle required to resume it; eliminating the
window would require a dedicated `CreateProcessW`/`STARTUPINFOEX` launcher.
Descendants created before assignment are therefore an accepted limitation.
Atomic-at-creation Job assignment is future hardening, not an acceptance claim
of this slice.

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
The four-second hard cap is intentionally below the roughly five-second window
commonly available during Windows logoff/shutdown before an application is
treated as unresponsive.

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
8. on ordinary mid-session timeout/cancellation, call `TerminateJobObject` on
   Windows (or direct-child kill elsewhere), keep the containment handle open,
   and await direct-child reap within a separate one-second per-operation budget;
9. join the already-running output drains, produce the current result type,
   remove the registry entry, notify waiters, then drop the cookie temp file.

If the caller future is dropped by the existing source-job cancellation
`select!`, the guard cancels the managed task. This closes the current gap
without threading source-job tokens through every metadata/comments/captions
function.

The one-second per-operation reap budget is independent of application
shutdown. If it expires, close the Windows Job Object handle (or issue the
platform direct-child kill), detach the stuck waiter, remove the registry entry,
and emit a sanitized warning containing only operation ID and stage. The
detached task continues owning the cookie `NamedTempFile`; if reap remains
stuck, that file can live until Extractum exits, when process teardown removes
the final owner. During app shutdown, the global three/four-second budgets take
precedence.

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

Dropping a protocol future during `write_all` or `read_line` can leave a partial
JSON frame in the pipe. The transport is marked tainted whenever shutdown
cancels an in-flight request. A tainted transport skips protocol `Stop` and
goes directly to transport-specific force cleanup; sending Stop over the same
pipe would be unreliable and could consume the whole graceful budget.

Spawn uses the shared admission permit across process creation, Job Object
assignment, and installation into `GeminiBrowserState`. Both launch modes use
one Tokio child/JSONL transport:

- development mode spawns `node <repo-dist-script>`;
- bundled mode resolves
  `current_exe().parent()/gemini-browser-sidecar[.exe]`, matching the path
  convention used by `tauri-plugin-shell` and the observed Tauri release output,
  then spawns that executable directly.

Bundled path resolution is unit-tested for Windows and non-Windows suffixes and
verified against a release build. Direct backend spawning does not broaden
webview shell capability; no frontend command or arbitrary executable path is
exposed. Both sidecar modes apply the existing `hide_console_window` helper on
Windows, preserving the no-console behavior previously supplied by the shell
plugin. Sidecar stderr is piped and drained concurrently so it cannot block the
child; only sanitized constant diagnostics may be logged. Shutdown closes
admission and waits for outstanding spawn/install permits before cancelling
requests and taking the installed handle.

For an idle, untainted transport, graceful shutdown sends the existing sidecar
protocol `Stop` command and waits for acknowledgement/termination within the
remaining shared budget. The current sidecar `Stop` closes the managed browser
session but leaves the JSONL loop waiting on stdin, so after receiving ACK the
parent must close/take sidecar stdin to deliver EOF, then await process exit. If
ACK, EOF-driven exit, or reap does not complete in budget:

- the unified Tokio transport terminates its Job Object tree and awaits the
  direct child within the hard cleanup cap.

Remove the Tauri shell transport enum arm, `request_shell`, `CommandEvent`
buffer reassembly, and transport-specific shell force-cleanup branch. The JSONL
request/response implementation is shared by Node-script and bundled-binary
modes. Because no other project code or capability uses `tauri-plugin-shell`,
also remove `.plugin(tauri_plugin_shell::init())` from `lib.rs` and remove the
Cargo dependency. Keep `bundle.externalBin` in `tauri.conf.json`: Tauri must
still package `gemini-browser-sidecar`; only runtime spawning changes.

The existing Drop implementation remains a best-effort fallback, not the
normal shutdown path. Its behavior must stay non-panicking.

The user-triggered mid-session provider stop preserves its current lifecycle:
send protocol Stop, receive ACK, then remove the sidecar from state and let its
contained Drop fallback initiate tree termination and best-effort reap. The
shutdown-only stdin-EOF wait is not added to this command in this slice.

## CDP Chrome Lifecycle

`GeminiBrowserState` remains the owner of `ChromeCdpProcess`. Shutdown takes the
handle out of state and calls an explicit `shutdown()` method that performs
contained-tree termination followed by direct-child `wait`. Chrome has no
Extractum protocol for graceful exit, so this immediate owned-process
termination is the normal path. Spawn, Job Object assignment, and installation
into state occur under one shared admission permit. Synchronous Chrome
spawn plus Job assignment and later kill/wait run through `spawn_blocking` so
neither `CreateProcess` nor cleanup stalls the Tokio worker that drives other
futures.

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
subsystems. All graceful subsystem futures start concurrently so a stalled
sidecar cannot prevent `yt-dlp` cancellation or Chrome cleanup from beginning.
The three-second deadline bounds total graceful cleanup, not three seconds per
subsystem. After that deadline, force cleanup runs concurrently for all
remaining handles under the independent four-second hard exit cap. The
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
- a stuck-reap timeout detaches the waiter, removes the registry entry, emits a
  sanitized warning, and keeps the cookie owner alive in the detached task;
- sidecar graceful Stop precedes force-kill;
- sidecar graceful Stop ACK is followed by stdin closure and EOF-driven process
  exit before force cleanup is considered;
- bundled sidecar path resolution matches `current_exe()` layout on Windows and
  non-Windows;
- both direct sidecar modes use `hide_console_window` on Windows and drain
  stderr without logging raw sidecar output;
- source contracts confirm the legacy Tauri shell transport/event-buffer code
  plus plugin registration/dependency are removed while `bundle.externalBin`
  remains configured;
- shutdown cancellation releases a mutex held by an in-flight long sidecar
  request before the graceful/force path takes its handle;
- a cancelled/partially written sidecar request taints the transport and skips
  graceful Stop;
- graceful cleanup for YouTube, sidecar, and Chrome starts concurrently;
- stalled Node-script and bundled-binary modes use the unified Tokio
  tree-termination/reap behavior;
- explicit Chrome shutdown and Drop fallback both kill/wait once;
- Chrome spawn/Job assignment and kill/wait execute through `spawn_blocking`;
- repeated `ExitRequested` events create one cleanup task, preserve the first
  requested exit code, and a `completed` phase permits final exit;
- a panicking or stalled cleanup task still exits through the four-second
  watchdog with the preserved code;
- one subsystem failure does not skip later cleanup;
- a source-level containment contract requires the already-owned raw child
  handle and rejects `OpenProcess`/`.pid()`-based assignment in that module;
- Windows Job Object closure terminates a real test descendant created after
  assignment as well as its direct test child;
- source-level contracts keep the Tauri lifecycle hook and forbid secret/path
  data in shutdown warnings.

Most process tests use injected fake launch/transport adapters; they do not
depend on installed `yt-dlp`, Node, or Chrome binaries. The Windows Job Object
integration test uses only an OS-provided inert process such as PowerShell or
`cmd.exe` to create a bounded parent/descendant fixture. Existing real-product
process smoke tests remain separate integration evidence.

The Tauri `RunEvent` callback remains a thin adapter over a testable shutdown
coordinator core. Tests inject the monotonic clock/watchdog scheduler and final
exit callback, so repeated-exit, preserved-code, panic, and hard-cap behavior
can be asserted without terminating the Rust test process.

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
  owned `yt-dlp.exe` or contained post-assignment helper descendant remains;
- start the Gemini sidecar and Extractum-owned CDP Chrome, close Extractum, and
  verify both owned processes terminate;
- in a packaged release, verify the bundled sidecar launches directly from the
  `current_exe()` directory without `tauri-plugin-shell`;
- close Extractum with an idle, untainted sidecar and verify logs/test telemetry
  show Stop ACK, stdin EOF closure, and normal process exit with no force-cleanup
  warning;
- close Extractum with no active external processes;
- confirm normal total exit delay is at most approximately three seconds plus
  small scheduling/process-reap overhead;
- force-terminate Extractum during an owned test process tree and confirm the
  Windows Job Object removes descendants created after assignment.

The normal target is three seconds plus small scheduling overhead; the hard
watchdog requires process exit by approximately four seconds even when cleanup
stalls.

Process identity is captured before exit from the owning handles/PIDs; tests do
not infer ownership by killing every process with a matching executable name.

## Documentation

Update:

- `docs/project.md`: external-process ownership and bounded shutdown;
- `docs/browser-providers-llm-troubleshooting.md`: sidecar/Chrome cleanup and
  recovery diagnostics, direct bundled spawn, and required `externalBin`
  packaging;
- `AGENTS.md`: require explicit kill/reap ownership for new child processes and
  prohibit dropping process futures as cancellation;
- `docs/value-registry.md` only if implementation introduces a persisted or API
  string value beyond this design (none is planned).

## Acceptance Criteria

- Timeout, caller cancellation, and app shutdown deterministically request
  termination of the direct owned `yt-dlp` process and normally confirm reap;
  a one-second stuck-reap degradation path is explicitly logged and detached.
- On Windows, raw-handle children and descendants created after Job Object
  assignment are terminated when their job closes, including on Extractum
  crash; pre-assignment descendants remain an explicit limitation.
- An idle, untainted Gemini sidecar receives graceful Stop before bounded force
  cleanup; a tainted transport skips directly to force cleanup.
- Release packaging retains `externalBin`, launches the bundled sidecar through
  Tokio, and no longer registers or depends on `tauri-plugin-shell`.
- Extractum-owned CDP Chrome receives contained-tree termination and is normally
  reaped; the hard watchdog remains the documented degradation path. Unrelated
  Chrome is untouched.
- New external operations are rejected after shutdown begins.
- Repeated exit requests do not duplicate cleanup or loop programmatic exit.
- The first requested exit code is preserved.
- Total graceful shutdown uses one three-second budget and the independent
  watchdog enforces an approximately four-second hard cap.
- Cleanup failures do not prevent other subsystem cleanup and do not leak
  sensitive process data.
- Graceful subsystem cleanup starts concurrently, and tainted sidecar transports
  skip protocol Stop.
- Automated tests, full Rust/TypeScript checks, current-state docs, and Windows
  release GUI verification describe and confirm the implemented behavior.
