export interface CdpPageLike {
  isClosed: () => boolean;
  url: () => string;
}

export function selectGeminiPage<T extends CdpPageLike>(pages: T[]): T | null {
  return pages.find(isUsableGeminiPage) ?? null;
}

export function isClosedTargetError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return /target closed|page closed|browser has been closed|context closed|connection closed|target page, context or browser has been closed/i.test(
    message,
  );
}

function isUsableGeminiPage(page: CdpPageLike): boolean {
  if (page.isClosed()) return false;

  let rawUrl: string;
  try {
    rawUrl = page.url();
  } catch {
    return false;
  }

  try {
    const parsed = new URL(rawUrl);
    return parsed.protocol === "https:" && parsed.hostname === "gemini.google.com";
  } catch {
    return false;
  }
}
