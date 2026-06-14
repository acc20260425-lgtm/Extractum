# LLM Provider Hardening Design

**Goal:** Make Extractum's backend interaction with LLM API providers consistent, privacy-aware, retry-aware, and easier to extend beyond the current Gemini and OmniRoute names.

## Scope

This design covers the backend path used by provider tests, analysis reports, project analysis, and analysis follow-up chat. It does not introduce automatic fallback to another model or provider; provider/model unavailability still stops the current request and reports the error.

## Current Shape

The UI calls typed Tauri commands in `src/lib/api/llm.ts`. The Rust backend resolves provider profiles in `src-tauri/src/llm/profiles.rs`, dispatches provider calls through `src-tauri/src/llm/runner.rs`, and sends actual HTTP requests from `gemini.rs` or `openai_compat.rs`. Analysis report and chat flows reuse this LLM layer through `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/chat.rs`.

## Design

### Resolved Profile Snapshot

Analysis runs must use the same resolved provider profile for persistence and execution. `start_analysis_report_run` resolves the profile once, derives the effective model once, stores that provider/profile/model in `analysis_runs`, and passes the resolved profile into the spawned pipeline. The pipeline must not re-resolve the profile after the run is inserted.

The API key remains in memory only. It is not stored in `analysis_runs`, diagnostics, or frontend state.

### Sanitized Provider Errors

Provider errors should stay useful for the operator but must not persist raw payloads, prompts, API keys, bearer tokens, cookies, or secret-bearing URLs. Analysis run persistence should sanitize provider-facing error strings before writing `analysis_runs.error` and before emitting persisted failure events.

Diagnostics already aggregate errors by kind instead of selecting raw error text; that behavior remains.

### OpenAI-Compatible Provider Naming

The backend should accept `openai_compatible` as the canonical OpenAI-compatible provider key while preserving `omniroute` as a compatibility alias. Stored profiles should use the canonical key after save. Existing saved `omniroute` profiles continue to load and run.

Display text can remain "OpenAI-compatible" because the concrete gateway is represented by `base_url`.

### Retry Policy

Gemini keeps its existing retry behavior for transient 500/503/504 responses. OpenAI-compatible streaming requests gain bounded retry for transient provider responses before streaming begins:

- retry HTTP 429, 500, 502, 503, and 504;
- do not retry auth, validation, or successful streams that later fail while parsing;
- use short linear backoff;
- cancellation still wins through the existing scheduler cancellation wrapper.

### Model Limit Preflight

The current analysis preflight protects by document count, chunk count, and estimated input chars. It should also compare the per-request estimated chunk size against the selected model's known input limit when available. This check uses model metadata already available from provider model listing and must not require an extra provider call during every run unless the model catalog is being explicitly loaded.

The first implementation can expose a pure helper that validates an `AnalysisRunPreflight` against an optional model input token/character budget. UI integration can follow once backend behavior is stable.

## Test Strategy

Use Rust unit tests for provider parsing, profile resolution semantics, sanitizer behavior, retry classification, and preflight limit helpers. Use existing frontend Vitest tests only when TypeScript wire types or UI API wrappers change.

## Non-Goals

- No automatic fallback to another provider or model.
- No database storage of API keys or resolved secret material.
- No broad redesign of analysis prompts or corpus chunking.
- No new provider-specific UI beyond canonical naming compatibility.
