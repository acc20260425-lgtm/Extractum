# Diagnostics UI Design

## Goal

Add a frontend surface for the existing backend `get_diagnostic_summary` Tauri command.
The UI should expose a sanitized local health summary for operator/support use without
turning diagnostics into Settings content, a raw JSON viewer, a log viewer, or a support
bundle exporter.

The first UI slice is read-only: sanitized summary cards and tables plus manual refresh.
It does not add support-bundle export, log capture, explicit copy actions, polling, or
diagnostics settings. Browser-native text selection is fine; UI buttons for copying
sections, tables, logs, JSON, payloads, or the full summary are out of scope.

## Placement Decision

Diagnostics will be a dedicated route at `/diagnostics`, exposed through the sidebar.
This keeps Settings focused on LLM/provider configuration and gives diagnostics a clear
operator/support boundary.

Approaches considered:

- Dedicated `/diagnostics` route: chosen because it is findable, avoids overloading
  Settings, and leaves room for later support-safe actions.
- Inline Settings section: rejected because Settings currently has an explicit contract
  around LLM provider profiles and test runs.
- Settings modal action: rejected because it is cramped and would become awkward if
  diagnostics grows into privacy preview, refresh state, or support-safe export later.

## Frontend Architecture

Add a dedicated diagnostics route at `src/routes/diagnostics/+page.svelte` and a
`Diagnostics` sidebar entry. Diagnostics is a separate operator/support surface, not
part of Settings. Settings remains focused on LLM/provider configuration.

Frontend code uses a narrow API boundary:

- `src/lib/types/diagnostics.ts` defines frontend types that mirror the backend
  camelCase diagnostic DTO.
- `src/lib/api/diagnostics.ts` owns the Tauri `get_diagnostic_summary` invocation and
  returns the typed DTO without adding detail fields.
- `src/lib/diagnostics-view-model.ts` owns pure UI helpers for diagnostics labels,
  status tones, timestamp formatting, stable count sorting, empty-section rows, and
  diagnostics-safe error formatting.
- `src/routes/diagnostics/+page.svelte` renders loading, error, summary, and
  per-section empty states. It calls the API wrapper on mount and from a manual Refresh
  action only.

The route must not call `invoke(...)` directly. It must not display raw JSON dumps,
raw command errors, logs, source content, prompts, cookies, API keys, Telegram session
data, provider payloads, local paths, source titles, URLs, profile labels, or hidden
diagnostic detail.

The current backend DTO serializes Rust `generated_at_unix: i64` as numeric
`app.generatedAtUnix` in camelCase. Frontend types should model this field as `number`;
view-model formatting still performs runtime validation so malformed or partial payloads
do not render `NaN` or crash the page.

## UI And States

The diagnostics page should feel like a compact operator panel, not an analytics
dashboard. The top area contains the `Diagnostics` title, short copy explaining that
the page shows a sanitized local health summary, a `Refresh` button, and a small
metadata line with app version/build mode and `Summary generated <time>`. Refresh is
disabled while loading or refreshing.

The frontend must derive all status display from `get_diagnostic_summary`; it must not
run additional provider, runtime, browser, Tauri, filesystem, or source checks from the
route.

Content ownership after load:

- Header metadata line: app version, build mode, and `Summary generated <time>`.
- Status strip: compact derived overview of SQLite availability, migration status,
  secure storage status, and `yt-dlp` status. The strip uses the same backend fields as
  the detailed panels, but only as high-level status/tone indicators. Build mode is not
  part of the health status strip.
- App/build panel: app name/version, build mode, and generated time as factual metadata.
- Database panel: SQLite availability, migration details, and account count.
- Runtime/provider panels: secure storage status, `yt-dlp` status, provider/profile
  configured counts, `yt-dlp` runtime state, and YouTube job aggregates.
- Aggregate tables: source, item, run, LLM request, YouTube job, and ingest counts
  grouped only by coarse allowed keys.
- Privacy panel: backend-provided `excluded_data_classes`, rendered as chips/list and
  kept visible so users can see what diagnostics intentionally exclude.

The status strip is an overview of important health signals. Detailed panels are the
source of supporting facts. Any repeated values must serve this overview/detail split;
implementation should not create multiple unrelated widgets that compete over the same
field.

