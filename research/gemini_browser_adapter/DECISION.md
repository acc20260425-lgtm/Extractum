# Gemini Browser Adapter MVP Decision

Date: 2026-06-20

Status: accepted for implementation planning

## Decision

Use the resilient-scoring adapter as the MVP implementation baseline.

The production-facing Gemini Browser Provider should start from the
`resilient-scoring` approach and selectively include safe telemetry support from
the `telemetry-assisted` variant:

- keep sanitized `networkSummary` in results and failure artifacts;
- keep URL redaction and reduced DOM artifact mode for live Gemini runs;
- do not parse private Gemini RPC payloads;
- do not treat endpoint names, response bodies, or websocket frames as a stable
  completion or answer contract.

## Why

The DOM-only baseline is useful as a control case, but it is too fragile for the
first production slice. It depends mostly on role, label, placeholder, and CSS
fallbacks. That is enough for deterministic mock pages, but it leaves little
diagnostic surface when real Gemini UI drift changes wrappers, button shape, or
composer structure.

The telemetry-assisted variant adds useful observability, but it is risky as a
primary contract. Network events can help explain rate limits, request timing,
or artifact evidence, yet Gemini Web internals are private and may change
without notice. Treating network payloads or endpoint shapes as adapter truth
would make the provider brittle and hard to audit.

The resilient-scoring variant is the best MVP trade-off. It stays local and
deterministic while improving drift tolerance and diagnostics:

- accessibility-first locators remain the first stable signal;
- deterministic scoring evaluates visible/editable prompt candidates and
  enabled send candidates;
- locator attempts are recorded for debugging;
- local `gemini-dom-contract.config.json` overrides provide a controlled repair
  path;
- typed failures preserve queue control instead of throwing opaque exceptions;
- failure artifacts make selector drift repairable without storing browser
  profile data.

## Evidence

The executable research harness compares all three variants against the same
matrix:

```powershell
npm run test:gemini-browser-adapter
```

The matrix covers `3` adapter variants against `18` mock scenarios for `54`
variant/scenario pairs. The latest accepted verification produced:

- missing matrix pairs: `0`;
- unexpected failures: `0`;
- required artifact incomplete count: `0`;
- false completion count: `0`;
- clean typed failure behavior for manual-action, timeout, parse-failure, and
  browser-close scenarios.

## Production Guidance

Implementation planning should use this shape:

- Rust/Tauri orchestrates provider state, queue integration, settings, and
  filesystem-owned artifact directories.
- TypeScript/Node sidecar owns Playwright browser interaction and the DOM
  contract.
- Python remains out of the production runtime.
- Live Gemini runs use reduced artifacts by default.
- Manual Google states such as login, CAPTCHA, account picker, and consent
  return typed statuses and require user action; the adapter must not automate
  them.
- Telemetry is diagnostic only. The answer contract remains DOM text extraction
  plus bounded completion signals.

## Out Of MVP

- VLM-based runtime recognition.
- Remote selector update servers.
- Private Gemini RPC parsing.
- Hidden automation of Google account controls.
- Automatic CAPTCHA, consent, account-picker, or login handling.

## Production Handoff - 2026-06-20

- MVP implementation plan:
  `docs/superpowers/plans/2026-06-20-gemini-browser-provider-mvp.md`.
- Sidecar packaging follow-up:
  `docs/superpowers/plans/2026-06-20-gemini-browser-sidecar-packaging.md`.
- Production code must not import from `research/gemini_browser_adapter`;
  research stays as a regression harness and evidence source.
