# Gemini Browser Pull Refresh Without Run Events Design

Date: 2026-06-22

Status: approved; implementation planned.

## Context

Gemini Browser now runs prompt-pack work through the Apalis-backed runtime. The
previous migration step removed state-bearing event payloads and reduced
`gemini-browser://run` to an invalidation-only signal:

- backend emits a minimal run-change event after durable run-log transitions;
- frontend ignores event payload state and rereads command read models;
- the Settings Gemini Browser panel uses a shared refresh scheduler for mount,
  user commands, and event-triggered refreshes.

That was an intentional intermediate step. The next selected approach is to
remove the `gemini-browser://run` transport completely. The UI should not listen
to Gemini Browser Tauri run events at all. It should read state through commands:

- provider status;
- run history;
- run details through a run-log detail command when a selected run must remain
  available beyond the visible history page.

This makes the UI model fully pull-based and removes the last pre-Apalis
run-event lifecycle from Gemini Browser.

## Goals

- Delete the Gemini Browser run-change Tauri event transport completely.
- Remove public frontend and backend symbols whose only purpose is
  `gemini-browser://run`.
- Keep Apalis responsible for execution and queue mechanics.
- Keep the Gemini Browser run log responsible for user-facing run history,
  terminal results, inspector data, and selected active prompt result.
- Keep cached provider status as a compact display/read model for
  `gemini_bridge_status`, not as run-history truth.
- Keep the Settings Gemini Browser panel responsive using command reads,
  scheduler coalescing, and bounded polling.
- Make missed UI notifications impossible as a backend failure mode because
  there is no notification channel left.

## Non-Goals

- No removal of Apalis.
- No return to the pre-Apalis in-memory queue or state-bearing event model.
- No new persistent queue UI project.
- No new global background polling service outside the mounted Settings panel.
- No change to prompt-pack runtime provider semantics.
- No change to Gemini Browser sidecar prompt sending or answer extraction.
- No change to prompt-pack progress events unrelated to Gemini Browser.
- No replacement of file-backed run logs with in-memory run state.

## Chosen Approach

Remove `gemini-browser://run` instead of replacing it with another UI event.

Backend lifecycle code still writes run-log transitions for queued, running,
terminal, timeout, failure, and cancellation states. It still updates cached
provider status snapshots on the normal path. It does not emit a Tauri event
after those writes.

Frontend code continues to use the shared Gemini Browser refresh scheduler as
the only path that mutates authoritative panel state. The scheduler reads:

- a cached/lightweight provider status read;
- `gemini_bridge_list_runs`;
- `gemini_bridge_get_run(run_id)` when a selected run detail must be refreshed.

The Settings panel starts a polling controller while mounted:

- an idle cadence discovers Gemini Browser work started outside the panel, such
  as prompt-pack YouTube Summary runs;
- an active cadence tracks queued/running work through lightweight run-log
  reads until it reaches a terminal run-log state;
- manual refreshes and command-triggered refreshes still call the same
  scheduler immediately.

This keeps the panel live enough for operator use without keeping any
state-bearing or invalidation-bearing Gemini Browser event channel.

## Source Of Truth

Gemini Browser has two durable or recoverable sources of truth:

- Apalis storage owns execution scheduling and queue mechanics.
- The Gemini Browser run log owns user-facing run history, run details,
  terminal result text, artifacts, and inspector data.

The cached provider status snapshot is a display read model. It may summarize
active run id, queue depth, latest message, and provider readiness, but it must
not become the source of final run status or terminal result data.

Transient in-memory state remains allowed for coordination:

- synchronous `send_single` waiters;
- cancellation tokens;
- active sidecar coordination;
- worker-local bookkeeping.

That state may coordinate live work, but it must not be exposed as run-history
truth and must not be required to reconstruct run history after restart.

## Backend Behavior

Backend code should remove the Gemini Browser run-change event concept from the
runtime lifecycle:

