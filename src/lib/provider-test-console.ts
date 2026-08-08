export function providerTestConsoleActions(setOpen: (open: boolean) => void) {
  return {
    open: () => setOpen(true),
    close: () => setOpen(false),
  };
}
