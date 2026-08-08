import { describe, expect, it, vi } from "vitest";
import path from "node:path";
import { createHash } from "node:crypto";
import ts from "typescript";
import { listen, once } from "@tauri-apps/api/event";

import { VITEST_PROJECT_DEFINITIONS } from "../../vitest.config";
import runnerCensus from "../../testing/runner-census.json";
import { installTauriScenario } from "../../e2e/fixtures/tauri";

import {
  buildLedgerDraft,
  createCliGitMetadata,
  discoverSourceReaders,
  discoverTestDeclarations,
  selectVitestTrackedPaths,
  validateSourceContractLedger,
} from "./extract-source-contract-ledger.mjs";

import {
  collectPlaywrightFiles,
  collectVitestFiles,
  discoverFilesystemCandidates,
  normalizeRepoPath,
  runTransitionValidation,
  validateCensusSchema,
  validateRunnerCensus,
} from "./testing-transition.mjs";
import {
  createLedgerLiveCensus,
  collectPlaywrightReplacementEvidence,
  collectTrackedTestSources,
  collectTelegramCargoReplacementEvidence,
  evaluateTelegramCargoTestIdentityOwnership,
} from "../validate-testing-transition.mjs";

const root = "C:/repo";
const census = {
  schemaVersion: 1,
  vitestOwners: [{ id: "vitest:root", args: [], ownerScript: "test" }],
  playwrightOwners: [{ id: "playwright:adapter", config: "research/adapter/playwright.config.ts", ownerScript: "test:adapter:e2e" }],
  nonstandardTests: [],
  fixtureExceptions: [],
};

function check(overrides: Record<string, unknown> = {}) {
  return {
    census,
    filesystemFiles: ["src/a.test.ts", "research/adapter/tests/e2e.spec.ts"],
    vitestFiles: { "vitest:root": ["src/a.test.ts"] },
    playwrightFiles: { "playwright:adapter": ["research/adapter/tests/e2e.spec.ts"] },
    ...overrides,
  };
}

