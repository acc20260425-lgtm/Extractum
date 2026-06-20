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
| Basic single prompt | `happy-path` | `ok` | Prompt submitted, final raw text captured |
| Wrapped DOM | `wrapped-dom` | `ok` | Locator cascade survives extra containers and changed classes |
| Textarea input | `textarea-input` | `ok` | Input finder uses role/placeholder/textarea fallback |
| Contenteditable input | `contenteditable-input` | `ok` | Input finder uses editable fallback |
| Icon-only send | `icon-send` | `ok` | Send finder uses role/title/structural fallback |
| Slow answer pauses | `slow-pauses` | `ok` | Completion waits through pauses longer than 800 ms |
| Never stabilizes | `never-stable` | `generation_timeout` | Screenshot, DOM snapshot, trace or telemetry |
| Login required | `login-required` | `login_required` | Manual-action message and screenshot |
| CAPTCHA text | `captcha` | `manual_action_required` or `captcha_required` | Manual-action message and screenshot |
| Account picker | `account-picker` | `manual_action_required` | Manual-action message and screenshot |
| Consent screen | `consent` | `manual_action_required` | Manual-action message and screenshot |
| Rate limit banner | `rate-limit` | `rate_limited` | Banner evidence and sanitized network summary if available |
| Unknown modal | `unknown-modal` | `manual_action_required` | Dialog evidence and screenshot |
| Broken answer container | `broken-answer` | `response_parse_failed` or `generation_timeout` | Locator attempts and DOM snapshot |
| Browser crash/close | `closed-page` | `browser_crashed` or `failed` | Status and sanitized error |

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
