import type { Page } from "@playwright/test";

export type TauriScenario = Readonly<{
  invokes?: Readonly<Record<string, unknown>>;
  events?: Readonly<Record<string, readonly unknown[]>>;
}>;

export async function installTauriScenario(page: Page, scenario: TauriScenario = {}) {
  await page.addInitScript(({ invokes = {}, events = {} }) => {
    type TauriEvent = { event: string; id: number; payload: unknown };

    const callbacks = new Map<number, (event: TauriEvent) => void>();
    const listeners = new Map<number, { event: string; callbackId: number }>();
    let nextCallbackId = 1;
    let nextListenerId = 1;

    const internals = {
      async invoke(command: string, args: Record<string, unknown> = {}) {
        if (command === "plugin:event|listen") {
          const listenerId = nextListenerId++;
          const event = String(args.event);
          const callbackId = Number(args.handler);
          listeners.set(listenerId, { event, callbackId });
          queueMicrotask(() => {
            for (const payload of events[event] ?? []) {
              callbacks.get(callbackId)?.({ event, id: listenerId, payload });
            }
          });
          return listenerId;
        }
        if (command === "plugin:event|unlisten") return undefined;
        if (!Object.hasOwn(invokes, command)) throw new Error(`Unexpected Tauri command: ${command}`);
        return invokes[command];
      },
      transformCallback(callback: (event: TauriEvent) => void) {
        const callbackId = nextCallbackId++;
        callbacks.set(callbackId, callback);
        return callbackId;
      },
      unregisterCallback(callbackId: number) {
        callbacks.delete(callbackId);
      },
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", { value: internals, configurable: true });
    Object.defineProperty(window, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
      value: {
        unregisterListener(_event: string, listenerId: number) {
          listeners.delete(listenerId);
        },
      },
      configurable: true,
    });
  }, scenario);
}
