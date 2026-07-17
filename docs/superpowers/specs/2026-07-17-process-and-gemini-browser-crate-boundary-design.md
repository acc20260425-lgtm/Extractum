# Process and Gemini Browser Crate Boundary Design

**Status:** Approved in conversation
**Date:** 2026-07-17

## Purpose

This specification fixes the just-in-time boundary for crate-roadmap phases 3
and 4. Phase 3 extracts shared operating-system process infrastructure into
`extractum-process`. Phase 4 then extracts a focused Gemini Browser engine
while keeping Tauri, SQL, Apalis, and application wiring in `extractum`.

The work is deliberately split into two independently planned and verified
slices. Phase 4 is planned only after Phase 3 is retained. Neither phase is a
literal folder move or a mass rewrite of existing consumers.

## Decision

The selected architecture is:

1. extract `external_process`, `child_process`, and `process_tree` into a
   shared lower-level crate;
2. preserve all application call sites through internal facade modules;
3. prepare Gemini Browser just in time by separating its Tauri and queue
   adapters from its portable engine;
4. extract the engine while leaving commands, application paths, database
   access, Apalis storage, and worker registration in the application crate.

The alternatives were rejected as follows:

- extracting Gemini first behind temporary process traits would create a
  disposable abstraction and a second migration when the shared process
  crate is eventually introduced;
- extracting only Gemini DTOs and obvious pure helpers would create a small
  library but leave the hot execution code in `extractum`, so it would not
  produce the intended focused development loop;
- moving the whole current `gemini_browser` directory would pull Tauri,
  SQLx, Apalis, migrations, and application state into the domain crate.

## Architectural Motivation

Phase 3 is retained for architectural reasons, not because a small focused
crate is expected to win its own performance comparison almost by definition.
It establishes the correct dependency direction for Phase 4, avoids temporary
process traits, and gives Gemini Browser, YouTube, and diagnostics one shared
owner for process admission, containment, and shutdown.

The focused process measurement is diagnostic evidence. The real performance
gate for Phase 3 is the application-shell regression cap from the focused-loop
policy. Phase 4, as a hot-domain extraction, must additionally pass the focused
domain retention gate.

## Fresh Evidence Snapshot

The just-in-time snapshot was taken on 2026-07-17 at commit `a04d49a9` with a
clean worktree.

### Process fan-in

| Module | References | Consumer files | Main consumers |
| --- | ---: | ---: | --- |
| `external_process` | 14 | 10 | Gemini Browser, seven YouTube files, `lib.rs` |
| `child_process` | 4 | 4 | diagnostics, Gemini Browser, two YouTube files |
| `process_tree` | 3 | 3 | Gemini Browser CDP/sidecar, YouTube runtime |
| `job_helpers` | 2 | 2 | YouTube jobs, Takeout state |

`job_helpers` is generic background-job state rather than OS-process
infrastructure. `job_helpers` stays app-side and is excluded from Phase 3.

### Recent co-change

Since 2026-06-01:

- 39 commits touched `gemini_browser`;
- 8 touched `external_process`;
- 4 touched `process_tree`;
- 1 each touched `child_process` and `job_helpers`;
- Gemini Browser co-changed once with `external_process` and twice with
  `process_tree`;
- no Gemini Browser commit co-changed with `child_process` or `job_helpers`.

These numbers do not argue that the process modules belong inside Gemini
Browser. Their wide fan-in and lower-level responsibility justify a shared
crate despite low direct co-change.

### Gemini Browser JIT inventories

- The module contains approximately 6,770 lines and 94 Rust tests.
- There are 54 references to `gemini_browser::` outside its directory.
- `jobs.rs` contains exactly two `db::get_pool` calls: enqueue and worker
  setup.
- Gemini Browser does not consume `sources::test_support`, so the roadmap's
  deferred fixture-ownership trigger does not fire in this phase.

## Target Dependency Structure

```text
extractum
  -> extractum-gemini-browser
       -> extractum-process
       -> extractum-core
  -> extractum-process
  -> extractum-core
```

There are no reverse dependencies on `extractum`. The application crate owns
Tauri integration, database access, migrations, Apalis registration, and
cross-domain orchestration.

## Phase 3: `extractum-process`

### Files and dependencies

