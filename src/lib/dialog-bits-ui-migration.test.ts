import { describe, expect, it } from "vitest";
import desktopDialogSource from "./components/desktop-dialog.svelte?raw";
import modalHostSource from "./components/modal-host.svelte?raw";

function cssRule(source: string, selector: string) {
  const start = source.indexOf(`${selector} {`);
  expect(start, `${selector} rule`).toBeGreaterThanOrEqual(0);

  const end = source.indexOf("\n  }", start);
  expect(end, `${selector} rule end`).toBeGreaterThan(start);

  return source.slice(start, end);
}

describe("dialog Bits UI migration", () => {
  it("uses Bits UI Dialog for desktop dialogs while keeping explicit close controls", () => {
    expect(desktopDialogSource).toContain('import { Dialog } from "bits-ui";');
    expect(desktopDialogSource).toContain("<Dialog.Root");
    expect(desktopDialogSource).toContain("<Dialog.Portal");
    expect(desktopDialogSource).toContain("<Dialog.Overlay");
    expect(desktopDialogSource).toContain("<Dialog.Content");
    expect(desktopDialogSource).toContain('interactOutsideBehavior="ignore"');
    expect(desktopDialogSource).toContain("<Dialog.Close");
    expect(desktopDialogSource).toContain("<X size={16}");

    expect(desktopDialogSource).not.toContain("function trapFocus");
    expect(desktopDialogSource).not.toContain("handleBackdropClick");
    expect(desktopDialogSource).not.toContain("previousFocusedElement");
  });

  it("uses Bits UI AlertDialog for global confirms without backdrop or Enter-to-confirm shortcuts", () => {
    expect(modalHostSource).toContain('import { AlertDialog } from "bits-ui";');
    expect(modalHostSource).toContain("<AlertDialog.Root");
    expect(modalHostSource).toContain("<AlertDialog.Portal");
    expect(modalHostSource).toContain("<AlertDialog.Overlay");
    expect(modalHostSource).toContain("<AlertDialog.Content");
    expect(modalHostSource).toContain('interactOutsideBehavior="ignore"');
    expect(modalHostSource).toContain("<AlertDialog.Cancel");
    expect(modalHostSource).toContain("<AlertDialog.Action");

    expect(modalHostSource).not.toContain("<svelte:window");
    expect(modalHostSource).not.toContain("function trapFocus");
    expect(modalHostSource).not.toContain('event.key === "Enter"');
    expect(modalHostSource).not.toContain("handleBackdropClick");
    expect(modalHostSource).not.toContain("previousFocusedElement");
  });

  it("keeps portaled dialog content above the overlay layer", () => {
    const dialogBackdropRule = cssRule(desktopDialogSource, ".dialog-backdrop");
    const dialogCardRule = cssRule(desktopDialogSource, ".dialog-card");
    const modalBackdropRule = cssRule(modalHostSource, ".modal-backdrop");
    const modalCardRule = cssRule(modalHostSource, ".modal-card");

    expect(dialogBackdropRule).toContain("z-index: 60;");
    expect(dialogCardRule).toContain("position: fixed;");
    expect(dialogCardRule).toContain("top: 50%;");
    expect(dialogCardRule).toContain("left: 50%;");
    expect(dialogCardRule).toContain("z-index: 61;");
    expect(dialogCardRule).toContain("translate: -50% -50%;");

    expect(modalBackdropRule).toContain("z-index: 70;");
    expect(modalCardRule).toContain("position: fixed;");
    expect(modalCardRule).toContain("top: 50%;");
    expect(modalCardRule).toContain("left: 50%;");
    expect(modalCardRule).toContain("z-index: 71;");
    expect(modalCardRule).toContain("translate: -50% -50%;");
  });
});
