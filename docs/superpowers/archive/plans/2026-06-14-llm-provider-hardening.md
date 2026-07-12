# LLM Provider Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden Extractum's LLM provider backend so analysis runs use stable resolved profile snapshots, persist sanitized provider errors, support canonical OpenAI-compatible naming, retry transient OpenAI-compatible failures, and expose model-limit preflight checks.

**Architecture:** Keep the existing provider adapter boundary in `src-tauri/src/llm`. Analysis report execution receives a resolved profile captured at run start instead of resolving again later. Provider retry and error sanitation live close to the Rust backend services that own those behaviors.

**Tech Stack:** Rust/Tauri 2, `reqwest`, `sqlx`, `tokio`, SvelteKit/Vitest for frontend command wrappers when wire types change.

---

## File Structure

- Modify `src-tauri/src/llm/mod.rs`: canonical provider parsing and display naming.
- Modify `src-tauri/src/llm/profiles.rs`: canonical storage of OpenAI-compatible provider keys while accepting aliases.
- Modify `src-tauri/src/llm/runner.rs`: dispatch canonical OpenAI-compatible provider kind.
- Modify `src-tauri/src/llm/openai_compat.rs`: transient retry classification and retry loop.
- Modify `src-tauri/src/analysis/report.rs`: pass resolved profile snapshot into the spawned pipeline and sanitize persisted provider failures.
- Modify `src-tauri/src/analysis/store.rs`: add reusable provider error sanitizer near existing snapshot sanitizer.
- Modify `src-tauri/src/analysis/corpus.rs`: add model-limit preflight helper.
- Test existing Rust modules in place with targeted `cargo test` filters.

---

### Task 1: Stable Resolved Profile Snapshot

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`

- [ ] **Step 1: Write the failing test**

Add a unit test in `src-tauri/src/analysis/report.rs` proving `ReportRunInput` carries a `ResolvedLlmProfile` and no longer carries a late-bound `profile_id`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p extractum --lib analysis::report::tests::report_run_input_carries_resolved_profile_snapshot`

Expected: fail because `ReportRunInput` still has `profile_id` instead of `resolved_profile`.

- [ ] **Step 3: Implement minimal code**

Change `ReportRunInput` to include `resolved_profile: ResolvedLlmProfile`; pass `resolved_profile.clone()` from `start_analysis_report_run`; remove the second `resolve_profile_for_backend` call inside `run_report_pipeline`.

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p extractum --lib analysis::report`

Expected: pass.

- [ ] **Step 5: Commit**

Commit message: `fix: use resolved llm profile snapshot for analysis runs`

### Task 2: Sanitized Provider Errors

**Files:**
- Modify: `src-tauri/src/analysis/store.rs`
- Modify: `src-tauri/src/analysis/report.rs`

- [ ] **Step 1: Write the failing test**

Add tests in `analysis::store::tests` for `sanitize_provider_error`, including API key, bearer token, raw prompt/payload text, and query-bearing URLs.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p extractum --lib analysis::store::tests::sanitize_provider_error_redacts_provider_payloads`

Expected: fail because the helper does not exist.

- [ ] **Step 3: Implement minimal code**

Add `sanitize_provider_error(category: &str, raw: &str) -> String` using the existing snapshot sanitizer as the base behavior. Use it in `fail_run` before writing `analysis_runs.error` and emitting the persisted failure event.

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p extractum --lib analysis::store analysis::report`

Expected: pass.

- [ ] **Step 5: Commit**

Commit message: `fix: sanitize persisted provider errors`

### Task 3: Canonical OpenAI-Compatible Provider Key

**Files:**
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/llm/profiles.rs`
- Modify: `src-tauri/src/llm/runner.rs`
- Modify: `src-tauri/src/llm/openai_compat.rs`
- Modify frontend strings only if TypeScript tests require it.

- [ ] **Step 1: Write failing tests**

Add Rust tests showing `ProviderKind::parse("openai_compatible")` and `ProviderKind::parse("omniroute")` both resolve to the OpenAI-compatible kind, and `ProviderKind::as_str()` returns `openai_compatible`.

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p extractum --lib llm::tests::provider_parse_accepts_openai_compatible_aliases`

Expected: fail because the canonical key is still `omniroute`.

- [ ] **Step 3: Implement minimal code**

Rename the enum variant to `OpenAiCompatible` or keep the variant while changing `as_str()` to `openai_compatible`; update matches in `mod.rs`, `runner.rs`, and `openai_compat.rs`. Preserve `omniroute` in `parse` as an alias.

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p extractum --lib llm`

Expected: pass.

- [ ] **Step 5: Commit**

Commit message: `feat: canonicalize openai compatible provider key`

### Task 4: Retry OpenAI-Compatible Transient Failures

**Files:**
- Modify: `src-tauri/src/llm/openai_compat.rs`

- [ ] **Step 1: Write failing tests**

Add pure unit tests for `is_retryable_openai_compat_status` covering 429, 500, 502, 503, 504 as retryable and 400, 401, 403, 404 as not retryable.

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p extractum --lib llm::openai_compat::tests::openai_compat_retry_status_policy_is_bounded_to_transient_failures`

Expected: fail because the helper does not exist.

- [ ] **Step 3: Implement minimal code**

Add constants for max attempts and retry delay. Wrap the initial `POST /chat/completions` send in a retry loop that retries only retryable HTTP statuses before streaming begins.

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p extractum --lib llm::openai_compat`

Expected: pass.

- [ ] **Step 5: Commit**

Commit message: `feat: retry transient openai compatible failures`

### Task 5: Model-Limit Preflight Helper

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`

- [ ] **Step 1: Write failing tests**

Add tests for a pure helper that returns `None` when no model input limit is known, returns `None` when estimated chunk size fits, and returns a clear validation message when estimated chunk size exceeds the known limit.

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p extractum --lib analysis::corpus::tests::model_limit_preflight_reports_oversized_chunks`

Expected: fail because the helper does not exist.

- [ ] **Step 3: Implement minimal code**

Add a helper that estimates the largest per-chunk input size from `estimated_input_chars / estimated_chunks` and compares it with an optional model input budget. Keep existing global preflight behavior unchanged.

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p extractum --lib analysis::corpus`

Expected: pass.

- [ ] **Step 5: Commit**

Commit message: `feat: add model limit preflight helper`

### Task 6: Final Verification

**Files:**
- No code changes unless verification exposes a regression.

- [ ] **Step 1: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all Rust tests pass.

- [ ] **Step 2: Run frontend tests**

Run: `npm.cmd test`

Expected: all Vitest tests pass.

- [ ] **Step 3: Run Svelte check**

Run: `npm.cmd run check`

Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Review git diff**

Run: `git diff --stat`

Expected: only LLM provider hardening docs and scoped backend changes.

---

## Self-Review

- Spec coverage: all five priority items are represented by Tasks 1-5.
- Placeholder scan: no deferred sections; each task has concrete files, commands, and acceptance criteria.
- Type consistency: plan uses existing `ResolvedLlmProfile`, `ProviderKind`, `AnalysisRunPreflight`, and Rust module names.
