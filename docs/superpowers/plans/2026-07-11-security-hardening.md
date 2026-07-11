# Security Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden production Tauri capabilities/CSP and LLM credential transport while preserving `npm.cmd run tauri dev` as the MCP-enabled workflow.

**Architecture:** A production-safe base Tauri config is augmented only for `tauri dev` by a Node CLI wrapper and MCP overlay. Rust validates endpoint transport and binds stored credentials to provider/origin; keyed legacy profiles materialize their effective URL during state load and request resolution. Verification combines contract/unit tests, self-managed smoke scripts, debug/release runtime checks, and a feature-gated DevTools release build.

**Tech Stack:** SvelteKit, TypeScript/Vitest, Node.js, Tauri 2.10.x, Rust, SQLite, OS keyring, `url`, `tauri-plugin-mcp-bridge` 0.11.

## Global Constraints

- Keep `npm.cmd run tauri dev` MCP-enabled; direct `npx tauri dev` is not the supported MCP workflow.
- Production and `build --debug` use `withGlobalTauri: false`; only `cfg(dev)` registers MCP and fixture commands.
- MCP listens on `127.0.0.1`; never add `unsafe-eval` or remote CSP origins.
- Keep `dangerousDisableAssetCspModification: false`; do not compare the runtime CSP string literally.
- Remote `http://` LLM endpoints are forbidden except `localhost` and IP loopback addresses.
- Never log or return credential values. SQLite remains backend-owned and migrations remain additive.
- Use `npm.cmd` on Windows and stage only task-owned files.

---

### Task 1: Project Tauri wrapper and configuration contracts

**Files:**
- Create: `scripts/tauri.mjs`
- Create: `scripts/tauri.test.ts`
- Create: `src/lib/tauri-security-config-contract.test.ts`
- Create: `src-tauri/tauri.mcp.conf.json`
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Produces: `buildTauriArgs(args: string[]): string[]`; base production config; dev-only MCP overlay.
- Consumes: repository-local `node_modules/@tauri-apps/cli/tauri.js`.

- [ ] **Step 1: Write failing wrapper tests** covering `dev`, `build`, other commands, `--config path`, `--config=path`, `-c path`, `-c=path`, and flags after `--`.

```ts
expect(buildTauriArgs(["dev"])).toEqual(["dev", "--config", "src-tauri/tauri.mcp.conf.json"]);
expect(buildTauriArgs(["dev", "--config=x.json"])).toEqual(["dev", "--config=x.json"]);
expect(buildTauriArgs(["dev", "--", "--config=x.json"])).toEqual(["dev", "--config", "src-tauri/tauri.mcp.conf.json", "--", "--config=x.json"]);
expect(buildTauriArgs(["build", "--debug"])).toEqual(["build", "--debug"]);
```

- [ ] **Step 2: Run the focused test and confirm RED.**

Run: `npm.cmd run test -- scripts/tauri.test.ts`
Expected: FAIL because `scripts/tauri.mjs` does not exist.

- [ ] **Step 3: Implement the wrapper and package entry.** Export `buildTauriArgs`, scan only before the first `--`, spawn `node` with the local Tauri CLI, and propagate exit code/signals.

```js
export function buildTauriArgs(args) {
  const delimiter = args.indexOf("--");
  const commandArgs = args.slice(0, delimiter < 0 ? args.length : delimiter);
  const hasConfig = commandArgs.some((arg) => arg === "--config" || arg === "-c" || arg.startsWith("--config=") || arg.startsWith("-c="));
  if (args[0] !== "dev" || hasConfig) return [...args];
  return ["dev", "--config", "src-tauri/tauri.mcp.conf.json", ...args.slice(1)];
}
```

Set `"tauri": "node scripts/tauri.mjs"` in `package.json`.

- [ ] **Step 4: Write failing config contract tests** asserting base `withGlobalTauri === false`, CSP directives, asset CSP modification enabled, overlay changes only `withGlobalTauri`, and no SQL permissions.

