# Source Provider Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare the shared source model, source UI, backend sync boundary, and analysis refs for non-Telegram providers without adding YouTube, RSS, or forum ingestion.

**Architecture:** Keep Telegram behavior intact while making the shared layer provider-neutral. Add frontend source capabilities, expose a provider subtype in source records, dispatch sync by provider, and move new analysis refs to local item identity while preserving legacy Telegram refs.

**Tech Stack:** Svelte 5, TypeScript, Vitest, Tauri, Rust, SQLx, SQLite migrations, Cargo tests.

---

## Scope And Guardrails

This plan implements provider readiness only. It does not add YouTube URL parsing, playlist sync, transcript retrieval, RSS ingest, forum ingest, OAuth, or a provider plugin registry.

The current working branch is `main`. Before implementation, confirm the tree is clean:

```powershell
git status --short --branch
```

Expected before starting:

```text
## main
```

If the tree is dirty, inspect the changes and do not overwrite user work.

## File Structure

- Modify `src/lib/types/sources.ts`: expand source provider types, add provider subtype and capabilities types, make `telegramSourceKind` nullable in the shared `Source`.
- Create `src/lib/source-capabilities.ts`: central place for provider capability derivation and source labels.
- Create `src/lib/source-capabilities.test.ts`: focused tests for Telegram and non-Telegram capability behavior.
- Modify `src/lib/analysis-source-state.ts`: use source capabilities for sync disabled reasons and re-export source label helpers for current route callers.
- Modify `src/lib/analysis-source-state.test.ts`: pin non-syncable YouTube video behavior and Telegram compatibility.
- Modify `src/lib/api/sources.ts`: map optional `source_subtype` and optional `telegram_source_kind`.
- Modify `src/lib/api/sources.test.ts`: pin source mapping for both Telegram and a non-Telegram raw source.
- Modify `src/lib/analysis-state.ts`: make topic selector decisions capability-driven.
- Modify `src/lib/analysis-state.test.ts`: pin topic selector behavior for topic-capable and non-topic-capable sources.
- Modify `src/lib/components/source-row.svelte`: show source kind, membership, and sync action from capabilities.
- Modify `src/lib/components/analysis/workspace-rail.svelte`: hide Takeout and membership UI unless capabilities allow them.
- Modify `src/lib/components/analysis/workspace-main.svelte`: render provider-neutral source labels and sync state.
- Modify `src/routes/analysis/+page.svelte`: pass source-aware label helpers to components.
- Create `src-tauri/migrations/15.sql`: add `source_subtype` compatibility column and backfill Telegram rows.
- Modify `src-tauri/src/sources/types.rs`: add provider constants, optional `source_subtype` on shared source records, and provider parsing tests.
- Modify `src-tauri/src/sources/test_support.rs`: keep test schema aligned with provider-ready source fields.
- Modify `src-tauri/src/sources/store.rs`: select, insert, and return `source_subtype`; keep Telegram add command explicit.
- Modify `src-tauri/src/sources/sync.rs`: add a pure provider dispatch decision and call Telegram sync only for Telegram sources.
- Modify `src-tauri/src/analysis/corpus.rs`: emit new provider-neutral refs for live corpus rows.
- Modify `src-tauri/src/analysis/trace.rs`: accept new `s{source_id}-i{item_id}` refs and legacy `s{source_id}-m{message_id}` refs.
- Modify `src-tauri/src/analysis/report.rs`: update analysis prompt wording and tests from Telegram messages to source documents.
- Modify `src-tauri/src/analysis/chat.rs`: update follow-up prompt wording from Telegram messages to source documents.
- Modify `src-tauri/src/analysis/mod.rs`: update the default report template wording to source documents.

---

### Task 1: Frontend Source Contract And Capabilities

**Files:**
- Modify: `src/lib/types/sources.ts`
- Create: `src/lib/source-capabilities.ts`
- Create: `src/lib/source-capabilities.test.ts`
- Modify: `src/lib/analysis-source-state.ts`
- Modify: `src/lib/analysis-source-state.test.ts`

- [x] **Step 1: Write the failing capability tests**

Create `src/lib/source-capabilities.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  membershipLabel,
  sourceCapabilities,
  sourceKindLabel,
} from "./source-capabilities";
import type { Source } from "./types/sources";

function source(overrides: Partial<Source>): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "channel",
    telegramSourceKind: "channel",
    accountId: 1,
    externalId: "123",
    title: "Source",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    avatarDataUrl: null,
    ...overrides,
  };
}

describe("source capabilities", () => {
  it("keeps Telegram channel capabilities compatible", () => {
    const capabilities = sourceCapabilities(source({}));

    expect(capabilities).toEqual({
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: true,
      hasMembershipState: true,
      contentLabel: "messages",
    });
    expect(sourceKindLabel(source({}))).toBe("channel");
    expect(membershipLabel(source({ isMember: true }))).toBe("subscribed");
    expect(membershipLabel(source({ isMember: false }))).toBe("not subscribed");
  });

  it("marks Telegram supergroups as topic and Takeout capable", () => {
    const capabilities = sourceCapabilities(source({
      sourceSubtype: "supergroup",
      telegramSourceKind: "supergroup",
    }));

    expect(capabilities.hasTopics).toBe(true);
    expect(capabilities.canImportArchive).toBe(true);
    expect(sourceKindLabel(source({
      sourceSubtype: "supergroup",
      telegramSourceKind: "supergroup",
    }))).toBe("supergroup");
  });

  it("describes manual YouTube videos without Telegram assumptions", () => {
    const video = source({
      sourceType: "youtube",
      sourceSubtype: "video",
      telegramSourceKind: null,
      accountId: null,
      externalId: "dQw4w9WgXcQ",
      isMember: false,
    });

    expect(sourceCapabilities(video)).toEqual({
      canSync: false,
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "videos",
    });
    expect(sourceKindLabel(video)).toBe("YouTube video");
    expect(membershipLabel(video)).toBe("");
  });

  it("describes future YouTube playlists as syncable videos", () => {
    const playlist = source({
      sourceType: "youtube",
      sourceSubtype: "playlist",
      telegramSourceKind: null,
      accountId: null,
      isMember: false,
    });

    expect(sourceCapabilities(playlist)).toMatchObject({
      canSync: true,
      requiresAccount: false,
      contentLabel: "videos",
    });
    expect(sourceKindLabel(playlist)).toBe("YouTube playlist");
  });
});
```

