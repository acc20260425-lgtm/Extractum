// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectToolbar from "./ProjectToolbar.svelte";
import type { PeriodPreset } from "$lib/ui/research-projects-period";
import type { ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const periodPresets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: 1, to: 2 },
  { id: "year:2025", label: "2025", from: 1, to: 2 },
];
const promptOptions: ComboOption[] = [{ value: "p1", label: "По умолчанию" }];
const modelOptions: ComboOption[] = [{ value: "m1", label: "gpt-4.1" }];

const base = {
  title: "Беларусь: медиаполе 2025",
  periodPresets,
  selectedPeriodId: "all",
  promptOptions,
  selectedPromptValue: "p1",
  modelOptions,
  selectedModelValue: "m1",
};

describe("ProjectToolbar", () => {
  it("renders title, selector triggers with current selections, and runs", async () => {
    const onRun = vi.fn();
    render(ProjectToolbar, { props: { ...base, onRun } });

    expect(screen.getByText("Беларусь: медиаполе 2025")).toBeTruthy();
    expect(screen.getByText("Период: Весь период")).toBeTruthy();
    expect(screen.getByText("Промпт: По умолчанию")).toBeTruthy();
    expect(screen.getByText("Модель: gpt-4.1")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Запустить анализ" }));
    expect(onRun).toHaveBeenCalledTimes(1);
  });

  it("disables the run button when runDisabled is set", () => {
    render(ProjectToolbar, { props: { ...base, runDisabled: true } });

    expect(
      screen.getByRole("button", { name: "Запустить анализ" }).hasAttribute("disabled"),
    ).toBe(true);
  });
});
