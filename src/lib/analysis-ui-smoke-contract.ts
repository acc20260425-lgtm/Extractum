export const NOTEBOOKLM_EXPORT_DISABLED_REASON_ID = "notebooklm-export-disabled-reason";
export const YOUTUBE_GROUP_NOTEBOOKLM_DISABLED_REASON =
  "YouTube source-group NotebookLM export is not implemented yet.";

export function notebookLmExportAccessibility(compact: boolean, exportDisabledReason: string | null) {
  const showReason = !compact && Boolean(exportDisabledReason);
  return {
    reasonId: NOTEBOOKLM_EXPORT_DISABLED_REASON_ID,
    ariaDescribedby: showReason ? NOTEBOOKLM_EXPORT_DISABLED_REASON_ID : undefined,
    showReason,
    buttonSmokeId: "notebooklm-export-button",
    reasonSmokeId: NOTEBOOKLM_EXPORT_DISABLED_REASON_ID,
  };
}
