import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import RunChatTab from "./run-chat-tab.svelte";

afterEach(cleanup);

describe("analysis companion layout", () => {
  it("does not add companion-width-specific inner layouts to Chat, Chunks, or Runs", () => {
    render(RunChatTab, {
      props: {
        currentRun: null,
        chatAvailability: {
          enabled: false,
          reason: "snapshot_unavailable",
          title: "Snapshot unavailable",
          description: "This completed run has no saved context for chat.",
        },
        loadingChat: false,
        chatMessages: [],
        chatQuestion: "",
        chatting: false,
        canCancelChat: false,
        clearingChat: false,
        selectedTraceRef: null,
        reportLines: () => [],
        onTraceRefSelect: vi.fn(),
        onAskQuestion: vi.fn(),
        onCancelChat: vi.fn(),
        onClearChat: vi.fn(),
        onChangeChatQuestion: vi.fn(),
      },
    });

    expect({
      availabilityCopies: screen.getAllByText("Snapshot unavailable").length,
      tablist: screen.queryByRole("tablist"),
      complementary: screen.queryByRole("complementary"),
    }).toEqual({ availabilityCopies: 1, tablist: null, complementary: null });
  });
});
