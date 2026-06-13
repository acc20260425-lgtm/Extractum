import type {
  AnalysisGroupSourceType,
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisSourceOption,
  AnalysisSourceOptionType,
  UpdateAnalysisSourceGroupInput,
} from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

export type LibrarySourceProvider = AnalysisSourceOptionType | "web" | "other";
export type ProjectStatus = "ready" | "running" | "needs_attention" | "empty";
export type LibrarySourceStatus = "active" | "needs_account" | "syncing" | "error" | "unavailable";

export type ResearchProjectBacking =
  | { kind: "source_group"; groupId: number; sourceType: AnalysisGroupSourceType }
  | { kind: "synthetic"; disabledReason: string };

export type ResearchProjectView = {
  id: string;
  title: string;
  description: string | null;
  periodLabel: string;
  sourceCount: number;
  evidenceCount: number;
  materialCount: number;
  lastRunLabel: string | null;
  status: ProjectStatus;
  backing: ResearchProjectBacking;
};

export type LibrarySourceView = {
  id: string;
  sourceId: number;
  provider: LibrarySourceProvider;
  title: string;
  subtitle: string | null;
  projectCount: number;
  lastCollectedLabel: string | null;
  localCopyLabel: string | null;
  status: LibrarySourceStatus;
  disabledReason: string | null;
  alreadyConnected: boolean;
  connectable: boolean;
};

export type ProjectSourceLinkView = {
  projectId: string;
  sourceId: string;
  provider: LibrarySourceProvider;
  title: string;
  connectionStatus: "connected" | "pending" | "failed" | "already_connected";
  filterSummary: string;
};

export type LibraryFilterState = {
  query: string;
  providers: LibrarySourceProvider[];
};

export type SourceGroupUpdateDecision =
  | {
      ok: true;
      input: UpdateAnalysisSourceGroupInput;
      connectedCount: number;
      refusedCount: number;
    }
  | { ok: false; reason: string; connectedCount: 0; refusedCount: number };

export const PROJECT_PERIOD_LABEL = "01.01.2024 - 31.05.2025";

const PROVIDER_LABELS: Record<LibrarySourceProvider, string> = {
  telegram: "Telegram",
  youtube: "YouTube",
  rss: "RSS",
  forum: "форумов",
  web: "Web",
  other: "источников этого типа",
};

function sourceProjectId(groupId: number) {
  return `source-group:${groupId}`;
}

function sourceRowId(sourceId: number) {
  return `source:${sourceId}`;
}

function materialLabel(count: number) {
  if (count === 1) return "1 материал";
  return `${count} материалов`;
}

function dateLabel(unixSeconds: number | null) {
  if (!unixSeconds) return null;
  return new Intl.DateTimeFormat("ru-RU", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixSeconds * 1000));
}

function latestRunLabel(project: AnalysisSourceGroup, runs: AnalysisRunSummary[]) {
  const run = runs
    .filter((candidate) => candidate.source_group_id === project.id)
    .sort((left, right) => right.created_at - left.created_at)[0];
  return run ? dateLabel(run.created_at) : null;
}

function projectStatus(group: AnalysisSourceGroup, runs: AnalysisRunSummary[]): ProjectStatus {
  if (
    runs.some(
      (run) =>
        run.source_group_id === group.id && (run.status === "queued" || run.status === "running"),
    )
  ) {
    return "running";
  }
  if (group.members.length === 0) return "empty";
  if (group.members.every((member) => member.item_count <= 0)) return "needs_attention";
  return "ready";
}

export function buildResearchProjectsView(
  groups: AnalysisSourceGroup[],
  runs: AnalysisRunSummary[] = [],
): ResearchProjectView[] {
  return groups.map((group) => {
    const materialCount = group.members.reduce((total, member) => total + member.item_count, 0);
    return {
      id: sourceProjectId(group.id),
      title: group.name,
      description: `${PROVIDER_LABELS[group.source_type]} проект, сохраненный через текущую модель источников.`,
      periodLabel: PROJECT_PERIOD_LABEL,
      sourceCount: group.members.length,
      evidenceCount: materialCount,
      materialCount,
      lastRunLabel: latestRunLabel(group, runs),
      status: projectStatus(group, runs),
      backing: { kind: "source_group", groupId: group.id, sourceType: group.source_type },
    };
  });
}

function groupMembership(groups: AnalysisSourceGroup[]) {
  const membership = new Map<number, Set<number>>();
  for (const group of groups) {
    for (const member of group.members) {
      const current = membership.get(member.source_id) ?? new Set<number>();
      current.add(group.id);
      membership.set(member.source_id, current);
    }
  }
  return membership;
}

function selectedGroup(groups: AnalysisSourceGroup[], selectedProjectId: string | null) {
  if (!selectedProjectId?.startsWith("source-group:")) return null;
  const groupId = Number(selectedProjectId.replace("source-group:", ""));
  return groups.find((group) => group.id === groupId) ?? null;
}

function unsupportedReason(provider: LibrarySourceProvider) {
  if (provider === "telegram" || provider === "youtube") return null;
  return `Подключение ${PROVIDER_LABELS[provider]} к проектам будет доступно после миграции библиотеки.`;
}

