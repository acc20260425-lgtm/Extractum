import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, expect, it } from "vitest";
import Inspector, { type InspectorSource } from "./Inspector.svelte";

afterEach(cleanup);

it("Inspector > presents source status and type", () => {
  const selected: InspectorSource = {
    title: "Belarus election briefing",
    handle: "youtube / playlist",
    statusLabel: "Synchronizing",
    syncStatus: "syncing",
    typeLabel: "YouTube playlist",
    typeDot: "#ff0033",
    materialsLabel: "95 materials",
    lastSyncLabel: "29 May 2025, 20:42",
  };

  render(Inspector, {
    open: true,
    selected,
    periodLabel: "2024-2025",
    promptLabel: "Evidence brief",
    modelLabel: "GPT-4.1",
  });

  expect(screen.getByText("Belarus election briefing")).toBeTruthy();
  const status = screen.getByText("Synchronizing");
  expect(status.getAttribute("data-status")).toBe("syncing");
  expect(screen.getByText("YouTube playlist")).toBeTruthy();
  expect(screen.getByText("95 materials")).toBeTruthy();
});
