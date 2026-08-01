# Extractum: AI Agent Guidelines

## 1. Architecture & Commands
- Extractum uses a fat frontend / thin backend model: SvelteKit and TypeScript own user-flow orchestration; Rust and Tauri own Telegram, SQLite, compression, session persistence, and other low-level integrations.
- Prefer small, explicit Tauri commands over broad generic commands.

## 2. Windows, Vite & Playwright Sandbox Rules
- On Windows, use `npm.cmd` rather than `npm` for repository scripts.
- Use `npm.cmd run tauri dev` for MCP-enabled Tauri development; direct `npx tauri dev` does not load the MCP overlay.
- Do not assume Vite uses port `5173`. Use the actual local URL printed by Vite.
- If a sandboxed server exits immediately or a self-managed analysis smoke check is required, follow the [Windows sandbox fallback and smoke order](docs/project.md#windows-sandbox-fallback).
- Browser console errors from missing Tauri IPC are expected when testing in Playwright.

## 3. Workflow Rules
- Update docs when behavior, architecture, commands, or agent workflow rules change.
- When adding or changing `status`, `state`, `kind`, `mode`, `phase`, `type`, `provider`, `subtype`, `scope`, `severity`, or similar string values, update `docs/value-registry.md` and check owner, persistence/API impact, UI impact, and fixtures.

## 4. Validation Rules
<!-- daily-development-loop -->
- Use the smallest relevant daily-loop command after a small change:
  - dirty frontend work: `npm.cmd run test:changed`;
  - the most recent linear checkpoint: `npm.cmd run test:changed:last`;
  - a known frontend source: `npm.cmd run test:related -- <forward-or-backslash-path>`;
  - broad Svelte/TypeScript work: also run `npm.cmd run check`.

<!-- focused-rust-loop -->
- Every implementation plan that changes Rust must include a `## Rust Verification Loops` section naming affected packages, narrow RED/GREEN tests, focused checks, package checkpoints, and end-of-slice workspace gates.
- After a small Rust change, use the owning package explicitly:
  - exact RED/GREEN test: `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact`;
  - focused check: `cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`;
  - task checkpoint: `cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets`.
- Changed, related, and focused package commands are accelerators, not completion evidence. An empty or unexpectedly small selection requires an explicit test or a wider run; `npm.cmd run verify` is the full gate and must run at the end of every Rust slice.
- Use `-p extractum` while code belongs to the application; use the extracted domain package after it moves. Check every directly affected package separately.
- A filtered Cargo run that reports `0 tests` is not verification. List tests first when the exact name is unknown, then run a non-empty selection.
- When a public cross-crate interface changes, checkpoint the immediate dependent package; unchanged internal work does not pay that cost after every edit.
- Every workspace member shares canonical `src-tauri/target`; do not create slice-specific `codex-*` targets for sequential work.
- For rare dependency-level native debugging, follow the [native-debug procedure](docs/project.md#native-dependency-debugging) rather than changing the normal build loop.

<!-- cargo-test-support-features -->
- Cross-package test-only seams use a narrow, non-default `*-test-support` feature enabled for the package only through the consumer's `[dev-dependencies]`; keep its normal `[dependencies]` edge feature-free. This provides development isolation, not a security boundary: expose only behavior-neutral helpers, never production capabilities or security controls.
- With Cargo resolver v2, consumer tests, examples, benches, and consumer `cargo check --all-targets` / `cargo test --all-targets` activate and unify the dev feature. Both `--all-targets` commands are feature-on evidence and cannot prove feature-off.
- Prove the producer's production surface with `cargo check --manifest-path src-tauri/Cargo.toml -p <producer> --lib --no-default-features`; prove the seam from its consumer with `cargo test --manifest-path src-tauri/Cargo.toml -p <consumer> --all-targets`. See `docs/superpowers/specs/2026-08-01-telegram-8c-extraction-design.md` for the worked example.

## 5. Data Grid & Date Formatting
- `ExtractumDataGrid` date/time columns use raw ISO strings, Unix seconds, Unix milliseconds, or `Date` values and opt into shared formatting only through an explicitly set `dateTimeFormat`.
- Do not pre-format grid date/time values into label-only columns.

## 6. Backend, Database & Migrations
- Keep SQLite migrations additive: do not delete, rename, or rewrite existing migration files.
- Do not introduce a second manual SQLite path or duplicate database connection strategy.
- Do not grant frontend `sql:*` permissions; SQLite remains backend-owned.

## 7. Security
- Saved LLM API keys belong in OS secure storage, not SQLite profile settings. When touching LLM profile or credential code, preserve profile scoping and provider-plus-normalized-origin binding.