- remove the `GeminiBrowserRunChangeEvent` Rust type;
- remove `GEMINI_BROWSER_RUN_CHANGE_EVENT`;
- remove helpers such as `run_change_event_from_run` and
  `emit_run_change_event_core`;
- remove `AppHandle.emit("gemini-browser://run", ...)` calls from Gemini
  Browser commands and worker paths;
- remove tests that assert event payloads or best-effort event delivery;
- keep tests that assert run-log transitions, waiter cleanup, cancellation
  cleanup, status snapshot behavior, and worker behavior.

Run lifecycle ordering still matters even without events:

1. On enqueue, create a queued run-log record.
2. If enqueue fails after the queued record exists, remove transient waiters,
   write the failed terminal run-log result, and return the enqueue error.
3. On worker pickup, mark the run log as running and update cached provider
   status snapshot as appropriate.
4. On terminal completion, timeout, failure, or cancellation, write the terminal
   run-log result, update cached provider status snapshot best-effort, unblock
   waiters, and clear transient active/cancel state.

No backend job transition may depend on frontend transport success. Since the
transport is removed, there is no emit failure path and no event retry path.

Status/open/resume commands should not emit anything and should not synthesize
run history. They may return current provider status or perform their existing
side effects, but the Settings panel treats command return values as local
command outcomes, not as authoritative replacement for the refresh read models.

## Read Model Commands

The pull UI depends on command read models:

- `gemini_bridge_status` returns the current provider display status for
  explicit UI refreshes and user actions.
- `gemini_bridge_status_snapshot` returns the cached provider display status
  without live sidecar/browser probing. Active polling must use this command
  for status reads.
- `gemini_bridge_list_runs` returns the recent run-log rows used by history and
  selected/active result UI.
- `gemini_bridge_get_run(run_id)` returns one run-log record by id for selected
  run details and inspector data.
- `gemini_bridge_open_run_folder` remains an action command, not a read model.

`gemini_bridge_get_run(run_id)` is required because `gemini_bridge_list_runs`
is limited and page-oriented. The panel may hydrate visible row details directly
from `list_runs`, but selected inspector/details UI must be able to reread the
selected run by id even after the row falls outside the current history limit.
The detail command reads the same file-backed run log and must not reintroduce
events or in-memory run-history state.

`gemini_bridge_status` may still perform a live sidecar/browser probe, but that
probe is outside the run-history truth model and must not run on the active
polling cadence. The current implementation performs a live `sidecar::status`
probe with a short timeout; calling that every second during worker execution
can contend with CDP/sidecar work. Polling must therefore prefer run-log reads
and `gemini_bridge_status_snapshot`. Live probe data may contribute to explicit
user action responses, but it must not overwrite lifecycle-owned cached
snapshot fields unless a separate merge/precedence contract is defined.
Run-history rows and terminal result display continue to come from
`gemini_bridge_list_runs` and `gemini_bridge_get_run`.

`gemini_bridge_status_snapshot` must not expose stale active/queue fields after
restart. The first pull refresh after application startup must await the Gemini
Browser startup reconciliation gate before returning status snapshot or
run-history data. The snapshot command then returns cached status derived from
reconciled Apalis/run-log state. A conservative non-running snapshot is allowed
only if reconciliation itself fails or times out, and it must include a
degraded/reconciliation-failed message. It is not allowed merely because
reconciliation is still pending. The snapshot command must never return an old
`running`, `active_run_id`, or `queue_depth` value from a previous process as
authoritative current state.

## Frontend API Surface

Frontend Gemini Browser API should remove event-specific exports:

- remove `GEMINI_BROWSER_RUN_CHANGE_EVENT`;
- remove `listenToGeminiBrowserRunChanges`;
- remove `GeminiBrowserRunChangeEvent`;
- keep command wrappers such as `geminiBridgeStatus`,
  `geminiBridgeStatusSnapshot`,
  `geminiBridgeListRuns`, `geminiBridgeSendSingle`,
  `geminiBridgeOpenBrowser`, `geminiBridgeResume`,
  `geminiBridgeStartCdpChrome`, `geminiBridgeStop`,
  `geminiBridgeGetRun`, and `geminiBridgeOpenRunFolder`.

