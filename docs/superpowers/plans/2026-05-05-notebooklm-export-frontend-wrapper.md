# NotebookLM Export Frontend Wrapper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Centralize NotebookLM export frontend command/event access in `$lib/api/notebooklm-export.ts` and remove NotebookLM-specific raw Tauri calls from the `/analysis` route.

**Status:** Completed and fast-forward merged into `main` at
`66f634e test(notebooklm): verify export wrapper integration`.

**Architecture:** Add a narrow typed wrapper that follows the existing `$lib/api/takeout-import.ts` and `$lib/api/analysis-runs.ts` patterns. Keep existing NotebookLM DTO fields, route-local form state, folder picker behavior, and lifecycle state unchanged, so this refactor only moves the Tauri boundary.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Tauri v2 API, Vitest.

---

## Context

Core Sources and Takeout import already have focused frontend API wrappers:

- `src/lib/api/sources.ts`
- `src/lib/api/takeout-import.ts`
- `src/lib/api/analysis-runs.ts`

Before this work, NotebookLM export had one raw command and one raw event
listener in `src/routes/analysis/+page.svelte`:

```text
export_source_to_notebooklm
notebooklm://export
```

This task is wrapper-only. Do not rename NotebookLM DTO fields, do not change
Rust commands, do not wrap the folder picker, and do not extract a NotebookLM
workflow controller.

Relevant existing NotebookLM types:

- `src/lib/types/sources.ts`
  - `NotebookLmExportRequest`
  - `NotebookLmExportResult`
  - `NotebookLmExportEvent`

Relevant route-local helpers that must stay unchanged:

- `createNotebookLmExportId()` in `src/routes/analysis/+page.svelte`
- `notebookLmExportRequestFromForm(...)` in `src/lib/analysis-state.ts`
- `notebookLmExportProgressFromEvent(...)` in `src/lib/analysis-state.ts`
- `notebookLmExportInitialProgress()` in `src/lib/analysis-state.ts`
- `notebookLmExportCompleteStatus(...)` in `src/lib/analysis-state.ts`

## Completed Work

- Created `src/lib/api/notebooklm-export.ts`: typed wrapper for NotebookLM
  export command and event listener.
- Created `src/lib/api/notebooklm-export.test.ts`: Vitest coverage for wrapper
  command name, payload shape, event constant, and listener forwarding.
- Modified `src/routes/analysis/+page.svelte`: replaced only NotebookLM export
  raw `invoke(...)` and `listen(...)` usage with wrapper calls.
- Verified no raw NotebookLM export command or event strings remain in the
  analysis route.
- Verified selected frontend tests, full frontend tests, Svelte/TypeScript
  checks, and whitespace checks.

## Self-Review Checklist

- The wrapper is the only new frontend API surface.
- The route no longer owns the NotebookLM export command name or event name.
- Existing NotebookLM DTO field names stay unchanged.
- `openDialog(...)` remains route-local.
- No Rust files are modified.
- No chat, template, source group, Takeout, or source management workflows are
  refactored in this task.
- Tests cover command name, request payload shape, event name, and listener
  forwarding.

## Commit Messages

```text
test(notebooklm): add export api wrapper contract tests
feat(notebooklm): add export api wrapper
refactor(notebooklm): use export api wrapper in analysis route
test(notebooklm): verify export wrapper integration
```
