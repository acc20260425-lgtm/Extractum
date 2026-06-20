# DOM Contract Notes

These notes distill the reference material into decisions for a future
`gemini-dom-contract.mjs` adapter. Raw research exports contain useful ideas,
but some suggestions are intentionally downgraded here for safety and
maintainability.

## Module Boundary

`gemini-dom-contract.mjs` should be the only module that knows Gemini Web DOM
details. Other sidecar modules call high-level operations:

- `ensureReady(page)`
- `openCleanChat(page)`
- `submitPrompt(page, prompt)`
- `waitForFinalAnswer(page, options)`
- `scanCriticalState(page)`
- `captureFailure(page, reason)`

The sidecar queue, session lifecycle, run log, and Tauri IPC should not import
Gemini selectors or know page structure.

## Locator Cascade

Preferred order:

1. `getByRole()` with accessible name where available.
2. `getByLabel()` and `getByPlaceholder()` for prompt input.
3. Stable visible text for user-facing states such as sign-in, rate limits,
   retry, consent, and manual action messages.
4. Scoped structural search from a composer-like container to text input and
   send button.
5. Fuzzy candidate scoring over visible/editable elements.
6. CSS/XPath only as a last-resort local override inside the adapter.

CSS classes, generated Angular/Web Component names, deep XPath, and positional
chains are not primary contracts.

## Completion Detection

Completion should not rely on one signal. A final answer is accepted only when:

- a non-empty answer has appeared after prompt submission;
- answer text has been stable for `quietMs`;
- generation controls indicate the run is no longer active;
- the prompt input is usable again;
- `scanCriticalState` reports no blocking condition;
- the hard timeout has not elapsed.

Suggested starting values:

- `quietMs`: 1600 ms
- `minStreamingMs`: 1500-3000 ms after the first answer mutation
- `pollMs`: 250 ms
- `hardTimeoutMs`: 120-300 seconds depending on the caller

The reference material mentions 800 ms quiet periods, but that is too easy to
confuse with a normal Gemini pause. Use 800 ms only in fast mock tests, not as
the runtime default.

## Network Telemetry

Playwright network and WebSocket events are useful for diagnostics:

- note whether Gemini-related requests happened after submit;
- record sanitized URLs, status codes, and content types;
- hint rate limits through HTTP 429 or forbidden/auth responses;
- record last network activity timestamps.

Network payload parsing is not an MVP contract. Raw reference files discuss
private `batchexecute`, `L5adhe`, and `StreamGenerate` details, but these are
closed implementation details and should remain telemetry-only unless a later
explicit design accepts the risk.

## Critical State Scanner

The scanner should detect and report, not click through:

- `login_required`
- `manual_action_required`
- `captcha_required`
- `account_picker`
- `consent_required`
- `subscription_required`
- `rate_limited`
- `unknown_modal`
- `browser_crashed`

On detection, the adapter should capture diagnostics, focus or bring forward
the visible browser where possible, pause the queue, and wait for a user-driven
Resume command.

## Failure Artifacts

Every timeout, selector miss, manual-action state, parse failure, or unexpected
adapter error should be able to produce:

- screenshot;
- HTML or reduced DOM snapshot;
- Playwright trace when enabled;
- locator attempt log;
- sanitized network summary;
- current URL;
- adapter status;
- recent run log lines.

Artifacts must not include cookies, browser profile data, credentials, tokens,
or Google account secrets.

## Selector Override

MVP can support a local JSON config under app data. It should be versioned and
logged in each run. It must describe locator hints, not arbitrary JavaScript.

Remote selector updates are postponed. A future design must require signatures
or checksums, schema versioning, rollback, and explicit audit logging before
remote selector maps are allowed.

## Self-Healing

Do not add VLM-based self-healing in MVP. The practical first stage is:

- log locator attempts;
- score local DOM candidates deterministically;
- capture artifacts on failure;
- update local override config manually when needed.

This gives repairability without adding probabilistic runtime behavior,
latency, privacy issues, or new external service dependencies.
