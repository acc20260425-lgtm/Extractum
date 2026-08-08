import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { PeriodPreset } from "$lib/ui/research-projects-period";
import ProjectToolbar from "./ProjectToolbar.svelte";

afterEach(cleanup);
afterEach(() => vi.unstubAllGlobals());

let resizeCallback: ResizeObserverCallback;
let resizeTarget: Element;

beforeEach(() => {
  class ResizeObserverStub {
    constructor(callback: ResizeObserverCallback) {
      resizeCallback = callback;
    }
    observe(target: Element) {
      resizeTarget = target;
      this.resize(800);
    }
    resize(width: number) {
      resizeCallback(
        [{ target: resizeTarget, contentRect: { width, height: 54 } } as ResizeObserverEntry],
        this as unknown as ResizeObserver,
      );
    }
    disconnect() {}
    unobserve() {}
  }
  vi.stubGlobal("ResizeObserver", ResizeObserverStub);
});

function resizeToolbar(width: number) {
  resizeCallback(
    [{ target: resizeTarget, contentRect: { width, height: 54 } } as ResizeObserverEntry],
    {} as ResizeObserver,
  );
}

it("ProjectToolbar > collapses secondary selectors on narrow layouts", async () => {
  const periodPresets: PeriodPreset[] = [{ id: "all", label: "All evidence", from: 0, to: 86_399 }];
  render(ProjectToolbar, {
    title: "Belarus media research",
    periodPresets,
    selectedPeriodId: "all",
    selectedPeriodLabel: "All evidence",
    promptOptions: [{ value: "brief", label: "Evidence brief" }],
    selectedPromptValue: "brief",
    modelOptions: [{ value: "gpt", label: "GPT-4.1", dot: "#10a37f" }],
    selectedModelValue: "gpt",
  });

  expect(screen.getByRole("combobox", { name: "Период" })).toBeTruthy();
  expect(screen.getByRole("combobox", { name: "Промпт" })).toBeTruthy();
  expect(screen.getByRole("combobox", { name: "Модель" })).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Параметры" })).toBeNull();

  resizeToolbar(500);
  await waitFor(() => {
    expect(screen.queryByRole("combobox", { name: "Период" })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Промпт" })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Модель" })).toBeNull();
  });
  const parameters = screen.getByRole("button", { name: "Параметры" });
  await fireEvent.click(parameters);
  expect((await screen.findAllByText("All evidence")).length).toBeGreaterThan(0);
  expect(screen.getAllByText("Evidence brief").length).toBeGreaterThan(0);
  expect(screen.getAllByText("GPT-4.1").length).toBeGreaterThan(0);
});
