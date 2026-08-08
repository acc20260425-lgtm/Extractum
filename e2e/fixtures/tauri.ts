import type { Page } from "@playwright/test";

export type TauriScenario = Readonly<{
  invokes?: Readonly<Record<string, unknown>>;
  events?: Readonly<Record<string, readonly unknown[]>>;
}>;

export async function installTauriScenario(page: Page, scenario: TauriScenario = {}) {
  await page.addInitScript(({ invokes = {}, events = {} }) => {
    type TauriEvent = { event: string; id: number; payload: unknown };

    const callbacks = new Map<number, { callback: (event: TauriEvent) => void; once: boolean }>();
    let nextCallbackId = 1;

    function runCallback(callbackId: number, event: TauriEvent) {
      const entry = callbacks.get(callbackId);
      if (!entry) return;
      entry.callback(event);
      if (entry.once) callbacks.delete(callbackId);
    }

    const internals = {
      async invoke(command: string, args: Record<string, unknown> = {}) {
        if (command === "plugin:event|listen") {
          const event = String(args.event);
          const callbackId = Number(args.handler);
          queueMicrotask(() => {
            for (const payload of events[event] ?? []) {
              runCallback(callbackId, { event, id: callbackId, payload });
            }
          });
          return callbackId;
        }
        if (command === "plugin:event|unlisten") {
          callbacks.delete(Number(args.eventId));
          return undefined;
        }
        if (!Object.hasOwn(invokes, command)) throw new Error(`Unexpected Tauri command: ${command}`);
        return invokes[command];
      },
      transformCallback(callback: (event: TauriEvent) => void, once = false) {
        const callbackId = nextCallbackId++;
        callbacks.set(callbackId, { callback, once });
        return callbackId;
      },
      unregisterCallback(callbackId: number) {
        callbacks.delete(callbackId);
      },
      callbacks,
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", { value: internals, configurable: true });
    Object.defineProperty(window, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
      value: {
        unregisterListener(_event: string, listenerId: number) {
          callbacks.delete(listenerId);
        },
      },
      configurable: true,
    });
  }, scenario);
}
