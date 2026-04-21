import { writable } from "svelte/store";

export type ToastKind = "error" | "success" | "info";

export interface ToastMessage {
  id: number;
  kind: ToastKind;
  message: string;
  duration_ms: number;
}

const MAX_VISIBLE_TOASTS = 4;
const DEFAULT_DURATION_MS = 5000;

const toastStore = writable<ToastMessage[]>([]);
const timers = new Map<number, ReturnType<typeof setTimeout>>();
let nextToastId = 1;

function clearToastTimer(id: number) {
  const timer = timers.get(id);
  if (timer !== undefined) {
    clearTimeout(timer);
    timers.delete(id);
  }
}

function scheduleDismiss(id: number, durationMs: number) {
  clearToastTimer(id);
  const timer = setTimeout(() => {
    dismissToast(id);
  }, durationMs);
  timers.set(id, timer);
}

export function dismissToast(id: number) {
  clearToastTimer(id);
  toastStore.update((messages) => messages.filter((message) => message.id !== id));
}

export function pushToast(
  kind: ToastKind,
  message: string,
  durationMs = DEFAULT_DURATION_MS
) {
  const trimmedMessage = message.trim();
  if (!trimmedMessage) return;

  const id = nextToastId++;
  const toast: ToastMessage = {
    id,
    kind,
    message: trimmedMessage,
    duration_ms: durationMs,
  };

  toastStore.update((messages) => {
    const nextMessages = [...messages, toast];
    if (nextMessages.length <= MAX_VISIBLE_TOASTS) {
      return nextMessages;
    }

    const overflow = nextMessages.length - MAX_VISIBLE_TOASTS;
    const removed = nextMessages.slice(0, overflow);
    for (const item of removed) {
      clearToastTimer(item.id);
    }
    return nextMessages.slice(overflow);
  });

  scheduleDismiss(id, durationMs);
}

export function pushErrorToast(message: string, durationMs = 6500) {
  pushToast("error", message, durationMs);
}

export const toasts = {
  subscribe: toastStore.subscribe,
};
