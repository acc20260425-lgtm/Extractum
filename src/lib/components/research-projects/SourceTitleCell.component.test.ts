// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/svelte";
import SourceTitleCell from "./SourceTitleCell.svelte";

afterEach(cleanup);

describe("SourceTitleCell", () => {
  it("renders the title, handle, and a provider-coded dot", () => {
    render(SourceTitleCell, {
      props: { row: { title: "ФинБеларусь", handle: "@finbelarus", provider: "telegram" } },
    });

    expect(screen.getByText("ФинБеларусь")).toBeTruthy();
    expect(screen.getByText("@finbelarus")).toBeTruthy();
    expect(screen.getByTestId("source-provider-dot").dataset.provider).toBe("telegram");
  });

  it("omits the handle element when there is no handle", () => {
    render(SourceTitleCell, {
      props: { row: { title: "No handle", handle: null, provider: "youtube" } },
    });

    expect(screen.getByText("No handle")).toBeTruthy();
    expect(screen.queryByTestId("source-handle")).toBeNull();
    expect(screen.getByTestId("source-provider-dot").dataset.provider).toBe("youtube");
  });
});