- [ ] **Step 5: Apply the production config.** Use the approved CSP, set `dangerousDisableAssetCspModification` to `false`, create an overlay containing only `{ "app": { "withGlobalTauri": true } }`, and remove all `sql:*` permissions.

- [ ] **Step 6: Run focused tests and commit.**

Run: `npm.cmd run test -- scripts/tauri.test.ts src/lib/tauri-security-config-contract.test.ts`
Expected: PASS.

```powershell
git add package.json scripts/tauri.mjs scripts/tauri.test.ts src/lib/tauri-security-config-contract.test.ts src-tauri/tauri.conf.json src-tauri/tauri.mcp.conf.json src-tauri/capabilities/default.json
git commit -m "feat: separate production and MCP Tauri config"
```

### Task 2: Development-only Rust commands and localhost MCP

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/tauri-security-config-contract.test.ts`

**Interfaces:**
- Produces: MCP builder `Builder::new().bind_address("127.0.0.1").build()` registered only under `cfg(dev)`.

- [ ] **Step 1: Extend the source contract test** to require `#[cfg(dev)]` around MCP plus every `seed_*`/`clear_*` import and handler registration. Reject `#[cfg(debug_assertions)]` only when it gates those named MCP/fixture blocks; do not ban `debug_assertions` elsewhere in `lib.rs`.
- [ ] **Step 2: Run RED.**

Run: `npm.cmd run test -- src/lib/tauri-security-config-contract.test.ts`
Expected: FAIL on current `debug_assertions` and default MCP initializer.

- [ ] **Step 3: Replace all development tooling gates and MCP initialization.**

```rust
#[cfg(dev)]
let builder = builder.plugin(
    tauri_plugin_mcp_bridge::Builder::new()
        .bind_address("127.0.0.1")
        .build(),
);
```

- [ ] **Step 4: Verify and commit.**

Run: `npm.cmd run test -- src/lib/tauri-security-config-contract.test.ts`
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

```powershell
git add src-tauri/src/lib.rs src/lib/tauri-security-config-contract.test.ts
git commit -m "feat: restrict development Tauri commands"
```

### Task 3: LLM endpoint transport validation

**Files:**
- Modify: `src-tauri/src/llm/mod.rs`

**Interfaces:**
- Produces: `normalize_base_url` accepting HTTPS and loopback HTTP only.

- [ ] **Step 1: Add table-driven failing tests** for HTTPS, `localhost`, `127.0.0.1`, `127.1.2.3`, `[::1]`, remote IPv4/IPv6 HTTP, hostname HTTP, and unsupported schemes.

```rust
assert!(normalize_base_url(ProviderKind::OpenAiCompatible, Some("http://[::1]:8080/v1")).is_ok());
assert!(normalize_base_url(ProviderKind::OpenAiCompatible, Some("http://example.com/v1")).is_err());
```

