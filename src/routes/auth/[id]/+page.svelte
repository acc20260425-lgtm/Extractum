<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { formatAppError } from "$lib/app-error";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import type { AccountRecord } from "$lib/types/accounts";

  const accountId = parseInt(page.params.id ?? "", 10);
  const hasValidAccountId = Number.isFinite(accountId);

  let label = $state("");
  let apiId = $state(0);
  let apiHash = $state("");
  let phone = $state("");
  let code = $state("");
  let status = $state("");
  let step = $state<"connecting" | "phone" | "code" | "done">("connecting");
  let loading = $state(false);

  async function loadAccount() {
    if (!hasValidAccountId) {
      status = "Invalid account ID. Redirecting to accounts...";
      await goto("/accounts");
      return;
    }

    try {
      const acc = await invoke<AccountRecord | null>("get_account", { accountId });
      if (!acc) {
        status = "Account not found";
        return;
      }
      label = acc.label;
      apiId = acc.api_id;
      apiHash = acc.api_hash;
      phone = acc.phone ?? "";

      await initClient();
    } catch (e) {
      status = formatAppError("loading the account", e);
    }
  }

  async function initClient() {
    loading = true;
    status = "Connecting...";
    try {
      const isAuth = await invoke<boolean>("tg_init", {
        accountId,
        apiId,
        apiHash,
      });
      if (isAuth) {
        step = "done";
        status = "Already authenticated.";
      } else {
        step = "phone";
        status = "";
      }
    } catch (e) {
      status = formatAppError("initializing Telegram", e);
      step = "phone";
    } finally {
      loading = false;
    }
  }

  async function sendCode() {
    loading = true;
    status = "";
    try {
      await invoke("tg_send_code", { accountId, phone });
      step = "code";
    } catch (e) {
      status = formatAppError("sending the Telegram code", e);
    } finally {
      loading = false;
    }
  }

  async function signIn() {
    loading = true;
    status = "";
    try {
      await invoke("tg_sign_in", { accountId, code });
      await invoke("set_account_phone", { accountId, phone });
      step = "done";
      status = "Signed in successfully.";
    } catch (e) {
      status = formatAppError("signing in", e);
    } finally {
      loading = false;
    }
  }

  async function logout() {
    loading = true;
    try {
      await invoke("tg_logout", { accountId });
      await invoke("clear_account_phone", { accountId });
      phone = "";
      step = "phone";
      status = "";
    } catch (e) {
      status = formatAppError("logging out", e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadAccount();
  });
</script>

<div class="back-row">
  <Button variant="ghost" size="sm" onclick={() => goto("/accounts")}>&larr; Accounts</Button>
</div>

<h1>{label || "Account"}</h1>

{#if status}
  <StatusMessage tone={status.startsWith("Error") ? "error" : "default"} className="page-status">
    {status}
  </StatusMessage>
{/if}

{#if step === "connecting"}
  <Card className="page-card">
    <EmptyState description="Connecting to Telegram..." />
  </Card>
{/if}

{#if step === "phone"}
  <Card className="page-card">
    <h3>Sign In</h3>
    <SurfaceCard className="auth-summary">
      <div class="summary-row">
        <span class="hint">API ID: {apiId}</span>
        <Badge variant="neutral">{label || "Account"}</Badge>
      </div>
    </SurfaceCard>
    <label>Phone number
      <Input
        type="tel"
        value={phone}
        placeholder="+79991234567"
        oninput={(event) => (phone = (event.currentTarget as HTMLInputElement).value)}
      />
    </label>
    <div class="action-row">
      <Button onclick={sendCode} disabled={loading || !phone}>
        {loading ? "Sending..." : "Send Code"}
      </Button>
    </div>
  </Card>
{/if}

{#if step === "code"}
  <Card className="page-card">
    <h3>Verification Code</h3>
    <SurfaceCard className="auth-summary">
      <div class="summary-row">
        <span class="hint">Check your Telegram app for the code.</span>
        <Badge variant="warning">Verification pending</Badge>
      </div>
    </SurfaceCard>
    <label>Code
      <Input
        type="text"
        value={code}
        placeholder="12345"
        oninput={(event) => (code = (event.currentTarget as HTMLInputElement).value)}
      />
    </label>
    <div class="action-row">
      <Button onclick={signIn} disabled={loading || !code}>
        {loading ? "Signing in..." : "Sign In"}
      </Button>
      <Button variant="secondary" onclick={() => (step = "phone")}>Back</Button>
    </div>
  </Card>
{/if}

{#if step === "done"}
  <Card className="page-card">
    <h3>Authenticated</h3>
    <SurfaceCard className="auth-summary">
      <div class="summary-stack">
        <div class="summary-row">
          <span class="hint">Phone: {phone}</span>
          <Badge variant="success">Ready</Badge>
        </div>
        <StatusMessage tone="default" size="sm" surface={false}>
          This account is authenticated and ready to load Telegram sources.
        </StatusMessage>
      </div>
    </SurfaceCard>
    <div class="action-row">
      <Button onclick={() => goto(`/sources?account=${accountId}`)}>View Sources</Button>
      <Button variant="danger-soft" onclick={logout} disabled={loading}>Logout</Button>
    </div>
  </Card>
{/if}

<style>
  .back-row {
    margin-bottom: 1rem;
  }

  :global(.ui-card.page-card) {
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  :global(.ui-surface-card.auth-summary) {
    padding: 0.85rem 1rem;
  }

  .summary-row,
  .summary-stack {
    display: flex;
    gap: 0.6rem;
  }

  .summary-row {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .summary-stack {
    flex-direction: column;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .hint {
    font-size: 0.85rem;
    color: var(--muted);
    margin: 0;
  }

  .action-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  :global(.page-status) {
    margin-bottom: 1rem;
  }
</style>
