# Frontend Architecture Evolution Analysis

> Date: 2026-05-30
> Scope: SvelteKit/Svelte frontend under `src`, analysis workspace UI, and
> Telegram Desktop reference review under `reference/tdesktop-dev`.

## Executive Summary

The frontend does not need a large architectural rewrite. The current
result-first analysis workspace is the right shape: compact source rail,
central report/source canvas, and companion tabs for evidence, chat, chunks,
and runs.

The Source Browser slices have shipped for live Telegram sources, YouTube
videos, YouTube playlists, live source groups, and available run snapshots.
They confirm the preferred frontend direction: keep route data ownership in
`/analysis`, add small focused components for provider-aware surfaces, and keep
browser tab state local to the shell.

The dedicated `/diagnostics` route has also shipped as a small route-owned
surface: one API wrapper, one pure view-model helper module, one repeated table
component, and no Settings integration. That is the preferred pattern for
future app-wide utility routes that are not part of the analysis workspace.

The useful Telegram Desktop lesson is interaction design for dense archive
navigation, not its Qt widget hierarchy or live-client state model. Extractum
should continue to keep frontend state local and typed, while improving the
source reader, topic navigation, job/status surfaces, saved-run discoverability,
and media evidence affordances.

## Current Shape

The frontend is a Svelte 5 app with a large `/analysis` route orchestrating:

- workspace selection and persisted UI state;
- source catalog and compact source rail;
- live and snapshot Source Browser tabs for single sources, source groups, and
  available run snapshots;
- report setup/output canvas;
- trace evidence, follow-up chat, chunk, and runs companion tabs;
- source jobs, Takeout jobs, NotebookLM export, and provider runtime status.

Outside `/analysis`, `/diagnostics` is the compact operator-health route. It
loads the sanitized backend summary through `loadDiagnosticSummary()`, derives
display state in `diagnostics-view-model.ts`, and keeps manual refresh and
privacy-boundary rendering local to the route.

The main pressure point is `src/routes/analysis/+page.svelte`. It already uses
smaller state/workflow modules, but the route still coordinates many
independent concerns. The shipped `SourceBrowserShell` is the pattern to
prefer: route-owned loading and callbacks, component-local interaction state,
subject-specific browser data objects, and narrow leaf components. Live
single-source browsing uses `sourceBrowserData`, live source groups use
`groupBrowserData`, and available run snapshots use `snapshotBrowserData`.
Future work should extract small, clear units only when a backlog slice
benefits from the boundary. A broad service-heavy frontend layer would add
indirection without matching the current Svelte app.

## Telegram Desktop-Informed Frontend Patterns

Reference review:

- local source tree: `reference/tdesktop-dev`
- reviewed areas: history widget scrolling/highlighting, forum topics,
  dialogs/admin-log search behavior, media display/download policy, and list
  refresh state
- date: 2026-05-29

Telegram Desktop is a live chat client. Extractum is an archive and analysis
workspace. Translate its patterns into reader and workflow affordances rather
than copying chat-client features.

### 1. Evidence-Centered Reader Navigation

Telegram Desktop treats a message timeline as a navigable surface: it can jump
to an item, highlight it, preload around scroll position, and provide return
paths after reply navigation.

Extractum already has the first pieces:

- selected trace refs scroll into view in the Telegram and YouTube readers;
- source rows are grouped by day or transcript segment;
- trace evidence can switch the central canvas into source mode;
- live source rows and saved run snapshot rows are separate source-view bases.

Recommended direction:

- add a bounded "jump to evidence" interaction that preserves the selected ref
  and applies a temporary highlight;
- add a lightweight return affordance after following evidence into source
  context;
- prefer load-around-ref behavior for evidence navigation over only loading
  older rows from the current page;
- keep reader grouping and refs provider-neutral in `source-reader-model.ts`.

### 2. Forum Topics As Navigation, Not Just Labels

