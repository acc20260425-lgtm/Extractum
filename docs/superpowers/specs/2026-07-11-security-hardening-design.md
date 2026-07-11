# Security Hardening Design

**Date:** 2026-07-11  
**Status:** revised after design review; awaiting renewed approval

## Goal

Reduce the production webview and LLM credential blast radius without changing
the established local Tauri MCP Bridge development command:

```powershell
npm.cmd run tauri dev
```

Production builds must not expose the global Tauri object or frontend SQLite
commands. MCP-assisted development must retain script execution, IPC
inspection, screenshots, element picking, and event forwarding.

## Scope

This slice includes:

- production and MCP-development Tauri configuration separation;
- removal of frontend SQL permissions;
- a production Content Security Policy;
- localhost-only MCP Bridge binding;
- preservation of `npm.cmd run tauri dev` through a CLI wrapper;
- rejection of remote plaintext LLM endpoints;
- protection against silently reusing an existing LLM key after its provider or
  network origin changes;
- automated regression coverage and project documentation updates.

This slice does not include startup cleanup barriers, project deletion/run
coordination, `yt-dlp` process ownership, frontend request races, or broader UI
refactoring. Those belong to later stabilization increments.

## Current Risks

The base Tauri configuration currently enables `withGlobalTauri` and disables
CSP. The main window capability grants `sql:allow-load`, `sql:allow-select`, and
`sql:allow-execute`, although frontend code does not import or use the SQL
plugin. This lets webview code bypass the explicit Rust command boundary.

LLM API keys are keyed only by profile id. Saving an existing profile without a
new key intentionally preserves the current secret. Today that preservation
also applies after changing provider or endpoint, and the OpenAI-compatible
base URL accepts remote `http://` hosts. Together these behaviors can send a
previously stored key to a different provider or over plaintext transport.

The MCP Bridge is a special debug dependency. Version `0.11.0` injects scripts
that call `window.__TAURI__.core.invoke` and `window.__TAURI__.event`; disabling
the global object for all builds would break important MCP operations.

Disabling `withGlobalTauri` is defense in depth, not the primary authorization
boundary. Bundled frontend modules can still call Tauri through
`@tauri-apps/api`. CSP and least-privilege capabilities, especially removal of
frontend SQL permissions, are the controls that restrict what compromised
webview code can do.

## Selected Approach

Use defense in depth with separate production and MCP-development
configuration, while keeping the existing npm command as the public developer
interface.

Alternatives rejected:

- Keeping `withGlobalTauri` enabled everywhere preserves the current workflow
  but unnecessarily exposes the global API in production.
- Introducing a separate user-facing MCP command is mechanically simple but
  changes the established project workflow.
- Storing a different secret for every provider/origin pair requires credential
  migration and is disproportionate for this stabilization slice.

## Tauri Configuration

### Production base

`src-tauri/tauri.conf.json` is the production-safe base configuration:

- `app.withGlobalTauri` is `false`;
- `app.security.csp` is non-null;
- `app.security.dangerousDisableAssetCspModification` remains `false`;
- the CSP allows bundled application resources, Tauri IPC, local asset images,
  data/blob images, fonts, and inline styles required by the current Svelte UI;
- it does not add `unsafe-eval`, arbitrary remote script sources, or arbitrary
  remote connection sources.

The intended production policy is equivalent to:

```text
default-src 'self';
connect-src 'self' ipc: http://ipc.localhost;
img-src 'self' asset: http://asset.localhost data: blob:;
style-src 'self' 'unsafe-inline';
font-src 'self' data:;
script-src 'self'
```

The exact JSON representation may be a string or Tauri directive map, but the
resulting policy must preserve these boundaries.

Tauri modifies the effective CSP for bundled assets at compile time by adding
the nonces and hashes required by application-owned inline scripts and styles.
That modification must remain enabled because SvelteKit's generated HTML
contains an inline hydration bootstrap. Configuration tests validate the
intended directives and that asset CSP modification is enabled; they must not
compare the final runtime CSP string literally. A bundled-application smoke
test verifies that hydration actually succeeds.

### MCP development overlay

Create `src-tauri/tauri.mcp.conf.json`, merged through Tauri CLI `--config` for
development only. It:

- sets only `app.withGlobalTauri` to `true`;
- does not override production CSP or add network origins.

Desktop `tauri dev` loads `devUrl` directly as an external HTTP page from Vite.
Source inspection of the locked Tauri `2.10.3` implementation confirms that
`devCsp` is selected internally but is not injected into this external
response; dev-server proxying is enabled only for mobile. Therefore the MCP
overlay must not pretend to enforce CSP or contain an HMR WebSocket exception.

Vite has two HMR branches, neither of which belongs in the Tauri overlay. The
normal desktop workflow leaves `TAURI_DEV_HOST` unset, so HMR uses the dev
server origin on port `1420`. With `TAURI_DEV_HOST` set for host/mobile use,
Vite explicitly uses that host on port `1421`. Any future development CSP must
be enforced by the Vite server itself and cover these branches dynamically.

