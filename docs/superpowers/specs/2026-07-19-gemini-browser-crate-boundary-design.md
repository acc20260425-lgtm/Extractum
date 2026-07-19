# Gemini Browser Crate Boundary Design

**Status:** Implemented and retained; [verification](../verification/2026-07-19-extractum-gemini-browser-extraction.md)
**Date:** 2026-07-19

**Roadmap authority:**
[`2026-07-17-crate-roadmap.md`](2026-07-17-crate-roadmap.md)

This specification supersedes only the Phase 4 architecture, dependency,
measurement, and execution clauses of the historical
[`2026-07-17 process and Gemini Browser boundary design`](2026-07-17-process-and-gemini-browser-crate-boundary-design.md).
It does not rewrite that document's Phase 3 record or historical evidence.
Phase 3 remains closed and not retained under the
[`2026-07-19 cancellation disposition`](../verification/2026-07-19-extractum-process-reapplication-cancellation.md).
The focused-loop specification and crate roadmap remain the current timing
authorities.

## Purpose

Phase 4 extracts the portable Gemini Browser domain engine into
`extractum-gemini-browser` without recreating the canceled
`extractum-process` boundary. The crate owns stable Gemini Browser behavior;
the application owns operating-system processes, Tauri, persistence adapters,
and application wiring.

The result must make repeated domain work independently checkable while
preserving the current IPC surface, serialized values, run-log format,
database behavior, and external Rust consumer paths.

## Decision

The selected boundary is a permanent domain-level browser-execution port:

1. `extractum-gemini-browser` owns DTOs, protocol rules, run-log behavior,
   portable runtime state, submission and status decisions, job lifecycle,
   cancellation, timeout classification, and startup reconciliation;
2. `extractum` owns Tauri commands and state registration, application path
   resolution, SQLx and Apalis storage, worker registration, and every
   concrete sidecar or Chrome process handle;
3. a narrow `BrowserExecutor` interface exposes browser operations such as
   `status`, `open`, `resume`, `send`, and `stop` without exposing how a
   process is spawned or contained;
4. the existing private `crate::gemini_browser` facade preserves current
   consumer paths during the extraction slice.

The port is not a generic process abstraction and is not a substitute
`extractum-process` crate. PID values, `Child`, `Command`, stdin/stdout handles,
`ProcessTreeGuard`, shutdown-admission types, `windows-sys`, and process-tree
operations never cross the new crate's public API.

The historical rejection of a temporary process trait assumed that a retained
shared process crate was imminent. That premise is void. A permanent
Gemini-domain port is now the stable boundary because concrete process
ownership deliberately remains in the application.

## Fresh Evidence Snapshot

The current snapshot was taken on 2026-07-19 at `8e4a3bd1` with a clean
worktree.

- `src-tauri/src/gemini_browser` contains approximately 6,770 lines and 94
  Rust tests.
- Since 2026-06-01, 39 commits touched the module. Under the fresh
  Rust-domain classification, which excludes `lib.rs`, manifests, migrations,
  documentation, frontend files, and other non-domain shell changes, 28 of 39
  commits (71.8%) touched no other categorized Rust domain.
- Raw joint-touch counts in the same window are `prompt_packs` 8,
  `external_process` 1, `process_tree` 2, `child_process` 0, `db` 1,
  `apalis_jobs` 1, and `job_helpers` 0. Some joint touches are broad mechanical
  formatting commits and are not interpreted as ownership evidence by
  themselves.
- Current external Rust consumers are the application root, Apalis integration
  tests, and the `prompt_packs` completion transport, DTO, Gemini stage,
  runtime/configuration, and YouTube-summary integration modules.
- No retained functional Gemini Browser change landed after the historical
  2026-07-17 snapshot; the only intervening ownership changes were the
  temporary process extraction and its exact revert.

The older roadmap's 59% solo / 7 `prompt_packs` figures are retained as a
historical snapshot produced by a different repository-wide module-bucketing
method. The fresh 71.8% / 8 figures above govern this just-in-time design and
do not reinterpret the historical calculation.

