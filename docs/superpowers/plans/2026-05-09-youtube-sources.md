# YouTube Sources MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add YouTube video and playlist sources as Extractum's first concrete non-Telegram ingest provider.

**Architecture:** The MVP is split into independent implementation parts. Each part ends with a project state that builds, passes its focused tests, and either preserves existing Telegram behavior or exposes one complete YouTube capability.

**Tech Stack:** Tauri 2, Rust 2021, sqlx SQLite, zstd, Svelte 5, Vitest, `yt-dlp`, approved targeted UI dependencies for new components only.

---

## Source Documents

This plan is based on:

- `reference/youtube/youtube_sources_mvp_v_1_specification.md`
- `reference/youtube/youtube_sources_implementation_plan.md`
- `reference/youtube/Рекомендуемый порядок работ.txt`

---

## Execution Order

Implement the parts in order. Do not start a later part while the previous part is failing tests or leaves UI commands wired to missing backend behavior.

1. [Part 1: Schema and Contracts](2026-05-09-youtube-sources-01-foundation.md)
2. [Part 2: Preview and Add Source](2026-05-09-youtube-sources-02-preview-and-add.md)
3. [Part 3: Jobs, Metadata, and Transcripts](2026-05-09-youtube-sources-03-jobs-metadata-transcripts.md)
4. [Part 4: Comments and Analysis](2026-05-09-youtube-sources-04-comments-and-analysis.md)
5. [Part 5: Auth and Settings](2026-05-09-youtube-sources-05-auth-and-settings.md)
6. [Part 6: UI Polish, Hardening, and Docs](2026-05-09-youtube-sources-06-ui-hardening-docs.md)

---

## Consistency Gates

Every part has its own focused verification commands. In addition, use these gates before moving to the next part:

- The app compiles.
- Existing Telegram source add/sync/list behavior still works.
- New YouTube UI is hidden or disabled until its backend command exists.
- New UI dependencies are introduced only inside new Svelte components or new local UI wrappers created for this MVP.
- No command creates partial transcript/comment data outside a transaction.
- Any persistent schema change is covered by a migration registration test.
- Any live YouTube behavior has fixture-based unit coverage plus a manual verification note.

---

## Frontend UI Library Policy

The YouTube MVP should keep Extractum's existing local UI system as the visual source of truth. Do not replace or restyle existing shared components with a full external UI kit.

Allowed targeted additions:

- `@lucide/svelte` for icons in new action buttons, status rows, and provider-specific controls.
- `bits-ui` for accessible headless primitives in new components, especially tabs, tooltips, popovers, dropdown menus, switches, and complex dialog behavior.
- `@tanstack/svelte-table` only if a new table component needs real sorting, filtering, keyboardable row selection, or column state. Do not use it for simple lists.
- `paneforge` only if a new workspace shell requires resizable panes. Do not introduce it for static panel layouts.

Guardrails:

- Wrap external primitives behind local components in `src/lib/components/ui` when they will be reused.
- Keep styling in local CSS and existing design tokens (`--panel`, `--border`, `--text`, `--primary`, etc.).
- Do not add Tailwind-first UI kits such as Skeleton, DaisyUI, or Flowbite for this MVP.
- Do not retrofit existing components solely to use a new library. Existing components may render new YouTube child components, but direct imports from new UI dependencies should stay inside new components or new local wrappers created for this MVP.
- Include `package.json` and `package-lock.json` changes in the same task commit that first uses a new UI dependency.

---

## Part Boundaries

### Part 1: Schema and Contracts

Adds database and type foundations only.

End state:

- `items.item_kind` exists.
- YouTube playlist and transcript tables exist.
- YouTube DTOs and URL parsing exist.
- Telegram behavior is unchanged.
- No YouTube add/sync UI is exposed.

### Part 2: Preview and Add Source

Adds `yt-dlp` preview and source creation.

End state:

- User can preview video and playlist URLs.
- User can save video and playlist sources.
- Playlist membership rows are created.
- Canonical video dedupe works.
- Transcript/comments sync is still absent.

### Part 3: Jobs, Metadata, and Transcripts

Adds the first syncable YouTube corpus unit.

End state:

- YouTube jobs can be listed, started, and cancelled.
- Metadata sync works.
- Transcript sync creates one `youtube_transcript` item and timestamp segments.
- Analysis does not yet need to include YouTube corpus.

### Part 4: Comments and Analysis

Makes YouTube analyzable.

End state:

- Comments sync creates `youtube_comment` items.
- Telegram and YouTube source groups cannot mix.
- Playlist analysis expands to child video sources.
- YouTube corpus modes work.
- Timestamp refs resolve into YouTube trace metadata.

### Part 5: Auth and Settings

Adds optional auth and sync/caption settings.

End state:

- Public flows still work without auth.
- Cookies live in OS secure storage.
- `yt-dlp` receives cookies only through temporary files.
- YouTube settings roundtrip through `/settings`.

### Part 6: UI Polish, Hardening, and Docs

Finishes MVP UX and documentation.

End state:

- Source cards and detail views expose the expected YouTube states.
- Playlist rows have per-video actions.
- The manual test matrix has been run.
- README and architecture/schema docs describe the final MVP.

---

## Final Acceptance Checklist

- [ ] User can preview and add YouTube video URLs.
- [ ] User can preview and add YouTube playlist URLs.
- [ ] `watch?v=...&list=...` is treated as playlist.
- [ ] Canonical video source dedupe by `video_id` works.
- [ ] Playlist membership reuses existing canonical video sources.
- [ ] Unavailable playlist rows remain visible without transcript items.
- [ ] Video metadata sync updates source metadata.
- [ ] Transcript sync creates one `youtube_transcript` item and timestamp segments.
- [ ] Comment sync creates `youtube_comment` items for top-level comments and replies.
- [ ] YouTube jobs report progress, warnings, cancellation, and per-video results.
- [ ] Telegram and YouTube source groups cannot mix.
- [ ] Playlist analysis expands to child video sources.
- [ ] YouTube analysis supports transcript-only, description, and comments-inclusive modes.
- [ ] Timestamp refs resolve to readable excerpts and YouTube timestamp URLs.
- [ ] Saved runs remain stable after later sync changes.
- [ ] Public preview works without auth when possible.
- [ ] Cookie secrets are stored only in OS secure storage.
- [ ] `yt-dlp` receives cookies only through temporary files.
- [ ] New YouTube UI uses approved targeted UI dependencies only through new components or local wrappers.
- [ ] Full Rust and frontend verification passes.
