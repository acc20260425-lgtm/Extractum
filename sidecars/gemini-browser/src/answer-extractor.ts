import type { Page } from "@playwright/test";
import { answerCandidates } from "./dom-contract.js";
import type {
  GeminiBrowserAnswerCompletionReason,
  GeminiBrowserAnswerExtractionDebug,
  GeminiBrowserAnswerGrouping,
  GeminiBrowserCandidateRejectReason,
} from "./protocol.js";

export const ANSWER_POLL_INTERVAL_MS = 500;
export const ANSWER_STABLE_MS = 8_000;
export const MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE = 3;
export const MAX_ANSWER_TIMEOUT_MS = 120_000;

export interface AnswerExtractionBaseline {
  groups: Array<{
    group_id: string;
    group_order: number;
    selector: string;
    text_length: number;
    block_lengths: number[];
  }>;
  highest_group_order: number;
}

export interface AnswerCandidateSummary {
  group_id: string;
  selector: string;
  grouping: GeminiBrowserAnswerGrouping;
  text: string;
  text_length: number;
  block_lengths: number[];
  block_count: number;
  group_order: number;
  score: number;
  signature: string;
  reject_reasons: GeminiBrowserCandidateRejectReason[];
}

export interface RejectedAnswerCandidate {
  selector: string;
  text_length: number;
  reasons: GeminiBrowserCandidateRejectReason[];
}

export interface AnswerExtractionSnapshot {
  elapsed_ms: number;
  busy_visible: boolean;
  raw_candidate_count: number;
  grouped_candidates: AnswerCandidateSummary[];
  rejected_candidates: RejectedAnswerCandidate[];
  selected_candidate_id: string | null;
  selected_candidate_signature: string | null;
  selected_candidate: AnswerCandidateSummary | null;
  selection_reason: string | null;
}

export interface AnswerExtractionResult {
  text: string | null;
  selector: string | null;
  waitedMs: number;
  completionReason: GeminiBrowserAnswerCompletionReason;
  debug: GeminiBrowserAnswerExtractionDebug;
  artifact: AnswerExtractionArtifactPayload;
}

export interface AnswerExtractionArtifactPayload {
  completion_reason: GeminiBrowserAnswerCompletionReason;
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate: {
    selector: string | null;
    grouping: GeminiBrowserAnswerGrouping;
    text_length: number;
    score: number | null;
    rank: number | null;
  };
  top_candidates: Array<{
    selector: string;
    grouping: GeminiBrowserAnswerGrouping;
    text_length: number;
    block_lengths: number[];
    score: number;
  }>;
  rejected: Array<{
    selector: string;
    text_length: number;
    reasons: GeminiBrowserCandidateRejectReason[];
  }>;
}

export class AnswerExtractionError extends Error {
  constructor(
    message: string,
    readonly artifact: AnswerExtractionArtifactPayload,
    cause: unknown,
  ) {
    super(message, { cause });
  }
}

interface CaptureInput {
  prompt: string;
  baseline: AnswerExtractionBaseline;
  elapsedMs: number;
  busyVisible: boolean;
}

interface PollCounters {
  lastGrowthElapsedMs: number | null;
  candidateSignatureChangedCount: number;
  stablePollCountAfterLastCandidateChange: number;
}

interface PollOptions {
  readSnapshot: (elapsedMs: number) => Promise<AnswerExtractionSnapshot>;
  answerStableMs: number;
  answerTimeoutMs?: number;
  pollIntervalMs: number;
  minStablePollsAfterSignatureChange: number;
  isBusyVisible: () => Promise<boolean>;
  now: () => number;
  waitForTimeout: (ms: number) => Promise<void>;
}

export async function captureAnswerBaseline(
  page: Page,
  prompt: string,
): Promise<AnswerExtractionBaseline> {
  const snapshot = await captureDomSnapshot(page, {
    prompt,
    baseline: { groups: [], highest_group_order: -1 },
    elapsedMs: 0,
    busyVisible: false,
    mode: "baseline",
  });
  return {
    groups: snapshot.grouped_candidates.map((candidate) => ({
      group_id: candidate.group_id,
      group_order: candidate.group_order,
      selector: candidate.selector,
      text_length: candidate.text_length,
      block_lengths: candidate.block_lengths,
    })),
    highest_group_order: snapshot.grouped_candidates.reduce(
      (highest, candidate) => Math.max(highest, candidate.group_order),
      -1,
    ),
  };
}