- [ ] **Step 2: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml normalize_base_url -- --nocapture`
Expected: remote HTTP case FAILS.

- [ ] **Step 3: Validate parsed hosts.** Accept HTTP only for case-insensitive `localhost` or `url.host().and_then(|h| h.parse::<IpAddr>().ok()).is_some_and(|ip| ip.is_loopback())`; return `AppError::validation` before request construction.
- [ ] **Step 4: Run PASS and commit.**

```powershell
git add src-tauri/src/llm/mod.rs
git commit -m "fix: reject remote plaintext LLM endpoints"
```

### Task 4: Credential-origin binding and materialization

**Files:**
- Modify: `src-tauri/src/llm/mod.rs`
- Modify: `src-tauri/src/llm/profiles.rs`
- Inspect/Modify: `src/routes/settings/+page.svelte`
- Modify: `src/lib/api/llm.test.ts`
- Modify: `src/lib/settings-profile-ux-contract.test.ts`

**Interfaces:**
- Produces: normalized credential scope `(ProviderKind, Option<Origin>)`; idempotent `materialize_keyed_profile_base_url`; fail-closed state loading.

- [ ] **Step 1: Add failing scope tests** for provider/scheme/host/effective-port changes, path-only changes, blank versus explicit default, replacement key, cleared key, and unchanged scope.
- [ ] **Step 2: Add failing persistence tests** proving keyed legacy profiles are written during both state load and resolution, unkeyed profiles remain blank, repeat materialization is harmless, and a forced DB write failure makes state load return an error. Make the test pool single-connection with `SqlitePoolOptions::new().max_connections(1)`, seed the keyed legacy profile, then execute `PRAGMA query_only = ON`; reads and keyring lookup still succeed, while the materialization write deterministically fails. This implements the fail-closed rule already recorded in the design's Error Handling section.

```rust
assert_eq!(state.profiles[0].base_url, DEFAULT_OPENAI_COMPAT_BASE_URL);
assert_eq!(read_setting(&pool, &profile_base_url_key("default")).await?, Some(DEFAULT_OPENAI_COMPAT_BASE_URL.into()));
```

- [ ] **Step 3: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml llm::profiles::tests -- --nocapture`
Expected: new scope/materialization tests FAIL.

- [ ] **Step 4: Implement scope comparison and save validation.** Expand defaults before origin comparison; require a replacement key or prior clear when scope changes; persist the effective URL whenever a key is created, replaced, or retained.
- [ ] **Step 5: Implement shared idempotent materialization.** After keyring lookup, write the normalized effective URL when keyed and missing/blank; call it from state loading and request resolution, propagating DB errors without logging secrets.
- [ ] **Step 6: Verify the frontend consequence explicitly.** Add an API test whose `get_llm_profiles` response contains a keyed OpenAI-compatible profile with `base_url: "http://localhost:20128/v1"` and assert the wrapper preserves that explicit value. Inspect `+page.svelte` and add/update the settings contract to require `baseUrl = profile.base_url`, so selecting the profile displays the materialized URL rather than replacing it with an empty/default sentinel. If the existing assignment already satisfies the contract, retain the production code unchanged and record that fact in the test name.

```ts
invokeMock.mockResolvedValueOnce({
  active_profile: "work",
  profiles: [{ profile_id: "work", provider: "openai_compatible", default_model: "model", api_key_configured: true, base_url: "http://localhost:20128/v1" }],
});
await expect(getLlmProfiles()).resolves.toMatchObject({ profiles: [{ base_url: "http://localhost:20128/v1" }] });
```

- [ ] **Step 7: Run Rust and frontend LLM/settings tests and commit.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml llm -- --nocapture`
Run: `npm.cmd run test -- src/lib/api/llm.test.ts src/lib/settings-profile-ux-contract.test.ts`
Expected: PASS.

```powershell
git add src-tauri/src/llm/mod.rs src-tauri/src/llm/profiles.rs src/routes/settings/+page.svelte src/lib/api/llm.test.ts src/lib/settings-profile-ux-contract.test.ts
git commit -m "feat: bind LLM keys to provider origins"
```

### Task 5: Verification-only production DevTools

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/tauri-security-config-contract.test.ts`

**Interfaces:**
- Produces: Cargo feature `csp-verification = ["tauri/devtools"]`; feature-gated `open_devtools()` hook.

- [ ] **Step 1: Add a failing source contract** requiring the feature mapping and `#[cfg(feature = "csp-verification")]` around `open_devtools()`.
- [ ] **Step 2: Run RED, implement the feature/hook, then run PASS.** The hook obtains `app.get_webview_window("main")` and calls `open_devtools()`; normal builds compile without the feature.
- [ ] **Step 3: Verify both feature sets and commit.**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Run: `cargo check --manifest-path src-tauri/Cargo.toml --features csp-verification`
Expected: both PASS.

```powershell
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src/lib/tauri-security-config-contract.test.ts
git commit -m "test: add production CSP inspection feature"
```

