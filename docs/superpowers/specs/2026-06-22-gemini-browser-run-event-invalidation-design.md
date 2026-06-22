# Gemini Browser Run Event Invalidation Design

Date: 2026-06-22

Status: approved direction from design discussion.

## Context

Gemini Browser now runs prompt-pack work through the Apalis-backed runtime. The
old pre-Apalis behavior still leaks through the run event contract:

- Backend emits `GeminiBrowserRunEvent` with `status`, `message`, and
  `queue_position`.
- Frontend listens to `gemini-browser://run` and treats the event payload as
  current run state.
- Tests still verify event payloads as if they were part of the state model.

That makes the event stream a second source of truth next to Apalis execution
state and the file-backed Gemini Browser run log. The chosen migration step is
to keep a small event signal for responsive UI updates, but remove
state-bearing event semantics.

## Goals

- Keep ownership explicit: Apalis is the source for execution and queue state;
  the Gemini Browser run log is the source for user-facing run history and
  results.
- Convert the Gemini Browser run event into an invalidation signal: "a run
  changed; reread state."
- Remove legacy event payload fields that duplicate state: `status`, `message`,
  and `queue_position`.
- Keep the Settings Gemini Browser panel responsive without introducing a job
  queue UI project.
- Prepare a later full removal of the event transport.

## Non-Goals

- No removal of Apalis.
- No new persistent queue UI.
- No change to prompt-pack runtime provider semantics.
- No change to Gemini Browser sidecar answer extraction.
- No replacement of run logs with in-memory state.
- No migration to polling-only UI in this slice.

## Chosen Approach

Use the existing event transport only as a lightweight invalidation signal.

The backend still emits a Tauri event after queued, running, and terminal
run-log transitions, including cancelled and degraded terminal outcomes, but
the event no longer carries authoritative status, message, result, or queue
position. A degraded terminal outcome is not a separate
`GeminiBrowserRunStatus`; it is a terminal run-log result whose surrounding
display/update path is degraded, such as cached status snapshot update failure
after the terminal run-log write. Do not emit run-change events for status
probes, open-browser calls, resume calls, or other provider operations unless
they also create one of those run-log transitions. On the normal path, the
event is emitted after the durable run log and cached provider snapshot have
both been updated. The frontend listener uses the event only to schedule a
refresh from the existing commands:

- `gemini_bridge_status`
- `gemini_bridge_list_runs`

The run log and Apalis-backed runtime remain responsible for factual run
state. The event becomes an optimization for freshness, not a state channel.

## Event Contract

Replace the state-bearing event type with a minimal change event.

```rust
pub struct GeminiBrowserRunChangeEvent {
    pub run_id: String,
    pub run_updated_at: String,
}
```

The wire event name may remain `gemini-browser://run` for this transition slice
so that the transport remains narrow and easy to remove later. Public symbols
must not keep the old state-bearing names. Code must rename helpers and types
around the new semantics, for example:

- Rust: `GeminiBrowserRunChangeEvent`
- TypeScript: `GeminiBrowserRunChangeEvent`
- Event constant: `GEMINI_BROWSER_RUN_CHANGE_EVENT`
- API helper: `listenToGeminiBrowserRunChanges`

Consumers must not infer run status from this event. They must reread status
and run log data.

`run_updated_at` is not a separate event timestamp. It must be copied from the
`updated_at` value of the corresponding run-log record after the transition
that triggered the event. It must not use a fresh emit-time clock, Apalis job
timestamp, or sidecar timestamp. The field is mandatory in this design so tests
can prove the event points at a concrete run-log version without becoming a new
state-bearing payload. Frontend code must not use `run_updated_at` for sorting,
status decisions, terminal-result decisions, or queue decisions. It is only a
diagnostic/version pointer back to the run-log record that should be reread.

The old public names `GeminiBrowserRunEvent` and `listenToGeminiBrowserRuns`
should be removed rather than kept as aliases. Keeping aliases would preserve
the legacy state-channel API shape even if the payload is smaller.

## Backend Behavior

Backend run lifecycle writes remain ordered around the durable run log first and
the cached provider status snapshot second. These are the read models consumed
by the Settings panel refresh path.

