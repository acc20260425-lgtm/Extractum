<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { formatAppError } from "$lib/app-error";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
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

<section class="page-shell">
  <header class="page-hero">
    <div class="page-hero-copy">
      <span class="page-eyebrow">Telegram authentication</span>
      <h1>{label || "Account"}</h1>
      <p>
        Connect this Telegram identity to the local workspace: initialize, confirm phone, verify
        code, then return to analysis.
      </p>
    </div>
    <div class="page-hero-meta">
      <Badge variant="info">{label || "Account"}</Badge>
      <Badge variant={step === "done" ? "success" : step === "code" ? "warning" : "neutral"}>
        {step === "connecting"
          ? "Connecting"
          : step === "phone"
            ? "Phone step"
            : step === "code"
              ? "Verification"
              : "Authenticated"}
      </Badge>
    </div>
  </header>

  {#if status}
    <StatusMessage tone={status.startsWith("Error") ? "error" : "default"} className="page-status">
      {status}
    </StatusMessage>
  {/if}

  <div class="page-grid auth-grid">
    <div class="page-stack">
      <section class="desk-panel auth-panel">
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">Authentication flow</span>
            <h2>
              {#if step === "connecting"}
                Connecting to Telegram
              {:else if step === "phone"}
                Sign in with phone
              {:else if step === "code"}
                Enter verification code
              {:else}
                Account ready
              {/if}
            </h2>
            <p>
              {#if step === "connecting"}
                Restoring or initializing the Telegram client for this account.
              {:else if step === "phone"}
                Send a login code to the phone linked with this Telegram identity.
              {:else if step === "code"}
                Enter the Telegram code to finish sign-in.
              {:else}
                This account is ready to sync sources.
              {/if}
            </p>
          </div>
        </div>

        {#if step === "connecting"}
          <EmptyState description="Connecting to Telegram..." />
        {:else if step === "phone"}
          <div class="form-stack">
            <label>Phone number
              <Input
                type="tel"
                value={phone}
                placeholder="+79991234567"
                oninput={(event) => (phone = (event.currentTarget as HTMLInputElement).value)}
              />
            </label>
          </div>
          <div class="action-row">
            <Button onclick={sendCode} disabled={loading || !phone}>
              {loading ? "Sending..." : "Send code"}
            </Button>
          </div>
        {:else if step === "code"}
          <div class="form-stack">
            <label>Code
              <Input
                type="text"
                value={code}
                placeholder="12345"
                oninput={(event) => (code = (event.currentTarget as HTMLInputElement).value)}
              />
            </label>
          </div>
          <div class="action-row">
            <Button onclick={signIn} disabled={loading || !code}>
              {loading ? "Signing in..." : "Sign in"}
            </Button>
            <Button variant="secondary" onclick={() => (step = "phone")}>Back</Button>
          </div>
        {:else}
          <SurfaceCard className="auth-success">
            <div class="summary-stack">
              <div class="summary-row">
                <span class="hint">Phone: {phone}</span>
                <Badge variant="success">Ready</Badge>
              </div>
              <StatusMessage tone="default" size="sm" surface={false}>
                This account is authenticated and ready to load sources.
              </StatusMessage>
            </div>
          </SurfaceCard>
          <div class="action-row">
            <Button onclick={() => goto("/analysis")}>Open workspace</Button>
            <Button variant="danger-soft" onclick={logout} disabled={loading}>Logout</Button>
          </div>
        {/if}
      </section>
    </div>

    <div class="page-stack">
      <section class="desk-panel desk-panel-subtle">
        <div class="panel-header-copy">
          <span class="page-eyebrow">Account context</span>
          <h3>Workspace identity</h3>
        </div>

        <SurfaceCard className="auth-summary">
          <div class="summary-row">
            <span class="hint">Account label</span>
            <Badge variant="neutral">{label || "Account"}</Badge>
          </div>
          <div class="summary-row">
            <span class="hint">API ID</span>
            <strong class="summary-value">{apiId || "Unknown"}</strong>
          </div>
          <div class="summary-row">
            <span class="hint">Phone</span>
            <strong class="summary-value">{phone || "Not set yet"}</strong>
          </div>
        </SurfaceCard>
      </section>

      <section class="desk-panel desk-panel-subtle">
        <div class="panel-header-copy">
          <span class="page-eyebrow">What happens next</span>
          <h3>After sign-in</h3>
          <p>
            Return to `Analysis` to sync sources, inspect recent messages, and launch a report.
          </p>
        </div>
      </section>
    </div>
  </div>
</section>

<style>
  .back-row {
    margin-bottom: 0.9rem;
  }

  :global(.ui-surface-card.auth-summary) {
    padding: 0.85rem 1rem;
  }

  :global(.ui-surface-card.auth-success) {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 72%, transparent), transparent);
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

  .summary-value {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
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
    margin-bottom: 0;
  }

  .auth-grid {
    align-items: start;
  }

  .auth-panel {
    gap: 0.95rem;
  }
</style>
