export const ACCOUNT_CREATION_MODAL = Object.freeze({
  initialOpen: false as boolean,
  triggerPlacement: "configured-accounts-header",
  triggerLabel: "Add",
  title: "New Telegram account",
});

export function createAccountCreationModalActions(setOpen: (open: boolean) => void) {
  return {
    open: () => setOpen(true),
    close: () => setOpen(false),
  };
}
