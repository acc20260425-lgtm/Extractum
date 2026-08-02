// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/svelte";
import SourceStatusCell from "./SourceStatusCell.svelte";

afterEach(cleanup);

describe("SourceStatusCell", () => {
  it("renders the status label and a status-coded dot", () => {
    render(SourceStatusCell, { props: { row: { syncStatus: "error", statusLabel: "error" } } });

    expect(screen.getByText("error")).toBeTruthy();
    expect(screen.getByTestId("source-status-dot").dataset.status).toBe("error");
  });

  it("reflects the unavailable status", () => {
    render(SourceStatusCell, {
      props: { row: { syncStatus: "unavailable", statusLabel: "unavailable" } },
    });

    expect(screen.getByTestId("source-status-dot").dataset.status).toBe("unavailable");
  });
});
