import { describe, expect, it } from "vitest";
import {
  ALL_TOPICS_KEY,
  applyAnalysisRunEvent,
  applyTakeoutImportJobs,
  applyTakeoutImportRecoveryStates,
  analysisTraceRefOrigin,
  canCancelAnalysisRun,
  createEmptyLiveRunState,
  filteredAnalysisGroups,
  filteredAnalysisRuns,
  filteredAnalysisSourceCatalog,
  focusedLiveRunState,
  focusedRunChunkSummaries,
  focusedRunStreamedOutput,
  currentTopicFilter,
  formatAnalysisRunProgress,
  hasRealForumTopics,
  isRunActive,
  isRunFocused,
  liveRunPhase,
  liveRunProgress,
  mergeAnalysisTraceRefs,
  normalizeSelectedTopicKey,
  notebookLmExportProgressFromEvent,
  notebookLmExportCompleteStatus,
  notebookLmExportInitialProgress,
  notebookLmExportRequestFromForm,
  openedRunResetState,
  pruneLiveRuns,
  runActivePhase,
  runActiveProgress,
  activeAnalysisRunIds,
  activeRunSyncDecision,
  analysisReportStartCommand,
  reportLaunchDisabledReason,
  runDeletionDecision,
  runDeletedStatus,
  shouldShowTopicSelector,
  selectedAnalysisGroup,
  selectedAnalysisTemplate,
  selectedAnalysisTraceRef,
  clearSourceActionPending,
  sourceActionPending,
  sourceDeletedStatus,
  sourceDeletionDialog,
  sourceDeletionResetState,
  sourceSyncStatus,
  syncRunSnapshot,
  takeoutImportEventDecision,
  takeoutRecoveryBody,
  takeoutRecoveryFacts,
  takeoutRecoverySeverity,
  takeoutRecoveryTitle,
  takeoutRecoveryWarningExplanations,
  upsertTakeoutImportJob,
  visibleTakeoutRecoveryForSource,
} from "./analysis-state";
import type {
  AnalysisPromptTemplate,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisSourceOption,
  AnalysisTraceRef,
} from "./types/analysis";
import type {
  NotebookLmExportEvent,
  NotebookLmExportResult,
  Source,
  SourceForumTopic,
  SyncSourceResult,
  TakeoutImportJobRecord,
  TakeoutImportRecoveryState,
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
    batch_id: 100,
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

function takeoutRecovery(
  overrides: Partial<TakeoutImportRecoveryState>,
): TakeoutImportRecoveryState {
  return Object.assign({
    batch_id: 10,
    source_id: 1,
    status: "running",
    recovery_kind: "interrupted",
    completeness: "unknown",
    item_inserted_count: 0,
    item_duplicate_count: 0,
    item_skipped_count: 0,
    item_observed_count: 0,
    warning_count: 0,
    warning_codes: [],
    terminal_error: null,
    started_at: 1_700_000,
    finished_at: null,
    updated_at: 1_700_030,
  }, overrides);
}

function syncResult(overrides: Partial<SyncSourceResult> = {}): SyncSourceResult {
  return {
    inserted: 10,
    skipped: 2,
    lastMessageId: 123,
    initialSyncPolicyApplied: null,
    warnings: [],
    ...overrides,
  };
}

function topic(overrides: Partial<SourceForumTopic>): SourceForumTopic {
  return {
    kind: "topic",
    key: "topic-1",
    title: "Topic",
    messageCount: 1,
    topicId: 1,
    topMessageId: 10,
    iconColor: null,
    iconEmojiId: null,
    isClosed: false,
    isPinned: false,
    isHidden: false,
    isDeleted: false,
    sortOrder: null,
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

function notebookResult(overrides: Partial<NotebookLmExportResult> = {}): NotebookLmExportResult {
  return {
    output_dir: "C:/Exports",
    files: [
      {
        path: "C:/Exports/source.md",
        message_count: 12,
        byte_size: 1024,
        approximate_word_count: 300,
      },
    ],
    glossary_file: null,
    exported_message_count: 12,
    skipped_message_count: 2,
    warning_count: 0,
    warnings: [],
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
    youtube_url: null,
    youtube_timestamp_seconds: null,
    youtube_display_label: null,
    is_synthetic: false,
    ...overrides,
  };
}

function runSummary(overrides: Partial<AnalysisRunSummary>): AnalysisRunSummary {
  return {
    id: 1,
    run_type: "daily",
    scope_type: "single_source",
    source_id: 2,
    source_title: "Source",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source",
    period_from: 100,
    period_to: 200,
    output_language: "en",
    prompt_template_id: 3,
    prompt_template_name: "Template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "gemini",
    model: "gemini-2.5-flash",
    youtube_corpus_mode: "transcript_description",
    status: "running",
    error: null,
    has_trace_data: false,
    snapshot_state: null,
    snapshot_captured_at: null,
    snapshot_error: null,
    created_at: 100,
    completed_at: null,
    ...overrides,
  };
}

function runDetail(overrides: Partial<AnalysisRunDetail>): AnalysisRunDetail {
  return {
    ...runSummary(overrides),
    result_markdown: "saved result",
    ...overrides,
  };
}

function promptTemplate(overrides: Partial<AnalysisPromptTemplate>): AnalysisPromptTemplate {
  return {
    id: 1,
    name: "Template",
    template_kind: "report",
    body: "Body",
    version: 1,
    is_builtin: false,
    created_at: 100,
    updated_at: 100,
    ...overrides,
  };
}

function sourceRecord(overrides: Partial<Source>): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "channel",
    accountId: null,
    externalId: "external-a",
    title: "Announcements",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    telegramUsername: null,
    avatarDataUrl: null,
    ...overrides,
  };
}

function sourceGroup(overrides: Partial<AnalysisSourceGroup>): AnalysisSourceGroup {
  return {
    id: 1,
    name: "Research",
    source_type: "telegram",
    members: [],
    created_at: 100,
    updated_at: 100,
    ...overrides,
  };
}

function sourceMetric(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 1,
    account_id: null,
    source_type: "telegram",
    title: "Announcements",
    item_count: 12,
    last_synced_at: 100,
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

  it("derives focused live run details with saved run fallbacks", () => {
    const liveRun = {
      ...createEmptyLiveRunState(),
      phase: "map",
      progress: "2/5",
      chunkSummaries: [
        {
          index: 1,
          total: 1,
          message_count: 4,
          summary: "chunk",
          topics: [],
          notable_points: [],
          candidate_refs: [],
        },
      ],
      streamedOutput: "streamed result",
    };
    const liveRuns = {
      7: liveRun,
    };
    const currentRun = runDetail({
      id: 7,
      status: "completed",
      result_markdown: "saved result",
    });

    expect(activeAnalysisRunIds([runSummary({ id: 7 }), runSummary({ id: 8 })]))
      .toEqual([7, 8]);
    expect(focusedLiveRunState(liveRuns, 7)).toBe(liveRun);
    expect(focusedLiveRunState(liveRuns, null)).toBeNull();
    expect(runActivePhase(liveRun, currentRun)).toBe("map");
    expect(runActivePhase(null, currentRun)).toBe("completed");
    expect(runActiveProgress(liveRun)).toBe("2/5");
    expect(focusedRunChunkSummaries(liveRun)).toEqual(liveRun.chunkSummaries);
    expect(focusedRunStreamedOutput(liveRun, currentRun)).toBe("streamed result");
    expect(focusedRunStreamedOutput(null, currentRun)).toBe("saved result");
    expect(focusedRunStreamedOutput(null, runDetail({ result_markdown: null }))).toBe("");
  });

  it("derives focused and active run flags from selected ids", () => {
    const liveRuns = {
      7: { ...createEmptyLiveRunState(), phase: "map", progress: "2/5" },
    };
    const currentRun = runDetail({ id: 8 });

    expect(liveRunPhase(liveRuns, 7)).toBe("map");
    expect(liveRunPhase(liveRuns, 9)).toBe("");
    expect(liveRunProgress(liveRuns, 7)).toBe("2/5");
    expect(liveRunProgress(liveRuns, 9)).toBe("");
    expect(isRunFocused(7, 7, currentRun)).toBe(true);
    expect(isRunFocused(8, null, currentRun)).toBe(true);
    expect(isRunFocused(9, null, currentRun)).toBe(false);
    expect(isRunActive(7, [7, 8])).toBe(true);
    expect(isRunActive(null, [7, 8])).toBe(false);
    expect(canCancelAnalysisRun(8, [7, 8])).toBe(true);
    expect(canCancelAnalysisRun(9, [7, 8])).toBe(false);
  });

  it("decides how active run snapshots should update route state", () => {
    const summaries = [
      runSummary({ id: 7, status: "running" }),
      runSummary({ id: 8, status: "queued" }),
    ];

    expect(activeRunSyncDecision(summaries, 7, null)).toEqual({
      activeRunIds: [7, 8],
      preserveRunId: null,
      runSnapshots: [
        { runId: 7, status: "running" },
        { runId: 8, status: "queued" },
      ],
      nextActiveRunId: 7,
      runToOpen: null,
    });

    expect(activeRunSyncDecision(summaries, 99, null)).toEqual({
      activeRunIds: [7, 8],
      preserveRunId: null,
      runSnapshots: [
        { runId: 7, status: "running" },
        { runId: 8, status: "queued" },
      ],
      nextActiveRunId: null,
      runToOpen: 7,
    });

    expect(activeRunSyncDecision([], 99, null)).toEqual({
      activeRunIds: [],
      preserveRunId: null,
      runSnapshots: [],
      nextActiveRunId: null,
      runToOpen: null,
    });

    expect(activeRunSyncDecision(summaries, 99, 42)).toEqual({
      activeRunIds: [7, 8],
      preserveRunId: 42,
      runSnapshots: [
        { runId: 7, status: "running" },
        { runId: 8, status: "queued" },
      ],
      nextActiveRunId: 99,
      runToOpen: null,
    });
  });

  it("filters analysis runs by selected status", () => {
    const completed = runSummary({ id: 1, status: "completed" });
    const failed = runSummary({ id: 2, status: "failed" });
    const running = runSummary({ id: 3, status: "running" });
    const runs = [completed, failed, running];

    expect(filteredAnalysisRuns(runs, "all")).toBe(runs);
    expect(filteredAnalysisRuns(runs, "completed")).toEqual([completed]);
    expect(filteredAnalysisRuns(runs, "failed")).toEqual([failed]);
  });

  it("filters source catalog by title, external id, or account label", () => {
    const sources = [
      sourceRecord({ id: 1, title: "Announcements", externalId: "channel-a", accountId: 10 }),
      sourceRecord({ id: 2, title: null, externalId: "market-feed", accountId: 20 }),
      sourceRecord({ id: 3, title: "Archive", externalId: "archive", accountId: null }),
    ];
    const accountLabel = (accountId: number | null) =>
      accountId === 10 ? "Primary Account" : "Backup Account";

    expect(filteredAnalysisSourceCatalog(sources, "", accountLabel)).toBe(sources);
    expect(filteredAnalysisSourceCatalog(sources, "announce", accountLabel)).toEqual([sources[0]]);
    expect(filteredAnalysisSourceCatalog(sources, "MARKET", accountLabel)).toEqual([sources[1]]);
    expect(filteredAnalysisSourceCatalog(sources, "primary", accountLabel)).toEqual([sources[0]]);
  });

  it("filters source groups by trimmed case-insensitive query", () => {
    const groups = [
      sourceGroup({ id: 1, name: "Research" }),
      sourceGroup({ id: 2, name: "Daily Briefing" }),
    ];

    expect(filteredAnalysisGroups(groups, " ")).toBe(groups);
    expect(filteredAnalysisGroups(groups, "brief")).toEqual([groups[1]]);
    expect(filteredAnalysisGroups(groups, "RESEARCH")).toEqual([groups[0]]);
  });

  it("selects templates, groups, and trace refs from route ids", () => {
    const templates = [
      promptTemplate({ id: 1, name: "Daily" }),
      promptTemplate({ id: 2, name: "Weekly" }),
    ];
    const groups = [
      sourceGroup({ id: 3, name: "Research" }),
      sourceGroup({ id: 4, name: "Ops" }),
    ];
    const refs = [
      traceRef({ ref: "ref-a" }),
      traceRef({ ref: "ref-b" }),
    ];

    expect(selectedAnalysisTemplate("2", templates)).toBe(templates[1]);
    expect(selectedAnalysisTemplate("", templates)).toBeNull();
    expect(selectedAnalysisTemplate("missing", templates)).toBeNull();
    expect(selectedAnalysisGroup("4", groups)).toBe(groups[1]);
    expect(selectedAnalysisGroup("", groups)).toBeNull();
    expect(selectedAnalysisGroup("missing", groups)).toBeNull();
    expect(selectedAnalysisTraceRef("ref-b", refs)).toBe(refs[1]);
    expect(selectedAnalysisTraceRef(null, refs)).toBeNull();
    expect(selectedAnalysisTraceRef("missing", refs)).toBeNull();
  });

  it("returns report start validation status before building a command", () => {
    const base = {
      analysisScope: "single_source" as const,
      selectedSourceId: "7",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "Russian",
      profileId: null,
      modelOverride: "",
      youtubeCorpusMode: "transcript_description" as const,
    };

    expect(analysisReportStartCommand({ ...base, selectedSourceId: "" })).toEqual({
      ok: false,
      status: "Select a source first.",
    });
    expect(analysisReportStartCommand({
      ...base,
      analysisScope: "source_group",
      selectedGroupId: "",
    })).toEqual({
      ok: false,
      status: "Select a source group first.",
    });
    expect(analysisReportStartCommand({ ...base, selectedTemplateId: "" })).toEqual({
      ok: false,
      status: "Select a report template first.",
    });
    expect(analysisReportStartCommand({ ...base, periodTo: "" })).toEqual({
      ok: false,
      status: "Select both dates first.",
    });
    expect(analysisReportStartCommand({
      ...base,
      periodFrom: "2026-05-04",
      periodTo: "2026-05-03",
    })).toEqual({
      ok: false,
      status: "The start date must not be after the end date.",
    });
    expect(analysisReportStartCommand({ ...base, outputLanguage: " " })).toEqual({
      ok: false,
      status: "Output language cannot be empty.",
    });
  });

  it("builds report start command args from valid report state", () => {
    expect(analysisReportStartCommand({
      analysisScope: "single_source",
      selectedSourceId: "7",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: " Russian ",
      profileId: null,
      modelOverride: " ",
      youtubeCorpusMode: "transcript_description_comments",
    })).toEqual({
      ok: true,
      command: {
        sourceId: 7,
        sourceGroupId: null,
        periodFrom: Math.floor(new Date("2026-05-01T00:00:00").getTime() / 1000),
        periodTo: Math.floor(new Date("2026-05-03T23:59:59").getTime() / 1000),
        outputLanguage: "Russian",
        promptTemplateId: 5,
        modelOverride: null,
        profileId: null,
        youtubeCorpusMode: "transcript_description_comments",
      },
    });

    expect(analysisReportStartCommand({
      analysisScope: "source_group",
      selectedSourceId: "7",
      selectedGroupId: "9",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "English",
      profileId: "research",
      modelOverride: "gemini-2.5-pro",
      youtubeCorpusMode: "transcript_only",
    })).toMatchObject({
      ok: true,
      command: {
        sourceId: null,
        sourceGroupId: 9,
        profileId: "research",
        modelOverride: "gemini-2.5-pro",
        youtubeCorpusMode: "transcript_only",
      },
    });
  });

  it("blocks report launch when LLM profile or source runtime is unusable", () => {
    const source = sourceRecord({ id: 7, title: "Announcements" });
    const base = {
      analysisScope: "single_source" as const,
      selectedSourceId: "7",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "Russian",
      profileId: null,
      modelOverride: "",
      youtubeCorpusMode: "transcript_description" as const,
      llmProfiles: [
        {
          profile_id: "default",
          api_key_configured: true,
        },
      ],
      activeLlmProfile: "default",
      currentSource: source,
      currentSourceMetric: sourceMetric({ id: 7, item_count: 12 }),
      currentGroup: null,
      sourceCatalog: [source],
      sourceSyncDisabledReason: () => null,
    };

    expect(reportLaunchDisabledReason({ ...base, llmProfiles: [] })).toBe(
      "Set up an LLM profile in Settings before running reports.",
    );
    expect(reportLaunchDisabledReason({
      ...base,
      llmProfiles: [{ ...base.llmProfiles[0], api_key_configured: false }],
    })).toBe('Add an API key for LLM profile "default" in Settings before running reports.');
    expect(reportLaunchDisabledReason({
      ...base,
      sourceSyncDisabledReason: () => "Initialize this account before syncing.",
    })).toBe("Initialize this account before syncing.");
    expect(reportLaunchDisabledReason(base)).toBeNull();
  });

  it("blocks report launch when the selected scope has no synced context", () => {
    const source = sourceRecord({ id: 7, title: "Announcements" });
    const base = {
      analysisScope: "single_source" as const,
      selectedSourceId: "7",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "Russian",
      profileId: null,
      modelOverride: "",
      youtubeCorpusMode: "transcript_description" as const,
      llmProfiles: [
        {
          profile_id: "default",
          api_key_configured: true,
        },
      ],
      activeLlmProfile: "default",
      currentSource: source,
      currentSourceMetric: sourceMetric({ id: 7, item_count: 0 }),
      currentGroup: null,
      sourceCatalog: [source],
      sourceSyncDisabledReason: () => null,
    };

    expect(reportLaunchDisabledReason(base)).toBe("Sync this source before running a report.");
    expect(reportLaunchDisabledReason({ ...base, currentSourceMetric: null }))
      .toBe("Sync this source before running a report.");
    expect(reportLaunchDisabledReason({
      ...base,
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: "3",
      currentSource: null,
      currentSourceMetric: null,
      currentGroup: sourceGroup({ id: 3, members: [] }),
    })).toBe("Add synced sources to this group before running a report.");
    expect(reportLaunchDisabledReason({
      ...base,
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: "3",
      currentSource: null,
      currentSourceMetric: null,
      currentGroup: sourceGroup({
        id: 3,
        members: [
          { source_id: 7, source_title: "Announcements", item_count: 0 },
        ],
      }),
    })).toBe("Add synced sources to this group before running a report.");
  });

  it("blocks report launch when a source group member has an unusable runtime", () => {
    const available = sourceRecord({ id: 10, title: "Ready source" });
    const unavailable = sourceRecord({ id: 11, title: "Offline source" });
    const group = sourceGroup({
      id: 3,
      name: "Research",
      members: [
        { source_id: 10, source_title: "Ready source", item_count: 4 },
        { source_id: 11, source_title: "Offline source", item_count: 0 },
      ],
    });

    expect(reportLaunchDisabledReason({
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: "3",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "Russian",
      profileId: null,
      modelOverride: "",
      youtubeCorpusMode: "transcript_description",
      llmProfiles: [
        {
          profile_id: "default",
          api_key_configured: true,
        },
      ],
      activeLlmProfile: "default",
      currentSource: null,
      currentSourceMetric: null,
      currentGroup: group,
      sourceCatalog: [available, unavailable],
      sourceSyncDisabledReason: (candidate) =>
        candidate.id === 11 ? "Sign in to this account again before syncing." : null,
    })).toBe('Offline source: Sign in to this account again before syncing.');
  });

  it("builds reset state only when the cleared run is currently opened", () => {
    const firstRun = runDetail({ id: 5 });
    const secondRun = runDetail({ id: 6 });
    const liveRuns = {
      5: createEmptyLiveRunState(),
      6: { ...createEmptyLiveRunState(), phase: "running" },
    };

    expect(openedRunResetState(4, 5, firstRun, liveRuns)).toBeNull();
    expect(openedRunResetState(5, 5, firstRun, liveRuns)).toEqual({
      activeRunId: null,
      currentRun: null,
      traceData: { refs: [] },
      savedTraceRefs: [],
      resolvedTraceRefs: [],
      selectedTraceRef: null,
      chatMessages: [],
      chatQuestion: "",
      chatting: false,
      activeChatRequestId: null,
      activeChatRunId: null,
      liveRuns: { 6: liveRuns[6] },
    });
    expect(openedRunResetState(6, null, secondRun, liveRuns)?.liveRuns).toEqual({
      5: liveRuns[5],
    });
    expect(liveRuns).toHaveProperty("6");
  });

  it("blocks deleting active runs and builds saved run deletion dialog", () => {
    expect(runDeletionDecision(runSummary({ id: 7, status: "running" }))).toEqual({
      ok: false,
      status: "Cancel or wait for this run before deleting it.",
    });
    expect(runDeletionDecision(runSummary({ id: 8, status: "queued" }))).toEqual({
      ok: false,
      status: "Cancel or wait for this run before deleting it.",
    });

    expect(runDeletionDecision(runSummary({
      id: 9,
      status: "completed",
      scope_label: "Announcements",
    }))).toEqual({
      ok: true,
      dialog: {
        title: "Delete saved run?",
        message:
          'The saved report for "Announcements" and its follow-up chat history will be removed from this device.',
        confirmLabel: "Delete",
        cancelLabel: "Cancel",
        tone: "danger",
      },
    });
  });

  it("formats saved run deletion status", () => {
    expect(runDeletedStatus(runSummary({ id: 12 }))).toBe("Saved run 12 deleted.");
  });

  it("builds source deletion dialog and status text from the display name", () => {
    const titled = sourceRecord({ title: "Announcements", externalId: "channel-1" });
    const untitled = sourceRecord({ title: null, externalId: "channel-2" });

    expect(sourceDeletionDialog(titled)).toEqual({
      title: "Delete source?",
      message:
        'The source "Announcements" and its synced local items will be removed from Extractum.\n\n' +
        "Saved analysis snapshots remain available as frozen run artifacts.",
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    expect(sourceDeletedStatus(titled)).toBe('Source "Announcements" deleted.');
    expect(sourceDeletedStatus(untitled)).toBe('Source "channel-2" deleted.');
  });

  it("builds reset state only when deleting the selected source", () => {
    expect(sourceDeletionResetState(3, "4")).toBeNull();
    expect(sourceDeletionResetState(3, "3")).toEqual({
      sourceItems: [],
      currentRun: null,
      activeRunId: null,
      traceData: { refs: [] },
      savedTraceRefs: [],
      resolvedTraceRefs: [],
      selectedTraceRef: null,
      chatMessages: [],
      chatQuestion: "",
      chatting: false,
      activeChatRequestId: null,
      activeChatRunId: null,
    });
  });

  it("updates source action pending maps without mutating current state", () => {
    const current = { 1: true, 2: true };

    expect(sourceActionPending(current, 3)).toEqual({ 1: true, 2: true, 3: true });
    expect(clearSourceActionPending(current, 1)).toEqual({ 2: true });
    expect(current).toEqual({ 1: true, 2: true });
  });

  it("formats source sync completion status from sync results", () => {
    expect(sourceSyncStatus(syncResult())).toBe(
      "Sync complete: inserted 10, skipped 2.",
    );

    expect(sourceSyncStatus(syncResult({
      initialSyncPolicyApplied: "recent_days:30",
      warnings: ["one", "two"],
    }))).toBe(
      "Sync complete: inserted 10, skipped 2. First sync policy applied: recent_days:30. Warnings: one two",
    );
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

  it("maps takeout recovery states by source id", () => {
    const first = takeoutRecovery({ source_id: 1, batch_id: 11 });
    const second = takeoutRecovery({ source_id: 2, batch_id: 12, recovery_kind: "failed" });

    expect(applyTakeoutImportRecoveryStates([first, second])).toEqual({
      1: first,
      2: second,
    });
  });

  it("hides durable recovery while an active takeout job exists for the source", () => {
    const recovery = takeoutRecovery({ source_id: 1 });
    const active = takeoutJob({ source_id: 1, status: "running" });
    const terminal = takeoutJob({ source_id: 1, status: "completed" });

    expect(visibleTakeoutRecoveryForSource(1, { 1: active }, { 1: recovery })).toBeNull();
    expect(visibleTakeoutRecoveryForSource(1, { 1: terminal }, { 1: recovery })).toBe(recovery);
    expect(visibleTakeoutRecoveryForSource(2, { 1: active }, { 1: recovery })).toBeNull();
  });

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

  it("formats takeout recovery facts and zero-count attempts", () => {
    expect(takeoutRecoveryFacts(takeoutRecovery({}))).toEqual([
      "No items were written in this attempt.",
    ]);
    expect(takeoutRecoveryFacts(takeoutRecovery({
      item_inserted_count: 2,
      item_duplicate_count: 3,
      item_skipped_count: 1,
      item_observed_count: 6,
      warning_count: 2,
    }))).toEqual([
      "2 inserted",
      "3 duplicates",
      "1 skipped",
      "6 observed",
      "2 warnings",
    ]);
  });

  it("decides route effects for Takeout import events", () => {
    expect(takeoutImportEventDecision(takeoutJob({
      source_id: 3,
      status: "completed",
      inserted: 12,
      skipped: 4,
    }), "3")).toEqual({
      status: "Takeout import complete: inserted 12, skipped 4.",
      reloadWorkspace: true,
      reloadSelectedSourceId: 3,
    });

    expect(takeoutImportEventDecision(takeoutJob({
      source_id: 3,
      status: "completed",
      inserted: 12,
      skipped: 4,
    }), "9")).toEqual({
      status: "Takeout import complete: inserted 12, skipped 4.",
      reloadWorkspace: true,
      reloadSelectedSourceId: null,
    });

    expect(takeoutImportEventDecision(takeoutJob({
      status: "failed",
      error: "bad export",
    }), "")).toEqual({
      status: "Takeout import failed: bad export",
      reloadWorkspace: false,
      reloadSelectedSourceId: null,
    });

    expect(takeoutImportEventDecision(takeoutJob({
      status: "cancelled",
      message: "Stopped by user.",
    }), "")).toEqual({
      status: "Stopped by user.",
      reloadWorkspace: false,
      reloadSelectedSourceId: null,
    });

    expect(takeoutImportEventDecision(takeoutJob({
      source_id: 3,
      status: "running",
      message: "Importing...",
    }), "3")).toEqual({
      status: "Importing...",
      reloadWorkspace: false,
      reloadSelectedSourceId: null,
    });

    expect(takeoutImportEventDecision(takeoutJob({
      source_id: 3,
      status: "running",
      message: "Importing...",
    }), "9")).toEqual({
      status: null,
      reloadWorkspace: false,
      reloadSelectedSourceId: null,
    });
  });

  it("normalizes selected topic keys only when real forum topics exist", () => {
    const topics = [
      topic({ kind: "uncategorized", key: "uncategorized", topicId: null }),
      topic({ key: "topic-1", topicId: 1 }),
    ];

    expect(hasRealForumTopics(topics)).toBe(true);
    expect(normalizeSelectedTopicKey(topics, "topic-1")).toBe("topic-1");
    expect(normalizeSelectedTopicKey(topics, "missing")).toBe("__all_topics__");
    expect(normalizeSelectedTopicKey([topic({ kind: "uncategorized", topicId: null })], "topic-1"))
      .toBe("__all_topics__");
  });

  it("builds topic filters from selected topic keys", () => {
    const topics = [
      topic({ kind: "uncategorized", key: "uncategorized", topicId: null }),
      topic({ key: "topic-1", topicId: 1 }),
      topic({ key: "deleted-topic", topicId: null }),
    ];

    expect(currentTopicFilter("__all_topics__", topics)).toBeNull();
    expect(currentTopicFilter("missing", topics)).toBeNull();
    expect(currentTopicFilter("topic-1", topics)).toEqual({
      kind: "topic",
      topicId: 1,
    });
    expect(currentTopicFilter("uncategorized", topics)).toEqual({
      kind: "uncategorized",
    });
    expect(currentTopicFilter("deleted-topic", topics)).toEqual({
      kind: "uncategorized",
    });
  });

  it("shows topic selector only for single-source supergroup topic workflows", () => {
    const supergroup = sourceRecord({
      sourceSubtype: "supergroup",
    });
    const channel = sourceRecord({
      sourceSubtype: "channel",
    });
    const topics = [
      topic({ kind: "uncategorized", key: "uncategorized", topicId: null }),
      topic({ key: "topic-1", topicId: 1 }),
    ];

    expect(shouldShowTopicSelector(null, "single_source", false, topics)).toBe(false);
    expect(shouldShowTopicSelector(supergroup, "source_group", false, topics))
      .toBe(false);
    expect(shouldShowTopicSelector(supergroup, "single_source", true, [])).toBe(true);
    expect(shouldShowTopicSelector(channel, "single_source", true, [])).toBe(false);
    expect(shouldShowTopicSelector(supergroup, "single_source", false, topics)).toBe(true);
    expect(shouldShowTopicSelector(supergroup, "single_source", false, [
      topic({ kind: "uncategorized", topicId: null }),
    ])).toBe(false);
  });

  it("uses source capabilities for topic selector loading state", () => {
    const youtubeVideo = sourceRecord({
      sourceType: "youtube",
      sourceSubtype: "video",
      accountId: null,
      isMember: false,
    });
    const forumThread = sourceRecord({
      sourceType: "forum",
      sourceSubtype: "thread",
      accountId: null,
      isMember: false,
    });
    const supergroup = sourceRecord({
      sourceSubtype: "supergroup",
    });

    expect(shouldShowTopicSelector(youtubeVideo, "single_source", true, [])).toBe(false);
    expect(shouldShowTopicSelector(forumThread, "single_source", true, [])).toBe(true);
    expect(shouldShowTopicSelector(supergroup, "single_source", true, [])).toBe(true);
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

  it("builds NotebookLM export request state from the export form", () => {
    const request = notebookLmExportRequestFromForm("export-a", 7, {
      outputDir: " C:/Exports ",
      range: "analysis_period",
      fromDate: "2026-05-03",
      toDate: "2026-05-04",
      includeMediaPlaceholders: true,
      minMessageLength: 5,
      maxWordsPerFile: 1000,
      maxBytesPerFile: 5000,
      overwriteExisting: false,
    });

    expect(request).toEqual({
      export_id: "export-a",
      source_id: 7,
      output_dir: "C:/Exports",
      period_from: Math.floor(new Date("2026-05-03T00:00:00").getTime() / 1000),
      period_to: Math.floor(new Date("2026-05-04T23:59:59").getTime() / 1000),
      include_media_placeholders: true,
      min_message_length: 5,
      max_words_per_file: 1000,
      max_bytes_per_file: 5000,
      overwrite_existing: false,
    });

    expect(notebookLmExportRequestFromForm("export-b", 8, {
      outputDir: "C:/All",
      range: "entire_history",
      fromDate: "2026-05-03",
      toDate: "2026-05-04",
      includeMediaPlaceholders: false,
      minMessageLength: 3,
      maxWordsPerFile: 2000,
      maxBytesPerFile: 8000,
      overwriteExisting: true,
    })).toMatchObject({
      export_id: "export-b",
      source_id: 8,
      output_dir: "C:/All",
      period_from: null,
      period_to: null,
      include_media_placeholders: false,
      overwrite_existing: true,
    });
  });

  it("formats NotebookLM export initial progress and completion status", () => {
    expect(notebookLmExportInitialProgress()).toEqual({
      phase: "loading",
      message: "Preparing NotebookLM export.",
      current: null,
      total: null,
    });
    expect(notebookLmExportCompleteStatus(notebookResult())).toBe(
      "NotebookLM export complete: 1 files, 12 messages.",
    );
    expect(notebookLmExportCompleteStatus(notebookResult({
      files: [],
      exported_message_count: 0,
    }))).toBe("NotebookLM export complete: 0 files, 0 messages.");
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
