import { describe, expect, it } from "vitest";
import rawSource from "./ComboSelect.svelte?raw";

// bits-ui Command throws under jsdom ("node.querySelector is not a function"),
// unlike the plain Popover. So — like the svar grid wrappers — ComboSelect is
// verified by source assertions; the live combobox is checked visually during
// integration.
const source = rawSource.replace(/\r\n/g, "\n");

describe("ComboSelect", () => {
  it("wires a searchable popover + command combobox", () => {
    expect(source).toContain("Popover.Root");
    expect(source).toContain("Command.Root");
    expect(source).toContain("Command.Input");
    expect(source).toContain("Command.Item");
    expect(source).toContain("Command.Empty");
  });

  it("binds option value/label search and forwards selection", () => {
    expect(source).toContain("value={option.value}");
    expect(source).toContain("keywords={[option.label]}");
    expect(source).toContain("onSelect={() => pick(option)}");
    expect(source).toContain("{triggerPrefix}: {selectedLabel}");
  });
});
