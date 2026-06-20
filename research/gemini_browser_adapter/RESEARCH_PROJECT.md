# Gemini Browser Adapter Research Project

Date: 2026-06-20

Status: active research scaffold

## Purpose

Extractum needs a browser-backed Gemini provider that can submit prompts through
the user's visible Gemini web session. The product design already defines the
provider-level shape. This research project focuses on the riskiest boundary:
the Node.js/Playwright adapter that must interact with `gemini.google.com`
without a public DOM contract.

The research outcome should give implementation slices a concrete, testable
contract for `gemini-dom-contract.mjs` and a local mock test surface before
any real Gemini automation is treated as reliable.

## Research Questions

1. Which locator cascade gives the best practical resilience without depending
   on brittle CSS classes or deep XPath?
2. Which completion detector avoids early completion during Gemini pauses while
   still returning within a bounded timeout?
3. Which manual-action states must be first-class provider states instead of
   internal exceptions?
4. What diagnostic artifacts are sufficient to repair selector drift without
   storing sensitive browser profile or account data?
5. What local mock Gemini scenarios are enough to make implementation progress
   reproducible without relying on a live Google session?

## Hypotheses

- Accessibility-first Playwright locators plus structural fallback will survive
  common cosmetic and wrapper-level DOM drift better than CSS/XPath-first
  scripts.
- Completion should be detected by intersecting signals: non-empty answer text,
  text stability, no active stop control, input usable, no critical anomaly,
  and a hard timeout.
- Network events are useful telemetry and rate-limit hints, but private Gemini
  endpoints must not be treated as the adapter contract.
- The adapter should favor typed failure plus artifacts over aggressive
  self-healing. Local selector override is acceptable; VLM/runtime visual
  reasoning is out of MVP scope.

## Non-Goals

- No automation of Google login, 2FA, CAPTCHA, consent, account picker, or
  account recovery.
- No storage of cookies, profile directories, credentials, auth state, or
  Google account material in this research folder.
- No dependency on real Gemini Web tests as the default verification path.
- No headless hidden browser flow for user-account interactions.
- No remote selector update mechanism unless a later design adds signatures,
  schema versions, rollback, and a ban on arbitrary JavaScript.
- No parsing of private Gemini RPC payloads as a stable source of truth.

## Deliverables

- A documented DOM contract for the future sidecar module.
- A mock Gemini page contract that simulates ready, generating, final answer,
  login required, manual action, rate limit, timeout, and broken DOM states.
- A resilience test matrix with success and clean-failure expectations.
- A diagnostic artifact contract covering screenshot, HTML or DOM snapshot,
  Playwright trace, locator attempts, sanitized network summary, URL, and
  status.
- A clear decision log separating accepted implementation guidance from raw
  reference ideas that are too fragile for MVP.

## Acceptance Criteria

The research project is ready for implementation planning when:

- `gemini-dom-contract.mjs` has a small high-level operation boundary that does
  not leak Gemini selectors outside the adapter.
- Each test matrix scenario has an expected status and required artifacts.
- Manual-action cases return typed states and never attempt to click through
  Google account controls.
- The default test path can run against a local mock page.
- Any live Gemini smoke test is marked optional/manual and produces sanitized
  artifacts only.

## Relationship To The Product Design

The product design defines the Extractum-facing provider: Settings UI, Tauri
commands, queue state, run logs, `ask_gemini_raw`, and parse/repair handoff.
This research project narrows in on the sidecar's unstable web UI boundary and
should feed implementation slices for the Node sidecar and its tests.
