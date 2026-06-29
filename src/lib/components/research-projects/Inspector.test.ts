// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import Inspector, { type InspectorSource } from "./Inspector.svelte";

afterEach(cleanup);

const selected: InspectorSource = {
  title: "ФинБеларусь · видео",
  handle: "youtube · плейлист",
  statusLabel: "sync",
  syncStatus: "syncing",
  materialsLabel: "95",
  lastSyncLabel: "29.05.25 · 20:42",
};

const base = {
  open: true,
  selected,
  periodLabel: "01.01.24 – 31.05.25",
  promptLabel: "Evidence brief",
  modelLabel: "GPT-4.1",
};

describe("Inspector", () => {
  it("renders the selected source details and project config when open", () => {
    render(Inspector, { props: base });

    expect(screen.getByText("Инспектор источника")).toBeTruthy();
    expect(screen.getByText("ФинБеларусь · видео")).toBeTruthy();
    expect(screen.getByText("youtube · плейлист")).toBeTruthy();
    expect(screen.getByText("95")).toBeTruthy();
    expect(screen.getByText("29.05.25 · 20:42")).toBeTruthy();
    expect(screen.getByText("Evidence brief")).toBeTruthy();
    expect(screen.getByText("GPT-4.1")).toBeTruthy();
  });

  it("collapses via the toggle button", async () => {
    const onToggle = vi.fn();
    render(Inspector, { props: { ...base, onToggle } });

    await fireEvent.click(screen.getByRole("button", { name: "Свернуть" }));
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("runs footer sync and disconnect actions for the selected source", async () => {
    const onSync = vi.fn();
    const onDisconnect = vi.fn();
    render(Inspector, { props: { ...base, onSync, onDisconnect } });

    await fireEvent.click(screen.getByRole("button", { name: "Синхронизировать" }));
    expect(onSync).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole("button", { name: "Отключить источник" }));
    expect(onDisconnect).toHaveBeenCalledTimes(1);
  });

  it("renders a collapsed rail with an expand affordance when closed", async () => {
    const onToggle = vi.fn();
    render(Inspector, { props: { ...base, open: false, onToggle } });

    expect(screen.queryByText("Конфигурация проекта")).toBeNull();
    expect(screen.getByText("Инспектор")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Развернуть инспектор" }));
    expect(onToggle).toHaveBeenCalledTimes(1);
  });
});