```text
src-tauri/crates/extractum-process/
  Cargo.toml
  src/
    lib.rs
    external_process.rs
    child_process.rs
    process_tree.rs
```

The crate may depend on `tokio`, `parking_lot`, `anyhow`, and target-specific
`windows-sys`. It does not need `extractum-core` today. Tauri, SQLx, Apalis,
Gemini Browser, YouTube, and the application crate are forbidden dependencies.

`lib.rs` exposes the three named modules without root glob exports:

```rust
pub mod child_process;
pub mod external_process;
pub mod process_tree;
```

### Public surface

Existing cross-module `pub(crate)` items become public only where a current
consumer requires them.

`external_process` exposes the current admission, shutdown coordinator,
watchdog, timing, callback, warning, and permit types/functions, including
`ExternalProcessShutdownState`, `ShutdownTiming`, `ShutdownStart`,
`ShutdownRun`, `AdmissionPermit`, `system_monotonic_clock`, and
`os_thread_watchdog_scheduler`.

`child_process` exposes `hide_console_window` and the cfg-controlled
`CREATE_NO_WINDOW` constant.

`process_tree` exposes `ProcessTreeGuard` and its existing `new`,
`assign_tokio`, `assign_std`, and `terminate` operations. Windows job-object
behavior and the non-Windows no-op implementation remain unchanged.

Test-only seams remain under `#[cfg(test)]` and do not become production API.
The implementation plan must enumerate the exact visibility changes before
editing.

### Application facades

Consumers are not mass-rewritten in the extraction slice. `extractum` keeps
these internal facade modules:

```rust
mod external_process {
    pub(crate) use extractum_process::external_process::*;
}

mod child_process {
    pub(crate) use extractum_process::child_process::*;
}

mod process_tree {
    pub(crate) use extractum_process::process_tree::*;
}
```

The globs are permitted only inside these private compatibility facades. The
new crate's public root remains curated and does not use glob re-exports.

### Phase 3 tests and retention

The current 20 tests move with their implementations:

- `external_process`: 12;
- `process_tree`: 7;
- `child_process`: 1.

The post-extraction inventory must contain all 20 in `extractum-process`, none
duplicated in the application crate, and no reduction in total workspace test
count. Focused process tests, full workspace tests/checks, Windows process-tree
behavior, and the non-Windows build surface must pass.

Phase 3 is retained when correctness passes and the application shell probe
regresses by no more than both 5% and 0.5 seconds. The focused process metric
is recorded but is not the architectural justification. One predeclared full
remeasurement is allowed for a marginal shell result; a confirmed violation
blocks Phase 4 and requires analysis or rollback.

## Phase 4: `extractum-gemini-browser`

### Target files

```text
src-tauri/crates/extractum-gemini-browser/src/
  lib.rs
  types.rs
  run_log.rs
  sidecar_launch.rs
  execution_state.rs
  sidecar_engine.rs
  cdp_chrome.rs
  job_runtime.rs
  job_execution.rs

src-tauri/src/gemini_browser/
  mod.rs
  commands.rs
  paths.rs
  jobs.rs
```

The application `gemini_browser` module remains the compatibility and
integration facade, so existing external `crate::gemini_browser::*` paths are
not rewritten in this slice.

### Current-file disposition

| Current file | Destination |
| --- | --- |
| `types.rs` | move to engine |
| `run_log.rs` | move to engine; receive pure `safe_run_id` from `paths.rs` |
| `sidecar_launch.rs` | move to engine |
| `cdp_chrome.rs` | move to engine and depend on `extractum-process` |
| `state.rs` | split; portable state moves, Tauri path resolution stays app-side |
| `sidecar.rs` | move engine behavior; replace `AppHandle` lookup with explicit shutdown state |
| `jobs.rs` | split between engine runtime/execution and app storage/worker adapter |
| `paths.rs` | remain app-side except `safe_run_id` |
| `commands.rs` | remain app-side |
| `mod.rs` | remain an app facade over adapters and engine exports |

### Engine dependencies

The engine may depend on `extractum-core`, `extractum-process`, serde,
serde_json, tokio, tokio-util, parking_lot, time, url, and protocol-level
libraries required by the moved code.

