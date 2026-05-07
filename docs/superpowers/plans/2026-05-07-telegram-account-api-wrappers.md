# Telegram Account API Wrappers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Telegram account and authentication Tauri command access out of route files into a typed frontend API wrapper.

**Architecture:** Create `src/lib/api/accounts.ts` as the single frontend owner of account/auth command names and payload shapes. Keep Svelte state, validation, navigation, modals, and lifecycle listeners in routes, and keep `analysis-workspace.ts` as a stable Analysis-facing facade that reuses the new account API.

**Tech Stack:** Svelte 5, TypeScript, Vitest, Tauri `invoke`, PowerShell on Windows.

---

## File Structure

- Create `src/lib/api/accounts.ts` for account and Telegram auth API wrappers.
- Create `src/lib/api/accounts.test.ts` for command-name and payload contract tests.
- Modify `src/lib/api/analysis-workspace.ts` so account-related workspace exports delegate to `accounts.ts`.
- Modify `src/lib/api/analysis-workspace.test.ts` so it tests only the Analysis source wrapper command directly.
- Modify `src/routes/accounts/+page.svelte` to import account API wrappers instead of raw `invoke`.
- Modify `src/routes/auth/[id]/+page.svelte` to import account API wrappers instead of raw `invoke`.
- Modify `docs/code-review-results-2026-05-03.md` and `docs/session-context-2026-05-03.md` after implementation verification.

## Task 1: Add Telegram Account API Wrapper

**Files:**
- Create: `src/lib/api/accounts.ts`
- Test: `src/lib/api/accounts.test.ts`

- [ ] **Step 1: Add failing account API wrapper tests**

Create `src/lib/api/accounts.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  clearAccountPhone,
  createAccount,
  deleteAccount,
  getAccount,
  getAccountRuntimeStatuses,
  initializeTelegramAccount,
  listAccounts,
  logoutTelegramAccount,
  sendTelegramCode,
  setAccountPhone,
  signInTelegramAccount,
} from "./accounts";
import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("account api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads accounts with the registered command name", async () => {
    const accounts: AccountRecord[] = [{
      id: 1,
      label: "Main",
      api_id: 123,
      phone: "+100",
      created_at: 10,
    }];
    invokeMock.mockResolvedValueOnce(accounts);

    await expect(listAccounts()).resolves.toEqual(accounts);

    expect(invokeMock).toHaveBeenLastCalledWith("list_accounts");
  });

  it("loads an account with the expected payload", async () => {
    const account: AccountRecord = {
      id: 7,
      label: "Personal",
      api_id: 456,
      phone: null,
      created_at: 20,
    };
    invokeMock.mockResolvedValueOnce(account);

    await expect(getAccount(7)).resolves.toEqual(account);

    expect(invokeMock).toHaveBeenLastCalledWith("get_account", { accountId: 7 });
  });

  it("creates an account with the expected payload", async () => {
    const account: AccountRecord = {
      id: 8,
      label: "Work",
      api_id: 789,
      phone: null,
      created_at: 30,
    };
    invokeMock.mockResolvedValueOnce(account);

    await expect(createAccount({
      label: "Work",
      apiId: 789,
      apiHash: "hash",
    })).resolves.toEqual(account);

    expect(invokeMock).toHaveBeenLastCalledWith("create_account", {
      label: "Work",
      apiId: 789,
      apiHash: "hash",
    });
  });

  it("deletes an account with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAccount(8)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_account", { accountId: 8 });
  });

  it("sets and clears an account phone with the expected payloads", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await expect(setAccountPhone({ accountId: 8, phone: "+123" })).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("set_account_phone", {
      accountId: 8,
      phone: "+123",
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(clearAccountPhone(8)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("clear_account_phone", { accountId: 8 });
  });

  it("loads account runtime statuses for the given account ids", async () => {
    const statuses: AccountRuntimeStatus[] = [{
      account_id: 1,
      status: "ready",
      message: null,
    }];
    invokeMock.mockResolvedValueOnce(statuses);

    await expect(getAccountRuntimeStatuses([1, 2])).resolves.toEqual(statuses);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_get_account_statuses", {
      accountIds: [1, 2],
    });
  });

  it("initializes a Telegram account with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(true);

    await expect(initializeTelegramAccount(8)).resolves.toBe(true);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_init", { accountId: 8 });
  });

  it("sends a Telegram code and signs in with the expected payloads", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await expect(sendTelegramCode({ accountId: 8, phone: "+123" })).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("tg_send_code", {
      accountId: 8,
      phone: "+123",
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(signInTelegramAccount({ accountId: 8, code: "12345" })).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("tg_sign_in", {
      accountId: 8,
      code: "12345",
    });
  });

  it("logs out a Telegram account with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(logoutTelegramAccount(8)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("tg_logout", { accountId: 8 });
  });
});
```