In `src/lib/analysis-source-state.test.ts`, extend the source fixture and add this test:

```ts
function source(overrides: Partial<Source>): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "channel",
    telegramSourceKind: "channel",
    accountId: 1,
    externalId: "@extractum",
    title: "Extractum",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    avatarDataUrl: null,
    ...overrides,
  };
}
```

Add this test in the existing `describe("analysis-source-state", ...)` block:

```ts
it("does not require Telegram account runtime for non-syncable manual sources", () => {
  expect(sourceSyncDisabledReason(source({
    sourceType: "youtube",
    sourceSubtype: "video",
    telegramSourceKind: null,
    accountId: null,
    isMember: false,
  }), {})).toBe("This source type is not syncable.");
});
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts
```

Expected failure:

```text
Cannot find module './source-capabilities'
```

The analysis-source-state test may also fail with TypeScript errors because `Source` does not yet have `sourceSubtype` and `telegramSourceKind` is not nullable.

- [x] **Step 3: Update source types**

In `src/lib/types/sources.ts`, replace the top source type section with:

```ts
export type TelegramSourceKind = "channel" | "supergroup" | "group";
export type SourceType = "telegram" | "youtube" | "rss" | "forum";
export type SourceSubtype =
  | TelegramSourceKind
  | "video"
  | "playlist"
  | "feed"
  | "thread"
  | "board"
  | "site";
export type SourceContentLabel = "messages" | "videos" | "posts" | "items";
export type InitialSyncMode = "recent_messages" | "recent_days";
```

In the same file, replace `Source` with:

```ts
export interface Source {
  id: number;
  sourceType: SourceType;
  sourceSubtype: SourceSubtype | null;
  telegramSourceKind: TelegramSourceKind | null;
  accountId: number | null;
  externalId: string;
  title: string | null;
  lastSyncState: number | null;
  lastSyncedAt: number | null;
  isMember: boolean;
  isActive: boolean;
  createdAt: number;
  avatarDataUrl: string | null;
}
```

Add this interface after `Source`:

```ts
export interface SourceCapabilities {
  canSync: boolean;
  canDelete: boolean;
  canImportArchive: boolean;
  hasTopics: boolean;
  requiresAccount: boolean;
  hasMembershipState: boolean;
  contentLabel: SourceContentLabel;
}
```

- [x] **Step 4: Add capability helper implementation**

Create `src/lib/source-capabilities.ts`:

```ts
import type {
  Source,
  SourceCapabilities,
  SourceSubtype,
  TelegramSourceKind,
} from "$lib/types/sources";

function telegramKind(source: Pick<Source, "telegramSourceKind" | "sourceSubtype">) {
  return source.telegramSourceKind ?? telegramSubtype(source.sourceSubtype);
}

function telegramSubtype(subtype: SourceSubtype | null): TelegramSourceKind | null {
  return subtype === "channel" || subtype === "supergroup" || subtype === "group"
    ? subtype
    : null;
}

export function sourceCapabilities(source: Source): SourceCapabilities {
  if (source.sourceType === "telegram") {
    const kind = telegramKind(source);
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: kind === "supergroup",
      hasTopics: kind === "supergroup",
      requiresAccount: true,
      hasMembershipState: true,
      contentLabel: "messages",
    };
  }

  if (source.sourceType === "youtube") {
    return {
      canSync: source.sourceSubtype === "playlist",
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "videos",
    };
  }

  if (source.sourceType === "rss") {
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: false,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "posts",
    };
  }

  if (source.sourceType === "forum") {
    return {
      canSync: true,
      canDelete: true,
      canImportArchive: false,
      hasTopics: true,
      requiresAccount: false,
      hasMembershipState: false,
      contentLabel: "posts",
    };
  }

  return {
    canSync: false,
    canDelete: true,
    canImportArchive: false,
    hasTopics: false,
    requiresAccount: false,
    hasMembershipState: false,
    contentLabel: "items",
  };
}

export function sourceKindLabel(source: Source) {
  if (source.sourceType === "telegram") {
    return telegramKind(source) ?? "telegram";
  }
  if (source.sourceType === "youtube") {
    return source.sourceSubtype === "playlist" ? "YouTube playlist" : "YouTube video";
  }
  if (source.sourceType === "rss") {
    return "RSS feed";
  }
  if (source.sourceType === "forum") {
    return source.sourceSubtype === "thread" ? "forum thread" : "forum";
  }
  return source.sourceType;
}

export function membershipLabel(source: Source) {
  if (!sourceCapabilities(source).hasMembershipState) {
    return "";
  }

  const kind = telegramKind(source);
  if (kind === "channel") {
    return source.isMember ? "subscribed" : "not subscribed";
  }
  return source.isMember ? "member" : "not a member";
}
```