1. A run is created in the run log as queued.
2. Provider status snapshot fields affected by that queued state are updated.
3. A change event is emitted after both read models are updated.
4. When an Apalis worker starts the job, the run log is marked running.
5. Provider status snapshot fields affected by the running state are updated.
6. A change event is emitted after both read models are updated.
7. When the job reaches a terminal result or cancellation, the run log is
   updated with the final result.
8. Provider status snapshot fields affected by the terminal state are updated.
9. A change event is emitted after both read models are updated.

This ordering is required because the frontend reacts to an event by reading
both `gemini_bridge_status` and `gemini_bridge_list_runs` immediately. Emitting
between a successful run log update and a successful status snapshot update can
produce contradictory UI state and is not allowed on the normal path.

Provider status snapshots may still exist as runtime display state for
`gemini_bridge_status`, but they must not become an alternate run-history
source. Run-history UI reads from the run log.

## Read Model Boundaries

The invalidation consistency guarantee covers only:

- the file-backed Gemini Browser run log read by `gemini_bridge_list_runs`;
- the cached provider status snapshot fields maintained by the Gemini Browser
  backend lifecycle and read by `gemini_bridge_status`.

If `gemini_bridge_status` also performs a live sidecar/browser probe, that live
probe is outside the invalidation ordering guarantee. It is best-effort current
state, not a read model that must be updated before an event can be emitted.
The event precondition must never depend on live browser or sidecar I/O.

The live probe must not write its result back into the lifecycle-owned cached
provider status snapshot in this slice. It may contribute to the immediate
`gemini_bridge_status` return value, but it must not overwrite lifecycle fields
such as active run, queue depth, latest run message, or terminal display state.
If a later design wants live probes to persist into the cached snapshot, it must
define a merge and precedence contract first. That contract must preserve run
log authority for run-history rows and terminal results.

## Failure Semantics

The run-change event is best-effort UI notification. It is not part of the
backend job state machine.

- If a run-log transition fails, the transition itself failed. Do not emit an
  invalidation event claiming that transition happened.
- If the run-log transition succeeds but the cached provider status snapshot
  update fails, do not roll back the run log and do not fail the job solely
  because the snapshot update failed. Record a diagnostic or warning, then
  attempt the invalidation event using the run-log record's `updated_at`.
  This is the degraded path: the durable run log is newer than the cached
  display snapshot.
- If Tauri event delivery fails after the run log and any successful snapshot
  update, do not fail the job and do not change the run result. Record a
  diagnostic or warning if the application has a suitable logging path.

This keeps UI transport and cached display state out of the authoritative
runtime state machine. A missed event may make the UI stale until the next
explicit refresh, but it must not corrupt or fail a completed run.

When the run log and cached provider status snapshot disagree, consumers must
treat the run log as the authoritative source for run-history rows, terminal
results, and inspector data. The provider status snapshot remains a compact
display summary and may lag during degraded recovery.

## Transient Coordination State

In-memory coordination is still allowed when it does not become run-history
truth. The Apalis runtime may keep transient state such as:

- waiters used by synchronous `send_single` calls to await one run result;
- cancellation tokens;
- active-run coordination needed to bridge command calls, worker execution, and
  sidecar lifetime.

This state is implementation coordination only. It may unblock waiters, cancel
work, or help route an active sidecar operation, but it must not be used as the
authoritative source for run history, final run status, or frontend refresh
data. If the process restarts, recoverable run state comes from Apalis storage
and the run log, not transient memory.

Startup reconciliation should repair run log records from Apalis state as it
does today. It does not need to replay events for historical changes; UI mount
already performs an explicit refresh.

## Frontend Behavior

The Settings Gemini Browser panel must stop reading `payload.status` or
`payload.message` from the event. The listener should only schedule a refresh.

All refresh sources must go through the same scheduler:

- initial mount;
- user commands such as Start Chrome, Resume, Stop, and Send Test Prompt;
- run-change invalidation events.

The scheduler must coalesce refreshes so several quick requests do not produce
parallel status/list-runs races. The behavior should be deterministic:

- If no refresh is running, start one.
- If a refresh is already running, remember that another refresh was requested.
- If several events arrive during one active refresh, collapse them into exactly
  one additional refresh.
- After the active refresh completes, run that additional refresh if requested.
- If the active refresh fails, the pending refresh flag must not be lost; the
  additional refresh still runs so a terminal update is not skipped because of a
  transient status/list-runs error.
