# Telegram Runtime Live Validation Plan

> **For agentic workers:** This is a manual validation checklist, not an implementation plan. Do not change code while executing it unless a concrete failure is reproduced and separately triaged.

**Goal:** Validate encrypted Telegram session storage and the remaining Telegram runtime paths against a real local app profile.

**Architecture:** Use the existing desktop app and real app-data profile. Observe session-file migration, runtime account restore state, logout/delete cleanup, and private-source behavior. Record exact outcomes before deciding whether any code follow-up is needed.

**Tech Stack:** Tauri 2 desktop app, SQLite app database, OS secure storage, Telegram runtime, PowerShell.

---

## Preconditions

- Work from repository root: `G:\Develop\Extractum`.
- Current branch should be `main`.
- Working tree should be clean.
- A real Telegram account is configured in the app.
- Existing app-data may include `telegram_<account_id>.session.json`.

Commands:

```powershell
git branch --show-current
git status --short
Get-ChildItem "$env:APPDATA\org.ai.extractum" -Filter "telegram_*.session.json"
```

Expected:

- branch is `main`;
- `git status --short` is empty;
- at least one Telegram session file may exist if an account was previously authenticated.

---

## Task 1: Validate Session File Migration

- [ ] **Step 1: Capture session file before app startup**

```powershell
$session = "$env:APPDATA\org.ai.extractum\telegram_1.session.json"
Get-Content -Raw $session
```

Expected before migration if this is an old session:

- JSON may contain plaintext fields such as `home_dc`, `dc_options`, or `updates_state`.

- [ ] **Step 2: Start the app**

```powershell
cd G:\Develop\Extractum\src-tauri
cargo run
```

Expected:

- app starts without migration checksum panic;
- account restore begins automatically;
- no terminal panic from SQL plugin or Telegram session decrypt/load.

- [ ] **Step 3: Inspect session file after startup**

```powershell
$session = "$env:APPDATA\org.ai.extractum\telegram_1.session.json"
Get-Content -Raw $session
```

Expected after migration:

- JSON contains `version`, `algorithm`, `nonce`, and `ciphertext`;
- JSON does not contain `home_dc`, `dc_options`, or `updates_state`;
- `algorithm` is `XChaCha20-Poly1305`.

---

## Task 2: Validate Runtime Restore

- [ ] **Step 1: Observe account status in the UI**

Expected:

- previously authenticated account reaches `ready`;
- if secure storage key is missing, only that account becomes `restore_failed` with a clear storage/auth message.

- [ ] **Step 2: Exercise an authenticated action**

Use an existing Telegram source or add/list a source that requires the restored account.

Expected:

- action uses the restored runtime session;
- no sign-in prompt appears for a valid migrated session.

---

## Task 3: Validate Re-Login Save Path

- [ ] **Step 1: If an account is not authenticated, complete sign-in**

Use the existing UI flow:

- initialize account;
- send code;
- sign in.

Expected:

- sign-in succeeds;
- `telegram_<account_id>.session.json` is written as encrypted envelope;
- file does not contain plaintext session fields.

---

## Task 4: Validate Logout Cleanup

- [ ] **Step 1: Log out an account from the UI**

Expected:

- runtime status returns to `not_initialized`;
- `telegram_<account_id>.session.json` is removed;
- signing in again is required.

- [ ] **Step 2: Confirm no session file remains**

```powershell
Get-ChildItem "$env:APPDATA\org.ai.extractum" -Filter "telegram_*.session.json"
```

Expected:

- logged-out account no longer has a session file.

---

## Task 5: Validate Account Delete Cleanup

- [ ] **Step 1: Delete a test account from the UI**

Expected:

- account row disappears from the UI;
- associated session file is removed;
- deleting a missing session file is treated as no-op;
- API hash cleanup errors, if any, surface after the row/runtime cleanup.

---

## Task 6: Validate Private Source Runtime Cases

- [ ] **Step 1: Add or refresh a dialog-picked private channel**

Expected:

- stored peer identity path is used where available;
- source resolves without public username dependency.

- [ ] **Step 2: Add or refresh a dialog-picked private supergroup**

Expected:

- stored peer identity path is used where available;
- source resolves without public username dependency.

- [ ] **Step 3: Record any fallback to dialog scanning**

Capture:

- source kind;
- account id;
- whether source was private/left/public;
- visible UI/backend error text;
- terminal logs if present.

---

## Completion Notes

Recorded on 2026-05-08:

- `telegram_1.session.json` was observed as an encrypted envelope:
  - `version`: `1`
  - `algorithm`: `XChaCha20-Poly1305`
  - contains `nonce` and `ciphertext`
  - does not contain plaintext `home_dc`, `dc_options`, or `updates_state`
- Account restore reached UI status: `Account ready`.
- UI showed: `This account is ready to sync sources.`
- Private supergroup source `WBChat` was visible as:
  - category: `Life`
  - kind: `supergroup`
  - message count: `73102 msgs`
  - membership: `member`
- Sync on private supergroup `WBChat` succeeded without re-login:
  - timestamp changed from `08.05.2026, 22:16:20` to `08.05.2026, 22:17:18`
- Logout and re-login were exercised for account `1`.
- After re-login, `telegram_1.session.json` was observed as a new encrypted envelope:
  - `version`: `1`
  - `algorithm`: `XChaCha20-Poly1305`
  - contains `nonce` and `ciphertext`
  - does not contain plaintext `home_dc`, `dc_options`, or `updates_state`
- No `restore_failed` state or auth error was observed during this validation slice.

Not yet validated in this slice:

- account delete cleanup removes session file, session key, and API hash secret;
- a separate private `channel` source case.

If failures are found, create a separate debugging task before changing code.
