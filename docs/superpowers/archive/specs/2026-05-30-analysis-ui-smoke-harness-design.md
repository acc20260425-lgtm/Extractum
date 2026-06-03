# Analysis UI Smoke Harness - Historical Note

> Status: shipped and archived. This note keeps the harness design intent, not
> the original execution plan.

## Decision

Analysis UI smoke coverage is opt-in and fixture-driven. It should exercise the
real app shell through the same browser/Tauri bridge assumptions as manual
smoke checks, while keeping fixtures small and deterministic.

## Rationale

- The analysis workspace has enough cross-panel state that unit tests alone do
  not catch layout and interaction regressions.
- Smoke runs should be explicit because they may need a running app, local
  fixtures, and browser automation.
- Artifacts should be useful for diagnosing failures without becoming part of
  normal product output.

## Preserved Contract

- Keep smoke scripts opt-in, such as `smoke:analysis`.
- Seed or load deterministic fixtures before interacting with the workspace.
- Prefer stable selectors and visible UI outcomes over brittle DOM structure.
- Capture screenshots or logs only as test artifacts.
- Keep the harness separate from production code paths.
