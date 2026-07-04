// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import OptionsPanel from "./OptionsPanel.svelte";
import type { ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const options: ComboOption[] = [
  { value: "gpt-4.1", label: "GPT-4.1", mono: "gpt-4.1", dot: "#10a37f", group: "OpenAI" },
  { value: "gpt-4o", label: "GPT-4o", mono: "gpt-4o", dot: "#10a37f", group: "OpenAI" },
  {
    value: "sonnet",
    label: "Claude Sonnet 4",
    mono: "claude-sonnet-4",
    dot: "#d97757",
    group: "Anthropic",
  },
  { value: "evidence", label: "Evidence brief", description: "Сводка с цитатами" },
];

describe("OptionsPanel", () => {
  it("renders options with group headings, dots, second lines and a check on the selected one", () => {
    render(OptionsPanel, {
      props: { options, selectedValue: "gpt-4o", placeholder: "Поиск модели…" },
    });
    expect(screen.getByText("OpenAI")).toBeTruthy();
    expect(screen.getByText("Anthropic")).toBeTruthy();
    expect(screen.getByText("claude-sonnet-4")).toBeTruthy();
    expect(screen.getByText("Сводка с цитатами")).toBeTruthy();
    const selected = screen.getByRole("option", { selected: true });
    expect(selected.textContent).toContain("GPT-4o");
    expect(selected.textContent).toContain("✓");
    expect(document.querySelectorAll(".options-panel__dot")).toHaveLength(3);
  });

  it("filters by label, description and mono; shows empty state", async () => {
    render(OptionsPanel, { props: { options, placeholder: "Поиск шаблона…" } });
    const input = screen.getByPlaceholderText("Поиск шаблона…");

    await fireEvent.input(input, { target: { value: "цитатами" } });
    expect(screen.getByText("Evidence brief")).toBeTruthy();
    expect(screen.queryByText("GPT-4.1")).toBeNull();

    await fireEvent.input(input, { target: { value: "claude-sonnet" } });
    expect(screen.getByText("Claude Sonnet 4")).toBeTruthy();

    await fireEvent.input(input, { target: { value: "нет-такого" } });
    expect(screen.getByText("Ничего не найдено")).toBeTruthy();
  });

  it("forwards selection", async () => {
    const onSelect = vi.fn();
    render(OptionsPanel, { props: { options, placeholder: "Поиск…", onSelect } });
    await fireEvent.click(screen.getByText("Evidence brief"));
    expect(onSelect).toHaveBeenCalledWith(options[3]);
  });
});
