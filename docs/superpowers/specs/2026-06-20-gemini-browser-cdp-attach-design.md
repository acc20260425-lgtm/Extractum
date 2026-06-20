# Gemini Browser User-Controlled Chrome CDP Attach Design

## Context

The current Gemini Browser Provider launches a Playwright-controlled browser profile. Google sign-in can reject that browser with "this browser or app may not be secure" and may require phone verification. Extractum must not automate Google login, phone verification, CAPTCHA, account picker, consent, or other account security flows.

This slice adds a user-controlled Chrome mode. The user starts ordinary Chrome with a remote debugging endpoint, completes Google/Gemini login manually, and Extractum attaches to that already-open browser through Chrome DevTools Protocol.

## Goals

- Let the Gemini Browser Provider use a normal user-controlled Chrome window instead of launching Playwright Chromium.
- Keep all Google authentication actions manual and visible to the user.
- Allow the existing prompt/send/read flow to reuse the same DOM adapter after a CDP page is attached.
- Provide typed provider statuses when Chrome CDP is unavailable or Gemini is not open.
- Keep the current managed Playwright browser mode as the default.

## Non-Goals

- No automation of Google sign-in, 2FA, phone verification, CAPTCHA, consent, or account picker.
- No storage, copying, import, export, or inspection of Google cookies.
- No automatic Chrome installation or browser binary discovery in this slice.
- No bundled Chromium delivery changes.
- No remote CDP endpoint support in v1. The CDP endpoint must be an allowlisted loopback URL: `http://127.0.0.1:<port>`, `http://localhost:<port>`, or `http://[::1]:<port>`.

## User Workflow

For CDP mode, the user starts Chrome manually:

```powershell
$profile = "$env:APPDATA\org.ai.extractum\gemini-browser\chrome-cdp-profile"
Start-Process chrome.exe -ArgumentList @(
  "--remote-debugging-port=9222",
  "--user-data-dir=$profile",
  "https://gemini.google.com/app"
)
```

The `--user-data-dir` must point to a dedicated Extractum/Gemini directory. Do not point CDP mode at the user's ordinary Chrome `Default` profile or any profile that contains unrelated personal browsing sessions. A CDP endpoint gives protocol clients broad control over the attached browser, so the profile must be intentionally scoped.

Then the user logs in and resolves any Google/Gemini account prompts inside that Chrome window. Extractum is configured with:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = "http://127.0.0.1:9222"
```

When the provider runs in this mode:

- `Resume` attaches to the existing Chrome endpoint and uses an existing Gemini tab if one is present.
- `Open` attaches to the existing Chrome endpoint and may open a Gemini tab if one is not present.
- If a Gemini tab is already open, the provider uses it.
- If Chrome is reachable but no Gemini tab exists, only `Open` may create a new tab and navigate it to `https://gemini.google.com/app`.
- `Stop` detaches from CDP but does not close the user-controlled Chrome process.

## Architecture

The sidecar receives a new browser mode decision:

- `managed`: existing `chromium.launchPersistentContext(browserProfileDir, ...)`.
- `cdp_attach`: `chromium.connectOverCDP(endpoint)`.

The initial v1 mode selection is environment-driven and should be treated as an operator/debug mode rather than polished packaged-app UX:

```text
EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT=http://127.0.0.1:9222
```

If the variable is set, CDP attach mode is used. If unset, the current managed mode remains unchanged.

A later product slice should add a settings UI for this mode. This v1 slice keeps the control surface narrow so the security boundary and operator workflow can be validated first.

The sidecar owns this decision because it already owns Playwright and browser lifecycle behavior. Rust/Tauri continues to call the same JSON-line commands: `status`, `open_browser`, `send_single`, `resume`, and `stop`.

## Ownership Invariants

