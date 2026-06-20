import { createRequire } from "node:module";
import type { AdapterVariant, GeminiAdapterStatus } from "./types";

const require = createRequire(import.meta.url);
const rawMatrixDefinition = require("../matrix-cases.json") as unknown;

export type MatrixAction = "probe" | "send";

export type MatrixScenario = {
  id: string;
  mockVariant: string;
  action: MatrixAction;
  expectedStatuses: GeminiAdapterStatus[];
  timeoutMs: number;
  quietMs: number;
  requiresRawText: boolean;
  requiresTelemetryArtifact: boolean;
  requiresHtmlArtifact: boolean;
  requiresScreenshotArtifact: boolean;
  requiresTelemetryNetwork: boolean;
  closePageBeforeRun: boolean;
};

type MatrixDefinition = {
  adapterVariants: AdapterVariant[];
  scenarios: MatrixScenario[];
};

const matrixDefinition = rawMatrixDefinition as MatrixDefinition;

export const matrixAdapterVariants = matrixDefinition.adapterVariants;
export const matrixScenarios = matrixDefinition.scenarios;

export function expectedMatrixPairTitles(): string[] {
  return matrixAdapterVariants.flatMap((variant) =>
    matrixScenarios.map((scenario) => `${variant} / ${scenario.id}`),
  );
}
