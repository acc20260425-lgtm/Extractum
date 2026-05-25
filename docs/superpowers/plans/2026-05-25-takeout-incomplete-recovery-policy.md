# Takeout Incomplete Recovery Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the safe Takeout incomplete-recovery policy copy and warning-code explanations without adding destructive actions, persisted dismissals, or true resume behavior.

**Architecture:** Keep Rust recovery state read-only and unchanged. Add deterministic frontend policy helpers in `src/lib/analysis-state.ts`, render them in `TakeoutRecoveryNotice`, and verify the behavior with focused Vitest coverage before updating the backlog.

**Tech Stack:** Svelte 5, TypeScript, Vitest, existing Tauri recovery DTOs.

---

## File Structure

- Modify `src/lib/analysis-state.ts`: derive recovery-kind-specific body copy and static warning-code explanations from the existing `TakeoutImportRecoveryState` DTO.
- Modify `src/lib/analysis-state.test.ts`: prove recovery policy mappings, warning-code explanations, fact formatting, and active-job hiding behavior.
- Modify `src/lib/components/analysis/takeout-recovery-notice.svelte`: render the policy body and full-mode warning explanations without creating a new recovery action surface.
- Modify `docs/backlog.md`: mark the safe recovery-policy slice complete and keep remaining Takeout follow-ups visible.

No Rust files, database migrations, Tauri commands, import semantics, or deduplication logic are part of this plan.

### Task 1: Recovery-Kind Policy Copy

**Files:**
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/components/analysis/takeout-recovery-notice.svelte`

- [x] **Step 1: Write the failing policy-copy test**

In `src/lib/analysis-state.test.ts`, replace the existing test named
`formats takeout recovery title, body, facts, and severity` with this test:

```ts
  it("formats distinct takeout recovery titles, bodies, and severity", () => {
    const cases: Array<[
      TakeoutImportRecoveryState["recovery_kind"],
      string,
      string,
      "danger" | "warning" | "neutral",
    ]> = [
      [
        "interrupted",
        "Previous Takeout import was interrupted",
        "The previous Takeout import stopped before Extractum could finish tracking it. Run Takeout again to start a fresh import; messages already saved locally will be deduplicated.",
        "warning",
      ],
      [
        "failed",
        "Previous Takeout import failed",
        "The previous Takeout import ended with an error. Run Takeout again to retry; messages already saved locally will be deduplicated.",
        "danger",
      ],
      [
        "cancelled",
        "Previous Takeout import was cancelled",
        "The previous Takeout import was cancelled. Run Takeout again to continue collecting available history; messages already saved locally will be deduplicated.",
        "neutral",
      ],
      [
        "partial_completed",
        "Previous Takeout import completed with partial history",
        "The previous Takeout import completed with partial history. Running Takeout again may collect more available history and will deduplicate messages already saved locally, but it does not guarantee a complete archive.",
        "warning",
      ],
    ];

    for (const [recoveryKind, title, body, severity] of cases) {
      const recovery = takeoutRecovery({ recovery_kind: recoveryKind });
      expect(takeoutRecoveryTitle(recovery)).toBe(title);
      expect(takeoutRecoveryBody(recovery)).toBe(body);
      expect(takeoutRecoverySeverity(recovery)).toBe(severity);
    }
  });
```

- [x] **Step 2: Run the focused test and verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: fail in `formats distinct takeout recovery titles, bodies, and severity` because `takeoutRecoveryBody` still returns the old generic body.

- [x] **Step 3: Implement the recovery-kind body map**

In `src/lib/analysis-state.ts`, replace the current no-argument `takeoutRecoveryBody` function:

```ts
export function takeoutRecoveryBody() {
  return "Run Takeout again to continue collecting available history. Messages already saved locally will be deduplicated.";
}
```

with:

```ts
const TAKEOUT_RECOVERY_BODIES: Record<
  TakeoutImportRecoveryState["recovery_kind"],
  string
> = {
  interrupted:
    "The previous Takeout import stopped before Extractum could finish tracking it. Run Takeout again to start a fresh import; messages already saved locally will be deduplicated.",
  failed:
    "The previous Takeout import ended with an error. Run Takeout again to retry; messages already saved locally will be deduplicated.",
  cancelled:
    "The previous Takeout import was cancelled. Run Takeout again to continue collecting available history; messages already saved locally will be deduplicated.",
  partial_completed:
    "The previous Takeout import completed with partial history. Running Takeout again may collect more available history and will deduplicate messages already saved locally, but it does not guarantee a complete archive.",
};

export function takeoutRecoveryBody(recovery: TakeoutImportRecoveryState) {
  return TAKEOUT_RECOVERY_BODIES[recovery.recovery_kind];
}
```

In `src/lib/components/analysis/takeout-recovery-notice.svelte`, replace:

```ts
  const body = $derived(takeoutRecoveryBody());
```

with:

```ts
  const body = $derived(takeoutRecoveryBody(recovery));
```

- [x] **Step 4: Run the focused test and type check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
npm.cmd run check
```

Expected: the focused Vitest file passes, and `svelte-check` exits 0.

- [x] **Step 5: Commit Task 1**

Run:

```powershell
git add src/lib/analysis-state.ts src/lib/analysis-state.test.ts src/lib/components/analysis/takeout-recovery-notice.svelte
git commit -m "feat: clarify takeout recovery policy copy"
```

### Task 2: Warning-Code Explanations

