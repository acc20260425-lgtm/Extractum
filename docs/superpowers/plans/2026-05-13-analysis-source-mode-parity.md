# Analysis Source Mode Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore live `/analysis` Source mode parity for YouTube comments/jobs/activity and Telegram topic filtering without weakening run snapshot read-only behavior.

**Architecture:** Extend the current result-first source readers instead of returning to the old `WorkspaceMain` detail cards. Keep YouTube job rendering in a small focused component, pass live actions through `ReportCanvas` and `ReportSourceSurface`, and render Telegram topic filtering only in live single-source Telegram views.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest raw-source contract tests, existing Tauri-backed source APIs.

---

## Execution Rules

- Work from a clean `main`.
- Use TDD for each task: write the failing raw-source or route contract test, run it and confirm failure, then implement.
- Commit after each implementation task.
- Do not touch backend APIs in this pass.
- Do not reintroduce `YoutubeSourceDetail` or `YoutubePlaylistDetail` into `ReportSourceSurface`.
- Run `mcp__svelte_server__.list_sections` before editing Svelte components, then use `mcp__svelte_server__.svelte_autofixer` on every changed Svelte component before committing.
- Use `apply_patch` for manual edits.

## File Structure

- Create `src/lib/components/analysis/youtube-source-activity.svelte`: renders YouTube source jobs, progress, warnings, errors, status badges, and cancel controls.
- Modify `src/lib/components/analysis/youtube-transcript-reader.svelte`: adds live comments sync/status and uses `YoutubeSourceActivity`.
- Modify `src/lib/components/analysis/youtube-playlist-reader.svelte`: uses `YoutubeSourceActivity` for playlist jobs.
- Modify `src/lib/components/analysis/report-source-surface.svelte`: passes live YouTube comments/job props into readers and renders Telegram topic selector for live single-source Telegram material.
- Modify `src/routes/analysis/+page.svelte`: makes `currentSourceJobs()` include jobs where the selected source is `related_source_id`.
- Modify tests:
  - `src/lib/analysis-source-readers.test.ts`
  - `src/lib/analysis-report-canvas.test.ts`
  - `src/lib/analysis-source-readers-route.test.ts`
  - `src/lib/analysis-redesign-safety-contract.test.ts`

---

### Task 1: Add YouTube Source Activity Component

**Files:**
- Create: `src/lib/components/analysis/youtube-source-activity.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [ ] **Step 1: Add the failing component contract test**

In `src/lib/analysis-source-readers.test.ts`, add the raw import near the other component imports:

```ts
import youtubeSourceActivitySource from "./components/analysis/youtube-source-activity.svelte?raw";
```

Add this test inside `describe("analysis source readers", () => { ... })`:

```ts
it("renders YouTube source job activity with progress warnings errors and cancel", () => {
  expect(youtubeSourceActivitySource).toContain('class="youtube-source-activity"');
  expect(youtubeSourceActivitySource).toContain("SourceJobRecord");
  expect(youtubeSourceActivitySource).toContain("progressLabel(job)");
  expect(youtubeSourceActivitySource).toContain("job.warnings");
  expect(youtubeSourceActivitySource).toContain("job.error");
  expect(youtubeSourceActivitySource).toContain("onCancelJob(job.job_id)");
  expect(youtubeSourceActivitySource).toContain("cancel_requested");
});
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts
```

Expected: FAIL because `youtube-source-activity.svelte` does not exist.

- [ ] **Step 3: Create the activity component**

Create `src/lib/components/analysis/youtube-source-activity.svelte`:

```svelte
<script lang="ts">
  import { Square } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { SourceJobRecord } from "$lib/types/sources";
  import type { BadgeVariant } from "$lib/components/ui/types";

  let {
    jobs,
    formatTimestamp,
    onCancelJob,
    title = "Source activity",
  }: {
    jobs: SourceJobRecord[];
    formatTimestamp: (value: number | null) => string;
    onCancelJob: (jobId: string) => void | Promise<void>;
    title?: string;
  } = $props();

  const visibleJobs = $derived(
    [...jobs].sort((left, right) => right.started_at - left.started_at).slice(0, 8),
  );

  function isActiveJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function statusVariant(status: SourceJobRecord["status"]): BadgeVariant {
    if (status === "failed" || status === "cancelled") return "danger";
    if (status === "succeeded") return "success";
    if (status === "cancel_requested") return "warning";
    return "info";
  }

  function jobLabel(job: SourceJobRecord) {
    return job.job_type.replaceAll("_", " ");
  }

  function statusLabel(job: SourceJobRecord) {
    return job.status.replaceAll("_", " ");
  }

  function progressLabel(job: SourceJobRecord) {
    if (job.progress_current === null || job.progress_total === null) return null;
    return `${job.progress_current}/${job.progress_total}`;
  }
