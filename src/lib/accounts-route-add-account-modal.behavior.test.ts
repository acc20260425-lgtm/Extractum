import { describe, expect, it } from "vitest";

describe("accounts route add-account modal", () => {
  it("keeps the account creation form behind a configured-accounts header action", async () => {
    const modulePath = "./accounts-route-add-account-modal";
    const { ACCOUNT_CREATION_MODAL } = await import(/* @vite-ignore */ modulePath);

    expect(ACCOUNT_CREATION_MODAL).toEqual({
      triggerPlacement: "configured-accounts-header",
      triggerLabel: "Add",
      title: "New Telegram account",
    });
  });
});
