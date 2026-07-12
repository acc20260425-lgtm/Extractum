# YouTube Summary Runtime Selection Design

**Status:** proposed for user review

## Problem

`YoutubeSummaryRunDialog` resets its runtime to `api` every time it opens. A user can prepare Gemini Browser or CDP Chrome and still start the summary through an API profile without noticing. This produced a real failed run against an unavailable local OpenAI-compatible endpoint while the browser processes were healthy.

## Decision

Treat the last YouTube Summary runtime and browser mode as local UI preferences. Persist them in `localStorage`; do not add backend settings, database rows, migrations, Tauri commands, or persisted/API enum values.

Use two scoped keys:

- `extractum.youtubeSummary.runtimeProvider`
- `extractum.youtubeSummary.browserProviderMode`

Accepted values remain the existing frontend contract:

- runtime: `api | gemini_browser`
- browser mode: `managed | cdp_attach`

Missing, malformed, or unsupported stored values fall back to `api` and `managed`. Storage access must be guarded for SSR/test environments and must not prevent the dialog from opening if browser storage is unavailable.

## Dialog Lifecycle

When the dialog opens:

1. restore and normalize the two preferences;
2. clear transient browser status, run history, errors, and preflight state;
3. load LLM profiles;
4. if the restored runtime is `gemini_browser`, refresh Browser Provider status;
5. run preflight only after the restored runtime is in state, so the request uses the intended provider.

When the user changes runtime or browser mode, persist the normalized selection immediately before refreshing status or preflight.

The runtime selection is global to this local Extractum installation rather than per project or source. Profile selection, model override, CDP endpoint, transient status, and preflight results remain outside this slice.

## Explicit Launch Affordance

Replace the generic `Start` label with a derived label:

- `Run via API`
- `Run via Gemini Browser`

The label and submitted `runtimeProvider` must derive from the same state. Existing Browser Provider blocking checks remain authoritative for enabling the Gemini Browser launch.

## Failure Handling

Storage read/write failures are non-fatal. Reads fall back to safe defaults; writes are best-effort and must not change the selected in-memory value or block preflight/start.

Backend preflight and run creation remain the source of truth. This design prevents accidental selection drift but does not silently switch providers when one runtime is unavailable.

## Testing

- Unit-test a small persistence helper with missing, valid, malformed, and storage-throwing cases.
- Update the dialog contract to require preference restoration before preflight, persistence on both selectors, and provider-specific CTA labels.
- Preserve existing API-wrapper and backend snapshot tests proving `runtimeProvider` and `browserProviderConfig` reach persisted runs unchanged.
- Run focused Vitest, full Vitest, and `npm.cmd run check`.

## Documentation and Registry Impact

No `docs/value-registry.md` update is required: this slice reuses existing runtime and browser-mode values and introduces only frontend-local storage keys. Add the behavior to current project documentation after implementation verification.

## Out of Scope

- automatic provider selection based on Browser Provider readiness;
- backend `app_settings` persistence;
- remembering API profile/model or CDP endpoint in this dialog;
- redesigning Browser Provider setup or run diagnostics;
- changing Prompt Pack DTOs, Tauri commands, or database schema.
