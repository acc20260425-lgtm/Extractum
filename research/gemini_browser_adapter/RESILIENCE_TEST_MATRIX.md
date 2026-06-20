# Resilience Test Matrix

The default validation target is a local mock Gemini-like page. Live Gemini
tests are optional manual smoke tests and should not be required for CI or
normal development.

## Core Rule

Every scenario must terminate in a bounded time. The adapter either returns a
successful raw answer, returns a successful readiness status, or returns a typed
failure with required artifacts. It must never hang indefinitely.

## Scenarios

| Scenario | Mock Variant | Expected Status | Required Evidence |
| --- | --- | --- | --- |
| Ready page | `ready` | `ready` | Status probe sees usable input and no critical state |
| Ready page missing send | `ready-missing-send` | `failed` | Probe returns typed failure and artifacts for incomplete composer |
| Ready page broken composer | `ready-broken` | `failed` | Probe returns typed failure and artifacts for missing composer controls |
| Basic single prompt | `happy-path` | `ok` | Prompt submitted, final raw text captured |
| Wrapped DOM | `wrapped-dom` | `ok` | Locator cascade survives extra containers and changed classes |
| Textarea input | `textarea-input` | `ok` | Input finder uses role/placeholder/textarea fallback |
| Contenteditable input | `contenteditable-input` | `ok` | Input finder uses editable fallback |
| Icon-only send | `icon-send` | `ok` | Send finder uses role/title/structural fallback |
| Slow answer pauses | `slow-pauses` | `ok` | Completion waits through pauses longer than 800 ms |
| Never stabilizes | `never-stable` | `generation_timeout` | Screenshot, DOM snapshot, trace or telemetry |
| Login required | `login-required` | `login_required` | Manual-action message and screenshot |
| CAPTCHA text | `captcha` | `captcha_required` | Manual-action message and screenshot |
| Account picker | `account-picker` | `account_picker` | Manual-action message and screenshot |
| Consent screen | `consent` | `consent_required` | Manual-action message and screenshot |
| Rate limit banner | `rate-limit` | `rate_limited` | Banner evidence and sanitized network summary if available |
| Unknown modal | `unknown-modal` | `manual_action_required` | Dialog evidence and screenshot |
| Broken answer container | `broken-answer` | `response_parse_failed` | Locator attempts and DOM snapshot |
| Browser crash/close | `closed-page` | `browser_crashed` | Status and sanitized error |

## Mock Page Requirements

The mock page should be deterministic and configurable through query params,
for example:

```text
http://127.0.0.1:<port>/mock-gemini?variant=wrapped-dom
```

Each variant should expose enough behavior to exercise the adapter without
importing adapter internals:

- user-visible input;
- send or submit action;
- answer area;
- streaming or delayed text updates;
- optional stop/interruption control;
- optional critical-state text, dialog, or URL-like marker;
- optional mock network responses.

## Artifact Expectations

Failure-oriented scenarios in the local research harness write artifacts under
the ignored repo-local run directory `research/gemini_browser_adapter/artifacts/`
so matrix runs are reproducible and easy to inspect. Live Gemini smoke tests or
production app experiments should pass an app-controlled run artifact directory
outside the repository by default. Sanitized fixture examples may later be
copied into this research folder if they contain no profile, account, token,
cookie, or private prompt data.

Required failure bundle:

- `telemetry.json`
- `page.html` or reduced DOM snapshot
- `failure.png` when screenshot capture is possible and the run is not using
  reduced live-safe artifact mode
- optional Playwright trace archive

Closed-page/browser-failure scenarios still require a typed status plus
`telemetry.json` and placeholder HTML/reduced DOM when the page object can no
longer be queried. Screenshot capture is optional for those cases.

`telemetry.json` should include locator attempts, status, sanitized URL,
network summary, timestamps, and error reason.

## Execution

Run the complete research matrix with:

```powershell
npm run test:gemini-browser-adapter
```

The Playwright JSON output is written to:

```text
research/gemini_browser_adapter/artifacts/playwright-results.json
```

The summarized matrix report is written to:

```text
research/gemini_browser_adapter/artifacts/matrix-report.md
```

The executable matrix is implemented in:

```text
research/gemini_browser_adapter/matrix-cases.json
research/gemini_browser_adapter/src/matrix-cases.ts
research/gemini_browser_adapter/tests/matrix.spec.ts
```

The matrix JSON is the single source of truth for adapter variants and scenario
IDs. `matrix-cases.ts` imports it for Playwright tests, and
`write-matrix-report.mjs` reads the same file for coverage validation. The
matrix covers all `3` adapter variants against all `18` scenarios. Expected
statuses and required evidence are asserted in `matrix.spec.ts`; report
generation fails when any expected variant/scenario pair is absent from the
Playwright JSON output.
