// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import Inspector, { type InspectorSource } from "./Inspector.svelte";
import rawSource from "./Inspector.svelte?raw";

afterEach(cleanup);

const source = rawSource.replace(/\r\n/g, "\n");

const selected: InspectorSource = {
  title: "ФинБеларусь · видео",
  handle: "youtube · плейлист",
  statusLabel: "sync",
  syncStatus: "syncing",
  typeLabel: "youtube",
  typeDot: "var(--extractum-provider-youtube)",
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
    expect(screen.getByText("Тип")).toBeTruthy();
    expect(screen.getByText("youtube")).toBeTruthy();
    expect(screen.getByText("95")).toBeTruthy();
    expect(screen.getByText("29.05.25 · 20:42")).toBeTruthy();
    expect(screen.getByText("Evidence brief")).toBeTruthy();
    expect(screen.getByText("GPT-4.1")).toBeTruthy();
  });

  it("renders the v11 status badge and source type dot", () => {
    render(Inspector, { props: base });

    const badge = screen.getByText("sync").closest(".inspector__status");
    expect(badge?.querySelector(".inspector__status-dot")).toBeTruthy();
    expect(source).toContain("color-mix(in srgb, var(--extractum-primary) 12%, transparent)");
    expect(source).toContain("style:background={selected.typeDot}");
  });

  it("collapses via the toggle button", async () => {
    const onToggle = vi.fn();
    render(Inspector, { props: { ...base, onToggle } });

    await fireEvent.click(screen.getByRole("button", { name: "Свернуть" }));
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("keeps the inspector toggle visibly framed with an icon", () => {
    expect(source).toContain('data-slot="button"');
    expect(source).toMatch(
      /button\.inspector__toggle\s*\{[\s\S]*border: 1px solid var\(--extractum-border\);[\s\S]*background: var\(--extractum-surface-raised\);/,
    );
    expect(source).toMatch(/button\.inspector__toggle :global\(svg\)\s*\{[\s\S]*stroke-width: 2.25;/);
  });

  it("runs footer sync, open and disconnect actions for the selected source", async () => {
    const onSync = vi.fn();
    const onOpen = vi.fn();
    const onDisconnect = vi.fn();
    render(Inspector, { props: { ...base, onSync, onOpen, onDisconnect } });

    await fireEvent.click(screen.getByRole("button", { name: "Синхронизировать" }));
    expect(onSync).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole("button", { name: "Открыть" }));
    expect(onOpen).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole("button", { name: "Убрать" }));
    expect(onDisconnect).toHaveBeenCalledTimes(1);
  });

  it("keeps the v11 inspector footer layout contract", () => {
    expect(source).toContain("inspector__footer-actions");
    expect(source).toContain("inspector__open");
    expect(source).toMatch(
      /\.inspector__sync\s*\{[\s\S]*height: 32px;[\s\S]*border: 1px solid var\(--extractum-primary\);[\s\S]*background: color-mix\(in srgb, var\(--extractum-primary\) 6%, transparent\);/,
    );
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