## Target Dependency Structure

```text
extractum
  |-- Tauri commands and AppHandle state
  |-- SQLx / Apalis adapters and worker registration
  |-- concrete sidecar and CDP process ownership
  `-- extractum-gemini-browser
        |-- domain DTOs and protocol
        |-- run log and portable runtime state
        `-- job lifecycle through BrowserExecutor
```

There is no dependency from `extractum-gemini-browser` back to `extractum` and
no dependency on `extractum-process`. The application is the only owner of
concrete OS-process infrastructure.

The expected production dependency roots are `serde`, `serde_json`,
`parking_lot`, `tokio`, `tokio-util`, `time`, and `url`. `tempfile` and the
minimum Tokio test features are expected only for tests. The implementation
plan must derive exact features from the final moved uses before creating the
manifest; it must remove an expected root if the prepared code does not use it
and must justify any additional root explicitly.

`extractum-core` is not prescribed merely to match the roadmap diagram. The
new crate defines its own non-serialized domain error and should depend on
core only if the prepared implementation has an actual retained core use.
Before generating either manifest, the implementation plan must inventory
`types.rs`, `run_log.rs`, and every moved fragment for direct or facade-backed
`extractum-core` API use, including `error`, `time`, `media_metadata`, and
`compression`. The current snapshot finds only app-facade `AppError` /
`AppResult` use in `run_log.rs` and no other core use in those two files; the
pre-manifest inventory must reprove that fact or name and justify the exact
core dependency before Cargo compilation becomes the first detector. Resolve
any mismatch in the preparation checkpoint; do not patch the manifest or move
source ad hoc inside the mechanical extraction checkpoint.

Forbidden dependencies include Tauri and its plugins, SQLx, Apalis,
Apalis-SQLite, Tower worker layers, `extractum-process`, `windows-sys`, and any
application crate. `reqwest` remains app-side with the concrete CDP endpoint
probe unless preparation proves a portable engine use that this design does
not currently identify.

## Ownership Boundary

### Crate-owned domain behavior

`extractum-gemini-browser` owns:

- all public Gemini Browser DTOs and their existing serde representation;
- run-ID validation and the run-log filesystem state machine;
- JSONL envelope/response encoding, decoding, request correlation, and resume
  response classification;
- pure sidecar launch-mode selection and pure CDP endpoint/launch-spec
  validation;
- active-run identity, cancellation tokens, cached provider status, and
  startup-reconciliation admission without `AppHandle`;
- worker readiness, waiter registration/completion, queued cancellation flags,
  and timeout policy;
- submission ordering and duplicate-run decisions;
- worker-entry reconciliation, delivered-job execution, terminal result
  construction, timeout/cancellation classification, and run-log transitions;
- snapshot-in / typed-actions-out startup reconciliation.

### Application-owned integration

`extractum` retains:

- every `#[tauri::command]`, `AppHandle`, `State`, event emission, path opener,
  and application-data-directory lookup;
- `paths.rs` directory creation and app-specific path resolution, except the
  pure run-ID validator;
- SQLx pool access, Apalis schema setup, storage construction, task building,
  idempotency queries, queue configuration, queue-state decoding, and worker
  registration;
- sidecar and CDP executable discovery, environment lookup, concrete spawn,
  stdin/stdout/stderr ownership, process containment, kill, reap, drop, and
  shutdown coordination;
- process-transport taint state and restart/discard decisions tied to those
  concrete handles;
- live CDP network probing and application error/telemetry adaptation;
- the thin Apalis delivery handler and Tauri compatibility facade.

Apalis owns durable delivery; the new crate owns the Gemini run lifecycle.
The app adapter may persist or acknowledge a typed outcome, but it must not
duplicate domain transition rules.

## Current-File Disposition