The API module should no longer import `listen` or `Event` from
`@tauri-apps/api/event` for Gemini Browser run lifecycle handling.

The old names from earlier phases must also stay absent:

- `GeminiBrowserRunEvent`;
- `listenToGeminiBrowserRuns`;
- `GEMINI_BROWSER_RUN_EVENT`.

Historical design docs may still mention those names, but production source and
current tests should not expose or use them.

## Frontend Refresh Model

The Settings Gemini Browser panel should continue to route all authoritative
state updates through one shared refresh scheduler.

Authoritative panel state includes:

- provider `status`;
- `runs`;
- selected run result;
- selected run details loaded by id when the selected row is not available in
  the visible run-history page;
- active prompt result derived from run log rows;
- status and history loading/error state.

Command return values may be used for local command messages, to decide whether
a command succeeded, or to display an immediate action error. They must not be
assigned directly into authoritative `status`, `runs`, or result state in a way
that bypasses the scheduler.

All refresh sources use the same scheduler:

- component mount;
- manual Refresh button;
- Start Chrome;
- Open Browser;
- Resume;
- Stop;
- Send Test Prompt;
- polling ticks.

There is no event listener refresh source after this design.

Long-running command flows must start pull refresh before awaiting terminal
command completion. `Send Test Prompt` is the important case: the current
`geminiBridgeSendSingle` promise resolves only after the Gemini Browser run is
terminal. Without events, waiting for that promise before refreshing hides the
queued/running states. The panel must:

1. create and select the run id locally;
2. record that run id in local pending-run state;
3. start the `geminiBridgeSendSingle(...)` promise;
4. immediately force active polling and schedule a refresh before awaiting the
   terminal promise;
5. continue active polling until a refresh/read model confirms that the pending
   run reached a terminal run-log state, `gemini_bridge_get_run(run_id)` returns
   not found after the command has failed, or the pending-run grace window
   expires;
6. after the terminal promise settles, schedule one final refresh through the
   scheduler.

The terminal command result may be used for a local message, but the result
display still comes from run-log refresh or `gemini_bridge_get_run(run_id)`.
A rejected `geminiBridgeSendSingle(...)` promise is not enough by itself to
clear local pending-run state, because backend enqueue can fail after creating
a queued row and writing a failed terminal run-log result. Unless backend later
returns a distinct error kind that proves no run-log row was created, the UI
must schedule a final refresh/get-run after rejection and clear pending state
only from the read-model result.

Not-found after a rejected command is provisional. To avoid racing file-system
visibility or backend cleanup, clear pending state on not-found only after the
command promise has settled and either:

- two consecutive post-settlement refreshes agree that `list_runs` does not
  include the run and `gemini_bridge_get_run(run_id)` returns not found; or
- a short post-settlement not-found retry window expires. The default retry
  window is 2000 ms.

If a terminal run-log row appears during that retry window, it wins and clears
pending state as a normal terminal result.

The existing scheduler semantics remain:

- no parallel status/history refresh races;
- several refresh requests during an active refresh collapse into one trailing
  refresh;
- each caller receives a promise for the refresh requested by that call, not
  for unrelated later trailing refreshes;
- expected `gemini_bridge_status`, `gemini_bridge_status_snapshot`,
  `gemini_bridge_list_runs`, and `gemini_bridge_get_run` failures are captured
  into UI error state instead of producing unhandled rejections;
- status, run-history, and selected-detail results are applied independently,
  so one failed read model does not hide another successful read model update;
- after disposal, scheduler callbacks must not mutate component state.

