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
position. The frontend listener uses the event only to schedule a refresh from
the existing commands:

- `gemini_bridge_status`
- `gemini_bridge_list_runs`

The run log and Apalis-backed runtime remain responsible for factual run
state. The event becomes an optimization for freshness, not a state channel.

## Event Contract

Replace the state-bearing event type with a minimal change event.

```rust
pub struct GeminiBrowserRunChangeEvent {
    pub run_id: String,
    pub changed_at: String,
}
```

The event name may remain `gemini-browser://run` for this transition slice so
that the transport remains narrow and easy to remove later. Code should rename
helpers and types around the new semantics, for example:

- Rust: `GeminiBrowserRunChangeEvent`
- TypeScript: `GeminiBrowserRunChangeEvent`
- API helper: `listenToGeminiBrowserRunChanges`

Consumers must not infer run status from this event. They must reread status
and run log data.

## Backend Behavior

Backend run lifecycle writes remain ordered around the run log:

1. A run is created in the run log as queued.
2. A change event is emitted after the queued record exists.
3. When an Apalis worker starts the job, the run log is marked running.
4. A change event is emitted after the running record is written.
5. When the job reaches a terminal result or cancellation, the run log is
   updated with the final result.
6. A change event is emitted after the terminal record is written.

Provider status snapshots may still exist as runtime display state for
`gemini_bridge_status`, but they must not become an alternate run-history
source. Run-history UI reads from the run log.

Startup reconciliation should repair run log records from Apalis state as it
does today. It does not need to replay events for historical changes; UI mount
already performs an explicit refresh.

## Frontend Behavior

The Settings Gemini Browser panel must stop reading `payload.status` or
`payload.message` from the event. The listener should only schedule a refresh.

Refresh scheduling should be coalesced so several quick events do not produce
parallel status/list-runs races. A simple in-flight refresh guard or trailing
refresh flag is enough:

- If no refresh is running, start one.
- If a refresh is already running, remember that another refresh was requested.
- After the active refresh completes, run one more refresh if needed.

Direct user commands such as Start Chrome, Resume, Stop, and Send Test Prompt
may continue to call `refresh()` after their command returns. The event listener
is for background Apalis transitions.

## Legacy Removal Scope

Remove these pre-Apalis state-channel traces:

- Rust `GeminiBrowserRunEvent.status`
- Rust `GeminiBrowserRunEvent.message`
- Rust `GeminiBrowserRunEvent.queue_position`
- TypeScript equivalents of those fields
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
