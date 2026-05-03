# Code Review Results - 2026-05-03

## Scope

This review covered the whole Extractum codebase with security findings intentionally out of scope.
The review focus was maintainability, consistency, extensibility, testability, and avoiding duplication.

The repository was clean at the start of the review. CodeRabbit could not be used because
`coderabbit --version` failed in this environment with `Wsl/Service/E_ACCESSDENIED`, so the results
below are from a manual review.

## Findings

### Major: Analysis route owns too many responsibilities

`src/routes/analysis/+page.svelte` had grown into a broad workflow controller: route state, Tauri I/O,
event reducers, source/group/template editing, chat, NotebookLM export, Takeout job state, trace state,
and UI composition all lived together.

The downstream symptom was a very large prop contract in
`src/lib/components/analysis/workspace-main.svelte`.

Impact:

- new analysis features are harder to test in isolation;
- event-driven state updates are hard to reason about;
- every feature addition risks touching unrelated workflow state.

Suggested fix:

- extract focused domain controllers/helpers for runs, chat, sources, Takeout import, and NotebookLM export;
- keep the route as a composition layer;
- test extracted pure reducers before doing broader UI refactors.

### Major: Large backend modules mix unrelated behavior

`src-tauri/src/sources.rs` mixed DTOs, sync settings, avatar cache, Telegram peer resolution, sync,
forum topic refresh, DB row mapping, and tests.

`src-tauri/src/takeout_import.rs` mixed job state, Tauri commands, Telegram RPC details, pagination,
fallback policy, export-DC behavior, and tests.

Impact:

- larger change blast radius;
- more difficult targeted tests;
- more context required for safe edits.

Suggested fix:

- split by existing behavior boundaries, not by abstract architecture;
- likely modules: `sources/peer_resolution`, `sources/sync`, `sources/items`, `sources/settings`,
  `takeout/state`, `takeout/pagination`, and `takeout/rpc`.

### Major: Frontend/backend contracts were manually mirrored

Frontend TypeScript DTOs and raw Tauri command/event strings were manually maintained beside Rust serde
structs. One concrete example was `/settings`, where LLM interfaces were declared directly in
`src/routes/settings/+page.svelte`, while Rust owned the corresponding structs in
`src-tauri/src/llm/types.rs`.

Impact:

- DTO drift can become silent runtime breakage;
- command names are harder to search and refactor safely;
- route files carry infrastructure detail they do not need.

Suggested fix:

- introduce typed `$lib/api/*` wrappers for Tauri commands and events;
- move route-local DTOs to shared frontend type modules;
- later consider generated TypeScript types from Rust if drift remains a recurring problem.

Status after stabilization increment:

- LLM settings now has `src/lib/types/llm.ts` and `src/lib/api/llm.ts`;
- `/settings` uses the LLM wrapper instead of local DTO declarations and raw command strings.

### Major: Error typing is only partial

The backend exposes `AppError`, but many lower-level helpers still return `Result<T, String>`.
`src-tauri/src/error.rs` also classifies arbitrary strings into error kinds by substring matching.

Impact:

- changing wording can change the frontend-visible error kind;
- tests for failure modes are weaker than the apparent typed API suggests;
- behavior is harder to reason about across DB, Telegram, LLM, and validation paths.

Suggested fix:

- keep `AppError` at command/service boundaries;
- add small typed conversion helpers for DB, Telegram, LLM, and validation paths;
- reduce reliance on message heuristics over time.

### Major: Frontend lacked a unit test harness

Before the stabilization increment, `package.json` had `check` and `build`, but no frontend test script.
There were no `*.test.*` or `*.spec.*` files.

Impact:

- pure frontend logic was protected only by type checking;
- route extraction/refactoring would be riskier than necessary;
- helpers such as `analysis-utils.ts` and `app-error.ts` had no regression coverage.

Suggested fix:

- add Vitest;
- start with pure helper tests;
- then cover extracted analysis event reducers when that refactor begins.

Status after stabilization increment:

- Vitest was added;
- `analysis-utils.ts`, `app-error.ts`, and the new LLM API wrapper now have unit tests.

### Minor: Agent-facing documentation was stale

`GEMINI.md` no longer matched the real Tauri command surface in `src-tauri/src/lib.rs`.
It also described older product status such as a minimal Gemini-only settings UI.

Impact:

- future AI-agent work could follow stale contracts;
- stale docs are especially risky in this repo because they are used as modification guidance.

Suggested fix:

- keep command lists and current product status aligned with `src-tauri/src/lib.rs`, `README.md`,
  and `docs/backlog.md`.

Status after stabilization increment:

- `GEMINI.md` was refreshed to match the current command surface and product state.

## Verification Recorded During Review

- `cargo test`: passed with 130 tests, 0 failed.
- `npm.cmd run check`: passed with 0 errors and 0 warnings when run outside the sandbox so Vite/esbuild
  could spawn.

The first `npm run check` attempt failed because PowerShell blocked `npm.ps1`.
The first sandboxed `npm.cmd run check` attempt failed with `spawn EPERM` from Vite/esbuild; rerunning
outside the sandbox passed.

## Recommended Follow-Up Order

1. Keep expanding frontend tests around pure helpers and extracted reducers.
2. Extract analysis event/state reducers before splitting the full UI workflow.
3. Add typed wrappers for the next compact Tauri surface after LLM settings.
4. Split `sources.rs` and `takeout_import.rs` only along behavior boundaries already covered by tests.
5. Continue with secure secret storage as a separate backlog item, not mixed into stabilization work.