| Current file | Phase 4 disposition |
| --- | --- |
| `types.rs` | Move to the crate byte-for-byte apart from import paths and curated exports. |
| `run_log.rs` | Move to the crate; replace `AppError` with the domain error and receive pure `safe_run_id`. |
| `sidecar_launch.rs` | Move pure path construction and launch-mode decisions; keep current-executable/environment resolution app-side. |
| `sidecar.rs` | Split: protocol codec and response classification move; concrete process, transport handles, stderr ownership, and shutdown stay app-side. |
| `cdp_chrome.rs` | Split: launch specification and endpoint validation move; discovery, network polling, child ownership, containment, and shutdown stay app-side. |
| `state.rs` | Split: portable execution/status/cancellation state moves; sidecar/CDP handles and Tauri emission stay in an app-owned composite state. |
| `jobs.rs` | Split: runtime and domain lifecycle move; SQLx/Apalis storage, queue mapping, worker construction, and the thin delivery adapter stay app-side. |
| `commands.rs` | Keep Tauri wrappers app-side; move pure submission/status/reconciliation decisions behind crate entry points. |
| `paths.rs` | Stay app-side except for pure run-ID validation. |
| `mod.rs` | Remain a private explicit compatibility facade over app adapters and curated crate exports. |

The file split is prepared and characterized before the physical move. The
preparation checkpoint and extraction checkpoint belong to one Phase 4 slice;
the actual cross-crate move remains mechanical and does not authorize an
unrelated service/store refactor.

## Public API and Data Flow

The crate root uses named modules and explicit re-exports. Public glob exports
are forbidden. Its stable surface is organized around:

- existing Gemini Browser DTOs;
- `GeminiBrowserError` and `GeminiBrowserResult<T>`;
- `GeminiBrowserJobRuntime`;
- `BrowserExecutor`;
- a delivered-job input/context and typed `DeliveryOutcome`;
- submission and run-log operations required by the app facade;
- reconciliation snapshots, normalized queue states, and typed actions;
- pure protocol, launch, and CDP validation types required by the app adapter.

The main execution flow is:

1. the app worker reads an Apalis job and converts it to the crate-owned
   delivered-job DTO;
2. it resolves app paths and supplies the portable runtime, a
   `BrowserExecutor`, and a typed status observer;
3. `execute_delivered_job` validates entry state, owns run-log transitions,
   enforces cancellation and timeout policy, and invokes browser-level
   operations;
4. it returns a `DeliveryOutcome` that distinguishes completion,
   already-terminal acknowledgement, cancellation, timeout, and failure;
5. the app maps that outcome to Apalis acknowledgement/failure and Tauri event
   delivery without reinterpreting strings.

The status observer is a typed notification seam, not a process capability.
It allows the application to emit the current Tauri event while the crate
remains unaware of `AppHandle`.

For startup reconciliation, the app normalizes persisted queue observations
before calling the crate. The crate accepts a run/queue snapshot and returns
typed reconciliation actions. The app applies the required persistence and
notification operations. Apalis state strings, SQL table names, and database
types never cross the boundary.

## BrowserExecutor Boundary

`BrowserExecutor` exposes only Gemini Browser concepts:

- obtain provider status;
- open or resume a browser session;
- send one `GeminiBrowserRunRequest` and return a
  `GeminiBrowserRunResult`;
- stop the active browser session.

The implementation plan may choose generic async methods or explicit future
return types, but it must not add a general-purpose spawning API. The port
must not expose executable paths, environment blocks, process IDs, child
handles, pipe handles, job objects, process-tree guards, or OS error types.

The app adapter remains responsible for process admission and shutdown. It
may use crate-owned protocol/codec helpers internally, but it stores and
destroys all concrete resources itself.

### Cancellation across the port

The crate does not pass its `CancellationToken` into `BrowserExecutor`.
Instead, it selects the in-flight executor future against its own token. When
cancellation wins, the crate drops/abandons that future, calls
`BrowserExecutor::stop` with a typed cancellation reason, and awaits the
idempotent stop operation before completing the cancelled outcome.

