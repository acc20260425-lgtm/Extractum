import path from "node:path";
import * as svelte from "svelte/compiler";
import ts from "typescript";
import { describe, expect, it } from "vitest";

import sourceContractLedger from "../../testing/source-contract-ledger.json";
import { createRepositoryIndex } from "./repository-index.mjs";
import { evaluateRule, registeredRuleIds } from "./repository-rules.mjs";

const root = path.resolve("repository-rule-fixture");
const TELEGRAM_PATH = "src/lib/telegram-contract-paths.ts";
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";

type RuleFixture = {
  positive: Record<string, string>;
  mutations: Record<string, Record<string, string>>;
};

const telegramPathHelper = String.raw`
  import { existsSync, readFileSync, realpathSync } from "node:fs";
  import path from "node:path";

  const repositoryRoot = "root";

  function assertRepositoryRelative(relativePath: string) {
    if (
      !relativePath
      || path.isAbsolute(relativePath)
      || relativePath.includes("\\")
      || relativePath.split("/").some((segment) => segment === "." || segment === "..")
    ) throw new Error("invalid path");
    const selected = path.resolve(repositoryRoot, relativePath);
    const relative = path.relative(repositoryRoot, selected);
    if (relative === "" || relative === ".." || relative.startsWith(".." + path.sep) || path.isAbsolute(relative)) {
      throw new Error("path escaped");
    }
    return selected;
  }

  export function resolveTelegramContractPath(relativePath: string) {
    const selected = assertRepositoryRelative(relativePath);
    if (!existsSync(selected)) throw new Error("missing path");
    const realSelected = realpathSync(selected);
    const realRelative = path.relative(repositoryRoot, realSelected);
    if (realRelative === ".." || realRelative.startsWith(".." + path.sep) || path.isAbsolute(realRelative)) {
      throw new Error("symlink escaped");
    }
    return realSelected;
  }

  export function readTelegramContractFile(relativePath: string) {
    return normalizeTelegramContractSourceText(readFileSync(resolveTelegramContractPath(relativePath), "utf8"));
  }

  export function normalizeTelegramContractSourceText(source: string) {
    return source;
  }
`;

const ruleFixtures: Record<string, RuleFixture> = {
  "rule:telegram-repository-path-safety": {
    positive: { [TELEGRAM_PATH]: telegramPathHelper },
    mutations: {
      "removes the Windows-separator rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace('      || relativePath.includes("\\\\")\n', ""),
      },
      "inverts the dot-segment predicate": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          'segment === "." || segment === ".."',
          'segment !== "." && segment !== ".."',
        ),
      },
      "inverts the missing-file predicate": {
        [TELEGRAM_PATH]: telegramPathHelper.replace("if (!existsSync(selected))", "if (existsSync(selected))"),
      },
    },
  },
  "rule:analysis-source-reader-surface-composition": {
    positive: {
      [ANALYSIS_SURFACE_PATH]: `
        <script lang="ts">
          import SourceBrowserShell from "./source-browser-shell.svelte";
        </script>
        <SourceBrowserShell />
      `,
    },
    mutations: {
      "restores a transitional source reader": {
        [ANALYSIS_SURFACE_PATH]: `
          <script lang="ts">
            import TelegramTimelineReader from "./telegram-timeline-reader.svelte";
          </script>
          <TelegramTimelineReader />
        `,
      },
    },
  },
};

function indexFor(sources: Record<string, string>) {
  return createRepositoryIndex({
    root,
    readFile(absolutePath: string) {
      const relativePath = path.relative(root, absolutePath).replaceAll("\\", "/");
      const source = sources[relativePath];
      if (source === undefined) throw new Error(`missing fixture: ${relativePath}`);
      return source;
    },
    ts,
    svelte,
    loadCargoMetadata: () => ({ packages: [] }),
  });
}

function inSlice3ARanges(id: string) {
  const number = Number(id.slice("SC-".length));
  return (number >= 29 && number <= 59)
    || (number >= 221 && number <= 278)
    || (number >= 561 && number <= 658);
}

describe("repository rule registry", () => {
  const allowedRuleIds = new Set(
    sourceContractLedger.rows
      .filter((row) => inSlice3ARanges(row.id))
      .flatMap((row) => "replacementIds" in row ? row.replacementIds ?? [] : [])
      .filter((id): id is string => id.startsWith("rule:")),
  );

  it("derives the frozen 22-ID allowlist and registers only the two Task 1 evaluators", () => {
    expect(allowedRuleIds.size).toBe(22);
    expect(registeredRuleIds).toEqual([
      "rule:analysis-source-reader-surface-composition",
      "rule:telegram-repository-path-safety",
    ]);
    for (const id of registeredRuleIds) expect(allowedRuleIds.has(id), id).toBe(true);
  });

  it("gives every registered evaluator its own positive fixture and violating mutation", () => {
    expect(Object.keys(ruleFixtures).sort()).toEqual(registeredRuleIds);

    for (const id of registeredRuleIds) {
      const fixture = ruleFixtures[id];
      expect(evaluateRule({ id, index: indexFor(fixture.positive) }), `${id} positive`).toEqual({
        id,
        violations: [],
      });
      expect(Object.keys(fixture.mutations), `${id} mutations`).not.toEqual([]);
      for (const [name, mutation] of Object.entries(fixture.mutations)) {
        expect(evaluateRule({ id, index: indexFor(mutation) }).violations, `${id}: ${name}`).not.toEqual([]);
      }
    }
  });

  it("converts declared-input parse failures to INFRA_ERROR violations", () => {
    const result = evaluateRule({
      id: "rule:analysis-source-reader-surface-composition",
      index: indexFor({ [ANALYSIS_SURFACE_PATH]: "<script>const value = ;</script>" }),
    });

    expect(result).toEqual({
      id: "rule:analysis-source-reader-surface-composition",
      violations: [expect.stringMatching(/^INFRA_ERROR:.*report-source-surface\.svelte/)],
    });
  });

  it("throws for an unknown rule ID", () => {
    expect(() => evaluateRule({ id: "rule:not-registered", index: indexFor({}) })).toThrow(/rule:not-registered/);
  });
});
