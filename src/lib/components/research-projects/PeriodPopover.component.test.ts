// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import PeriodPopover from "./PeriodPopover.svelte";
import type { PeriodPreset } from "$lib/ui/research-projects-period";

afterEach(cleanup);

const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;
const presets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: unix(2024, 3, 14), to: unix(2025, 5, 31) },
  { id: "year:2025", label: "2025", from: unix(2025, 1, 1), to: unix(2025, 5, 31) },
];

describe("PeriodPopover", () => {
  it("exposes the trigger as a v11 combobox with aria-expanded state", () => {
    const { unmount } = render(PeriodPopover, {
      props: {
        presets,
        selectedId: "all",
        triggerLabel: "Весь период",
        ariaLabel: "Период",
      },
    });
    const closedTrigger = screen.getByRole("combobox", { name: "Период" });
    expect(closedTrigger.getAttribute("aria-expanded")).toBe("false");
    unmount();

    render(PeriodPopover, {
      props: {
        presets,
        selectedId: "all",
        triggerLabel: "Весь период",
        ariaLabel: "Период",
        open: true,
      },
    });
    const openTrigger = screen.getByRole("combobox", { name: "Период" });
    expect(openTrigger.getAttribute("aria-expanded")).toBe("true");
  });

  it("renders a prefix-free trigger with a caret", () => {
    render(PeriodPopover, {
      props: { presets, selectedId: "all", triggerLabel: "Весь период" },
    });
    const trigger = document.querySelector(".period-popover__trigger");
    expect(trigger?.textContent).toContain("Весь период");
    expect(trigger?.textContent).not.toContain("Период:");
    expect(trigger?.textContent).toContain("▾");
  });

  it("opens the panel and forwards preset selection", async () => {
    const onSelect = vi.fn();
    render(PeriodPopover, {
      props: {
        presets,
        selectedId: "all",
        triggerLabel: "Весь период",
        dataRange: { from: presets[0].from, to: presets[0].to },
        open: true,
        onSelect,
      },
    });
    expect(await screen.findByText(/Данные проекта:/)).toBeTruthy();
    await fireEvent.click(screen.getByText("2025"));
    expect(onSelect).toHaveBeenCalledWith(presets[1]);
  });
});