The privacy panel is always visible after a successful load. If
`privacy.excludedDataClasses` is missing, not an array, or empty despite the backend
contract, render a static fallback note: `This diagnostics view is designed to show
sanitized fields only. The backend did not report excluded data classes for this
summary.` Do not hide the privacy panel in this partial-data case.

`Summary generated <time>` uses the backend `app.generatedAtUnix` numeric Unix seconds
as an absolute UTC timestamp formatted by `formatSummaryGeneratedAt(value)` as
`Summary generated YYYY-MM-DD HH:mm:ss UTC`. `formatSummaryGeneratedAt` must validate
that `value` is a finite number before constructing a `Date`; invalid types, `NaN`,
infinite values, or missing timestamps render `Summary generated Unknown`.

States:

- Initial loading: render the page shell with compact loading status.
- Loaded: render cards/tables only; no raw JSON dump.
- Refreshing: keep the current summary visible, disable Refresh, and show subtle
  `Refreshing...` status.
- Initial error: show a sanitized app error message through
  `formatDiagnosticError("loading diagnostics", error)` from
  `src/lib/diagnostics-view-model.ts`; never render a raw exception object, stack trace,
  or stringified payload.
- Refresh error after a previous load: keep the previous summary visible and show a
  compact sanitized error near the Refresh control.
- Empty/partial: if a count group has no rows, show a quiet empty row such as
  `No diagnostic counts reported` rather than hiding the whole section.

Navigation/app topbar updates:

- Sidebar gets `Diagnostics` with the lucide `ShieldCheck` icon.
- App topbar route label becomes `Diagnostics` when the pathname starts with
  `/diagnostics`.

## Data Flow And Privacy Boundary

The data path stays deliberately narrow:

`onMount / Refresh` -> `loadDiagnosticSummary()` -> `invoke("get_diagnostic_summary")`
-> `DiagnosticSummaryDto` -> `src/lib/diagnostics-view-model.ts` helpers -> route render.

`src/routes/diagnostics/+page.svelte` keeps concrete separate Svelte state variables:
`summary: DiagnosticSummaryDto | null`, `loading: boolean`, `refreshing: boolean`,
`status: string`, and `error: string | null`. It should not use a combined state value
whose meaning changes by branch. Initial load can render shell + error only. Refresh
failure after a successful load keeps the previous summary visible and shows a compact
sanitized error near the refresh control.

Privacy boundary:

- UI displays only backend-returned allow-listed fields.
- UI does not derive new diagnostics from browser, Tauri, frontend environment,
  filesystem, provider state, or source state.
- UI does not render raw JSON.
- UI does not stringify unknown objects.
- UI/API wrapper does not log raw unknown errors or payloads.
- UI labels must avoid implying missing detail is available elsewhere.
- Unknown/new enum values render as readable fallback labels and neutral status tone,
  not a page crash.
- View-model helpers must not resolve ids, source labels, URLs, profile names, paths,
  or any hidden detail not present in the diagnostic DTO.

`src/lib/diagnostics-view-model.ts` is required for labels/grouping and safe UI
derivations: status tone mapping, timestamp formatting, title-case labels for coarse
keys, stable sorting of counts, empty-section helpers, and `formatDiagnosticError()`.
These helpers should be unit-tested without Svelte.

Status tone helpers return existing UI tones. Recognized healthy states use
`Badge` variant `success`; warning/degraded states use `warning`; failed/unavailable
states use `danger`; informational states may use `info`; unknown/new enum values use
`Badge` variant `neutral`. If the fallback is rendered through `StatusMessage`, use
`tone="muted"`. Unknown enum values must not receive success, warning, or danger
semantics. The current `Badge` component already supports `neutral`; the diagnostics UI
should use that existing variant and should not introduce new visual tone names.

Known status tone buckets for the first slice:

- `success`: `available`, `current`, `synced`, `ready`, `succeeded`, `completed`,
  `complete`, `none`.
- `info`: `pending`, `queued`, `running`, `cancel_requested`, `partial`, `present`.
- `warning`: `never_synced`, `missing_key`, `not_configured`, `unavailable`,
  `not_found`, `timed_out`, `cancelled`.
- `danger`: `failed`, `check_failed`, `error`, `internal`, `network`, `auth`,
  `validation`.
- `neutral`: any unknown, empty, or newly introduced status key.

Build mode is factual metadata rather than a health status. If it is rendered with a
badge tone, use `release` -> `success`, `debug` -> `info`, and unknown build modes ->
`neutral`.