- Managed mode owns the Playwright persistent context it launches. It may close that context and terminate the managed browser process on `Stop`.
- CDP attach mode owns only the Playwright CDP connection plus the selected Gemini page reference. It does not own Chrome, the user's browser context, or unrelated tabs.
- CDP `Stop` must detach from the CDP connection and clear Extractum's references, but must not call APIs that close the user's browser context or user tabs.
- Shared adapter code must branch lifecycle cleanup by mode. A generic `context.close()` cleanup is correct for managed mode and unsafe for CDP attach mode.

## Sidecar Behavior

`openBrowser(browserProfileDir)` becomes mode-aware:

- In managed mode, keep the existing persistent context launch.
- In CDP attach mode:
  - parse and validate `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT` against the loopback allowlist before calling Playwright;
  - connect to the endpoint with `chromium.connectOverCDP(endpoint)`;
  - keep the returned `Browser` connection separately from managed-mode `BrowserContext`;
  - select `browser.contexts()[0]` when available;
  - if `browser.contexts()` is empty, return `needs_manual_action` rather than creating a new context;
  - select an existing Gemini page with the deterministic rule below;
  - for `Open` only, create or reuse a page and navigate it to `https://gemini.google.com/app` when no Gemini page exists;
  - store the attached browser/context/page references for later `sendSingle`.

`Resume` in CDP mode must not create a new Gemini tab. If no Gemini page exists, it returns `needs_manual_action` with a message asking the user to open Gemini in the attached Chrome profile or use `Open`.

Gemini page selection:

- ignore closed pages and pages whose URL cannot be read;
- match only URLs whose hostname is exactly `gemini.google.com`;
- do not select `accounts.google.com`, Google consent pages, or other Google hosts as Gemini pages;
- prefer the active/frontmost page if Playwright exposes that signal in the attached context;
- otherwise pick the first matching page in the existing Playwright page order so repeated selection is deterministic.

`status(browserProfileDir)` reports:

- `ready` with message `Chrome CDP attached.` when attached and a page exists.
- `not_started` with message `Chrome CDP endpoint is configured but not attached.` before first attach.
- `needs_manual_action` with manual action `start_chrome_cdp` when Chrome CDP is not reachable or needs operator setup.

`status()` in CDP mode performs endpoint validation and a light reachability/protocol check when `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT` is set. It must not create pages, navigate, or attach long-lived automation state just to answer status. This gives the UI actionable setup messages before the user presses `Open` or `Resume` while preserving the user-controlled browser boundary.

`stop()` in CDP attach mode detaches from the Playwright connection and clears local references, but it does not terminate Chrome.

## Provider UI

The existing Browser Providers panel remains the v1 surface. It should show mode-aware status text from the backend/sidecar. A later UI polish can add a compact CDP setup hint, but this slice does not need a full wizard.

Implementation must extend shared Rust/frontend manual-action types with `start_chrome_cdp` and add a status label/message mapping. The UI should show this as an operator setup action, not as an unknown modal/account flow.

The key operator-facing distinction:

- Managed mode: `Open` launches browser.
- CDP mode: `Resume` attaches to an existing Gemini tab in already-open Chrome; `Open` may attach and create a Gemini tab if needed.

## Error Handling

Typed outcomes should stay visible and actionable:

- CDP endpoint fails allowlist validation: `needs_manual_action`, manual action `start_chrome_cdp`, message `Chrome CDP endpoint must be a loopback HTTP URL.`
- CDP endpoint unreachable: `needs_manual_action`, manual action `start_chrome_cdp`, message `Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.`
- CDP endpoint reachable but not a Chrome debugging endpoint: `needs_manual_action`, manual action `start_chrome_cdp`, message `Chrome CDP endpoint did not expose a Chrome debugging protocol.`
- CDP connection succeeds but no usable browser context is available: `needs_manual_action`, manual action `start_chrome_cdp`, message `Chrome CDP connected but no user profile context was available. Restart Chrome with a dedicated user data directory.`
- CDP protocol permission/version mismatch: `needs_manual_action`, manual action `start_chrome_cdp`, message `Chrome CDP protocol was incompatible with the sidecar.`
- CDP connected but Gemini requires login/consent/account action: existing `needs_login` or `needs_manual_action` result from the DOM adapter.
- No composer after wait: keep current `needs_login` result until the DOM contract is refined further.
- CDP browser/page disconnected mid-run: `browser_crashed` with sanitized artifacts and message `Chrome CDP connection closed during the run.`