**Files:**
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/components/analysis/takeout-recovery-notice.svelte`

- [x] **Step 1: Write the failing warning-explanation test**

In the import list at the top of `src/lib/analysis-state.test.ts`, add `takeoutRecoveryWarningExplanations` next to the other Takeout recovery helpers:

```ts
  takeoutRecoverySeverity,
  takeoutRecoveryTitle,
  takeoutRecoveryWarningExplanations,
  upsertTakeoutImportJob,
```

Add this test after the recovery title/body/severity test:

```ts
  it("explains known takeout recovery warning codes without inventing unknown explanations", () => {
    expect(takeoutRecoveryWarningExplanations(takeoutRecovery({
      warning_codes: [
        "only_my_messages_fallback",
        "migrated_history_deferred",
        "export_dc_fallback",
        "finish_takeout_failed",
        "new_future_warning",
      ],
    }))).toEqual([
      "Telegram limited available channel or supergroup history; the import used the only-my-messages fallback.",
      "Migrated small-group history was detected and intentionally deferred.",
      "The import used the home-DC fallback after an export-DC path was attempted.",
      "Extractum could not cleanly finish the Takeout session after a terminal error. Local provenance remains available.",
    ]);

    expect(takeoutRecoveryWarningExplanations(takeoutRecovery({
      warning_codes: ["new_future_warning"],
    }))).toEqual([]);
  });
```

- [x] **Step 2: Run the focused test and verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: fail because `takeoutRecoveryWarningExplanations` is not exported yet.

- [x] **Step 3: Implement the warning-code helper**

In `src/lib/analysis-state.ts`, add this constant near the recovery body map:

```ts
const TAKEOUT_RECOVERY_WARNING_EXPLANATIONS: Record<string, string> = {
  only_my_messages_fallback:
    "Telegram limited available channel or supergroup history; the import used the only-my-messages fallback.",
  migrated_history_deferred:
    "Migrated small-group history was detected and intentionally deferred.",
  export_dc_fallback:
    "The import used the home-DC fallback after an export-DC path was attempted.",
  finish_takeout_failed:
    "Extractum could not cleanly finish the Takeout session after a terminal error. Local provenance remains available.",
};
```

Add this exported helper after `takeoutRecoverySeverity`:

```ts
export function takeoutRecoveryWarningExplanations(
  recovery: TakeoutImportRecoveryState,
) {
  return recovery.warning_codes.flatMap((code) => {
    const explanation = TAKEOUT_RECOVERY_WARNING_EXPLANATIONS[code];
    return explanation ? [explanation] : [];
  });
}
```

- [x] **Step 4: Render explanations in full recovery notices only**

In `src/lib/components/analysis/takeout-recovery-notice.svelte`, add the helper to the import:

```ts
    takeoutRecoverySeverity,
    takeoutRecoveryTitle,
    takeoutRecoveryWarningExplanations,
  } from "$lib/analysis-state";
```

Add a derived value after `facts`:

```ts
  const warningExplanations = $derived(takeoutRecoveryWarningExplanations(recovery));
```

After the warning-code badge block, add:

```svelte
  {#if !compact && warningExplanations.length > 0}
    <ul class="takeout-recovery-explanations">
      {#each warningExplanations as explanation (explanation)}
        <li>{explanation}</li>
      {/each}
    </ul>
  {/if}
```

Add CSS before `.takeout-recovery-error`:

```css
  .takeout-recovery-explanations {
    margin: 0;
    padding-left: 1rem;
    color: var(--muted);
    font-size: 0.78rem;
    line-height: 1.4;
  }

  .takeout-recovery-explanations li + li {
    margin-top: 0.2rem;
  }
```

- [x] **Step 5: Run the focused test and type check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
npm.cmd run check
```

Expected: the focused Vitest file passes, and `svelte-check` exits 0. The compact notice path still renders badges only because the explanations block is guarded by `!compact`.

- [x] **Step 6: Commit Task 2**

Run:

```powershell
git add src/lib/analysis-state.ts src/lib/analysis-state.test.ts src/lib/components/analysis/takeout-recovery-notice.svelte
git commit -m "feat: explain takeout recovery warning codes"
```

### Task 3: Backlog And Final Verification

**Files:**
- Modify: `docs/backlog.md`

- [x] **Step 1: Update the Takeout backlog item**

In `docs/backlog.md`, replace:

```markdown
- [ ] define richer incomplete-import recovery actions and user policy beyond
  the shipped read-only recovery state
```

with:

```markdown
- [x] define richer incomplete-import recovery actions and user policy beyond
  the shipped read-only recovery state
  - Implemented the safe recovery-policy slice: failed, cancelled,
    interrupted, and partial-completed Takeout notices now describe the
    safe re-run policy and known warning-code limitations without adding
    discard, persisted dismiss, or true resume behavior.
```

- [x] **Step 2: Run final frontend verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- `npm.cmd test` exits 0 and reports the Vitest suite passed.
- `npm.cmd run check` exits 0.
- `git diff --check` exits 0 with no whitespace errors.

No `cargo test` is required for this slice because no Rust code, migrations, or backend DTOs changed.

- [x] **Step 3: Commit Task 3**

Run:

```powershell
git add docs/backlog.md
git commit -m "docs: record takeout recovery policy completion"
```

- [x] **Step 4: Confirm final repository state**

Run:

```powershell
git status --short --branch
git log --oneline -3
```

Expected: `git status --short --branch` shows `## main`, and the latest three commits include the two feature commits plus the backlog completion commit.