- [ ] **Step 2: Run the focused RED test**

Run:

```powershell
npm.cmd test -- src/lib/api/accounts.test.ts
```

Expected: FAIL because `src/lib/api/accounts.ts` does not exist yet.

- [ ] **Step 3: Add the account API wrapper**

Create `src/lib/api/accounts.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";

export interface CreateAccountInput {
  label: string;
  apiId: number;
  apiHash: string;
}

export interface AccountIdInput {
  accountId: number;
}

export interface AccountPhoneInput extends AccountIdInput {
  phone: string;
}

export interface AccountCodeInput extends AccountIdInput {
  code: string;
}

export function listAccounts() {
  return invoke<AccountRecord[]>("list_accounts");
}

export function getAccount(accountId: number) {
  return invoke<AccountRecord | null>("get_account", { accountId });
}

export function createAccount(input: CreateAccountInput) {
  return invoke<AccountRecord>("create_account", { ...input });
}

export function deleteAccount(accountId: number) {
  return invoke<void>("delete_account", { accountId });
}

export function setAccountPhone(input: AccountPhoneInput) {
  return invoke<void>("set_account_phone", { ...input });
}

export function clearAccountPhone(accountId: number) {
  return invoke<void>("clear_account_phone", { accountId });
}

export function getAccountRuntimeStatuses(accountIds: number[]) {
  return invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", { accountIds });
}

export function initializeTelegramAccount(accountId: number) {
  return invoke<boolean>("tg_init", { accountId });
}

export function sendTelegramCode(input: AccountPhoneInput) {
  return invoke<void>("tg_send_code", { ...input });
}

export function signInTelegramAccount(input: AccountCodeInput) {
  return invoke<void>("tg_sign_in", { ...input });
}

export function logoutTelegramAccount(accountId: number) {
  return invoke<void>("tg_logout", { accountId });
}
```

- [ ] **Step 4: Run focused GREEN verification**

Run:

```powershell
npm.cmd test -- src/lib/api/accounts.test.ts
```

Expected: PASS for `src/lib/api/accounts.test.ts`.

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add src/lib/api/accounts.ts src/lib/api/accounts.test.ts
git commit -m "refactor(accounts): add api wrappers"
```

## Task 2: Reuse Account API From Analysis Workspace

**Files:**
- Modify: `src/lib/api/analysis-workspace.ts`
- Modify: `src/lib/api/analysis-workspace.test.ts`
- Test: `src/lib/api/accounts.test.ts`
- Test: `src/lib/api/analysis-workspace.test.ts`

- [ ] **Step 1: Update the analysis workspace API facade**

Replace the contents of `src/lib/api/analysis-workspace.ts` with:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AnalysisSourceOption } from "$lib/types/analysis";

export {
  getAccountRuntimeStatuses as getWorkspaceAccountStatuses,
  listAccounts as listWorkspaceAccounts,
} from "$lib/api/accounts";

export function listAnalysisSources() {
  return invoke<AnalysisSourceOption[]>("list_analysis_sources");
}
```

