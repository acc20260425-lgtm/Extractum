import { describe, expect, it } from "vitest";
import {
  applyAnalysisRunEvent,
  applyTakeoutImportJobs,
  analysisTraceRefOrigin,
  createEmptyLiveRunState,
  formatAnalysisRunProgress,
  hasRealForumTopics,
  mergeAnalysisTraceRefs,
  normalizeSelectedTopicKey,
  notebookLmExportProgressFromEvent,
  pruneLiveRuns,
  syncRunSnapshot,
  upsertTakeoutImportJob,
} from "./analysis-state";
import type { AnalysisRunEvent, AnalysisTraceRef } from "./types/analysis";
import type {
  NotebookLmExportEvent,
  SourceForumTopicRecord,
  TakeoutImportJobRecord,
} from "./types/sources";

function analysisEvent(overrides: Partial<AnalysisRunEvent>): AnalysisRunEvent {
  return {
    run_id: 7,
    request_id: null,
    kind: "progress",
    phase: "map",
    queue_position: null,
    message: null,
    progress_current: null,
    progress_total: null,
    delta: null,
    chunk_summary: null,
    error: null,
    ...overrides,
  };
}

function takeoutJob(overrides: Partial<TakeoutImportJobRecord>): TakeoutImportJobRecord {
  return {
    job_id: "job-a",
    source_id: 1,
    account_id: 2,
    status: "running",
    phase: "importing_history",
    message: null,
    inserted: 0,
    skipped: 0,
    progress_current: null,
    progress_total: null,
    started_at: 100,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function topic(overrides: Partial<SourceForumTopicRecord>): SourceForumTopicRecord {
  return {
    kind: "topic",
    key: "topic-1",
    title: "Topic",
    message_count: 1,
    topic_id: 1,
    top_message_id: 10,
    icon_color: null,
    icon_emoji_id: null,
    is_closed: false,
    is_pinned: false,
    is_hidden: false,
    is_deleted: false,
    sort_order: null,
    ...overrides,
  };
}

function notebookEvent(overrides: Partial<NotebookLmExportEvent> = {}): NotebookLmExportEvent {
  return {
    export_id: "export-a",
    source_id: 1,
    kind: "progress",
    phase: "writing",
    message: "Writing files",
    progress_current: 2,
    progress_total: 5,
    file_path: null,
    error: null,
    ...overrides,
  };
}

function traceRef(overrides: Partial<AnalysisTraceRef>): AnalysisTraceRef {
  return {
    ref: "ref-a",
    item_id: 1,
    source_id: 2,
    external_id: "external-a",
    published_at: 100,
    excerpt: "excerpt",
    ...overrides,
  };
}

describe("analysis-state", () => {
  it("updates live run state from progress, deltas, and chunk summaries", () => {
    const first = applyAnalysisRunEvent(createEmptyLiveRunState(), analysisEvent({
      progress_current: 1,
      progress_total: 3,
      delta: "hello ",
      chunk_summary: {
        index: 2,
        total: 3,
        message_count: 10,
        summary: "second",
        topics: [],
        notable_points: [],
        candidate_refs: [],
      },
    }));

    const next = applyAnalysisRunEvent(first, analysisEvent({
      phase: "",
      queue_position: 4,
      delta: "world",
      chunk_summary: {
        index: 1,
        total: 3,
        message_count: 8,
        summary: "first",
        topics: [],
        notable_points: [],
        candidate_refs: [],
      },
    }));

    expect(next).toMatchObject({
      phase: "map",
      progress: "Queue 4",
      queuePosition: 4,
      streamedOutput: "hello world",
    });
    expect(next.chunkSummaries.map((chunk) => chunk.index)).toEqual([1, 2]);
  });

  it("clears progress for terminal run events while keeping streamed output", () => {
    const current = {
      ...createEmptyLiveRunState(),
      phase: "reduce",
      progress: "2/3",
      queuePosition: 1,
      streamedOutput: "partial",
    };

    const next = applyAnalysisRunEvent(current, analysisEvent({
      kind: "completed",
      phase: "",
      queue_position: null,
    }));

    expect(next).toEqual({
      ...current,
      phase: "completed",
      progress: "",
      queuePosition: null,
    });
  });

  it("syncs and prunes live run maps without mutating existing entries", () => {
    const liveRuns = {
      1: { ...createEmptyLiveRunState(), phase: "running", progress: "1/4", queuePosition: 3 },
      2: { ...createEmptyLiveRunState(), phase: "running" },
    };

    const synced = syncRunSnapshot(liveRuns, 1, "completed");
    expect(synced[1]).toMatchObject({ phase: "completed", progress: "", queuePosition: null });
    expect(liveRuns[1]).toMatchObject({ phase: "running", progress: "1/4", queuePosition: 3 });

    expect(pruneLiveRuns(synced, [2], 1)).toEqual(synced);
    expect(Object.keys(pruneLiveRuns(synced, [2]))).toEqual(["2"]);
  });

  it("formats run progress from counters, queue position, terminal events, or prior progress", () => {
    expect(formatAnalysisRunProgress(analysisEvent({
      progress_current: 2,
      progress_total: 9,
    }), "old")).toBe("2/9");
    expect(formatAnalysisRunProgress(analysisEvent({ queue_position: 6 }), "old")).toBe("Queue 6");
    expect(formatAnalysisRunProgress(analysisEvent({ kind: "failed" }), "old")).toBe("");
    expect(formatAnalysisRunProgress(analysisEvent({ phase: "map" }), "old")).toBe("old");
  });

  it("keeps the newest Takeout import job per source", () => {
    const older = takeoutJob({ job_id: "older", source_id: 1, started_at: 100 });
    const newer = takeoutJob({ job_id: "newer", source_id: 1, started_at: 200 });
    const other = takeoutJob({ job_id: "other", source_id: 2, started_at: 150 });

    expect(applyTakeoutImportJobs([newer, older, other])).toEqual({
      1: newer,
      2: other,
    });
    expect(upsertTakeoutImportJob({ 1: newer }, older)).toEqual({ 1: newer });
    expect(upsertTakeoutImportJob({ 1: older }, newer)).toEqual({ 1: newer });
  });

  it("normalizes selected topic keys only when real forum topics exist", () => {
    const topics = [
      topic({ kind: "uncategorized", key: "uncategorized", topic_id: null }),
      topic({ key: "topic-1", topic_id: 1 }),
    ];

    expect(hasRealForumTopics(topics)).toBe(true);
    expect(normalizeSelectedTopicKey(topics, "topic-1")).toBe("topic-1");
    expect(normalizeSelectedTopicKey(topics, "missing")).toBe("__all_topics__");
    expect(normalizeSelectedTopicKey([topic({ kind: "uncategorized", topic_id: null })], "topic-1"))
      .toBe("__all_topics__");
  });

  it("maps matching NotebookLM export events to progress state", () => {
    expect(notebookLmExportProgressFromEvent("export-a", notebookEvent())).toEqual({
      progress: {
        phase: "writing",
        message: "Writing files",
        current: 2,
        total: 5,
      },
      status: null,
    });

    expect(notebookLmExportProgressFromEvent("export-a", notebookEvent({
      kind: "failed",
      message: null,
      error: "disk full",
    }))).toEqual({
      progress: {
        phase: "writing",
        message: "disk full",
        current: 2,
        total: 5,
      },
      status: "NotebookLM export failed: disk full",
    });

    expect(notebookLmExportProgressFromEvent("other", notebookEvent())).toBeNull();
  });

  it("merges trace refs by ref while keeping existing duplicates and sorting by publish time", () => {
    const existing = traceRef({
      ref: "ref-b",
      item_id: 2,
      external_id: "existing-b",
      published_at: 200,
    });
    const incomingDuplicate = traceRef({
      ref: "ref-b",
      item_id: 99,
      external_id: "incoming-b",
      published_at: 50,
    });
    const incomingNew = traceRef({
      ref: "ref-a",
      item_id: 1,
      external_id: "incoming-a",
      published_at: 100,
    });

    expect(mergeAnalysisTraceRefs([existing], [incomingDuplicate, incomingNew])).toEqual([
      incomingNew,
      existing,
    ]);
    expect(mergeAnalysisTraceRefs([existing], [])).toEqual([existing]);
  });

  it("reports trace ref origin with saved refs taking priority over resolved refs", () => {
    expect(analysisTraceRefOrigin("ref-a", ["ref-a"], ["ref-a"])).toBe("saved");
    expect(analysisTraceRefOrigin("ref-b", ["ref-a"], ["ref-b"])).toBe("resolved");
    expect(analysisTraceRefOrigin("ref-c", ["ref-a"], ["ref-b"])).toBe("unknown");
  });
});
