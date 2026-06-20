# Gemini Browser Adapter Research

This research project turns the Gemini Browser Provider design and reference
material into a reproducible engineering research track for the unstable
Gemini web UI boundary.

The goal is not to automate Google account controls or to prove that Gemini Web
can never change enough to break automation. The goal is narrower and more
useful: design and later validate a `gemini-dom-contract.mjs` layer that either
continues to work across common UI drift, or fails quickly with typed status and
diagnostic artifacts.

## Inputs

Primary product design:

- `docs/superpowers/specs/2026-06-19-gemini-browser-provider-design.md`

Reference material:

- `reference/адаптера веб-интерфейса Google Gemini/gemini-dom-contract-resilience-addendum-2026-06-19.md`
- `reference/адаптера веб-интерфейса Google Gemini/gemini-dom-contract-resilience-addendum-2026-06-19-original.md`
- `reference/адаптера веб-интерфейса Google Gemini/Разработка-сверхнозостойкого-(anti-fragile)-Node.jsPlaywright-адаптера-для-автоматиза.md`
- `reference/адаптера веб-интерфейса Google Gemini/Устойчивый-Playwright-адаптер-для-Gemini.md`

## Project Files

- `RESEARCH_PROJECT.md` defines scope, hypotheses, deliverables, and acceptance
  criteria.
- `DOM_CONTRACT_NOTES.md` distills the DOM adapter decisions from the reference
  material.
- `RESILIENCE_TEST_MATRIX.md` defines the mock Gemini scenarios that should
  prove the adapter can succeed or fail cleanly.
- `TOOLS_AND_METHODS.md` defines the tools, implementation variants, and
  comparison method for the research loop.
- `mock-gemini/README.md` defines the future local mock page contract.
- `artifacts/` is reserved for future sanitized screenshots, traces, telemetry
  samples, and run logs.

## Boundary

This project must not store browser profile data, cookies, credentials, Google
auth state, or live Gemini chat transcripts. Real Gemini smoke testing is
manual and optional. The default research loop uses local mock pages and
sanitized artifacts.
