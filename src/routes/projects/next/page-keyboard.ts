export function projectSourceKeyboardContract({
  hasProject,
  activeSection,
  dialogsClosed,
  keyboardHint,
  activate,
  inspect,
  escape,
}: {
  hasProject: boolean;
  activeSection: string;
  dialogsClosed: boolean;
  keyboardHint: string;
  activate: (id: string) => void;
  inspect: (id: string) => void;
  escape: () => boolean;
}) {
  return {
    enabled: hasProject && activeSection === "sources" && dialogsClosed,
    keyboardHint,
    onKeyboardActivateSource: activate,
    onKeyboardInspectSource: inspect,
    onKeyboardEscape: escape,
  };
}
