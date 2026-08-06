import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const REVIEW_BASE_COMMIT = "a54507d63420bb870c3870c91d7e22b050abae3e";
const PATH_PRESENT_CLOSED_ROW_IDS = ["SC-000355", "SC-000366"];
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
        if (replacementId?.startsWith("test:playwright:")) issues.push(`${prefix} browser owner requires program amendment`);
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

function expectedSample(decisions, protectedIds) {
  const population = decisions
    .filter((decision) => decision.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && decision.class !== "D5_ACCEPTED_LOSS" && !protectedIds.has(decision.id) && !decision.resolution?.subgroups)
    .map((decision) => decision.id).sort(compareId);
  return { population, rowIds: [...population].sort((left, right) => compareText(sha256Text(left), sha256Text(right))).slice(0, Math.ceil(population.length * 0.1)) };
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

  const currentById = new Map(currentLedger.rows.map((row) => [row?.id, row]));
  const resolvedOwners = new Set(baseLedger.rows.flatMap((row) => [
    ...(row?.replacementIds ?? []),
    ...(row?.subgroups ?? []).flatMap((subgroup) => subgroup?.replacementIds ?? []),
  ]));
  if (currentLedger.rows.length !== baseLedger.rows.length) issues.push("ledger: row count changed");
  for (const baseRow of baseLedger.rows) {
    const currentRow = currentById.get(baseRow.id);
    if (!currentRow) {
      issues.push(`ledger: missing row: ${baseRow.id}`);
      continue;
    }
    if (canonicalJson(resolutionless(baseRow)) !== canonicalJson(resolutionless(currentRow))) issues.push(`${baseRow.id}: immutable row field drift`);
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

  const protectedRows = Array.isArray(artifact.protectedRows) ? artifact.protectedRows : [];
  const protectedIds = new Set(exactIds(protectedRows));
  if (artifact.protectedRowsDigest !== sha256Text(canonicalJson(protectedRows))) issues.push("protectedRows: digest mismatch");
  const criticalityIds = new Set(exactIds(artifact.criticalitySources));
  for (const protectedRow of protectedRows) if (!baseOpenIds.has(protectedRow?.id) || !criticalityIds.has(protectedRow?.criticalityRef)) issues.push(`protectedRows: invalid criticality source for ${protectedRow?.id}`);
  for (const decision of decisions) {
    const baseRow = baseLedger.rows.find((row) => row.id === decision.id);
    if (!baseRow) continue;
    const protectedRow = protectedRows.find((item) => item.id === decision.id);
    if ((protectedRow || decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR") && (!criticalityIds.has(decision.criticalityRef) || (protectedRow && protectedRow.criticalityRef !== decision.criticalityRef))) issues.push(`${decision.id}: protected behavior requires a valid criticalityRef`);
    validateResolution(decision, baseRow, protectedIds, resolvedOwners, issues);
  }

  const independent = artifact.independentReview;
  if (!independent || typeof independent !== "object") issues.push("independentReview: missing");
  else {
    if (!independent.authorRunId || !independent.reviewer?.agentTaskId || independent.authorRunId === independent.reviewer.agentTaskId) issues.push("independentReview: authorRunId and reviewer agentTaskId must differ");
    if (independent.reviewer?.contextPolicy !== "blind-no-proposed-class-or-reason") issues.push("independentReview: unsupported contextPolicy");
    const blinds = Array.isArray(independent.blindResults) ? independent.blindResults : [];
    const blindById = new Map(blinds.map((item) => [item?.id, item]));
    for (const decision of decisions) {
      const blind = blindById.get(decision.id);
      if (!blind) { issues.push(`${decision.id}: missing blind result`); continue; }
      if (blind.reviewerRunId !== independent.reviewer?.agentTaskId) issues.push(`${decision.id}: reviewerRunId differs from reviewer task`);
      if (blind.sourceHash !== decision.sourceHash) issues.push(`${decision.id}: blind sourceHash drift`);
      if (blind.class !== decision.class) issues.push(`${decision.id}: author/blind class comparison is incorrect`);
      if (typeof blind.reason !== "string" || !blind.reason.trim()) issues.push(`${decision.id}: blind result requires a substantive reason`);
    }
    const sample = independent.deterministicSample;
    const expected = expectedSample(decisions, protectedIds);
    if (!sample || sample.algorithm !== "sha256-id-lowest-10-percent") issues.push("independentReview: unsupported deterministic sample algorithm");
    else {
      if (canonicalJson(sample.population) !== canonicalJson(expected.population)) issues.push("independentReview: deterministic sample population is stale");
      if (sample.populationDigest !== sha256Text(canonicalJson(expected.population))) issues.push("independentReview: deterministic sample digest mismatch");
      if (canonicalJson(sample.rowIds) !== canonicalJson(expected.rowIds)) issues.push("independentReview: deterministic sample IDs are not deterministic");
      if (!Number.isInteger(sample.iterations) || sample.iterations < 1 || sample.iterations > 3) issues.push("independentReview: deterministic sample iterations must be 1..3");
    }
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
    for (const key of RESOLUTION_KEYS) delete row[key];
    if (own(resolution, "subgroups")) {
      row.subgroups = resolution.subgroups;
      changedPaths.push(`rows[${decision.id}].subgroups`);
    } else {
      row.disposition = resolution.disposition;
      changedPaths.push(`rows[${decision.id}].disposition`);
      if (own(resolution, "replacementIds")) {
        row.replacementIds = resolution.replacementIds;
        changedPaths.push(`rows[${decision.id}].replacementIds`);
      }
      if (own(resolution, "deletionReason")) {
        row.deletionReason = resolution.deletionReason;
        changedPaths.push(`rows[${decision.id}].deletionReason`);
      }
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
  const simulatedIssues = validateReview({ ...input, currentLedger: first.ledger });
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
