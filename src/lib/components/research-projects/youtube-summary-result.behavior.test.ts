import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor, within } from "@testing-library/svelte";
const api = vi.hoisted(() => ({ getPromptPackResult: vi.fn(), getPromptPackValidationFindings: vi.fn() }));
vi.mock("$lib/api/prompt-packs", () => api);
import YoutubeSummaryResultView from "./YoutubeSummaryResultView.svelte";
const result = { resultStatus: "complete", canonical: { pack_version: "1", output_language: "en", source_refs: [{ source_id: 1 }], claims: [{ claim_id: "c1", text: "A claim" }], evidence: [{ evidence_id: "e1", text: "Evidence text" }], limitations: [{ code: "limited", message: "Limited" }], warnings: [{ code: "warning", message: "Warning" }], quality_flags: [{ code: "quality", message: "Quality" }], outputs: { sections: [{ section_id: "section_summary", body: "Readable overall summary" }], pack_data: { youtube_summary: { videos: [{ title: "Evidence video", summary_text: "**Safe** summary" }] } } } } };
beforeEach(() => { api.getPromptPackResult.mockResolvedValue(result); api.getPromptPackValidationFindings.mockResolvedValue([{ severity: "warning", code: "vf", message: "Finding", createdAt: "now" }]); });
afterEach(() => { cleanup(); vi.clearAllMocks(); });
  it("renders video summary text through the safe markdown renderer only in video sections", async () => {
    render(YoutubeSummaryResultView, { runId: 91 }); await screen.findByText("Evidence video"); const videos = screen.getByRole("heading", { name: "Videos" }).parentElement!;
    expect(within(videos).getByText("Safe")).toBeTruthy(); expect(within(videos).getByText("Safe").tagName).toBe("STRONG"); expect(within(videos).queryByText("**Safe** summary")).toBeNull(); expect(screen.getByText("Readable overall summary")).toBeTruthy(); expect(screen.getByText("Readable overall summary").tagName).toBe("P"); expect(screen.getAllByText("Safe")).toHaveLength(1);
  });
  it("loads and displays canonical result structures", async () => {
    render(YoutubeSummaryResultView, { runId: 91 }); await waitFor(() => expect(api.getPromptPackResult).toHaveBeenCalledWith(91));
    expect(screen.getByText("A claim")).toBeTruthy(); expect(screen.getByText("Evidence text")).toBeTruthy(); expect(screen.getByText("Limited")).toBeTruthy(); expect(screen.getByText("Quality")).toBeTruthy();
  });
  it("renders the overall readable summary from canonical sections", async () => {
    render(YoutubeSummaryResultView, { runId: 91 }); const summary = await screen.findByText("Readable overall summary");
    expect(summary).toBeTruthy(); expect(summary.closest(".summary-box")).toBeTruthy(); expect(screen.getByRole("heading", { name: "Summary" })).toBeTruthy(); expect(summary.textContent).toBe("Readable overall summary");
  });