- A pending refresh caused by requests that arrived during a failed active
  refresh triggers at most one immediate additional refresh. That additional
  refresh must not become an automatic infinite retry loop; any further retry
  requires a new mount, command, event, or explicit refresh request.

The refresh implementation must apply status and run-history results
independently. A `gemini_bridge_status` failure must not prevent a successful
`gemini_bridge_list_runs` result from updating the run-history UI. Use
independent awaits, `Promise.allSettled`, or an equivalent pattern rather than a
single `Promise.all` path that drops both results when one request fails.

If `gemini_bridge_list_runs` succeeds and `gemini_bridge_status` fails, update
`runs` and active prompt result state from the log, preserve or mark the status
summary as degraded, and show the status error separately. If
`gemini_bridge_status` succeeds and `gemini_bridge_list_runs` fails, update the
status summary but keep the previous run-history rows and show the history
error separately.

User commands may schedule a refresh after their command returns, but they
must not bypass the shared scheduler.

Command return values must not become a second authoritative UI write path for
the Settings panel. Command handlers may use returned values to show a local
message, command error, or immediate affordance, but authoritative panel
`status`, `runs`, and selected/active `result` state must be updated through the
shared refresh path. For example, `Resume` or `Open` must not permanently set
panel `status` from its direct command response, and `Send Test Prompt` must not
permanently set panel `result` from the direct command response while bypassing
run-log refresh.

## Legacy Removal Scope

Remove these pre-Apalis state-channel traces:

- Rust `GeminiBrowserRunEvent.status`
- Rust `GeminiBrowserRunEvent.message`
- Rust `GeminiBrowserRunEvent.queue_position`
- TypeScript equivalents of those fields
- Public API names `GeminiBrowserRunEvent` and `listenToGeminiBrowserRuns`
- Frontend assignments from event payload to user-facing message/status
- Tests that treat event payload as authoritative state

Keep these intentionally:

- Apalis-backed job runtime
- File-backed Gemini Browser run log with 24-hour retention
- Existing Tauri commands for status, list runs, and send single
- A temporary run-change event transport used only for invalidation

## Testing Strategy

Rust tests should verify:

- queued, running, and terminal transitions still update the run log, with
  cancellation covered as a terminal outcome;
- failed run-log transitions do not emit invalidation events;
- emitted change events contain only invalidation data;
- emitted change events copy `run_updated_at` from the run-log record;
- Tauri event emit failure does not fail or roll back a successful run-log
  transition, including queued, running, and terminal transitions, with
  cancellation covered as a terminal outcome;
- status probe, open-browser, and resume operations that do not create a
  run-log transition do not emit run-change events;
- cached provider status snapshot failure after a successful run-log transition
  does not roll back the run log or fail the job solely because of UI display
  state;
- live sidecar/browser status probes used by `gemini_bridge_status` do not write
  back into the lifecycle-owned cached provider status snapshot;
- no event payload test depends on status, message, or queue position;
- prompt-pack Gemini Browser execution still obtains its result from the
  Apalis/run-log path.

Frontend tests should verify:

- API helper listens on the Gemini Browser run event name with the new change
  event type;
- the Settings panel responds to a change event by refreshing status and run
  history;
- the Settings panel does not copy event payload fields into `message`,
  `status`, `runs`, or `result`;
- run history updates when `gemini_bridge_list_runs` succeeds even if
  `gemini_bridge_status` fails;
- mount, user commands, and run-change events all use the same refresh
  scheduler;
- command return values do not directly assign authoritative panel `status`,
  `runs`, or selected/active `result` state outside the shared refresh path;
- refresh requests are coalesced when several events arrive quickly.

Manual verification should include:

- Start Chrome / Resume still works.
- Settings test prompt updates history through refresh.
- With the Settings panel open, run a test prompt and verify the history and
  inspector update after refresh from run log data, not from state-bearing event
  payload.
- YouTube Summary through Gemini Browser still completes.

## Later Full Event Removal

This design intentionally makes a later polling-only or explicit-refresh design
smaller. After this slice, the UI no longer depends on event payload data. A
future slice can remove the event transport itself and replace it with polling,
explicit refresh after commands, or a separate watcher without changing the run
state model again.
