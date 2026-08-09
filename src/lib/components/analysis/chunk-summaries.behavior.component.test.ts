import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import ChunkSummaries from "./chunk-summaries.svelte";

afterEach(cleanup);

describe("analysis companion layout", () => {
  it("does not add companion-width-specific inner layouts to Chat, Chunks, or Runs", () => {
    const view = render(ChunkSummaries, {
      props: {
        summaries: [{
          index: 1,
          total: 2,
          message_count: 8,
          summary: "The first evidence batch establishes the timeline.",
          topics: ["Timeline"],
          notable_points: ["The launch preceded the follow-up."],
          candidate_refs: ["msg:42"],
        }],
        running: true,
        framed: false,
      },
    });

    expect({
      renderedSummary: screen.getByText("The first evidence batch establishes the timeline.").textContent,
      sectionCount: view.container.querySelectorAll("section").length,
      tablist: screen.queryByRole("tablist"),
    }).toEqual({
      renderedSummary: "The first evidence batch establishes the timeline.",
      sectionCount: 1,
      tablist: null,
    });
  });
});