function providerMismatchReason(project: AnalysisSourceGroup | null, provider: LibrarySourceProvider) {
  if (!project) return "Выберите проект с сохраняемой группой источников.";
  if (provider === project.source_type) return null;
  return `Этот проект сейчас сохраняет только ${PROVIDER_LABELS[project.source_type]} источники.`;
}

function activeJobBySource(sourceJobs: SourceJobRecord[]) {
  const jobsBySource = new Map<number, SourceJobRecord>();
  for (const job of sourceJobs) {
    if (job.status !== "queued" && job.status !== "running" && job.status !== "failed") {
      continue;
    }
    const current = jobsBySource.get(job.source_id);
    if (!current || job.started_at > current.started_at) {
      jobsBySource.set(job.source_id, job);
    }
  }
  return jobsBySource;
}

function jobBlockedState(job: SourceJobRecord | undefined) {
  if (!job) return null;
  if (job.status === "queued" || job.status === "running") {
    return {
      status: "syncing" as const,
      disabledReason: "Источник сейчас синхронизируется.",
    };
  }
  if (job.status === "failed") {
    return {
      status: "error" as const,
      disabledReason: job.error
        ? `Последняя синхронизация завершилась ошибкой: ${job.error}`
        : "Последняя синхронизация завершилась ошибкой.",
    };
  }
  return null;
}

export function buildLibrarySourcesView(
  sources: AnalysisSourceOption[],
  groups: AnalysisSourceGroup[],
  selectedProjectId: string | null,
  sourceJobs: SourceJobRecord[] = [],
): LibrarySourceView[] {
  const membership = groupMembership(groups);
  const project = selectedGroup(groups, selectedProjectId);
  const connectedIds = new Set(project?.members.map((member) => member.source_id) ?? []);
  const jobsBySource = activeJobBySource(sourceJobs);

  return sources.map((source) => {
    const provider = source.source_type;
    const alreadyConnected = connectedIds.has(source.id);
    const jobState = jobBlockedState(jobsBySource.get(source.id));
    const disabledReason = jobState?.disabledReason ?? (alreadyConnected
      ? "Источник уже подключен к этому проекту."
      : unsupportedReason(provider) ?? providerMismatchReason(project, provider));
    const connectable = disabledReason === null;

    return {
      id: sourceRowId(source.id),
      sourceId: source.id,
      provider,
      title: source.title ?? `Source #${source.id}`,
      subtitle: source.account_id ? `Account #${source.account_id}` : null,
      projectCount: membership.get(source.id)?.size ?? 0,
      lastCollectedLabel: dateLabel(source.last_synced_at),
      localCopyLabel: materialLabel(source.item_count),
      status: jobState?.status ?? (connectable || alreadyConnected ? "active" : "unavailable"),
      disabledReason,
      alreadyConnected,
      connectable,
    };
  });
}

export function filterLibrarySources(
  sources: LibrarySourceView[],
  filters: LibraryFilterState,
) {
  const query = filters.query.trim().toLocaleLowerCase();
  const providers = new Set(filters.providers);
  return sources.filter((source) => {
    const matchesQuery =
      !query || `${source.title} ${source.subtitle ?? ""}`.toLocaleLowerCase().includes(query);
    const matchesProvider = providers.size === 0 || providers.has(source.provider);
    return matchesQuery && matchesProvider;
  });
}

export function connectableSelection(sources: LibrarySourceView[], selectedIds: Set<string>) {
  return sources.filter((source) => selectedIds.has(source.id) && source.connectable);
}

export function buildSourceGroupUpdateInput(
  project: ResearchProjectView | null,
  group: AnalysisSourceGroup | null,
  selectedIds: Set<string>,
  librarySources: LibrarySourceView[],
): SourceGroupUpdateDecision {
  const selected = librarySources.filter((source) => selectedIds.has(source.id));
  const connectable = selected.filter((source) => source.connectable);

  if (!project || project.backing.kind !== "source_group" || !group) {
    return {
      ok: false,
      reason: "Этот проект пока нельзя сохранить через текущую модель групп источников.",
      connectedCount: 0,
      refusedCount: selected.length,
    };
  }

  const sourceType = project.backing.sourceType;
  const allowed = connectable.filter((source) => source.provider === sourceType);
  if (allowed.length === 0) {
    return {
      ok: false,
      reason: "В выбранных строках нет источников, которые можно подключить к этому проекту.",
      connectedCount: 0,
      refusedCount: selected.length,
    };
  }

  const sourceIds = Array.from(
    new Set([
      ...group.members.map((member) => member.source_id),
      ...allowed.map((source) => source.sourceId),
    ]),
  ).sort((left, right) => left - right);

  return {
    ok: true,
    input: {
      groupId: group.id,
      name: group.name,
      sourceType,
      sourceIds,
    },
    connectedCount: allowed.length,
    refusedCount: selected.length - allowed.length,
  };
}

export function buildProjectSourceLinksView(
  projectId: string | null,
  librarySources: LibrarySourceView[],
): ProjectSourceLinkView[] {
  if (!projectId) return [];
  return librarySources
    .filter((source) => source.alreadyConnected)
    .map((source) => ({
      projectId,
      sourceId: source.id,
      provider: source.provider,
      title: source.title,
      connectionStatus: "connected",
      filterSummary: "Фильтры проекта применяются при запуске анализа.",
    }));
}
