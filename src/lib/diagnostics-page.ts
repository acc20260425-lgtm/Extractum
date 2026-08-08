export const EMPTY_DIAGNOSTIC_ISSUES_MESSAGE = "No diagnostic issue rows match this view.";

export function diagnosticsTablesBeforeOverview(mode: "issues" | "all") {
  return mode === "issues";
}

export async function runDiagnosticsRefresh<T>({
  initial,
  load,
  onStart,
  onSuccess,
  onError,
  onFinish,
}: {
  initial: boolean;
  load: () => Promise<T>;
  onStart: (initial: boolean) => void;
  onSuccess: (summary: T) => void;
  onError: (error: unknown, initial: boolean) => void;
  onFinish: (initial: boolean) => void;
}) {
  onStart(initial);
  try {
    onSuccess(await load());
  } catch (error) {
    onError(error, initial);
  } finally {
    onFinish(initial);
  }
}
