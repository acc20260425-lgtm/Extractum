<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import {
    takeoutRecoveryBody,
    takeoutRecoveryFacts,
    takeoutRecoverySeverity,
    takeoutRecoveryTitle,
  } from "$lib/analysis-state";
  import type { TakeoutImportRecoveryState } from "$lib/types/sources";

  let {
    recovery,
    compact = false,
  }: {
    recovery: TakeoutImportRecoveryState;
    compact?: boolean;
  } = $props();

  const title = $derived(takeoutRecoveryTitle(recovery));
  const body = $derived(takeoutRecoveryBody());
  const severity = $derived(takeoutRecoverySeverity(recovery));
  const facts = $derived(takeoutRecoveryFacts(recovery));
  const showTerminalError = $derived(
    recovery.recovery_kind === "failed" && !!recovery.terminal_error,
  );
</script>

<section class="takeout-recovery-notice" class:compact aria-label={title}>
  <div class="takeout-recovery-heading">
    <Badge variant={severity}>{recovery.recovery_kind.replaceAll("_", " ")}</Badge>
    <strong>{title}</strong>
  </div>
  {#if !compact}
    <p>{body}</p>
  {/if}
  <div class="takeout-recovery-facts">
    {#each facts as fact (fact)}
      <span>{fact}</span>
    {/each}
  </div>
  {#if recovery.warning_codes.length > 0}
    <div class="takeout-recovery-codes">
      {#each recovery.warning_codes as code (code)}
        <Badge variant="neutral">{code}</Badge>
      {/each}
    </div>
  {/if}
  {#if showTerminalError}
    <p class="takeout-recovery-error">{recovery.terminal_error}</p>
  {/if}
</section>

<style>
  .takeout-recovery-notice {
    display: grid;
    gap: 0.45rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: color-mix(in srgb, var(--panel-hover) 60%, transparent);
    padding: 0.75rem;
    color: var(--text);
  }

  .takeout-recovery-notice.compact {
    padding: 0.6rem;
  }

  .takeout-recovery-heading,
  .takeout-recovery-facts,
  .takeout-recovery-codes {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem;
    min-width: 0;
  }

  .takeout-recovery-heading strong {
    min-width: 0;
    font-size: 0.86rem;
    line-height: 1.25;
  }

  .takeout-recovery-notice p {
    margin: 0;
    color: var(--muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .takeout-recovery-facts span {
    color: var(--muted);
    font-size: 0.78rem;
    line-height: 1.35;
  }

  .takeout-recovery-error {
    color: var(--danger);
  }
</style>