The app executor marks any abandoned concrete transport tainted before reuse,
then stops/reaps the owned sidecar or CDP resource. Repeated stop requests are
safe. A stop error is preserved as diagnostic evidence but cannot turn a
cancelled run into success. A response that completes after cancellation is
ignored, and the terminal cancelled run-log entry and status snapshot cannot
be overwritten by that late success. Timeout uses the same ownership path
with a distinct typed timeout reason.

## Error Handling

The crate uses a typed, non-serialized domain error. It distinguishes at least
validation, not-found, conflict, protocol/transport failure, browser failure,
timeout, cancellation, and internal invariant failure. `DeliveryOutcome`
represents expected terminal job states directly rather than encoding them in
error messages.

The app owns the explicit mapping from domain errors to the existing
`AppErrorKind` and outward messages. The extraction does not change current
IPC error serialization. Timeout and cancellation decisions must no longer
depend on matching message text such as `"timed out"`.

Typed classification must preserve the existing outward text byte-for-byte:
the waiter error is `"Gemini Browser job timed out waiting for worker result"`,
the execution result uses `"Gemini Browser job timed out after {seconds}s"`,
and cancellation uses `"Cancelled"`. Characterization must pin the exact
`GeminiBrowserRunResult` fields, serde JSON, and persisted pretty-JSON run-log
bytes for both timeout paths and for queued and active cancellation. Tests may
normalize only pre-existing nondeterministic timestamp fields; status, message,
elapsed-time, and other stable fields remain exact. Replacing the string-based
detector is an internal classification change, not permission to rewrite user-
visible messages or run-log representation.

No new serialized `status`, `state`, `kind`, `mode`, or related string value is
introduced. If implementation discovers that a new serialized value is
actually necessary, it is outside this approved design and requires the
normal `docs/value-registry.md` review before use.

## Visibility and Compatibility

Existing public DTOs keep their names and serde representations. The private
application facade continues to expose the current
`crate::gemini_browser::*` paths used by `lib.rs`, `apalis_jobs`, and
`prompt_packs`; those consumers are not mass-rewritten in this slice.

The only existing internal item classes permitted to widen across the new
crate edge are:

- `GeminiBrowserJobRuntime`, `GeminiBrowserArtifactMode`, and
  `GeminiBrowserJob`, plus the queue receipt currently represented by
  `QueuedGeminiBrowserJob`;
- `create_queued_run`, `finish_run`, `list_runs`, `mark_running`, `read_run`,
  and `recorded_run_dir`;
- `safe_run_id`;
- `GeminiBrowserSidecarLaunch`, `GeminiBrowserBuildProfile`,
  `GEMINI_BROWSER_SIDECAR_NAME`, `bundled_sidecar_path`,
  `dev_sidecar_script`, and `resolve_launch_mode`;
- `ChromeCdpLaunchSpec`, `build_chrome_cdp_launch_spec`,
  `start_chrome_result`, and the prepared pure endpoint-validation API;
- prepared high-level submission, status, execution, reconciliation, codec,
  and executor entry points required by the app adapter.

Internal maps, cancellation tokens, process-taint mutation, worker-status
channels, test constructors, codec parsing helpers, and result-construction
helpers remain private behind those operations. The implementation plan must
turn this class-level list into an exact symbol inventory before changing
visibility. Any additional widening is a design deviation and requires review.

## Persistence and Lifecycle Rules

| State | Owner |
| --- | --- |
| Apalis delivery and queue persistence | app-side Apalis/SQLite adapter |
| Domain run state and run-log files | `extractum-gemini-browser` |
| Worker waiters/readiness and queued cancellation flags | `extractum-gemini-browser` |
| Active run, cancellation token, provider snapshot | `extractum-gemini-browser` |
| Concrete sidecar/CDP handles, transport taint, and process shutdown | app-side process adapter |

The current ordering invariants remain unchanged:

- queued run-log creation precedes durable enqueue handoff;
- enqueue failure removes the waiter and produces the existing terminal
  failed run;