- [x] **Step 5: Update analysis source state to use capabilities**

In `src/lib/analysis-source-state.ts`, add:

```ts
import {
  membershipLabel as sourceMembershipLabel,
  sourceCapabilities,
  sourceKindLabel as providerSourceKindLabel,
} from "$lib/source-capabilities";
```

Replace the existing `sourceKindLabel` and `membershipLabel` functions with:

```ts
export function sourceKindLabel(source: Source) {
  return providerSourceKindLabel(source);
}

export function membershipLabel(source: Source) {
  return sourceMembershipLabel(source);
}
```

Replace `sourceSyncDisabledReason` with:

```ts
export function sourceSyncDisabledReason(
  source: Source,
  accountStatuses: Record<number, AccountRuntimeStatus>,
) {
  const capabilities = sourceCapabilities(source);
  if (!capabilities.canSync) return "This source type is not syncable.";
  if (!capabilities.requiresAccount) return null;

  const runtime = runtimeStatus(source.accountId, accountStatuses);
  if (source.accountId === null) return "Source is not linked to an account.";
  if (!runtime || runtime.status === "not_initialized") {
    return "Initialize this account before syncing.";
  }
  if (runtime.status === "restoring") {
    return "This account is still restoring.";
  }
  if (runtime.status === "reauth_required") {
    return "Sign in to this account again before syncing.";
  }
  if (runtime.status === "restore_failed") {
    return runtime.message ?? "The saved Telegram session could not be restored.";
  }
  return null;
}
```

Update the existing "formats Telegram source kind and membership labels" test to call the new source-aware helpers:

```ts
expect(sourceKindLabel(source({ telegramSourceKind: "channel", sourceSubtype: "channel" }))).toBe("channel");
expect(sourceKindLabel(source({ telegramSourceKind: "supergroup", sourceSubtype: "supergroup" }))).toBe("supergroup");
expect(sourceKindLabel(source({ telegramSourceKind: "group", sourceSubtype: "group" }))).toBe("group");

expect(membershipLabel(source({ telegramSourceKind: "channel", sourceSubtype: "channel", isMember: true }))).toBe("subscribed");
expect(membershipLabel(source({ telegramSourceKind: "channel", sourceSubtype: "channel", isMember: false }))).toBe("not subscribed");
expect(membershipLabel(source({ telegramSourceKind: "group", sourceSubtype: "group", isMember: true }))).toBe("member");
expect(membershipLabel(source({ telegramSourceKind: "group", sourceSubtype: "group", isMember: false }))).toBe("not a member");
```

- [x] **Step 6: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts
```

Expected output:

```text
Test Files  2 passed
```

- [x] **Step 7: Commit Task 1**

```powershell
git add src/lib/types/sources.ts src/lib/source-capabilities.ts src/lib/source-capabilities.test.ts src/lib/analysis-source-state.ts src/lib/analysis-source-state.test.ts
git commit -m "refactor(sources): add provider capabilities"
```

---

### Task 2: Source API Mapping For Provider Fields

**Files:**
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/api/sources.test.ts`

- [x] **Step 1: Write failing mapping tests**

In `src/lib/api/sources.test.ts`, update existing raw Telegram fixtures to include:

```ts
source_subtype: "channel",
```

for channels and:

```ts
source_subtype: "supergroup",
```

for supergroups.

Add this test:

```ts
it("maps non-Telegram source fields without requiring telegram_source_kind", async () => {
  invokeMock.mockResolvedValueOnce([
    {
      id: 10,
      source_type: "youtube",
      source_subtype: "video",
      account_id: null,
      external_id: "dQw4w9WgXcQ",
      title: "Demo video",
      last_sync_state: null,
      last_synced_at: null,
      is_member: false,
      is_active: true,
      created_at: 1_700_500,
      avatar_data_url: null,
    },
  ]);

  await expect(listSources(null)).resolves.toEqual([
    {
      id: 10,
      sourceType: "youtube",
      sourceSubtype: "video",
      telegramSourceKind: null,
      accountId: null,
      externalId: "dQw4w9WgXcQ",
      title: "Demo video",
      lastSyncState: null,
      lastSyncedAt: null,
      isMember: false,
      isActive: true,
      createdAt: 1_700_500,
      avatarDataUrl: null,
    },
  ]);
});
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/api/sources.test.ts
```

Expected failure includes a mismatch where `sourceSubtype` is absent and `telegramSourceKind` is not `null`.

- [x] **Step 3: Update raw source mapping**

In `src/lib/api/sources.ts`, add `SourceSubtype` to the type import:

```ts
SourceSubtype,
```

Replace `RawSource` with:

```ts
interface RawSource {
  id: number;
  source_type: Source["sourceType"];
  source_subtype?: SourceSubtype | null;
  telegram_source_kind?: TelegramSourceKind | null;
  account_id: number | null;
  external_id: string;
  title: string | null;
  last_sync_state: number | null;
  last_synced_at: number | null;
  is_member: boolean;
  is_active: boolean;
  created_at: number;
  avatar_data_url: string | null;
}
```

Replace `mapSource` with:

