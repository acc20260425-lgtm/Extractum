import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const REVIEW_BASE_COMMIT = "a54507d63420bb870c3870c91d7e22b050abae3e";
const PATH_PRESENT_CLOSED_ROW_IDS = ["SC-000355", "SC-000366"];
const MANDATORY_P0_SEED_IDS = ["SC-000420", "SC-000511", "SC-000512", "SC-000513", "SC-000514", "SC-000515", "SC-000555", "SC-000556", "SC-000557", "SC-000558", "SC-000559", "SC-000560"];
const APPROVED_PLAYWRIGHT_OWNERS = new Map([
  ["SC-000312", "test:playwright:e2e/app-shell-responsive.spec.ts#mobile-menu-trigger-responsive-visibility"],
  ["SC-000344", "test:playwright:e2e/research-projects-sources-filter-row.spec.ts#filters-available-across-responsive-layouts"],
  ["SC-000385", "test:playwright:e2e/dialog-layering.spec.ts#dialog-content-visible-interactive-above-overlay"],
]);
const APPROVED_PLAYWRIGHT_CRITICALITY = "TESTING_BROWSER_COMPONENT_OWNERSHIP";
const REASON_CLASS_CATALOG = [
  { id: "D1_COMPLETED_HISTORY_ONLY", rule: "Completed historical evidence with no future-defect seam." },
  { id: "D2_IMPLEMENTATION_SHAPE", rule: "Implementation-shape assertion with no durable behavioral obligation." },
  { id: "D3_NON_OBSERVABLE_VISUAL", rule: "Non-normative visual detail with no truthful non-browser seam." },
  { id: "D4_DUPLICATE_EVIDENCE", rule: "Duplicate behavior already owned by a base-closed exact owner." },
  { id: "A1_EXISTING_STRUCTURED_OWNER", rule: "Existing repository rule owns the structured architecture invariant." },
  { id: "T1_EXISTING_TOOL_OWNER", rule: "Existing tool owns the executable invariant." },
  { id: "B1_EXISTING_BEHAVIOR_OWNER", rule: "Existing base-closed behavior owner proves the invariant." },
  { id: "B2_NEW_CHEAP_BEHAVIOR", rule: "A new focused jsdom or Cargo behavior owner is cheap and truthful." },
  { id: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", rule: "A protected behavior needs a future owner and normative criticality." },
  { id: "D5_ACCEPTED_LOSS", rule: "Explicitly accepted non-protected behavior loss." },
];
const CRITICALITY_CATALOG = [
  { id: "AGENTS_WINDOWS_SANDBOX", citation: "AGENTS.md \u00a72" },
  { id: "AGENTS_DATABASE_OWNERSHIP", citation: "AGENTS.md \u00a76" },
  { id: "AGENTS_SECURITY", citation: "AGENTS.md \u00a77" },
  { id: "TESTING_SOURCE_CONTRACT_REPLACEMENT", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#source-contract-replacement" },
  { id: "TESTING_BROWSER_COMPONENT_OWNERSHIP", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#browser-and-component-ownership" },
  { id: "TESTING_COVERAGE_FLAKE_QUARANTINE", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#coverage-flake-and-quarantine-policy" },
];
const CRITICALITY_SOURCES = new Map(CRITICALITY_CATALOG.map((source) => [source.id, source.citation]));
const CLASS_DISPOSITIONS = new Map([
  ["D1_COMPLETED_HISTORY_ONLY", "delete"],
  ["D2_IMPLEMENTATION_SHAPE", "delete"],
  ["D3_NON_OBSERVABLE_VISUAL", "delete"],
  ["D4_DUPLICATE_EVIDENCE", "delete"],
  ["A1_EXISTING_STRUCTURED_OWNER", "architecture"],
  ["T1_EXISTING_TOOL_OWNER", "tool_owned"],
  ["B1_EXISTING_BEHAVIOR_OWNER", "behavior"],
  ["B2_NEW_CHEAP_BEHAVIOR", "behavior"],
  ["B3_PROTECTED_EXPENSIVE_BEHAVIOR", "behavior"],
  ["D5_ACCEPTED_LOSS", "delete"],
]);
const OWNER_CLASSES = new Set(["D4_DUPLICATE_EVIDENCE", "A1_EXISTING_STRUCTURED_OWNER", "T1_EXISTING_TOOL_OWNER", "B1_EXISTING_BEHAVIOR_OWNER"]);
const DELETE_CLASSES = new Set([...CLASS_DISPOSITIONS].filter(([, disposition]) => disposition === "delete").map(([reasonClass]) => reasonClass));
const RESOLUTION_KEYS = new Set(["disposition", "replacementIds", "deletionReason", "subgroups"]);

const compareText = (left, right) => String(left).localeCompare(String(right));
const compareId = (left, right) => Number(String(left).slice(3)) - Number(String(right).slice(3)) || compareText(left, right);
const own = (value, key) => Object.prototype.hasOwnProperty.call(value ?? {}, key);
const normalizedPath = (value) => String(value ?? "").replaceAll("\\", "/").replace(/^\.\//, "");
const clone = (value) => structuredClone(value);

/** Serialize JSON deterministically without whitespace or a trailing newline. */
export function canonicalJson(value) {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  return `{${Object.keys(value).sort(compareText).map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
}

export function sha256Text(value) {
  return createHash("sha256").update(String(value)).digest("hex");
}

/** Return only fields that are allowed to be copied into a ledger resolution. */
export function resolutionForDecision(decision) {
  const resolution = decision?.resolution;
  if (!resolution || typeof resolution !== "object" || Array.isArray(resolution)) return undefined;
  if (own(resolution, "subgroups")) return { subgroups: clone(resolution.subgroups) };
  const result = { disposition: resolution.disposition };
  if (own(resolution, "replacementIds")) result.replacementIds = clone(resolution.replacementIds);
  if (own(resolution, "deletionReason")) result.deletionReason = resolution.deletionReason;
  return result;
}

function resolutionless(row) {
  const result = {};
  for (const [key, value] of Object.entries(row ?? {})) if (!RESOLUTION_KEYS.has(key)) result[key] = value;
  return result;
}

function resolutionOnly(row) {
  return Object.fromEntries([...RESOLUTION_KEYS].filter((key) => own(row, key)).map((key) => [key, row[key]]));
}

function envelopeWithoutRows(ledger) {
  const { rows, ...envelope } = ledger ?? {};
  return envelope;
}

function baseOpenIdsFor(baseLedger, baseTrackedPaths) {
  const tracked = baseTrackedPaths instanceof Set ? baseTrackedPaths : new Set();
  return new Set((baseLedger?.rows ?? [])
    .filter((row) => tracked.has(normalizedPath(row?.path)) && !PATH_PRESENT_CLOSED_ROW_IDS.includes(row?.id))
    .map((row) => row.id));
}

function issueList(issues) {
  return [...new Set(issues)].sort(compareText);
}

function exactIds(items) {
  return Array.isArray(items) ? items.map((item) => item?.id).filter((id) => typeof id === "string") : [];
}

function resolutionIds(row) {
  if (row?.disposition === "delete") return [];
  return [
    ...(row?.replacementIds ?? []),
    ...(row?.subgroups ?? []).flatMap((subgroup) => subgroup?.disposition === "delete" ? [] : subgroup?.replacementIds ?? []),
  ].filter((id) => typeof id === "string" && !id.startsWith("test:playwright:"));
}

function validateResolution(decision, row, protectedIds, resolvedOwners, issues) {
  const prefix = `${decision.id}:`;
  const resolution = decision.resolution;
  const requiredDisposition = CLASS_DISPOSITIONS.get(decision.class);
  if (!requiredDisposition) {
    if (decision.class === "UNCLASSIFIED") issues.push(`${prefix} UNCLASSIFIED blocks apply`);
    else issues.push(`${prefix} unsupported reason class: ${decision.class}`);
    return;
  }
  if (!resolution || typeof resolution !== "object" || Array.isArray(resolution)) {
    issues.push(`${prefix} ${decision.class} requires a resolution`);
    return;
  }
  if (Object.keys(resolution).some((key) => !RESOLUTION_KEYS.has(key))) issues.push(`${prefix} resolution contains an unsupported field`);
  const isMixed = own(resolution, "subgroups");
  const groups = isMixed ? resolution.subgroups : [resolution];
  if (isMixed && (!Array.isArray(groups) || groups.length < 2)) issues.push(`${prefix} mixed resolution requires at least two subgroups`);
  if (isMixed && ["disposition", "replacementIds", "deletionReason"].some((key) => own(resolution, key))) issues.push(`${prefix} mixed resolution has top-level resolution`);
  const ownedOrdinals = new Set();
  const deletedOrdinals = new Set();
  for (const subgroup of Array.isArray(groups) ? groups : []) {
    if (!subgroup || typeof subgroup !== "object" || Array.isArray(subgroup)) {
      issues.push(`${prefix} invalid subgroup`);
      continue;
    }
    if (isMixed) {
      if (typeof subgroup.invariant !== "string" || !subgroup.invariant.trim()) issues.push(`${prefix} subgroup requires an invariant`);
      if (!Array.isArray(subgroup.assertionOrdinals) || !subgroup.assertionOrdinals.length) issues.push(`${prefix} subgroup requires assertion ordinals`);
      for (const ordinal of subgroup.assertionOrdinals ?? []) {
        if (!Number.isInteger(ordinal) || ordinal < 1 || ordinal > row.assertionCount) issues.push(`${prefix} invalid subgroup assertion ordinal: ${ordinal}`);
        else if (ownedOrdinals.has(ordinal)) issues.push(`${prefix} duplicate subgroup assertion ordinal: ${ordinal}`);
        else ownedOrdinals.add(ordinal);
      }
    }
    if (subgroup.disposition !== requiredDisposition) issues.push(`${prefix} ${decision.class} requires ${requiredDisposition} disposition`);
    if (subgroup.disposition === "delete") {
      if (own(subgroup, "replacementIds")) issues.push(`${prefix} delete resolution must not include replacementIds`);
      if (typeof subgroup.deletionReason !== "string" || !subgroup.deletionReason.trim()) issues.push(`${prefix} delete resolution requires a deletionReason`);
      if (typeof subgroup.deletionReason === "string" && (subgroup.deletionReason.trim() === decision.class || !subgroup.deletionReason.includes(decision.class))) issues.push(`${prefix} deletion reason must contain row-specific text`);
      for (const ordinal of isMixed ? subgroup.assertionOrdinals ?? [] : Array.from({ length: row.assertionCount }, (_, index) => index + 1)) deletedOrdinals.add(ordinal);
    } else {
      if (!Array.isArray(subgroup.replacementIds) || !subgroup.replacementIds.length) issues.push(`${prefix} retained resolution requires replacementIds`);
      if (own(subgroup, "deletionReason")) issues.push(`${prefix} retained resolution must not include deletionReason`);
      for (const replacementId of subgroup.replacementIds ?? []) {
        if (typeof replacementId !== "string" || !replacementId.trim()) issues.push(`${prefix} invalid replacementId`);
        if (replacementId?.startsWith("test:playwright:")) {
          if (APPROVED_PLAYWRIGHT_OWNERS.get(decision.id) !== replacementId) issues.push(`${prefix} browser owner requires program amendment`);
          else {
            if (decision.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR") issues.push(`${prefix} approved browser owner requires B3_PROTECTED_EXPENSIVE_BEHAVIOR`);
            if (decision.criticalityRef !== APPROVED_PLAYWRIGHT_CRITICALITY) issues.push(`${prefix} approved browser owner requires TESTING_BROWSER_COMPONENT_OWNERSHIP`);
          }
        }
      }
    }
  }
  if (isMixed) for (let ordinal = 1; ordinal <= row.assertionCount; ordinal++) if (!ownedOrdinals.has(ordinal)) issues.push(`${prefix} incomplete subgroup assertion ordinals`);
  if (protectedIds.has(decision.id) && DELETE_CLASSES.has(decision.class)) issues.push(`${prefix} protected row cannot use a deletion class`);
  if (OWNER_CLASSES.has(decision.class)) {
    const replacements = (Array.isArray(groups) ? groups : []).flatMap((group) => group?.replacementIds ?? []);
    const hasExactOwner = decision.class === "D4_DUPLICATE_EVIDENCE"
      ? Array.isArray(decision.ownerEvidence) && decision.ownerEvidence.some((owner) => resolvedOwners.has(owner))
      : Array.isArray(decision.ownerEvidence) && decision.ownerEvidence.some((owner) => replacements.includes(owner) && resolvedOwners.has(owner));
    if (!hasExactOwner) issues.push(`${prefix} ${decision.class} requires exact owner evidence`);
  }
  if (decision.class === "D5_ACCEPTED_LOSS") {
    const lost = new Set();
    let validDescriptions = Array.isArray(decision.lostBehavior) && decision.lostBehavior.length > 0;
    for (const item of decision.lostBehavior ?? []) {
      if (typeof item?.behavior !== "string" || !item.behavior.trim()) validDescriptions = false;
      for (const ordinal of item?.assertionOrdinals ?? []) {
        if (!Number.isInteger(ordinal) || !deletedOrdinals.has(ordinal) || lost.has(ordinal)) validDescriptions = false;
        lost.add(ordinal);
      }
    }
    if (!validDescriptions || lost.size !== deletedOrdinals.size || [...deletedOrdinals].some((ordinal) => !lost.has(ordinal))) issues.push(`${prefix} D5 lostBehavior must cover exactly its deleted assertion ordinals`);
  }
}

const COMPONENT_REPLACEMENT_COMMAND = "node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/projects-workspace.behavior.component.test.ts src/lib/analysis-report-canvas.behavior.component.test.ts src/lib/analysis-report-canvas-route-receiver.behavior.component.test.ts";
const COMPONENT_STARTUP_COMMAND = "node scripts/run-vitest.mjs run --project component src/lib/components/research-projects/SourceStatusCell.component.test.ts";
const VERIFY_GATE_INVENTORY = [
  "npm run check:gemini-browser-sidecar-binary", "node scripts/validate-testing-transition.mjs", "npm run test:unit",
  "npm run test:component", "npm run test:architecture", "npm run test:legacy-contract", "npm run test:integration:os",
  "npm run test:e2e", "npm run check", "npm run check:rustfmt",
  "cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets",
  "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets", "git diff HEAD --check",
];
const LEGACY_BASE_FILES_COUNT = 73;
const LEGACY_BASE_FILES_DIGEST = "5cb1c11d81755cc2b64e5f7bcc6abfff10c4dc50d7e1101d53d8ff692a6bd90e";
const VERIFY_RED_OBSERVATION = {
  executionOrder: 1,
  seconds: 53.987,
  exitCode: 1,
  failedGate: "npm run test:unit",
  failure: "Static draft integration RED: source-contract-redisposition-review.json did not exist.",
};
const VERIFY_GREEN_OBSERVATION = {
  executionOrder: 2,
  seconds: 244.6,
  exitCode: 0,
  authorization: "explicit user-authorized compensating verify after complete GREEN",
};

function equalNumber(left, right) {
  return Number.isFinite(left) && Number.isFinite(right) && Math.abs(left - right) < 1e-12;
}

function retainedMedian(mechanism, name, command, issues) {
  if (!mechanism || typeof mechanism !== "object") {
    issues.push(`mechanisms.${name}: missing`);
    return undefined;
  }
  if (mechanism.command !== command) issues.push(`mechanisms.${name}: command mismatch`);
  if (mechanism.warmupExitCode !== 0) issues.push(`mechanisms.${name}: warmupExitCode must be 0`);
  if (!Array.isArray(mechanism.retainedSeconds) || mechanism.retainedSeconds.length !== 3 || mechanism.retainedSeconds.some((value) => !Number.isFinite(value) || value <= 0)) {
    issues.push(`mechanisms.${name}: retainedSeconds must contain three positive observations`);
    return undefined;
  }
  const median = [...mechanism.retainedSeconds].sort((left, right) => left - right)[1];
  if (!equalNumber(mechanism.medianSeconds, median)) issues.push(`mechanisms.${name}: medianSeconds must equal the retained median`);
  return median;
}

function futureOwnerSummary(decisions, baseLedger) {
  const rowsById = new Map((baseLedger?.rows ?? []).map((row) => [row.id, row]));
  const byMechanism = new Map();
  const proposedRows = new Set();
  let proposedOrdinals = 0;
  for (const decision of decisions ?? []) {
    if (!["B2_NEW_CHEAP_BEHAVIOR", "B3_PROTECTED_EXPENSIVE_BEHAVIOR"].includes(decision?.class)) continue;
    const row = rowsById.get(decision.id);
    const groups = decision.resolution?.subgroups ?? [decision.resolution];
    for (const group of groups) {
      const mechanisms = new Set((group?.replacementIds ?? []).flatMap((id) => {
        if (id?.startsWith("test:vitest:")) return ["jsdom"];
        if (id?.startsWith("test:cargo:")) return ["cargo"];
        if (id?.startsWith("test:playwright:")) return ["playwright"];
        return [];
      }));
      if (!mechanisms.size) continue;
      const ordinals = decision.resolution?.subgroups ? group.assertionOrdinals ?? [] : Array.from({ length: row?.assertionCount ?? 0 }, (_, index) => index + 1);
      for (const mechanism of mechanisms) {
        const summary = byMechanism.get(mechanism) ?? { rows: new Set(), assertionOrdinals: 0 };
        summary.rows.add(decision.id);
        summary.assertionOrdinals += ordinals.length;
        byMechanism.set(mechanism, summary);
      }
      if (decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && mechanisms.has("jsdom")) {
        proposedRows.add(decision.id);
        proposedOrdinals += ordinals.length;
      }
    }
  }
  return {
    futureOwnersByMechanism: Object.fromEntries([...byMechanism].map(([mechanism, summary]) => [mechanism, {
      rows: summary.rows.size,
      assertionOrdinals: summary.assertionOrdinals,
    }])),
    proposedRows: proposedRows.size,
    proposedOrdinals,
  };
}

function validateTimingAndForecast(artifact, baseLedger, issues) {
  const mechanisms = artifact.mechanisms;
  const replacement = retainedMedian(mechanisms?.componentReplacement, "componentReplacement", COMPONENT_REPLACEMENT_COMMAND, issues);
  const startup = retainedMedian(mechanisms?.componentStartup, "componentStartup", COMPONENT_STARTUP_COMMAND, issues);
  const legacy = retainedMedian(mechanisms?.legacyOwner, "legacyOwner", "npm.cmd run test:legacy-contract", issues);
  const baseFiles = mechanisms?.legacyOwner?.baseFiles;
  if (!Array.isArray(baseFiles) || !baseFiles.length || baseFiles.some((file) => typeof file !== "string" || !file || file !== normalizedPath(file) || file.startsWith("../")) || canonicalJson(baseFiles) !== canonicalJson([...baseFiles].sort(compareText))) {
    issues.push("mechanisms.legacyOwner: baseFiles must be sorted normalized repository-relative paths");
  }
  if (!Array.isArray(baseFiles) || baseFiles.length !== LEGACY_BASE_FILES_COUNT || sha256Text(canonicalJson(baseFiles)) !== LEGACY_BASE_FILES_DIGEST) issues.push("mechanisms.legacyOwner: baseFiles do not match the pinned legacy listing");
  const verify = mechanisms?.verify;
  let freshVerifySeconds;
  if (!verify || typeof verify !== "object") issues.push("mechanisms.verify: missing");
  else {
    if (verify.command !== "npm.cmd run verify") issues.push("mechanisms.verify: command mismatch");
    if (canonicalJson(verify.historicalSeconds) !== canonicalJson([208.1, 321.3, 383.4])) issues.push("mechanisms.verify: historicalSeconds mismatch");
    if (canonicalJson(verify.gateInventory) !== canonicalJson(VERIFY_GATE_INVENTORY)) issues.push("mechanisms.verify: gateInventory mismatch");
    if (verify.successfulBaseline === null) {
      if (canonicalJson(verify.observations) !== canonicalJson([VERIFY_RED_OBSERVATION]) || own(verify, "seconds") || own(verify, "exitCode")) issues.push("mechanisms.verify: pending baseline must retain the approved RED observation");
    } else {
      if (canonicalJson(verify.successfulBaseline) !== canonicalJson(VERIFY_GREEN_OBSERVATION)) issues.push("mechanisms.verify: successfulBaseline must match the approved execution record");
      if (canonicalJson(verify.observations) !== canonicalJson([VERIFY_RED_OBSERVATION, VERIFY_GREEN_OBSERVATION])) issues.push("mechanisms.verify: observations must match the approved execution record");
      if (verify.seconds !== verify.successfulBaseline?.seconds || verify.exitCode !== verify.successfulBaseline?.exitCode) issues.push("mechanisms.verify: top-level seconds and exitCode must match successfulBaseline");
      if (verify.exitCode !== 0 || !Number.isFinite(verify.seconds) || verify.seconds <= 0) issues.push("mechanisms.verify: successful baseline requires exitCode 0 and positive seconds");
      else freshVerifySeconds = verify.seconds;
    }
  }
  const forecast = artifact.forecast;
  if (!forecast || typeof forecast !== "object" || replacement === undefined || startup === undefined || legacy === undefined) {
    if (!forecast || typeof forecast !== "object") issues.push("forecast: missing");
    return;
  }
  const proposedRows = forecast.proposedNewJsdomRows;
  const ownerSummary = futureOwnerSummary(artifact.decisions, baseLedger);
  if (canonicalJson(forecast.futureOwnersByMechanism) !== canonicalJson(ownerSummary.futureOwnersByMechanism)) issues.push("forecast: futureOwnersByMechanism does not match owner grouping");
  if (forecast.proposedNewJsdomRows !== ownerSummary.proposedRows || forecast.proposedNewJsdomOrdinals !== ownerSummary.proposedOrdinals) issues.push("forecast: proposed jsdom rows or ordinals do not match B3 owner grouping");
  const approvedBrowserScope = [...APPROVED_PLAYWRIGHT_OWNERS.keys()].every((id) => artifact.decisions.some((decision) => decision.id === id));
  if (approvedBrowserScope) {
    const approvedRows = new Set();
    let approvedOrdinals = 0;
    const rowsById = new Map((baseLedger?.rows ?? []).map((row) => [row.id, row]));
    for (const decision of artifact.decisions) {
      const approvedId = APPROVED_PLAYWRIGHT_OWNERS.get(decision.id);
      if (!approvedId || decision.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR" || decision.criticalityRef !== APPROVED_PLAYWRIGHT_CRITICALITY) continue;
      for (const group of decision.resolution?.subgroups ?? [decision.resolution]) {
        if (!(group?.replacementIds ?? []).includes(approvedId)) continue;
        approvedRows.add(decision.id);
        approvedOrdinals += decision.resolution?.subgroups ? (group.assertionOrdinals ?? []).length : (rowsById.get(decision.id)?.assertionCount ?? 0);
      }
    }
    if (approvedRows.size !== 3 || approvedOrdinals !== 21) issues.push("forecast: approved Playwright owner mapping must equal 3 rows / 21 assertion ordinals");
  }
  if (!Number.isInteger(proposedRows) || proposedRows < 0) issues.push("forecast: proposedNewJsdomRows must be a non-negative integer");
  if (proposedRows > 46) issues.push("forecast: proposedNewJsdomRows exceeds replacementUnitCeiling");
  if (forecast.replacementUnitCeiling !== 46) issues.push("forecast: replacementUnitCeiling must equal 46");
  const upperBoundPerRow = replacement / 46;
  const scalablePerRow = Math.max(0, replacement - startup) / 46;
  const upperForecastSeconds = upperBoundPerRow * proposedRows;
  const scalableForecastSeconds = scalablePerRow * proposedRows;
  const netGateForecastSeconds = Math.max(0, scalableForecastSeconds - legacy);
  const derived = [
    ["upperBoundPerRow", upperBoundPerRow], ["scalablePerRow", scalablePerRow], ["upperForecastSeconds", upperForecastSeconds],
    ["scalableForecastSeconds", scalableForecastSeconds], ["netGateForecastSeconds", netGateForecastSeconds], ["removableLegacySeconds", legacy],
  ];
  for (const [field, value] of derived) if (!equalNumber(forecast[field], value)) issues.push(`forecast: ${field} does not match retained timing arithmetic`);
  if (forecast.legacyDominatesFullUnitCeiling !== (legacy >= scalablePerRow * 46)) issues.push("forecast: legacyDominatesFullUnitCeiling mismatch");
  if (freshVerifySeconds === undefined) {
    if (forecast.netGateForecastPercent !== null) issues.push("forecast: netGateForecastPercent must be null while the successful baseline is pending");
  } else {
    if (!equalNumber(forecast.scalableForecastPercent, scalableForecastSeconds / freshVerifySeconds * 100)) issues.push("forecast: scalableForecastPercent does not match the fresh verify baseline");
    if (!equalNumber(forecast.netGateForecastPercent, netGateForecastSeconds / freshVerifySeconds * 100)) issues.push("forecast: netGateForecastPercent does not match the fresh verify baseline");
  }
}

function expectedSample(decisions, protectedIds) {
  const population = decisions
    .filter((decision) => decision.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && decision.class !== "D5_ACCEPTED_LOSS" && !protectedIds.has(decision.id) && !decision.resolution?.subgroups)
    .map((decision) => decision.id).sort(compareId);
  return { population, rowIds: [...population].sort((left, right) => compareText(sha256Text(left), sha256Text(right))).slice(0, Math.ceil(population.length * 0.1)) };
}

function reviewFingerprint(item, protectedRow = false) {
  const ownerEvidence = OWNER_CLASSES.has(item?.class)
    ? [...new Set(Array.isArray(item?.ownerEvidence) ? item.ownerEvidence : [])].sort(compareText)
    : [];
  const criticalityRef = protectedRow || item?.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR" ? item?.criticalityRef ?? null : null;
  return canonicalJson({ class: item?.class, ownerEvidence, criticalityRef });
}

function expectedComparison(rowIds, matches) {
  return rowIds.every((id) => matches.get(id) === true) ? "agree" : "rule_changed";
}

function validateSimulatedLedger({ artifact, baseLedger, currentLedger, baseTrackedPaths }) {
  const issues = [];
  const baseOpenIds = baseOpenIdsFor(baseLedger, baseTrackedPaths);
  const decisionById = new Map((artifact?.decisions ?? []).map((decision) => [decision.id, decision]));
  const expectedLedger = applyReview({ artifact, baseLedger, currentLedger: baseLedger }).ledger;
  if (canonicalJson(currentLedger.rows.map((row) => row?.id)) !== canonicalJson(baseLedger.rows.map((row) => row?.id))) issues.push("simulated ledger: row ID order drift from review base");
  const expectedById = new Map(expectedLedger.rows.map((row) => [row.id, row]));
  const currentById = new Map(currentLedger.rows.map((row) => [row.id, row]));
  for (const baseRow of baseLedger.rows) {
    const expected = expectedById.get(baseRow.id);
    const current = currentById.get(baseRow.id);
    if (!current || !expected) {
      issues.push(`simulated ledger: missing row: ${baseRow.id}`);
      continue;
    }
    if (canonicalJson(resolutionless(expected)) !== canonicalJson(resolutionless(current)) || canonicalJson(resolutionOnly(expected)) !== canonicalJson(resolutionOnly(current))) issues.push(`simulated ledger: unexpected row output: ${baseRow.id}`);
    const replacementIds = [...(current?.replacementIds ?? []), ...(current?.subgroups ?? []).flatMap((subgroup) => subgroup?.replacementIds ?? [])];
    if (baseOpenIds.has(baseRow.id)) {
      const decision = decisionById.get(baseRow.id);
      if (replacementIds.some((id) => id?.startsWith("test:playwright:") && APPROVED_PLAYWRIGHT_OWNERS.get(decision?.id) !== id)) issues.push(`${baseRow.id}: browser owner requires program amendment`);
    }
  }
  return issueList(issues);
}

/**
 * Validate a proposed artifact against its immutable review-base ledger.
 * The base-open set is derived from the review-base tracked paths, never from decisions.
 */
export function validateReview({ artifact, baseLedger, currentLedger, baseTrackedPaths } = {}) {
  const issues = [];
  if (!artifact || !baseLedger || !currentLedger || !Array.isArray(baseLedger.rows) || !Array.isArray(currentLedger.rows)) return ["invalid review input"];
  if (artifact.schemaVersion !== 1) issues.push("artifact: schemaVersion must be 1");
  if (artifact.reviewBaseCommit !== REVIEW_BASE_COMMIT) issues.push(`artifact: reviewBaseCommit must be ${REVIEW_BASE_COMMIT}`);
  if (artifact.ledgerFrozenAtCommit !== baseLedger.frozenAtCommit) issues.push("artifact: ledgerFrozenAtCommit must equal base ledger frozenAtCommit");
  if (canonicalJson(baseLedger.sourceReaderExceptions) !== canonicalJson(currentLedger.sourceReaderExceptions)) issues.push("ledger: sourceReaderExceptions changed");
  if (canonicalJson(envelopeWithoutRows(baseLedger)) !== canonicalJson(envelopeWithoutRows(currentLedger))) issues.push("ledger: immutable envelope field changed");

  const declaredExceptions = artifact.scope?.pathPresentClosedRowIds;
  if (canonicalJson(declaredExceptions) !== canonicalJson(PATH_PRESENT_CLOSED_ROW_IDS)) issues.push("scope: pathPresentClosedRowIds must equal SC-000355, SC-000366");
  const tracked = baseTrackedPaths instanceof Set ? new Set([...baseTrackedPaths].map(normalizedPath)) : undefined;
  if (!tracked) issues.push("scope: baseTrackedPaths must be a Set");
  for (const id of PATH_PRESENT_CLOSED_ROW_IDS) {
    const row = baseLedger.rows.find((item) => item?.id === id);
    if (!row || !tracked?.has(normalizedPath(row.path))) issues.push(`scope: required path-present closed row is not tracked: ${id}`);
  }
  const baseOpenIds = baseOpenIdsFor(baseLedger, tracked);
  const closedRows = baseLedger.rows.filter((row) => !baseOpenIds.has(row.id));
  if (artifact.scope?.openRows !== baseOpenIds.size) issues.push(`scope: expected ${artifact.scope?.openRows ?? "unknown"} base-open rows, found ${baseOpenIds.size}`);
  if (artifact.scope?.closedRows !== closedRows.length) issues.push(`scope: expected ${artifact.scope?.closedRows ?? "unknown"} closed rows, found ${closedRows.length}`);
  if (artifact.scope?.closedRowsDigest !== sha256Text(canonicalJson(closedRows))) issues.push("scope: closedRowsDigest mismatch");

  if (canonicalJson(currentLedger.rows.map((row) => row?.id)) !== canonicalJson(baseLedger.rows.map((row) => row?.id))) issues.push("ledger: current row ID order drift from review base");
  const currentById = new Map(currentLedger.rows.map((row) => [row?.id, row]));
  const resolvedOwners = new Set(closedRows.flatMap(resolutionIds));
  if (currentLedger.rows.length !== baseLedger.rows.length) issues.push("ledger: row count changed");
  for (const baseRow of baseLedger.rows) {
    const currentRow = currentById.get(baseRow.id);
    if (!currentRow) {
      issues.push(`ledger: missing row: ${baseRow.id}`);
      continue;
    }
    if (canonicalJson(resolutionless(baseRow)) !== canonicalJson(resolutionless(currentRow))) issues.push(`${baseRow.id}: immutable row field drift`);
    if (baseOpenIds.has(baseRow.id) && canonicalJson(resolutionOnly(baseRow)) !== canonicalJson(resolutionOnly(currentRow))) issues.push(`${baseRow.id}: current resolution drift from review base`);
  }
  const currentClosedRows = currentLedger.rows.filter((row) => !baseOpenIds.has(row?.id));
  if (canonicalJson(currentClosedRows) !== canonicalJson(closedRows)) issues.push("scope: closed rows changed");

  const decisions = Array.isArray(artifact.decisions) ? [...artifact.decisions].sort((left, right) => compareId(left?.id, right?.id)) : [];
  const decisionIds = new Set();
  for (const decision of decisions) {
    if (!decision || typeof decision !== "object") {
      issues.push("scope: invalid decision");
      continue;
    }
    if (decisionIds.has(decision.id)) issues.push(`scope: duplicate decision id: ${decision.id}`);
    decisionIds.add(decision.id);
    if (!baseOpenIds.has(decision.id)) issues.push(`scope: decision is not a base-open row: ${decision.id}`);
    const baseRow = baseLedger.rows.find((row) => row.id === decision.id);
    if (baseRow && decision.sourceHash !== baseRow.sourceHash) issues.push(`${decision.id}: decision sourceHash drift`);
    if (decision.authorRunId !== artifact.independentReview?.authorRunId) issues.push(`${decision.id}: authorRunId differs from artifact author run`);
    if (decision.class !== "UNCLASSIFIED" && (typeof decision.reason !== "string" || !decision.reason.trim())) issues.push(`${decision.id}: classified decision requires a substantive reason`);
    if (decision.class === "UNCLASSIFIED" && (own(decision, "reason") || own(decision, "resolution"))) issues.push(`${decision.id}: UNCLASSIFIED may not contain reason or resolution`);
    if (Object.keys(decision).some((key) => ["override", "individualOverride"].includes(key))) issues.push(`${decision.id}: individual override is forbidden`);
  }
  if (decisionIds.size !== baseOpenIds.size) issues.push("scope: expected one decision for every base-open row");
  for (const id of baseOpenIds) if (!decisionIds.has(id)) issues.push(`scope: missing decision: ${id}`);

  if (canonicalJson(artifact.reasonClasses) !== canonicalJson(REASON_CLASS_CATALOG)) issues.push("reasonClasses: must equal the approved catalog");

  const protectedRows = Array.isArray(artifact.protectedRows) ? artifact.protectedRows : [];
  const protectedIds = new Set(exactIds(protectedRows));
  if (protectedIds.size !== protectedRows.length) issues.push("protectedRows: duplicate id");
  if (artifact.protectedRowsDigest !== sha256Text(canonicalJson(protectedRows))) issues.push("protectedRows: digest mismatch");
  const criticalityIds = new Set();
  const seenCriticalityIds = new Set();
  if (!Array.isArray(artifact.criticalitySources)) issues.push("criticalitySources: must be an array");
  if (canonicalJson(artifact.criticalitySources) !== canonicalJson(CRITICALITY_CATALOG)) issues.push("criticalitySources: must equal the approved catalog");
  for (const source of artifact.criticalitySources ?? []) {
    if (seenCriticalityIds.has(source?.id)) issues.push(`criticalitySources: duplicate id: ${source?.id}`);
    seenCriticalityIds.add(source?.id);
    if (Object.keys(source ?? {}).sort(compareText).join(",") !== "citation,id" || CRITICALITY_SOURCES.get(source?.id) !== source?.citation) issues.push(`criticalitySources: invalid built-in mapping: ${source?.id}`);
    else criticalityIds.add(source.id);
  }
  for (const id of MANDATORY_P0_SEED_IDS) if (!protectedIds.has(id)) issues.push("protectedRows: mandatory P0 seed set mismatch");
  for (const protectedRow of protectedRows) if (!baseOpenIds.has(protectedRow?.id) || !criticalityIds.has(protectedRow?.criticalityRef)) issues.push(`protectedRows: invalid criticality source for ${protectedRow?.id}`);
  for (const decision of decisions) {
    const baseRow = baseLedger.rows.find((row) => row.id === decision.id);
    if (!baseRow) continue;
    const protectedRow = protectedRows.find((item) => item.id === decision.id);
    if (decision.class !== "UNCLASSIFIED" && (protectedRow || decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR") && (!criticalityIds.has(decision.criticalityRef) || (protectedRow && protectedRow.criticalityRef !== decision.criticalityRef))) issues.push(`${decision.id}: protected behavior requires a valid criticalityRef`);
    validateResolution(decision, baseRow, protectedIds, resolvedOwners, issues);
  }

  validateTimingAndForecast(artifact, baseLedger, issues);

  const independent = artifact.independentReview;
  if (!independent || typeof independent !== "object") issues.push("independentReview: missing");
  else {
    if (!independent.authorRunId || !independent.reviewer?.agentTaskId || independent.authorRunId === independent.reviewer.agentTaskId) issues.push("independentReview: authorRunId and reviewer agentTaskId must differ");
    const reviewer = independent.reviewer;
    if (!reviewer?.reviewerRunId || new Set([independent.authorRunId, reviewer?.agentTaskId, reviewer?.reviewerRunId]).size !== 3) issues.push("independentReview: authorRunId, reviewer agentTaskId, and reviewerRunId must be distinct");
    if (reviewer?.contextPolicy !== "blind-no-proposed-class-or-reason") issues.push("independentReview: unsupported contextPolicy");
    const sha256Pattern = /^[0-9a-f]{64}$/;
    if (!sha256Pattern.test(reviewer?.packetSha256 ?? "")) issues.push("independentReview: reviewer packetSha256 must be lowercase SHA-256");
    if (!sha256Pattern.test(reviewer?.outputSha256 ?? "")) issues.push("independentReview: reviewer outputSha256 must be lowercase SHA-256");
    if (reviewer?.packetSha256 && reviewer.packetSha256 === reviewer?.outputSha256) issues.push("independentReview: packet and output SHA-256 values must differ");
    if (typeof reviewer?.packetPath !== "string" || !reviewer.packetPath.trim() || typeof reviewer?.outputPath !== "string" || !reviewer.outputPath.trim()) issues.push("independentReview: reviewer packetPath and outputPath are required");
    else if (!reviewer.packetPath.includes(reviewer.packetSha256) || !reviewer.outputPath.includes(reviewer.outputSha256)) issues.push("independentReview: reviewer evidence paths must contain their SHA-256 values");
    const sample = independent.deterministicSample;
    const expected = expectedSample(decisions, protectedIds);
    const requiredBlindIds = new Set([
      ...(Array.isArray(independent.calibrations) ? independent.calibrations : []).flatMap((item) => item?.rowIds ?? []),
      ...(Array.isArray(independent.mandatoryCohorts) ? independent.mandatoryCohorts : []).flatMap((item) => item?.rowIds ?? []),
      ...(Array.isArray(sample?.rowIds) ? sample.rowIds : []),
    ]);
    const requiredBlindIdsNumeric = [...requiredBlindIds].sort(compareId);
    const blinds = Array.isArray(independent.blindResults) ? independent.blindResults : [];
    const blindById = new Map();
    const fingerprintMatches = new Map();
    for (const blind of blinds) {
      if (blindById.has(blind?.id)) issues.push(`independentReview: duplicate blind result: ${blind?.id}`);
      blindById.set(blind?.id, blind);
      if (!requiredBlindIds.has(blind?.id)) issues.push(`${blind?.id}: extraneous blind result`);
      if (!CLASS_DISPOSITIONS.has(blind?.class) || blind?.class === "UNCLASSIFIED") issues.push(`${blind?.id}: blind result has an unsupported class`);
      if (typeof blind?.reason !== "string" || !blind.reason.trim()) issues.push(`${blind?.id}: blind result requires a substantive reason`);
      if (blind?.ownerEvidence !== undefined && (!Array.isArray(blind.ownerEvidence) || blind.ownerEvidence.some((owner) => typeof owner !== "string" || !owner.trim()))) issues.push(`${blind?.id}: blind ownerEvidence is invalid`);
      if (blind?.criticalityRef !== undefined && !criticalityIds.has(blind.criticalityRef)) issues.push(`${blind?.id}: blind criticalityRef is invalid`);
      if (blind?.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && !criticalityIds.has(blind?.criticalityRef)) issues.push(`${blind?.id}: blind protected behavior requires a valid criticalityRef`);
      if (OWNER_CLASSES.has(blind?.class) && (!Array.isArray(blind.ownerEvidence) || !blind.ownerEvidence.some((owner) => resolvedOwners.has(owner)))) issues.push(`${blind?.id}: blind existing owner evidence is unresolved`);
    }
    for (const id of requiredBlindIds) {
      const decision = decisions.find((item) => item.id === id);
      if (!decision) { issues.push(`${id}: blind scope references no decision`); continue; }
      const blind = blindById.get(id);
      if (!blind) { issues.push(`${id}: missing blind result`); continue; }
      if (blind.reviewerRunId !== reviewer?.reviewerRunId) issues.push(`${decision.id}: reviewerRunId differs from reviewer run`);
      if (blind.sourceHash !== decision.sourceHash) issues.push(`${decision.id}: blind sourceHash drift`);
      if (blind.class !== decision.class) issues.push(`${decision.id}: author/blind class comparison is incorrect`);
      const protectedRow = protectedIds.has(id);
      fingerprintMatches.set(id, reviewFingerprint(decision, protectedRow) === reviewFingerprint(blind, protectedRow));
    }
    const validIterations = independent.validIterations;
    if (!Number.isInteger(validIterations) || validIterations < 0 || validIterations > 3 || validIterations !== sample?.iterations) {
      issues.push("independentReview: validIterations must be 0..3 and equal deterministicSample.iterations");
    }
    const shards = Array.isArray(independent.shards) ? independent.shards : [];
    const shardRows = [];
    for (let index = 0; index < shards.length; index += 1) {
      const shard = shards[index];
      const rowIds = Array.isArray(shard?.rowIds) ? shard.rowIds : [];
      if (shard?.index !== index) issues.push(`independentReview: shard indexes must be contiguous from zero`);
      if (rowIds.length < 1 || rowIds.length > 24) issues.push(`independentReview: shard ${index} must contain 1..24 rows`);
      shardRows.push(...rowIds);
      const hashed = sha256Pattern.test(shard?.packetSha256 ?? "")
        && sha256Pattern.test(shard?.outputSha256 ?? "")
        && shard.packetSha256 !== shard.outputSha256
        && typeof shard?.packetPath === "string" && shard.packetPath.includes(shard.packetSha256)
        && typeof shard?.outputPath === "string" && shard.outputPath.includes(shard.outputSha256);
      if (!hashed) issues.push(`independentReview: shard ${index} requires content-addressed packet/output evidence`);
    }
    if (canonicalJson(shardRows) !== canonicalJson(requiredBlindIdsNumeric)
      || canonicalJson(blinds.map((blind) => blind?.id)) !== canonicalJson(requiredBlindIdsNumeric)) {
      issues.push("independentReview: shard rows must exactly cover required blind IDs in numeric order");
    }
    if (!sha256Pattern.test(independent.mergedOutputSha256 ?? "")
      || independent.mergedOutputSha256 !== sha256Text(canonicalJson(blinds))) {
      issues.push("independentReview: merged output digest mismatch");
    }
    if (!sample || sample.algorithm !== "sha256-id-lowest-10-percent") issues.push("independentReview: unsupported deterministic sample algorithm");
    else {
      if (canonicalJson(sample.population) !== canonicalJson(expected.population)) issues.push("independentReview: deterministic sample population is stale");
      if (sample.populationDigest !== sha256Text(canonicalJson(expected.population))) issues.push("independentReview: deterministic sample digest mismatch");
      if (canonicalJson(sample.rowIds) !== canonicalJson(expected.rowIds)) issues.push("independentReview: deterministic sample IDs are not deterministic");
      if (!Number.isInteger(sample.iterations) || sample.iterations < 0 || sample.iterations > 3) issues.push("independentReview: deterministic sample iterations must be 0..3");
    }
    for (const cohort of Array.isArray(independent.calibrations) ? independent.calibrations : []) {
      if (!Array.isArray(cohort?.rowIds)) {
        issues.push("calibrations: rowIds must be an array");
        continue;
      }
      const rowIds = cohort.rowIds;
      const expectedComparisonValue = rowIds.length ? expectedComparison(rowIds, fingerprintMatches) : "no_match";
      if (!rowIds.length && cohort?.result !== expectedComparisonValue) {
        issues.push("calibrations: empty calibration must record no_match");
        continue;
      }
      if (cohort?.result !== expectedComparisonValue) issues.push(`calibrations: recorded comparison must be ${expectedComparisonValue}`);
    }
    for (const cohort of Array.isArray(independent.mandatoryCohorts) ? independent.mandatoryCohorts : []) {
      const expectedComparisonValue = expectedComparison(Array.isArray(cohort?.rowIds) ? cohort.rowIds : [], fingerprintMatches);
      if (cohort?.comparison !== expectedComparisonValue) issues.push(`mandatoryCohorts: recorded comparison must be ${expectedComparisonValue}`);
    }
    if (sample) {
      const expectedComparisonValue = expectedComparison(Array.isArray(sample.rowIds) ? sample.rowIds : [], fingerprintMatches);
      if (sample.comparison !== expectedComparisonValue) issues.push(`deterministicSample: recorded comparison must be ${expectedComparisonValue}`);
    }
    const mismatchIds = [...fingerprintMatches].filter(([, matches]) => !matches).map(([id]) => id).sort(compareId);
    const disagreementIds = Array.isArray(independent.disagreements)
      ? independent.disagreements.flatMap((item) => Array.isArray(item?.rowIds) ? item.rowIds : [])
      : [];
    for (const disagreement of independent.disagreements ?? []) {
      if (!Array.isArray(disagreement?.rowIds) || !disagreement.rowIds.length
        || typeof disagreement.oldClass !== "string" || typeof disagreement.newClass !== "string"
        || typeof disagreement.groupRuleChange !== "string" || !disagreement.groupRuleChange.trim()) {
        issues.push("independentReview: invalid disagreement");
      }
    }
    if (canonicalJson(disagreementIds) !== canonicalJson(mismatchIds)) issues.push("independentReview: disagreements must equal fingerprint mismatch IDs");
  }
  return issueList(issues);
}

/** Apply artifact resolutions without mutating any input. */
export function applyReview({ artifact, baseLedger, currentLedger, baseTrackedPaths } = {}) {
  const ledger = clone(currentLedger);
  const rowsById = new Map(ledger.rows.map((row) => [row.id, row]));
  const changedPaths = [];
  for (const decision of [...(artifact?.decisions ?? [])].sort((left, right) => compareId(left.id, right.id))) {
    const row = rowsById.get(decision.id);
    const resolution = resolutionForDecision(decision);
    if (!row || !resolution) continue;
    const before = Object.fromEntries([...RESOLUTION_KEYS].filter((key) => own(row, key)).map((key) => [key, clone(row[key])]));
    for (const key of RESOLUTION_KEYS) delete row[key];
    if (own(resolution, "subgroups")) {
      row.subgroups = resolution.subgroups;
    } else {
      row.disposition = resolution.disposition;
      if (own(resolution, "replacementIds")) {
        row.replacementIds = resolution.replacementIds;
      }
      if (own(resolution, "deletionReason")) {
        row.deletionReason = resolution.deletionReason;
      }
    }
    for (const key of RESOLUTION_KEYS) {
      if (own(before, key) !== own(row, key) || (own(before, key) && canonicalJson(before[key]) !== canonicalJson(row[key]))) changedPaths.push(`rows[${decision.id}].${key}`);
    }
  }
  return { ledger, changedPaths: [...new Set(changedPaths)].sort(compareText) };
}

export async function loadBaseLedger({ repoRoot, commit }) {
  const output = execFileSync("git", ["show", `${commit}:testing/source-contract-ledger.json`], { cwd: repoRoot, encoding: "utf8", shell: false, windowsHide: true });
  return JSON.parse(output);
}

export async function loadBaseTrackedPaths({ repoRoot, commit }) {
  const output = execFileSync("git", ["ls-tree", "-r", "--name-only", commit], { cwd: repoRoot, encoding: "utf8", shell: false, windowsHide: true });
  return new Set(output.split(/\r?\n/).filter(Boolean).map(normalizedPath));
}

function report(artifact, baseLedger, changedPaths) {
  const decisions = [...(artifact.decisions ?? [])].sort((left, right) => compareId(left.id, right.id));
  const pathById = new Map((baseLedger.rows ?? []).map((row) => [row.id, row.path]));
  const groups = new Map();
  for (const decision of decisions) {
    const key = decision.class ?? "UNKNOWN";
    const paths = groups.get(key) ?? new Map();
    const legacyPath = pathById.get(decision.id) ?? "unknown";
    (paths.get(legacyPath) ?? paths.set(legacyPath, []).get(legacyPath)).push(decision.id);
    groups.set(key, paths);
  }
  for (const [reasonClass, paths] of [...groups].sort(([left], [right]) => compareText(left, right))) {
    process.stdout.write(`${reasonClass}:\n`);
    for (const [legacyPath, ids] of [...paths].sort(([left], [right]) => compareText(left, right))) process.stdout.write(`  ${legacyPath}: ${ids.sort(compareId).join(", ")}\n`);
  }
  process.stdout.write(`changed JSON paths: ${changedPaths.join(", ")}\n`);
}

async function runCli() {
  const mode = process.argv[2];
  if (!["check", "apply"].includes(mode)) throw new Error("Use: node scripts/testing/source-contract-redisposition-review.mjs <check|apply>");
  const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
  const artifactPath = path.join(repoRoot, "testing", "source-contract-redisposition-review.json");
  const ledgerPath = path.join(repoRoot, "testing", "source-contract-ledger.json");
  const artifact = JSON.parse(await readFile(artifactPath, "utf8"));
  if (artifact.reviewBaseCommit !== REVIEW_BASE_COMMIT) {
    process.stderr.write(`artifact: reviewBaseCommit must be ${REVIEW_BASE_COMMIT}\n`);
    process.exitCode = 1;
    return;
  }
  const [baseLedger, baseTrackedPaths, currentLedger] = await Promise.all([
    loadBaseLedger({ repoRoot, commit: artifact.reviewBaseCommit }),
    loadBaseTrackedPaths({ repoRoot, commit: artifact.reviewBaseCommit }),
    readFile(ledgerPath, "utf8").then(JSON.parse),
  ]);
  const input = { artifact, baseLedger, currentLedger, baseTrackedPaths };
  const issues = validateReview(input);
  if (issues.length) {
    for (const issue of issues) process.stderr.write(`${issue}\n`);
    process.exitCode = 1;
    return;
  }
  const first = applyReview(input);
  const simulatedIssues = validateSimulatedLedger({ ...input, currentLedger: first.ledger });
  if (simulatedIssues.length) {
    for (const issue of simulatedIssues) process.stderr.write(`simulated: ${issue}\n`);
    process.exitCode = 1;
    return;
  }
  const second = applyReview({ ...input, currentLedger: first.ledger });
  if (canonicalJson(first.ledger) !== canonicalJson(second.ledger)) throw new Error("apply is not byte-identical on its second in-memory application");
  report(artifact, baseLedger, first.changedPaths);
  if (mode === "apply") await writeFile(ledgerPath, `${canonicalJson(first.ledger)}\n`, "utf8");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runCli().catch((error) => {
    process.stderr.write(`${error instanceof Error ? error.message : error}\n`);
    process.exitCode = 1;
  });
}