### Task 6: Project documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/project.md`
- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `AGENTS.md`
- Modify: `.claude/skills/run-app/SKILL.md`
- Inspect: `docs/value-registry.md`

**Interfaces:**
- Produces: current command, security boundary, credential, smoke ordering, and CSP verification documentation.

- [ ] **Step 1: Update docs** with the unchanged MCP command, direct `npx` limitation, non-MCP `build --debug`, localhost binding, removed frontend SQL access, HTTPS/loopback rule, visible keyed URL materialization, fail-closed state load, dormant-legacy limitation, and manual CSP procedure.
- [ ] **Step 2: Update agent workflow** to use `npm.cmd run tauri dev`, forbid frontend SQL permissions, and preserve smoke ordering.
- [ ] **Step 3: Inspect value registry.** Do not edit it unless implementation introduced a controlled string value; `csp-verification` is a Cargo feature, not persisted domain state.
- [ ] **Step 4: Validate links/text and commit.**

Run: `rg -n "npx tauri dev|npm.cmd run tauri dev|csp-verification|remote.*http|SQL" README.md docs/project.md docs/browser-providers-llm-troubleshooting.md AGENTS.md .claude/skills/run-app/SKILL.md`
Expected: the current workflow and limitations appear in all owning documents.

```powershell
git add README.md docs/project.md docs/browser-providers-llm-troubleshooting.md AGENTS.md .claude/skills/run-app/SKILL.md
git commit -m "docs: document security hardening workflow"
```

### Task 7: Full automated and live verification

**Files:**
- Create: `docs/superpowers/verification/2026-07-11-security-hardening.md`

**Interfaces:**
- Consumes: all preceding tasks.
- Produces: reproducible evidence and any precisely documented baseline failures.

- [ ] **Step 1: Run automated verification.**

```powershell
npm.cmd run test
npm.cmd run check
npm.cmd run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: PASS, except a pre-existing failure must be recorded with evidence and must not conceal a new regression.

- [ ] **Step 2: Run manual MCP verification.** Start `npm.cmd run tauri dev`; obtain the discovered MCP Bridge port, then verify its actual listener address:

```powershell
$appPid = (Get-Process -Name extractum | Sort-Object StartTime -Descending | Select-Object -First 1).Id
Get-NetTCPConnection -State Listen | Where-Object { $_.OwningProcess -eq $appPid } | Select-Object LocalAddress, LocalPort, OwningProcess
```

Expected: the row whose port matches MCP discovery has `LocalAddress` exactly `127.0.0.1`; no row for that port uses `0.0.0.0`, `::`, or a LAN address. Then confirm window discovery, JS result, screenshot, element picker, IPC monitor start/capture/stop; run `npm.cmd run smoke:cancellation` and stop the app.
- [ ] **Step 3: Run self-managed analysis smoke only after ports are free.**

Run: `npm.cmd run smoke:analysis`
Expected: PASS with fixture cleanup; it launches and stops its own isolated app.

- [ ] **Step 4: Verify debug build boundary.** Build `npm.cmd run tauri build -- --debug --no-bundle`, launch it, confirm no MCP listener, and in DevTools require `window.__TAURI_INTERNALS__.invoke("seed_analysis_redesign_fixtures")` to reject as unknown command.
- [ ] **Step 5: Verify the normal release.** Build `npm.cmd run tauri build -- --no-bundle`, launch it, and smoke analysis, projects, library, settings, and diagnostics with no MCP listener.
- [ ] **Step 6: Manually inspect production CSP.** Build `npm.cmd run tauri build -- --no-bundle --features csp-verification`, repeat the release smoke in the automatically opened DevTools, and confirm no CSP refusal messages.
- [ ] **Step 7: Record exact commands/results and commit evidence.**

```powershell
git add docs/superpowers/verification/2026-07-11-security-hardening.md
git commit -m "docs: record security hardening verification"
```
