import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";
import type { GeminiBrowserRunRequest, GeminiBrowserRunResult } from "./protocol";
import { redactText, redactUrl } from "./redaction";

export async function captureFailureArtifacts(input: {
  page: Page;
  artifactDir: string;
  request: GeminiBrowserRunRequest;
  status: GeminiBrowserRunResult["status"];
  message: string;
}): Promise<GeminiBrowserRunResult["artifacts"]> {
  await mkdir(input.artifactDir, { recursive: true });
  const telemetryPath = join(input.artifactDir, "telemetry.json");
  const htmlPath = input.request.artifact_mode === "full" ? join(input.artifactDir, "page.html") : null;
  const screenshotPath = input.request.artifact_mode === "full" ? join(input.artifactDir, "page.png") : null;
  let artifactWriteError: string | null = null;

  let pageUrl = "about:blank";
  try {
    pageUrl = input.page.url();
  } catch {
    pageUrl = "about:blank";
  }

  const telemetry = {
    status: input.status,
    message: input.message,
    url: redactUrl(pageUrl),
    artifact_mode: input.request.artifact_mode,
  };

  await writeFile(telemetryPath, JSON.stringify(telemetry, null, 2)).catch((error) => {
    artifactWriteError = String(error);
  });

  if (htmlPath) {
    const html = await input.page
      .content()
      .catch(() => "<html><body>[page unavailable]</body></html>");
    await writeFile(htmlPath, redactText(html, input.request.prompt)).catch((error) => {
      artifactWriteError = String(error);
    });
  }

  if (screenshotPath) {
    await input.page.screenshot({ path: screenshotPath, fullPage: true }).catch((error) => {
      artifactWriteError = String(error);
    });
  }

  return {
    run_dir: input.artifactDir,
    html: htmlPath,
    screenshot: screenshotPath,
    telemetry: telemetryPath,
    artifact_write_error: artifactWriteError,
  };
}