</script>

{#if visibleJobs.length > 0}
  <section class="youtube-source-activity" aria-label={title}>
    <div class="activity-heading">
      <span class="eyebrow">{title}</span>
      <Badge variant="neutral">{visibleJobs.length} recent</Badge>
    </div>

    <div class="activity-list">
      {#each visibleJobs as job (job.job_id)}
        {@const progress = progressLabel(job)}
        <article class="activity-row">
          <div class="activity-copy">
            <strong>{jobLabel(job)}</strong>
            <span>{job.message ?? job.error ?? statusLabel(job)}</span>
            <small>
              Started {formatTimestamp(job.started_at)}
              {#if job.finished_at !== null}
                - Finished {formatTimestamp(job.finished_at)}
              {/if}
            </small>
            {#if progress}
              <small>Progress {progress}</small>
            {/if}
            {#if job.warnings.length > 0}
              <ul class="warning-list" aria-label="Job warnings">
                {#each job.warnings as warning, index (`${job.job_id}-warning-${index}`)}
                  <li>{warning}</li>
                {/each}
              </ul>
            {/if}
            {#if job.error}
              <small class="job-error">{job.error}</small>
            {/if}
          </div>

          <div class="activity-actions">
            <Badge variant={statusVariant(job.status)}>{statusLabel(job)}</Badge>
            {#if isActiveJob(job)}
              <Button
                type="button"
                size="sm"
                variant="secondary"
                disabled={job.status === "cancel_requested"}
                onclick={() => onCancelJob(job.job_id)}
              >
                <Square size={13} aria-hidden="true" /> Cancel
              </Button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  </section>
{/if}

<style>
  .youtube-source-activity {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.75rem;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 58%, transparent);
  }

  .activity-heading,
  .activity-row,
  .activity-actions {
    display: flex;
    align-items: flex-start;
    gap: 0.55rem;
  }

  .activity-heading {
    justify-content: space-between;
    align-items: center;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .activity-row {
    justify-content: space-between;
    padding: 0.6rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .activity-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
  }

  .activity-copy span,
  .activity-copy small,
  .warning-list {
    color: var(--muted);
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  .warning-list {
    margin: 0.15rem 0 0;
    padding-left: 1rem;
  }

  .job-error {
    color: var(--danger);
  }

  .activity-actions {
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  @media (max-width: 760px) {
    .activity-row {
      flex-direction: column;
    }

    .activity-actions {
      justify-content: flex-start;
    }
  }
</style>
```

- [ ] **Step 4: Autofix the Svelte component**

Run the Svelte autofixer on `YoutubeSourceActivity.svelte`:

```text
Use mcp__svelte_server__.svelte_autofixer with filename "YoutubeSourceActivity.svelte" and desired_svelte_version 5.
```

Expected: no blocking Svelte issues remain.

- [ ] **Step 5: Verify the task**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts
npm.cmd run check
git diff --check
```

Expected:

```text
analysis-source-readers.test.ts passes
svelte-check found 0 errors and 0 warnings
git diff --check exits 0
```

- [ ] **Step 6: Commit**

Run:

```powershell
git add src/lib/components/analysis/youtube-source-activity.svelte src/lib/analysis-source-readers.test.ts
git commit -m "feat: add youtube source activity reader"
```

---

### Task 2: Restore YouTube Video Comments And Activity In Transcript Reader

**Files:**
- Modify: `src/lib/components/analysis/youtube-transcript-reader.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`

- [ ] **Step 1: Add failing source reader contract tests**

In `src/lib/analysis-source-readers.test.ts`, add:

```ts
it("restores live YouTube video comments sync status and activity in transcript reader", () => {
  expect(youtubeTranscriptSource).toContain("onSyncComments");
  expect(youtubeTranscriptSource).toContain("sourceJobs");
  expect(youtubeTranscriptSource).toContain("<YoutubeSourceActivity");
  expect(youtubeTranscriptSource).toContain("summary.comments.label");
  expect(youtubeTranscriptSource).toContain("summary.comments.itemCount");
  expect(youtubeTranscriptSource).toContain("summary.comments.lastSyncedAt");
  expect(youtubeTranscriptSource).toContain("Sync comments");
});

it("passes live YouTube video comments and jobs only into live transcript readers", () => {
  expect(reportSourceSurfaceSource).toContain("sourceJobs={sourceJobs}");
  expect(reportSourceSurfaceSource).toContain("onSyncComments={() => onSyncYoutubeComments(currentSource.id)}");
  expect(reportSourceSurfaceSource).toContain("onCancelSourceJob={onCancelSourceJob}");
  expect(reportSourceSurfaceSource).toContain("showSyncActions={false}");
  expect(reportSourceSurfaceSource).not.toContain("onSyncComments={() => {}}");
});
```

In `src/lib/analysis-report-canvas.test.ts`, add:

```ts
it("passes YouTube comments and source activity callbacks through the report canvas", () => {
  expect(reportCanvasSource).toContain("onSyncYoutubeComments={onSyncYoutubeComments}");
  expect(reportCanvasSource).toContain("onCancelSourceJob={onCancelSourceJob}");
  expect(reportCanvasSource).toContain("{sourceJobs}");
  expect(reportSourceSurfaceSource).toContain("onSyncYoutubeComments");
});
```

In `src/lib/analysis-redesign-safety-contract.test.ts`, extend the source ingest activity test with:

```ts
expect(reportSourceSurfaceSource).toContain("onSyncYoutubeComments");
expect(reportSourceSurfaceSource).toContain("onCancelSourceJob={onCancelSourceJob}");
expect(youtubeTranscriptSource).toContain("showSyncActions");
expect(youtubeTranscriptSource).toContain("Sync comments");
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: FAIL because `YoutubeTranscriptReader` does not yet accept comments/job props and `ReportSourceSurface` does not pass them.

- [ ] **Step 3: Update `YoutubeTranscriptReader` props and imports**

In `src/lib/components/analysis/youtube-transcript-reader.svelte`, add imports:

```svelte
  import YoutubeSourceActivity from "$lib/components/analysis/youtube-source-activity.svelte";
  import type { SourceJobRecord } from "$lib/types/sources";
```

Change the prop destructuring to include comments and jobs:

```svelte
    sourceJobs = [],
    onSyncComments = null,
    onCancelSourceJob = async () => {},
```

Use this prop type block for the new props:

```ts
    sourceJobs?: SourceJobRecord[];
    onSyncComments?: (() => void | Promise<void>) | null;
    onCancelSourceJob?: (jobId: string) => void | Promise<void>;
```

- [ ] **Step 4: Render comments status and comments sync**

In the transcript header metadata, after the captions badges, add:

```svelte
          <Badge variant={summary.comments.state === "synced" ? "success" : summary.comments.state === "failed" ? "danger" : "neutral"}>
            Comments {summary.comments.label}
          </Badge>
          <Badge variant="neutral">{summary.comments.itemCount} comments</Badge>
          <Badge variant="neutral">Comments synced {formatTimestamp(summary.comments.lastSyncedAt)}</Badge>
```

In `.transcript-actions`, add the comments button after `Sync transcript`:

```svelte
        {#if onSyncComments}
          <Button type="button" size="sm" variant="secondary" onclick={onSyncComments}>Sync comments</Button>
        {/if}
```

After the search field, render source activity only for live sync-enabled readers:

```svelte
  {#if showSyncActions}
    <YoutubeSourceActivity jobs={sourceJobs} {formatTimestamp} onCancelJob={onCancelSourceJob} />
  {/if}
```

- [ ] **Step 5: Wire live video props in `ReportSourceSurface`**

In `src/lib/components/analysis/report-source-surface.svelte`, make sure these props are destructured from `$props()`:

```ts
    sourceJobs,
    onSyncYoutubeComments,
    onCancelSourceJob,
```

In the live YouTube video `<YoutubeTranscriptReader>` call, add:

```svelte
          sourceJobs={sourceJobs}
          onSyncComments={() => onSyncYoutubeComments(currentSource.id)}
          onCancelSourceJob={onCancelSourceJob}
```

Do not add these props to the run snapshot `<YoutubeTranscriptReader>` call.

- [ ] **Step 6: Autofix changed Svelte components**

Run Svelte autofixer for:

```text
YoutubeTranscriptReader.svelte
ReportSourceSurface.svelte
```

Expected: no blocking Svelte issues remain.

- [ ] **Step 7: Verify the task**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts
npm.cmd run check
git diff --check
```

Expected:

```text
targeted tests pass
svelte-check found 0 errors and 0 warnings
git diff --check exits 0
```

- [ ] **Step 8: Commit**

Run:

```powershell
git add src/lib/components/analysis/youtube-transcript-reader.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-redesign-safety-contract.test.ts
git commit -m "feat: restore youtube video source actions"
```

---

### Task 3: Restore YouTube Playlist Source Activity

**Files:**
- Modify: `src/lib/components/analysis/youtube-playlist-reader.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-redesign-safety-contract.test.ts`

- [ ] **Step 1: Add failing playlist activity tests**

In `src/lib/analysis-source-readers.test.ts`, extend the playlist test:

```ts
it("renders YouTube playlist source activity and cancellation", () => {
  expect(youtubePlaylistSource).toContain("sourceJobs");
  expect(youtubePlaylistSource).toContain("<YoutubeSourceActivity");
  expect(youtubePlaylistSource).toContain("onCancelSourceJob");
  expect(reportSourceSurfaceSource).toContain("sourceJobs={sourceJobs}");
  expect(reportSourceSurfaceSource).toContain("onCancelSourceJob={onCancelSourceJob}");
});
```

In `src/lib/analysis-redesign-safety-contract.test.ts`, extend the YouTube source material test:

```ts
expect(youtubePlaylistSource).toContain("<YoutubeSourceActivity");
expect(youtubePlaylistSource).toContain("onCancelSourceJob");
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: FAIL because `YoutubePlaylistReader` does not yet render `YoutubeSourceActivity`.

- [ ] **Step 3: Update `YoutubePlaylistReader` props and imports**

In `src/lib/components/analysis/youtube-playlist-reader.svelte`, add:

```svelte
  import YoutubeSourceActivity from "$lib/components/analysis/youtube-source-activity.svelte";
  import type { SourceJobRecord } from "$lib/types/sources";
```

Add props:

```svelte
    sourceJobs = [],
    onCancelSourceJob = async () => {},
```

Add prop types:

```ts
    sourceJobs?: SourceJobRecord[];
    onCancelSourceJob?: (jobId: string) => void | Promise<void>;
```

- [ ] **Step 4: Render playlist activity**

After the playlist status block and before the playlist items, add:

```svelte
    <YoutubeSourceActivity
      jobs={sourceJobs}
      {formatTimestamp}
      onCancelJob={onCancelSourceJob}
      title="Playlist activity"
    />
```

Keep existing `Sync all`, `Retry failed`, `Open video source`, `Sync this video`, and `Retry this video` behavior unchanged.

- [ ] **Step 5: Wire playlist props in `ReportSourceSurface`**

In the live YouTube playlist `<YoutubePlaylistReader>` call in `src/lib/components/analysis/report-source-surface.svelte`, add:

```svelte
          sourceJobs={sourceJobs}
          onCancelSourceJob={onCancelSourceJob}
```

- [ ] **Step 6: Autofix changed Svelte components**

Run Svelte autofixer for:

```text
YoutubePlaylistReader.svelte
ReportSourceSurface.svelte
```

Expected: no blocking Svelte issues remain.

- [ ] **Step 7: Verify the task**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-redesign-safety-contract.test.ts
npm.cmd run check
git diff --check
```

Expected:

```text
targeted tests pass
svelte-check found 0 errors and 0 warnings
git diff --check exits 0
```

- [ ] **Step 8: Commit**

Run:

```powershell
git add src/lib/components/analysis/youtube-playlist-reader.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-redesign-safety-contract.test.ts
git commit -m "feat: restore youtube playlist source activity"
```

---

### Task 4: Include Related YouTube Jobs For Selected Video Sources

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-source-readers-route.test.ts`

- [ ] **Step 1: Add the failing route wiring test**

In `src/lib/analysis-source-readers-route.test.ts`, add:

```ts
it("includes related playlist-video jobs in selected video source activity", () => {
  expect(analysisPageSource).toContain("related_source_id === source.id");
  expect(analysisPageSource).toContain("seenSourceJobIds");
  expect(analysisPageSource).toContain("right.started_at - left.started_at");
});
```

- [ ] **Step 2: Run the route test and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers-route.test.ts
```

Expected: FAIL because `currentSourceJobs()` only returns jobs keyed by `source.id`.

- [ ] **Step 3: Update `currentSourceJobs()`**

In `src/routes/analysis/+page.svelte`, replace the existing `currentSourceJobs()` function with:

```ts
  function currentSourceJobs() {
    const source = currentSource();
    if (!source) return [];

    const directJobs = sourceJobsBySource[source.id] ?? [];
    const seenSourceJobIds = new Set(directJobs.map((job) => job.job_id));
    const relatedJobs = Object.values(sourceJobsBySource)
      .flat()
      .filter((job) => {
        if (job.related_source_id !== source.id) return false;
        if (seenSourceJobIds.has(job.job_id)) return false;
        seenSourceJobIds.add(job.job_id);
        return true;
      });

    return [...directJobs, ...relatedJobs].sort((left, right) => right.started_at - left.started_at);
  }
```

- [ ] **Step 4: Verify the task**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers-route.test.ts
npm.cmd run check
git diff --check
```

Expected:

```text
analysis-source-readers-route.test.ts passes
svelte-check found 0 errors and 0 warnings
git diff --check exits 0
```

- [ ] **Step 5: Commit**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-source-readers-route.test.ts
git commit -m "fix: include related youtube source jobs"
```

---

### Task 5: Restore Telegram Topic Selector In Live Source Mode

**Files:**
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/analysis-source-readers.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-source-readers-route.test.ts`

- [ ] **Step 1: Add failing topic selector tests**

In `src/lib/analysis-source-readers.test.ts`, add:

```ts
it("renders Telegram topic filtering only in live single-source mode", () => {
  expect(reportSourceSurfaceSource).toContain('class="topic-filter"');
  expect(reportSourceSurfaceSource).toContain("showTopicSelector");
  expect(reportSourceSurfaceSource).toContain("sourceTopics");
  expect(reportSourceSurfaceSource).toContain("loadingSourceTopics");
  expect(reportSourceSurfaceSource).toContain("selectedTopicKey");
  expect(reportSourceSurfaceSource).toContain("onChangeSelectedTopicKey");
  expect(reportSourceSurfaceSource).toContain("__all_topics__");
  expect(sourceGroupReaderSource).not.toContain("topic-filter");
});
```

In `src/lib/analysis-report-canvas.test.ts`, add:

```ts
it("passes Telegram topic state into the source surface", () => {
  expect(reportCanvasSource).toContain("{sourceTopics}");
  expect(reportCanvasSource).toContain("{loadingSourceTopics}");
  expect(reportCanvasSource).toContain("{selectedTopicKey}");
  expect(reportCanvasSource).toContain("{showTopicSelector}");
  expect(reportCanvasSource).toContain("onChangeSelectedTopicKey={onChangeSelectedTopicKey}");
});
```

In `src/lib/analysis-source-readers-route.test.ts`, extend the prop wiring test:

```ts
expect(analysisPageSource).toContain("{sourceTopics}");
expect(analysisPageSource).toContain("{loadingSourceTopics}");
expect(analysisPageSource).toContain("{selectedTopicKey}");
expect(analysisPageSource).toContain("showTopicSelector={shouldShowTopicSelector()}");
expect(analysisPageSource).toContain("onChangeSelectedTopicKey={(value) => void changeSelectedTopicKey(value)}");
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers-route.test.ts
```

Expected: FAIL because `ReportSourceSurface` does not render a topic selector.

- [ ] **Step 3: Import `Select` and destructure topic props**

In `src/lib/components/analysis/report-source-surface.svelte`, add:

```svelte
  import Select from "$lib/components/ui/Select.svelte";
```

Make sure these props are destructured from `$props()`:

```ts
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    onChangeSelectedTopicKey,
```

Add this derived value and sorter near the other derived values:

```ts
  const sortedSourceTopics = $derived([...sourceTopics].sort(compareTopics));

  function compareTopics(left: SourceForumTopic, right: SourceForumTopic) {
    if (left.kind !== right.kind) {
      return left.kind === "topic" ? -1 : 1;
    }

    if (left.isDeleted !== right.isDeleted) {
      return left.isDeleted ? 1 : -1;
    }

    const titleOrder = left.title.localeCompare(right.title, undefined, {
      sensitivity: "base",
      numeric: true,
    });
    if (titleOrder !== 0) {
      return titleOrder;
    }

    return left.key.localeCompare(right.key, undefined, {
      sensitivity: "base",
      numeric: true,
    });
  }

  function changeSelectedTopic(event: Event) {
    onChangeSelectedTopicKey((event.currentTarget as HTMLSelectElement).value);
  }
```

- [ ] **Step 4: Render the live Telegram topic selector**

In the live single-source Telegram branch, immediately before `<TelegramTimelineReader`, add:

```svelte
        {#if showTopicSelector}
          <label class="topic-filter">
            <span>Topic view</span>
            <Select value={selectedTopicKey} disabled={loadingSourceTopics} onchange={changeSelectedTopic}>
              <option value="__all_topics__">All topics</option>
              {#if loadingSourceTopics && sourceTopics.length === 0}
                <option value="__loading_topics__" disabled>Loading topics...</option>
              {:else}
                {#each sortedSourceTopics as topic (topic.key)}
                  <option value={topic.key}>{topic.title} ({topic.messageCount})</option>
                {/each}
              {/if}
            </Select>
          </label>
        {/if}
```

Do not render this selector in:

- the `currentRun && sourceViewBasis === "run_snapshot"` branch;
- the `analysisScope === "source_group"` branch;
- the live YouTube video or playlist branches.

- [ ] **Step 5: Add topic selector CSS**

In the `<style>` block of `report-source-surface.svelte`, add:

```css
  .topic-filter {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-self: flex-start;
    min-width: min(18rem, 100%);
    color: var(--muted);
    font-size: 0.74rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .topic-filter :global(select) {
    min-width: 14rem;
    text-transform: none;
    letter-spacing: 0;
    font-size: 0.9rem;
    color: var(--text);
  }
```

- [ ] **Step 6: Autofix changed Svelte component**

Run Svelte autofixer for:

```text
ReportSourceSurface.svelte
```

Expected: no blocking Svelte issues remain.

- [ ] **Step 7: Verify the task**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers-route.test.ts
npm.cmd run check
git diff --check
```

Expected:

```text
targeted tests pass
svelte-check found 0 errors and 0 warnings
git diff --check exits 0
```

- [ ] **Step 8: Commit**

Run:

```powershell
git add src/lib/components/analysis/report-source-surface.svelte src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers-route.test.ts
git commit -m "feat: restore telegram topic filtering in source mode"
```

---

### Task 6: Full Verification And Runtime Smoke

**Files:**
- Modify: `docs/superpowers/plans/2026-05-13-analysis-source-mode-parity.md`

- [ ] **Step 1: Run final targeted tests**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-source-readers.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected:

```text
Test Files  4 passed
Tests       all tests in the selected files passed
```

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 3: Run the full Vitest suite**

Run:

```powershell
npm.cmd test -- --run
```

Expected:

```text
Test Files  all passed
Tests       all passed
```

- [ ] **Step 4: Run whitespace verification**

Run:

```powershell
git diff --check
```

Expected:

```text
no whitespace errors
```

On Windows, Git may print CRLF conversion warnings. Treat CRLF warnings as non-blocking only when the command exit code is 0.

- [ ] **Step 5: Runtime smoke in Tauri if the app is running**

If a Tauri app is already running with the MCP bridge, use the Tauri MCP tools:

```text
mcp__tauri__.driver_session action=start port=9223
mcp__tauri__.webview_dom_snapshot type=accessibility
```

Verify these states manually:

- live YouTube video source shows transcript reader, `Sync comments`, comments status, and source activity when jobs exist;
- live YouTube playlist source shows playlist reader, playlist actions, comments badges, and source activity when jobs exist;
- live Telegram source with forum topics shows `Topic view` and topic changes reload the timeline;
- run snapshot source mode does not show `Sync comments`, `Source activity`, or `Topic view`.

If the app is not running, record that runtime smoke was skipped because no running Tauri session was available.

- [ ] **Step 6: Update this plan with verification evidence**

Append a `## Verification Evidence` section to
`docs/superpowers/plans/2026-05-13-analysis-source-mode-parity.md`. The section
must include:

- date;
- targeted test command and exact passed file/test counts;
- `npm.cmd run check` command and exact `svelte-check` result;
- full `npm.cmd test -- --run` command and exact passed file/test counts;
- `git diff --check` command, exit status, and any CRLF-only warnings;
- Tauri MCP runtime smoke result or the exact skipped reason.

Do not commit the verification section if any required command failed.

- [ ] **Step 7: Commit verification documentation**

Run:

```powershell
git add docs/superpowers/plans/2026-05-13-analysis-source-mode-parity.md
git commit -m "docs: record source mode parity verification"
```

- [ ] **Step 8: Final status**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## main
```

Report:

- commit hashes for each task;
- targeted test counts;
- `npm.cmd run check` result;
- full test counts;
- `git diff --check` result;
- runtime smoke result or skipped reason.
