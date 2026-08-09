import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ComponentProps } from "svelte";
import type { Source } from "$lib/types/sources";
import ReportSetupPanel from "./report-setup-panel.svelte";

afterEach(cleanup);

type Props = ComponentProps<typeof ReportSetupPanel>;

function source(): Source {
  return {
    id: 7,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 2,
    externalId: "research",
    title: "Research channel",
    lastSyncState: 4,
    lastSyncedAt: 1_700_000_000,
    isMember: true,
    isActive: true,
    createdAt: 1_699_000_000,
    telegramUsername: "research",
    avatarDataUrl: null,
    migratedHistoryStatus: "available",
    migratedHistoryDetectedAt: 1_699_500_000,
    migratedHistoryRefreshedAt: 1_700_000_000,
    migratedHistoryRowCount: 14,
    migratedHistoryImportCompleted: true,
  };
}

function props(overrides: Partial<Props> = {}): Props {
  const currentSource = source();
  const selectedTemplate = {
    id: 4,
    name: "Daily brief",
    template_kind: "report",
    body: "Summarize the evidence.",
    version: 2,
    is_builtin: false,
    created_at: 1,
    updated_at: 2,
  };
  return {
    workspaceSelection: { kind: "source", sourceId: 7 },
    currentSource,
    currentGroup: null,
    currentSourceMetric: { id: 7, account_id: 2, source_type: "telegram", title: "Research channel", item_count: 38, last_synced_at: 1_700_000_000 },
    currentScopeTitle: "Research channel",
    currentScopeSummary: "38 synced messages",
    periodFrom: "2026-08-01",
    periodTo: "2026-08-09",
    selectedTemplateId: "4",
    loadingTemplates: false,
    templates: [selectedTemplate],
    outputLanguage: "English",
    youtubeCorpusMode: "transcript_only",
    includeMigratedHistory: false,
    canIncludeMigratedHistory: true,
    llmProfiles: [{ profile_id: "research", provider: "openai", default_model: "gpt-default", api_key_configured: true, base_url: "" }],
    activeLlmProfile: "research",
    selectedLlmProfileId: "research",
    selectedLlmModel: "__profile_default__",
    customModelOverride: "",
    llmProviderModels: [{ model: "gpt-fast", name: "gpt-fast", display_name: "GPT Fast", description: "Fast", input_token_limit: 1000, output_token_limit: 500, supported_generation_methods: ["generate"] }],
    loadingLlmProviderModels: false,
    llmModelStatus: "Models loaded",
    startingReport: false,
    currentScopeHasSavedRuns: false,
    selectedRunIsActive: false,
    activeProgress: "",
    activePhase: "",
    selectedTemplate,
    syncingIds: {},
    formatTimestamp: (value) => `time:${value}`,
    formatPeriod: () => "Aug 1-Aug 9",
    phaseLabel: (value) => value,
    accountLabel: () => "Research account",
    sourceSyncDisabledReason: () => null,
    reportLaunchDisabledReason: null,
    startOfDayUnix: () => 1,
    endOfDayUnix: () => 2,
    onChangePeriodFrom: vi.fn(),
    onChangePeriodTo: vi.fn(),
    onChangeSelectedTemplateId: vi.fn(),
    onChangeOutputLanguage: vi.fn(),
    onChangeYoutubeCorpusMode: vi.fn(),
    onChangeIncludeMigratedHistory: vi.fn(),
    onChangeLlmProfile: vi.fn(),
    onChangeLlmModel: vi.fn(),
    onChangeCustomModelOverride: vi.fn(),
    onRunReport: vi.fn(),
    onSyncCurrentSource: vi.fn(),
    ...overrides,
  };
}

describe("analysis LLM run controls", () => {
  it("uses profile and model selects instead of a plain model override field", async () => {
    const onChangeLlmProfile = vi.fn();
    const onChangeLlmModel = vi.fn();
    render(ReportSetupPanel, { props: props({ onChangeLlmProfile, onChangeLlmModel }) });

    expect(screen.getByRole("combobox", { name: "LLM profile" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "research - openai/gpt-default" })).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "Model" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Profile default - gpt-default" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "GPT Fast - gpt-fast" })).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: "Custom model" })).toBeNull();
  });
});

describe("analysis redesign final route contract", () => {
  it("keeps report setup out of the primary opened-run reading surface", async () => {
    const onChangePeriodFrom = vi.fn();
    const onChangeSelectedTemplateId = vi.fn();
    const onRunReport = vi.fn();
    const onSyncCurrentSource = vi.fn();
    render(ReportSetupPanel, { props: props({ onChangePeriodFrom, onChangeSelectedTemplateId, onRunReport, onSyncCurrentSource }) });

    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Research channel" })).toBeTruthy();
    expect(screen.getAllByText("38 synced messages")).toHaveLength(2);
    expect(screen.getByText("Aug 1-Aug 9")).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "Prompt template" })).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "Output language" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Run report" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync source" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Start the first report" })).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Opened run metadata" })).toBeNull();
    expect(screen.queryByText("Run details")).toBeNull();
    await fireEvent.input(screen.getByLabelText("Period from"), { target: { value: "2026-08-02" } });
    expect(onChangePeriodFrom).toHaveBeenCalledWith("2026-08-02");
    await fireEvent.change(screen.getByRole("combobox", { name: "Prompt template" }), { target: { value: "4" } });
    expect(onChangeSelectedTemplateId).toHaveBeenCalledWith("4");
    await fireEvent.click(screen.getByRole("button", { name: "Run report" }));
    expect(onRunReport).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByRole("button", { name: "Sync source" }));
    expect(onSyncCurrentSource).toHaveBeenCalledWith(7);
  });

  it("passes migrated historical scope opt-in through report setup", async () => {
    const onChangeIncludeMigratedHistory = vi.fn();
    render(ReportSetupPanel, { props: props({ onChangeIncludeMigratedHistory }) });

    const checkbox = screen.getByRole("checkbox", { name: /Include migrated historical scope/ });
    expect(checkbox).toBeTruthy();
    expect((checkbox as HTMLInputElement).checked).toBe(false);
    await fireEvent.click(checkbox);
    expect(onChangeIncludeMigratedHistory).toHaveBeenCalledWith(true);
  });
});