Known limitation for this slice: "no composer after wait" is still a coarse fallback. In CDP mode the user may already be logged in while Gemini shows consent, age gate, account picker, disabled Workspace state, region block, model unavailable UI, or prompt-blocked UI. This slice keeps the existing fallback but records it as a DOM-contract follow-up rather than treating every no-composer state as proven logout.

## Security Boundary

Extractum attaches only to a user-provided loopback endpoint. The default documented endpoint is `http://127.0.0.1:9222`. Endpoint validation rejects `0.0.0.0`, LAN IPs, non-loopback hostnames, non-HTTP schemes, credentials in URLs, and any host outside `127.0.0.1`, `localhost`, or `[::1]`.

Extractum must not:

- read or export cookies;
- automate account recovery or phone verification;
- run CDP against arbitrary remote hosts;
- close the user's Chrome process on `Stop`.

Artifacts remain redacted/reduced by default for settings test runs.

## Testing

Unit and integration checks for this slice:

- Sidecar mode resolver chooses CDP attach when `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT` is present.
- CDP endpoint validation accepts `http://127.0.0.1:9222`, `http://localhost:9222`, and `http://[::1]:9222`.
- CDP endpoint validation rejects `http://192.168.1.20:9222`, `http://0.0.0.0:9222`, missing scheme values such as `127.0.0.1:9222`, non-HTTP schemes, credentials in URLs, and arbitrary hostnames.
- CDP page selection ignores non-Gemini Google pages, ignores unreadable/closed pages, and deterministically prefers the active/frontmost Gemini page when available or the first matching Gemini page otherwise.
- CDP `Open` creates a Gemini tab when Chrome is connected but no Gemini page exists.
- CDP `Resume` reports manual action when Chrome is connected but no Gemini page exists.
- CDP `status` checks validate and lightly probe the endpoint without opening pages or creating long-lived browser automation state.
- CDP attach failures return typed provider statuses for invalid endpoint, unreachable endpoint, non-Chrome endpoint, empty user profile context, protocol mismatch, and mid-run disconnect.
- `stop()` in CDP mode detaches without attempting to kill Chrome.
- Shared Rust/frontend manual-action types include `start_chrome_cdp`, and UI status label tests cover it.
- Local/mock CDP tests cover endpoint validation and page-selection logic without requiring Google login state. These can use small fake Browser/Context/Page adapters or a local Playwright-controlled Chromium CDP fixture that serves a non-Google page.
- Existing managed browser tests and smokes continue to pass.

Manual validation:

1. Start Chrome with `--remote-debugging-port=9222` and a dedicated profile.
2. Login to Gemini manually in that Chrome.
3. Start Extractum with `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT=http://127.0.0.1:9222`.
4. Use `Settings -> Browser Providers -> Resume`.
5. Send the one-sentence test prompt and verify a new run returns `ok`.

## Acceptance Criteria

- CDP mode works without launching a Playwright-owned browser.
- CDP endpoints outside the loopback allowlist are rejected before Playwright connects.
- `Resume` attaches only to an existing user-controlled Chrome/Gemini tab.
- `Open` may create only a Gemini tab and must not perform account/security actions.
- If Chrome is not running with CDP, the provider tells the user what to start.
- Empty CDP browser contexts are reported as operator setup failures; v1 does not create a new context because it may not share the user's profile/login state.
- Existing managed mode still works when the CDP env var is absent.
- No Google login automation is introduced.
