import { existsSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";
import {
  probeReadyDomOnly,
  probeReadyResilientScoring,
  probeReadyTelemetryAssisted,
  sendSingleDomOnly,
  sendSingleResilientScoring,
  sendSingleTelemetryAssisted,
} from "../src/dom-contract";
import { matrixAdapterVariants, matrixScenarios } from "../src/matrix-cases";
import type { SendSingleOptions } from "../src/dom-contract";
import type { AdapterVariant, GeminiAdapterResult } from "../src/types";

type AdapterHarness = {
  send(page: Parameters<typeof sendSingleDomOnly>[0], prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult>;
  probe(page: Parameters<typeof sendSingleDomOnly>[0], options: SendSingleOptions): Promise<GeminiAdapterResult>;
};

const adapters: Record<AdapterVariant, AdapterHarness> = {
  "dom-only": {
    send: sendSingleDomOnly,
    probe: probeReadyDomOnly,
  },
  "resilient-scoring": {
    send: sendSingleResilientScoring,
    probe: probeReadyResilientScoring,
  },
  "telemetry-assisted": {
    send: sendSingleTelemetryAssisted,
    probe: probeReadyTelemetryAssisted,
  },
};

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test.describe("Gemini adapter executable scenario matrix", () => {
  for (const variant of matrixAdapterVariants) {
    for (const scenario of matrixScenarios) {
      test(`${variant} / ${scenario.id}`, async ({ page }) => {
        await page.goto(server.url(scenario.mockVariant));
        if (scenario.closePageBeforeRun) {
          await page.close();
        }

        const artifactDir = path.join(
          "research/gemini_browser_adapter/artifacts/matrix",
          variant,
          scenario.id,
        );

        const result =
          scenario.action === "probe"
            ? await adapters[variant].probe(page, {
                timeoutMs: scenario.timeoutMs,
                quietMs: scenario.quietMs,
                artifactDir,
              })
            : await adapters[variant].send(page, "hello from matrix", {
                timeoutMs: scenario.timeoutMs,
                quietMs: scenario.quietMs,
                artifactDir,
              });

        const hasScreenshotArtifact = Boolean(result.artifacts?.screenshotPath && existsSync(result.artifacts.screenshotPath));
        const hasHtmlArtifact = Boolean(result.artifacts?.htmlPath && existsSync(result.artifacts.htmlPath));
        const hasTelemetryArtifact = Boolean(result.artifacts?.telemetryPath && existsSync(result.artifacts.telemetryPath));

        await mkdir(artifactDir, { recursive: true });
        await writeFile(
          path.join(artifactDir, "result.json"),
          JSON.stringify(
            {
              variant,
              scenarioId: scenario.id,
              status: result.status,
              expectedStatuses: scenario.expectedStatuses,
              elapsedMs: result.elapsedMs,
              rawTextPresent: Boolean((result.rawText ?? "").trim()),
              artifacts: {
                screenshot: hasScreenshotArtifact,
                html: hasHtmlArtifact,
                telemetry: hasTelemetryArtifact,
              },
              expectedArtifacts: {
                screenshot: scenario.requiresScreenshotArtifact,
                html: scenario.requiresHtmlArtifact,
                telemetry: scenario.requiresTelemetryArtifact,
              },
              falseCompletion: result.status === "ok" && !scenario.expectedStatuses.includes("ok"),
              unexpectedStatus: !scenario.expectedStatuses.includes(result.status),
              timeoutOrHang: result.status === "generation_timeout" || result.elapsedMs >= scenario.timeoutMs,
            },
            null,
            2,
          ),
          "utf8",
        );

        expect(result.variant).toBe(variant);
        expect(scenario.expectedStatuses).toContain(result.status);

        if (scenario.requiresRawText) {
          expect(result.rawText ?? "").toContain("Mock final answer");
        } else {
          expect(result.rawText ?? "").not.toContain("Mock final answer");
        }

        if (scenario.requiresScreenshotArtifact) expect(hasScreenshotArtifact).toBe(true);
        if (scenario.requiresHtmlArtifact) expect(hasHtmlArtifact).toBe(true);
        if (scenario.requiresTelemetryArtifact) expect(hasTelemetryArtifact).toBe(true);

        if (scenario.requiresTelemetryNetwork && variant === "telemetry-assisted") {
          expect(result.networkSummary.some((event) => event.kind === "response")).toBe(true);
          expect(result.networkSummary.some((event) => event.url.includes("/mock-gemini-event"))).toBe(true);
        }
      });
    }
  }
});
