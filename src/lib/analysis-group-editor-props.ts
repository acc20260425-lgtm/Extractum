export const REPORT_SETUP_GROUP_EDITOR_PROPS = Object.freeze({});

export function reportCanvasGroupEditorProps(selectedGroupEditorId: string) {
  return { selectedGroupEditorId };
}

export function sourceGroupEditorSelectionProps(selectedGroupEditorId: string) {
  return { selectedGroupId: selectedGroupEditorId };
}