describe("runner census validation", () => {
  it("app and adapter census fixtures are clean and diagnose an app collision exactly", () => {
    const appAndAdapterCensus = {
      schemaVersion: 1,
      vitestOwners: [{ id: "vitest:unit-node", args: ["--project", "unit-node"], ownerScript: "test:unit" }],
      playwrightOwners: [
        { id: "playwright:gemini-browser-adapter", config: "research/gemini_browser_adapter/playwright.config.ts", ownerScript: "test:gemini-browser-adapter:e2e" },
        { id: "playwright:app-e2e", config: "e2e/playwright.config.ts", ownerScript: "test:app:e2e" },
      ],
      nonstandardTests: [],
      fixtureExceptions: [],
    };
    const cleanFixture = {
      census: appAndAdapterCensus,
      filesystemFiles: ["src/unit.test.ts", "research/gemini_browser_adapter/tests/adapter.spec.ts", "e2e/smoke.spec.ts"],
      vitestFiles: { "vitest:unit-node": ["src/unit.test.ts"] },
      playwrightFiles: {
        "playwright:gemini-browser-adapter": ["research/gemini_browser_adapter/tests/adapter.spec.ts"],
        "playwright:app-e2e": ["e2e/smoke.spec.ts"],
      },
    };

    expect(validateRunnerCensus(cleanFixture)).toEqual([]);
    expect(validateRunnerCensus({
      ...cleanFixture,
      vitestFiles: { "vitest:unit-node": ["src/unit.test.ts", "e2e/smoke.spec.ts"] },
    })).toContain("duplicate ownership: e2e/smoke.spec.ts -> playwright:app-e2e, vitest:unit-node");
  });

  it("tauri fixture protocol delivers configured events and removes callback registrations", async () => {
    const addInitScript = vi.fn(async (installer: (scenario: unknown) => void, scenario: unknown) => installer(scenario));
    vi.stubGlobal("window", {});

    try {
      await installTauriScenario({ addInitScript } as never, {
        invokes: { configured_command: { configured: true } },
        events: {
          progress: ["first", "second"],
          complete: ["once", "ignored"],
          manual_once: ["first", "ignored"],
        },
      });

      const internals = (window as typeof window & { __TAURI_INTERNALS__: {
        invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
        transformCallback(callback: (event: unknown) => void, once?: boolean): number;
        unregisterCallback(callbackId: number): void;
        callbacks: Map<number, unknown>;
      } }).__TAURI_INTERNALS__;
      const transformCallback = internals.transformCallback.bind(internals);
      const callbackIds: number[] = [];
      internals.transformCallback = (callback, once = false) => {
        const callbackId = transformCallback(callback, once);
        callbackIds.push(callbackId);
        return callbackId;
      };
      const callbackId = internals.transformCallback(() => {});

      expect(await internals.invoke("plugin:event|listen", { event: "manual", handler: callbackId })).toBe(callbackId);
      expect(await internals.invoke("configured_command")).toEqual({ configured: true });
      await expect(internals.invoke("unknown_command")).rejects.toThrow("Unexpected Tauri command: unknown_command");

      const manualOnce = vi.fn();
      const manualOnceId = internals.transformCallback(manualOnce, true);
      await internals.invoke("plugin:event|listen", { event: "manual_once", handler: manualOnceId });

      const progress: unknown[] = [];
      const unlistenProgress = await listen("progress", (event) => progress.push(event.payload));
      const progressCallbackId = callbackIds.at(-1);
      const complete: unknown[] = [];
      await once("complete", (event) => complete.push(event.payload));
      await Promise.resolve();

      expect(progress).toEqual(["first", "second"]);
      expect(complete).toEqual(["once"]);
      expect(manualOnce).toHaveBeenCalledTimes(1);
      expect(internals.callbacks.has(manualOnceId)).toBe(false);

      const manualUnregisterId = internals.transformCallback(() => {});
      await internals.invoke("plugin:event|listen", { event: "manual_unregistered", handler: manualUnregisterId });
      expect(internals.callbacks.has(manualUnregisterId)).toBe(true);
      internals.unregisterCallback(manualUnregisterId);
      expect(internals.callbacks.has(manualUnregisterId)).toBe(false);

      await unlistenProgress();
      await internals.invoke("plugin:event|listen", { event: "progress", handler: progressCallbackId });
      await Promise.resolve();
      expect(progress).toEqual(["first", "second"]);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it("application Playwright owner is the sole owner for app e2e candidates", () => {
    const playwrightOwners = runnerCensus.playwrightOwners as Array<{ id: string; config: string; ownerScript: string }>;
    const unitNode = VITEST_PROJECT_DEFINITIONS.find((definition) => definition.name === "unit-node");

    expect(playwrightOwners).toEqual(expect.arrayContaining([
      expect.objectContaining({
        id: "playwright:gemini-browser-adapter",
        config: "research/gemini_browser_adapter/playwright.config.ts",
        ownerScript: "test:gemini-browser-adapter:e2e",
      }),
      expect.objectContaining({
        id: "playwright:app-e2e",
        config: "e2e/playwright.config.ts",
        ownerScript: "test:app:e2e",
      }),
    ]));
    expect(unitNode?.exclude).toContain("e2e/**/*.spec.ts");

    expect(validateRunnerCensus({
      census: runnerCensus,
      filesystemFiles: ["e2e/smoke.spec.ts"],
      vitestFiles: Object.fromEntries(runnerCensus.vitestOwners.map((owner: { id: string }) => [owner.id, []])),
      playwrightFiles: {
        "playwright:gemini-browser-adapter": [],
        "playwright:app-e2e": ["e2e/smoke.spec.ts"],
      },
    })).not.toEqual(expect.arrayContaining([
      "duplicate ownership: e2e/smoke.spec.ts -> playwright:app-e2e, vitest:unit-node",
      "unowned filesystem candidate: e2e/smoke.spec.ts",
    ]));
  });

  it("accepts exact bidirectional ownership", () => {
    expect(validateRunnerCensus(check())).toEqual([]);
  });

  it("reports sorted actionable ownership, candidate, empty-owner, and runner failures", () => {
    expect(validateRunnerCensus(check({
      filesystemFiles: ["src/missing.test.ts"],
      vitestFiles: { "vitest:root": ["src/extra.test.ts"] },
      playwrightFiles: { "playwright:adapter": [] },
      runnerIssues: ["runner failed: vitest:root"],
    }))).toEqual([
      "collected non-candidate: vitest:root -> src/extra.test.ts",
      "empty owner: playwright:adapter",
      "runner failed: vitest:root",
      "unowned filesystem candidate: src/missing.test.ts",
    ]);
  });

  it("rejects duplicate ownership even if a fixture exception names the path", () => {
    const issues = validateRunnerCensus(check({
      vitestFiles: { "vitest:root": ["src/a.test.ts"] },
      playwrightFiles: { "playwright:adapter": ["src/a.test.ts", "research/adapter/tests/e2e.spec.ts"] },
      census: { ...census, fixtureExceptions: [{ path: "src/a.test.ts", reason: "fixture", owner: "fixture" }] },
    }));

    expect(issues).toContain("duplicate ownership: src/a.test.ts -> playwright:adapter, vitest:root");
    expect(issues).toContain("stale fixture exception: src/a.test.ts");
  });

  it("requires exact, necessary exceptions with a path, reason, and owner", () => {
    const issues = validateRunnerCensus(check({
      filesystemFiles: ["src/fixture.test.ts"],
      vitestFiles: { "vitest:root": ["scripts/nonstandard.test.ts"] },
      playwrightFiles: { "playwright:adapter": [] },
      census: {
        ...census,
        nonstandardTests: [{ path: "scripts/nonstandard.test.ts", reason: "custom runner", owner: "vitest:root" }],
        fixtureExceptions: [{ path: "src/fixture.test.ts", reason: "fixture", owner: "fixture" }],
      },
    }));
    expect(issues).toContain("empty owner: playwright:adapter");
    expect(issues).not.toContain("unowned filesystem candidate: src/fixture.test.ts");
    expect(issues).not.toContain("collected non-candidate: vitest:root -> scripts/nonstandard.test.ts");
  });

  it("rejects unsafe, duplicate, stale, and broad exception paths", () => {
    const issues = validateRunnerCensus(check({
      census: {
        ...census,
        fixtureExceptions: [
          { path: "src/*.test.ts", reason: "broad", owner: "fixture" },
          { path: "src/*.test.ts", reason: "duplicate", owner: "fixture" },
          { path: "src/stale.test.ts", reason: "stale", owner: "fixture" },
        ],
      },
    }));
    expect(issues).toEqual(expect.arrayContaining([
      "duplicate fixture exception: src/*.test.ts",
      "invalid fixture exception path: src/*.test.ts",
      "stale fixture exception: src/stale.test.ts",
    ]));
  });

  it.each([null, "not-an-entry", { path: "src/a.test.ts" }])("returns schema issues for malformed nonstandard exceptions: %j", (entry) => {
    expect(() => validateRunnerCensus(check({
      census: { ...census, nonstandardTests: [entry] },
    }))).not.toThrow();
    expect(validateRunnerCensus(check({ census: { ...census, nonstandardTests: [entry] } }))).toEqual(
      expect.arrayContaining([expect.stringMatching(/^invalid nonstandard exception/)]),
    );
  });

  it.each([
    ["vitestOwners", null, "invalid Vitest owner"],
    ["vitestOwners", "not-an-owner", "invalid Vitest owner"],
    ["playwrightOwners", null, "invalid Playwright owner"],
    ["playwrightOwners", "not-an-owner", "invalid Playwright owner"],
  ])("returns schema issues for malformed %s entries", (ownerKey, entry, issue) => {
    const malformed = { ...census, [ownerKey]: [entry] };
    expect(() => validateRunnerCensus(check({ census: malformed }))).not.toThrow();
    expect(validateRunnerCensus(check({ census: malformed }))).toEqual(expect.arrayContaining([issue]));
  });

  it("reports unknown census and exception fields", () => {
    expect(validateCensusSchema({ ...census, unexpected: true })).toEqual(["unknown runner census field: unexpected"]);
    expect(validateRunnerCensus(check({
      census: { ...census, fixtureExceptions: [{ path: "src/a.test.ts", reason: "fixture", owner: "fixture", extra: true }] },
    }))).toEqual(expect.arrayContaining(["unknown fixture exception field: extra"]));
  });
});

describe("Telegram Cargo test identity ownership", () => {
  const authority = {
    preNewApp: ["app::tests::owned"],
    phase8BNewApp: [],
    preNewStaged: ["telegram_impl::session::tests::owned"],
    phase8BNewStaged: [],
  };
  const passingLists = {
    extractum: { exitCode: 0, stdout: "app::tests::owned: test\n1 test, 0 benchmarks\n" },
    "extractum-telegram": { exitCode: 0, stdout: "session::tests::owned: test\n1 test, 0 benchmarks\n" },
  };
  const passingIgnoredLists = {
    extractum: { exitCode: 0, stdout: "0 tests, 0 benchmarks\n" },
    "extractum-telegram": { exitCode: 0, stdout: "0 tests, 0 benchmarks\n" },
  };
  const verifySteps = [{ command: "cargo", args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets"] }];

  it("accepts exact declared identities under their terminal Cargo owners", () => {
    expect(evaluateTelegramCargoTestIdentityOwnership({ authority, listResults: passingLists, ignoredListResults: passingIgnoredLists, verifySteps })).toEqual([]);
  });

  it("fails closed on missing, duplicate, wrong-package, failed-list, and missing-owner evidence", () => {
    const missing = structuredClone(passingLists);
    missing.extractum.stdout = "0 tests, 0 benchmarks\n";
    expect(evaluateTelegramCargoTestIdentityOwnership({ authority, listResults: missing, ignoredListResults: passingIgnoredLists, verifySteps })).toEqual(
      expect.arrayContaining([expect.stringMatching(/missing.*app::tests::owned/i)]),
    );

    const duplicateAndWrong = structuredClone(passingLists);
    duplicateAndWrong["extractum-telegram"].stdout += "session::tests::owned: test\napp::tests::owned: test\n";
    expect(evaluateTelegramCargoTestIdentityOwnership({ authority, listResults: duplicateAndWrong, ignoredListResults: passingIgnoredLists, verifySteps })).toEqual(
      expect.arrayContaining([expect.stringMatching(/duplicate.*session::tests::owned/i), expect.stringMatching(/wrong package.*app::tests::owned/i)]),
    );

    const failed = structuredClone(passingLists);
    failed.extractum.exitCode = 101;
    expect(evaluateTelegramCargoTestIdentityOwnership({ authority, listResults: failed, ignoredListResults: passingIgnoredLists, verifySteps })).toEqual(
      expect.arrayContaining([expect.stringMatching(/extractum.*failed/i)]),
    );
    expect(evaluateTelegramCargoTestIdentityOwnership({ authority, listResults: passingLists, ignoredListResults: passingIgnoredLists, verifySteps: [] })).toEqual(
      expect.arrayContaining([expect.stringMatching(/verify owner.*extractum/i), expect.stringMatching(/verify owner.*extractum-telegram/i)]),
    );
  });

  it("rejects a declared staged identity that remains under the app package", () => {
    const leaked = structuredClone(passingLists);
    leaked.extractum.stdout += "telegram_impl::session::tests::owned: test\n";

    expect(evaluateTelegramCargoTestIdentityOwnership({ authority, listResults: leaked, ignoredListResults: passingIgnoredLists, verifySteps })).toEqual(
      expect.arrayContaining([expect.stringMatching(/extractum.*telegram_impl::session::tests::owned/i)]),
    );
  });

  it("lists each referenced Cargo package at most once and skips unreferenced lists", () => {
    const runCargoList = vi.fn((packageName: string, options?: { ignoredOnly?: boolean }) => options?.ignoredOnly
      ? passingIgnoredLists[packageName as keyof typeof passingIgnoredLists]
      : passingLists[packageName as keyof typeof passingLists]);
    const ledger = { rows: [{ replacementIds: [
      "test:cargo:extractum::app::tests::owned",
      "test:cargo:extractum::app::tests::owned",
      "tool:telegram-cargo-test-identity-ownership",
    ] }] };

    const evidence = collectTelegramCargoReplacementEvidence({ ledger, authority, verifySteps, runCargoList });

    expect(runCargoList.mock.calls).toEqual([
      ["extractum", { ignoredOnly: false }],
      ["extractum", { ignoredOnly: true }],
      ["extractum-telegram", { ignoredOnly: false }],
      ["extractum-telegram", { ignoredOnly: true }],
    ]);
    expect(evidence.resolvedReplacementIds).toEqual(new Set([
      "test:cargo:extractum::app::tests::owned",
      "tool:telegram-cargo-test-identity-ownership",
    ]));
    const unused = vi.fn();
    expect(collectTelegramCargoReplacementEvidence({ ledger: { rows: [] }, authority, verifySteps, runCargoList: unused }))
      .toMatchObject({ issues: [], resolvedReplacementIds: new Set() });
    expect(unused).not.toHaveBeenCalled();
  });

  it("lists and resolves referenced Cargo packages outside the Telegram pair", () => {
    const runCargoList = vi.fn((packageName: string, options?: { ignoredOnly?: boolean }) => options?.ignoredOnly
      ? { exitCode: 0, stdout: "0 tests, 0 benchmarks\n" }
      : packageName === "extractum-analysis"
      ? { exitCode: 0, stdout: "state::tests::owned: test\n1 test, 0 benchmarks\n" }
      : passingLists[packageName as keyof typeof passingLists]);
    const ledger = { rows: [{ replacementIds: [
      "test:cargo:extractum-analysis::state::tests::owned",
    ] }] };

    const evidence = collectTelegramCargoReplacementEvidence({ ledger, authority, verifySteps, runCargoList });

    expect(runCargoList.mock.calls).toEqual([
      ["extractum-analysis", { ignoredOnly: false }],
      ["extractum-analysis", { ignoredOnly: true }],
    ]);
    expect(evidence.issues).toEqual([]);
    expect(evidence.resolvedReplacementIds).toEqual(new Set([
      "test:cargo:extractum-analysis::state::tests::owned",
    ]));
  });

  it("does not resolve a Cargo replacement when the exact test is ignored", () => {
    const runCargoList = vi.fn((_packageName: string, options?: { ignoredOnly?: boolean }) => ({
      exitCode: 0,
      stdout: options?.ignoredOnly
        ? "state::tests::ignored_owner: test\n1 test, 0 benchmarks\n"
        : "state::tests::ignored_owner: test\n1 test, 0 benchmarks\n",
    }));
    const ledger = { rows: [{ replacementIds: [
      "test:cargo:extractum-analysis::state::tests::ignored_owner",
    ] }] };

    const evidence = collectTelegramCargoReplacementEvidence({ ledger, authority, verifySteps, runCargoList });

    expect(evidence.resolvedReplacementIds).toEqual(new Set());
    expect(evidence.issues).toContain("extractum-analysis: ignored replacement identity state::tests::ignored_owner");
  });

  it("closes a transition-only tool replacement only with resolved Cargo identity evidence", () => {
    const historical = {
      id: "SC-000001",
      path: "src/deleted.test.ts",
      title: "Cargo test identity replacement",
      sourceHash: "a".repeat(64),
      assertionCount: 1,
      lineage: [],
      invariant: "The declared Cargo test identity remains under its terminal package.",
      disposition: "tool_owned",
      replacementIds: ["tool:telegram-cargo-test-identity-ownership"],
    };
    const context = {
      ledger: { schemaVersion: 1, frozenAtCommit: "f".repeat(40), sourceReaderExceptions: [], rows: [historical] },
      declarationInventory: [],
      sourceReaders: [],
    };

    expect(validateSourceContractLedger({
      ...context,
      resolvedReplacementIds: new Set(["tool:telegram-cargo-test-identity-ownership"]),
    }).rows).toEqual([{ id: "SC-000001", state: "closed" }]);
    expect(validateSourceContractLedger(context).rows).toEqual([{ id: "SC-000001", state: "open" }]);
  });
});

describe("live source-contract test discovery", () => {
  it("collects the normalized union of tracked tests and every live runner-owned path", () => {
    const readSource = vi.fn(() => "test source");

    expect(collectTrackedTestSources({
      root,
      tracked: ["src/current.test.ts", "src/unapproved.spec.ts", "research/adapter/tests/e2e.spec.ts"],
      vitestFiles: { "vitest:root": ["src/current.test.ts", "scripts\\untracked-vitest.spec.ts"] },
      playwrightFiles: { "playwright:adapter": ["research/adapter/tests/e2e.spec.ts", "research\\untracked-e2e.spec.ts"] },
      readSource,
    })).toEqual({
      tests: [
        { path: "research/adapter/tests/e2e.spec.ts", source: "test source" },
        { path: "research/untracked-e2e.spec.ts", source: "test source" },
        { path: "scripts/untracked-vitest.spec.ts", source: "test source" },
        { path: "src/current.test.ts", source: "test source" },
        { path: "src/unapproved.spec.ts", source: "test source" },
      ],
      issues: [],
    });
  });

  it("allows a missing tracked test only when the ledger owns its transition", () => {
    const readSource = vi.fn((candidate: string) => {
      if (candidate.endsWith("deleted.test.ts")) {
        const error = new Error("missing") as NodeJS.ErrnoException;
        error.code = "ENOENT";
        throw error;
      }
      return "test source";
    });

    expect(collectTrackedTestSources({
      root,
      tracked: ["src/current.test.ts", "src/deleted.test.ts", "src/not-a-test.ts"],
      authorizedMissingPaths: new Set(["src/deleted.test.ts"]),
      readSource,
    })).toEqual({
      tests: [{ path: "src/current.test.ts", source: "test source" }],
      issues: [],
    });
  });

  it("reports a missing tracked test that has no ledger ownership", () => {
    const readSource = vi.fn(() => {
      const error = new Error("missing") as NodeJS.ErrnoException;
      error.code = "ENOENT";
      throw error;
    });

    expect(collectTrackedTestSources({
      root,
      tracked: ["src/accidentally-deleted.test.ts"],
      authorizedMissingPaths: new Set(["src/ledger-owned.test.ts"]),
      readSource,
    })).toEqual({
      tests: [],
      issues: ["missing tracked test without ledger ownership: src/accidentally-deleted.test.ts"],
    });
  });

  it("bridges collected Vitest ownership into replacement evidence", () => {
    expect(createLedgerLiveCensus({
      census,
      runnerResult: {
        vitestFiles: { "vitest:root": ["src/replacement.test.ts"] },
        playwrightFiles: { "playwright:adapter": ["research/adapter/tests/e2e.spec.ts"] },
      },
    })).toEqual({
      vitestOwners: census.vitestOwners,
      vitestFiles: { "vitest:root": ["src/replacement.test.ts"] },
      playwrightOwners: census.playwrightOwners,
      playwrightFiles: { "playwright:adapter": ["research/adapter/tests/e2e.spec.ts"] },
    });
  });

  it.each([
    { name: "one exact declaration and owner", expected: true },
    { name: "missing declaration", declarations: [], expected: false },
    { name: "wrong declaration path", declarations: [{ path: "e2e/other.spec.ts", title: "responsive behavior", eligibility: "eligible" }], expected: false },
    { name: "wrong declaration title", declarations: [{ path: "e2e/responsive.spec.ts", title: "other behavior", eligibility: "eligible" }], expected: false },
    { name: "ineligible declaration", declarations: [{ path: "e2e/responsive.spec.ts", title: "responsive behavior", eligibility: "ineligible" }], expected: false },
    { name: "duplicate declaration", declarations: [
      { path: "e2e/responsive.spec.ts", title: "responsive behavior", eligibility: "eligible" },
      { path: "e2e/responsive.spec.ts", title: "responsive behavior", eligibility: "eligible" },
    ], expected: false },
    { name: "missing owner", owners: [], files: {}, expected: false },
    { name: "duplicate owner", owners: [
      { id: "playwright:app-e2e", ownerScript: "test:app:e2e" },
      { id: "playwright:duplicate", ownerScript: "test:duplicate:e2e" },
    ], files: {
      "playwright:app-e2e": ["e2e/responsive.spec.ts"],
      "playwright:duplicate": ["e2e/responsive.spec.ts"],
    }, verifySteps: [{ npmScript: "test:app:e2e" }, { npmScript: "test:duplicate:e2e" }], expected: false },
    { name: "ungated owner", verifySteps: [], expected: false },
  ])("resolves Playwright replacement evidence with exact cardinality: $name", ({
    declarations,
    owners,
    files,
    verifySteps,
    expected,
  }) => {
    const replacementId = "test:playwright:e2e/responsive.spec.ts#responsive behavior";
    const ledger = { rows: [{ replacementIds: [replacementId] }] };
    const declarationInventory = declarations ?? [{
      path: "e2e/responsive.spec.ts",
      title: "responsive behavior",
      eligibility: "eligible",
    }];
    const liveCensus = {
      playwrightOwners: owners ?? [{
        id: "playwright:app-e2e",
        ownerScript: "test:app:e2e",
      }],
      playwrightFiles: files ?? { "playwright:app-e2e": ["e2e/responsive.spec.ts"] },
    };

    expect(collectPlaywrightReplacementEvidence({
      ledger,
      declarationInventory,
      liveCensus,
      verifySteps: verifySteps ?? [{ npmScript: "test:app:e2e" }],
    })).toEqual(expected ? new Set([replacementId]) : new Set());
  });
});

describe("filesystem and runner collection", () => {
  const stats = ({ file = true, symlink = false } = {}) => ({ isFile: () => file, isSymbolicLink: () => symlink });

  it("discovers only existing candidate files from the required nul-delimited Git command", async () => {
    const runGit = vi.fn(async () => ({ exitCode: 0, stdout: "src/a.test.ts\0src/b.spec.mtsx\0src/readme.ts\0deleted.test.ts\0" }));
    const found = await discoverFilesystemCandidates(root, runGit, (candidate) => {
      if (candidate.endsWith("deleted.test.ts")) {
        const error = new Error("missing") as NodeJS.ErrnoException;
        error.code = "ENOENT";
        throw error;
      }
      return stats();
    });
    expect(runGit).toHaveBeenCalledWith(["ls-files", "--cached", "--others", "--exclude-standard", "-z"], root);
    expect(found).toEqual({ files: ["src/a.test.ts", "src/b.spec.mtsx"], issues: [] });
  });

  it("reports Git failure and supported symlink candidates instead of silently omitting them", async () => {
    await expect(discoverFilesystemCandidates(root, async () => ({ exitCode: 2, stdout: "" }))).resolves.toEqual({ files: [], issues: ["git ls-files failed"] });
    await expect(discoverFilesystemCandidates(root, async () => ({ exitCode: 0, stdout: "src/link.test.ts\0" }), () => stats({ file: false, symlink: true })))
      .resolves.toEqual({ files: [], issues: ["unsupported candidate symlink: src/link.test.ts"] });
  });

  it("normalizes Windows and repository-relative paths while rejecting escapes", () => {
    expect(normalizeRepoPath("C:\\repo", "C:\\repo\\src\\a.test.ts")).toEqual({ path: "src/a.test.ts" });
    expect(normalizeRepoPath(root, "/outside/a.test.ts")).toEqual({ issue: "repository escape: /outside/a.test.ts" });
    expect(normalizeRepoPath(root, "../outside/a.test.ts")).toEqual({ issue: "repository escape: ../outside/a.test.ts" });
  });

  it("collects Vitest paths and reports non-zero and spawn errors", async () => {
    const owner = census.vitestOwners[0];
    const good = await collectVitestFiles(owner, { repoRoot: root, runCommand: vi.fn(async () => ({ exitCode: 0, stdout: "src/a.test.ts\r\nsrc/b.spec.ts\n" })) });
    expect(good.files).toEqual(["src/a.test.ts", "src/b.spec.ts"]);
    await expect(collectVitestFiles(owner, { repoRoot: root, runCommand: async () => ({ exitCode: 1, stdout: "", error: new Error("spawn") }) }))
      .resolves.toMatchObject({ issues: ["runner spawn error: vitest:root: spawn", "runner failed: vitest:root"] });
  });

  it("parses Playwright suites and rejects malformed JSON and any errors value", async () => {
    const owner = { ...census.playwrightOwners[0], config: "research/gemini_browser_adapter/playwright.config.ts" };
    const invoke = async (stdout: string) => collectPlaywrightFiles(owner, {
      repoRoot: process.cwd(),
      resolveCli: () => "playwright-cli.mjs",
      runCommand: async () => ({ exitCode: 0, stdout }),
    });
    await expect(invoke(JSON.stringify({ config: { rootDir: "research/gemini_browser_adapter/tests" }, suites: [{ suites: [{ file: "nested.spec.ts", specs: [] }], specs: [{ file: "root.spec.ts" }] }] })))
      .resolves.toMatchObject({ files: ["research/gemini_browser_adapter/tests/nested.spec.ts", "research/gemini_browser_adapter/tests/root.spec.ts"], issues: [] });
    await expect(invoke(JSON.stringify({ config: { rootDir: "research/gemini_browser_adapter/tests" }, suites: [], errors: [] }))).resolves.toMatchObject({ files: [], issues: [] });
    await expect(invoke("not-json")).resolves.toMatchObject({ issues: ["playwright:adapter: malformed Playwright JSON"] });
    await expect(invoke(JSON.stringify({ suites: [], errors: { message: "bad" } }))).resolves.toMatchObject({ issues: ["playwright:adapter: Playwright errors are not empty"] });
    await expect(invoke(JSON.stringify({ suites: [], errors: ["bad"] }))).resolves.toMatchObject({ issues: ["playwright:adapter: Playwright errors are not empty"] });
  });

  it("resolves Playwright suite files from the JSON report rootDir without parsing a computed config testDir", async () => {
    const owner = { ...census.playwrightOwners[0], config: "research/adapter/playwright.config.ts" };
    const report = {
      config: { rootDir: "research/adapter/computed-test-root" },
      suites: [{ file: "nested/suite.spec.ts", specs: [{ file: "root.spec.ts" }] }],
    };
    const result = await collectPlaywrightFiles(owner, {
      repoRoot: root,
      resolveCli: () => "playwright-cli.mjs",
      runCommand: async () => ({ exitCode: 0, stdout: JSON.stringify(report) }),
    });

    expect(result).toEqual({
      files: ["research/adapter/computed-test-root/nested/suite.spec.ts", "research/adapter/computed-test-root/root.spec.ts"],
      issues: [],
    });
  });

  it.each([
    [{ suites: [] }, "missing Playwright config.rootDir"],
    [{ config: { rootDir: 7 }, suites: [] }, "invalid Playwright config.rootDir"],
    [{ config: { rootDir: "../outside" }, suites: [] }, "Playwright config.rootDir escapes repository: ../outside"],
  ])("fails closed for invalid Playwright JSON rootDir", async (report, issue) => {
    const owner = { ...census.playwrightOwners[0], config: "research/adapter/playwright.config.ts" };
    const result = await collectPlaywrightFiles(owner, {
      repoRoot: root,
      resolveCli: () => "playwright-cli.mjs",
      runCommand: async () => ({ exitCode: 0, stdout: JSON.stringify(report) }),
    });

    expect(result).toEqual({ files: [], issues: [`playwright:adapter: ${issue}`] });
  });
});

describe("transition carrier", () => {
  it("runs injected checks in order, prints issues, and returns non-zero", async () => {
    const calls: string[] = [];
    const stdout = { write: vi.fn() };
    const stderr = { write: vi.fn() };
    const result = await runTransitionValidation({
      repoRoot: root,
      checks: [
        async () => { calls.push("one"); return []; },
        async () => { calls.push("two"); return { issues: ["second issue"], summary: "Runner census: injected" }; },
      ],
      stdout,
      stderr,
    });

    expect(calls).toEqual(["one", "two"]);
    expect(result.exitCode).toBe(1);
    expect(stderr.write).toHaveBeenCalledWith("second issue\n");
    expect(stdout.write).toHaveBeenCalledWith(expect.stringContaining("Runner census:"));
  });
});

describe("bounded source-contract ledger", () => {
  const sha = (value: string) => createHash("sha256").update(value.replace(/\r\n?/g, "\n")).digest("hex");
  const gitMetadata = (
    trackedPaths: string[],
    pathKinds: Record<string, string> = {},
    ignoredPaths: string[] = [],
  ) => ({ trackedPaths: new Set(trackedPaths), pathKinds: new Map(Object.entries(pathKinds)), ignoredPaths: new Set(ignoredPaths) });
  const envelope = (rows: unknown[], sourceReaderExceptions: unknown[] = []) => ({
    schemaVersion: 1,
    frozenAtCommit: "a".repeat(40),
    sourceReaderExceptions,
    rows,
  });
  const simpleRow = (declaration: any, overrides: Record<string, unknown> = {}) => ({
    id: "SC-000001",
    path: declaration.path,
    title: declaration.title,
    sourceHash: sha(declaration.sourceSlice),
    assertionCount: declaration.assertionOrdinals.length,
    ...(declaration.authorityText ? { authorityHash: sha(declaration.authorityText) } : {}),
    lineage: [],
    invariant: "Keep the source-backed invariant.",
    disposition: "behavior",
    replacementIds: ["test:vitest:src/replacement.test.ts#replacement"],
    ...overrides,
  });

  it("builds a complete static declaration inventory with nested titles, both .each forms, and Node assert ordinals", () => {
    const declarations = discoverTestDeclarations([{
      path: "src/inventory.test.ts",
      source: [
        'import assert from "node:assert/strict";',
        'const cases = [["one", 1]] as const;',
        'describe("outer", () => {',
        '  it.each(cases)("named %s", (_name, value) => { expect(value).toBe(1); });',
        '  test.each([["two", 2]])("inline %s", (_name, value) => { assert(value === 2); });',
        '});',
      ].join("\r\n"),
    }], ts);

    expect(declarations.map((entry: any) => entry.title)).toEqual(["outer > named %s", "outer > inline %s"]);
    expect(declarations.map((entry: any) => entry.assertionOrdinals)).toEqual([[1], [1]]);
    expect(declarations[0].eachAuthorityText).toBe('const cases = [["one", 1]] as const;');
    expect(declarations[1].eachAuthorityText).toBe('[["two", 2]]');
    expect(declarations.every((entry: any) => !entry.sourceSlice.includes("\r"))).toBe(true);
  });

  it("turns factory declarations, computed titles, dynamic .each, and ambiguous bindings into exact manual requirements", () => {
    const source = [
      'const title = "computed";',
      'const cases = getCases();',
      'const source = "one";',
      'const source = "two";',
      'it(title, () => expect(source).toBe("one"));',
      'it.each(cases)("case", () => expect(1).toBe(1));',
      'it("factory", makeAssertion());',
    ].join("\n");
    const declarations = discoverTestDeclarations([{ path: "src/manual.test.ts", source }], ts);

    expect(declarations).toHaveLength(0);
    expect(declarations.manualRequirements).toEqual([
      expect.objectContaining({ path: "src/manual.test.ts", sourceRange: "5:1-5:44", reason: expect.stringMatching(/computed test title|ambiguous lexical binding/) }),
      expect.objectContaining({ path: "src/manual.test.ts", sourceRange: "6:1-6:48", reason: "unresolved dynamic .each table" }),
      expect.objectContaining({ path: "src/manual.test.ts", sourceRange: "7:1-7:31", reason: "dynamic test factory" }),
    ]);
    expect(declarations.manualRequirements.every((entry: any) => entry.sourceSlice && entry.assertionOrdinals && entry.sourceOffset >= 0)).toBe(true);
  });

  it("seeds only exact source-reading helper exports for extensionless named and namespace imports", () => {
    const files = [{ path: "src/helper.test.ts", source: [
      'import { readAnalysisContractSource as readAnalysis, normalizeAnalysisContractSourceText } from "./analysis-contract-paths";',
      'import * as telegram from "./telegram-contract-paths";',
      'import { readPromptPackDomainSource, promptPackCrateExtracted } from "./prompt-pack-contract-paths";',
      'const analysis = readAnalysis({ before: "a.rs", after: { owner: "app", path: "b.rs" } });',
      'const telegramSource = telegram.readTelegramContractFile("src/live.ts");',
      'const prompt = readPromptPackDomainSource("lib.rs");',
      'it("analysis", () => expect(analysis).toContain("x"));',
      'it("telegram", () => expect(telegramSource).toContain("x"));',
      'it("prompt", () => expect(prompt).toContain("x"));',
      'it("non-readers", () => expect([normalizeAnalysisContractSourceText("x"), telegram.phase8BCheckpointNumber("8b-preparation"), promptPackCrateExtracted]).toBeDefined());',
    ].join("\n") }];
    const readers = discoverSourceReaders(files, gitMetadata([]), ts);
    const inventory = discoverTestDeclarations(files, ts);
    const draft = buildLedgerDraft({ declarationInventory: inventory, sourceReaders: readers, frozenAtCommit: "b".repeat(40) });

    expect(readers.filter((reader: any) => reader.kind === "contract-path-helper").map((reader: any) => reader.exportName).sort()).toEqual([
      "readAnalysisContractSource", "readPromptPackDomainSource", "readTelegramContractFile",
    ]);
    expect(draft.rows.map((row: any) => row.title)).toEqual(["analysis", "telegram", "prompt"]);
  });

  it("ties named and namespace node:fs calls to their imports and never seeds local same-name functions", () => {
    const files = [{ path: "src/fs.test.ts", source: [
      'import { readFileSync as readSource } from "node:fs";',
      'import * as fs from "node:fs";',
      'const one = readSource("src/one.ts", "utf8");',
      'const two = fs.readFileSync("src/two.ts", "utf8");',
      'function readFileSync(_path: string) { return "local"; }',
      'const local = readFileSync("src/local.ts");',
      'it("one", () => expect(one).toContain("x"));',
      'it("two", () => expect(two).toContain("x"));',
      'it("local", () => expect(local).toContain("x"));',
    ].join("\n") }];
    const readers = discoverSourceReaders(files, gitMetadata(["src/one.ts", "src/two.ts", "src/local.ts"]), ts);

    expect(readers.map((reader: any) => reader.authorityPath)).toEqual(["src/one.ts", "src/two.ts"]);
    const draft = buildLedgerDraft({ declarationInventory: discoverTestDeclarations(files, ts), sourceReaders: readers, frozenAtCommit: "c".repeat(40) });
    expect(draft.rows.map((row: any) => row.title)).toEqual(["one", "two"]);
  });

  it("discovers CommonJS namespace, destructured, aliased, and dynamic node:fs reader flows", () => {
    const files = [{ path: "src/commonjs-fs.test.ts", source: [
      'let uninitialized;',
      'const fs = require("node:fs");',
      'const { readFileSync } = require("node:fs");',
      'const { readFile: readAsync } = require("node:fs/promises");',
      'const { [readerName]: destructuredDynamic } = require("node:fs");',
      'const readAlias = fs.readFileSync;',
      'const dynamicReader = fs[readerName];',
      'const one = fs.readFileSync("src/one.ts", "utf8");',
      'const two = readFileSync("src/two.ts", "utf8");',
      'const three = await readAsync("src/three.ts", "utf8");',
      'const four = readAlias("src/four.ts", "utf8");',
      'const unknown = dynamicReader(dynamicPath, "utf8");',
      'const five = require("node:fs")["readFileSync"]("src/five.ts", "utf8");',
      'const directUnknown = require("node:fs")[readerName](dynamicPath, "utf8");',
      'const destructuredUnknown = destructuredDynamic(dynamicPath, "utf8");',
      'it("reads", () => expect([one, two, three, four, five, unknown, directUnknown, destructuredUnknown]).toBeDefined());',
    ].join("\n") }];

    const readers = discoverSourceReaders(files, gitMetadata([
      "src/one.ts", "src/two.ts", "src/three.ts", "src/four.ts", "src/five.ts",
    ]), ts);

    expect(readers.map((reader: any) => [reader.authorityPath, reader.kind, reader.reason])).toEqual([
      ["src/one.ts", "fs-read", undefined],
      ["src/two.ts", "fs-read", undefined],
      ["src/three.ts", "fs-read", undefined],
      ["src/four.ts", "fs-read", undefined],
      [undefined, "manual", "dynamic or unknown filesystem reader flow"],
      ["src/five.ts", "fs-read", undefined],
      [undefined, "manual", "dynamic or unknown filesystem reader flow"],
      [undefined, "manual", "dynamic or unknown filesystem reader flow"],
    ]);
  });

  it("uses lexical binding identity so a shadowed name does not propagate source authority", () => {
    const files = [{ path: "src/shadow.test.ts", source: [
      'import source from "./live.ts?raw";',
      'it("local", () => { const source = "local"; expect(source).toBe("local"); });',
      'it("external", () => expect(source).toContain("export"));',
    ].join("\n") }];
    const readers = discoverSourceReaders(files, gitMetadata(["src/live.ts"]), ts);
    const draft = buildLedgerDraft({ declarationInventory: discoverTestDeclarations(files, ts), sourceReaders: readers, frozenAtCommit: "d".repeat(40) });
    expect(draft.rows.map((row: any) => row.title)).toEqual(["external"]);
  });

  it("emits exact manual readers for dynamic fs, untracked raw imports, unknown wrappers, and unresolved raw globs", () => {
    const files = [{ path: "src/unknown.test.ts", source: [
      'import { readFileSync } from "node:fs";',
      'import missing from "./missing.ts?raw";',
      'import { readSourceFile } from "./unknown-reader";',
      'const dynamic = readFileSync(dynamicPath, "utf8");',
      'const wrapped = readSourceFile("src/live.ts");',
      'const modules = import.meta.glob(pattern, { query: "?raw" });',
      'it("unknown", () => expect([missing, dynamic, wrapped, modules]).toBeDefined());',
    ].join("\n") }];
    const readers = discoverSourceReaders(files, gitMetadata(["src/live.ts"]), ts);
    expect(readers.filter((reader: any) => reader.kind === "manual")).toEqual([
      expect.objectContaining({ sourceRange: "2:1-2:40", reason: "untracked raw import" }),
      expect.objectContaining({ sourceRange: "4:17-4:50", reason: "dynamic or unknown filesystem authority" }),
      expect.objectContaining({ sourceRange: "5:17-5:46", reason: "unknown source-reader wrapper" }),
      expect.objectContaining({ sourceRange: "6:17-6:61", reason: "dynamic raw glob" }),
    ]);
  });

  it("resolves brace alternatives in static raw globs without creating a manual reader", () => {
    const files = [{
      path: "src/glob-reader.test.ts",
      source: 'const modules = import.meta.glob("./parts/*.{test,spec}.{ts,tsx}", { query: "?raw" });',
    }];
    const readers = discoverSourceReaders(files, gitMetadata([
      "src/parts/alpha.test.ts",
      "src/parts/beta.spec.tsx",
      "src/parts/ignored.test.js",
    ]), ts);

    expect(readers).toEqual([
      expect.objectContaining({ kind: "import-meta-glob", authorityPath: "src/parts/alpha.test.ts", classification: "test" }),
      expect.objectContaining({ kind: "import-meta-glob", authorityPath: "src/parts/beta.spec.tsx", classification: "test" }),
    ]);
    expect(readers.some((reader: any) => reader.kind === "manual")).toBe(false);
  });

  it("fails closed for malformed nested brace globs", () => {
    const files = [{
      path: "src/glob-reader.test.ts",
      source: 'const modules = import.meta.glob("./parts/{{alpha,beta}.test.ts", { query: "?raw" });',
    }];
    const readers = discoverSourceReaders(files, gitMetadata([
      "src/parts/beta.test.ts",
      "src/parts/{beta.test.ts",
    ]), ts);

    expect(readers).toEqual([
      expect.objectContaining({ kind: "manual", reason: "raw glob resolved no tracked or ignored authority" }),
    ]);
    expect(readers[0]).not.toHaveProperty("authorityPath");
  });

  it("hashes raw, fs, helper, glob, and .each authority text into obligated rows", () => {
    const files = [{ path: "src/authority.test.ts", source: [
      'import raw from "./raw.ts?raw";',
      'import { readFileSync } from "node:fs";',
      'const fsSource = readFileSync("src/fs.ts", "utf8");',
      'const modules = import.meta.glob("./parts/*.ts", { query: "?raw" });',
      'const cases = [["x"]] as const;',
      'it.each(cases)("authority %s", () => expect([raw, fsSource, modules]).toBeDefined());',
    ].join("\n") }];
    const metadata = gitMetadata(["src/raw.ts", "src/fs.ts", "src/parts/a.ts"]);
    const inventory = discoverTestDeclarations(files, ts);
    const readers = discoverSourceReaders(files, metadata, ts);
    const draft = buildLedgerDraft({ declarationInventory: inventory, sourceReaders: readers, frozenAtCommit: "e".repeat(40) });
    const authorityText = [
      'import raw from "./raw.ts?raw";',
      'readFileSync("src/fs.ts", "utf8")',
      'import.meta.glob("./parts/*.ts", { query: "?raw" })',
      'const cases = [["x"]] as const;',
    ].join("\n");
    expect(draft.rows[0].authorityHash).toBe(sha(authorityText));
    expect(draft.rows[0]).not.toHaveProperty("authorityTextForAudit");
  });

  it("classifies exact production, fixture, generated, ignored, output, test, and temp provenance from injected metadata", () => {
    const files = [{ path: "src/provenance.test.ts", source: [
      'import { mkdtempSync, readFileSync } from "node:fs";',
      'import { tmpdir } from "node:os";',
      'import path from "node:path";',
      'const tempRoot = mkdtempSync(path.join(tmpdir(), "audit-"));',
      'const production = readFileSync("src/live.ts", "utf8");',
      'const fixture = readFileSync("samples/input.txt", "utf8");',
      'const generated = readFileSync("cache/generated.txt", "utf8");',
      'const output = readFileSync("reports/result.json", "utf8");',
      'const ignored = readFileSync("artifacts/run.json", "utf8");',
      'const testSource = readFileSync("src/other.test.ts", "utf8");',
      'const temporary = readFileSync(path.join(tempRoot, "value.txt"), "utf8");',
    ].join("\n") }];
    const metadata = gitMetadata(
      ["src/live.ts", "samples/input.txt", "cache/generated.txt", "reports/result.json", "src/other.test.ts"],
      { "src/live.ts": "production", "samples/input.txt": "fixture", "cache/generated.txt": "generated", "reports/result.json": "output" },
      ["artifacts/run.json"],
    );
    const readers = discoverSourceReaders(files, metadata, ts);
    expect(readers.map((reader: any) => [reader.authorityPath, reader.classification])).toEqual([
      ["src/live.ts", "production"], ["samples/input.txt", "fixture"], ["cache/generated.txt", "generated"],
      ["reports/result.json", "output"], ["artifacts/run.json", "ignored"], ["src/other.test.ts", "test"],
      [undefined, "temp"],
    ]);
  });

  it("keeps full replacement inventory separate from the source-reader-derived obligated cohort", () => {
    const files = [{ path: "src/mixed.test.ts", source: [
      'import source from "./live.ts?raw";',
      'it("legacy", () => expect(source).toContain("export"));',
      'it("replacement", () => expect(1 + 1).toBe(2));',
    ].join("\n") }];
    const inventory = discoverTestDeclarations(files, ts);
    const readers = discoverSourceReaders(files, gitMetadata(["src/live.ts"]), ts);
    const draft = buildLedgerDraft({ declarationInventory: inventory, sourceReaders: readers, frozenAtCommit: "f".repeat(40) });
    expect(draft.rows.map((row: any) => row.title)).toEqual(["legacy"]);

    const historical = simpleRow(inventory[0], { path: "src/old.test.ts", title: "old", replacementIds: ["test:vitest:src/mixed.test.ts#replacement"] });
    const result = validateSourceContractLedger({
      ledger: envelope([historical]), declarationInventory: inventory, sourceReaders: [],
      liveCensus: { vitestOwners: [{ id: "vitest:root", ownerScript: "test" }], vitestFiles: { "vitest:root": ["src/mixed.test.ts"] } },
      verifySteps: [{ npmScript: "test" }],
    });
    expect(result.issues).toEqual([]);
    expect(result.rows).toEqual([{ id: "SC-000001", state: "closed" }]);
  });

  it("limits the CLI declaration inventory to paths proven by the freeze-time Vitest list", () => {
    expect(selectVitestTrackedPaths(
      ["src/unit.test.ts", "research/e2e.spec.ts", "src/not-a-test.ts"],
      { "src/unit.test.ts": ["unit"], "scripts/untracked.test.ts": ["other"] },
    )).toEqual(["src/unit.test.ts"]);
  });

  it("builds exact manual rows only with non-empty freeze-time runner titles and validates them against current requirements", () => {
    const files = [{ path: "src/manual-reader.test.ts", source: [
      'import { readFileSync } from "node:fs";',
      'const source = readFileSync(dynamicPath, "utf8");',
      'it("manual reader", () => expect(source).toContain("x"));',
    ].join("\n") }];
    const inventory = discoverTestDeclarations(files, ts);
    const readers = discoverSourceReaders(files, gitMetadata([]), ts);
    expect(() => buildLedgerDraft({ declarationInventory: inventory, sourceReaders: readers, frozenAtCommit: "1".repeat(40) })).toThrow(/runnerTitles/);

    const draft = buildLedgerDraft({
      declarationInventory: inventory,
      sourceReaders: readers,
      frozenAtCommit: "1".repeat(40),
      runnerTitlesByPath: { "src/manual-reader.test.ts": ["manual reader"] },
    });
    expect(draft.rows).toEqual([expect.objectContaining({
      manual: { sourceRange: "3:1-3:57", reason: "dynamic or unknown filesystem authority", runnerTitles: ["manual reader"] },
      sourceHash: sha('it("manual reader", () => expect(source).toContain("x"))'),
      assertionCount: 1,
    })]);
    const reviewed = { ...draft.rows[0], invariant: "Review the dynamic reader.", disposition: "delete", deletionReason: "The implementation-text assertion will be removed.", replacementIds: undefined };
    delete reviewed.replacementIds;
    expect(validateSourceContractLedger({ ledger: envelope([reviewed]), declarationInventory: inventory, sourceReaders: readers, runnerTitlesByPath: { "src/manual-reader.test.ts": ["manual reader"] } }).issues).toEqual([]);
  });

  it("requires draft output to remain under repo artifacts and be proven ignored", () => {
    const base = { declarationInventory: [], sourceReaders: [], frozenAtCommit: "2".repeat(40), repoRoot: "C:/repo" };
    expect(() => buildLedgerDraft({ ...base, outputPath: "../escape.json", isIgnoredPath: () => true })).toThrow(/inside artifacts/);
    expect(() => buildLedgerDraft({ ...base, outputPath: "artifacts/draft.json", isIgnoredPath: () => false })).toThrow(/Git-ignored/);
    expect(() => buildLedgerDraft({ ...base, outputPath: "artifacts/draft.json" })).toThrow(/isIgnoredPath/);
  });

  it("validates strict recursive schemas, exact unions, anchored namespaces, lineage, and nulls deterministically", () => {
    const badRows = [
      null,
      { id: "SC-000001", path: "src/x.test.ts", title: "x", manual: { sourceRange: "*", reason: "", runnerTitles: [], extra: true }, sourceHash: "bad", assertionCount: -1, lineage: ["../old.test.ts", "../old.test.ts"], invariant: "", disposition: "delete", replacementIds: ["junk test:vitest:x#y"], status: "pending" },
      { id: "SC-000002", path: "src/y.test.ts", title: "y", sourceHash: "b".repeat(64), assertionCount: 2, lineage: [], invariant: "mixed", disposition: "behavior", subgroups: [{ assertionOrdinals: [1, 1], invariant: "", disposition: "behavior", replacementIds: ["rule:"], extra: true }] },
    ];
    const ledger = { ...envelope(badRows, [{ path: "src/*.test.ts", sourceRange: "*", reason: "", owner: "", extra: true }]), extra: true };
    const first = validateSourceContractLedger({ ledger, declarationInventory: [], sourceReaders: [] });
    const second = validateSourceContractLedger({ ledger, declarationInventory: [], sourceReaders: [] });
    expect(() => validateSourceContractLedger({ ledger, declarationInventory: [], sourceReaders: [] })).not.toThrow();
    expect(first).toEqual(second);
    expect(first.issues).toEqual(expect.arrayContaining([
      expect.stringContaining("unknown ledger envelope field"), expect.stringContaining("invalid ledger row"),
      expect.stringContaining("invalid manual row"), expect.stringContaining("invalid lineage"),
      expect.stringContaining("mixed row has top-level resolution"), expect.stringContaining("overlapping subgroup assertion ordinal"),
      expect.stringContaining("unknown replacement namespace"), expect.stringContaining("invalid sourceReaderException"),
    ]));
  });

  it("requires exact necessary fixture exceptions and rejects broad, empty, duplicate, and stale entries", () => {
    const readers = [{ path: "src/x.test.ts", sourceRange: "2:16-2:60", sourceSlice: 'readFileSync("samples/a.txt", "utf8")', kind: "fs-read", classification: "fixture", authorityPath: "samples/a.txt" }];
    const exact = { path: "src/x.test.ts", sourceRange: "2:16-2:60", reason: "Reads the parser fixture.", owner: "parser fixture" };
    expect(validateSourceContractLedger({ ledger: envelope([], [exact]), declarationInventory: [], sourceReaders: readers }).issues).toEqual([]);
    const issues = validateSourceContractLedger({ ledger: envelope([], [exact, exact, { path: "src/*.test.ts", sourceRange: "*", reason: "", owner: "" }, { path: "src/stale.test.ts", sourceRange: "1:1-1:2", reason: "stale", owner: "owner" }]), declarationInventory: [], sourceReaders: readers }).issues;
    expect(issues).toEqual(expect.arrayContaining([
      expect.stringContaining("duplicate sourceReaderException"), expect.stringContaining("invalid sourceReaderException"), expect.stringContaining("stale sourceReaderException"),
    ]));
  });

  it("derives truthful open and closed states for present, deleted, unresolved, and mixed historical rows", () => {
    const current = { path: "src/current.test.ts", title: "current", eligibility: "eligible", sourceSlice: 'it("current", () => expect(1).toBe(1))', sourceOffset: 0, assertionOrdinals: [1], referencedBindingKeys: [], sourceRange: "1:1-1:42" };
    const replacement = { path: "src/replacement.test.ts", title: "replacement", eligibility: "eligible", sourceSlice: 'it("replacement", () => expect(1).toBe(1))', sourceOffset: 0, assertionOrdinals: [1], referencedBindingKeys: [], sourceRange: "1:1-1:50" };
    const rows = [
      simpleRow(current, { id: "SC-000001", replacementIds: ["test:vitest:src/replacement.test.ts#replacement"] }),
      simpleRow(current, { id: "SC-000002", path: "src/deleted.test.ts", title: "deleted", disposition: "delete", deletionReason: "Formatting-only assertion retired.", replacementIds: undefined }),
      { ...simpleRow(current, { id: "SC-000003", path: "src/mixed.test.ts", title: "mixed", assertionCount: 2 }), disposition: undefined, replacementIds: undefined, subgroups: [
        { assertionOrdinals: [1], invariant: "one", disposition: "behavior", replacementIds: ["test:vitest:src/replacement.test.ts#replacement"] },
        { assertionOrdinals: [2], invariant: "two", disposition: "delete", deletionReason: "Exact formatting assertion retired." },
      ] },
      simpleRow(current, { id: "SC-000004", path: "src/unresolved.test.ts", title: "unresolved", replacementIds: ["rule:future"] }),
    ].map((row: any) => Object.fromEntries(Object.entries(row).filter(([, value]) => value !== undefined)));
    const result = validateSourceContractLedger({
      ledger: envelope(rows), declarationInventory: [current, replacement], sourceReaders: [{ path: current.path, sourceRange: current.sourceRange, sourceOffset: 0, kind: "raw-import", classification: "production", bindingKey: "src/current.test.ts:binding:0", authorityText: "authority", dependentDeclarationKeys: [`${current.path}#${current.title}`] }],
      liveCensus: { vitestOwners: [{ id: "vitest:root", ownerScript: "test" }], vitestFiles: { "vitest:root": ["src/replacement.test.ts"] } }, verifySteps: [{ npmScript: "test" }],
    });
    expect(result.rows).toEqual([
      { id: "SC-000001", state: "open" }, { id: "SC-000002", state: "closed" },
      { id: "SC-000003", state: "closed" }, { id: "SC-000004", state: "open" },
    ]);
    expect(result.issues).toContain("SC-000004: unresolved historical row");
  });

  it("closes only unconditionally eligible Vitest replacement declarations", () => {
    const source = [
      'it("normal", () => expect(1).toBe(1));',
      'it.skip("skipped", () => expect(1).toBe(1));',
      'test.skip("test skipped", () => expect(1).toBe(1));',
      'describe.skip("skipped suite", () => { it("child", () => expect(1).toBe(1)); });',
      'it.runIf(false)("run false", () => expect(1).toBe(1));',
      'it.skipIf(true)("skip true", () => expect(1).toBe(1));',
      'it.runIf(runtimeFlag)("dynamic", () => expect(1).toBe(1));',
    ].join("\n");
    const inventory = discoverTestDeclarations([{ path: "src/eligibility.test.ts", source }], ts);
    expect(inventory.map((entry: any) => [entry.title, entry.eligibility])).toEqual([
      ["normal", "eligible"],
      ["skipped", "ineligible"],
      ["test skipped", "ineligible"],
      ["skipped suite > child", "ineligible"],
      ["run false", "ineligible"],
      ["skip true", "ineligible"],
      ["dynamic", "unknown"],
    ]);
    const titles = inventory.map((entry: any) => entry.title);
    const rows = titles.map((title: string, index: number) => ({
      id: `SC-${String(index + 1).padStart(6, "0")}`,
      path: `src/deleted-${index}.test.ts`,
      title: `deleted ${index}`,
      sourceHash: "a".repeat(64),
      assertionCount: 1,
      lineage: [],
      invariant: "Keep executable behavior.",
      disposition: "behavior",
      replacementIds: [`test:vitest:src/eligibility.test.ts#${title}`],
    }));
    const result = validateSourceContractLedger({
      ledger: envelope(rows),
      declarationInventory: inventory,
      sourceReaders: [],
      liveCensus: { vitestOwners: [{ id: "vitest:root", ownerScript: "test" }], vitestFiles: { "vitest:root": ["src/eligibility.test.ts"] } },
      verifySteps: [{ npmScript: "test" }],
    });

    expect(result.rows).toEqual([
      { id: "SC-000001", state: "closed" },
      ...rows.slice(1).map((row: any) => ({ id: row.id, state: "open" })),
    ]);
  });

  it("rejects duplicate replacement IDs and placeholder deletion reasons", () => {
    const replacement = discoverTestDeclarations([{
      path: "src/replacement.test.ts",
      source: 'it("replacement", () => expect(1).toBe(1));',
    }], ts)[0];
    const duplicate = simpleRow(replacement, {
      path: "src/deleted.test.ts",
      title: "deleted",
      replacementIds: [
        "test:vitest:src/replacement.test.ts#replacement",
        "test:vitest:src/replacement.test.ts#replacement",
      ],
    });
    const placeholder = simpleRow(replacement, {
      id: "SC-000002",
      path: "src/deleted-too.test.ts",
      title: "deleted too",
      disposition: "delete",
      deletionReason: "TODO",
      replacementIds: undefined,
    });
    delete placeholder.replacementIds;
    const result = validateSourceContractLedger({
      ledger: envelope([duplicate, placeholder]),
      declarationInventory: [replacement],
      sourceReaders: [],
      liveCensus: { vitestOwners: [{ id: "vitest:root", ownerScript: "test" }], vitestFiles: { "vitest:root": ["src/replacement.test.ts"] } },
      verifySteps: [{ npmScript: "test" }],
    });

    expect(result.issues).toEqual(expect.arrayContaining([
      "SC-000001: duplicate replacementId: test:vitest:src/replacement.test.ts#replacement",
      "SC-000002: delete requires a specific non-placeholder deletionReason",
    ]));
    expect(result.rows).toEqual([
      { id: "SC-000001", state: "open" },
      { id: "SC-000002", state: "open" },
    ]);
  });

  it("consolidates manual readers into one exact row per known declaration title", () => {
    const files = [{ path: "src/consolidated.test.ts", source: [
      'import { readFileSync } from "node:fs";',
      'const source = readFileSync(dynamicPath, "utf8");',
      'describe("suite", () => {',
      '  it("one", () => expect(source).toContain("one"));',
      '  it("two", () => expect(source).toContain("two"));',
      '});',
    ].join("\n") }];
    const inventory = discoverTestDeclarations(files, ts);
    const readers = discoverSourceReaders(files, gitMetadata([]), ts);
    const draft = buildLedgerDraft({
      declarationInventory: inventory, sourceReaders: readers, frozenAtCommit: "3".repeat(40),
      runnerTitlesByPath: { "src/consolidated.test.ts": ["suite > one", "suite > two"] },
    });
    expect(draft.rows).toHaveLength(2);
    expect(draft.rows.map((row: any) => row.manual.runnerTitles)).toEqual([["suite > one"], ["suite > two"]]);
    expect(draft.rows.map((row: any) => row.sourceHash)).toEqual(inventory.map((entry: any) => sha(entry.sourceSlice)));
  });

  it("reconciles static each templates to their exact expanded freeze-time titles", () => {
    const files = [{ path: "src/each-manual.test.ts", source: [
      'import { readFileSync } from "node:fs";',
      'const source = readFileSync(dynamicPath, "utf8");',
      'it("reads raw", () => expect(source).toContain("x"));',
      'it.each([{ kind: "a" }, { kind: "b" }])("reads $kind", ({ kind }) => expect([source, kind]).toBeDefined());',
    ].join("\n") }];
    const draft = buildLedgerDraft({
      declarationInventory: discoverTestDeclarations(files, ts),
      sourceReaders: discoverSourceReaders(files, gitMetadata([]), ts),
      frozenAtCommit: "9".repeat(40),
      runnerTitlesByPath: { "src/each-manual.test.ts": ["reads raw", "reads 'a'", "reads 'b'"] },
    });
    expect(draft.rows).toEqual(expect.arrayContaining([expect.objectContaining({
      manual: expect.objectContaining({ runnerTitles: ["reads 'a'", "reads 'b'"] }),
    })]));
    expect(draft.rows.flatMap((row: any) => row.manual.runnerTitles)).toEqual(["reads raw", "reads 'a'", "reads 'b'"]);
  });

  it("retains known manual titles and reconciles structurally unknown suites without reusing file-wide titles", () => {
    const files = [{ path: "src/reconcile.test.ts", source: [
      'import source from "./live.ts?raw";',
      'describe(makeTitle(), () => { it("inside", () => expect(source).toContain("x")); });',
      'it("outside", () => expect(source).toContain("x"));',
    ].join("\n") }];
    const inventory = discoverTestDeclarations(files, ts);
    const readers = discoverSourceReaders(files, gitMetadata(["src/live.ts"]), ts);
    const draft = buildLedgerDraft({
      declarationInventory: inventory, sourceReaders: readers, frozenAtCommit: "4".repeat(40),
      runnerTitlesByPath: { "src/reconcile.test.ts": ["dynamic > inside", "outside"] },
    });
    expect(draft.rows).toHaveLength(2);
    expect(draft.rows.find((row: any) => row.manual)?.manual.runnerTitles).toEqual(["dynamic > inside"]);
    expect(draft.rows.find((row: any) => row.title)?.title).toBe("outside");
  });

  it("makes registration functions and unsupported destructuring flows exact manual declarations", () => {
    const registration = discoverTestDeclarations([{ path: "src/register.test.ts", source: [
      'function register() { it("factory", () => expect(1).toBe(1)); }',
      'register();',
    ].join("\n") }], ts);
    expect(registration).toHaveLength(0);
    expect(registration.manualRequirements).toEqual([expect.objectContaining({
      sourceRange: "1:23-1:61", reason: "test declaration inside registration function", title: "factory",
    })]);

    const files = [{ path: "src/destructure.test.ts", source: [
      'import raw from "./live.ts?raw";',
      'const holder = { raw };',
      'const { raw: source } = holder;',
      'it("destructured", () => expect(source).toContain("x"));',
    ].join("\n") }];
    const inventory = discoverTestDeclarations(files, ts);
    expect(inventory).toHaveLength(0);
    expect(inventory.manualRequirements).toEqual([expect.objectContaining({
      reason: "unsupported destructuring or alias flow", title: "destructured",
    })]);
  });

  it("unwraps awaited mkdtemp and tmpdir provenance so proven temp reads create no obligations", () => {
    const files = [{ path: "src/async-temp.test.ts", source: [
      'import { mkdtemp, readFile } from "node:fs/promises";',
      'import { tmpdir } from "node:os";',
      'import path from "node:path";',
      'const root = await mkdtemp(path.join(tmpdir(), "audit-"));',
      'const text = await readFile(path.join(root, "value.txt"), "utf8");',
      'it("temp", () => expect(text).toContain("x"));',
    ].join("\n") }];
    const readers = discoverSourceReaders(files, gitMetadata([]), ts);
    expect(readers).toEqual([expect.objectContaining({ classification: "temp", kind: "fs-read" })]);
    expect(buildLedgerDraft({ declarationInventory: discoverTestDeclarations(files, ts), sourceReaders: readers, frozenAtCommit: "5".repeat(40) }).rows).toEqual([]);
  });

  it("validates exact metadata kinds and fails closed for unclassified directory entries", () => {
    const files = [{ path: "src/kinds.test.ts", source: [
      'import { readFileSync, readdirSync } from "node:fs";',
      'const config = readFileSync("config/app.json", "utf8");',
      'const docs = readFileSync("docs/guide.md", "utf8");',
      'const entries = readdirSync("src/generated");',
      'it("kinds", () => expect([config, docs, entries]).toBeDefined());',
    ].join("\n") }];
    const metadata = {
      trackedPaths: new Set(["config/app.json", "docs/guide.md", "src/generated/a.ts"]),
      pathKinds: new Map([["config/app.json", "configuration"], ["docs/guide.md", "documentation"]]),
      ignoredPaths: new Set(),
      directoryEntries: new Map([["src/kinds.test.ts:4:17-4:45", ["src/generated/a.ts"]]]),
    };
    const readers = discoverSourceReaders(files, metadata, ts);
    expect(readers).toEqual(expect.arrayContaining([
      expect.objectContaining({ classification: "configuration" }),
      expect.objectContaining({ classification: "documentation" }),
      expect.objectContaining({ kind: "manual", reason: "unclassified directory entry: src/generated/a.ts" }),
    ]));
    const draft = buildLedgerDraft({
      declarationInventory: discoverTestDeclarations(files, ts), sourceReaders: readers, frozenAtCommit: "6".repeat(40),
      runnerTitlesByPath: { "src/kinds.test.ts": ["kinds"] },
    });
    expect(draft.rows).toHaveLength(1);
    expect(() => discoverSourceReaders(files, { ...metadata, pathKinds: new Map([["config/app.json", "prodution"]]) }, ts)).toThrow(/invalid path kind/);
  });

  it("supplies bounded exact CLI fixture and directory provenance with non-stale exceptions", () => {
    const metadata = createCliGitMetadata(new Set([
      "src-tauri/src/analysis/test_schema.rs",
      "src-tauri/crates/extractum-analysis/src/test_schema.rs",
      "src-tauri/src/prompt_packs/lib.rs",
      "src-tauri/src/prompt_packs/runtime.rs",
    ]), new Set());
    expect(metadata.pathKinds.get("src-tauri/src/analysis/test_schema.rs")).toBe("fixture");
    expect(metadata.readerSites.size).toBeGreaterThan(0);
    expect(metadata.readerSites.get("research/gemini_browser_adapter/tests/failure-artifacts.spec.ts:32:32-32:86")).toEqual({
      authorities: [{ path: "research/gemini_browser_adapter/artifacts/test-timeout", classification: "output" }],
    });
    expect(metadata.readerSites.get("research/gemini_browser_adapter/tests/failure-artifacts.spec.ts:83:16-83:65")).toEqual({
      authorities: [{ path: "research/gemini_browser_adapter/artifacts/test-reduced", classification: "output" }],
    });
    expect(metadata.directoryEntries.get("src/lib/prompt-pack-application-contract.test.ts:10:29-10:84")).toEqual([
      "src-tauri/src/prompt_packs/lib.rs", "src-tauri/src/prompt_packs/runtime.rs",
    ]);
  });

  it("counts named Node assert imports and preserves public referencedSymbols", () => {
    const declarations = discoverTestDeclarations([{ path: "src/assert.test.ts", source: [
      'import { strictEqual } from "node:assert";',
      'const source = "value";',
      'it("asserts", () => strictEqual(source, "value"));',
    ].join("\n") }], ts);
    expect(declarations[0]).toMatchObject({ assertionOrdinals: [1], referencedSymbols: ["source", "strictEqual"] });
  });

  it("matches bounded recursive raw globs", () => {
    const files = [{ path: "src/glob.test.ts", source: [
      'const modules = import.meta.glob("./parts/**/*.ts", { query: "?raw" });',
      'it("glob", () => expect(modules).toBeDefined());',
    ].join("\n") }];
    const readers = discoverSourceReaders(files, gitMetadata(["src/parts/a.ts", "src/parts/a/b.ts", "src/parts/a/b.txt"]), ts);
    expect(readers.map((reader: any) => reader.authorityPath)).toEqual(["src/parts/a.ts", "src/parts/a/b.ts"]);
  });

  it("never reports a schema-invalid absent simple row as closed", () => {
    const row = {
      id: "SC-000001", path: "src/old.test.ts", title: "old", sourceHash: "a".repeat(64), assertionCount: 1,
      lineage: [], invariant: "invalid", disposition: "delete", deletionReason: "specific", extra: true,
    };
    const result = validateSourceContractLedger({ ledger: envelope([row]), declarationInventory: [], sourceReaders: [] });
    expect(result.issues).toContain("SC-000001: unknown ledger row field: extra");
    expect(result.rows).toEqual([{ id: "SC-000001", state: "open" }]);
  });

  it("never applies numeric reader containment across test-file boundaries", () => {
    const files = [
      { path: "src/a.test.ts", source: [
        'import { readFileSync } from "node:fs";',
        'const source = readFileSync(dynamicPath, "utf8");',
        'it("source owner", () => expect(source).toContain("x"));',
      ].join("\n") },
      { path: "src/b.test.ts", source: 'it("unrelated declaration with enough padding", () => expect("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").toContain("x"));' },
    ];
    const draft = buildLedgerDraft({
      declarationInventory: discoverTestDeclarations(files, ts),
      sourceReaders: discoverSourceReaders(files, gitMetadata([]), ts),
      runnerTitlesByPath: { "src/a.test.ts": ["source owner"], "src/b.test.ts": ["unrelated declaration with enough padding"] },
      frozenAtCommit: "b".repeat(40),
    });
    expect(draft.rows.map((row: any) => row.path)).toEqual(["src/a.test.ts"]);
  });

  it("discovers children of the Vitest suite options overload individually", () => {
    const declarations = discoverTestDeclarations([{ path: "src/options.test.ts", source: [
      'describe("suite", { timeout: 30_000 }, () => {',
      '  it("one", () => expect(1).toBe(1));',
      '  it("two", () => expect(2).toBe(2));',
      '});',
    ].join("\n") }], ts);
    expect(declarations.map((item: any) => item.title)).toEqual(["suite > one", "suite > two"]);
    expect(declarations.manualRequirements).toEqual([]);
  });

  it("discovers documented conditional test modifiers as individual declarations", () => {
    const declarations = discoverTestDeclarations([{ path: "src/conditional.test.ts", source:
      'it.runIf(process.platform === "win32")("conditional", () => expect(1).toBe(1));' }], ts);
    expect(declarations.map((item: any) => item.title)).toEqual(["conditional"]);
    expect(declarations.manualRequirements).toEqual([]);
  });

  it("makes variable-bound arrow and function-expression registration factories exact manual requirements", () => {
    const declarations = discoverTestDeclarations([{ path: "src/variable-register.test.ts", source: [
      'const registerArrow = () => { it("arrow factory", () => expect(1).toBe(1)); };',
      'const registerFunction = function () { it("function factory", () => expect(2).toBe(2)); };',
      'registerArrow();',
      'registerFunction();',
    ].join("\n") }], ts);
    expect(declarations).toHaveLength(0);
    expect(declarations.manualRequirements).toEqual([
      expect.objectContaining({ reason: "test declaration inside registration function", title: "arrow factory" }),
      expect.objectContaining({ reason: "test declaration inside registration function", title: "function factory" }),
    ]);
  });

  it("rejects duplicate manual path and runner-title ownership across rows", () => {
    const manualRow = (id: string, sourceRange: string) => ({
      id, path: "src/collision.test.ts", manual: { sourceRange, reason: "ambiguous each expansion", runnerTitles: ["case shared"] },
      sourceHash: "c".repeat(64), assertionCount: 1, lineage: [], invariant: "Review exact declaration.",
      disposition: "delete", deletionReason: "Remove the obsolete source assertion.",
    });
    const result = validateSourceContractLedger({
      ledger: envelope([manualRow("SC-000001", "1:1-1:10"), manualRow("SC-000002", "2:1-2:10")]),
      declarationInventory: [], sourceReaders: [],
    });
    expect(result.issues).toContain("duplicate manual runner-title ownership: src/collision.test.ts#case shared");
    expect(result.rows).toEqual([{ id: "SC-000001", state: "open" }, { id: "SC-000002", state: "open" }]);
  });
});