```ts
function mapSource(source: RawSource): Source {
  return {
    id: source.id,
    sourceType: source.source_type,
    sourceSubtype: source.source_subtype ?? source.telegram_source_kind ?? null,
    telegramSourceKind: source.telegram_source_kind ?? null,
    accountId: source.account_id,
    externalId: source.external_id,
    title: source.title,
    lastSyncState: source.last_sync_state,
    lastSyncedAt: source.last_synced_at,
    isMember: source.is_member,
    isActive: source.is_active,
    createdAt: source.created_at,
    avatarDataUrl: source.avatar_data_url,
  };
}
```

- [x] **Step 4: Run focused tests**

Run:

```powershell
npm.cmd test -- src/lib/api/sources.test.ts
```

Expected output:

```text
Test Files  1 passed
```

- [x] **Step 5: Commit Task 2**

```powershell
git add src/lib/api/sources.ts src/lib/api/sources.test.ts
git commit -m "refactor(sources): map provider source fields"
```

---

### Task 3: Capability-Driven UI And Topic Decisions

**Files:**
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/components/source-row.svelte`
- Modify: `src/lib/components/analysis/workspace-rail.svelte`
- Modify: `src/lib/components/analysis/workspace-main.svelte`
- Modify: `src/routes/analysis/+page.svelte`

- [x] **Step 1: Write failing topic selector tests**

In `src/lib/analysis-state.test.ts`, update `sourceRecord` with:

```ts
sourceSubtype: "channel",
```

Add this test near the existing topic selector test:

```ts
it("uses source capabilities for topic selector loading state", () => {
  const youtubeVideo = sourceRecord({
    sourceType: "youtube",
    sourceSubtype: "video",
    telegramSourceKind: null,
    accountId: null,
    isMember: false,
  });
  const supergroup = sourceRecord({
    sourceSubtype: "supergroup",
    telegramSourceKind: "supergroup",
  });

  expect(shouldShowTopicSelector(youtubeVideo, "single_source", true, [])).toBe(false);
  expect(shouldShowTopicSelector(supergroup, "single_source", true, [])).toBe(true);
});
```

Replace the old local `supergroup` and `channel` objects in the existing "shows topic selector only..." test with `sourceRecord(...)` objects:

```ts
const supergroup = sourceRecord({
  sourceSubtype: "supergroup",
  telegramSourceKind: "supergroup",
});
const channel = sourceRecord({
  sourceSubtype: "channel",
  telegramSourceKind: "channel",
});
```

- [x] **Step 2: Run analysis-state tests to verify failure**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected failure references `shouldShowTopicSelector` still accepting `Pick<Source, "telegramSourceKind">`.

- [x] **Step 3: Update topic selector logic**

In `src/lib/analysis-state.ts`, add:

```ts
import { sourceCapabilities } from "$lib/source-capabilities";
```

Replace `shouldShowTopicSelector` with:

```ts
export function shouldShowTopicSelector(
  source: Source | null,
  analysisScope: "single_source" | "source_group",
  loadingSourceTopics: boolean,
  topics: SourceForumTopic[],
) {
  if (!source || analysisScope !== "single_source") {
    return false;
  }

  if (loadingSourceTopics) {
    return sourceCapabilities(source).hasTopics;
  }

  return hasRealForumTopics(topics);
}
```

- [x] **Step 4: Update `source-row.svelte` to use capabilities**

In `src/lib/components/source-row.svelte`, import helpers:

```ts
import { membershipLabel, sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
```

Remove `sourceKindLabel` and `membershipLabel` from props. Add derived values:

```ts
const capabilities = $derived(sourceCapabilities(source));
const kindLabel = $derived(sourceKindLabel(source));
const sourceMembershipLabel = $derived(membershipLabel(source));
```

Replace the kind and membership badge block with:

```svelte
<Badge>{kindLabel}</Badge>
{#if source.lastSyncedAt !== null}
  <Badge>synced {formatDate(source.lastSyncedAt)}</Badge>
{/if}
{#if capabilities.requiresAccount && source.accountId !== null}
  {#if runtimeBadgeLabel}
    <Badge
      variant="warning"
      title={sourceRuntimeStatus?.status === "restore_failed" && sourceRuntimeStatus.message
        ? sourceRuntimeStatus.message
        : undefined}
    >
      {runtimeBadgeLabel}
    </Badge>
  {/if}
{/if}
{#if capabilities.hasMembershipState && sourceMembershipLabel}
  <Badge variant={source.isMember ? "member" : undefined}>{sourceMembershipLabel}</Badge>
{/if}
{#if capabilities.canSync}
  <Button
    size="sm"
    onclick={() => onSync(source.id)}
    disabled={syncing || deleting || syncReason !== null}
    title={syncReason ?? undefined}
  >
    {syncing ? "Syncing..." : "Sync"}
  </Button>
{/if}
```

- [x] **Step 5: Update `workspace-rail.svelte` props and actions**

In `src/lib/components/analysis/workspace-rail.svelte`, import helpers:

```ts
import { membershipLabel, sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
```

Remove `sourceKindLabel` and `membershipLabel` from the props type and destructuring.

Inside the source loop, after `{@const syncReason = sourceSyncDisabledReason(source)}`, add:

```svelte
{@const capabilities = sourceCapabilities(source)}
{@const kindLabel = sourceKindLabel(source)}
{@const sourceMembershipLabel = membershipLabel(source)}
```

Replace:

```svelte
<span>{sourceKindLabel(source.telegramSourceKind)}</span>
```

with:

```svelte
<span>{kindLabel}</span>
```

Replace the membership badge with:

```svelte
{#if capabilities.hasMembershipState && sourceMembershipLabel}
  <Badge>{sourceMembershipLabel}</Badge>
{/if}
```

Wrap the Sync button with:

```svelte
{#if capabilities.canSync}
  <Button
    size="sm"
    variant="secondary"
    onclick={() => onSyncSource(source.id)}
    disabled={!!syncingIds[source.id] || deleting || takeoutActive || syncReason !== null}
    title={takeoutActive ? "Takeout import is active." : syncReason ?? undefined}
  >
    {syncingIds[source.id] ? "Syncing..." : "Sync"}
  </Button>
{/if}
```

Wrap the Takeout button/cancel block with:

```svelte
{#if capabilities.canImportArchive}
  {#if takeoutActive && takeoutJob}
    <Button
      size="sm"
      variant="secondary"
      onclick={() => onCancelTakeoutImport(takeoutJob.job_id)}
      disabled={takeoutJob.status === "cancel_requested"}
    >
      {takeoutJob.status === "cancel_requested" ? "Cancelling..." : "Cancel"}
    </Button>
  {:else}
    <Button
      size="sm"
      variant="secondary"
      onclick={() => onStartTakeoutImport(source.id)}
      disabled={startingTakeout || deleting || !!syncingIds[source.id] || syncReason !== null}
      title={syncReason ?? undefined}
    >
      {startingTakeout ? "Starting..." : "Takeout"}
    </Button>
  {/if}
{/if}
```

- [x] **Step 6: Update `workspace-main.svelte` source labels**

In `src/lib/components/analysis/workspace-main.svelte`, import:

```ts
import { sourceKindLabel } from "$lib/source-capabilities";
```

Remove `sourceKindLabel` from props. Replace:

```svelte
<Badge variant="info">{sourceKindLabel(currentSource.telegramSourceKind)}</Badge>
```

with:

```svelte
<Badge variant="info">{sourceKindLabel(currentSource)}</Badge>
```

- [x] **Step 7: Update route prop passing**

In `src/routes/analysis/+page.svelte`, remove `sourceKindLabel` and `membershipLabel` from the `analysis-source-state` import and from `<WorkspaceRail>` and `<WorkspaceMain>` props.

Keep `sourceSyncDisabledReason` unchanged as the route-local wrapper around `getSourceSyncDisabledReason`.

- [x] **Step 8: Run focused frontend tests and Svelte check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts
npm.cmd run check
```

Expected output:

```text
Test Files  3 passed
```

and:

```text
svelte-check found 0 errors and 0 warnings
```

- [x] **Step 9: Commit Task 3**

```powershell
git add src/lib/analysis-state.ts src/lib/analysis-state.test.ts src/lib/components/source-row.svelte src/lib/components/analysis/workspace-rail.svelte src/lib/components/analysis/workspace-main.svelte src/routes/analysis/+page.svelte
git commit -m "refactor(analysis): gate source UI by capabilities"
```

---

### Task 4: Backend Source Record Provider Readiness

**Files:**
- Create: `src-tauri/migrations/15.sql`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/sync.rs` for source record fixture alignment.
- Modify: `src-tauri/src/sources/peer_resolution.rs` for source sync target fixture alignment.

- [x] **Step 1: Write backend record tests first**

In `src-tauri/src/sources/types.rs`, extend the test module import:

```rust
use super::{SourceType, TelegramSourceKind};
```

Add this test:

```rust
#[test]
fn source_type_serializes_supported_provider_values() {
    assert_eq!(
        serde_json::to_string(&SourceType::Telegram).expect("serialize"),
        "\"telegram\""
    );
    assert_eq!(
        serde_json::to_string(&SourceType::Youtube).expect("serialize"),
        "\"youtube\""
    );
    assert_eq!(
        serde_json::to_string(&SourceType::Rss).expect("serialize"),
        "\"rss\""
    );
    assert_eq!(
        serde_json::to_string(&SourceType::Forum).expect("serialize"),
        "\"forum\""
    );
}
```

In `src-tauri/src/sources/store.rs`, add this unit test inside the existing test module:

```rust
#[test]
fn source_record_parts_allow_non_telegram_source() {
    let record = source_record_from_row_parts(
        SourceRecordRow {
            id: 10,
            source_type: "youtube".to_string(),
            source_subtype: Some("video".to_string()),
            telegram_source_kind: None,
            account_id: None,
            external_id: "dQw4w9WgXcQ".to_string(),
            title: Some("Demo video".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
            last_synced_at: None,
            is_active: true,
            is_member: false,
            created_at: 1_700_500,
        },
        None,
    );

    assert_eq!(record.source_type, "youtube");
    assert_eq!(record.source_subtype.as_deref(), Some("video"));
    assert_eq!(record.telegram_source_kind, None);
    assert_eq!(record.account_id, None);
}
```

- [x] **Step 2: Run backend tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test sources::
```

Expected failure includes missing `SourceType::Youtube` and missing `source_record_from_row_parts` or missing `source_subtype` fields.

- [x] **Step 3: Add migration 15**

Create `src-tauri/migrations/15.sql`:

```sql
ALTER TABLE sources ADD COLUMN source_subtype TEXT;

UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype IS NULL;
```

- [x] **Step 4: Update backend source types**

In `src-tauri/src/sources/types.rs`, add constants:

```rust
pub(crate) const YOUTUBE_SOURCE_TYPE: &str = "youtube";
pub(crate) const RSS_SOURCE_TYPE: &str = "rss";
pub(crate) const FORUM_SOURCE_TYPE: &str = "forum";
```

Replace `SourceType` with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Telegram,
    Youtube,
    Rss,
    Forum,
}

impl SourceType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => TELEGRAM_SOURCE_TYPE,
            Self::Youtube => YOUTUBE_SOURCE_TYPE,
            Self::Rss => RSS_SOURCE_TYPE,
            Self::Forum => FORUM_SOURCE_TYPE,
        }
    }
}
```

Add `source_subtype` and make shared Telegram kind optional in `SourceRecord`:

```rust
pub struct SourceRecord {
    pub id: i64,
    pub source_type: String,
    pub source_subtype: Option<String>,
    pub telegram_source_kind: Option<String>,
    pub account_id: Option<i64>,
    pub external_id: String,
    pub title: Option<String>,
    pub last_sync_state: Option<i64>,
    pub last_synced_at: Option<i64>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub avatar_data_url: Option<String>,
}
```

Add `source_subtype: Option<String>` to `SourceSyncTarget`, and add `source_subtype: Option<String>` plus `telegram_source_kind: Option<String>` to `SourceRecordRow`.

Keep `SourceSyncTarget.telegram_source_kind` as `String` for this readiness pass so existing Telegram sync code stays narrow. If `load_source` sees a null Telegram kind for a Telegram source, it should return a validation error in Task 5 before Telegram sync uses it.

- [x] **Step 5: Update test support schema**

In `src-tauri/src/sources/test_support.rs`, change the sources table definition to:

```sql
CREATE TABLE sources (
    id INTEGER PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_subtype TEXT,
    telegram_source_kind TEXT,
    account_id INTEGER,
    external_id TEXT NOT NULL,
    title TEXT,
    metadata_zstd BLOB,
    last_sync_state INTEGER,
    last_synced_at INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_member INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
)
```

- [x] **Step 6: Update store queries and record mapping**

In `src-tauri/src/sources/store.rs`, update the `INSERT INTO sources` field list in `add_telegram_source` to include `source_subtype` immediately after `source_type`:

```sql
INSERT INTO sources (
    source_type,
    source_subtype,
    telegram_source_kind,
    external_id,
    title,
    metadata_zstd,
    is_active,
    is_member,
    account_id,
    created_at
)
VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)
```

Bind the subtype:

```rust
.bind(SourceType::Telegram.as_str())
.bind(&resolved.telegram_source_kind)
.bind(&resolved.telegram_source_kind)
```

Add `source_subtype` to every `RETURNING` and `SELECT` for `SourceRecordRow`. Example list query:

```sql
SELECT id, source_type, source_subtype, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources ORDER BY created_at DESC
```

Update `load_source` to select `source_subtype` too:

```sql
SELECT id, source_type, source_subtype, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?
```

Add this helper below `list_sources`:

```rust
fn source_record_from_row_parts(
    row: SourceRecordRow,
    avatar_data_url: Option<String>,
) -> SourceRecord {
    SourceRecord {
        id: row.id,
        source_type: row.source_type,
        source_subtype: row.source_subtype,
        telegram_source_kind: row.telegram_source_kind,
        account_id: row.account_id,
        external_id: row.external_id,
        title: row.title,
        last_sync_state: row.last_sync_state,
        last_synced_at: row.last_synced_at,
        is_member: row.is_member,
        is_active: row.is_active,
        created_at: row.created_at,
        avatar_data_url,
    }
}
```

Replace the final `Ok(SourceRecord { ... })` in `source_record_from_row` with:

```rust
Ok(source_record_from_row_parts(row, avatar_data_url))
```

- [x] **Step 7: Run sources backend tests**

Run:

```powershell
cd src-tauri
cargo test sources::
```

Expected output:

```text
test result: ok.
```

- [x] **Step 8: Commit Task 4**

```powershell
git add src-tauri/migrations/15.sql src-tauri/src/sources/types.rs src-tauri/src/sources/test_support.rs src-tauri/src/sources/store.rs src-tauri/src/sources/sync.rs src-tauri/src/sources/peer_resolution.rs
git commit -m "refactor(sources): expose provider subtype"
```

---

### Task 5: Backend Sync Provider Dispatcher

**Files:**
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/types.rs` if Task 4 leaves a compile gap around `SourceSyncTarget`

- [x] **Step 1: Add dispatcher tests first**

In `src-tauri/src/sources/sync.rs`, extend the test module import:

```rust
use super::{determine_sync_policy, finalize_sync, sync_provider_for_source, SyncProvider};
```

Add tests:

```rust
#[test]
fn sync_provider_accepts_telegram_sources() {
    let source = SourceSyncTarget {
        id: 1,
        source_type: TELEGRAM_SOURCE_TYPE.to_string(),
        source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
        telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
        account_id: Some(1),
        external_id: "12345".to_string(),
        title: Some("Example".to_string()),
        metadata_zstd: None,
        last_sync_state: None,
    };

    assert_eq!(sync_provider_for_source(&source).unwrap(), SyncProvider::Telegram);
}

#[test]
fn sync_provider_rejects_manual_youtube_video_sources() {
    let source = SourceSyncTarget {
        id: 7,
        source_type: "youtube".to_string(),
        source_subtype: Some("video".to_string()),
        telegram_source_kind: "".to_string(),
        account_id: None,
        external_id: "dQw4w9WgXcQ".to_string(),
        title: Some("Demo video".to_string()),
        metadata_zstd: None,
        last_sync_state: None,
    };

    let error = sync_provider_for_source(&source).expect_err("manual video is not syncable");

    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("Source 7"));
    assert!(error.message.contains("youtube"));
    assert!(error.message.contains("not syncable"));
}
```

- [x] **Step 2: Run sync tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test sources::sync
```

Expected failure includes missing `SyncProvider` or `sync_provider_for_source`.

- [x] **Step 3: Implement pure dispatch decision**

In `src-tauri/src/sources/sync.rs`, add near the local structs:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SyncProvider {
    Telegram,
}

fn sync_provider_for_source(source: &SourceSyncTarget) -> AppResult<SyncProvider> {
    match source.source_type.as_str() {
        crate::sources::types::TELEGRAM_SOURCE_TYPE => Ok(SyncProvider::Telegram),
        other => Err(AppError::validation(format!(
            "Source {} with source_type '{}' is not syncable",
            source.id, other
        ))),
    }
}
```

- [x] **Step 4: Use dispatch in `sync_source`**

In `sync_source`, after loading `source`, add:

```rust
let provider = sync_provider_for_source(&source)?;
match provider {
    SyncProvider::Telegram => sync_telegram_source(handle, state, source).await,
}
```

Extract the existing body after `let source = load_source(&pool, source_id).await?;` into:

```rust
async fn sync_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source: SourceSyncTarget,
) -> AppResult<SyncResult> {
    let pool = get_pool(&handle).await?;
    let source_id = source.id;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;

    let runtime = crate::telegram::get_authorized_runtime(&state, account_id).await?;
    let client = runtime.client;
    let resolved_peer = resolve_and_refresh_peer(&handle, &client, &source, account_id).await?;
    let forum_topic_warnings =
        refresh_forum_topics(&pool, &client, resolved_peer.peer, &source).await;
    let sync_policy = determine_sync_policy(&pool, &source).await?;
    let ingest = persist_items(&pool, &client, resolved_peer.peer, &source, &sync_policy).await?;
    let last_sync_state = finalize_sync(
        &pool,
        &source,
        sync_policy.previous_last_sync,
        ingest.max_message_id,
        resolved_peer.refreshed_metadata_zstd,
    )
    .await?;

    Ok(SyncResult {
        inserted: ingest.inserted,
        skipped: ingest.skipped,
        last_message_id: last_sync_state,
        initial_sync_policy_applied: sync_policy.initial_sync_policy_applied,
        warnings: forum_topic_warnings,
    })
}
```

Keep the ingest lock acquisition in `sync_source`, before the dispatch, so delete/sync coordination remains unchanged.

- [x] **Step 5: Run sync tests**

Run:

```powershell
cd src-tauri
cargo test sources::sync
```

Expected output:

```text
test result: ok.
```

- [x] **Step 6: Commit Task 5**

```powershell
git add src-tauri/src/sources/sync.rs src-tauri/src/sources/types.rs
git commit -m "refactor(sources): dispatch sync by provider"
```

---

### Task 6: Provider-Neutral Analysis Refs And Prompt Wording

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/trace.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/mod.rs`

- [x] **Step 1: Write failing ref tests**

In `src-tauri/src/analysis/trace.rs`, update the test import:

```rust
use super::{build_trace_refs, clip_excerpt, normalize_ref};
```

Add:

```rust
#[test]
fn normalize_ref_accepts_item_refs_and_legacy_message_refs() {
    assert_eq!(normalize_ref("[s12-i845]").as_deref(), Some("s12-i845"));
    assert_eq!(normalize_ref("s12-m845").as_deref(), Some("s12-m845"));
    assert_eq!(normalize_ref("s12-iabc"), None);
    assert_eq!(normalize_ref("x12-i845"), None);
}
```

In `src-tauri/src/analysis/corpus.rs`, update expectations in tests so live corpus refs use item ids:

```rust
assert_eq!(corpus[1].r#ref, "s4-i12");
```

where the expected item id is the row `id`. Keep snapshot tests that load saved `s*-m*` refs unchanged, because legacy snapshots must remain readable.

In `src-tauri/src/analysis/report.rs`, change `SAMPLE_JSON` to:

```rust
const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-i2"]}"#;
```

Change `sample_corpus_message().r#ref` to:

```rust
r#ref: "s2-i1".to_string(),
```

Add assertions in `build_map_request_keeps_run_scoped_request_and_profile`:

```rust
assert!(request.messages[0].content.contains("source document excerpts"));
assert!(request.messages[1].content.contains("Documents:"));
```

Add an assertion in `build_reduce_request_keeps_run_scoped_request_and_profile`:

```rust
assert!(request.messages[0].content.contains("[s12-i845]"));
```

- [x] **Step 2: Run analysis tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test analysis::
```

Expected failure includes `normalize_ref("[s12-i845]")` returning `None` and prompt wording assertions failing.

- [x] **Step 3: Change live corpus refs**

In `src-tauri/src/analysis/corpus.rs`, replace:

```rust
r#ref: format!("s{}-m{}", row.source_id, row.external_id),
```

with:

```rust
r#ref: format!("s{}-i{}", row.source_id, row.id),
```

- [x] **Step 4: Accept new refs and preserve legacy refs**

In `src-tauri/src/analysis/trace.rs`, replace `normalize_ref` with:

```rust
pub(crate) fn normalize_ref(candidate: &str) -> Option<String> {
    let candidate = candidate.trim().trim_matches('[').trim_matches(']');
    for separator in ["-i", "-m"] {
        let Some((source_part, item_part)) = candidate.split_once(separator) else {
            continue;
        };
        if !source_part.starts_with('s') {
            return None;
        }
        let source_digits = &source_part[1..];
        if source_digits.is_empty()
            || item_part.is_empty()
            || !source_digits.chars().all(|c| c.is_ascii_digit())
            || !item_part.chars().all(|c| c.is_ascii_digit())
        {
            return None;
        }

        return Some(format!("s{source_digits}{separator}{item_part}"));
    }

    None
}
```

- [x] **Step 5: Update report prompt wording**

In `src-tauri/src/analysis/report.rs`, replace the map system prompt content with:

```rust
content: "You analyze source document excerpts. Return a strict JSON object only with keys: summary, topics, notable_points, candidate_refs. Do not wrap JSON in markdown fences. Use only refs that appear in the provided documents.".to_string(),
```

Replace the map user prompt format string with:

```rust
"Chunk {chunk_index} of {total_chunks}.\nSummarize the source documents below for later reduction.\n\nDocuments:\n\n{}"
```

Replace the reduce system prompt with:

```rust
"You write grounded markdown reports over already-summarized source documents.\nAnswer in {}.\nUse markdown only.\nEvery important conclusion must cite one or more refs like [s12-i845].\nDo not invent facts beyond the provided chunk summaries."
```

In `run_report_pipeline`, replace messages:

```rust
.message("Loading synced source documents from local storage...".to_string())
```

and:

```rust
"No synced source documents were found for the selected analysis scope and period".to_string()
```

and:

```rust
"Loaded {} source documents. Preparing chunks..."
```

- [x] **Step 6: Update chat and default template wording**

In `src-tauri/src/analysis/chat.rs`, replace the system prompt in `build_chat_request` with:

```rust
"You answer follow-up questions about a saved source analysis report.\nAnswer in {}.\nUse markdown only.\nGround every important claim in the saved report or the provided source document excerpts.\nWhen referring to source evidence, cite refs like [s12-i845].\nDo not invent facts beyond the saved report and provided excerpts."
```

Replace the user prompt label:

```rust
"Additional local source document matches for the current question:\n\n{}"
```

In `src-tauri/src/analysis/mod.rs`, replace `default_report_template_body()` with:

```rust
fn default_report_template_body() -> &'static str {
    r#"Create a grounded report over the provided source documents.

Focus on:
- the main topics and recurring themes
- the most notable claims, updates, and shifts
- supporting examples from the source material

Always keep the report concise, readable, and useful for later follow-up analysis."#
}
```

- [x] **Step 7: Run analysis tests**

Run:

```powershell
cd src-tauri
cargo test analysis::
```

Expected output:

```text
test result: ok.
```

- [x] **Step 8: Commit Task 6**

```powershell
git add src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/trace.rs src-tauri/src/analysis/report.rs src-tauri/src/analysis/chat.rs src-tauri/src/analysis/mod.rs
git commit -m "refactor(analysis): use provider-neutral refs"
```

---

### Task 7: Full Verification And Route Boundary Check

**Files:**
- No required code changes.
- Modify docs only if implementation discovers a durable caveat that should be recorded in `docs/code-review-results-2026-05-03.md`.

- [x] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts src/lib/api/sources.test.ts src/lib/analysis-state.test.ts
```

Expected output:

```text
Test Files  4 passed
```

- [x] **Step 2: Run frontend type and Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected output:

```text
svelte-check found 0 errors and 0 warnings
```

- [x] **Step 3: Run focused backend tests**

Run:

```powershell
cd src-tauri
cargo test sources:: analysis::
```

If Cargo treats the two filters as unexpected arguments, run the two commands separately:

```powershell
cd src-tauri
cargo test sources::
cargo test analysis::
```

Expected output for each test command:

```text
test result: ok.
```

- [x] **Step 4: Check route raw Tauri boundaries**

Run from the repository root:

```powershell
rg -n "\binvoke\s*(<|\()|@tauri-apps/api/core" src/routes
rg -n "@tauri-apps/api/event|listen<" src/routes
```

Expected result for each command:

```text
no output, exit code 1
```

- [x] **Step 5: Check whitespace**

Run:

```powershell
git diff --check
```

Expected output:

```text
exit code 0
```

LF/CRLF warnings are acceptable if the command exits with 0.

- [x] **Step 6: Final commit if any verification-only docs changed**

If Task 7 changed docs, commit them:

```powershell
git add docs/code-review-results-2026-05-03.md
git commit -m "docs(sources): record provider readiness follow-up"
```

If Task 7 did not change files, do not create an empty commit.

- [x] **Step 7: Final status check**

Run:

```powershell
git status --short --branch
git log --oneline -8
```

Expected status:

```text
## main
```

Expected latest implementation commits include:

```text
refactor(analysis): use provider-neutral refs
refactor(sources): dispatch sync by provider
refactor(sources): expose provider subtype
refactor(analysis): gate source UI by capabilities
refactor(sources): map provider source fields
refactor(sources): add provider capabilities
```

## Self-Review Notes

- Spec coverage: the tasks cover shared source model, capabilities, backend provider boundaries, sync dispatch, corpus refs, explicit unsupported sync validation, and tests. Concrete provider ingestion is excluded by scope.
- Type consistency: the plan uses `sourceType`, `sourceSubtype`, `telegramSourceKind`, `SourceCapabilities`, and `sourceCapabilities(source)` consistently across tasks.
- Verification coverage: the plan ends with focused frontend tests, Svelte check, focused Rust tests, route raw Tauri searches, whitespace check, and git status.