- a queued cancellation is recorded until later delivery acknowledges it;
- active cancellation requests browser stop and prevents a later success from
  overwriting the terminal cancellation;
- timeout clears active/cancelled state and is reported as a typed timeout;
- startup reconciliation never fabricates an Apalis status when queue
  inspection is unavailable.

## Implementation Shape

Phase 4 is one implementation plan and one retention decision with three
separately verified checkpoints:

1. **Characterization and seam preparation:** add or identify exact tests for
   transition ordering, cancellation, timeout, reconciliation, status
   publication, and process adapter behavior; introduce the permanent
   `BrowserExecutor` seam inside the app without moving files. Before replacing
   `is_worker_timeout_result`, record RED/GREEN characterization for the exact
   waiter-timeout, execution-timeout, queued-cancellation, and
   active-cancellation result fields and persisted JSON bytes declared above.
2. **Mechanical extraction:** create the workspace member, move prepared
   domain modules/tests, split the mixed files according to the table above,
   and preserve the private app facade and external consumer paths.
3. **Integration and evidence:** run both package checkpoints, the app
   dependent checkpoint, workspace and release gates, process smokes, and the
   small advisory timing procedure; record a verification document.

The preparation checkpoint must remain behavior-preserving and scoped to the
cross-crate seam. It does not authorize a frontend change, queue redesign,
database migration, generalized process service, or broad import cleanup.

## Test Inventory and Source Contracts

The baseline inventory is 94 Gemini Browser tests:

- `cdp_chrome`: 8;
- `commands`: 17;
- `jobs`: 36;
- `run_log`: 7;
- `sidecar`: 7;
- `sidecar_launch`: 6;
- `state`: 7;
- `types`: 6.

The approved disposition is 75 tests in `extractum-gemini-browser` and 19 in
`extractum`. The following 19 tests remain app-side:

- CDP process/readiness: `explicit_shutdown_kills_and_reaps_the_owned_child_once`,
  `drop_falls_back_to_owned_child_shutdown`,
  `shutdown_does_not_claim_or_kill_an_already_exited_child`,
  `shutdown_reaps_when_the_child_has_already_exited_during_kill`,
  `wait_for_cdp_endpoint_accepts_json_version_response`, and
  `wait_for_cdp_endpoint_reports_unreachable_endpoint`;
- sidecar process transport:
  `stderr_drain_consumes_sidecar_output_concurrently`;
- app process state:
  `cancelled_run_marks_the_sidecar_transport_tainted`;
- Apalis/SQL job integration:
  `apalis_storage_uses_shared_main_extractum_db_identity`,
  `apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job`,
  `apalis_storage_preserves_existing_sqlx_migration_history_table`,
  `apalis_storage_shares_extractum_db_without_locking_app_pool`,
  `enqueue_duplicate_run_id_returns_conflict`,
  `enqueue_persists_job_before_worker_startup`,
  `worker_picks_up_job_quickly_after_idle`,
  `restart_worker_processes_pending_job_after_runtime_restart`,
  `apalis_sqlite_status_probe_documents_actual_status_values`,
  `gemini_browser_jobs_are_built_with_one_total_attempt`, and
  `failed_gemini_browser_job_is_not_retried`.

The other 75 baseline tests move with the domain logic. That set consists of
all 6 `types`, 7 `run_log`, 6 `sidecar_launch`, and 17 `commands` tests; the 2
pure CDP launch/validation tests; 6 generic sidecar protocol tests; 6 portable
state tests; and the remaining 25 `jobs` tests. Before editing, the
implementation plan must materialize the complete 94-name map and prove that
each name occurs exactly once after the split. New characterization tests may
increase the total, but no baseline test may be dropped, silently renamed, or
copied into both packages.

A dedicated Vitest source-boundary contract must enforce:

- workspace membership and the intended single app-to-domain dependency edge;
- the exact allowed Cargo dependency roots and features;
- the corresponding `src-tauri/Cargo.lock` package and dependency-edge update;
- a curated crate root with no public glob export;
- explicit private app-facade re-exports and unchanged external consumer
  paths;
- moved-not-copied implementation and test ownership;
- the complete 94-name baseline move map;
- no Tauri, plugin, SQLx, Apalis, Tower worker, application DB, or migration
  import in the new crate;
- no `extractum-process`, `external_process`, `child_process`,
  `process_tree`, `ProcessTreeGuard`, `windows-sys`, `std::process::Child`,
  `std::process::Command`, or `tokio::process` use in the new crate;
- no `AppHandle`, application `AppError`, Apalis state string, or SQL table
  name in the crate API;
- concrete process handles and spawn/containment/shutdown functions remaining
  app-side;
- the app adapter not constructing or storing the crate-owned
  `CancellationToken`;
- SQL/Apalis storage and worker-registration functions remaining app-side;
- domain run-transition helpers residing only in the new crate;
- removal of app-side message-prefix timeout detection, including the current
  `is_worker_timeout_result` path;
- exact preservation tests for both legacy timeout messages, the cancellation
  message, their serialized result fields, and persisted run-log JSON;
- test-only helpers remaining non-public.

The test-map assertion must use a frozen set of all 94 baseline names, parse
the post-split Rust sources in both packages, prove 94 unique matches and one
occurrence per name, and compare the app-owned subset with the exact 19-name
list above. Checking only the numeric `75 / 19` totals is insufficient.

Existing workspace-member allowlists in repository contracts are updated in
the same implementation slice.

## Rust Verification Loops

The implementation plan must contain the repository-required `## Rust
Verification Loops` section. During preparation, code still owned by the app
uses exact non-empty tests and focused checks against `-p extractum`. After a
move, the same behavior uses `-p extractum-gemini-browser`. Every public API
change adds the immediate dependent checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

The new package checkpoint is:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
```

The end-of-slice completion gates remain:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

The portable engine also retains the Linux package check from the historical
Phase 4 proposal:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --target x86_64-unknown-linux-gnu
```

Release evidence always includes the following fixed smoke set because this
slice necessarily changes the sidecar, CDP, state, and executor seams:

```powershell
npm.cmd run smoke:gemini-browser-sidecar:node
npm.cmd run build:gemini-browser-sidecar
npm.cmd run check:gemini-browser-sidecar-binary
npm.cmd run smoke:gemini-browser-sidecar:binary
```

The CDP negative path is also mandatory. In a `try/finally` block, set
`EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT=http://127.0.0.1:65530`, run:

```powershell
npm.cmd run smoke:gemini-browser-sidecar:resume:node -- --expect-manual-action=start_chrome_cdp
```

and remove the environment variable in `finally`. The expected result is the
existing `needs_manual_action` / `start_chrome_cdp` response.

Finally run `npm.cmd run tauri -- build --no-bundle`, start the resulting app,
request Gemini Browser status so that the app owns an active managed sidecar,
close the app normally, and confirm that the app and its owned sidecar have
both exited. This is a visible correctness smoke, not an automated timing
probe; it must not navigate to Gemini or modify Google account state.

These checks are mandatory rather than selected post hoc. The plan must reuse
the existing commands and ordinary app lifecycle. It must not build a new
quiet-window, Job Object, or process-scanning measurement harness. A helper or
launch-control failure is infrastructure; a completion failure requires a
confirmed application or owned-sidecar behavior failure. MSI/WiX remains
excluded.

## Advisory Compile-Time Measurement

The current
[`focused Rust loop`](2026-07-17-focused-rust-loop-design.md) is normative.
Phase 4 uses the same logical inert domain edit before and after extraction:

- before: edit the selected Gemini source while it belongs to `extractum` and
  run the focused `-p extractum --all-targets` check;
- after: apply the same edit to the moved source and run the focused
  `-p extractum-gemini-browser --all-targets` check.

