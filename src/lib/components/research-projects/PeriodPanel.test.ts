// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import PeriodPanel from "./PeriodPanel.svelte";
import type { PeriodPreset } from "$lib/ui/research-projects-period";

afterEach(cleanup);

const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;

const presets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: unix(2024, 3, 14), to: unix(2025, 5, 31) },
  { id: "year:2025", label: "2025", from: unix(2025, 1, 1), to: unix(2025, 5, 31) },
];

const dataRange = { from: unix(2024, 3, 14), to: unix(2025, 5, 31) };

describe("PeriodPanel", () => {
  it("shows the project data span and preset sub-ranges with a check on the selected one", () => {
    render(PeriodPanel, { props: { presets, selectedId: "all", dataRange } });
    expect(screen.getByText(/Данные проекта: 14\.03\.24 – 31\.05\.25/)).toBeTruthy();
    expect(screen.getByText("14.03.24 – 31.05.25")).toBeTruthy();
    expect(screen.getByText("01.01.25 – 31.05.25")).toBeTruthy();
    const selected = screen.getByRole("option", { selected: true });
    expect(selected.textContent).toContain("Весь период");
    expect(selected.textContent).toContain("✓");
  });

  it("selects a preset", async () => {
    const onSelect = vi.fn();
    render(PeriodPanel, { props: { presets, selectedId: "all", dataRange, onSelect } });
    await fireEvent.click(screen.getByText("2025"));
    expect(onSelect).toHaveBeenCalledWith(presets[1]);
  });

  it("applies a custom range as a synthetic preset (day bounds)", async () => {
    const onSelect = vi.fn();
    render(PeriodPanel, { props: { presets, selectedId: "all", dataRange, onSelect } });
    await fireEvent.input(screen.getByLabelText("Дата начала"), {
      target: { value: "2025-02-01" },
    });
    await fireEvent.input(screen.getByLabelText("Дата конца"), {
      target: { value: "2025-02-28" },
    });
    await fireEvent.click(screen.getByText("Применить диапазон"));
    const preset = onSelect.mock.calls[0][0] as PeriodPreset;
    expect(preset.id).toBe("custom");
    expect(preset.label).toBe("01.02.25–28.02.25");
    expect(preset.from).toBe(new Date("2025-02-01T00:00:00").getTime() / 1000);
    expect(preset.to).toBe(new Date("2025-02-28T00:00:00").getTime() / 1000 + 86_399);
  });

  it("disables apply on missing or inverted dates", async () => {
    render(PeriodPanel, { props: { presets, dataRange } });
    const apply = () => screen.getByText("Применить диапазон") as HTMLButtonElement;
    expect(apply().disabled).toBe(true);

    await fireEvent.input(screen.getByLabelText("Дата начала"), {
      target: { value: "2025-03-01" },
    });
    await fireEvent.input(screen.getByLabelText("Дата конца"), {
      target: { value: "2025-02-01" },
    });
    expect(apply().disabled).toBe(true);
  });
});