It must not depend on Tauri, any Tauri plugin, SQLx, Apalis, Apalis SQLite,
Tower worker registration, application migrations, `crate::db`, or
`sources::test_support`.

## Execution State and Process Engines

`execution_state.rs` owns the active run ID, active `CancellationToken`,
sidecar-tainted flag, sidecar and CDP process handles, provider status
snapshot, startup-reconciliation guard, and run-status to provider-status
mapping.

The state no longer accepts `AppHandle`. App-side code resolves the browser
profile directory and passes the resulting value to state initialization or
snapshot access.

`sidecar_engine.rs` owns launch-mode execution, process containment, JSONL
transport, request IDs, stderr draining, response decoding, status/open/
resume/send/stop behavior, cancellation selection, and tainted-process
handling. The app obtains `ExternalProcessShutdownState` from Tauri state and
passes it explicitly. No temporary spawner trait is introduced.

`cdp_chrome.rs` keeps Chrome discovery, launch specs, containment, endpoint
polling, start result, and shutdown. It depends directly on
`extractum_process::process_tree::ProcessTreeGuard`.

## The `jobs.rs` Ownership Boundary

The governing rule is: Apalis owns delivery of a job; the engine owns the
Gemini run lifecycle.

### App-side storage and worker adapter

The following stay in application `jobs.rs`:

- queue name and poll configuration;
- Apalis storage access and migrations;
- storage construction;
- task construction and idempotency key;
- SQL idempotency query;
- enqueue error mapping;
- both `db::get_pool` calls;
- `enqueue_gemini_browser_job`;
- Apalis `WorkerBuilder`, concurrency, and `TimeoutLayer`;
- worker registration;
- the thin Apalis handler that resolves app state/paths and calls engine;
- conversion from Apalis queue observations to domain run status.

### Engine job runtime

`job_runtime.rs` owns `GeminiBrowserJobRuntime`, its waiter map, queued cancel
set, worker status channel, waiter/execution/hard-guard timeout policy,
readiness decisions, waiter completion, cancel flags, worker lifecycle, and
test constructors.

The hard-guard duration is engine policy, while the app adapter applies it to
the Tower layer used by Apalis registration.

### Engine job execution

`job_execution.rs` owns `GeminiBrowserArtifactMode`, `GeminiBrowserJob`, job to
run-request conversion, queued and active cancellation orchestration, startup
run-log reconciliation, worker-entry reconciliation, execution timeout,
completion and timeout transitions, result constructors, waiter completion,
cancel cleanup, run-log lookup, and sidecar fallback behavior.

The app handler constructs a concrete context containing engine/runtime state,
shutdown state, and already-resolved directories, then calls one engine entry
point. The context contains no `AppHandle`, SQL pool, or Apalis type.

### Cancellation ownership

Cancellation has two intentional levels:

- before execution, `GeminiBrowserJobRuntime.cancelled_runs` records the run
  ID; when Apalis later delivers the job, engine execution records a cancelled
  run, completes the waiter, clears the marker, and acknowledges delivery;
- during execution, `GeminiBrowserState` owns the `CancellationToken`; sidecar
  requests select on it, mark a cancelled process tainted, and discard/stop
  that process.

The token is not owned by Tauri and is not persisted in SQL. The extraction
does not add manual deletion of queued Apalis rows.

### Persistence ownership

| State | Owner |
| --- | --- |
| queue delivery (`Pending`, `Running`, `Done`, and related states) | app-side Apalis/SQLite |
| domain run (`Queued`, `Running`, `Cancelled`, `Failed`, `Ok`, and related states) | engine run log |
| current provider snapshot | engine execution state |

Only engine code performs domain-run transitions: queued creation, running
entry, terminal completion, cancellation, timeout, interrupted-worker
recovery, and startup reconciliation. The storage adapter does not write
`GeminiBrowserRunStatus`.

Startup reconciliation accepts an optional domain-level status lookup. `None`
means the current degraded run-log-only behavior; tests or a future app adapter
may supply a lookup returning `Option<GeminiBrowserRunStatus>`. Apalis types,
table names, and state strings do not cross into the engine.

## Phase 4 Implementation Shape

Phase 4 is not one mechanical commit.

