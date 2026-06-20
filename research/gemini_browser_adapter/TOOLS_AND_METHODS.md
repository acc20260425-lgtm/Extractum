# Tools And Methods

This document defines the research tooling and comparison method for the
Gemini browser adapter. The goal is to compare a small number of implementation
strategies against the same mock scenarios before choosing the MVP shape.

## Number Of Variants

Try three implementation variants:

1. DOM-only baseline.
2. Resilient scoring adapter.
3. Telemetry-assisted adapter.

Three variants are enough to compare meaningful trade-offs without turning the
research track into an open-ended experiment. The final MVP recommendation is
expected to be variant 2 with selected safe telemetry pieces from variant 3.

## Variant 1: DOM-Only Baseline

Purpose: establish the simplest working baseline.

Techniques:

- Playwright `getByRole`, `getByLabel`, `getByPlaceholder`, and `getByText`.
- Minimal structural fallback for prompt input and send button.
- MutationObserver quiet-period completion.
- Stop button and input-enabled signals.
- Typed failure on timeout or selector miss.

Use this variant to measure how much resilience is possible before adding
candidate scoring or telemetry.

## Variant 2: Resilient Scoring Adapter

Purpose: define the likely MVP adapter.

Techniques:

- Everything from variant 1.
- Locator attempt logging.
- Deterministic fuzzy scoring for visible/editable candidates.
- Local `gemini-dom-contract.config.json` override.
- Failure artifact bundle: screenshot, HTML or reduced DOM snapshot, telemetry
  JSON, and optional trace.
- Manual-action scanner that pauses the queue and waits for Resume.

This is the recommended default because it improves diagnosability and DOM
drift tolerance while staying deterministic and local.

## Variant 3: Telemetry-Assisted Adapter

Purpose: test whether network events add useful resilience without making the
adapter depend on private Gemini internals.

Techniques:

- Everything from variant 2.
- Playwright `request`, `response`, and `websocket` event collection.
- Sanitized URL/status/content-type summaries.
- Rate-limit hints from HTTP 429 or visible banners.
- Last network activity timestamp as a weak completion hint.

Network data remains telemetry only. Do not parse private Gemini RPC payloads,
and do not treat endpoint names or response shapes as the adapter contract.

## Tools

- `@playwright/test` for browser-level adapter tests against mock pages.
- A small local Node HTTP or static server for mock Gemini variants.
- Vitest or existing TypeScript test tooling for pure functions: locator
  scoring, status mapping, artifact schema validation, and URL redaction.
- JSON fixtures for DOM snapshots, locator attempts, and telemetry examples.
- Playwright screenshots and trace archives for failure bundles.
- Local `gemini-dom-contract.config.json` for selector hints and overrides.

The tools should be runnable without a Google session. Live Gemini smoke tests
are optional and manual.

## Runtime Boundary

The production app should stay on Rust/Tauri, Svelte/TypeScript, and a
Node/TypeScript sidecar. Python is allowed only for local research tooling under
`research/`, such as fixture generation, DOM snapshot analysis, matrix result
summaries, and report generation.

Do not add Python as a packaged runtime dependency, production sidecar, or
Gemini Browser Provider execution layer.

## Method

Run all variants against the same scenario matrix:

1. Start the mock Gemini server.
2. Run the adapter strategy against every mock variant.
3. Record result status, elapsed time, answer text presence, and artifacts.
4. Verify that every scenario terminates within its hard timeout.
5. Compare diagnostics quality when the adapter fails.

The executable command is `npm run test:gemini-browser-adapter`. It uses a
wrapper runner so the matrix report is still generated after a Playwright e2e
failure. The wrapper clears stale matrix `result.json` files and stale
Playwright JSON output immediately before each e2e run.

The matrix should report:

- success count, including `ok` and `ready`;
- `ok` count;
- `ready` count;
- clean typed failure count;
- unexpected failure count;
- timeout/hang count;
- required artifact completeness;
- false completion count;
- average and worst-case elapsed time.

## Evaluation Criteria

Prefer the implementation that:

- never hangs indefinitely;
- keeps Gemini selectors inside `src/dom-contract.ts` and local overrides inside
  `gemini-dom-contract.config.json`;
- returns typed states for manual action, rate limit, timeout, parse failure,
  and browser failure;
- preserves raw text when available;
- produces enough artifacts to repair locator drift;
- avoids storing profile, cookie, credential, token, or account data;
- keeps network and self-healing behavior deterministic and auditable.

## Out Of MVP Scope

- Python runtime dependencies in the production app.
- VLM-based runtime recognition.
- Screenshot upload to external services.
- Remote selector update servers.
- Private Gemini RPC payload parsing as a completion or answer contract.
- Hidden automation of Google account, CAPTCHA, consent, or account-picker
  flows.