Each state receives one discarded warm-up and three recorded samples. Record
raw values and the median, restore probe bytes in a `finally` path, and perform
one SHA-256 source check plus one clean-worktree check after each complete
series.

Do not add a shell A/B series, quiet-window coordinator, process scanner, Job
Object, Defender or power-profile capture, stability rule, automatic retry,
per-sample ledger, or cumulative timing ledger. A failed or incomplete timing
series produces no performance conclusion after source restoration; timing
alone never rejects, reverts, or retains the slice.

Record the duration already emitted by the one successful mandatory
end-of-slice workspace check. Two adjacent completed crate-extraction slices
whose ordinary workspace-check results are each at or above 15,000 ms trigger
a separate owner-approved performance investigation. Phase 4 alone cannot
trigger that two-slice rule, and the result is not a Phase 4 cap.

The historical 25% / 2.0-second focused gate, 2,000 ms / 20% shell cap, and
cumulative ledger are inactive and must not reappear in the implementation
plan.

## Failure and Rollback

- A baseline correctness failure stops implementation until the retained
  workspace is green.
- A candidate package, app-dependent, workspace, portability, build, startup,
  or fixed sidecar/CDP/shutdown smoke failure keeps the slice incomplete.
- A measurement failure stops only the advisory measurement after exact probe
  restoration; it is not converted into a candidate correctness failure.
- A process-smoke harness failure is infrastructure until the helper itself is
  proven sound; only confirmed application behavior is a candidate failure.
- Timing regression is recorded and reported but never triggers automatic
  rollback.
- If correctness cannot be restored, rollback removes only the unretained
  Phase 4 candidate and preserves the design and verification evidence.

## Acceptance Criteria

1. `extractum-gemini-browser` is a workspace member with no reverse dependency
   and no `extractum-process` dependency.
2. Concrete sidecar/CDP process ownership remains entirely in `extractum`.
3. The crate owns domain DTOs, run log, portable runtime state, job lifecycle,
   typed timeout/cancellation, and reconciliation decisions.
4. Tauri, application path resolution and directory creation, SQLx/Apalis,
   migrations, worker registration, and process shutdown remain app-side.
5. The private app facade preserves existing external Rust consumer paths and
   IPC/serde behavior.
6. Typed timeout/cancellation replaces string classification without changing
   the exact existing result messages, stable serde fields, or persisted
   run-log JSON bytes.
7. Every one of the 94 baseline Gemini tests has one declared and passing
   owner; new characterization tests also pass.
8. The source-boundary contract proves the dependency, visibility,
   moved-not-copied, and forbidden-process invariants.
9. New-crate, app-dependent, workspace, Linux portability, no-bundle, startup,
   and the fixed sidecar/CDP/shutdown smoke gates pass.
10. The advisory record contains either the focused raw values/medians and
   ordinary workspace duration, or an explicit `incomplete / no conclusion`
   result with exact probe restoration proven; neither outcome adds a timing
   veto.
11. A literal verification document records the final module/test inventories,
    commands, results, and any infrastructure exclusions.

## Non-Goals

- Recreating `extractum-process` through a generic trait or hidden process
  service.
- Moving PID, child, pipe, containment, kill, reap, or shutdown ownership into
  the domain crate.
- Changing frontend or IPC contracts, serialized value strings, run-log
  layout, database schema, migrations, queue backend, or retry semantics.
- Mass-rewriting `prompt_packs`, `lib.rs`, or other external consumers.
- Extracting all of `commands.rs`, `jobs.rs`, `state.rs`, `sidecar.rs`, or
  `cdp_chrome.rs` wholesale.
- Repairing MSI/WiX packaging.
- Restoring automatic compile-time retention gates or the canceled diagnostic
  machinery.

## Resulting Plan

After the owner reviews this written specification, it produces one Phase 4
implementation plan for seam preparation, mechanical extraction, integration,
and verification. Phase 3 reapplication and the process-shell anomaly tracks
are not prerequisites.