1. **Characterization and internal split:** remove `AppHandle` from portable
   state/sidecar APIs, characterize both cancellation paths and timeout policy,
   split storage concerns from runtime/execution inside the app module, and
   preserve behavior under focused `extractum` tests.
2. **Engine extraction:** create the crate, move the prepared modules and
   tests, preserve the app facade, and update only the internal adapter calls.
3. **Integration and measurement:** run focused engine checks, app dependent
   checkpoints, workspace gates, release no-bundle build, process shutdown
   smokes, and write literal verification evidence.

No disposable process abstraction or mass external call-site migration is
allowed in these steps.

## Test Inventory and Contracts

The baseline contains 94 Gemini Browser tests:

- `cdp_chrome`: 8;
- `commands`: 17;
- `jobs`: 36;
- `run_log`: 7;
- `sidecar`: 7;
- `sidecar_launch`: 6;
- `state`: 7;
- `types`: 6.

The command tests remain app-side. Tests for types, run log, sidecar launch,
CDP, sidecar engine, and portable state move with their owners. Before editing
`jobs.rs`, the implementation plan must classify all 36 job tests by exact
name: runtime/execution tests move to the engine; SQL/Apalis/storage tests stay
in the app. A complete rename/move map covers all 94 baseline names. Each old
name must either remain or map to one declared new name; copying a test into
both packages is a failure.

Source contracts enforce:

- workspace membership and curated crate roots;
- moved-not-copied implementations and tests;
- the three Phase 3 facades and the Gemini app facade;
- unchanged external consumer paths within each extraction slice;
- `job_helpers` remaining app-side;
- forbidden dependencies/imports in both new crates;
- both `db::get_pool` calls remaining app-side;
- `CancellationToken` and domain transitions residing in engine code;
- SQL/Apalis storage functions residing only in the app adapter.

## Measurement and Retention

Every implementation plan records fresh baselines rather than reusing earlier
measurements. The focused-loop specification is normative for environment,
warm-up, five-sample medians, byte restoration, failure classification, and
threshold calculation.

Phase 4 compares the same logical engine edit before and after extraction:

- before: inert edit in the Gemini source inside `extractum`, followed by a
  focused `-p extractum` check;
- after: the same edit in the moved source, followed by a focused
  `-p extractum-gemini-browser` check.

Retention requires both at least 25% and at least 2.0 seconds focused median
improvement. The retained application shell probe may regress by no more than
both 5% and 0.5 seconds. Focused success does not prove downstream integration;
the app package checkpoint and full workspace gates remain mandatory.

## Verification

Each Rust task uses the narrow owning-package RED/GREEN test and focused check
required by the focused-loop policy. Each completed phase runs at least:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Phase 4 additionally requires the complete app Gemini group, complete engine
tests, `npm.cmd run check`, a release
`npm.cmd run tauri -- build --no-bundle`, startup smoke, sidecar/CDP smoke, and
shutdown smoke with an active external process.

MSI/WiX remains excluded because its failure predates these changes; the
verification record states this explicitly.

## Failure and Rollback

- Infrastructure failures invalidate the affected measurement session.
- Baseline failures stop implementation until a valid baseline is restored.
- Candidate correctness failures block measurement and retention.
- A Phase 3 shell regression blocks Phase 4.
- A Phase 4 candidate that fails either focused retention or shell regression
  records an honest negative result and is not retained merely because its
  architecture is attractive.
- A failed workspace or smoke gate is never waived by focused performance.
- Rollback removes only the unretained candidate slice and preserves its
  design and verification evidence.

## Resulting Plans

This approved design produces two separate implementation plans:

1. `extractum-process` extraction;
2. Gemini Browser internal preparation and focused engine extraction.

The second plan is written only after Phase 3 is implemented, verified, and
retained.

## Non-Goals

- No mass rewrite of YouTube, diagnostics, prompt-pack, or application import
  paths in an extraction slice.
- No move of `job_helpers` into `extractum-process`.
- No Tauri, SQLx, or Apalis dependency in the engine crate.
- No deletion or rewriting of existing migrations.
- No queue-backend redesign or manual cancellation of Apalis rows.
- No universal fixture crate while `sources::test_support` is not consumed by
  the candidate domain.
- No MSI/WiX repair in these phases.
- No weakening of workspace completion gates.
