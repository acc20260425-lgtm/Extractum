import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  createAccount: vi.fn(),
  deleteAccount: vi.fn(),
  getAccountRuntimeStatuses: vi.fn(),
  listAccounts: vi.fn(),
  listenToAccountRuntimeStatus: vi.fn(),
}));

vi.mock("$lib/api/accounts", () => api);
vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$lib/modals", () => ({ openConfirmModal: vi.fn() }));
vi.mock("$lib/toasts", () => ({ pushErrorToast: vi.fn() }));
vi.mock("$lib/components/desktop-dialog.svelte", async () => ({
  default: (await import("$lib/testing/DialogReceiver.svelte")).default,
}));
vi.mock("$lib/components/settings/youtube-settings-panel.svelte", async () => ({
  default: (await import("$lib/testing/EmptyReceiver.svelte")).default,
}));

import AccountsPage from "./+page.svelte";

beforeEach(() => {
  api.listAccounts.mockResolvedValue([]);
  api.getAccountRuntimeStatuses.mockResolvedValue([]);
  api.listenToAccountRuntimeStatus.mockResolvedValue(() => {});
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("accounts route add-account modal", () => {
  it("keeps the account creation form behind a configured-accounts header action", async () => {
    render(AccountsPage);
    await waitFor(() => expect(api.listAccounts).toHaveBeenCalledOnce());

    expect(screen.queryByRole("dialog", { name: "New Telegram account" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(screen.getByRole("dialog", { name: "New Telegram account" })).not.toBeNull();
    expect(screen.getByLabelText("Label")).not.toBeNull();
    expect(screen.getByLabelText("API ID")).not.toBeNull();
    expect(screen.getByLabelText("API Hash")).not.toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Close dialog" }));
    expect(screen.queryByRole("dialog", { name: "New Telegram account" })).toBeNull();
  });
});