"One scheduler" means one arbitration and state-application path, not
necessarily one identical command set for every refresh. The scheduler may have
light and full modes, for example `scheduleRefresh({ mode: "light" })` for
polling and `scheduleRefresh({ mode: "full" })` for explicit user refreshes.
Both modes must still coalesce through the same scheduler and apply results
through the same authoritative callbacks. Active polling uses the light mode and
must not call live sidecar/CDP probes. Manual or command-driven full refreshes
may include a live status probe when that is useful and bounded.

Refresh mode command contract:

| Mode | Commands | Live sidecar/CDP probe | Purpose |
| --- | --- | --- | --- |
| `light` | `gemini_bridge_status_snapshot`, `gemini_bridge_list_runs(limit)`, and `gemini_bridge_get_run(selectedRunId)` when selected detail is missing, stale, pending, or outside the visible history page | Not allowed | Active/idle polling, pending-run tracking, low-impact history refresh |
| `full` | `gemini_bridge_status`, `gemini_bridge_list_runs(limit)`, and `gemini_bridge_get_run(selectedRunId)` when a selected detail exists | Allowed only inside `gemini_bridge_status` | Manual refresh, mount refresh when explicit live status is desired, and post-command refreshes that need live provider diagnostics |

Neither mode may perform action commands such as open browser, start Chrome,
stop, resume, send prompt, or open run folder. Those commands remain separate
user actions that may schedule a refresh after they start or settle.

Refresh modes have priority: `light < full`. If a refresh is active and another
request arrives, the scheduler stores the strongest pending mode requested while
that active refresh is running. A full request must never be downgraded to a
light trailing refresh. For example, if active polling started a light refresh
and the user presses manual Refresh during it, the trailing refresh must run in
full mode and the manual caller's promise resolves only after that full refresh
is applied. If several pending callers share one trailing refresh, they may all
be satisfied by the strongest pending mode. A later light polling request must
not downgrade an already-pending full refresh.

Active refresh dominance is also required. If the active refresh is already at
least as strong as a new request, the new caller can join the active refresh
instead of forcing a trailing refresh. For example, if a full refresh is active
and a polling tick requests light, the polling request should attach to or skip
the active full refresh; it should not create a redundant trailing light
refresh. A trailing refresh is needed only when the pending request is stronger
than the active refresh, or when the caller explicitly requires a second read
after the active refresh finishes.

Promise semantics for dominance must be explicit:

- If a caller attaches to an active refresh that already satisfies the requested
  mode, `scheduleRefresh()` returns that active refresh promise.
- If a polling tick chooses to skip because an active stronger refresh is
  already in flight, the polling controller should not call `scheduleRefresh()`;
  it schedules the next timer after the active refresh settles, or treats the
  skipped tick as a resolved no-op.
- If a caller requires a second read after the active refresh, it must request a
  trailing refresh explicitly, for example with a `forceTrailing` option. That
  call returns the trailing refresh promise.
- User command flows that await UI freshness should receive the promise for the
  refresh that will actually apply the requested mode, whether that is the
  active refresh they attached to or a stronger trailing refresh.

The scheduler or the panel wrapper must expose a disposal/generation contract.
For example, the scheduler can provide `dispose()`, or the panel can pass a
mounted generation token checked by every `applyStatus`, `applyRuns`, and
detail/result callback. Clearing timers alone is not enough because an in-flight
refresh can complete after unmount.

Selected-run detail refreshes also need request identity. Every
`gemini_bridge_get_run(run_id)` request must capture the selected run id, or an
incrementing selected-detail request token, at dispatch time. The response may
update selected detail only if the component is still mounted and the captured
run id or token still matches the current selection. If the user selects run A,
then run B, and the A detail response arrives after B, the A response must be
ignored rather than overwriting B.

