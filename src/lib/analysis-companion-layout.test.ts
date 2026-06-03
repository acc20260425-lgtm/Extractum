import { describe, expect, it } from "vitest";
import rawAnalysisPageSource from "../routes/analysis/+page.svelte?raw";
import rawChunkSummariesSource from "./components/analysis/chunk-summaries.svelte?raw";
import rawRunChatTabSource from "./components/analysis/run-chat-tab.svelte?raw";
import rawRunCompanionRunsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import rawRunEvidenceTabSource from "./components/analysis/run-evidence-tab.svelte?raw";
import rawTracePanelSource from "./components/analysis/trace-panel.svelte?raw";

const analysisPageSource = normalizeLineEndings(rawAnalysisPageSource);
const chunkSummariesSource = normalizeLineEndings(rawChunkSummariesSource);
const runChatTabSource = normalizeLineEndings(rawRunChatTabSource);
const runCompanionRunsTabSource = normalizeLineEndings(rawRunCompanionRunsTabSource);
const runEvidenceTabSource = normalizeLineEndings(rawRunEvidenceTabSource);
const tracePanelSource = normalizeLineEndings(rawTracePanelSource);

function normalizeLineEndings(source: string) {
  return source.replace(/\r\n/g, "\n");
}

function cssBlock(source: string, marker: string) {
  const startIndex = source.indexOf(marker);
  expect(startIndex, `missing marker: ${marker}`).toBeGreaterThanOrEqual(0);
  const openBraceIndex = source.indexOf("{", startIndex);
  expect(openBraceIndex, `missing opening brace after ${marker}`).toBeGreaterThan(startIndex);

  let depth = 0;
  for (let index = openBraceIndex; index < source.length; index += 1) {
    const character = source[index];
    if (character === "{") {
      depth += 1;
    } else if (character === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(startIndex, index + 1);
      }
    }
  }

  throw new Error(`missing closing brace for ${marker}`);
}

const desktopCompanionColumnPattern =
  /minmax\(21rem,\s*clamp\(22rem,\s*26vw,\s*26rem\)\)/;

describe("analysis companion layout", () => {
  it("keeps the desktop companion visible while preserving narrow stacking breakpoints", () => {
    const workspaceRule = cssBlock(analysisPageSource, ".analysis-workspace");
    const mediumBreakpoint = cssBlock(analysisPageSource, "@media (max-width: 1180px)");
    const narrowBreakpoint = cssBlock(analysisPageSource, "@media (max-width: 900px)");

    expect(workspaceRule).toContain("minmax(4.25rem, 4.75rem)");
    expect(workspaceRule).toContain("minmax(0, 1fr)");
    expect(workspaceRule).toMatch(desktopCompanionColumnPattern);
    expect(workspaceRule).not.toContain("minmax(320px, 430px)");

    expect(mediumBreakpoint).toContain("@media (max-width: 1180px)");
    expect(mediumBreakpoint).toContain("grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr);");
    expect(mediumBreakpoint).toContain("grid-column: 2;");

    expect(narrowBreakpoint).toContain("@media (max-width: 900px)");
    expect(narrowBreakpoint).toContain("grid-template-columns: 1fr;");
    expect(narrowBreakpoint).toContain("grid-column: 1;");
  });

  it("uses Evidence panel width, not viewport width, for trace list/detail columns", () => {
    const evidenceRootRule = cssBlock(runEvidenceTabSource, ".run-evidence-tab");
    const traceBaseRule = cssBlock(tracePanelSource, ".trace-layout");
    const containerRule = cssBlock(tracePanelSource, "@container (min-width: 33rem)");

    expect(evidenceRootRule).toContain("container-type: inline-size;");

    expect(traceBaseRule).toContain("grid-template-columns: minmax(0, 1fr);");
    expect(tracePanelSource).not.toContain("@media (min-width: 1280px)");

    expect(containerRule).toContain(".trace-layout {");
    expect(containerRule).toContain("grid-template-columns: minmax(12rem, 0.9fr) minmax(16rem, 1.1fr);");
    expect(containerRule).toContain("align-items: start;");
    expect(containerRule).toContain(".trace-detail {");
    expect(containerRule).toContain("padding-left: 0.9rem;");
    expect(containerRule).toContain("border-left: 1px solid var(--border);");
    expect(containerRule).not.toContain("minmax(0, 0.95fr) minmax(0, 1.05fr)");
  });

  it("does not add companion-width-specific inner layouts to Chat, Chunks, or Runs", () => {
    for (const source of [runChatTabSource, chunkSummariesSource, runCompanionRunsTabSource]) {
      expect(source).not.toContain("container-type:");
      expect(source).not.toContain("@container");
      expect(source).not.toContain("analysis companion width");
    }
  });
});
