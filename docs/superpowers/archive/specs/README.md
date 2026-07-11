# Archived Specs

This directory stores historical Superpowers design specs for work that has
shipped, been superseded, or been folded into current-state documentation.

Use these files for rationale and regression context only. Before relying on
behavior described here, confirm the current state in root docs such as
`docs/project.md`, `docs/database-schema.md`, `docs/architecture-deep-dive.md`,
or `docs/backlog.md`.

Active or still-relevant specs belong in `docs/superpowers/specs/`.

Security Hardening and Hidden Child Processes specs from 2026-07-11 record the
shipped production Tauri/CSP/credential boundaries and the Windows
`CREATE_NO_WINDOW` policy for `yt-dlp` launchers.

NotebookLM source-group export specs from 2026-05-31 record the shipped
Telegram source-group export scope, validation rules, package layout, and UI
enablement boundaries.

The 2026-05-29 and 2026-05-30 Source Browser specs are historical rationale for
the shipped Source Browser architecture. Current behavior is summarized in
`docs/project.md`, `docs/design-document.md`, and
`docs/frontend-architecture-evolution-analysis.md`.

Analysis workspace parity specs record shipped canvas-level workspace tool
contracts for setup and opened-run states.

Saved Runs affordance specs from 2026-05-31 record the shipped missing-legacy
and capture-failed snapshot affordances plus the smoke coverage fixtures for
those degraded saved-run states.

Evidence Source Navigation specs from 2026-05-31 record the shipped scoped
Evidence to Source jump, focused load, one-shot highlight, and return workflow.

Analysis Companion Width specs from 2026-05-31 record the shipped desktop
companion column widening and Evidence container-query reflow.

Sanitized Diagnostics specs from 2026-06-02 record the shipped backend
diagnostic summary command, allow-list DTO, redaction policy, aggregate-only
queries, runtime checks, and no-UI/no-support-bundle boundary.

Diagnostics UI specs from 2026-06-03 record the shipped read-only diagnostics
route, privacy boundary, manual refresh behavior, source-contract restrictions,
and explicit exclusions for raw JSON, logs, copy actions, polling, and support
bundle export.

Diagnostics Problem-First specs from 2026-06-04 record the shipped issue-mode
layout that surfaces filtered issue tables before the healthy diagnostics
overview while preserving overview-first order for `All tables`.