The same selected run id still needs a version guard. The panel should track the
latest applied `updated_at` for the selected run from either `list_runs` or
`gemini_bridge_get_run(run_id)`. A detail response may update selected detail
only if its parsed `updated_at` is not older than the latest applied version for
that run id. Equal timestamps are allowed; older timestamps are ignored. If a
timestamp cannot be parsed, the response must not overwrite a parsed newer
version. This prevents an older `get_run(run_id)` response, started while the
run was still `running`, from arriving after a newer terminal `list_runs` row
and reverting the inspector back to stale running data.

## Polling Controller

The Settings panel owns a polling controller only while it is mounted.

Use two cadences:

- idle cadence: 5000 ms;
- active cadence: 1000 ms for lightweight run-log/detail refreshes.

The idle cadence exists because Gemini Browser runs can be started outside the
Settings panel, for example through a prompt-pack workflow. Without run events,
an open Settings panel would otherwise have no way to discover those external
runs until the user manually refreshes.

The active cadence is used when the panel knows Gemini Browser work is in
flight. Active polling must be lightweight: it reads run logs and selected run
details, and it reads cached provider status only if that read is guaranteed not
to live-probe the browser/sidecar. Live `sidecar::status` probing must stay on
explicit user actions or manual/command-driven full refreshes. Polling ticks,
including idle polling ticks, use light mode.

A run is considered active for polling if either read model indicates work in
progress:

- a local pending run id exists for a command started by this panel and that
  pending run has not been cleared by a confirmed terminal run-log state, a
  confirmed not-found detail read after command failure, or the pending-run
  grace window;
- cached provider status is `running`;
- cached provider status has a non-null `active_run_id`;
- cached provider status reports `queue_depth > 0`;
- any visible run-log row is `queued` or `running`.

Terminal run statuses such as `ok`, `ready`, `needs_login`,
`needs_manual_action`, `blocked`, `timeout`, `browser_crashed`, `failed`, and
`cancelled` do not by themselves keep active polling alive.

Visible non-terminal rows are not allowed to keep one-second polling alive
forever. Backend startup/worker reconciliation should terminalize orphaned
queued/running rows whenever Apalis state proves they are no longer executable.
The frontend still needs a guard for stale data: a visible `queued` or
`running` row keeps active polling only while its age is within the active run
grace window. The default grace window is 30 minutes, which is larger than the
current default worker execution timeout plus hard guard. After the grace
window, the panel should downgrade to idle cadence, keep the stale row visible,
and surface a degraded/stale hint rather than polling every second until run-log
retention cleanup removes it.

Status-derived activity needs the same guard. A cached status snapshot that says
`running`, has `active_run_id`, or has `queue_depth > 0` keeps active polling
only when it is corroborated by a local pending run id, a visible non-terminal
run-log row, or a status activity grace timer. The panel starts that grace timer
when it first observes an uncorroborated active-looking status signature. The
default grace window is also 30 minutes. After the grace window, the panel
downgrades to idle cadence and shows a stale/degraded status hint until a later
refresh either corroborates the status from the run log or observes an inactive
status snapshot. This avoids one-second polling forever if the cached status
snapshot gets stuck after a crash or reconciliation bug.

Polling rules:

- start the controller after the initial mount refresh is scheduled;
- use the active cadence whenever active work is detected;
- fall back to the idle cadence when no active work is detected;
- manual and command refreshes may run immediately and should not wait for the
  next timer tick;
- a polling timer tick must not enqueue endless trailing refreshes;
- if the previous polling refresh is still active when the next polling tick
  would run, skip that tick or schedule the next timer only after the in-flight
  polling refresh settles;
- use recursive `setTimeout` after refresh completion or an equivalent
  `pollingRefreshInFlight` guard, not a naked `setInterval` that calls
  `scheduleRefresh()` regardless of in-flight work;
- scheduler trailing refresh behavior remains available for user/manual/command
  requests, but polling ticks should not create a back-to-back refresh loop when
  one refresh takes longer than the polling cadence;
- on component unmount, clear timers and dispose or invalidate the scheduler
  callbacks so late completions cannot write component state.