Telegram Desktop models forum topics as a list with recent topics, active
subsection state, and stale targeted refreshes. Extractum should not copy
unread counts or chat-list ordering, but the topic navigation shape is useful.

Recommended direction:

- present Telegram forum topics as a compact reader filter layer, not only as
  badges inside messages;
- include `All`, recent/known topics, `General`, and unresolved or unrecognized
  topic states when the backend exposes them;
- keep migrated small-group history separate from current supergroup forum
  topics unless a future merged timeline explicitly defines the behavior;
- reuse source-level topic resolver state to explain stale or incomplete topic
  coverage.

### 3. Search And Filters As First-Class Workflow Controls

Telegram Desktop uses local search modes with cancel/apply behavior and delayed
application for large lists. Extractum already has useful saved-run filters in
the companion runs tab.

Recommended direction:

- continue the saved-runs discoverability work with source, group, provider,
  profile, model, template, status, and date filters;
- add source-reader search only when it can be scoped clearly to the current
  source or run snapshot;
- prefer compact segmented controls, chips, and typed inputs over a separate
  heavy search route;
- keep large-history narrowing local and predictable before adding global
  indexing features.

### 4. Unified Activity And Status Surfaces

Telegram Desktop consistently ties request ids, cancellation, delayed work, and
terminal cleanup to visible UI state. Extractum already has this pattern split
across Takeout jobs, source jobs, YouTube runtime state, analysis runs, and LLM
chat requests.

Shipped baseline:

- live Telegram sources, YouTube videos, and YouTube playlists now expose source
  status and jobs through `SourceActivityView` inside `SourceBrowserShell`;
- provider tabs keep contextual CTAs while detailed job cards live in
  Activity.

Recommended direction:

- extend the shipped activity/status pattern to any remaining non-shell source
  surfaces when a concrete workflow needs it;
- show active, terminal, warning, cancel-requested, retryable, and recovery
  states consistently across Telegram, Takeout, migrated-history import, and
  YouTube jobs;
- keep destructive actions disabled while same-source ingest work is active;
- keep status text sanitized and aligned with the secret-safety backlog.

### 5. Media Evidence Cards With Explicit Policy

Telegram Desktop has rich media previews, timestamp links, and a dedicated media
download policy. Extractum should keep its current metadata-first default.

Recommended direction:

- evolve the current Telegram media card into a shared evidence media card;
- show media kind, filename, MIME type, summary, availability, and citation
  context without downloading bytes by default;
- add previews only after explicit download/storage/bandwidth policy exists;
- keep text-only analysis available when providers or user settings do not
  allow multimodal input.

## What Not To Copy From Telegram Desktop

- Do not copy Qt widget architecture, custom painting systems, or chat-client
  cache structures.
- Do not persist unread, muted, draft, notification, pinned-chat, or chat-list
  ordering state unless a future Extractum workflow needs it directly.
- Do not turn source browsing into a full Telegram client.
- Do not hide media downloads inside normal sync.
- Do not introduce a service-heavy frontend architecture.
- Do not split every large Svelte file only because it is large. Split when the
  boundary reduces risk or unlocks a backlog slice.

## Suggested Order

1. Improve source-reader evidence navigation: jump, highlight, return, and
   load-around-ref behavior.
2. Add a compact Telegram topic navigation/filter surface for source browsing.
3. Extend the shipped source Activity pattern across remaining source surfaces
   when they gain new sync, Takeout, migrated-history, or recovery controls.
4. Finish saved-run filtering and cleanup affordances for large histories.
5. Evolve media evidence cards after the media download and preview policy is
   approved.
6. Extract analysis route orchestration only along the boundaries touched by
   the above slices.

## Expected Payoff

These changes should make the analysis workspace better for repeated research
work without changing the product's architecture:

- faster navigation from report evidence back to exact source context;
- clearer Telegram topic browsing and export follow-up scope;
- fewer duplicated status/action patterns across providers;
- saved-run history that remains useful as it grows;
- media UX that is ready for future opt-in downloads without surprising users.
