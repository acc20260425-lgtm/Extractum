import { writable } from "svelte/store";

export interface ConfirmModalOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  tone?: "default" | "danger";
}

export interface ConfirmModalState extends Required<ConfirmModalOptions> {
  kind: "confirm";
}

type ModalState = ConfirmModalState | null;

const modalStore = writable<ModalState>(null);
let pendingResolver: ((value: boolean) => void) | null = null;

function closePending(result: boolean) {
  if (pendingResolver) {
    pendingResolver(result);
    pendingResolver = null;
  }
  modalStore.set(null);
}

export function openConfirmModal(options: ConfirmModalOptions) {
  if (pendingResolver) {
    pendingResolver(false);
    pendingResolver = null;
  }

  modalStore.set({
    kind: "confirm",
    title: options.title,
    message: options.message,
    confirmLabel: options.confirmLabel ?? "Confirm",
    cancelLabel: options.cancelLabel ?? "Cancel",
    tone: options.tone ?? "default",
  });

  return new Promise<boolean>((resolve) => {
    pendingResolver = resolve;
  });
}

export function confirmActiveModal() {
  closePending(true);
}

export function dismissActiveModal() {
  closePending(false);
}

export const activeModal = {
  subscribe: modalStore.subscribe,
};