Refresh errors do not create a tight retry loop. After an error, the next retry
comes from the current cadence or from a new explicit user command/manual
refresh. If every requested read model fails, preserve the last known UI state,
show degraded/error state, and keep polling at the current cadence until the
next successful read can recalculate active versus idle mode.

Persistent failures must degrade active polling. If active polling sees three
consecutive polling refreshes where every requested read model fails, the
controller should switch to degraded idle cadence even if the last known state
looked active. Manual refreshes and user commands may still request immediate
full refreshes. The degraded state clears after any polling refresh successfully
applies at least one read model, after which active versus idle cadence is
recomputed from the fresh data.

The idle cadence is deliberately bounded and panel-scoped. This design does not
introduce a global background polling service or a queue-wide UI observer.

## Prompt-Pack Interaction

Prompt-pack flows such as YouTube Summary may submit Gemini Browser work without
the Settings panel being the initiating UI. Removing run events means the
Settings panel discovers that work through idle polling when the panel is open.

Prompt-pack progress UI should continue to use its own prompt-pack runtime
state and progress mechanisms. This design does not require prompt-pack flows to
wait for the Settings panel, and it does not require the Settings panel to be
open for Gemini Browser runs to execute.

If a prompt-pack Gemini Browser run starts and finishes between two idle polling
ticks, the next history refresh should still show the terminal run-log row. The
panel may not display every intermediate queued/running state in that short
case. That is acceptable because run logs, not UI events, are the durable
history.

## Failure Semantics

With no run event transport, there is no UI notification failure path.

Backend failures are judged by durable runtime operations:

- Apalis enqueue success/failure;
- run-log transition success/failure;
- sidecar execution success/failure;
- waiter/cancellation cleanup;
- cached provider status snapshot update where applicable.

Frontend freshness failures are judged by command reads:

- `gemini_bridge_status` may fail independently;
- `gemini_bridge_status_snapshot` may fail independently;
- `gemini_bridge_list_runs` may fail independently;
- `gemini_bridge_get_run(run_id)` may fail independently for a selected detail
  view;
- any combination of those failures should apply successful read results and
  preserve only the portions whose read failed.

Selected run detail has explicit failure states:

- If `gemini_bridge_list_runs` succeeds and includes the selected run, that row
  refreshes the selected detail immediately.
- If the selected run is not in the visible history page,
  `gemini_bridge_get_run(run_id)` is the selected-detail source.
- If `gemini_bridge_get_run(run_id)` returns not found because the run was
  deleted, expired by retention, or never existed, the UI must clear or mark the
  selected detail as unavailable. It may show a previous detail only as stale
  diagnostic context, not as authoritative current data.
- If `gemini_bridge_get_run(run_id)` fails transiently, the UI may preserve the
  previous selected detail but must show a detail error/stale marker.
- If status or status-snapshot and list-runs fail but get-run succeeds,
  selected detail may still update.
- If get-run fails but list-runs succeeds and includes the selected run, the
  list row wins for visible details.
- If all requested read models fail, preserve previous authoritative state,
  surface degraded/error state for each failed region, and wait for the next
  scheduled refresh or user action.

No backend run should be marked failed because the Settings panel is closed,
polling is stopped, a timer is delayed, or a frontend refresh request fails.

## Migration Notes

Implementation should remove the event in one slice rather than keep aliases:

1. Delete backend event type, constants, helpers, and emit call sites.
2. Update backend tests from event assertions to run-log/status/cleanup
   assertions.
3. Delete frontend event type, event constant, and listener helper.
4. Update frontend API tests to assert event exports are absent.
5. Remove the Settings panel listener and replace it with the mounted polling
   controller.
6. Add or expose `gemini_bridge_get_run(run_id)` for selected run details.
7. Add `gemini_bridge_status_snapshot` and `geminiBridgeStatusSnapshot` as the
   polling status boundary. This command returns cached provider status and must
   not call live `sidecar::status`.