export async function captureAnswerExtractionSnapshot(
  page: Page,
  input: CaptureInput,
): Promise<AnswerExtractionSnapshot> {
  return captureDomSnapshot(page, { ...input, mode: "post_submit" });
}

export async function pollAnswerUntilComplete(
  page: Page,
  options: {
    prompt: string;
    baseline: AnswerExtractionBaseline;
    isBusyVisible: () => Promise<boolean>;
    answerTimeoutMs?: number;
  },
): Promise<AnswerExtractionResult> {
  let latestArtifact: AnswerExtractionArtifactPayload | null = null;
  try {
    return await pollAnswerSnapshotsUntilComplete({
      readSnapshot: async (elapsedMs) => {
        const busyVisible = await options.isBusyVisible();
        const snapshot = await captureAnswerExtractionSnapshot(page, {
          prompt: options.prompt,
          baseline: options.baseline,
          elapsedMs,
          busyVisible,
        });
        latestArtifact = toAnswerExtractionArtifact(snapshot, snapshot.selected_candidate?.text.length ?? 0, "missing");
        return snapshot;
      },
      answerStableMs: ANSWER_STABLE_MS,
      answerTimeoutMs: options.answerTimeoutMs,
      pollIntervalMs: ANSWER_POLL_INTERVAL_MS,
      minStablePollsAfterSignatureChange: MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE,
      isBusyVisible: options.isBusyVisible,
      now: () => Date.now(),
      waitForTimeout: (ms) => page.waitForTimeout(ms),
    });
  } catch (error) {
    if (isClosedTargetError(error)) {
      throw new AnswerExtractionError(
        "Answer extraction stopped because the browser target closed.",
        latestArtifact ?? emptyAnswerExtractionArtifact("missing"),
        error,
      );
    }
    return missingResult(
      0,
      latestArtifact ?? emptyAnswerExtractionArtifact("missing"),
      buildEmptyDebug("missing"),
    );
  }
}

export async function pollAnswerSnapshotsUntilComplete(options: PollOptions): Promise<AnswerExtractionResult> {
  const startedAt = options.now();
  const timeoutMs = options.answerTimeoutMs ?? MAX_ANSWER_TIMEOUT_MS;
  let lastSignature: string | null = null;
  let lastSignatureChangeAt = 0;
  let latestSnapshot: AnswerExtractionSnapshot | null = null;
  let latestTextSnapshot: AnswerExtractionSnapshot | null = null;
  let latestText: string | null = null;
  const counters: PollCounters = {
    lastGrowthElapsedMs: null,
    candidateSignatureChangedCount: 0,
    stablePollCountAfterLastCandidateChange: 0,
  };

  while (true) {
    const elapsedMs = options.now() - startedAt;
    const snapshot = await options.readSnapshot(elapsedMs);
    latestSnapshot = snapshot;

    const signature = snapshot.selected_candidate_signature;
    if (signature && signature !== lastSignature) {
      lastSignature = signature;
      lastSignatureChangeAt = elapsedMs;
      counters.lastGrowthElapsedMs = elapsedMs;
      counters.candidateSignatureChangedCount += 1;
      counters.stablePollCountAfterLastCandidateChange = 0;
    } else if (signature) {
      counters.stablePollCountAfterLastCandidateChange += 1;
    }

    if (snapshot.selected_candidate?.text) {
      latestTextSnapshot = snapshot;
      latestText = snapshot.selected_candidate.text;
    }

    const busyVisible = snapshot.busy_visible || (await options.isBusyVisible());
    const quietForMs = elapsedMs - lastSignatureChangeAt;
    const stableEnough =
      Boolean(snapshot.selected_candidate?.text) &&
      !busyVisible &&
      quietForMs >= options.answerStableMs &&
      counters.stablePollCountAfterLastCandidateChange >= options.minStablePollsAfterSignatureChange;
    if (stableEnough && snapshot.selected_candidate) {
      return resultFromSnapshot(
        snapshot,
        "stable",
        snapshot.selected_candidate.text.length,
        elapsedMs,
        {
          ...counters,
          stablePollCountAfterLastCandidateChange: options.minStablePollsAfterSignatureChange,
        },
      );
    }

    if (elapsedMs >= timeoutMs) {
      if (latestTextSnapshot && latestText) {
        return resultFromSnapshot(
          latestTextSnapshot,
          "timeout_latest",
          latestText.length,
          elapsedMs,
          counters,
        );
      }
      const fallback = latestSnapshot ?? emptySnapshot(elapsedMs);
      return missingResult(
        elapsedMs,
        toAnswerExtractionArtifact(fallback, 0, "missing"),
        buildExtractionDebug(fallback, 0, "missing", counters),
      );
    }

    await options.waitForTimeout(options.pollIntervalMs);
  }
}

