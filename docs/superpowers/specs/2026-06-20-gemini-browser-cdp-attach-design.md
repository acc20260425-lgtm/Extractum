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
- No remote CDP endpoint support beyond localhost by default.

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

Then the user logs in and resolves any Google/Gemini account prompts inside that Chrome window. Extractum is configured with:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = "http://127.0.0.1:9222"
```

When the provider runs in this mode:

- `Open` and `Resume` attach to the existing Chrome endpoint.
- If a Gemini tab is already open, the provider uses it.
- If Chrome is reachable but no Gemini tab exists, the provider opens `https://gemini.google.com/app` in the attached browser context.
- `Stop` detaches from CDP but does not close the user-controlled Chrome process.

## Architecture

The sidecar receives a new browser mode decision:

- `managed`: existing `chromium.launchPersistentContext(browserProfileDir, ...)`.
- `cdp_attach`: `chromium.connectOverCDP(endpoint)`.

The initial v1 mode selection is environment-driven:

```text
EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT=http://127.0.0.1:9222
```

If the variable is set, CDP attach mode is used. If unset, the current managed mode remains unchanged.

The sidecar owns this decision because it already owns Playwright and browser lifecycle behavior. Rust/Tauri continues to call the same JSON-line commands: `status`, `open_browser`, `send_single`, `resume`, and `stop`.

## Sidecar Behavior

`openBrowser(browserProfileDir)` becomes mode-aware:

- In managed mode, keep the existing persistent context launch.
- In CDP attach mode:
  - connect to `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT`;
  - select an existing page whose URL starts with `https://gemini.google.com/`;
  - otherwise create or reuse a page and navigate it to `https://gemini.google.com/app`;
  - store the attached browser/context/page references for later `sendSingle`.

`status(browserProfileDir)` reports:

- `ready` with message `Chrome CDP attached.` when attached and a page exists.
- `not_started` with message `Chrome CDP endpoint is configured but not attached.` before first attach.
- `needs_manual_action` with manual action `unknown_modal` when CDP endpoint is unreachable or not a Chrome debugging endpoint.

`stop()` in CDP attach mode detaches from the Playwright connection and clears local references, but it does not terminate Chrome.

## Provider UI

The existing Browser Providers panel remains the v1 surface. It should show mode-aware status text from the backend/sidecar. A later UI polish can add a compact CDP setup hint, but this slice does not need a full wizard.

The key operator-facing distinction:

- Managed mode: `Open` launches browser.
- CDP mode: `Open`/`Resume` attaches to already-open Chrome.

## Error Handling

Typed outcomes should stay visible and actionable:

- CDP endpoint unreachable: `needs_manual_action`, message `Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.`
- CDP connected but Gemini requires login/consent/account action: existing `needs_login` or `needs_manual_action` result from the DOM adapter.
- No composer after wait: keep current `needs_login` result until the DOM contract is refined further.
- CDP browser/page closed mid-run: `browser_crashed` or `failed` with sanitized artifacts.

## Security Boundary

Extractum attaches only to a user-provided local endpoint. The default documented endpoint is `http://127.0.0.1:9222`.

Extractum must not:

- read or export cookies;
- automate account recovery or phone verification;
- run CDP against arbitrary remote hosts by default;
- close the user's Chrome process on `Stop`.

Artifacts remain redacted/reduced by default for settings test runs.

## Testing

Unit and integration checks for this slice:

- Sidecar mode resolver chooses CDP attach when `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT` is present.
- CDP page selection prefers an existing `gemini.google.com` page.
- CDP page selection opens Gemini when Chrome is connected but no Gemini page exists.
- CDP attach failure returns a typed provider status instead of throwing.
- `stop()` in CDP mode detaches without attempting to kill Chrome.
- Existing managed browser tests and smokes continue to pass.

Manual validation:

1. Start Chrome with `--remote-debugging-port=9222` and a dedicated profile.
2. Login to Gemini manually in that Chrome.
3. Start Extractum with `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT=http://127.0.0.1:9222`.
4. Use `Settings -> Browser Providers -> Resume`.
5. Send the one-sentence test prompt and verify a new run returns `ok`.

## Acceptance Criteria

- CDP mode works without launching a Playwright-owned browser.
- `Resume` opens or attaches to a visible user-controlled Chrome/Gemini tab.
- If Chrome is not running with CDP, the provider tells the user what to start.
- Existing managed mode still works when the CDP env var is absent.
- No Google login automation is introduced.