8. Keep the refresh scheduler and command-triggered refresh paths.
9. Update docs and troubleshooting references that currently describe
   `gemini-browser://run` as an active runtime mechanism.

Historical plans may keep their historical event references, but current
architecture docs should describe Gemini Browser Settings UI freshness as
command-read plus polling.

## Testing Strategy

Rust tests should verify:

- queued, running, terminal, timeout, failure, and cancellation transitions
  still write correct run-log records without any event callback;
- enqueue failure still removes waiters and writes failed terminal run-log state
  when a queued record already exists;
- worker timeout still clears active and cancelled state;
- status/open/resume commands do not emit or require a run-change transport;
- live status probes do not overwrite lifecycle-owned cached status snapshot
  fields;
- `gemini_bridge_status_snapshot` returns cached provider status without calling
  live `sidecar::status`;
- `gemini_bridge_status_snapshot` waits for or uses reconciled startup state and
  does not return stale `running`/`active_run_id`/`queue_depth` from a previous
  process as current state;
- the first snapshot/list-runs pull after startup waits for the reconciliation
  gate, and conservative non-running snapshot fallback is used only when
  reconciliation fails or times out;
- `gemini_bridge_get_run(run_id)` returns selected run details from the
  file-backed run log and returns a not-found error for missing/expired runs;
- no production Gemini Browser backend source contains
  `GEMINI_BROWSER_RUN_CHANGE_EVENT`, `GeminiBrowserRunChangeEvent`,
  `run_change_event_from_run`, `emit_run_change_event`, or
  `gemini-browser://run`.

Frontend tests should verify:

- `src/lib/api/gemini-browser.ts` no longer exports
  `GEMINI_BROWSER_RUN_CHANGE_EVENT` or `listenToGeminiBrowserRunChanges`;
- `src/lib/types/gemini-browser.ts` no longer exports
  `GeminiBrowserRunChangeEvent`;
- the Settings panel imports no Gemini Browser event listener helper;
- the Settings panel contains no `payload.` reads for Gemini Browser run
  events;
- mount schedules an immediate refresh and starts the polling controller;
- manual refresh and command flows call the shared scheduler;
- scheduler mode coalescing keeps the strongest pending mode, so a full refresh
  requested during an active light refresh is not downgraded to light;
- polling light requests do not create redundant trailing light refreshes while
  a full refresh is already active;
- a light request during an active full refresh either skips scheduling or
  receives the active full refresh promise, and a caller that requests
  `forceTrailing` receives the trailing refresh promise;
- light refreshes call only `geminiBridgeStatusSnapshot`,
  `geminiBridgeListRuns`, and selected `geminiBridgeGetRun` when needed;
- full refreshes may call `geminiBridgeStatus`, `geminiBridgeListRuns`, and
  selected `geminiBridgeGetRun`, but no action commands;
- `Send Test Prompt` starts active polling and schedules a refresh before the
  terminal `geminiBridgeSendSingle` promise resolves;
- a local pending run id keeps active polling alive even if the first refresh
  happens before the queued run-log row is visible;
- rejected `geminiBridgeSendSingle` promises do not clear local pending-run
  state until a final refresh/get-run confirms terminal/not-found or the grace
  window expires;
- not-found after rejected `geminiBridgeSendSingle` clears local pending state
  only after two consecutive post-settlement not-found refreshes or after the
  short post-settlement not-found retry window expires;
- idle polling discovers runs that appear after mount;
- active polling continues while status or run rows indicate queued/running
  work;
- active polling calls `geminiBridgeStatusSnapshot`, not the live-probing
  `geminiBridgeStatus`;
- polling skips or waits when the previous polling refresh is still in flight,
  instead of creating immediate trailing refreshes forever;
- stale non-terminal rows older than the active grace window downgrade polling
  to idle cadence and surface a stale/degraded hint;
