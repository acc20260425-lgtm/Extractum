// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectToolbar from "./ProjectToolbar.svelte";
import rawSource from "./ProjectToolbar.svelte?raw";
import type { PeriodPreset } from "$lib/ui/research-projects-period";
import type { ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const source = rawSource.replace(/\r\n/g, "\n");
const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;

const periodPresets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: unix(2024, 3, 14), to: unix(2025, 5, 31) },
  { id: "year:2025", label: "2025", from: unix(2025, 1, 1), to: unix(2025, 5, 31) },
];
const promptOptions: ComboOption[] = [{ value: "p1", label: "Evidence brief" }];
const modelOptions: ComboOption[] = [
  { value: "m1", label: "GPT-4.1", mono: "gpt-4.1", dot: "#10a37f" },
];

const base = {
  title: "Беларусь: медиаполе 2025",
  periodPresets,
  selectedPeriodId: "all",
  selectedPeriodLabel: "Весь период",
  promptOptions,
  selectedPromptValue: "p1",
  modelOptions,
  selectedModelValue: "m1",
};

describe("ProjectToolbar", () => {
  it("renders the eyebrow, title and prefix-free triggers", () => {
    render(ProjectToolbar, { props: { ...base } });

    expect(screen.getByText("Research project")).toBeTruthy();
    expect(screen.getByText("Беларусь: медиаполе 2025")).toBeTruthy();
    expect(document.querySelector(".period-popover__trigger")?.textContent).toContain(
      "Весь период",
    );
    expect(screen.queryByText(/Период:/)).toBeNull();
    expect(screen.queryByText(/Промпт:/)).toBeNull();
    expect(screen.queryByText(/Модель:/)).toBeNull();
  });

  it("runs from the wide run button (narrow duplicate exists in DOM)", async () => {
    const onRun = vi.fn();
    render(ProjectToolbar, { props: { ...base, onRun } });

    // container queries не вычисляются в jsdom — обе кнопки в DOM
    const buttons = screen.getAllByRole("button", { name: "Запустить" });
    expect(buttons.length).toBeGreaterThanOrEqual(1);
    await fireEvent.click(buttons[0]);
    expect(onRun).toHaveBeenCalledTimes(1);
  });

  it("disables both run buttons when runDisabled", () => {
    render(ProjectToolbar, { props: { ...base, runDisabled: true } });
    for (const button of screen.getAllByRole("button", { name: "Запустить" })) {
      expect((button as HTMLButtonElement).disabled).toBe(true);
    }
  });

  it("collapses selectors into «Параметры» below 600px via a container query", () => {
    expect(source).toContain("container-type: inline-size");
    expect(source).toContain("@container tb (max-width: 600px)");
    expect(source).toContain("Параметры");
    expect(source).toContain("Параметры запуска");
  });

  it("highlights open triggers via the popover data-state attribute", () => {
    expect(source).toContain('[data-state="open"]');
  });

  it("keeps only one wide selector popover expanded at a time", async () => {
    render(ProjectToolbar, {
      props: {
        ...base,
        dataRange: { from: periodPresets[0].from, to: periodPresets[0].to },
      },
    });

    const period = screen.getByRole("combobox", { name: "Период" });
    const prompt = screen.getByRole("combobox", { name: "Промпт" });
    const model = screen.getByRole("combobox", { name: "Модель" });

    expect(period.getAttribute("aria-expanded")).toBe("false");
    expect(prompt.getAttribute("aria-expanded")).toBe("false");
    expect(model.getAttribute("aria-expanded")).toBe("false");

    await fireEvent.click(period);
    expect(period.getAttribute("aria-expanded")).toBe("true");
    expect(await screen.findByText(/Данные проекта:/)).toBeTruthy();

    await fireEvent.click(prompt);
    expect(period.getAttribute("aria-expanded")).toBe("false");
    expect(prompt.getAttribute("aria-expanded")).toBe("true");
    expect(screen.queryByText(/Данные проекта:/)).toBeNull();
    expect(screen.getByPlaceholderText("Поиск шаблона…")).toBeTruthy();

    await fireEvent.click(model);
    expect(prompt.getAttribute("aria-expanded")).toBe("false");
    expect(model.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByPlaceholderText("Поиск модели…")).toBeTruthy();
  });
});