function resultFromSnapshot(
  snapshot: AnswerExtractionSnapshot,
  completionReason: GeminiBrowserAnswerCompletionReason,
  returnedTextLength: number,
  waitedMs: number,
  counters: PollCounters,
): AnswerExtractionResult {
  return {
    text: snapshot.selected_candidate?.text ?? null,
    selector: snapshot.selected_candidate?.selector ?? null,
    waitedMs,
    completionReason,
    debug: buildExtractionDebug(snapshot, returnedTextLength, completionReason, counters),
    artifact: toAnswerExtractionArtifact(snapshot, returnedTextLength, completionReason),
  };
}

function missingResult(
  waitedMs: number,
  artifact: AnswerExtractionArtifactPayload,
  debug: GeminiBrowserAnswerExtractionDebug,
): AnswerExtractionResult {
  return {
    text: null,
    selector: null,
    waitedMs,
    completionReason: "missing",
    debug,
    artifact,
  };
}

function buildExtractionDebug(
  snapshot: AnswerExtractionSnapshot,
  returnedTextLength: number,
  completionReason: GeminiBrowserAnswerCompletionReason,
  counters: PollCounters,
): GeminiBrowserAnswerExtractionDebug {
  const candidates = [...snapshot.grouped_candidates].sort((a, b) => b.score - a.score || b.text_length - a.text_length);
  const selected = snapshot.selected_candidate;
  const selectedRank = selected ? candidates.findIndex((candidate) => candidate.group_id === selected.group_id) + 1 : null;
  const selectedLength = selected?.text_length ?? 0;
  const largerRejected = snapshot.rejected_candidates.filter((candidate) => candidate.text_length > selectedLength);
  const largerValid = snapshot.grouped_candidates.some((candidate) => candidate.text_length > selectedLength);
  return {
    raw_candidate_count: snapshot.raw_candidate_count,
    grouped_candidate_count: snapshot.grouped_candidates.length,
    selected_candidate_length: selectedLength,
    returned_text_length: returnedTextLength,
    selected_grouping: selected?.grouping ?? "unknown",
    selected_candidate_rank: selectedRank && selectedRank > 0 ? selectedRank : null,
    selected_score: selected?.score ?? null,
    largest_candidate_length: Math.max(0, ...snapshot.grouped_candidates.map((candidate) => candidate.text_length)),
    larger_valid_candidate_available: Boolean(selected) && largerValid,
    larger_rejected_candidate_count: largerRejected.length,
    larger_rejected_reasons: uniqueRejectReasons(largerRejected),
    top_candidate_lengths: candidates.slice(0, 5).map((candidate) => candidate.text_length),
    busy_visible_at_completion: snapshot.busy_visible,
    last_growth_elapsed_ms: counters.lastGrowthElapsedMs,
    candidate_signature_changed_count: counters.candidateSignatureChangedCount,
    stable_poll_count_after_last_candidate_change: counters.stablePollCountAfterLastCandidateChange,
  };
}

function toAnswerExtractionArtifact(
  snapshot: AnswerExtractionSnapshot,
  _returnedTextLength: number,
  completionReason: GeminiBrowserAnswerCompletionReason,
): AnswerExtractionArtifactPayload {
  const candidates = [...snapshot.grouped_candidates].sort((a, b) => b.score - a.score || b.text_length - a.text_length);
  const selected = snapshot.selected_candidate;
  const selectedRank = selected ? candidates.findIndex((candidate) => candidate.group_id === selected.group_id) + 1 : null;
  return {
    completion_reason: completionReason,
    raw_candidate_count: snapshot.raw_candidate_count,
    grouped_candidate_count: snapshot.grouped_candidates.length,
    selected_candidate: {
      selector: selected?.selector ?? null,
      grouping: selected?.grouping ?? "unknown",
      text_length: selected?.text_length ?? 0,
      score: selected?.score ?? null,
      rank: selectedRank && selectedRank > 0 ? selectedRank : null,
    },
    top_candidates: candidates.slice(0, 5).map((candidate) => ({
      selector: candidate.selector,
      grouping: candidate.grouping,
      text_length: candidate.text_length,
      block_lengths: candidate.block_lengths,
      score: candidate.score,
    })),
    rejected: snapshot.rejected_candidates.slice(0, 10).map((candidate) => ({
      selector: candidate.selector,
      text_length: candidate.text_length,
      reasons: candidate.reasons,
    })),
  };
}