Count rows sort by their grouping keys first, lexicographically after display-label
normalization, and by `count` only as a final tie-breaker when all grouping keys match.
This keeps refreshes from reordering unrelated rows just because counts changed.

`formatDiagnosticError(action, error)` wraps the existing `src/lib/app-error.ts`
`formatAppError`: first recognize the sanitized/serialized `AppError` values returned
by `get_diagnostic_summary`, then delegate to `formatAppError(action, error)` for those
recognized values. Unknown non-app errors must use a generic diagnostics failure
message. The wrapper must not log raw unknown errors, call `formatAppError` for unknown
objects, stringify unknown objects, expose stack traces, or preserve unknown detail-ish
fields.

Tests for `formatDiagnosticError` must include an unknown object carrying detail-ish
fields such as `stack`, `payload`, `url`, `path`, `raw`, and `message`. The formatted
message must use the generic diagnostics failure fallback and must not include those
field values or `[object Object]`.

## Error Handling And Testing

Error handling:

- API wrapper only invokes and returns the typed DTO; it does not `console.error` raw
  unknown errors.
- Route formats command failures through
  `formatDiagnosticError("loading diagnostics", error)` from
  `src/lib/diagnostics-view-model.ts`.
- Refresh failures after load keep the last-known-good summary visible.
- Unknown statuses render neutral tone and readable fallback labels.
- Missing or empty arrays render per-section empty rows.
- Missing or empty `privacy.excludedDataClasses` renders the static privacy fallback
  note while keeping the privacy panel visible.
- The page never shows raw backend payloads, stack traces, serialized unknown errors,
  or logs.

Testing approach:

- API wrapper test: `loadDiagnosticSummary()` calls `get_diagnostic_summary` exactly
  once and returns the typed DTO, including numeric `app.generatedAtUnix` as received.
- API wrapper privacy test: the wrapper does not add raw labels, paths, URLs, ids, base
  URLs, raw payload fields, log fields, stack fields, or extra lookup fields beyond the
  diagnostic DTO.
- View-model helper tests for `src/lib/diagnostics-view-model.ts`: status tone mapping,
  known status allow-list buckets, unknown status fallback to `neutral`, build mode tone
  mapping (`release` -> `success`, `debug` -> `info`, unknown -> `neutral`),
  `formatSummaryGeneratedAt` timestamp label
  (`Summary generated YYYY-MM-DD HH:mm:ss UTC`), `formatSummaryGeneratedAt` invalid
  timestamp/type fallback, stable count sorting by grouping keys before count,
  empty-section rows, privacy fallback note, `formatDiagnosticError` delegation to
  `formatAppError` for recognized `AppError` values, generic fallback behavior for
  unknown non-app errors, and malicious unknown-object error inputs with `stack`,
  `payload`, `url`, `path`, `raw`, and `message` fields.
- Route/source contract test: diagnostics route imports the API wrapper and does not
  contain `invoke(`. Source-contract scans must read only production route/component
  files, such as `src/routes/diagnostics/+page.svelte` and shared production
  components used by that route; test files are not part of the scan.
- Navigation contract test: sidebar includes `Diagnostics`, `ShieldCheck`, and app
  topbar maps `/diagnostics` to `Diagnostics`.
- Settings contract test stays intact: Settings remains focused on LLM provider
  profiles/test runs and does not import diagnostics UI.
- Component/source contract test: diagnostics page does not contain raw JSON/log
  affordances such as `JSON.stringify`, `Raw JSON`, `Copy payload`, `Copy JSON`,
  `Copy logs`, `Copy table`, `Copy section`, `Copy summary`, or log-oriented copy.
  This scan also applies only to production route/component files so the test can safely
  mention forbidden strings in its own assertions.

Verification order:

1. Run targeted Vitest tests for diagnostics API/view-model/source contracts.
2. Run Svelte autofixer/check for the new diagnostics route when Svelte code is added.
3. Run the full gate: `npm.cmd run verify` on Windows or `npm run verify` elsewhere.

## Out Of Scope

- Automatic polling or background refresh.
- Support bundle export.
- Explicit copy actions for sections, tables, logs, JSON, payloads, or the full summary.
  Browser-native text selection remains allowed.
- Frontend environment checks outside `get_diagnostic_summary`.
- Settings integration beyond keeping existing Settings contracts intact.
- Backend DTO changes unless implementation discovers a missing allow-listed aggregate
  that needs a separate backend design.