The MCP Bridge permission remains available to the main window only during
`tauri dev`. Change Rust plugin registration from `cfg(debug_assertions)` to
`cfg(dev)`. Consequently, `tauri build --debug` remains a normal non-MCP bundle
with `withGlobalTauri: false`, rather than a partially enabled bridge.

Apply the same `cfg(dev)` boundary to the existing `seed_*` and `clear_*` smoke
fixture command imports and registrations. They are development tooling and
must not remain callable from a `tauri build --debug` webview.

### Window capabilities

Remove the frontend SQL permission set from
`src-tauri/capabilities/default.json`:

- `sql:default`;
- `sql:allow-execute`;
- `sql:allow-select`;
- `sql:allow-load`.

Retain permissions that have current frontend consumers, including
`mcp-bridge:default` for debug builds. Removing webview SQL permissions does not
remove the Rust `tauri-plugin-sql` pool or migrations; backend SQLite ownership
stays unchanged.

## Preserving the Existing Development Command

Keep these public commands and meanings:

```powershell
npm.cmd run tauri dev
npm.cmd run tauri build
```

Replace the direct package script with a small cross-platform Node wrapper. The
wrapper forwards all arguments to the repository-local Tauri CLI and:

- when the first Tauri subcommand is `dev`, adds
  `--config src-tauri/tauri.mcp.conf.json` unless the caller already supplied a
  config override;
- when the subcommand is `build`, forwards arguments without the MCP overlay;
- for other subcommands, forwards arguments unchanged;
- propagates the child exit code and termination signal.

Config detection recognizes `--config path`, `--config=path`, `-c path`, and
`-c=path` before the first `--` argument delimiter. Arguments after that
delimiter belong to the launched application and do not suppress the project
overlay.

Therefore the familiar `npm.cmd run tauri dev` remains MCP-enabled, while
`npm.cmd run tauri build` always starts from the production-safe base.

Direct `npx tauri dev` bypasses this project wrapper. It is not a supported MCP
launch command unless the caller supplies the MCP config explicitly.

## MCP Bridge Boundary

Replace the default MCP Bridge initialization with a builder configured for
`127.0.0.1`. The development bridge must not listen on `0.0.0.0` because this
desktop workflow does not require remote-device access.

This is supported by the locked `tauri-plugin-mcp-bridge` version `0.11.0` via
`Builder::new().bind_address("127.0.0.1").build()` and is therefore not a
planning assumption.

The implementation is accepted only if live MCP verification confirms:

- connection through localhost;
- window discovery;
- JavaScript execution with a returned result;
- screenshot capture;
- element picker event delivery;
- IPC monitor start, event capture, and stop.

Failure under the production CSP must not be addressed by relaxing production
policy without proving the exact bundled application requirement. The MCP
overlay does not alter CSP. If an enforced development-server CSP is added in a
future slice and the bridge requires `unsafe-eval`, implementation stops for
design review instead of adding it automatically.

## LLM Endpoint Transport Policy

OpenAI-compatible base URLs continue to accept `https://` endpoints.

Plain `http://` is accepted only when the parsed host is local:

- exact hostname `localhost`;
- an IPv4 address for which `IpAddr::is_loopback()` is true;
- an IPv6 address for which `IpAddr::is_loopback()` is true.

Coverage includes bracketed IPv6 URL syntax such as `http://[::1]:8080`.

Remote HTTP hosts return a typed validation error before any network request is
created. Existing loopback defaults such as `http://localhost:20128/v1` remain
valid.

## LLM Credential Scope Policy

An existing configured key may be preserved on save only when its credential
scope is unchanged. Credential scope consists of:

- normalized provider kind; and
- normalized URL origin: scheme, host, and effective port.

The origin is computed from the effective endpoint after expanding a blank or
missing base URL to the provider default. For OpenAI-compatible profiles, an
empty base URL and the explicit current default
`http://localhost:20128/v1` therefore have the same origin. A provider without
a configurable endpoint has no URL origin component. If a future release
changes a provider default, existing keys must remain bound to the origin at
which they were configured.

The persistence mechanism makes that rule executable: whenever a save creates,
replaces, or preserves a configured key, it writes the effective normalized
base URL explicitly instead of storing a blank/default sentinel. During this
slice, a one-time startup backfill also materializes the current effective base
URL for every legacy profile that has a configured key but a missing or blank
endpoint setting. The backfill runs before such profiles can be resolved for a
request. Profiles without keys may continue to use an implicit provider
default.

Path-only changes under the same origin do not require a new key. Provider,
scheme, host, or effective-port changes do.

When a configured key exists and credential scope changes:

- a non-empty replacement key allows the save;
- an empty/missing replacement key returns a typed validation error;
- clearing the stored key first allows saving the changed profile in an
  unconfigured state.

The validation error must tell the user to enter a new key or clear the saved
key before changing provider/endpoint. It must not include the key or other
secret values.

New profiles without a key and existing profiles without a configured key may
still be saved. Normal edits to model, label-independent settings, or a path on
the same origin preserve the existing key.

