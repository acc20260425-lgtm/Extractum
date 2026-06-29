// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import RunDock from "./RunDock.svelte";

afterEach(cleanup);

describe("RunDock", () => {
  it("shows the active run label, queue count, and exports on click", async () => {
    const onExport = vi.fn();
    render(RunDock, {
      props: { activeRunLabel: "Финтех-мониторинг · идёт анализ", queueCount: 2, onExport },
    });

    expect(screen.getByText("Финтех-мониторинг · идёт анализ")).toBeTruthy();
    expect(screen.getByText("Очередь: 2")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Экспорт" }));
    expect(onExport).toHaveBeenCalledTimes(1);
  });

  it("shows an idle label when there is no active run", () => {
    render(RunDock, { props: { activeRunLabel: null, queueCount: 0 } });

    expect(screen.getByText("Нет активных прогонов")).toBeTruthy();
  });
});
