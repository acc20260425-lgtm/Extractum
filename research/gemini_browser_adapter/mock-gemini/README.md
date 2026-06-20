# Mock Gemini Contract

This directory is reserved for a future local Gemini-like mock page. The mock
is the primary way to validate the adapter without relying on a live Google
session.

## Goals

- Simulate the visible behavior the adapter depends on.
- Provide deterministic variants for success and failure paths.
- Let Playwright tests verify locator resilience, completion detection, manual
  action detection, and artifact capture.

## Planned Variants

- `ready`
- `happy-path`
- `wrapped-dom`
- `textarea-input`
- `contenteditable-input`
- `icon-send`
- `slow-pauses`
- `never-stable`
- `login-required`
- `captcha`
- `account-picker`
- `consent`
- `rate-limit`
- `unknown-modal`
- `broken-answer`
- `closed-page`

## Non-Goals

- Do not mimic private Gemini RPC payloads as a stable contract.
- Do not require real Google auth.
- Do not include real user prompts or Gemini answers.
- Do not store browser profile data.

## Future Implementation Notes

The mock can be a tiny local static page plus JavaScript state machine, or a
small Node test server if network telemetry needs to be simulated. The adapter
tests should interact with it only through browser-visible behavior.