function emptyAnswerExtractionArtifact(
  completionReason: GeminiBrowserAnswerCompletionReason,
): AnswerExtractionArtifactPayload {
  return {
    completion_reason: completionReason,
    raw_candidate_count: 0,
    grouped_candidate_count: 0,
    selected_candidate: {
      selector: null,
      grouping: "unknown",
      text_length: 0,
      score: null,
      rank: null,
    },
    top_candidates: [],
    rejected: [],
  };
}

function buildEmptyDebug(completionReason: GeminiBrowserAnswerCompletionReason): GeminiBrowserAnswerExtractionDebug {
  return buildExtractionDebug(emptySnapshot(0), 0, completionReason, {
    lastGrowthElapsedMs: null,
    candidateSignatureChangedCount: 0,
    stablePollCountAfterLastCandidateChange: 0,
  });
}

function emptySnapshot(elapsedMs: number): AnswerExtractionSnapshot {
  return {
    elapsed_ms: elapsedMs,
    busy_visible: false,
    raw_candidate_count: 0,
    grouped_candidates: [],
    rejected_candidates: [],
    selected_candidate_id: null,
    selected_candidate_signature: null,
    selected_candidate: null,
    selection_reason: null,
  };
}

function uniqueRejectReasons(candidates: RejectedAnswerCandidate[]): GeminiBrowserCandidateRejectReason[] {
  return [...new Set(candidates.flatMap((candidate) => candidate.reasons))];
}