- [ ] **Step 2: Update the analysis workspace API test**

Replace the contents of `src/lib/api/analysis-workspace.test.ts` with:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import { listAnalysisSources } from "./analysis-workspace";
import type { AnalysisSourceOption } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("analysis workspace api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads analysis source metrics with the registered command name", async () => {
    const sources: AnalysisSourceOption[] = [{
      id: 7,
      account_id: 1,
      title: "Source",
      item_count: 12,
      last_synced_at: 100,
    }];
    invokeMock.mockResolvedValueOnce(sources);

    await expect(listAnalysisSources()).resolves.toEqual(sources);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_sources");
  });
});
```

- [ ] **Step 3: Run focused verification**

Run:

```powershell
npm.cmd test -- src/lib/api/accounts.test.ts src/lib/api/analysis-workspace.test.ts
```

Expected: PASS for both API test files.

- [ ] **Step 4: Verify analysis workspace consumers still compile**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 5: Commit Task 2**

Run:

```powershell
git add src/lib/api/analysis-workspace.ts src/lib/api/analysis-workspace.test.ts
git commit -m "refactor(analysis): reuse account api wrappers"
```

## Task 3: Wire Account Routes To API Wrappers

**Files:**
- Modify: `src/routes/accounts/+page.svelte`
- Modify: `src/routes/auth/[id]/+page.svelte`
- Test: `src/lib/api/accounts.test.ts`
- Test: `src/lib/api/analysis-workspace.test.ts`

- [ ] **Step 1: Update Accounts route imports**

In `src/routes/accounts/+page.svelte`, remove:

```ts
import { invoke } from "@tauri-apps/api/core";
```

Add this import after the existing app/navigation import:

```ts
import {
  createAccount as createAccountRecord,
  deleteAccount as deleteAccountRecord,
  getAccountRuntimeStatuses,
  listAccounts as listAccountRecords,
} from "$lib/api/accounts";
```

- [ ] **Step 2: Replace Accounts route raw command calls**

In `loadAccounts`, replace:

```ts
accounts = await invoke<AccountRecord[]>("list_accounts");
```

with:

```ts
accounts = await listAccountRecords();
```

In `loadAccountStatuses`, replace:

```ts
const statuses = await invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", {
  accountIds: accounts.map((account) => account.id),
});
```

with:

```ts
const statuses = await getAccountRuntimeStatuses(accounts.map((account) => account.id));
```

In `createAccount`, replace:

```ts
await invoke("create_account", {
  label: newLabel.trim(),
  apiId: parsedApiId,
  apiHash: newApiHash.trim(),
});
```

with:

```ts
await createAccountRecord({
  label: newLabel.trim(),
  apiId: parsedApiId,
  apiHash: newApiHash.trim(),
});
```

In `deleteAccount`, replace:

```ts
await invoke("delete_account", { accountId: account.id });
```

with:

```ts
await deleteAccountRecord(account.id);
```

- [ ] **Step 3: Update Auth route imports**

In `src/routes/auth/[id]/+page.svelte`, remove:

```ts
import { invoke } from "@tauri-apps/api/core";
```

Add this import after the existing app/navigation import:

```ts
import {
  clearAccountPhone,
  getAccount,
  initializeTelegramAccount,
  logoutTelegramAccount,
  sendTelegramCode,
  setAccountPhone,
  signInTelegramAccount,
} from "$lib/api/accounts";
```

- [ ] **Step 4: Replace Auth route raw command calls**

In `loadAccount`, replace:

```ts
const acc = await invoke<AccountRecord | null>("get_account", { accountId });
```

with:

```ts
const acc = await getAccount(accountId);
```

In `initClient`, replace:

```ts
const isAuth = await invoke<boolean>("tg_init", {
  accountId,
});
```

with:

```ts
const isAuth = await initializeTelegramAccount(accountId);
```

In `sendCode`, replace:

```ts
await invoke("tg_send_code", { accountId, phone });
```

with:

```ts
await sendTelegramCode({ accountId, phone });
```

In `signIn`, replace:

```ts
await invoke("tg_sign_in", { accountId, code });
await invoke("set_account_phone", { accountId, phone });
```

with:

```ts
await signInTelegramAccount({ accountId, code });
await setAccountPhone({ accountId, phone });
```

In `logout`, replace:

```ts
await invoke("tg_logout", { accountId });
await invoke("clear_account_phone", { accountId });
```

with:

```ts
await logoutTelegramAccount(accountId);
await clearAccountPhone(accountId);
```

- [ ] **Step 5: Verify raw account/auth route command access is gone**

Run:

```powershell
rg -n "\binvoke\s*(<|\()|@tauri-apps/api/core" src/routes/accounts/+page.svelte 'src/routes/auth/[id]/+page.svelte'
```

Expected: no output and exit code 1.

- [ ] **Step 6: Run focused frontend verification**

Run:

```powershell
npm.cmd test -- src/lib/api/accounts.test.ts src/lib/api/analysis-workspace.test.ts
npm.cmd run check
```

Expected:

- both API test files pass;
- Svelte check reports 0 errors and 0 warnings.

- [ ] **Step 7: Commit Task 3**

Run:

```powershell
git add src/routes/accounts/+page.svelte src/routes/auth/[id]/+page.svelte
git commit -m "refactor(accounts): use api wrappers in routes"
```

## Task 4: Refresh Review Docs and Session Handoff

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Re-evaluate remaining raw frontend Tauri route command usage**

Run:

```powershell
rg -n "\binvoke\s*(<|\()|@tauri-apps/api/core" src/routes src/lib/api
```

Expected:

- no raw account/auth route command access remains in `src/routes/accounts` or
  `src/routes/auth`;
- remaining `invoke` usage is either in `$lib/api/*` wrappers or unrelated
  route surfaces that become candidates for a future workstream.

- [ ] **Step 2: Update the review document**

In `docs/code-review-results-2026-05-03.md`, update resolved work to include:

```text
Telegram account and authentication command access is centralized in
`src/lib/api/accounts.ts`; the Accounts and Auth routes no longer invoke those
Tauri commands directly.
```

Update the moderate frontend/backend contract finding so the list of compact
frontend API wrappers includes Telegram accounts/auth.

Keep the recommended follow-up order unless the raw-command search from Step 1
shows there are no remaining compact non-source command surfaces to wrap.

- [ ] **Step 3: Refresh the session handoff**

In `docs/session-context-2026-05-03.md`, record:

- current workstream `Telegram account API wrappers`;
- source docs:
  - `docs/superpowers/specs/2026-05-07-telegram-account-api-wrappers-design.md`;
  - `docs/superpowers/plans/2026-05-07-telegram-account-api-wrappers.md`;
- completed commits from Tasks 1-3;
- verification commands and results;
- current branch and clean/dirty state;
- that the no-worktree, one-task-per-turn workflow remains active.

- [ ] **Step 4: Run final verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- `npm.cmd test` passes;
- `npm.cmd run check` reports 0 errors and 0 warnings;
- `git diff --check` exits 0.

- [ ] **Step 5: Commit Task 4**

Run:

```powershell
git add docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md
git commit -m "docs(session): refresh account api handoff"
```

## Execution Notes

- Use the existing workflow preference: no git worktree and inline execution.
- Execute exactly one top-level task per user turn.
- Commit at the end of each top-level task.
- If `npm.cmd test` or `npm.cmd run check` fails in the default sandbox with
  `spawn EPERM`, rerun the same command with approval outside the sandbox.
- If git writes fail in the default sandbox with `.git/index.lock` permission
  errors, rerun the same git command with approval outside the sandbox.
