import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import "$lib/testing/dom-assertions";
const api = vi.hoisted(() => ({ listAccounts: vi.fn(), getAccountRuntimeStatuses: vi.fn(), listTelegramSources: vi.fn(), addTelegramSource: vi.fn() }));
vi.mock("$lib/api/accounts", () => ({ listAccounts: api.listAccounts, getAccountRuntimeStatuses: api.getAccountRuntimeStatuses }));
vi.mock("$lib/api/sources", () => ({ listTelegramSources: api.listTelegramSources, addTelegramSource: api.addTelegramSource }));
import LibraryTelegramDialogImport from "./LibraryTelegramDialogImport.svelte";
beforeEach(() => { api.listAccounts.mockResolvedValue([{ id: 7, label: "Primary", api_id: 1, api_hash: "hash", phone: null }]); api.getAccountRuntimeStatuses.mockResolvedValue([{ account_id: 7, status: "ready", message: null }]); api.listTelegramSources.mockResolvedValue([{ id: 99, title: "Research chat", username: "research", sourceSubtype: "supergroup" }]); api.addTelegramSource.mockResolvedValue({ id: 55, title: "Research chat", externalId: "99" }); });
afterEach(() => { cleanup(); vi.clearAllMocks(); });
describe("library add source contract", () => {
  it("adds Telegram sources only from selected account dialogs", async () => {
    const changed = vi.fn(); const status = vi.fn(); render(LibraryTelegramDialogImport, { onSourcesChanged: changed, onStatus: status });
    await waitFor(() => expect(api.listAccounts).toHaveBeenCalledOnce());
    expect(api.getAccountRuntimeStatuses).toHaveBeenCalledWith([7]);
    expect(api.listTelegramSources).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Load dialogs" }));
    await screen.findByText("Research chat");
    expect(api.listTelegramSources).toHaveBeenCalledWith(7);
    await fireEvent.click(screen.getByRole("button", { name: /Research chat/ }));
    expect(screen.getByRole("button", { name: "Add selected" })).toBeEnabled();
    await fireEvent.click(screen.getByRole("button", { name: "Add selected" }));
    await waitFor(() => expect(api.addTelegramSource).toHaveBeenCalledWith({ accountId: 7, sourceRef: "99", expectedSubtype: "supergroup" }));
    expect(changed).toHaveBeenCalledWith(55);
    expect(status).toHaveBeenCalledWith('Source "Research chat" added.');
    expect(screen.getByRole("region", { name: "Telegram dialog import" })).toBeTruthy();
  });
  it("keeps Telegram project connection on the scalar callback path", async () => {
    const changed = vi.fn(); render(LibraryTelegramDialogImport, { onSourcesChanged: changed, onStatus: vi.fn() });
    await screen.findByRole("button", { name: "Load dialogs" }); await fireEvent.click(screen.getByRole("button", { name: "Load dialogs" })); await screen.findByText("Research chat"); await fireEvent.click(screen.getByRole("button", { name: /Research chat/ })); await fireEvent.click(screen.getByRole("button", { name: "Add selected" }));
    await waitFor(() => expect(changed).toHaveBeenCalledWith(55));
    expect(changed).toHaveBeenCalledOnce();
    expect(changed.mock.calls[0]).toEqual([55]);
  });
});
