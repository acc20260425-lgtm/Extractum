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

That makes the event stream a second source of truth next to Apalis state and
the file-backed Gemini Browser run log. The chosen migration step is to keep a
small event signal for responsive UI updates, but remove state-bearing event
semantics.

## Goals

- Make Apalis plus the Gemini Browser run log the source of truth for run
  lifecycle state.
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

The backend still emits a Tauri event after meaningful run changes, but the
event no longer carries authoritative status, message, result, or queue
position. On the normal path, the event is emitted after the durable run log and
cached provider snapshot have both been updated. The frontend listener uses the
event only to schedule a refresh from the existing commands:

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
timestamp, or sidecar timestamp. If the implementation does not need this value,
the timestamp field may be removed instead of replaced with another source.

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

## Failure Semantics

The run-change event is best-effort UI notification. It is not part of the
backend job state machine.

- If a run-log transition fails, the transition itself failed. Do not emit an
  invalidation event claiming that transition happened.
- If the run-log transition succeeds but the cached provider status snapshot
  update fails, do not roll back the run log and do not fail the job solely
  because the snapshot update failed. Record a diagnostic or warning, then
  attempt the invalidation event using the run-log record's `updated_at`.
- If Tauri event delivery fails after the run log and any successful snapshot
  update, do not fail the job and do not change the run result. Record a
  diagnostic or warning if the application has a suitable logging path.

This keeps UI transport and cached display state out of the authoritative
runtime state machine. A missed event may make the UI stale until the next
explicit refresh, but it must not corrupt or fail a completed run.

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

Refresh scheduling must be coalesced so several quick events do not produce
parallel status/list-runs races. The behavior should be deterministic:

- If no refresh is running, start one.
- If a refresh is already running, remember that another refresh was requested.
- If several events arrive during one active refresh, collapse them into exactly
  one additional refresh.
- After the active refresh completes, run that additional refresh if requested.
- If the active refresh fails, the pending refresh flag must not be lost; the
  additional refresh still runs so a terminal update is not skipped because of a
  transient status/list-runs error.

Direct user commands such as Start Chrome, Resume, Stop, and Send Test Prompt
may continue to call `refresh()` after their command returns. The event listener
is for background Apalis transitions.

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

- queued, running, cancelled, and terminal transitions still update the run log;
- emitted change events contain only invalidation data;
- emitted change events copy `run_updated_at` from the run-log record;
- Tauri event emit failure does not fail a completed job;
- cached provider status snapshot failure after a successful run-log transition
  does not roll back the run log or fail the job solely because of UI display
  state;
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
- refresh requests are coalesced when several events arrive quickly.

Manual verification should include:

- Start Chrome / Resume still works.
- Settings test prompt updates history through refresh.
- YouTube Summary through Gemini Browser still completes.

## Later Full Event Removal

This design intentionally makes a later polling-only or explicit-refresh design
smaller. After this slice, the UI no longer depends on event payload data. A
future slice can remove the event transport itself and replace it with polling,
explicit refresh after commands, or a separate watcher without changing the run
state model again.
