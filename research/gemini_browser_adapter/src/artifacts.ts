import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "@playwright/test";
import { redactUrl, reducedDomSnapshot } from "./redaction";
import type { FailureArtifacts, LocatorAttempt, NetworkEventSummary } from "./types";

export type CaptureFailureInput = {
  page: Page;
  artifactDir: string;
  reason: string;
  locatorAttempts: LocatorAttempt[];
  networkSummary: NetworkEventSummary[];
  artifactMode: "full" | "reduced";
};

function safePageUrl(page: Page): string {
  try {
    return redactUrl(page.url());
  } catch {
    return "about:blank#page-url-unavailable";
  }
}

async function safeHtmlSnapshot(input: CaptureFailureInput): Promise<string> {
  const fallback = '<!doctype html><html><body data-capture-error="page_content_unavailable"></body></html>';
  if (input.artifactMode === "reduced") {
    const reduced = await reducedDomSnapshot(input.page).catch(() => "");
    return reduced || fallback;
  }
  return await input.page.content().catch(() => fallback);
}

export async function captureFailureArtifacts(input: CaptureFailureInput): Promise<FailureArtifacts> {
  const screenshotPath = input.artifactMode === "reduced" ? null : path.join(input.artifactDir, "failure.png");
  const htmlPath = path.join(input.artifactDir, "page.html");
  const telemetryPath = path.join(input.artifactDir, "telemetry.json");
  const artifactDirReady = await mkdir(input.artifactDir, { recursive: true })
    .then(() => true)
    .catch(() => false);

  if (!artifactDirReady) {
    return {
      screenshotPath: null,
      htmlPath: null,
      telemetryPath: null,
      tracePath: null,
    };
  }

  const html = await safeHtmlSnapshot(input);
  const pageUrl = safePageUrl(input.page);

  const screenshotWritten = screenshotPath
    ? await input.page.screenshot({ path: screenshotPath, fullPage: true }).then(() => true).catch(() => false)
    : false;
  const htmlWritten = await writeFile(htmlPath, html, "utf8")
    .then(() => true)
    .catch(() => false);
  const telemetryWritten = await writeFile(
    telemetryPath,
    JSON.stringify(
      {
        reason: input.reason,
        url: pageUrl,
        locatorAttempts: input.locatorAttempts,
        networkSummary: input.networkSummary,
        capturedAt: new Date().toISOString(),
      },
      null,
      2,
    ),
    "utf8",
  )
    .then(() => true)
    .catch(() => false);

  return {
    screenshotPath: screenshotWritten ? screenshotPath : null,
    htmlPath: htmlWritten ? htmlPath : null,
    telemetryPath: telemetryWritten ? telemetryPath : null,
    tracePath: null,
  };
}
