import { describe, expect, it, vi } from "vitest";

describe("accounts route add-account modal", () => {
  it("keeps the account creation form behind a configured-accounts header action", async () => {
    const modulePath = "./accounts-route-add-account-modal";
    const { ACCOUNT_CREATION_MODAL, createAccountCreationModalActions } = await import(/* @vite-ignore */ modulePath);
    const setOpen = vi.fn();
    const actions = createAccountCreationModalActions(setOpen);

    expect(ACCOUNT_CREATION_MODAL).toMatchObject({
      initialOpen: false,
      triggerPlacement: "configured-accounts-header",
      triggerLabel: "Add",
      title: "New Telegram account",
    });
    actions.open();
    actions.close();
    expect(setOpen.mock.calls).toEqual([[true], [false]]);
  });
});
