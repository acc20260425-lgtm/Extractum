# YouTube Thumbnail CSP Regression Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or executing-plans task-by-task.

**Goal:** Restore YouTube previews without remote CSP image origins.

**Architecture:** Rust fetches only allowlisted HTTPS YouTube thumbnail hosts with redirects disabled, validates image magic bytes, and returns typed data-URL results with in-flight dedup only. One Svelte thumbnail component owns a bounded 128-entry success LRU and placeholder/fallback rendering.

**Tech Stack:** Tauri/Rust, reqwest, base64, SvelteKit, Vitest.

## Global Constraints

- Do not add remote image origins or weaken CSP.
- Allow only `i.ytimg.com`, `i9.ytimg.com`, `img.youtube.com`, `yt3.ggpht.com` over HTTPS.
- Disable redirects; validate JPEG/PNG/WebP magic bytes and enforce a 1 MiB response limit.
- Backend deduplicates only in-flight requests. Frontend keeps at most 128 successful data URLs and terminal validation errors; transient network/HTTP errors are retried later.

### Task 1: Backend data URL resolver

**Files:** `src-tauri/src/youtube/thumbnail.rs`, `src-tauri/src/youtube/mod.rs`, `src-tauri/src/lib.rs`.

- [ ] Write RED Rust tests for host/scheme rejection, redirect-disabled client policy, 1 MiB limit, magic-byte rejection, typed terminal/transient errors, and concurrent in-flight dedup.
- [ ] Implement `YoutubeThumbnailState` and `resolve_youtube_thumbnail(url)` command returning a typed data-URL result; register state/command.
- [ ] Run focused Rust tests; commit `fix: proxy YouTube thumbnails through memory`.

### Task 2: Shared Svelte thumbnail component

**Files:** `src/lib/components/youtube-thumbnail.svelte`, `src/lib/youtube-thumbnail.ts`, `src/lib/youtube-thumbnail.test.ts`, thumbnail-owning components.

- [ ] Write RED Vitest tests for a 128-entry success LRU, terminal validation memoization, transient retry, and `url` plus local `fallbackSrc` rendering.
- [ ] Implement `YoutubeThumbnail` as the only async owner and replace direct remote thumbnail `<img>` usages while passing existing avatar data URLs as `fallbackSrc`.
- [ ] Run focused frontend tests; commit `fix: render YouTube previews from backend data URLs`.

### Task 3: CSP regression verification

**Files:** `src/lib/tauri-security-config-contract.test.ts`, `docs/browser-providers-llm-troubleshooting.md`.

- [ ] Add a CSP contract asserting no remote image origin; keep component behavior in render tests rather than source-grep contracts.
- [ ] Run check, Rust/frontend tests, and CSP feature release inspection with previews and no image CSP refusal.
- [ ] Commit `test: cover local YouTube thumbnail previews`.
