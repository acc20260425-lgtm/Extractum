// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectToolbar from "./ProjectToolbar.svelte";

afterEach(cleanup);

const base = {
  title: "Беларусь: медиаполе 2025",
  periodLabel: "Весь период",
  promptLabel: "По умолчанию",
  modelLabel: "gpt-4.1",
};

describe("ProjectToolbar", () => {
  it("renders the title, selector labels, and runs on click", async () => {
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
