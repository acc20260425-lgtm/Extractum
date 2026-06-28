// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import PeriodPopover from "./PeriodPopover.svelte";
import type { PeriodPreset } from "$lib/ui/research-projects-period";

afterEach(cleanup);

const presets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: 1, to: 2 },
  { id: "year:2025", label: "2025", from: 1, to: 2 },
];

describe("PeriodPopover", () => {
  it("lists presets when open and selects one", async () => {
    const onSelect = vi.fn();
    render(PeriodPopover, {
      props: { presets, triggerLabel: "Весь период", open: true, onSelect },
    });

    await fireEvent.click(screen.getByText("2025"));

    expect(onSelect).toHaveBeenCalledWith(presets[1]);
  });
});
