export const ACCOUNT_CREATION_MODAL = Object.freeze({
  triggerPlacement: "configured-accounts-header",
  triggerLabel: "Add",
  title: "New Telegram account",
});

export function accountCreationModalActions(setOpen: (open: boolean) => void) {
  return {
    open: () => setOpen(true),
    close: () => setOpen(false),
  };
}
