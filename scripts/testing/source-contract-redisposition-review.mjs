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
const REQUIRED_D3_REASONS = new Map([
  ["SC-000323", "Visible active-row focus/selection styling is a non-normative visual affordance with no truthful non-browser seam."],
  ["SC-000352", "Two-line title clipping is a non-normative rendered-layout detail with no truthful non-browser seam."],
  ["SC-000353", "Cell centering and overflow ellipsis are non-normative rendered-layout details with no truthful non-browser seam."],
]);
const REVIEW_EVIDENCE_DIRECTORY = "testing/source-contract-redisposition-evidence";
const REVIEW_PACKET_SCHEMA_VERSION = 3;
const REASON_CLASS_CATALOG = [
  { id: "D1_COMPLETED_HISTORY_ONLY", rule: "Completed historical evidence only; forbidden whenever a future code, configuration, or test change could violate the invariant as a defect." },
  { id: "D2_IMPLEMENTATION_SHAPE", rule: "Exact names, source strings, statement order, formatting, or composition with no independent contract; apply before visual-only D3." },
  { id: "D3_NON_OBSERVABLE_VISUAL", rule: "Non-normative CSS, DOM order, or rendered visual detail with no truthful non-browser seam; implementation shape remains D2." },
  { id: "D4_DUPLICATE_EVIDENCE", rule: "A base-closed exact owner already proves the complete invariant, so the source-contract row is duplicate evidence; apply before retained-owner classes." },
  { id: "A1_EXISTING_STRUCTURED_OWNER", rule: "An existing structured repository rule, compiler, or Cargo-metadata owner is the durable architecture replacement." },
  { id: "T1_EXISTING_TOOL_OWNER", rule: "An existing maintained tool owns the generic executable invariant." },
  { id: "B1_EXISTING_BEHAVIOR_OWNER", rule: "An existing executable test proves behavior through a public seam and the row must retain a behavior resolution; use D4 first when a base-closed owner makes the row wholly duplicate." },
  { id: "B2_NEW_CHEAP_BEHAVIOR", rule: "A new focused Node/jsdom or Cargo test through a cheap public seam is required; P0 membership or a normative citation alone does not make the owner B3." },
  { id: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", rule: "A real browser or OS-process runtime test is required, no cheaper truthful seam exists, and an approved normative criticality source requires retention." },
  { id: "D5_ACCEPTED_LOSS", rule: "Real non-P0 behavior has only an expensive truthful mechanism and no normative source requires retention, so the enumerated loss is accepted." },
];
const APPROVED_RULE_ADJUDICATIONS = [
  {
    rule: "D1 applies only when the invariant is exhausted by a completed event and no future repository change can violate it as a defect. Exact current source, symbol, file, or test topology without an independent contract is D2.",
    rowIds: ["SC-000077", "SC-000080", "SC-000081", "SC-000092", "SC-000203"],
    finalClass: "D2_IMPLEMENTATION_SHAPE",
  },
  {
    rule: "A resolved base-closed owner that exactly proves the full invariant makes the legacy evidence D4 before A1.",
    rowIds: ["SC-000356", "SC-000357", "SC-000358"],
    finalClass: "D4_DUPLICATE_EVIDENCE",
  },
  {
    rule: "B2/B3 is determined by the truthful observation seam, not the runner prefix; real browser or OS-process observation is B3.",
    rowIds: ["SC-000420"],
    finalClass: "B3_PROTECTED_EXPENSIVE_BEHAVIOR",
  },
  {
    rule: "A retained owner must prove the complete frozen invariant. Adjacent behavior evidence does not own a no-copy or placement topology assertion.",
    rowIds: ["SC-000449"],
    finalClass: "D2_IMPLEMENTATION_SHAPE",
  },
];
const APPROVED_RULE_DISAGREEMENTS = [
  { rowIds: APPROVED_RULE_ADJUDICATIONS[0].rowIds, oldClass: "D1_COMPLETED_HISTORY_ONLY", newClass: "D2_IMPLEMENTATION_SHAPE", groupRuleChange: APPROVED_RULE_ADJUDICATIONS[0].rule },
  { rowIds: APPROVED_RULE_ADJUDICATIONS[1].rowIds, oldClass: "A1_EXISTING_STRUCTURED_OWNER", newClass: "D4_DUPLICATE_EVIDENCE", groupRuleChange: APPROVED_RULE_ADJUDICATIONS[1].rule },
  { rowIds: APPROVED_RULE_ADJUDICATIONS[2].rowIds, oldClass: "B2_NEW_CHEAP_BEHAVIOR", newClass: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", groupRuleChange: APPROVED_RULE_ADJUDICATIONS[2].rule },
  { rowIds: APPROVED_RULE_ADJUDICATIONS[3].rowIds, oldClass: "B2_NEW_CHEAP_BEHAVIOR", newClass: "D2_IMPLEMENTATION_SHAPE", groupRuleChange: APPROVED_RULE_ADJUDICATIONS[3].rule },
];
const APPROVED_CALIBRATION_PAIRS = [
  { class: "D1_COMPLETED_HISTORY_ONLY", adjacentClass: "D2_IMPLEMENTATION_SHAPE" },
  { class: "D2_IMPLEMENTATION_SHAPE", adjacentClass: "D1_COMPLETED_HISTORY_ONLY" },
  { class: "D3_NON_OBSERVABLE_VISUAL", adjacentClass: "D5_ACCEPTED_LOSS/B3_PROTECTED_EXPENSIVE_BEHAVIOR" },
  { class: "D4_DUPLICATE_EVIDENCE", adjacentClass: "A1_EXISTING_STRUCTURED_OWNER/B1_EXISTING_BEHAVIOR_OWNER" },
  { class: "A1_EXISTING_STRUCTURED_OWNER", adjacentClass: "D4_DUPLICATE_EVIDENCE/T1_EXISTING_TOOL_OWNER" },
  { class: "T1_EXISTING_TOOL_OWNER", adjacentClass: "A1_EXISTING_STRUCTURED_OWNER" },
  { class: "B1_EXISTING_BEHAVIOR_OWNER", adjacentClass: "D4_DUPLICATE_EVIDENCE/B2_NEW_CHEAP_BEHAVIOR" },
  { class: "B2_NEW_CHEAP_BEHAVIOR", adjacentClass: "B1_EXISTING_BEHAVIOR_OWNER/B3_PROTECTED_EXPENSIVE_BEHAVIOR" },
  { class: "B3_PROTECTED_EXPENSIVE_BEHAVIOR", adjacentClass: "D3_NON_OBSERVABLE_VISUAL/B2_NEW_CHEAP_BEHAVIOR/D5_ACCEPTED_LOSS" },
  { class: "D5_ACCEPTED_LOSS", adjacentClass: "D3_NON_OBSERVABLE_VISUAL/B3_PROTECTED_EXPENSIVE_BEHAVIOR" },
];
const APPROVED_MANDATORY_COHORT_NAMES = [
  "protected-p0", "security", "import-boundary", "process-lifecycle", "large-contract-26",
  "emerging-b3", "emerging-d5", "emerging-mixed", "known-browser-five",
];
const APPROVED_MANDATORY_COHORT_MEMBERS = new Map([
  ["security", ["SC-000441", "SC-000443", "SC-000555", "SC-000556", "SC-000557", "SC-000558", "SC-000559", "SC-000560"]],
  ["import-boundary", ["SC-000511", "SC-000512", "SC-000513", "SC-000514", "SC-000515"]],
  ["process-lifecycle", ["SC-000015", "SC-000017", "SC-000018", "SC-000023", "SC-000386", "SC-000387", "SC-000388", "SC-000389", "SC-000390", "SC-000420"]],
  ["large-contract-26", [
    ...Array.from({ length: 16 }, (_, index) => `SC-${String(index + 77).padStart(6, "0")}`),
    ...Array.from({ length: 10 }, (_, index) => `SC-${String(index + 356).padStart(6, "0")}`),
  ]],
  ["known-browser-five", ["SC-000312", "SC-000323", "SC-000352", "SC-000353", "SC-000385"]],
]);
const CRITICALITY_CATALOG = [
  { id: "AGENTS_WINDOWS_SANDBOX", citation: "AGENTS.md \u00a72" },
  { id: "AGENTS_DATABASE_OWNERSHIP", citation: "AGENTS.md \u00a76" },
  { id: "AGENTS_SECURITY", citation: "AGENTS.md \u00a77" },
  { id: "TESTING_SOURCE_CONTRACT_REPLACEMENT", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#source-contract-replacement" },
  { id: "TESTING_BROWSER_COMPONENT_OWNERSHIP", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#browser-and-component-ownership" },
  { id: "TESTING_COVERAGE_FLAKE_QUARANTINE", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#coverage-flake-and-quarantine-policy" },
];
const CRITICALITY_SOURCES = new Map(CRITICALITY_CATALOG.map((source) => [source.id, source.citation]));
const NORMATIVE_SOURCE_CATALOG = [
  { id: "AGENTS_WINDOWS_SANDBOX", citation: "AGENTS.md \u00a72", scope: "Windows commands, Tauri MCP development, actual Vite URL discovery, sandbox fallback, and expected missing-IPC browser-console behavior." },
  { id: "AGENTS_DATABASE_OWNERSHIP", citation: "AGENTS.md \u00a76", scope: "Additive SQLite migrations, one backend-owned connection strategy, and no frontend sql:* permissions." },
  { id: "AGENTS_SECURITY", citation: "AGENTS.md \u00a77", scope: "OS secure storage for saved LLM keys with profile and provider-plus-normalized-origin binding." },
  { id: "TESTING_SOURCE_CONTRACT_REPLACEMENT", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#source-contract-replacement", scope: "Replace valuable invariants with public behavior, structured architecture, or maintained tools; retire implementation text with specific reviewed decisions." },
  { id: "TESTING_BROWSER_COMPONENT_OWNERSHIP", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#browser-and-component-ownership", scope: "Chromium ownership belongs to Playwright, while component behavior belongs to jsdom; critical browser scenarios and navigation/error rendering remain explicit." },
  { id: "TESTING_COVERAGE_FLAKE_QUARANTINE", citation: "docs/superpowers/specs/2026-08-01-testing-infrastructure-redesign-design.md#coverage-flake-and-quarantine-policy", scope: "Critical process-cleanup modules require deterministic cleanup and explicit Windows capability failures." },
];
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
const BLIND_RESULT_KEYS = new Set(["id", "sourceHash", "class", "reason", "ownerEvidence", "criticalityRef"]);
const BLIND_OWNER_CLASSES = new Set(["D4_DUPLICATE_EVIDENCE", "A1_EXISTING_STRUCTURED_OWNER", "T1_EXISTING_TOOL_OWNER", "B1_EXISTING_BEHAVIOR_OWNER"]);
const BLIND_FUTURE_CLASSES = new Set(["B2_NEW_CHEAP_BEHAVIOR", "B3_PROTECTED_EXPENSIVE_BEHAVIOR"]);
const LARGE_CONTRACT_IDS = new Set(APPROVED_MANDATORY_COHORT_MEMBERS.get("large-contract-26"));
const TAIL_FAMILY_RULES = [
  { name: "testing/process infrastructure", matches: (value) => value.startsWith("scripts/") },
  { name: "analysis", matches: (value) => value.includes("analysis") },
  { name: "projects/library", matches: (value) => /research-projects|project-|projects\/|library/.test(value) },
  { name: "prompt packs/YouTube", matches: (value) => /prompt-pack|youtube-summary/.test(value) },
  { name: "Gemini browser", matches: (value) => /gemini-browser|provider-test-console/.test(value) },
  { name: "Rust/security boundaries", matches: (value) => /crate|rust-workspace|tauri-security|external-process|hidden-child|focused-rust/.test(value) },
  { name: "other product", matches: () => true },
];

const compareText = (left, right) => String(left).localeCompare(String(right));
const compareId = (left, right) => Number(String(left).slice(3)) - Number(String(right).slice(3)) || compareText(left, right);
const own = (value, key) => Object.prototype.hasOwnProperty.call(value ?? {}, key);
const normalizedPath = (value) => String(value ?? "").replaceAll("\\", "/").replace(/^\.\//, "");
const clone = (value) => structuredClone(value);

/** Derive the frozen Task 4 tail-family manifests in first-match order. */
export function deriveTailFamilyManifests({ baseLedger, baseTrackedPaths } = {}) {
  const openIds = baseOpenIdsFor(baseLedger, baseTrackedPaths);
  const families = TAIL_FAMILY_RULES.map(({ name }) => ({ name, paths: [], rowIds: [] }));
  for (const row of baseLedger?.rows ?? []) {
    if (!openIds.has(row?.id) || LARGE_CONTRACT_IDS.has(row.id)) continue;
    const normalized = normalizedPath(row.path).toLowerCase();
    const index = TAIL_FAMILY_RULES.findIndex(({ matches }) => matches(normalized));
    families[index].rowIds.push(row.id);
    families[index].paths.push(normalizedPath(row.path));
  }
  return families.map((family) => ({
    name: family.name,
    paths: [...new Set(family.paths)].sort(compareText),
    rowIds: [...family.rowIds].sort(compareId),
  }));
}

/** Serialize JSON deterministically without whitespace or a trailing newline. */
export function canonicalJson(value) {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  return `{${Object.keys(value).sort(compareText).map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
}

export function sha256Text(value) {
  return createHash("sha256").update(String(value)).digest("hex");
}

function evidenceBuffer(value) {
  if (Buffer.isBuffer(value)) return value;
  if (value instanceof Uint8Array) return Buffer.from(value);
  if (typeof value === "string") return Buffer.from(value, "utf8");
  return undefined;
}

function evidenceText(value) {
  return evidenceBuffer(value)?.toString("utf8");
}

function sha256Evidence(value) {
  const bytes = evidenceBuffer(value);
  return bytes ? createHash("sha256").update(bytes).digest("hex") : undefined;
}

function evidenceReferences(artifact) {
  const references = [];
  const reviewer = artifact?.independentReview?.reviewer;
  references.push({ label: "merged packet", path: reviewer?.packetPath });
  references.push({ label: "merged output", path: reviewer?.outputPath });
  for (const [index, shard] of (artifact?.independentReview?.shards ?? []).entries()) {
    references.push({ label: `shard ${index} packet`, path: shard?.packetPath });
    references.push({ label: `shard ${index} output`, path: shard?.outputPath });
  }
  if (artifact?.tailReview?.status === "accepted") {
    const tailReviewer = artifact.tailReview.reviewer;
    references.push({ label: "tail merged packet", path: tailReviewer?.packetPath });
    references.push({ label: "tail merged output", path: tailReviewer?.outputPath });
    for (const [index, shard] of (artifact.tailReview.packetShards ?? []).entries()) {
      references.push({ label: `tail shard ${index} packet`, path: shard?.packetPath });
      references.push({ label: `tail shard ${index} output`, path: shard?.outputPath });
    }
  }
  if (["accepted", "blocked"].includes(artifact?.tailReview?.status)) {
    for (const entry of artifact.tailReview.iterationHistory ?? []) {
      references.push({ label: `tail iteration ${entry?.iteration} merged packet`, path: entry?.reviewer?.packetPath });
      references.push({ label: `tail iteration ${entry?.iteration} merged output`, path: entry?.reviewer?.outputPath });
      for (const [index, shard] of (entry?.packetShards ?? []).entries()) {
        references.push({ label: `tail iteration ${entry?.iteration} shard ${index} packet`, path: shard?.packetPath });
        references.push({ label: `tail iteration ${entry?.iteration} shard ${index} output`, path: shard?.outputPath });
      }
    }
  }
  return references;
}

function confinedEvidencePath(value) {
  if (typeof value !== "string" || !value || value.includes("\\") || path.isAbsolute(value)) return false;
  const normalized = path.posix.normalize(value);
  return normalized === value
    && path.posix.dirname(normalized) === REVIEW_EVIDENCE_DIRECTORY
    && !path.posix.basename(normalized).startsWith(".");
}

/** Load only artifact-referenced evidence after proving every path stays in the committed evidence directory. */
export async function loadReviewEvidence({ repoRoot, artifact }) {
  const evidenceBytes = new Map();
  const issues = [];
  for (const reference of evidenceReferences(artifact)) {
    if (!confinedEvidencePath(reference.path)) {
      issues.push(`independentReview: ${reference.label} path must be confined to ${REVIEW_EVIDENCE_DIRECTORY}`);
      continue;
    }
    const absolute = path.resolve(repoRoot, ...reference.path.split("/"));
    const evidenceRoot = path.resolve(repoRoot, ...REVIEW_EVIDENCE_DIRECTORY.split("/"));
    if (path.dirname(absolute) !== evidenceRoot) {
      issues.push(`independentReview: ${reference.label} path must be confined to ${REVIEW_EVIDENCE_DIRECTORY}`);
      continue;
    }
    try {
      evidenceBytes.set(reference.path, await readFile(absolute));
    } catch (error) {
      if (error?.code === "ENOENT") issues.push(`independentReview: ${reference.label} evidence file is missing`);
      else issues.push(`independentReview: ${reference.label} evidence could not be read: ${error instanceof Error ? error.message : error}`);
    }
  }
  return { evidenceBytes, issues: issueList(issues) };
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

/** Validate the sole approved max-three group-boundary adjudication. */
export function validateRuleAdjudications({ independentReview, decisions } = {}) {
  const issues = [];
  const adjudications = independentReview?.ruleAdjudications ?? [];
  const hasAdjudications = Array.isArray(adjudications) && adjudications.length > 0;
  if (independentReview?.validIterations === 3 || hasAdjudications) {
    if (canonicalJson(adjudications) !== canonicalJson(APPROVED_RULE_ADJUDICATIONS)) {
      issues.push("independentReview: ruleAdjudications must equal the approved max-three allowlist");
    }
    if (independentReview?.validIterations !== 3) {
      issues.push("independentReview: ruleAdjudications require validIterations 3");
    }
    if (canonicalJson(independentReview?.disagreements ?? []) !== canonicalJson(APPROVED_RULE_DISAGREEMENTS)) {
      issues.push("independentReview: retained adjudication disagreements must equal the approved groups");
    }
    const decisionsById = new Map((decisions ?? []).map((decision) => [decision.id, decision]));
    const blindsById = new Map((independentReview?.blindResults ?? []).map((blind) => [blind.id, blind]));
    for (const adjudication of APPROVED_RULE_ADJUDICATIONS) {
      for (const id of adjudication.rowIds) {
        if (decisionsById.get(id)?.class !== adjudication.finalClass) {
          issues.push(`${id}: adjudicated final class must be ${adjudication.finalClass}`);
        }
        if (blindsById.get(id)?.class !== adjudication.finalClass) {
          issues.push(`${id}: retained blind fingerprint must equal adjudicated final class ${adjudication.finalClass}`);
        }
      }
    }
  }
  return issueList(issues);
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
  const requiredD3Reason = REQUIRED_D3_REASONS.get(decision.id);
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
      if (typeof subgroup.deletionReason === "string" && (subgroup.deletionReason.trim() === decision.class || (!subgroup.deletionReason.includes(decision.class) && !requiredD3Reason))) issues.push(`${prefix} deletion reason must contain row-specific text`);
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
  if (requiredD3Reason && (decision.class !== "D3_NON_OBSERVABLE_VISUAL"
    || decision.reason !== requiredD3Reason
    || resolution?.deletionReason !== `D3 non-observable-visual: ${requiredD3Reason}`)) {
    issues.push(`${prefix} must use the exact approved D3 reason and deletion prefix`);
  }
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
  if (artifact.scope?.openRows === 436) {
    const baseRowById = new Map((baseLedger?.rows ?? []).map((row) => [row.id, row]));
    const beforeByDisposition = {};
    const afterByDisposition = {};
    for (const decision of artifact.decisions ?? []) {
      const before = baseRowById.get(decision.id)?.subgroups ? "mixed" : baseRowById.get(decision.id)?.disposition;
      if (before) beforeByDisposition[before] = (beforeByDisposition[before] ?? 0) + 1;
      const after = decision.resolution?.subgroups ? "mixed" : decision.resolution?.disposition;
      if (after) afterByDisposition[after] = (afterByDisposition[after] ?? 0) + 1;
    }
    if (canonicalJson(forecast.beforeByDisposition) !== canonicalJson(beforeByDisposition)) issues.push("forecast: beforeByDisposition does not match frozen decisions");
    if (canonicalJson(forecast.afterByDisposition) !== canonicalJson(afterByDisposition)) issues.push("forecast: afterByDisposition does not match review decisions");
    if (forecast.removableLegacyFiles !== baseFiles?.length) issues.push("forecast: removableLegacyFiles must equal the pinned legacy file count");
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

function validateAcceptedLoss(artifact, issues) {
  if (artifact.scope?.openRows !== 436) return;
  const d5 = (artifact.decisions ?? []).filter((decision) => decision.class === "D5_ACCEPTED_LOSS").sort((left, right) => compareId(left.id, right.id));
  const items = d5.flatMap((decision) => (decision.lostBehavior ?? []).map((item) => ({
    id: decision.id,
    assertionOrdinals: item.assertionOrdinals,
    behavior: item.behavior,
  }))).sort((left, right) => compareId(left.id, right.id) || (left.assertionOrdinals?.[0] ?? 0) - (right.assertionOrdinals?.[0] ?? 0));
  const expected = {
    rows: d5.length,
    assertionOrdinals: items.reduce((total, item) => total + (item.assertionOrdinals?.length ?? 0), 0),
    items,
  };
  if (canonicalJson(artifact.acceptedLoss) !== canonicalJson(expected)) issues.push("acceptedLoss: summary does not match D5 decisions");
}

function expectedSample(decisions, protectedIds) {
  const population = decisions
    .filter((decision) => decision.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && decision.class !== "D5_ACCEPTED_LOSS" && !protectedIds.has(decision.id) && !decision.resolution?.subgroups)
    .map((decision) => decision.id).sort(compareId);
  return { population, rowIds: [...population].sort((left, right) => compareText(sha256Text(left), sha256Text(right))).slice(0, Math.ceil(population.length * 0.1)) };
}

function expectedTailReviewPopulation(artifact) {
  const tailIds = new Set((artifact.tailFamilies ?? []).flatMap((family) => family?.rowIds ?? []));
  const excluded = new Set([
    ...LARGE_CONTRACT_IDS,
    ...(artifact.protectedRows ?? []).map((row) => row.id),
    ...(artifact.independentReview?.calibrations ?? []).flatMap((cohort) => cohort?.rowIds ?? []),
    ...(artifact.independentReview?.mandatoryCohorts ?? [])
      .filter((cohort) => ["security", "import-boundary", "process-lifecycle"].includes(cohort?.name))
      .flatMap((cohort) => cohort?.rowIds ?? []),
  ]);
  const b3RowIds = (artifact.decisions ?? []).filter((decision) => decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR").map((decision) => decision.id).sort(compareId);
  const d5RowIds = (artifact.decisions ?? []).filter((decision) => decision.class === "D5_ACCEPTED_LOSS").map((decision) => decision.id).sort(compareId);
  const mixedRowIds = (artifact.decisions ?? []).filter((decision) => Array.isArray(decision.resolution?.subgroups)).map((decision) => decision.id).sort(compareId);
  for (const id of [...b3RowIds, ...d5RowIds, ...mixedRowIds]) excluded.add(id);
  const population = (artifact.decisions ?? []).map((decision) => decision.id).filter((id) => tailIds.has(id) && !excluded.has(id)).sort(compareId);
  const rowIds = [...population].sort((left, right) => compareText(sha256Text(left), sha256Text(right))).slice(0, Math.ceil(population.length * 0.1));
  const requiredBlindRowIds = [...new Set([...rowIds, ...b3RowIds, ...d5RowIds, ...mixedRowIds])].sort(compareId);
  return { population, rowIds, b3RowIds, d5RowIds, mixedRowIds, requiredBlindRowIds };
}

function validateTailReviewScaffold(artifact, issues) {
  if (artifact.scope?.openRows !== 436) return;
  const tail = artifact.tailReview;
  if (!tail || typeof tail !== "object") {
    issues.push("tailReview: missing");
    return;
  }
  const expected = expectedTailReviewPopulation(artifact);
  if (!tail.authorRunId || tail.authorRunId === artifact.independentReview?.authorRunId || tail.authorRunId === tail.reviewerRunId) issues.push("tailReview: separate author and reviewer run IDs are required");
  if (!Number.isInteger(tail.tailValidIterations) || tail.tailValidIterations < 0 || tail.tailValidIterations > 3 || tail.tailValidIterations !== tail.deterministicSample?.iterations) issues.push("tailReview: tailValidIterations must be 0..3 and equal sample iterations");
  const sample = tail.deterministicSample;
  if (sample?.algorithm !== "sha256-id-lowest-10-percent"
    || canonicalJson(sample?.population) !== canonicalJson(expected.population)
    || sample?.populationDigest !== sha256Text(canonicalJson(expected.population))
    || canonicalJson(sample?.rowIds) !== canonicalJson(expected.rowIds)) issues.push("tailReview: deterministic sample must equal the current tail population");
  if (canonicalJson(tail.riskCohorts) !== canonicalJson({ b3RowIds: expected.b3RowIds, d5RowIds: expected.d5RowIds, mixedRowIds: expected.mixedRowIds })) issues.push("tailReview: risk cohorts must equal all B3, D5, and mixed decisions");
  if (canonicalJson(tail.requiredBlindRowIds) !== canonicalJson(expected.requiredBlindRowIds)
    || tail.requiredPopulationDigest !== sha256Text(canonicalJson(expected.requiredBlindRowIds))) issues.push("tailReview: required blind population is stale");
  const shards = Array.isArray(tail.packetShards) ? tail.packetShards : [];
  const covered = [];
  const hashPattern = /^[0-9a-f]{64}$/;
  for (let index = 0; index < shards.length; index += 1) {
    const shard = shards[index];
    if (shard?.index !== index || !Array.isArray(shard?.rowIds) || shard.rowIds.length < 1 || shard.rowIds.length > 24) issues.push(`tailReview: packet shard ${index} must be contiguous and contain 1..24 rows`);
    covered.push(...(shard?.rowIds ?? []));
    if (!hashPattern.test(shard?.packetSha256 ?? "") || typeof shard?.packetPath !== "string" || !shard.packetPath.includes(shard.packetSha256)) issues.push(`tailReview: packet shard ${index} requires content-addressed evidence`);
    if (["accepted", "blocked"].includes(tail.status)) {
      validateEvidencePath({ value: shard?.outputPath, hash: shard?.outputSha256, label: `output shard ${index} output`, scope: "tailReview", issues });
    }
  }
  if (canonicalJson(covered) !== canonicalJson(expected.requiredBlindRowIds)) issues.push("tailReview: packet shards must exactly cover required blind IDs in numeric order");
  if (tail.status === "awaiting_blind_review") issues.push("tailReview: awaiting independent review");
  else if (!["accepted", "blocked"].includes(tail.status)) issues.push("tailReview: unsupported status");
  issues.push(...validateTailIterationProtocol({ tailReview: tail }));
}

/** Validate the standalone, bounded Task 4 tail fixed-point history. */
export function validateTailIterationProtocol({ tailReview: tail } = {}) {
  const issues = [];
  const count = tail?.tailValidIterations;
  const history = Array.isArray(tail?.iterationHistory) ? tail.iterationHistory : [];
  const hashPattern = /^[0-9a-f]{64}$/;
  if (!Number.isInteger(count) || count < 0 || count > 3 || count !== tail?.deterministicSample?.iterations) issues.push("tailReview: tailValidIterations must be 0..3 and equal sample iterations");
  if (count > 3 || history.length > 3) issues.push("tailReview: a fourth valid tail iteration is forbidden");
  if (!Number.isInteger(count) || count < 0 || count > 3 || history.length !== count
    || history.some((entry, index) => entry?.iteration !== index + 1)) {
    issues.push("tailReview: iteration history must be contiguous and complete");
  }
  for (const [index, entry] of history.entries()) {
    const setIsNumericStable = (items) => Array.isArray(items)
      && items.every((id) => typeof id === "string" && /^SC-[0-9]{6}$/.test(id))
      && new Set(items).size === items.length
      && canonicalJson(items) === canonicalJson([...items].sort(compareId));
    const sampleSelectionFor = (population) => [...population]
      .sort((left, right) => compareText(sha256Text(left), sha256Text(right)))
      .slice(0, Math.ceil(population.length * 0.1));
    const populationSetsComplete = setIsNumericStable(entry?.requiredBlindRowIds)
      && setIsNumericStable(entry?.samplePopulation)
      && Array.isArray(entry?.sampleRowIds)
      && setIsNumericStable(entry?.resultingRequiredBlindRowIds)
      && setIsNumericStable(entry?.resultingSamplePopulation)
      && Array.isArray(entry?.resultingSampleRowIds);
    if (!populationSetsComplete) issues.push(`tailReview: iteration ${index + 1} population sets metadata is incomplete`);
    const inputSampleSelection = populationSetsComplete ? sampleSelectionFor(entry.samplePopulation) : [];
    const resultSampleSelection = populationSetsComplete ? sampleSelectionFor(entry.resultingSamplePopulation) : [];
    if (populationSetsComplete && canonicalJson(entry.sampleRowIds) !== canonicalJson(inputSampleSelection)) issues.push(`tailReview: iteration ${index + 1} input sample selection must equal deterministic ten percent`);
    if (populationSetsComplete && canonicalJson(entry.resultingSampleRowIds) !== canonicalJson(resultSampleSelection)) issues.push(`tailReview: iteration ${index + 1} resulting sample selection must equal deterministic ten percent`);
    if (populationSetsComplete && entry.requiredPopulationDigest !== sha256Text(canonicalJson(entry.requiredBlindRowIds))) issues.push(`tailReview: iteration ${index + 1} required population digest mismatch`);
    if (populationSetsComplete && entry.samplePopulationDigest !== sha256Text(canonicalJson(entry.samplePopulation))) issues.push(`tailReview: iteration ${index + 1} sample population digest mismatch`);
    if (populationSetsComplete && entry.resultingRequiredPopulationDigest !== sha256Text(canonicalJson(entry.resultingRequiredBlindRowIds))) issues.push(`tailReview: iteration ${index + 1} resulting required population digest mismatch`);
    if (populationSetsComplete && entry.resultingSamplePopulationDigest !== sha256Text(canonicalJson(entry.resultingSamplePopulation))) issues.push(`tailReview: iteration ${index + 1} resulting sample population digest mismatch`);
    const requiredSetChanged = populationSetsComplete && canonicalJson(entry.requiredBlindRowIds) !== canonicalJson(entry.resultingRequiredBlindRowIds);
    const sampleSetChanged = populationSetsComplete && (canonicalJson(entry.samplePopulation) !== canonicalJson(entry.resultingSamplePopulation)
      || canonicalJson(entry.sampleRowIds) !== canonicalJson(entry.resultingSampleRowIds));
    const populationChangeSupported = sampleSetChanged;
    const changed = entry?.task4RuleChanged === true || entry?.samplePopulationChanged === true;
    if (!hashPattern.test(entry?.requiredPopulationDigest ?? "") || !hashPattern.test(entry?.samplePopulationDigest ?? "")
      || typeof entry?.task4RuleChanged !== "boolean" || typeof entry?.samplePopulationChanged !== "boolean"
      || entry?.result !== (changed ? "rule_changed" : "fixed_point")) {
      issues.push(`tailReview: iteration ${index + 1} record is incomplete or inconsistent`);
    }
    if (entry?.samplePopulationChanged !== populationChangeSupported) issues.push(`tailReview: iteration ${index + 1} samplePopulationChanged is not supported by resulting population digests`);
    if (entry?.result === "fixed_point" && (entry?.task4RuleChanged || entry?.samplePopulationChanged || requiredSetChanged || sampleSetChanged
      || entry?.requiredPopulationDigest !== entry?.resultingRequiredPopulationDigest || entry?.samplePopulationDigest !== entry?.resultingSamplePopulationDigest)) {
      issues.push(`tailReview: iteration ${index + 1} fixed point must preserve all input population sets`);
    }
    if (index < history.length - 1) {
      if (!changed) issues.push("tailReview: only the final valid iteration may be a fixed point");
      const next = history[index + 1];
      if (canonicalJson(next?.requiredBlindRowIds) !== canonicalJson(entry?.resultingRequiredBlindRowIds)
        || canonicalJson(next?.samplePopulation) !== canonicalJson(entry?.resultingSamplePopulation)
        || canonicalJson(next?.sampleRowIds) !== canonicalJson(entry?.resultingSampleRowIds)
        || next?.samplePopulationDigest !== entry?.resultingSamplePopulationDigest || next?.requiredPopulationDigest !== entry?.resultingRequiredPopulationDigest) {
        issues.push(`tailReview: iteration ${index + 2} input population sets must equal iteration ${index + 1} results`);
      }
    }
  }
  const final = history.at(-1);
  if (final && (final.requiredPopulationDigest !== tail?.requiredPopulationDigest
    || final.samplePopulationDigest !== tail?.deterministicSample?.populationDigest)) {
    issues.push("tailReview: final iteration must compare the current tail population");
  }
  const finalChanged = final?.task4RuleChanged === true || final?.samplePopulationChanged === true;
  if (tail?.status === "accepted" && (!final || finalChanged || final.result !== "fixed_point")) issues.push("tailReview: accepted tail history must end at an unchanged fixed point");
  if (tail?.status === "blocked") {
    if (count !== 3 || !finalChanged || final?.result !== "rule_changed") issues.push("tailReview: blocked state requires a changing third valid iteration");
    else issues.push("tailReview: third valid tail iteration changed rules or population; explicit rule amendment required");
  }
  if (Array.isArray(tail?.ruleAdjudications) && tail.ruleAdjudications.length) issues.push("tailReview: Task 3 adjudications cannot authorize tail disagreements");
  if (!Array.isArray(tail?.invalidAttempts) || tail.invalidAttempts.some((attempt) => !Number.isInteger(attempt?.shardIndex)
    || attempt.shardIndex < 0 || !hashPattern.test(attempt?.sha256 ?? "") || typeof attempt?.reason !== "string" || !attempt.reason.trim())) {
    issues.push("tailReview: invalid attempts must be well-formed non-counting evidence");
  }
  return issueList(issues);
}

/** Keep blind coverage of Task 3 rows non-authoritative while enforcing Task 4 convergence. */
export function validateTailFingerprints({ tailReview, decisions, protectedIds = new Set() } = {}) {
  const issues = [];
  const decisionById = new Map((decisions ?? []).map((decision) => [decision.id, decision]));
  const blindById = new Map((tailReview?.blindResults ?? []).map((blind) => [blind.id, blind]));
  const requiredIds = tailReview?.requiredBlindRowIds ?? [...blindById.keys()].sort(compareId);
  const mismatches = [];
  const task4Mismatches = [];
  for (const id of requiredIds) {
    const decision = decisionById.get(id);
    const blind = blindById.get(id);
    if (!decision || !blind) {
      issues.push(`${id}: tail fingerprint requires both author and blind decisions`);
      continue;
    }
    const task4Authored = decision.authorRunId === tailReview?.authorRunId;
    if (reviewFingerprint(decision, protectedIds.has(id), task4Authored) !== reviewFingerprint(blind, protectedIds.has(id), task4Authored)) {
      mismatches.push(id);
      if (task4Authored) task4Mismatches.push(id);
    }
  }
  const coverageOnly = mismatches.filter((id) => !task4Mismatches.includes(id)).sort(compareId);
  if (canonicalJson(tailReview?.coverageOnlyTask3MismatchIds ?? []) !== canonicalJson(coverageOnly)) {
    issues.push("tailReview: coverage-only Task 3 mismatch IDs must equal the derived non-authoritative set");
  }
  if (task4Mismatches.length) issues.push(`tailReview: Task 4-authored fingerprint mismatches are not accepted: ${task4Mismatches.sort(compareId).join(", ")}`);
  return issueList(issues);
}

/** Validate ignored author-side tail packets before any blind reviewer sees them. */
export function validateTailPacketScaffold({ artifact, baseLedger, packetBytes } = {}) {
  const issues = [];
  if (!(packetBytes instanceof Map) || !baseLedger || !artifact?.tailReview) return ["tailReview: invalid packet scaffold input"];
  const tail = artifact.tailReview;
  const openIds = new Set((artifact.decisions ?? []).map((decision) => decision.id));
  const baseRowById = new Map((baseLedger.rows ?? []).map((row) => [row.id, row]));
  const resolvedOwnerRows = new Map();
  for (const row of baseLedger.rows ?? []) {
    if (openIds.has(row.id)) continue;
    for (const owner of resolutionIds(row)) {
      const ids = resolvedOwnerRows.get(owner) ?? [];
      ids.push(row.id);
      resolvedOwnerRows.set(owner, ids);
    }
  }
  const topKeys = ["normativeSourceCatalog", "orderedClassCatalog", "packetSchemaVersion", "requiredPopulationDigest", "reviewerRunId", "rows", "shardCount", "shardIndex"];
  const rowKeys = ["assertionOrdinals", "candidateOwners", "declaration", "id", "invariant", "sourceHash"];
  const candidateKeys = new Set(["capability", "closedRowIds", "id", "mechanism", "normativeSource", "resolvedByBaseClosedRow", "status"]);
  const mechanismFor = (id) => id?.startsWith("test:vitest:") ? "jsdom"
    : id?.startsWith("test:cargo:") ? "cargo"
      : id?.startsWith("test:playwright:") ? "playwright"
        : id?.startsWith("rule:") ? "structured-rule"
          : id?.startsWith("tool:") ? "tool" : "unknown";

  for (const [index, reference] of (tail.packetShards ?? []).entries()) {
    const bytes = packetBytes.get(reference.packetPath);
    const text = evidenceText(bytes);
    if (text === undefined) {
      issues.push(`tailReview: packet shard ${index} bytes are missing`);
      continue;
    }
    if (sha256Evidence(bytes) !== reference.packetSha256) issues.push(`tailReview: packet shard ${index} byte hash mismatch`);
    let packet;
    try { packet = JSON.parse(text); } catch { issues.push(`tailReview: packet shard ${index} is malformed JSON`); continue; }
    if (canonicalJson(Object.keys(packet).sort(compareText)) !== canonicalJson(topKeys)) issues.push(`tailReview: packet shard ${index} top-level schema mismatch`);
    if (packet.packetSchemaVersion !== REVIEW_PACKET_SCHEMA_VERSION || packet.reviewerRunId !== tail.reviewerRunId
      || packet.shardIndex !== index || packet.shardCount !== tail.packetShards.length
      || packet.requiredPopulationDigest !== tail.requiredPopulationDigest) issues.push(`tailReview: packet shard ${index} metadata mismatch`);
    if (canonicalJson(packet.orderedClassCatalog) !== canonicalJson(REASON_CLASS_CATALOG)
      || canonicalJson(packet.normativeSourceCatalog) !== canonicalJson(NORMATIVE_SOURCE_CATALOG)) issues.push(`tailReview: packet shard ${index} catalog mismatch`);
    const rows = Array.isArray(packet.rows) ? packet.rows : [];
    if (canonicalJson(rows.map((row) => row?.id)) !== canonicalJson(reference.rowIds)) issues.push(`tailReview: packet shard ${index} row coverage/order mismatch`);
    const seen = new Set();
    for (const row of rows) {
      if (canonicalJson(Object.keys(row ?? {}).sort(compareText)) !== canonicalJson(rowKeys)) issues.push(`${row?.id}: tail packet row schema leaks author/reviewer fields`);
      if (seen.has(row?.id)) issues.push(`${row?.id}: duplicate tail packet row`);
      seen.add(row?.id);
      const baseRow = baseRowById.get(row?.id);
      if (!baseRow || row.sourceHash !== baseRow.sourceHash || row.invariant !== baseRow.invariant) issues.push(`${row?.id}: tail packet frozen row binding mismatch`);
      const ordinals = Array.from({ length: baseRow?.assertionCount ?? 0 }, (_, ordinal) => ordinal + 1);
      if (canonicalJson(row.assertionOrdinals) !== canonicalJson(ordinals)) issues.push(`${row?.id}: tail packet assertion ordinals mismatch`);
      if (typeof row.declaration !== "string" || sha256Text(row.declaration.replace(/\r\n?/g, "\n")) !== row.sourceHash) issues.push(`${row?.id}: tail packet declaration hash mismatch`);
      const expectedOwnerIds = [...new Set(baseRow?.replacementIds ?? [])];
      const candidates = Array.isArray(row.candidateOwners) ? row.candidateOwners : [];
      if (canonicalJson(candidates.map((candidate) => candidate?.id)) !== canonicalJson(expectedOwnerIds)) issues.push(`${row?.id}: tail packet candidate inventory mismatch`);
      for (const candidate of candidates) {
        if (!candidate || typeof candidate !== "object" || Object.keys(candidate).some((key) => !candidateKeys.has(key))) issues.push(`${row?.id}: tail packet candidate schema mismatch`);
        const closedRowIds = resolvedOwnerRows.get(candidate?.id);
        const expected = {
          id: candidate?.id,
          mechanism: mechanismFor(candidate?.id),
          capability: baseRow?.invariant,
          status: closedRowIds?.length ? "resolved" : candidate?.id?.startsWith("test:playwright:") ? "approved-future" : "future",
          ...(closedRowIds?.length ? { resolvedByBaseClosedRow: true, closedRowIds } : {}),
          ...(candidate?.id?.startsWith("test:playwright:") ? { normativeSource: APPROVED_PLAYWRIGHT_CRITICALITY } : {}),
        };
        if (canonicalJson(candidate) !== canonicalJson(expected)) issues.push(`${row?.id}: tail packet candidate status/citation mismatch`);
        if (candidate?.id?.startsWith("test:playwright:") && APPROVED_PLAYWRIGHT_OWNERS.get(row.id) !== candidate.id) issues.push(`${row?.id}: tail packet Playwright candidate is not allowlisted`);
      }
    }
  }
  return issueList(issues);
}

/** Validate blind tail output transport before it can become accepted evidence. */
export function validateTailOutputScaffold({ artifact, outputs, outputBytes, packetBytes = outputBytes } = {}) {
  const issues = [];
  if (!(outputBytes instanceof Map) || !(packetBytes instanceof Map) || !artifact?.tailReview || !Array.isArray(outputs)) return ["tailReview: invalid output scaffold input"];
  const decisionById = new Map((artifact.decisions ?? []).map((decision) => [decision.id, decision]));
  const topKeys = ["packetSha256", "results", "reviewerRunId", "stopConditions"];
  for (const [index, reference] of outputs.entries()) {
    validateEvidencePath({ value: reference?.outputPath, hash: reference?.outputSha256, label: `output shard ${index} output`, scope: "tailReview", issues });
    const bytes = outputBytes.get(reference.outputPath);
    const text = evidenceText(bytes);
    if (text === undefined) { issues.push(`tailReview: output shard ${index} bytes are missing`); continue; }
    if (sha256Evidence(bytes) !== reference.outputSha256) issues.push(`tailReview: output shard ${index} byte hash mismatch`);
    let output;
    try { output = JSON.parse(text); } catch { issues.push(`tailReview: output shard ${index} is malformed JSON`); continue; }
    if (canonicalJson(Object.keys(output).sort(compareText)) !== canonicalJson(topKeys)) issues.push(`tailReview: output shard ${index} top-level schema mismatch`);
    const packet = artifact.tailReview.packetShards[index];
    let packetBody;
    try { packetBody = JSON.parse(evidenceText(packetBytes.get(packet?.packetPath))); } catch { issues.push(`tailReview: output shard ${index} packet evidence is missing or malformed`); }
    if (output.reviewerRunId !== artifact.tailReview.reviewerRunId || output.packetSha256 !== packet?.packetSha256) issues.push(`tailReview: output shard ${index} reviewer/packet binding mismatch`);
    if (!Array.isArray(output.stopConditions) || output.stopConditions.length) issues.push(`tailReview: output shard ${index} stopConditions must be empty`);
    const results = Array.isArray(output.results) ? output.results : [];
    if (results.length !== (packet?.rowIds ?? []).length) issues.push(`tailReview: output shard ${index} result count mismatch`);
    for (let resultIndex = 0; resultIndex < results.length; resultIndex += 1) {
      const result = results[resultIndex];
      const id = packet?.rowIds?.[resultIndex];
      const decision = decisionById.get(id);
      const packetRow = packetBody?.rows?.[resultIndex];
      const candidates = new Map((packetRow?.candidateOwners ?? []).map((candidate) => [candidate.id, candidate]));
      if (Object.keys(result ?? {}).some((key) => !BLIND_RESULT_KEYS.has(key))) issues.push(`${result?.id}: tail output result schema leaks author resolution fields`);
      if (result?.id !== id || result?.sourceHash !== decision?.sourceHash) issues.push(`tailReview: output shard ${index} result order/source binding mismatch`);
      if (!CLASS_DISPOSITIONS.has(result?.class) || typeof result?.reason !== "string" || !result.reason.trim()) issues.push(`${result?.id}: tail output requires a supported class and substantive reason`);
      const selected = Array.isArray(result?.ownerEvidence) ? result.ownerEvidence : [];
      if (result?.ownerEvidence !== undefined && (!selected.length || selected.some((owner) => typeof owner !== "string" || !owner.trim()))) issues.push(`${result?.id}: tail output ownerEvidence is invalid`);
      if (selected.some((owner) => !candidates.has(owner))) issues.push(`${result?.id}: tail output selected an invented candidate owner`);
      if (BLIND_OWNER_CLASSES.has(result?.class) && (!selected.length || selected.some((owner) => {
        const candidate = candidates.get(owner);
        return !(candidate?.resolvedByBaseClosedRow === true || candidate?.status === "resolved");
      }))) issues.push(`${result?.id}: tail output existing-owner class requires resolved candidate membership`);
      if (BLIND_FUTURE_CLASSES.has(result?.class) && (!selected.length || selected.some((owner) => !["future", "unresolved", "approved-future"].includes(candidates.get(owner)?.status)))) issues.push(`${result?.id}: tail output future-owner class requires future candidate membership`);
      if (result?.criticalityRef !== undefined && !CRITICALITY_SOURCES.has(result.criticalityRef)) issues.push(`${result?.id}: tail output citation is invalid`);
      if (result?.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && !CRITICALITY_SOURCES.has(result?.criticalityRef)) issues.push(`${result?.id}: tail output B3 requires an exact criticalityRef`);
      const requiredSources = [...new Set(selected.map((owner) => candidates.get(owner)?.normativeSource).filter(Boolean))];
      if (requiredSources.some((source) => result?.criticalityRef !== source)) issues.push(`${result?.id}: tail output owner requires its exact citation`);
      for (const owner of selected) if (owner.startsWith("test:playwright:")
        && (APPROVED_PLAYWRIGHT_OWNERS.get(result.id) !== owner || result.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR" || result.criticalityRef !== APPROVED_PLAYWRIGHT_CRITICALITY)) {
        issues.push(`${result.id}: tail output Playwright mapping/class/citation mismatch`);
      }
    }
  }
  return issueList(issues);
}

function prefixedIterationIssues(iteration, nestedIssues, prefix) {
  return nestedIssues.map((issue) => issue.startsWith("tailReview: ")
    ? issue.replace("tailReview: ", `tailReview: iteration ${iteration} ${prefix}`)
    : issue);
}

function validateTailIterationEvidence({ artifact, baseLedger, evidenceBytes, protectedIds, issues }) {
  const tail = artifact.tailReview;
  if (!["accepted", "blocked"].includes(tail?.status)) return;
  const history = Array.isArray(tail.iterationHistory) ? tail.iterationHistory : [];
  const seenRuns = new Set();
  const seenPaths = new Set();
  const iterationResults = [];
  for (const [historyIndex, entry] of history.entries()) {
    const iteration = historyIndex + 1;
    const reviewer = entry?.reviewer;
    const shards = Array.isArray(entry?.packetShards) ? entry.packetShards : [];
    const metadataComplete = typeof entry?.reviewerRunId === "string" && entry.reviewerRunId
      && Array.isArray(entry?.requiredBlindRowIds) && Array.isArray(entry?.task4Disagreements)
      && Array.isArray(entry?.samplePopulation) && Array.isArray(entry?.sampleRowIds)
      && Array.isArray(entry?.resultingRequiredBlindRowIds) && Array.isArray(entry?.resultingSamplePopulation) && Array.isArray(entry?.resultingSampleRowIds)
      && Array.isArray(entry?.coverageOnlyTask3MismatchIds) && reviewer && typeof reviewer === "object"
      && /^[0-9a-f]{64}$/.test(entry?.resultingRequiredPopulationDigest ?? "")
      && /^[0-9a-f]{64}$/.test(entry?.resultingSamplePopulationDigest ?? "")
      && /^[0-9a-f]{64}$/.test(entry?.mergedOutputSha256 ?? "");
    if (!metadataComplete) {
      issues.push(`tailReview: iteration ${iteration} evidence metadata is incomplete`);
      continue;
    }
    const evidencePaths = [reviewer.packetPath, reviewer.outputPath, ...shards.flatMap((shard) => [shard?.packetPath, shard?.outputPath])];
    if (seenRuns.has(entry.reviewerRunId) || evidencePaths.some((value) => seenPaths.has(value))) issues.push("tailReview: iteration reviewerRunIds and evidence references must be unique");
    seenRuns.add(entry.reviewerRunId);
    evidencePaths.forEach((value) => seenPaths.add(value));
    if (reviewer.reviewerRunId !== entry.reviewerRunId || typeof reviewer.agentTaskId !== "string" || !reviewer.agentTaskId.trim()
      || reviewer.contextPolicy !== "blind-no-proposed-class-or-reason") issues.push(`tailReview: iteration ${iteration} reviewer identity/context mismatch`);
    if (entry.requiredPopulationDigest !== sha256Text(canonicalJson(entry.requiredBlindRowIds))) issues.push(`tailReview: iteration ${iteration} required population digest mismatch`);
    const covered = shards.flatMap((shard) => shard?.rowIds ?? []);
    if (canonicalJson(covered) !== canonicalJson(entry.requiredBlindRowIds)) issues.push(`tailReview: iteration ${iteration} packet shards must exactly cover its required blind IDs`);
    if (shards.some((shard, index) => shard?.index !== index || !Array.isArray(shard?.rowIds) || shard.rowIds.length < 1 || shard.rowIds.length > 24)) issues.push(`tailReview: iteration ${iteration} packet shard indices must be contiguous and unique`);

    const iterationTail = { ...tail, reviewerRunId: entry.reviewerRunId, requiredPopulationDigest: entry.requiredPopulationDigest, packetShards: shards };
    const iterationArtifact = { ...artifact, tailReview: iterationTail };
    const nestedPrefix = historyIndex === history.length - 1 ? "" : "";
    const packetIssues = validateTailPacketScaffold({ artifact: iterationArtifact, baseLedger, packetBytes: evidenceBytes });
    const outputIssues = validateTailOutputScaffold({ artifact: iterationArtifact, outputs: shards, outputBytes: evidenceBytes, packetBytes: evidenceBytes });
    if (historyIndex === history.length - 1 && history.length === 1) {
      issues.push(...packetIssues, ...outputIssues);
    } else {
      issues.push(...prefixedIterationIssues(iteration, packetIssues, nestedPrefix), ...prefixedIterationIssues(iteration, outputIssues, nestedPrefix));
    }

    const pairs = [
      { label: "merged packet", path: reviewer.packetPath, hash: reviewer.packetSha256 },
      { label: "merged output", path: reviewer.outputPath, hash: reviewer.outputSha256 },
    ];
    for (const pair of pairs) {
      const pathIssues = [];
      validateEvidencePath({ value: pair.path, hash: pair.hash, label: pair.label, scope: "tailReview", issues: pathIssues });
      issues.push(...prefixedIterationIssues(iteration, pathIssues, ""));
      const bytes = evidenceBytes instanceof Map ? evidenceBytes.get(pair.path) : undefined;
      if (evidenceText(bytes) === undefined) issues.push(`tailReview: iteration ${iteration} ${pair.label} evidence file is missing`);
      else if (sha256Evidence(bytes) !== pair.hash) issues.push(`tailReview: iteration ${iteration} ${pair.label} byte hash mismatch`);
    }
    const parseIssues = [];
    const mergedPacket = parseEvidenceJson({ bytes: evidenceBytes?.get(reviewer.packetPath), label: `tail iteration ${iteration} merged packet`, issues: parseIssues });
    const mergedOutput = parseEvidenceJson({ bytes: evidenceBytes?.get(reviewer.outputPath), label: `tail iteration ${iteration} merged output`, issues: parseIssues });
    const shardPackets = shards.map((shard) => parseEvidenceJson({ bytes: evidenceBytes?.get(shard.packetPath), label: `tail iteration ${iteration} shard ${shard.index} packet`, issues: parseIssues })).filter(Boolean);
    const shardOutputs = shards.map((shard) => parseEvidenceJson({ bytes: evidenceBytes?.get(shard.outputPath), label: `tail iteration ${iteration} shard ${shard.index} output`, issues: parseIssues })).filter(Boolean);
    issues.push(...parseIssues.map((issue) => issue.replace(/^independentReview: tail /, "tailReview: ")));
    if (mergedPacket && (mergedPacket.shardIndex !== "merged" || mergedPacket.reviewerRunId !== entry.reviewerRunId
      || mergedPacket.requiredPopulationDigest !== entry.requiredPopulationDigest
      || canonicalJson(mergedPacket.rows) !== canonicalJson(shardPackets.flatMap((packet) => packet.rows ?? [])))) issues.push(`tailReview: iteration ${iteration} merged packet does not reconstruct its shard packets`);
    let reconstructed = [];
    if (mergedOutput) {
      if (mergedOutput.reviewerRunId !== entry.reviewerRunId || mergedOutput.packetSha256 !== reviewer.packetSha256
        || !Array.isArray(mergedOutput.stopConditions) || mergedOutput.stopConditions.length
        || canonicalJson(mergedOutput.results) !== canonicalJson(shardOutputs.flatMap((output) => output.results ?? []))) issues.push(`tailReview: iteration ${iteration} merged output does not reconstruct its shard outputs`);
      reconstructed = (mergedOutput.results ?? []).map((result) => ({ ...result, reviewerRunId: entry.reviewerRunId }));
      if (sha256Text(canonicalJson(reconstructed)) !== entry.mergedOutputSha256) issues.push(`tailReview: iteration ${iteration} merged blind-result digest mismatch`);
    }
    const blindById = new Map(reconstructed.map((result) => [result.id, result]));
    const disagreementIds = [];
    for (const disagreement of entry.task4Disagreements) {
      const decision = (artifact.decisions ?? []).find((item) => item.id === disagreement?.id);
      const blind = blindById.get(disagreement?.id);
      if (!decision || decision.authorRunId !== tail.authorRunId || !disagreement?.authorFingerprint || !blind
        || reviewFingerprint(disagreement.authorFingerprint, protectedIds.has(disagreement?.id), true) === reviewFingerprint(blind, protectedIds.has(disagreement?.id), true)) {
        issues.push(`tailReview: iteration ${iteration} has an unsupported Task 4 disagreement: ${disagreement?.id}`);
      } else disagreementIds.push(disagreement.id);
    }
    if (entry.task4RuleChanged !== (disagreementIds.length > 0)) issues.push(`tailReview: iteration ${iteration} task4RuleChanged is not supported by reconstructed disagreements`);
    const fingerprintTail = { ...tail, reviewerRunId: entry.reviewerRunId, requiredBlindRowIds: entry.requiredBlindRowIds, blindResults: reconstructed, coverageOnlyTask3MismatchIds: entry.coverageOnlyTask3MismatchIds };
    issues.push(...prefixedIterationIssues(iteration, validateTailFingerprints({ tailReview: fingerprintTail, decisions: artifact.decisions, protectedIds }), ""));
    iterationResults.push({ entry, reconstructed });
  }

  const final = iterationResults.at(-1);
  if (!final) return;
  const finalEntry = final.entry;
  if (tail.status === "accepted") {
    const finalPopulationSetsAgree = canonicalJson(tail.requiredBlindRowIds) === canonicalJson(finalEntry.resultingRequiredBlindRowIds)
      && tail.requiredPopulationDigest === finalEntry.resultingRequiredPopulationDigest
      && canonicalJson(tail.deterministicSample?.population) === canonicalJson(finalEntry.resultingSamplePopulation)
      && tail.deterministicSample?.populationDigest === finalEntry.resultingSamplePopulationDigest
      && canonicalJson(tail.deterministicSample?.rowIds) === canonicalJson(finalEntry.resultingSampleRowIds);
    if (!finalPopulationSetsAgree) issues.push("tailReview: final top-level population sets must equal the last fixed-point results");
    const finalBindingsAgree = tail.reviewerRunId === finalEntry.reviewerRunId
      && canonicalJson(tail.packetShards) === canonicalJson(finalEntry.packetShards)
      && canonicalJson(tail.reviewer) === canonicalJson(finalEntry.reviewer)
      && tail.mergedOutputSha256 === finalEntry.mergedOutputSha256
      && finalPopulationSetsAgree
      && canonicalJson(tail.coverageOnlyTask3MismatchIds) === canonicalJson(finalEntry.coverageOnlyTask3MismatchIds)
      && canonicalJson(tail.blindResults) === canonicalJson(final.reconstructed)
      && finalEntry.result === "fixed_point" && !finalEntry.task4RuleChanged && !finalEntry.samplePopulationChanged;
    if (!finalBindingsAgree) issues.push("tailReview: final top-level state must equal the last fixed-point iteration");
    if ((tail.disagreements ?? []).length || finalEntry.task4Disagreements.length) issues.push("tailReview: fixed-point Task 4 disagreements must be empty");
  }
}

function reviewFingerprint(item, protectedRow = false, includeFutureOwners = false) {
  const fingerprintOwnerClasses = OWNER_CLASSES.has(item?.class) || (includeFutureOwners && BLIND_FUTURE_CLASSES.has(item?.class));
  const resolution = item?.resolution;
  const resolutionOwners = [
    ...(resolution?.disposition === "delete" ? [] : resolution?.replacementIds ?? []),
    ...(resolution?.subgroups ?? []).flatMap((subgroup) => subgroup?.disposition === "delete" ? [] : subgroup?.replacementIds ?? []),
  ];
  const ownerEvidence = fingerprintOwnerClasses
    ? [...new Set(Array.isArray(item?.ownerEvidence) ? item.ownerEvidence : resolutionOwners)].sort(compareText)
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

function parseEvidenceJson({ bytes, label, issues }) {
  const text = evidenceText(bytes);
  if (text === undefined) {
    issues.push(`independentReview: ${label} evidence file is missing`);
    return undefined;
  }
  try {
    return JSON.parse(text);
  } catch {
    issues.push(`independentReview: ${label} is malformed JSON`);
    return undefined;
  }
}

function validateEvidencePath({ value, hash, label, issues, scope = "independentReview" }) {
  if (!confinedEvidencePath(value)) {
    issues.push(`${scope}: ${label}Path must be confined to ${REVIEW_EVIDENCE_DIRECTORY}`);
    return;
  }
  if (!/^[0-9a-f]{64}$/.test(hash ?? "") || !value.includes(hash)) issues.push(`${scope}: ${label}Path must contain its SHA-256`);
}

function validateCommittedReviewEvidence({ artifact, baseLedger, resolvedOwners, requiredBlindIdsNumeric, evidenceBytes, issues }) {
  if (!(evidenceBytes instanceof Map)) return;
  const independent = artifact.independentReview;
  const reviewer = independent?.reviewer;
  const decisionById = new Map((artifact.decisions ?? []).map((decision) => [decision.id, decision]));
  const baseRowById = new Map((baseLedger.rows ?? []).map((row) => [row.id, row]));
  const expectedPopulationDigest = sha256Text(canonicalJson(requiredBlindIdsNumeric));
  const packetTopKeys = ["normativeSourceCatalog", "orderedClassCatalog", "packetSchemaVersion", "requiredPopulationDigest", "reviewerRunId", "rows", "shardCount", "shardIndex"];
  const packetRowKeys = ["assertionOrdinals", "candidateOwners", "declaration", "id", "invariant", "sourceHash"];
  const candidateKeys = new Set(["capability", "closedRowIds", "id", "mechanism", "normativeSource", "resolvedByBaseClosedRow", "status"]);
  const outputTopKeys = ["packetSha256", "results", "reviewerRunId", "stopConditions"];

  const validatePair = ({ label, packetPath, packetSha256, outputPath, outputSha256, expectedIndex, expectedRowIds }) => {
    validateEvidencePath({ value: packetPath, hash: packetSha256, label: `${label} packet`, issues });
    validateEvidencePath({ value: outputPath, hash: outputSha256, label: `${label} output`, issues });
    const packetBytes = evidenceBytes.get(packetPath);
    const outputBytes = evidenceBytes.get(outputPath);
    if (evidenceText(packetBytes) !== undefined && sha256Evidence(packetBytes) !== packetSha256) issues.push(`independentReview: ${label} packet byte hash mismatch`);
    if (evidenceText(outputBytes) !== undefined && sha256Evidence(outputBytes) !== outputSha256) issues.push(`independentReview: ${label} output byte hash mismatch`);
    const packet = parseEvidenceJson({ bytes: packetBytes, label: `${label} packet`, issues });
    const output = parseEvidenceJson({ bytes: outputBytes, label: `${label} output`, issues });
    if (!packet || !output) return { packet, output, rows: [], results: [] };

    if (canonicalJson(Object.keys(packet).sort(compareText)) !== canonicalJson(packetTopKeys)) issues.push(`independentReview: ${label} packet top-level schema mismatch`);
    if (packet.packetSchemaVersion !== REVIEW_PACKET_SCHEMA_VERSION) issues.push(`independentReview: ${label} packetSchemaVersion must be ${REVIEW_PACKET_SCHEMA_VERSION}`);
    if (packet.reviewerRunId !== reviewer?.reviewerRunId) issues.push(`independentReview: ${label} packet reviewerRunId mismatch`);
    if (packet.shardIndex !== expectedIndex) issues.push(`independentReview: ${label} packet shardIndex mismatch`);
    if (packet.shardCount !== independent.shards.length) issues.push(`independentReview: ${label} packet shardCount mismatch`);
    if (packet.requiredPopulationDigest !== expectedPopulationDigest) issues.push(`independentReview: ${label} packet population digest mismatch`);
    if (canonicalJson(packet.orderedClassCatalog) !== canonicalJson(REASON_CLASS_CATALOG)) issues.push(`independentReview: ${label} packet class catalog mismatch`);
    if (canonicalJson(packet.normativeSourceCatalog) !== canonicalJson(NORMATIVE_SOURCE_CATALOG)) issues.push(`independentReview: ${label} packet normative catalog mismatch`);
    const rows = Array.isArray(packet.rows) ? packet.rows : [];
    if (!Array.isArray(packet.rows)) issues.push(`independentReview: ${label} packet rows must be an array`);
    if (canonicalJson(rows.map((row) => row?.id)) !== canonicalJson(expectedRowIds)) issues.push(`independentReview: ${label} packet row coverage/order mismatch`);
    if (typeof expectedIndex === "number" && (rows.length < 1 || rows.length > 24)) issues.push(`independentReview: ${label} packet must contain 1..24 rows`);

    const packetRowsById = new Map();
    for (const row of rows) {
      if (canonicalJson(Object.keys(row ?? {}).sort(compareText)) !== canonicalJson(packetRowKeys)) issues.push(`${row?.id}: blind packet row schema mismatch`);
      if (packetRowsById.has(row?.id)) issues.push(`${row?.id}: duplicate blind packet row`);
      packetRowsById.set(row?.id, row);
      const decision = decisionById.get(row?.id);
      const baseRow = baseRowById.get(row?.id);
      if (!decision || !baseRow || row?.sourceHash !== decision.sourceHash || row?.sourceHash !== baseRow.sourceHash) issues.push(`${row?.id}: blind packet sourceHash binding mismatch`);
      if (row?.invariant !== baseRow?.invariant) issues.push(`${row?.id}: blind packet invariant binding mismatch`);
      const expectedOrdinals = Array.from({ length: baseRow?.assertionCount ?? 0 }, (_, index) => index + 1);
      if (canonicalJson(row?.assertionOrdinals) !== canonicalJson(expectedOrdinals)) issues.push(`${row?.id}: blind packet assertion ordinals mismatch`);
      if (typeof row?.declaration !== "string" || !row.declaration.trim()) issues.push(`${row?.id}: blind packet declaration is required`);
      else if (sha256Text(row.declaration.replace(/\r\n?/g, "\n")) !== row.sourceHash) issues.push(`${row?.id}: blind packet declaration hash must equal sourceHash`);
      if (!Array.isArray(row?.candidateOwners)) issues.push(`${row?.id}: blind packet candidateOwners must be an array`);
      const candidateIds = new Set();
      for (const candidate of row?.candidateOwners ?? []) {
        if (!candidate || typeof candidate !== "object" || Object.keys(candidate).some((key) => !candidateKeys.has(key))) issues.push(`${row?.id}: blind packet candidate owner schema mismatch`);
        if (typeof candidate?.id !== "string" || !candidate.id.trim() || candidateIds.has(candidate.id)) issues.push(`${row?.id}: blind packet candidate owner IDs must be unique non-empty strings`);
        candidateIds.add(candidate?.id);
        if (candidate?.closedRowIds !== undefined && (!Array.isArray(candidate.closedRowIds)
          || !candidate.closedRowIds.length
          || new Set(candidate.closedRowIds).size !== candidate.closedRowIds.length
          || candidate.closedRowIds.some((id) => typeof id !== "string" || !baseRowById.has(id) || decisionById.has(id)))) {
          issues.push(`${row?.id}: blind packet candidate closedRowIds must name unique base-closed rows`);
        }
        if (candidate?.normativeSource !== undefined && !CRITICALITY_SOURCES.has(candidate.normativeSource)) issues.push(`${row?.id}: blind packet candidate owner has an invented citation`);
        const resolved = candidate?.resolvedByBaseClosedRow === true || candidate?.status === "resolved" || candidate?.status === undefined;
        if (resolved && !resolvedOwners.has(candidate?.id)) issues.push(`${row?.id}: blind packet candidate owner falsely claims base-closed resolution`);
        if (candidate?.id?.startsWith("test:playwright:") && (APPROVED_PLAYWRIGHT_OWNERS.get(row.id) !== candidate.id || candidate.normativeSource !== APPROVED_PLAYWRIGHT_CRITICALITY)) issues.push(`${row?.id}: blind packet Playwright candidate is not allowlisted`);
      }
    }

    if (canonicalJson(Object.keys(output).sort(compareText)) !== canonicalJson(outputTopKeys)) issues.push(`independentReview: ${label} output top-level schema mismatch`);
    if (output.reviewerRunId !== reviewer?.reviewerRunId) issues.push(`independentReview: ${label} output reviewerRunId mismatch`);
    if (output.packetSha256 !== packetSha256) issues.push(`independentReview: ${label} output packet binding mismatch`);
    if (!Array.isArray(output.stopConditions) || output.stopConditions.length) issues.push(`independentReview: ${label} output stopConditions must be empty`);
    const results = Array.isArray(output.results) ? output.results : [];
    if (!Array.isArray(output.results) || results.length !== rows.length) issues.push(`independentReview: ${label} output result count mismatch`);
    for (let index = 0; index < rows.length; index += 1) {
      const row = rows[index];
      const result = results[index] ?? {};
      if (Object.keys(result).some((key) => !BLIND_RESULT_KEYS.has(key))) issues.push(`${result?.id}: blind output result schema mismatch`);
      if (result.id !== row.id || result.sourceHash !== row.sourceHash) issues.push(`independentReview: ${label} output result order/source binding mismatch`);
      if (!CLASS_DISPOSITIONS.has(result.class)) issues.push(`${result?.id}: blind output class is unsupported`);
      if (typeof result.reason !== "string" || !result.reason.trim()) issues.push(`${result?.id}: blind output reason is required`);
      if (result.criticalityRef !== undefined && !CRITICALITY_SOURCES.has(result.criticalityRef)) issues.push(`${result?.id}: blind output has an invented citation`);
      const candidates = new Map((row.candidateOwners ?? []).map((candidate) => [candidate.id, candidate]));
      const selected = Array.isArray(result.ownerEvidence) ? result.ownerEvidence : [];
      if (result.ownerEvidence !== undefined && (!selected.length || selected.some((owner) => typeof owner !== "string" || !owner.trim()))) issues.push(`${result?.id}: blind output ownerEvidence is invalid`);
      if (selected.some((owner) => !candidates.has(owner))) issues.push(`${result?.id}: blind output selected an invented candidate owner`);
      if (BLIND_OWNER_CLASSES.has(result.class) && (!selected.length || selected.some((owner) => {
        const candidate = candidates.get(owner);
        return !(candidate?.resolvedByBaseClosedRow === true || candidate?.status === "resolved" || candidate?.status === undefined);
      }))) issues.push(`${result?.id}: blind output existing-owner class requires resolved candidate membership`);
      if (BLIND_FUTURE_CLASSES.has(result.class) && (!selected.length || selected.some((owner) => !["future", "unresolved", "approved-future"].includes(candidates.get(owner)?.status)))) issues.push(`${result?.id}: blind output future-owner class requires future candidate membership`);
      if (result.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR" && !CRITICALITY_SOURCES.has(result.criticalityRef)) issues.push(`${result?.id}: blind output B3 requires a valid criticalityRef`);
      const requiredSources = [...new Set(selected.map((owner) => candidates.get(owner)?.normativeSource).filter(Boolean))];
      if (requiredSources.length === 1 && result.criticalityRef !== requiredSources[0]) issues.push(`${result?.id}: blind output owner requires its exact citation`);
      const selectedPlaywright = selected.filter((owner) => owner.startsWith("test:playwright:"));
      if (selectedPlaywright.length && (selectedPlaywright.length !== 1 || selectedPlaywright[0] !== APPROVED_PLAYWRIGHT_OWNERS.get(result.id) || result.class !== "B3_PROTECTED_EXPENSIVE_BEHAVIOR" || result.criticalityRef !== APPROVED_PLAYWRIGHT_CRITICALITY)) issues.push(`${result?.id}: blind output Playwright mapping/class/citation mismatch`);
    }
    return { packet, output, rows, results };
  };

  const shardPairs = independent.shards.map((shard, index) => validatePair({
    label: `shard ${index}`,
    packetPath: shard.packetPath,
    packetSha256: shard.packetSha256,
    outputPath: shard.outputPath,
    outputSha256: shard.outputSha256,
    expectedIndex: index,
    expectedRowIds: shard.rowIds,
  }));
  const mergedRows = shardPairs.flatMap((pair) => pair.rows);
  const mergedResults = shardPairs.flatMap((pair) => pair.results);
  const mergedPair = validatePair({
    label: "merged",
    packetPath: reviewer.packetPath,
    packetSha256: reviewer.packetSha256,
    outputPath: reviewer.outputPath,
    outputSha256: reviewer.outputSha256,
    expectedIndex: "merged",
    expectedRowIds: requiredBlindIdsNumeric,
  });
  if (canonicalJson(mergedPair.rows) !== canonicalJson(mergedRows)) issues.push("independentReview: merged packet does not reconstruct the accepted shard packets");
  if (canonicalJson(mergedPair.results) !== canonicalJson(mergedResults)) issues.push("independentReview: merged output does not reconstruct the accepted shard outputs");
  const reconstructedBlindResults = mergedResults.map((result) => ({ ...result, reviewerRunId: reviewer.reviewerRunId }));
  if (canonicalJson(reconstructedBlindResults) !== canonicalJson(independent.blindResults)) issues.push("independentReview: committed shard outputs do not reconstruct blindResults");
  if (sha256Text(canonicalJson(reconstructedBlindResults)) !== independent.mergedOutputSha256) issues.push("independentReview: committed shard outputs do not match mergedOutputSha256");
}

/**
 * Validate a proposed artifact against its immutable review-base ledger.
 * The base-open set is derived from the review-base tracked paths, never from decisions.
 */
export function validateReview({ artifact, baseLedger, currentLedger, baseTrackedPaths, evidenceBytes } = {}) {
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
  const expectedTailFamilies = deriveTailFamilyManifests({ baseLedger, baseTrackedPaths: tracked });
  if (artifact.scope?.openRows === 436 && canonicalJson(artifact.tailFamilies) !== canonicalJson(expectedTailFamilies)) {
    issues.push("tailFamilies: manifests must equal the frozen first-match partition");
  }

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
    const allowedAuthorRuns = new Set([artifact.independentReview?.authorRunId, artifact.tailReview?.authorRunId].filter(Boolean));
    if (!allowedAuthorRuns.has(decision.authorRunId)) issues.push(`${decision.id}: authorRunId differs from artifact author runs`);
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
  validateAcceptedLoss(artifact, issues);
  validateTailReviewScaffold(artifact, issues);
  validateTailIterationEvidence({ artifact, baseLedger, evidenceBytes, protectedIds, issues });

  const independent = artifact.independentReview;
  issues.push(...validateRuleAdjudications({ independentReview: independent, decisions }));
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
    validateCommittedReviewEvidence({ artifact, baseLedger, resolvedOwners, requiredBlindIdsNumeric, evidenceBytes, issues });
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
    const calibrationClasses = new Set();
    if (artifact.scope?.openRows === 436 && canonicalJson((independent.calibrations ?? []).map((cohort) => ({
      class: cohort?.class,
      adjacentClass: cohort?.adjacentClass,
    }))) !== canonicalJson(APPROVED_CALIBRATION_PAIRS)) {
      issues.push("calibrations: class/adjacentClass pairs must equal the approved ordered registry");
    }
    for (const cohort of Array.isArray(independent.calibrations) ? independent.calibrations : []) {
      if (!Array.isArray(cohort?.rowIds)) {
        issues.push("calibrations: rowIds must be an array");
        continue;
      }
      if (!CLASS_DISPOSITIONS.has(cohort?.class)) issues.push(`calibrations: unsupported class ${cohort?.class}`);
      if (calibrationClasses.has(cohort?.class)) issues.push(`calibrations: duplicate class ${cohort?.class}`);
      calibrationClasses.add(cohort?.class);
      const rowIds = cohort.rowIds;
      for (const id of rowIds) {
        const decision = decisions.find((item) => item.id === id);
        if (decision && decision.class !== cohort.class) issues.push(`calibrations: ${id} is ${decision.class}, not ${cohort.class}`);
      }
      if (!rowIds.length && decisions.some((decision) => decision.class === cohort.class)) {
        issues.push(`calibrations: ${cohort.class} cannot record no_match while classified examples exist`);
      }
      const expectedComparisonValue = rowIds.length ? expectedComparison(rowIds, fingerprintMatches) : "no_match";
      if (!rowIds.length && cohort?.result !== expectedComparisonValue) {
        issues.push("calibrations: empty calibration must record no_match");
        continue;
      }
      if (cohort?.result !== expectedComparisonValue) issues.push(`calibrations: recorded comparison must be ${expectedComparisonValue}`);
    }
    const mandatoryCohorts = Array.isArray(independent.mandatoryCohorts) ? independent.mandatoryCohorts : [];
    if (artifact.scope?.openRows === 436
      && canonicalJson(mandatoryCohorts.map((cohort) => cohort?.name)) !== canonicalJson(APPROVED_MANDATORY_COHORT_NAMES)) {
      issues.push("mandatoryCohorts: names must equal the approved ordered registry");
    }
    if (artifact.scope?.openRows === 436) {
      const exactMemberships = new Map(APPROVED_MANDATORY_COHORT_MEMBERS);
      exactMemberships.set("protected-p0", protectedRows.map((row) => row.id).sort(compareId));
      for (const [name, expectedIds] of exactMemberships) {
        const cohort = mandatoryCohorts.find((item) => item?.name === name);
        if (canonicalJson(cohort?.rowIds) !== canonicalJson(expectedIds)) {
          issues.push(`mandatoryCohorts: ${name} rowIds must equal approved sorted membership`);
        }
      }
    }
    const emergingExpectations = new Map([
      ["emerging-b3", decisions.filter((decision) => decision.class === "B3_PROTECTED_EXPENSIVE_BEHAVIOR").map((decision) => decision.id).sort(compareId)],
      ["emerging-d5", decisions.filter((decision) => decision.class === "D5_ACCEPTED_LOSS").map((decision) => decision.id).sort(compareId)],
      ["emerging-mixed", decisions.filter((decision) => Array.isArray(decision.resolution?.subgroups)).map((decision) => decision.id).sort(compareId)],
    ]);
    for (const [name, expectedIds] of emergingExpectations) {
      const cohorts = mandatoryCohorts.filter((cohort) => cohort?.name === name);
      if (artifact.scope?.openRows === 436 && cohorts.length !== 1) issues.push(`mandatoryCohorts: ${name} must appear exactly once`);
      for (const cohort of cohorts) if (canonicalJson(cohort?.rowIds) !== canonicalJson(expectedIds)) issues.push(`mandatoryCohorts: ${name} must exactly equal the classified ${name === "emerging-b3" ? "B3 rows" : name === "emerging-d5" ? "D5 rows" : "mixed rows"}`);
    }
    for (const cohort of mandatoryCohorts) {
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
    const approvedAdjudication = independent.validIterations === 3
      && canonicalJson(independent.ruleAdjudications) === canonicalJson(APPROVED_RULE_ADJUDICATIONS);
    if (!approvedAdjudication && canonicalJson(disagreementIds) !== canonicalJson(mismatchIds)) {
      issues.push("independentReview: disagreements must equal fingerprint mismatch IDs");
    }
    if (approvedAdjudication && mismatchIds.length) {
      issues.push("independentReview: adjudicated final decisions must match retained blind fingerprints");
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
  const [baseLedger, baseTrackedPaths, currentLedger, evidence] = await Promise.all([
    loadBaseLedger({ repoRoot, commit: artifact.reviewBaseCommit }),
    loadBaseTrackedPaths({ repoRoot, commit: artifact.reviewBaseCommit }),
    readFile(ledgerPath, "utf8").then(JSON.parse),
    loadReviewEvidence({ repoRoot, artifact }),
  ]);
  if (evidence.issues.length) {
    for (const issue of evidence.issues) process.stderr.write(`${issue}\n`);
    process.exitCode = 1;
    return;
  }
  const input = { artifact, baseLedger, currentLedger, baseTrackedPaths, evidenceBytes: evidence.evidenceBytes };
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
