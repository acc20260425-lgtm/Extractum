# Telegram Account API Wrappers Design

## Summary

Centralize the remaining frontend Tauri command access for Telegram account
management and authentication in a typed `$lib/api/accounts.ts` module.

The work removes raw account/auth `invoke(...)` calls from
`src/routes/accounts/+page.svelte` and `src/routes/auth/[id]/+page.svelte`.
It also lets `src/lib/api/analysis-workspace.ts` reuse the same account API for
shared account-listing and runtime-status calls.

## Goals

- Add a typed frontend API wrapper for Telegram account lifecycle commands.
- Remove raw `@tauri-apps/api/core` imports from the Accounts and Auth routes.
- Avoid duplicated wrappers for `list_accounts` and `tg_get_account_statuses`
  between the Analysis workspace API and the account management surface.
- Keep route-local Svelte state, UI validation, modals, navigation, and
  lifecycle listeners in the routes.
- Add focused unit tests that pin every wrapper command name and payload shape.

## Non-Goals

- Do not change backend Rust commands, database schema, or Tauri wire behavior.
- Do not redesign the Accounts or Auth UI.
- Do not extract route state machines or listener lifecycle in this workstream.
- Do not introduce generated Rust-to-TypeScript types.
- Do not move credential validation out of the routes unless it is already
  required by the wrapper input shape.

## Current State

Existing compact API wrappers cover Analysis runs, chat, trace, workspace
loading, source groups/templates, sources, Takeout import, NotebookLM export,
and LLM commands.

The remaining frontend raw account/auth command usage is concentrated in:

- `src/routes/accounts/+page.svelte`
  - `list_accounts`
  - `tg_get_account_statuses`
  - `create_account`
  - `delete_account`
- `src/routes/auth/[id]/+page.svelte`
  - `get_account`
  - `tg_init`
  - `tg_send_code`
  - `tg_sign_in`
  - `set_account_phone`
  - `tg_logout`
  - `clear_account_phone`

`src/lib/api/analysis-workspace.ts` already wraps `list_accounts` and
`tg_get_account_statuses` for the Analysis page, so the new account API should
become the single owner of those command names.

## API Boundary

Create `src/lib/api/accounts.ts` as the frontend owner of account and Telegram
auth command names.

Export these input types:

```ts
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
```

Export these functions:

```ts
export function listAccounts(): Promise<AccountRecord[]>;
export function getAccount(accountId: number): Promise<AccountRecord | null>;
export function createAccount(input: CreateAccountInput): Promise<AccountRecord>;
export function deleteAccount(accountId: number): Promise<void>;
export function setAccountPhone(input: AccountPhoneInput): Promise<void>;
export function clearAccountPhone(accountId: number): Promise<void>;
export function getAccountRuntimeStatuses(accountIds: number[]): Promise<AccountRuntimeStatus[]>;
export function initializeTelegramAccount(accountId: number): Promise<boolean>;
export function sendTelegramCode(input: AccountPhoneInput): Promise<void>;
export function signInTelegramAccount(input: AccountCodeInput): Promise<void>;
export function logoutTelegramAccount(accountId: number): Promise<void>;
```

The wrappers should preserve the existing camelCase Tauri argument keys used by
the frontend:

- `{ accountId }`
- `{ accountIds }`
- `{ label, apiId, apiHash }`
- `{ accountId, phone }`
- `{ accountId, code }`

## Route Integration

`src/routes/accounts/+page.svelte` should import account API functions instead
of `invoke`. Its behavior stays the same:

- route-side parsing and validation for the new account form;
- `formatAppError(...)` wording;
- confirm modal before deletion;
- runtime status listener handling.

`src/routes/auth/[id]/+page.svelte` should import account API functions instead
of `invoke`. Its behavior stays the same:

- invalid account id redirect;
- initialization on mount;
- send-code, sign-in, and logout button behavior;
- phone persistence after sign-in;
- phone clearing after logout;
- route-local status and step state.

## Analysis Workspace API Reuse

`src/lib/api/analysis-workspace.ts` should keep its existing public function
names so `src/lib/analysis-workspace-workflow.ts` and route wiring do not need
to change.

The account-related functions should delegate to `src/lib/api/accounts.ts`:

```ts
export { listAccounts as listWorkspaceAccounts } from "$lib/api/accounts";
export {
  getAccountRuntimeStatuses as getWorkspaceAccountStatuses,
} from "$lib/api/accounts";
```

`listAnalysisSources()` remains in `analysis-workspace.ts` because it belongs
to the Analysis workspace command surface.

## Testing

Add `src/lib/api/accounts.test.ts` with command contract tests for every new
wrapper.

Update `src/lib/api/analysis-workspace.test.ts` so it verifies only
`listAnalysisSources()` directly invokes Tauri. The account-listing/status
command contract should live in `accounts.test.ts`.

After route wiring, verify that raw account/auth command access has moved:

```powershell
rg -n "\binvoke\s*(<|\()|@tauri-apps/api/core" src/routes/accounts/+page.svelte 'src/routes/auth/[id]/+page.svelte'
```

Expected: no output.

Focused verification:

```powershell
npm.cmd test -- src/lib/api/accounts.test.ts src/lib/api/analysis-workspace.test.ts
npm.cmd run check
```

Final verification:

```powershell
npm.cmd test
npm.cmd run check
git diff --check
```

## Follow-Up

After this workstream, the review follow-up about remaining compact non-source
Tauri wrappers should be re-evaluated by searching for route-local raw
`invoke(...)` calls. The next small surface should be selected from the
remaining search results, not guessed in advance.
