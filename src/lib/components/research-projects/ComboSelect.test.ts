// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ComboSelect, { type ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const options: ComboOption[] = [
  { value: "p1", label: "Evidence brief", description: "Сводка с цитатами" },
  { value: "p2", label: "Risk monitor" },
];

describe("ComboSelect", () => {
  it("shows the selected label without a prefix and a caret on the trigger", () => {
    render(ComboSelect, {
      props: { options, selectedValue: "p1", placeholder: "Поиск шаблона…" },
    });
    const trigger = document.querySelector(".combo-select__trigger");
    expect(trigger?.textContent).toContain("Evidence brief");
    expect(trigger?.textContent).not.toContain("Промпт:");
    expect(trigger?.textContent).toContain("▾");
  });

  it("falls back to the placeholder label when nothing is selected", () => {
    render(ComboSelect, {
      props: { options, placeholder: "Поиск…", triggerFallback: "Промпт" },
    });
    expect(document.querySelector(".combo-select__trigger")?.textContent).toContain("Промпт");
  });

  it("opens the options panel and forwards selection", async () => {
    const onSelect = vi.fn();
    render(ComboSelect, {
      props: { options, selectedValue: "p1", placeholder: "Поиск шаблона…", open: true, onSelect },
    });
    await fireEvent.click(await screen.findByText("Risk monitor"));
    expect(onSelect).toHaveBeenCalledWith(options[1]);
  });
});