## Error Handling

- Configuration wrapper failures print the failed executable/action and exit
  non-zero without printing environment variables or secrets.
- CSP and capability regression tests fail with messages naming the forbidden
  permission or configuration value.
- Endpoint and credential-scope errors use `AppError::validation`.
- Network code remains responsible for runtime connection errors only after
  configuration validation succeeds.

## Testing Strategy

Implementation follows red-green-refactor cycles.

Automated coverage must include:

- wrapper arguments for `dev`, `build`, other subcommands, all supported config
  flag forms, and the `--` delimiter;
- base config has `withGlobalTauri: false`, a non-null CSP, and asset CSP
  modification enabled;
- MCP overlay changes only `withGlobalTauri` and contains no CSP/HMR exception;
- frontend capability contains no SQL permissions;
- MCP plugin binding is localhost-only;
- source-level contract checks confirm MCP and fixture command imports and
  registrations use `cfg(dev)`, not `debug_assertions`;
- remote HTTP is rejected while HTTPS, IPv4 loopback HTTP, and bracketed IPv6
  loopback HTTP are accepted;
- blank/missing and explicit provider-default endpoints normalize to the same
  credential origin;
- saves with configured keys materialize their effective base URL;
- startup backfill materializes legacy keyed profiles before request use;
- unchanged provider/origin preserves a key;
- provider, scheme, host, and port changes require a replacement key;
- path-only changes preserve a key;
- clearing the old key permits an unconfigured scope change;
- replacement-key saves succeed and replace the stored secret.

Focused tests run first. Final verification includes:

```powershell
npm.cmd run test
npm.cmd run check
npm.cmd run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

The two currently stale raw-source frontend contract failures are a known
baseline issue. This slice must either fix them when its config tests touch the
same contracts or report them explicitly; it must not conceal new failures.

Live MCP verification uses the unchanged command:

```powershell
npm.cmd run tauri dev
```

Production CSP verification must additionally build and launch bundled assets:

```powershell
npm.cmd run tauri build -- --no-bundle
```

Launch the generated release executable and smoke-test hydration plus the main
screens: analysis, projects, project library, settings, and diagnostics. Confirm
that navigation and basic interactions work and that no MCP Bridge listener is
present.

Absence of CSP violations is observed through a separate verification-only
release-profile build. Add a Cargo feature such as `csp-verification` that
enables `tauri/devtools`, plus a feature-gated setup hook that opens DevTools.
Build it with the same production CSP and bundled assets using a config overlay
that does not modify security policy:

```powershell
npm.cmd run tauri build -- --no-bundle --features csp-verification --config src-tauri/tauri.csp-verification.conf.json
```

Inspect the WebView console while repeating the smoke navigation and require no
`securitypolicyviolation`/CSP refusal messages. Automated config tests must
prove that this overlay changes only the window's DevTools setting. The normal
release build remains free of the feature and cannot open DevTools. These
checks are required because the Vite development server cannot prove that
Tauri's compile-time CSP asset modification and production asset protocol work
correctly.

## Documentation

The implementation must update current project documentation, not only this
design record:

- `README.md`: development and production commands, MCP behavior, and security
  boundary;
- `docs/project.md`: verification/launch workflow and credential transport
  rules;
- `docs/browser-providers-llm-troubleshooting.md`: MCP launch and connection
  troubleshooting;
- `AGENTS.md`: preserve `npm.cmd run tauri dev`, explain the automatic MCP
  overlay, and prohibit direct frontend SQL permissions;
- `.claude/skills/run-app/SKILL.md`: use the project npm command instead of
  direct `npx tauri dev`;
- `docs/value-registry.md` only if implementation introduces a new controlled
  status, state, kind, mode, provider, scope, reason, or similar machine value.

Documentation must state explicitly that direct `npx tauri dev` does not apply
the project MCP overlay automatically. It must also state that
`tauri build --debug` is a non-MCP application build; MCP debugging is provided
only by the project `tauri dev` workflow.

## Acceptance Criteria

- `npm.cmd run tauri dev` launches an MCP-enabled debug app without extra user
  arguments.
- `npm.cmd run tauri build` uses `withGlobalTauri: false` and the production CSP.
- `npm.cmd run tauri build -- --debug` does not register the MCP Bridge.
- `npm.cmd run tauri build -- --debug` exposes none of the development fixture
  commands gated by `cfg(dev)`.
- The built release application hydrates and its main screens work under the
  effective production CSP; the verification-only DevTools build shows no CSP
  violations during the same smoke path.
- MCP Bridge listens only on localhost and passes the defined live smoke checks.
- The main webview cannot load, select, or execute SQLite through plugin
  permissions.
- Remote plaintext LLM endpoints are rejected before requests are sent.
- A stored key cannot silently cross provider or network-origin boundaries.
- Existing same-scope profiles and local HTTP provider workflows continue to
  work.
- Automated tests, current-state docs, and agent workflow docs describe the
  implemented behavior.