async function captureDomSnapshot(
  page: Page,
  input: CaptureInput & { mode: "baseline" | "post_submit" },
): Promise<AnswerExtractionSnapshot> {
  try {
    return await page.evaluate(
      ({ selectors, prompt, baseline, elapsedMs, busyVisible, mode }) => {
        type RejectReason =
          | "baseline"
          | "composer"
          | "prompt_container"
          | "navigation"
          | "account_or_login"
          | "controls"
          | "multi_turn"
          | "not_visible"
          | "empty"
          | "lower_score";
        type Grouping = "assistant_turn" | "single_node" | "unknown";
        interface Candidate {
          group_id: string;
          selector: string;
          grouping: Grouping;
          text: string;
          text_length: number;
          block_lengths: number[];
          block_count: number;
          group_order: number;
          score: number;
          signature: string;
          reject_reasons: RejectReason[];
        }
        interface Rejected {
          selector: string;
          text_length: number;
          reasons: RejectReason[];
        }

        const baselineIds = new Set(baseline.groups.map((group) => group.group_id));
        const grouped = new Map<string, Candidate>();
        const rejected: Rejected[] = [];
        let rawCount = 0;

        function visible(element: Element): boolean {
          const html = element as HTMLElement;
          const style = window.getComputedStyle(html);
          const rect = html.getBoundingClientRect();
          return style.display !== "none" && style.visibility !== "hidden" && rect.width >= 0 && rect.height >= 0;
        }

        function normalizedText(element: Element): string {
          const text = (element.textContent ?? "").replace(/\u00a0/g, " ");
          return text
            .split(/\n+/)
            .map((line) => line.replace(/\s+/g, " ").trim())
            .filter(Boolean)
            .join("\n");
        }

        function hasComposerControls(element: Element): boolean {
          return Boolean(
            element.querySelector("textarea, [contenteditable='true'], form, button[aria-label*='send' i], button[type='submit']"),
          );
        }

        function hasNavigationOrAccount(element: Element): boolean {
          return Boolean(
            element.closest("nav, header, footer") ||
              element.querySelector("a[href*='accounts.google'], [aria-label*='account' i], [data-testid*='account' i]"),
          );
        }

        function responseIndex(element: Element): number | null {
          const raw = element.getAttribute("data-response-index");
          if (!raw) return null;
          const parsed = Number.parseInt(raw, 10);
          return Number.isFinite(parsed) ? parsed : null;
        }

        function groupAncestor(element: Element): { element: Element; grouping: Grouping } {
          let current: Element | null = element;
          for (let depth = 0; current && depth <= 6; depth += 1) {
            const index = responseIndex(current);
            const turn = current.getAttribute("data-turn");
            const role = current.getAttribute("data-message-author-role") ?? current.getAttribute("data-role");
            if (index !== null || turn === "assistant" || role === "assistant") {
              return { element: current, grouping: "assistant_turn" };
            }
            current = current.parentElement;
          }
          const section = element.closest("section, article, li");
          if (section) {
            return { element: section, grouping: "single_node" };
          }
          return { element, grouping: "single_node" };
        }

        function textBlocks(group: Element, fallback: Element): string[] {
          const messageNodes = Array.from(group.querySelectorAll("message-content"));
          const nodes = messageNodes.length > 0 ? messageNodes : [fallback];
          return nodes.flatMap((node) => {
            const listItems = Array.from(node.querySelectorAll("li"));
            if (listItems.length > 0) {
              return listItems.map((item) => normalizedText(item)).filter(Boolean);
            }
            return [normalizedText(node)].filter(Boolean);
          });
        }

        function groupOrder(group: Element, fallbackOrder: number): number {
          const index = responseIndex(group);
          if (index !== null) return index;
          return fallbackOrder;
        }

        function groupId(group: Element, order: number): string {
          const index = responseIndex(group);
          if (index !== null) return `response:${index}`;
          const turn = group.getAttribute("data-turn");
          if (turn) return `${turn}:${order}`;
          return `group:${order}`;
        }

        selectors.forEach(({ selector, score: selectorScore }) => {
          document.querySelectorAll(selector).forEach((element, index) => {
            rawCount += 1;
            const { element: group, grouping } = groupAncestor(element);
            const reasons: RejectReason[] = [];
            const blocks = textBlocks(group, element);
            const text = blocks.join("\n");
            const order = groupOrder(group, rawCount + index);
            const id = groupId(group, order);
            if (!visible(group)) reasons.push("not_visible");
            if (!text) reasons.push("empty");
            if (hasComposerControls(group)) reasons.push("composer");
            if (hasNavigationOrAccount(group)) reasons.push("account_or_login");
            if (mode === "post_submit" && (baselineIds.has(id) || order <= baseline.highest_group_order)) {
              reasons.push("baseline");
            }
            if (prompt && text === prompt && hasComposerControls(group)) {
              reasons.push("prompt_container");
            }

            const summary = {
              selector,
              text_length: text.length,
              reasons,
            };
            if (reasons.length > 0) {
              rejected.push(summary);
              return;
            }
            const blockLengths = blocks.map((block) => block.length);
            const candidate: Candidate = {
              group_id: id,
              selector,
              grouping,
              text,
              text_length: text.length,
              block_lengths: blockLengths,
              block_count: blockLengths.length,
              group_order: order,
              score: selectorScore + text.length + (grouping === "assistant_turn" ? 50 : 0),
              signature: `${selector}|${id}|${order}|${grouping}|${blockLengths.length}|${blockLengths.join(".")}|${text.length}`,
              reject_reasons: [],
            };
            const existing = grouped.get(id);
            if (!existing || candidate.score > existing.score) {
              grouped.set(id, candidate);
            }
          });
        });

        const groupedCandidates = Array.from(grouped.values()).sort(
          (left, right) => right.score - left.score || right.text_length - left.text_length || right.group_order - left.group_order,
        );
        const selected = groupedCandidates[0] ?? null;
        return {
          elapsed_ms: elapsedMs,
          busy_visible: busyVisible,
          raw_candidate_count: rawCount,
          grouped_candidates: groupedCandidates,
          rejected_candidates: rejected,
          selected_candidate_id: selected?.group_id ?? null,
          selected_candidate_signature: selected?.signature ?? null,
          selected_candidate: selected,
          selection_reason: selected ? "highest_score" : null,
        };
      },
      {
        selectors: answerCandidates.map((candidate) => ({
          selector: candidate.selector,
          score: candidate.score,
        })),
        prompt: input.prompt,
        baseline: input.baseline,
        elapsedMs: input.elapsedMs,
        busyVisible: input.busyVisible,
        mode: input.mode,
      },
    );
  } catch (error) {
    if (isClosedTargetError(error)) {
      throw new AnswerExtractionError(
        "Answer extraction snapshot failed because the browser target closed.",
        emptyAnswerExtractionArtifact("missing"),
        error,
      );
    }
    return emptySnapshot(input.elapsedMs);
  }
}

function isClosedTargetError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return /target.*closed|page.*closed|context.*closed|browser.*closed/i.test(message);
}