- stale status-derived activity older than the active grace window downgrades
  polling to idle cadence when not corroborated by local pending runs or
  non-terminal run-log rows;
- polling falls back to idle cadence after terminal rows and non-running status;
- unmount clears polling timers and avoids late state writes through a
  dispose/generation token contract;
- selected run details remain available through `geminiBridgeGetRun(runId)`
  even when the selected run falls outside the visible history page;
- out-of-order selected-detail responses are ignored when they do not match the
  current selected run id or request token;
- selected-detail responses with an older `updated_at` than the latest applied
  selected run version are ignored even when the run id matches;
- selected run details are cleared or marked unavailable when
  `geminiBridgeGetRun(runId)` returns not found/expired, and transient detail
  failures mark preserved detail as stale;
- status failure does not prevent successful run-history application;
- run-history failure does not prevent successful status application;
- status, run-history, and selected-detail failures are applied independently;
- all requested read model failures preserve previous authoritative state and
  surface per-region error state;
- three consecutive active polling refreshes where every requested read model
  fails downgrade the controller to degraded idle cadence until a later
  successful read model is applied;
- no direct assignment of authoritative status/runs/result state from command
  return values bypasses the scheduler.

Source-level checks may be used for event removal, but they should be scoped to
production Gemini Browser API, types, backend, and panel paths so historical
docs do not fail the check.

## Manual Verification

1. Open the Settings Gemini Browser panel.
2. Confirm the panel loads status and run history without a Tauri run-event
   listener.
3. Use Send Test Prompt and confirm the history moves through queued/running to
   a terminal result through command refresh/polling before the terminal
   command promise is awaited for final display.
4. Run a YouTube Summary flow with Gemini Browser while the Settings panel is
   already open and idle; confirm the panel discovers the run within the idle
   polling cadence and tracks it at the active cadence.
5. Close and reopen the Settings panel; confirm terminal run history is restored
   from the run log.
6. Select an older run that is not in the visible history limit and confirm the
   inspector can load it by id.
7. Confirm active polling uses `gemini_bridge_status_snapshot` rather than the
   live-probing `gemini_bridge_status`.
8. Temporarily make `gemini_bridge_status_snapshot` fail while
   `gemini_bridge_list_runs` succeeds; confirm run history still updates.
9. Temporarily make `gemini_bridge_list_runs` fail while
   `gemini_bridge_status_snapshot` succeeds; confirm status still updates and
   previous history remains visible.
10. Temporarily make `gemini_bridge_get_run(run_id)` return not found for the
   selected run and confirm the inspector marks the detail unavailable rather
   than showing stale data as authoritative.
11. Select run A, immediately select run B, and confirm a late detail response
   for A does not overwrite B.
12. Simulate a stale `get_run` response for the currently selected run and
   confirm it cannot overwrite a newer `list_runs` row with later `updated_at`.
13. Temporarily make all polling read models fail repeatedly and confirm active
   polling degrades to idle cadence after the configured failure threshold.

## Acceptance Criteria

- No production frontend or backend Gemini Browser runtime source listens to or
  emits `gemini-browser://run`.
- No production Gemini Browser API exports a run-change event helper or event
  payload type.
- The Settings panel remains responsive through command reads and bounded
  polling.
- Active polling reads status through `gemini_bridge_status_snapshot` and does
  not live-probe sidecar/CDP at one-second cadence.
- Full refresh requests cannot be downgraded by concurrent light polling
  refreshes.
- Light and full refresh modes use the documented command sets and promise
  semantics.
- Long-running commands expose queued/running states through polling before
  their terminal promises resolve.
- Gemini Browser run history and result display remain based on run logs.
- Selected run details are readable by id from the run log.
- Selected run details are guarded by selection identity and per-run
  `updated_at` version.
- Prompt-pack Gemini Browser runs execute without requiring Settings panel
  events.
- Automated Rust and frontend tests cover event removal, polling behavior, and
  refresh failure behavior.
