export const PROJECT_SOURCE_KEYBOARD_HINT = "↑↓ строка · Enter инспектор";

export function projectSourceKeyboardEnabled({
  hasProject,
  activeSection,
  connectOpen,
  addSourceOpen,
  disconnectOpen,
}: {
  hasProject: boolean;
  activeSection: string;
  connectOpen: boolean;
  addSourceOpen: boolean;
  disconnectOpen: boolean;
}) {
  return hasProject && activeSection === "sources" && !connectOpen && !addSourceOpen && !disconnectOpen;
}

export function projectSourceKeyboardCallbacks({
  activate,
  inspect,
  escape,
}: {
  activate: (id: string) => void;
  inspect: (id: string) => void;
  escape: () => boolean;
}) {
  return {
    onKeyboardActivateSource: activate,
    onKeyboardInspectSource: inspect,
    onKeyboardEscape: escape,
  };
}
