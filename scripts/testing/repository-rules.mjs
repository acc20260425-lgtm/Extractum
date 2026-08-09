import { createHash } from "node:crypto";
import { generateIdentityAuthority } from "../telegram-8b-test-identities.mjs";
import { generateSymbolAuthority } from "../telegram-8b-symbol-map.mjs";
import { generateFeatureBaseline } from "../telegram-grammers-feature-baseline.mjs";

const TELEGRAM_PATH = "src/lib/telegram-contract-paths.ts";
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";
const SOURCE_BROWSER_SHELL_PATH = "src/lib/components/analysis/source-browser-shell.svelte";
const SOURCE_GROUP_SOURCES_PATH = "src/lib/components/analysis/source-group-sources-view.svelte";
const SOURCE_GROUP_ACTIVITY_PATH = "src/lib/components/analysis/source-group-activity-view.svelte";
const SOURCE_BROWSER_SHELL_MODULE = "$lib/components/analysis/source-browser-shell.svelte";
const SOURCE_ACTIVITY_MODULE = "$lib/components/analysis/source-activity-view.svelte";
const SOURCE_GROUP_ACTIVITY_MODULE = "$lib/components/analysis/source-group-activity-view.svelte";
const EMPTY_STATE_MODULE = "$lib/components/ui/EmptyState.svelte";
const DATA_GRID_PATH = "src/lib/components/extractum-ui/DataGrid.svelte";
const TREE_DATA_GRID_PATH = "src/lib/components/extractum-ui/TreeDataGrid.svelte";
const GRID_RUNTIME_PATH = "src/lib/components/extractum-ui/data-grid-date-format.ts";
const LLM_LIB_PATH = "src-tauri/crates/extractum-llm/src/lib.rs";
const LLM_GEMINI_PATH = "src-tauri/crates/extractum-llm/src/gemini.rs";
const LLM_OPENAI_COMPAT_PATH = "src-tauri/crates/extractum-llm/src/openai_compat.rs";
const LLM_TYPES_PATH = "src-tauri/crates/extractum-llm/src/types.rs";
const LLM_PROVIDER_PATH = "src-tauri/crates/extractum-llm/src/provider.rs";
const LLM_RUNNER_PATH = "src-tauri/crates/extractum-llm/src/runner.rs";
const LLM_SCHEDULER_PATH = "src-tauri/crates/extractum-llm/src/scheduler.rs";
const LLM_STREAMING_PATH = "src-tauri/crates/extractum-llm/src/streaming.rs";
const LLM_BUILD_PATH = "src-tauri/crates/extractum-llm/build.rs";
const LLM_SRC_PREFIX = "src-tauri/crates/extractum-llm/src/";
const EXPECTED_LLM_RUST_FILES = new Set([
  LLM_GEMINI_PATH,
  LLM_LIB_PATH,
  LLM_OPENAI_COMPAT_PATH,
  LLM_PROVIDER_PATH,
  LLM_RUNNER_PATH,
  LLM_SCHEDULER_PATH,
  LLM_STREAMING_PATH,
  LLM_TYPES_PATH,
]);
const APPROVED_SVAR_GRID_PATHS = new Set([DATA_GRID_PATH, TREE_DATA_GRID_PATH, GRID_RUNTIME_PATH]);
const EXPECTED_LLM_ROOT = `
mod gemini;
mod openai_compat;
mod provider;
mod runner;
mod scheduler;
mod streaming;
mod types;
pub use provider::{list_provider_models, normalize_base_url, resolve_model_input_token_limit, resolve_model_output_token_limit, ProviderKind};
pub use runner::{resolve_effective_model, run_llm_collect_with_profile, run_llm_stream_with_profile, validate_request};
pub use scheduler::{llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key, LlmRequestControl, LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmRequestSnapshot, LlmRequestSnapshotState, LlmSchedulerState};
pub use types::{LlmChatRequest, LlmCompletion, LlmMessage, LlmProviderAccess, LlmProviderModel, LlmUsage, ResolvedLlmProfile};
`;
const EXPECTED_LLM_CREDENTIAL_TYPES = new Map([
  ["LlmProviderAccess", {
    fields: ["provider", "api_key", "base_url"],
    methods: new Map([["new", "pub"], ["provider", "pub(super)"], ["api_key", "pub(super)"], ["base_url", "pub(super)"]]),
  }],
  ["ResolvedLlmProfile", {
    fields: ["profile_id", "default_model", "provider_access"],
    methods: new Map([["new", "pub"], ["profile_id", "pub"], ["provider", "pub"], ["default_model", "pub"], ["base_url", "pub"], ["provider_access", "pub(super)"]]),
  }],
]);
const EXPECTED_LLM_PUBLIC_METHODS = new Map([
  ["ProviderKind", { path: LLM_PROVIDER_PATH, methods: ["as_str", "parse"] }],
  ["LlmRequestControl", { path: LLM_SCHEDULER_PATH, methods: ["run_cancellable"] }],
  ["LlmSchedulerState", {
    path: LLM_SCHEDULER_PATH,
    methods: ["active_owner_run_ids", "cancel_request", "cancel_run_requests", "new", "request_snapshots", "run_request"],
  }],
]);
const ALLOWED_LLM_MODULE_ATTRIBUTES = new Map([
  [LLM_LIB_PATH, new Set()],
  [LLM_GEMINI_PATH, new Set([
    "#[derive(Serialize)]",
    "#[derive(Serialize,Deserialize,Debug,PartialEq,Eq)]",
    "#[derive(Deserialize,Debug)]",
    "#[derive(Clone,Deserialize,Debug)]",
    "#[derive(Deserialize)]",
    "#[serde(rename_all=\"camelCase\")]",
  ])],
  [LLM_OPENAI_COMPAT_PATH, new Set([
    "#[derive(Clone)]",
    "#[derive(Serialize)]",
    "#[derive(Deserialize,Debug)]",
    "#[derive(Clone,Deserialize,Debug)]",
    "#[derive(Deserialize)]",
  ])],
  [LLM_TYPES_PATH, new Set([
    "#[derive(Clone)]",
    "#[derive(Clone,Serialize,Deserialize,Debug,PartialEq,Eq)]",
  ])],
  [LLM_PROVIDER_PATH, new Set([
    "#[derive(Clone,Copy,Serialize,Deserialize,PartialEq,Eq,Debug)]",
    "#[serde(rename_all=\"snake_case\")]",
  ])],
  [LLM_RUNNER_PATH, new Set()],
  [LLM_SCHEDULER_PATH, new Set([
    "#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]",
    "#[derive(Clone,Debug,PartialEq,Eq,Hash)]",
    "#[derive(Clone,Debug)]",
    "#[derive(Clone,Debug,PartialEq,Eq,Serialize)]",
    "#[derive(Clone)]",
    "#[derive(Debug,Clone,PartialEq,Eq)]",
    "#[derive(Clone,Copy,Debug,PartialEq,Eq)]",
    "#[derive(Default)]",
    "#[serde(rename_all=\"snake_case\")]",
  ])],
  [LLM_STREAMING_PATH, new Set()],
]);
const ALLOWED_LLM_MACROS = new Map([
  [LLM_LIB_PATH, new Set()],
  [LLM_GEMINI_PATH, new Set(["format", "matches", "vec"])],
  [LLM_OPENAI_COMPAT_PATH, new Set(["format", "matches"])],
  [LLM_TYPES_PATH, new Set(["todo"])],
  [LLM_PROVIDER_PATH, new Set(["format", "matches", "todo"])],
  [LLM_RUNNER_PATH, new Set(["format", "todo"])],
  [LLM_SCHEDULER_PATH, new Set(["format", "todo", "tokio::select"])],
  [LLM_STREAMING_PATH, new Set()],
]);
const ALLOWED_LLM_TYPE_ALIASES = new Map([
  [LLM_SCHEDULER_PATH, new Set(["typeQueueCallback=Arc<dynFn(usize)+Send+Sync>;"])],
]);
const ALLOWED_LLM_RENAMED_IMPORTS = new Map([
  [LLM_GEMINI_PATH, new Set(["usereqwest::ClientasHttpClient;"])],
  [LLM_OPENAI_COMPAT_PATH, new Set(["usereqwest::ClientasHttpClient;"])],
]);
const CANONICAL_LEAF_PATHS = [
  ["SourceGroupSourcesView", SOURCE_GROUP_SOURCES_PATH, "$lib/components/analysis/source-group-sources-view.svelte"],
  ["SnapshotGroupSourcesView", "src/lib/components/analysis/snapshot-group-sources-view.svelte", "$lib/components/analysis/snapshot-group-sources-view.svelte"],
  ["SnapshotItemsView", "src/lib/components/analysis/snapshot-items-view.svelte", "$lib/components/analysis/snapshot-items-view.svelte"],
  ["RunSnapshotMetadataView", "src/lib/components/analysis/run-snapshot-metadata-view.svelte", "$lib/components/analysis/run-snapshot-metadata-view.svelte"],
];
const HIGHLIGHT_STYLE_TARGETS = [
  ["src/lib/components/analysis/telegram-timeline-reader.svelte", "li", "primary", true],
  ["src/lib/components/analysis/youtube-transcript-reader.svelte", "li", "primary", true],
  ["src/lib/components/analysis/snapshot-items-view.svelte", "article", "accent", true],
  ["src/lib/components/analysis/snapshot-group-sources-view.svelte", "li", "accent", true],
  ["src/lib/components/analysis/universal-items-view.svelte", "article", "accent", false],
  ["src/lib/components/analysis/youtube-comments-view.svelte", "article", "accent", false],
];
const PHASE_8A_PLAN_PATH = "docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md";
const PHASE_8B_PLAN_PATH = "docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md";
const SYMBOL_MAP_PATH = "src/lib/telegram-8b-symbol-map.json";
const TEST_IDENTITIES_PATH = "src/lib/telegram-8b-test-identities.json";
const STAGING_SHA_PATH = "src/lib/telegram-8b-staging-sha256.json";
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";
const PHASE_8_ROADMAP_PATH = "docs/superpowers/specs/2026-07-17-crate-roadmap.md";
const PHASE_8_DESIGN_PATH = "docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md";
const FROZEN_STAGING_SHA256 = "12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9";
const EXPECTED_PRODUCER_DEPENDENCIES = [
  ["base64", null, [], true, null, null],
  ["chacha20poly1305", null, ["std"], true, null, null],
  ["extractum-core", null, [], true, null, null],
  ["grammers-client", null, [], false, null, null],
  ["grammers-mtsender", null, [], true, null, null],
  ["grammers-session", null, ["serde"], false, null, null],
  ["grammers-tl-types", null, ["deserializable-functions"], true, null, null],
  ["rand_core", null, ["getrandom"], true, null, null],
  ["secrecy", null, [], true, null, null],
  ["serde", null, ["derive"], true, null, null],
  ["serde_json", null, [], true, null, null],
  ["tokio", null, ["rt", "sync", "time"], true, null, null],
  ["tokio", "dev", ["macros", "test-util"], true, null, null],
];
const EXPECTED_APP_TELEGRAM_DEPENDENCIES = [
  ["extractum-telegram", null, [], true, null, null],
  ["extractum-telegram", "dev", ["app-test-support"], true, null, null],
];

const TRANSITIONAL_SOURCE_COMPONENTS = [
  ["RunCompanionTabs", "$lib/components/analysis/run-companion-tabs.svelte"],
  ["SourceContextPanel", "$lib/components/analysis/source-context-panel.svelte"],
  ["SourceGroupReader", "$lib/components/analysis/source-group-reader.svelte"],
  ["TelegramTimelineReader", "$lib/components/analysis/telegram-timeline-reader.svelte"],
  ["YoutubePlaylistDetail", "$lib/components/analysis/youtube-playlist-detail.svelte"],
  ["YoutubePlaylistReader", "$lib/components/analysis/youtube-playlist-reader.svelte"],
  ["YoutubeSourceDetail", "$lib/components/analysis/youtube-source-detail.svelte"],
  ["YoutubeTranscriptReader", "$lib/components/analysis/youtube-transcript-reader.svelte"],
];

function identifier(expression, name) {
  return expression?.kind === "identifier" && expression.name === name;
}

function stringLiteral(expression, value) {
  return expression?.kind === "string" && expression.value === value;
}

function evaluateAnalysisSourceReaderSurfaceComposition(index) {
  const facts = index.getSvelte(ANALYSIS_SURFACE_PATH);
  const violations = [];

  if (!hasComponentFromModule(facts, SOURCE_BROWSER_SHELL_MODULE)) {
    violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell must own source browsing`);
  }
  for (const [name, moduleSource] of TRANSITIONAL_SOURCE_COMPONENTS) {
    if (hasComponentFromModule(facts, moduleSource)) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: transitional ${name} composition is forbidden`);
    }
  }
  return violations;
}

function evaluateAnalysisSourceBrowserExplicitSubjectContract(index) {
  const facts = index.getSvelte(ANALYSIS_SURFACE_PATH);
  const shells = componentsFromModule(facts, SOURCE_BROWSER_SHELL_MODULE);
  const violations = [];
  if (!shells.length) {
    return [`${ANALYSIS_SURFACE_PATH}: missing SourceBrowserShell composition`];
  }
  for (const [position, shell] of shells.entries()) {
    if (!shell.attributes.includes("subject")) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell ${position + 1} must receive subject`);
    }
    if (shell.attributes.includes("source")) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell ${position + 1} must not receive legacy source`);
    }
  }
  return violations;
}

function selectorHas(selector, type, name, value) {
  return selector.some((part) => part.type === type
    && part.name === name
    && (value === undefined || part.value === value));
}

function evaluateAnalysisEvidenceHighlightTokenStyling(index) {
  const violations = [];
  for (const [path, tag, token, paired] of HIGHLIGHT_STYLE_TARGETS) {
    const facts = index.getSvelte(path);
    const matchingRule = facts.styleRules.find((rule) => {
      const evidenceSelector = rule.selectors.some((selector) =>
        selectorHas(selector, "tag", tag)
        && selectorHas(selector, "attribute", "data-evidence-highlighted", "true"));
      const selectedSelector = rule.selectors.some((selector) =>
        selectorHas(selector, "tag", tag) && selectorHas(selector, "class", "selected"));
      const semanticToken = rule.declarations.some(({ value }) => value.includes(`var(--${token})`));
      return evidenceSelector && (!paired || selectedSelector) && semanticToken;
    });
    if (!matchingRule) {
      violations.push(`${path}: evidence highlight must use the semantic --${token} selection token${paired ? " beside selected styling" : ""}`);
    }
  }
  return violations;
}

function routeApiImport(source) {
  return source === "$lib/api"
    || source.startsWith("$lib/api/")
    || source === "@tauri-apps/api"
    || source.startsWith("@tauri-apps/api/");
}

function importedDefaultSource(facts, localName) {
  const matches = facts.imports.filter(({ bindings }) => bindings.some((binding) =>
    binding.imported === "default" && binding.local === localName && !binding.typeOnly));
  return matches.length === 1 ? matches[0].source : undefined;
}

function componentsFromModule(facts, moduleSource) {
  return facts.components.filter((component) => importedDefaultSource(facts, component.name) === moduleSource);
}

function hasComponentFromModule(facts, moduleSource) {
  return componentsFromModule(facts, moduleSource).length > 0;
}

function matchingBrace(source, open) {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}" && --depth === 0) return index;
  }
  throw new Error("unterminated Rust block");
}

const RUST_IDENTIFIER_SOURCE = String.raw`(?:r#)?(?:_|\p{ID_Start})(?:_|\p{ID_Continue})*`;
const RUST_IDENTIFIER_AT = new RegExp(`^${RUST_IDENTIFIER_SOURCE}`, "u");
const RUST_IDENTIFIER_GLOBAL = new RegExp(RUST_IDENTIFIER_SOURCE, "gu");
const RUST_IDENTIFIER_CONTINUE = /(?:_|\p{ID_Continue})/u;

function rustIdentifierAt(source, start) {
  const match = source.slice(start).match(RUST_IDENTIFIER_AT);
  if (!match || match[0] === "_" || match[0] === "r#_") return null;
  const semantic = match[0].startsWith("r#") ? match[0].slice(2) : match[0];
  return {
    end: start + match[0].length,
    raw: match[0],
    value: semantic.normalize("NFC"),
  };
}

function rustIdentifierValues(source) {
  return [...source.matchAll(RUST_IDENTIFIER_GLOBAL)]
    .filter((match) => match[0] !== "_" && match[0] !== "r#_")
    .map((match) => (match[0].startsWith("r#") ? match[0].slice(2) : match[0]).normalize("NFC"));
}

function sanitizeRust(source) {
  const output = [...source];
  const blank = (start, end) => {
    for (let index = start; index < end; index += 1) if (output[index] !== "\n" && output[index] !== "\r") output[index] = " ";
  };
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const stop = end < 0 ? source.length : end;
      blank(index, stop);
      index = stop;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith("/*", cursor)) { depth += 1; cursor += 2; }
        else if (source.startsWith("*/", cursor)) { depth -= 1; cursor += 2; }
        else cursor += 1;
      }
      if (depth !== 0) throw new Error("unterminated Rust block comment");
      blank(index, cursor);
      index = cursor;
      continue;
    }
    const raw = source.slice(index).match(/^(?:b|c)?r(#{0,})"/);
    if (raw) {
      const terminator = `"${raw[1]}`;
      const end = source.indexOf(terminator, index + raw[0].length);
      if (end < 0) throw new Error("unterminated Rust raw string");
      const stop = end + terminator.length;
      blank(index, stop);
      index = stop;
      continue;
    }
    const character = source.slice(index).match(/^(?:b)?'(?:\\(?:.|u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2})|[^'\\\r\n])'/);
    if (character) {
      blank(index, index + character[0].length);
      index += character[0].length;
      continue;
    }
    const quoteStart = source[index] === '"'
      ? index
      : source.startsWith('b"', index) || source.startsWith('c"', index) ? index + 1 : -1;
    if (quoteStart >= 0) {
      let cursor = quoteStart + 1;
      while (cursor < source.length) {
        if (source[cursor] === "\\") cursor += 2;
        else if (source[cursor] === '"') { cursor += 1; break; }
        else cursor += 1;
      }
      if (cursor > source.length || source[cursor - 1] !== '"') throw new Error("unterminated Rust string");
      blank(index, cursor);
      index = cursor;
      continue;
    }
    index += 1;
  }
  return output.join("");
}

function withoutCfgTestModules(source) {
  let output = source;
  const pattern = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{/;
  while (true) {
    const structural = sanitizeRust(output);
    const match = structural.match(pattern);
    if (!match || match.index === undefined) break;
    const start = match.index;
    const open = structural.indexOf("{", start + match[0].lastIndexOf("{"));
    output = `${output.slice(0, start)}${output.slice(matchingBrace(structural, open) + 1)}`;
  }
  return output;
}

function compactRust(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/.*$/gm, "")
    .replace(/\s+/g, "")
    .replace(/,}/g, "}");
}

function namedRustBlock(source, prefix) {
  const structural = sanitizeRust(source);
  const start = structural.indexOf(prefix);
  if (start < 0) throw new Error(`missing Rust declaration: ${prefix}`);
  const open = structural.indexOf("{", start + prefix.length);
  if (open < 0) throw new Error(`missing Rust body: ${prefix}`);
  return source.slice(open + 1, matchingBrace(structural, open));
}

function namedRustImplBlocks(source, name) {
  const structural = sanitizeRust(source);
  const blocks = [];
  for (let index = 0; index < structural.length; index += 1) {
    if (!structural.startsWith("impl", index)
      || RUST_IDENTIFIER_CONTINUE.test(structural[index - 1] ?? "")
      || RUST_IDENTIFIER_CONTINUE.test(structural[index + 4] ?? "")
      || structural.slice(Math.max(0, index - 2), index) === "r#") continue;

    let angle = 0;
    let parentheses = 0;
    let brackets = 0;
    let open = -1;
    for (let cursor = index + 4; cursor < structural.length; cursor += 1) {
      const character = structural[cursor];
      if (character === "<") angle += 1;
      else if (character === ">" && angle > 0) angle -= 1;
      else if (character === "(") parentheses += 1;
      else if (character === ")" && parentheses > 0) parentheses -= 1;
      else if (character === "[") brackets += 1;
      else if (character === "]" && brackets > 0) brackets -= 1;
      else if (character === "{" && angle === 0 && parentheses === 0 && brackets === 0) { open = cursor; break; }
      else if (character === ";" && angle === 0 && parentheses === 0 && brackets === 0) break;
    }
    if (open < 0) continue;
    const header = structural.slice(index + 4, open);
    let target = header.trimStart();
    if (target.startsWith("<")) {
      let genericDepth = 0;
      let genericEnd = -1;
      for (let cursor = 0; cursor < target.length; cursor += 1) {
        if (target[cursor] === "<") genericDepth += 1;
        else if (target[cursor] === ">" && --genericDepth === 0) { genericEnd = cursor + 1; break; }
      }
      if (genericEnd < 0) throw new Error("unterminated Rust impl generic clause");
      target = target.slice(genericEnd).trimStart();
    }
    target = target.split(/\bwhere\b/, 1)[0];
    if (/\bfor\b/.test(target)) continue;
    let previous;
    do {
      previous = target;
      target = target.replace(/<[^<>]*>/g, " ");
    } while (target !== previous);
    const identifiers = rustIdentifierValues(target);
    const close = matchingBrace(structural, open);
    if (identifiers.at(-1) === name) blocks.push(source.slice(open + 1, close));
  }
  return blocks;
}

function rustStructFields(body) {
  const fields = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index <= body.length; index += 1) {
    const character = body[index];
    if (["(", "[", "{"].includes(character)) depth += 1;
    if ([")", "]", "}"].includes(character)) depth -= 1;
    if ((character === "," || index === body.length) && depth === 0) {
      const declaration = body.slice(start, index).trim();
      start = index + 1;
      if (!declaration) continue;
      const match = declaration.match(new RegExp(`^(pub(?:\\([^)]*\\))?\\s+)?(${RUST_IDENTIFIER_SOURCE})\\s*:`, "u"));
      if (!match) throw new Error(`unrecognized Rust field: ${declaration}`);
      fields.push({ name: (match[2].startsWith("r#") ? match[2].slice(2) : match[2]).normalize("NFC"), visibility: match[1]?.trim() ?? "private" });
    }
  }
  return fields;
}

function rustImplMethods(body) {
  const methods = [];
  const structural = sanitizeRust(body);
  const skipWhitespace = (start) => {
    let cursor = start;
    while (/\s/.test(structural[cursor] ?? "")) cursor += 1;
    return cursor;
  };
  const methodAt = (start) => {
    let cursor = skipWhitespace(start);
    let token = rustIdentifierAt(structural, cursor);
    let visibility = "private";
    if (token?.raw === "pub") {
      visibility = "pub";
      cursor = skipWhitespace(token.end);
      if (structural[cursor] === "(") {
        const close = structural.indexOf(")", cursor + 1);
        if (close < 0) throw new Error("unterminated Rust method visibility");
        visibility = structural.slice(token.end - 3, close + 1).replace(/\s+/g, "");
        cursor = skipWhitespace(close + 1);
      }
      token = rustIdentifierAt(structural, cursor);
    }
    while (["const", "async", "unsafe", "extern"].includes(token?.raw)) {
      cursor = skipWhitespace(token.end);
      token = rustIdentifierAt(structural, cursor);
    }
    if (token?.raw !== "fn") return null;
    cursor = skipWhitespace(token.end);
    const name = rustIdentifierAt(structural, cursor);
    return name ? { name: name.value, visibility, end: name.end } : null;
  };
  let depth = 0;
  for (let index = 0; index < structural.length; index += 1) {
    if (structural[index] === "{") depth += 1;
    else if (structural[index] === "}") depth -= 1;
    if (depth !== 0) continue;
    const method = methodAt(index);
    if (!method) continue;
    methods.push({ name: method.name, visibility: method.visibility });
    index = method.end - 1;
  }
  return methods;
}

function rustAssociatedItemHazards(body) {
  const hazards = [];
  const structural = sanitizeRust(body);
  const skipWhitespace = (start) => {
    let cursor = start;
    while (/\s/.test(structural[cursor] ?? "")) cursor += 1;
    return cursor;
  };
  const classifyItem = (start) => {
    let cursor = skipWhitespace(start);
    let token = rustIdentifierAt(structural, cursor);
    if (!token) return `unrecognized token ${structural[cursor]}`;
    if (token.raw === "pub") {
      cursor = skipWhitespace(token.end);
      if (structural[cursor] === "(") {
        const close = structural.indexOf(")", cursor + 1);
        if (close < 0) return "unterminated visibility";
        cursor = skipWhitespace(close + 1);
      }
      token = rustIdentifierAt(structural, cursor);
    }
    const first = token?.raw;
    while (["const", "async", "unsafe", "extern"].includes(token?.raw)) {
      cursor = skipWhitespace(token.end);
      token = rustIdentifierAt(structural, cursor);
    }
    if (token?.raw === "fn") return null;
    if (token && structural[skipWhitespace(token.end)] === "!") return `item macro ${token.value}!`;
    if (["const", "type", "static"].includes(first)) return `associated ${first}`;
    return `unrecognized associated item ${first ?? "token"}`;
  };
  let depth = 0;
  let itemStart = true;
  for (let index = 0; index < structural.length; index += 1) {
    if (structural[index] === "{") { depth += 1; continue; }
    if (structural[index] === "}") {
      depth -= 1;
      if (depth === 0) itemStart = true;
      continue;
    }
    if (depth !== 0) continue;
    if (structural[index] === ";") {
      itemStart = true;
      continue;
    }
    if (!itemStart || /\s/.test(structural[index])) continue;
    if (structural[index] === "#") hazards.push("attribute");
    else {
      const hazard = classifyItem(index);
      if (hazard) hazards.push(hazard);
    }
    itemStart = false;
  }
  return hazards;
}

function rustInherentMethods(source, name) {
  return namedRustImplBlocks(source, name).flatMap(rustImplMethods);
}

function rustInherentHazards(source, name) {
  return namedRustImplBlocks(source, name).flatMap(rustAssociatedItemHazards);
}

function rustModuleItemHazards(source, allowedAttributes) {
  const hazards = [];
  const structural = sanitizeRust(source);
  const skipWhitespace = (start) => {
    let cursor = start;
    while (/\s/.test(structural[cursor] ?? "")) cursor += 1;
    return cursor;
  };
  const attributeEnd = (start) => {
    const open = structural.indexOf("[", start + 1);
    if (open < 0 || structural.slice(start, open).replace(/\s/g, "") !== "#") return -1;
    let depth = 0;
    for (let cursor = open; cursor < structural.length; cursor += 1) {
      if (structural[cursor] === "[") depth += 1;
      else if (structural[cursor] === "]" && --depth === 0) return cursor;
    }
    return -1;
  };
  const itemMacroAt = (start) => {
    let cursor = skipWhitespace(start);
    let token = rustIdentifierAt(structural, cursor);
    if (!token) return null;
    const parts = [token.value];
    cursor = skipWhitespace(token.end);
    while (structural.startsWith("::", cursor)) {
      cursor = skipWhitespace(cursor + 2);
      token = rustIdentifierAt(structural, cursor);
      if (!token) return null;
      parts.push(token.value);
      cursor = skipWhitespace(token.end);
    }
    return structural[cursor] === "!" ? `${parts.join("::")}!` : null;
  };
  let depth = 0;
  let itemStart = true;
  for (let index = 0; index < structural.length; index += 1) {
    if (structural[index] === "{") { depth += 1; continue; }
    if (structural[index] === "}") {
      depth -= 1;
      if (depth === 0) itemStart = true;
      continue;
    }
    if (depth !== 0) continue;
    if (structural[index] === ";") { itemStart = true; continue; }
    if (!itemStart || /\s/.test(structural[index])) continue;
    if (structural[index] === "#") {
      const end = attributeEnd(index);
      if (end < 0) {
        hazards.push("malformed or inner module attribute");
        itemStart = false;
      } else {
        const attribute = source.slice(index, end + 1).replace(/\s+/g, "");
        if (!allowedAttributes.has(attribute)) hazards.push(`unknown module item attribute ${attribute}`);
        index = end;
      }
      continue;
    }
    const itemMacro = itemMacroAt(index);
    if (itemMacro) hazards.push(`module item macro ${itemMacro}`);
    itemStart = false;
  }
  return hazards;
}

function rustMacroHazards(source, allowedMacros) {
  const hazards = [];
  const structural = sanitizeRust(source);
  const nonMacroBangPrefixes = new Set(["if", "while", "return", "let"]);
  const skipWhitespace = (start) => {
    let cursor = start;
    while (/\s/.test(structural[cursor] ?? "")) cursor += 1;
    return cursor;
  };
  for (let index = 0; index < structural.length; index += 1) {
    if (RUST_IDENTIFIER_CONTINUE.test(structural[index - 1] ?? "")) continue;
    let token = rustIdentifierAt(structural, index);
    if (!token) continue;
    const parts = [token.value];
    let cursor = skipWhitespace(token.end);
    while (structural.startsWith("::", cursor)) {
      cursor = skipWhitespace(cursor + 2);
      token = rustIdentifierAt(structural, cursor);
      if (!token) break;
      parts.push(token.value);
      cursor = skipWhitespace(token.end);
    }
    if (structural[cursor] !== "!") continue;
    const name = parts.join("::");
    if (parts.length === 1 && nonMacroBangPrefixes.has(name)) continue;
    if (!allowedMacros.has(name)) hazards.push(`${name}!`);
    index = cursor;
  }
  return hazards;
}

function rustTypeItems(source, path) {
  const aliases = [];
  const hazards = [];
  const structural = sanitizeRust(source);
  const skipWhitespace = (start) => {
    let cursor = start;
    while (/\s/.test(structural[cursor] ?? "")) cursor += 1;
    return cursor;
  };
  const pattern = /type/g;
  let match;
  while ((match = pattern.exec(structural)) !== null) {
    const start = match.index;
    if (structural.slice(Math.max(0, start - 2), start) === "r#"
      || RUST_IDENTIFIER_CONTINUE.test(structural[start - 1] ?? "")
      || RUST_IDENTIFIER_CONTINUE.test(structural[start + 4] ?? "")) continue;
    let cursor = skipWhitespace(start + 4);
    const nameToken = rustIdentifierAt(structural, cursor);
    if (!nameToken) {
      hazards.push(`${path}: type item has a missing or unparseable name`);
      continue;
    }
    cursor = nameToken.end;
    const stack = [];
    const equals = [];
    let end = -1;
    let malformed = false;
    const pairs = new Map([[")", "("], ["]", "["], ["}", "{"], [">", "<"]]);
    for (; cursor < structural.length; cursor += 1) {
      const character = structural[cursor];
      if (["(", "[", "{", "<"].includes(character)) stack.push(character);
      else if (pairs.has(character)) {
        if (stack.at(-1) === pairs.get(character)) stack.pop();
        else if (character !== ">") { malformed = true; break; }
      } else if (character === "=" && stack.length === 0) equals.push(cursor);
      else if (character === ";" && stack.length === 0) { end = cursor; break; }
    }
    if (malformed || end < 0 || equals.length !== 1) {
      hazards.push(`${path}: type item ${nameToken.value} has an incomplete or ambiguous alias body`);
      continue;
    }
    const targetSource = structural.slice(equals[0] + 1, end).trim();
    let targetCursor = 0;
    while (["&", "("].includes(targetSource[targetCursor])) targetCursor += 1;
    while (/\s/.test(targetSource[targetCursor] ?? "")) targetCursor += 1;
    let targetToken = rustIdentifierAt(targetSource, targetCursor);
    const targetParts = [];
    if (targetToken) {
      targetParts.push(targetToken.value);
      let pathCursor = targetToken.end;
      while (targetSource.startsWith("::", pathCursor)) {
        pathCursor += 2;
        targetToken = rustIdentifierAt(targetSource, pathCursor);
        if (!targetToken) break;
        targetParts.push(targetToken.value);
        pathCursor = targetToken.end;
      }
    }
    if (targetParts.length === 0) {
      hazards.push(`${path}: type item ${nameToken.value} has an unparseable alias target`);
      continue;
    }
    aliases.push({
      name: nameToken.value,
      normalized: source.slice(start, end + 1).replace(/\s+/g, ""),
      path,
      target: targetParts.at(-1),
    });
    pattern.lastIndex = end + 1;
  }
  return { aliases, hazards };
}

function rustUseAliases(source, path) {
  const aliases = [];
  const hazards = [];
  const structural = sanitizeRust(source);
  const pattern = /\buse\s+([^;]+);/g;
  for (const match of structural.matchAll(pattern)) {
    const declaration = source.slice(match.index, match.index + match[0].length).replace(/\s+/g, "");
    if (match[1].includes("*")) hazards.push(`${path}: glob import ${declaration} is ambiguous`);
    const renamedPattern = new RegExp(`((?:${RUST_IDENTIFIER_SOURCE})(?:::(?:${RUST_IDENTIFIER_SOURCE}))*)\\s+as\\s+(${RUST_IDENTIFIER_SOURCE})`, "gu");
    const renamed = [...match[1].matchAll(renamedPattern)];
    if (renamed.length > 0 && !(ALLOWED_LLM_RENAMED_IMPORTS.get(path) ?? new Set()).has(declaration)) {
      hazards.push(`${path}: unapproved renamed import ${declaration}`);
    }
    for (const rename of renamed) {
      aliases.push({
        name: (rename[2].startsWith("r#") ? rename[2].slice(2) : rename[2]).normalize("NFC"),
        normalized: declaration,
        path,
        target: rustIdentifierValues(rename[1]).at(-1) ?? null,
      });
    }
  }
  return { aliases, hazards };
}

function rustStructuralIdentifierHazards(source) {
  const hazards = [];
  const structural = sanitizeRust(source);
  const skipWhitespace = (start) => {
    let cursor = start;
    while (/\s/.test(structural[cursor] ?? "")) cursor += 1;
    return cursor;
  };
  for (const match of structural.matchAll(/fn|type|use|impl/g)) {
    const start = match.index;
    const keyword = match[0];
    if (structural.slice(Math.max(0, start - 2), start) === "r#"
      || RUST_IDENTIFIER_CONTINUE.test(structural[start - 1] ?? "")
      || RUST_IDENTIFIER_CONTINUE.test(structural[start + keyword.length] ?? "")) continue;
    let cursor = skipWhitespace(start + keyword.length);
    if (keyword === "impl" && structural[cursor] === "<") {
      let depth = 0;
      for (; cursor < structural.length; cursor += 1) {
        if (structural[cursor] === "<") depth += 1;
        else if (structural[cursor] === ">" && --depth === 0) { cursor += 1; break; }
      }
      cursor = skipWhitespace(cursor);
    }
    if (keyword === "impl" && structural[cursor] === "!") cursor = skipWhitespace(cursor + 1);
    if (keyword === "use" && structural.startsWith("::", cursor)) cursor = skipWhitespace(cursor + 2);
    if (!rustIdentifierAt(structural, cursor)) hazards.push(`${keyword} has a missing or unparseable identifier`);
  }
  return hazards;
}

function resolveOwnedAlias(alias, aliasesByName, ownedNames, trail = new Set()) {
  if (!alias.target) return null;
  if (ownedNames.has(alias.target)) return alias.target;
  if (trail.has(alias.name)) return "CYCLE";
  const candidates = aliasesByName.get(alias.target) ?? [];
  if (candidates.length > 1) return "AMBIGUOUS";
  if (candidates.length === 0) return null;
  return resolveOwnedAlias(candidates[0], aliasesByName, ownedNames, new Set([...trail, alias.name]));
}

function hasExactCloneStruct(source, name) {
  const structural = sanitizeRust(source);
  const pattern = new RegExp(`#\\s*\\[\\s*derive\\s*\\(\\s*Clone\\s*\\)\\s*\\]\\s*pub\\s+struct\\s+${name}\\b`, "g");
  return [...structural.matchAll(pattern)].length === 1;
}

function evaluateExtractumLlmPublicApiBoundary(index) {
  const violations = [];
  const rustFiles = index.listFiles()
    .filter((path) => path.startsWith(LLM_SRC_PREFIX) && path.endsWith(".rs"))
    .sort();
  if (rustFiles.join("\n") !== [...EXPECTED_LLM_RUST_FILES].sort().join("\n")) {
    violations.push(`${LLM_SRC_PREFIX}: production Rust module inventory drifted`);
  }
  if (index.listFiles().includes(LLM_BUILD_PATH)) {
    violations.push(`${LLM_BUILD_PATH}: build-time Rust generation is forbidden`);
  }
  const root = withoutCfgTestModules(index.getText(LLM_LIB_PATH));
  if (compactRust(root) !== compactRust(EXPECTED_LLM_ROOT)) {
    violations.push(`${LLM_LIB_PATH}: closed root module/export surface drifted`);
  }
  const types = withoutCfgTestModules(index.getText(LLM_TYPES_PATH));
  const controlledSources = new Map(rustFiles.map((path) => [
    path,
    path === LLM_LIB_PATH ? root : path === LLM_TYPES_PATH ? types : withoutCfgTestModules(index.getText(path)),
  ]));
  const typeItemResults = [...controlledSources].map(([path, source]) => rustTypeItems(source, path));
  for (const hazard of typeItemResults.flatMap((result) => result.hazards)) violations.push(hazard);
  const typeAliases = typeItemResults.flatMap((result) => result.aliases);
  const useAliasResults = [...controlledSources].map(([path, source]) => rustUseAliases(source, path));
  for (const hazard of useAliasResults.flatMap((result) => result.hazards)) violations.push(hazard);
  const allAliases = [...typeAliases, ...useAliasResults.flatMap((result) => result.aliases)];
  const aliasesByName = new Map();
  for (const alias of allAliases) {
    const aliases = aliasesByName.get(alias.name) ?? [];
    aliases.push(alias);
    aliasesByName.set(alias.name, aliases);
  }
  for (const alias of typeAliases) {
    if (!(ALLOWED_LLM_TYPE_ALIASES.get(alias.path) ?? new Set()).has(alias.normalized)) {
      violations.push(`${alias.path}: unapproved type alias ${alias.name} could hide an owned inherent impl`);
    }
  }
  const ownedNames = new Set([
    ...EXPECTED_LLM_CREDENTIAL_TYPES.keys(),
    ...EXPECTED_LLM_PUBLIC_METHODS.keys(),
  ]);
  const aliasesByOwnedName = new Map([...ownedNames].map((name) => [name, []]));
  for (const alias of allAliases) {
    const resolved = resolveOwnedAlias(alias, aliasesByName, ownedNames);
    if (resolved === "CYCLE" || resolved === "AMBIGUOUS") {
      violations.push(`${alias.path}: type alias ${alias.name} has a fail-closed ${resolved.toLowerCase()} resolution`);
    } else if (resolved) {
      aliasesByOwnedName.get(resolved).push(alias.name);
    }
  }
  for (const [path, source] of controlledSources) {
    if (rustModuleItemHazards(source, ALLOWED_LLM_MODULE_ATTRIBUTES.get(path) ?? new Set()).length > 0) {
      violations.push(`${path}: contains forbidden module item macros or attributes`);
    }
    const macroHazards = rustMacroHazards(source, ALLOWED_LLM_MACROS.get(path) ?? new Set());
    if (macroHazards.length > 0) {
      violations.push(`${path}: contains unknown macro invocations capable of hiding generated items: ${macroHazards.join(", ")}`);
    }
    const identifierHazards = rustStructuralIdentifierHazards(source);
    if (identifierHazards.length > 0) {
      violations.push(`${path}: contains malformed structural identifiers: ${identifierHazards.join(", ")}`);
    }
  }
  for (const [name, expected] of EXPECTED_LLM_CREDENTIAL_TYPES) {
    if (!hasExactCloneStruct(types, name)) violations.push(`${LLM_TYPES_PATH}: ${name} must be exactly #[derive(Clone)] pub struct`);
    const fields = rustStructFields(namedRustBlock(types, `pub struct ${name}`));
    if (fields.map((field) => field.name).join("\n") !== expected.fields.join("\n")) {
      violations.push(`${LLM_TYPES_PATH}: ${name} field set drifted`);
    }
    if (fields.some((field) => field.visibility !== "private")) {
      violations.push(`${LLM_TYPES_PATH}: ${name} fields must remain private`);
    }
    const implNames = [name, ...aliasesByOwnedName.get(name)];
    const methods = implNames.flatMap((implName) =>
      [...controlledSources.values()].flatMap((source) => rustInherentMethods(source, implName)));
    if (methods.length !== expected.methods.size || methods.some((method) => expected.methods.get(method.name) !== method.visibility)) {
      violations.push(`${LLM_TYPES_PATH}: ${name} method/visibility set drifted`);
    }
    if (implNames.flatMap((implName) =>
      [...controlledSources.values()].flatMap((source) => rustInherentHazards(source, implName))).length > 0) {
      violations.push(`${LLM_TYPES_PATH}: ${name} contains forbidden associated-item macros, attributes, or non-method items`);
    }
  }
  for (const [name, expected] of EXPECTED_LLM_PUBLIC_METHODS) {
    const implNames = [name, ...aliasesByOwnedName.get(name)];
    const methods = implNames.flatMap((implName) =>
      [...controlledSources.values()].flatMap((source) => rustInherentMethods(source, implName)))
      .filter((method) => method.visibility === "pub")
      .map((method) => method.name)
      .sort();
    if (methods.join("\n") !== [...expected.methods].sort().join("\n")) {
      violations.push(`${expected.path}: ${name} public method set drifted`);
    }
    if (implNames.flatMap((implName) =>
      [...controlledSources.values()].flatMap((source) => rustInherentHazards(source, implName))).length > 0) {
      violations.push(`${expected.path}: ${name} contains forbidden associated-item macros, attributes, or non-method items`);
    }
  }
  return violations;
}

function hasNamedComponentFromModule(facts, componentName, moduleSource, importedName) {
  return facts.components.some(({ name }) => name === componentName)
    && facts.imports.some(({ source, bindings }) => source === moduleSource
      && bindings.some((binding) => binding.imported === importedName
        && binding.local === componentName
        && !binding.typeOnly));
}

function selectorContainsGlobalClass(selector, className) {
  return selector.some((part) => part.type === "pseudo"
    && part.name === "global"
    && part.arguments.some((argument) => argument.type === "class" && argument.name === className));
}

function evaluateExtractumGridWrapperBoundary(index) {
  const violations = [];
  const productionFiles = index.listFiles().filter((file) =>
    file.startsWith("src/")
    && /\.(?:svelte|[cm]?[jt]sx?)$/.test(file)
    && !/\.(?:test|spec)\./.test(file));
  for (const file of productionFiles) {
    const facts = file.endsWith(".svelte") ? index.getSvelte(file) : index.getTypeScript(file);
    const svarImports = facts.imports.filter(({ source }) => source.startsWith("@svar-ui/"));
    if (svarImports.length && !APPROVED_SVAR_GRID_PATHS.has(file)) {
      violations.push(`${file}: direct SVAR imports must stay inside Extractum grid wrappers`);
    }
  }

  for (const [file, tree] of [[DATA_GRID_PATH, false], [TREE_DATA_GRID_PATH, true]]) {
    const facts = index.getSvelte(file);
    for (const [component, moduleSource] of [
      ["Grid", "@svar-ui/svelte-grid"],
      ["Willow", "@svar-ui/svelte-grid"],
      ["Locale", "@svar-ui/svelte-core"],
    ]) {
      if (!hasNamedComponentFromModule(facts, component, moduleSource, component)) {
        violations.push(`${file}: missing ${component} wrapper composition`);
      }
    }
    if (tree && !facts.components.some(({ name, attributes }) => name === "Grid" && attributes.includes("tree"))) {
      violations.push(`${file}: tree wrapper must compose Grid with tree enabled`);
    }
  }

  const treeFacts = index.getSvelte(TREE_DATA_GRID_PATH);
  const hasScopedSvarCellStyle = treeFacts.styleRules.some((rule) =>
    rule.selectors.some((selector) => selector.some((part) =>
      part.type === "class" && part.name === "extractum-tree-data-grid")
      && selectorContainsGlobalClass(selector, "wx-cell")));
  if (!hasScopedSvarCellStyle) {
    violations.push(`${TREE_DATA_GRID_PATH}: SVAR cell styling must remain scoped by the Extractum tree wrapper`);
  }
  return violations;
}

function exactActivityTab(expression) {
  return expression?.kind === "binary"
    && expression.operator === "==="
    && ((identifier(expression.left, "activeTab") && stringLiteral(expression.right, "activity"))
      || (stringLiteral(expression.left, "activity") && identifier(expression.right, "activeTab")));
}

function conjunctionTerms(expression) {
  return expression?.kind === "binary" && expression.operator === "&&"
    ? [...conjunctionTerms(expression.left), ...conjunctionTerms(expression.right)]
    : [expression];
}

function exactPositiveActivityBranch(component, expectedIdentifiers) {
  return component.branches?.some((branch) => {
    if (branch.polarity !== "consequent") return false;
    const terms = conjunctionTerms(branch.condition);
    if (terms.length !== expectedIdentifiers.length + 1) return false;
    if (terms.filter(exactActivityTab).length !== 1) return false;
    return expectedIdentifiers.every((name) => terms.filter((term) => identifier(term, name)).length === 1);
  }) ?? false;
}

function evaluateAnalysisSourceGroupTabLeafBoundary(index) {
  const facts = index.getSvelte(SOURCE_GROUP_SOURCES_PATH);
  const violations = [];
  for (const [required, moduleSource] of [
    ["TelegramTimelineReader", "$lib/components/analysis/telegram-timeline-reader.svelte"],
    ["YoutubeTranscriptReader", "$lib/components/analysis/youtube-transcript-reader.svelte"],
  ]) {
    if (!hasComponentFromModule(facts, moduleSource)) {
      violations.push(`${SOURCE_GROUP_SOURCES_PATH}: missing leaf reader ${required}`);
    }
  }
  for (const [forbidden, moduleSource] of [
    ["SourceBrowserShell", SOURCE_BROWSER_SHELL_MODULE],
    ["SourceActivityView", SOURCE_ACTIVITY_MODULE],
  ]) {
    if (hasComponentFromModule(facts, moduleSource)) {
      violations.push(`${SOURCE_GROUP_SOURCES_PATH}: route-owning ${forbidden} is forbidden`);
    }
  }
  const routeImports = facts.imports.map(({ source }) => source).filter(routeApiImport);
  if (routeImports.length) {
    violations.push(`${SOURCE_GROUP_SOURCES_PATH}: route API imports are forbidden: ${routeImports.sort().join(", ")}`);
  }
  return violations;
}

function evaluateAnalysisSourceBrowserCanonicalComposition(index) {
  const surface = index.getSvelte(ANALYSIS_SURFACE_PATH);
  const shell = index.getSvelte(SOURCE_BROWSER_SHELL_PATH);
  const violations = [];
  if (!hasComponentFromModule(surface, SOURCE_BROWSER_SHELL_MODULE)) {
    violations.push(`${ANALYSIS_SURFACE_PATH}: missing canonical SourceBrowserShell`);
  }
  for (const [transitional, moduleSource] of TRANSITIONAL_SOURCE_COMPONENTS) {
    if (hasComponentFromModule(surface, moduleSource)) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: transitional ${transitional} composition is forbidden`);
    }
  }
  for (const [component, path, moduleSource] of CANONICAL_LEAF_PATHS) {
    if (!hasComponentFromModule(shell, moduleSource)) {
      violations.push(`${SOURCE_BROWSER_SHELL_PATH}: missing canonical leaf ${component}`);
    }
    const leaf = index.getSvelte(path);
    const routeImports = leaf.imports.map(({ source }) => source).filter(routeApiImport);
    if (routeImports.length) {
      violations.push(`${path}: route API imports are forbidden: ${routeImports.sort().join(", ")}`);
    }
    if (hasComponentFromModule(leaf, SOURCE_BROWSER_SHELL_MODULE)) {
      violations.push(`${path}: nested SourceBrowserShell is forbidden`);
    }
  }
  return violations;
}

function evaluateAnalysisSourceGroupActivityBoundary(index) {
  const shell = index.getSvelte(SOURCE_BROWSER_SHELL_PATH);
  const groupActivity = index.getSvelte(SOURCE_GROUP_ACTIVITY_PATH);
  const violations = [];
  const groupBranches = componentsFromModule(shell, SOURCE_GROUP_ACTIVITY_MODULE);
  const sourceBranches = componentsFromModule(shell, SOURCE_ACTIVITY_MODULE);
  if (!groupBranches.some((component) => exactPositiveActivityBranch(component, ["groupSubject"]))) {
    violations.push(`${SOURCE_BROWSER_SHELL_PATH}: SourceGroupActivityView must be under the exact positive activity/groupSubject branch`);
  }
  if (!sourceBranches.some((component) => exactPositiveActivityBranch(component, ["sourceSubject", "sourceData"]))) {
    violations.push(`${SOURCE_BROWSER_SHELL_PATH}: SourceActivityView must be under the exact positive activity/sourceSubject/sourceData branch`);
  }
  if (!hasComponentFromModule(groupActivity, EMPTY_STATE_MODULE)) {
    violations.push(`${SOURCE_GROUP_ACTIVITY_PATH}: missing group-owned activity leaf`);
  }
  if (hasComponentFromModule(groupActivity, SOURCE_ACTIVITY_MODULE)) {
    violations.push(`${SOURCE_GROUP_ACTIVITY_PATH}: per-source SourceActivityView is forbidden`);
  }
  const routeImports = groupActivity.imports.map(({ source }) => source).filter(routeApiImport);
  if (routeImports.length) {
    violations.push(`${SOURCE_GROUP_ACTIVITY_PATH}: route API imports are forbidden: ${routeImports.sort().join(", ")}`);
  }
  return violations;
}

function sameJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function uniqueMatch(source, pattern) {
  const matches = [...source.matchAll(pattern)];
  return matches.length === 1 ? matches[0] : undefined;
}

function retainedPhase8StatusViolations(index) {
  const violations = [];
  const phase8B = index.getText(PHASE_8B_PLAN_PATH).replaceAll("\r\n", "\n");
  const sectionStart = phase8B.indexOf("### Phase 8B Status State Machine\n");
  const sectionEnd = phase8B.indexOf("\n`TelegramLifecycle` gains exact", sectionStart);
  const section = sectionStart >= 0 && sectionEnd > sectionStart
    ? phase8B.slice(sectionStart, sectionEnd)
    : "";
  const checkpointRange = uniqueMatch(
    section,
    /roadmap `8B preparation Checkpoint (\d+) retained` through\n\s+`8B preparation Checkpoint (\d+) retained`; design\n\s+`Approved; 8B preparation Checkpoint N retained`/g,
  );
  if (checkpointRange?.[1] !== "1" || checkpointRange?.[2] !== "8") {
    violations.push(`${PHASE_8B_PLAN_PATH}: retained checkpoint status range drifted`);
  }
  const terminal = [...section.matchAll(/roadmap `([^`]+)`; design `([^`]+)`/g)]
    .find((match) => match[1] === "8B preparation retained; 8C pending");
  if (terminal?.[1] !== "8B preparation retained; 8C pending"
    || terminal?.[2] !== "Approved; 8B preparation retained; 8C pending") {
    violations.push(`${PHASE_8B_PLAN_PATH}: terminal Phase 8B retained status pair drifted`);
  }

  const roadmap = index.getText(PHASE_8_ROADMAP_PATH);
  const roadmapStatus = uniqueMatch(roadmap, /^### Phase 8 — `extractum-telegram` \(([^)]+)\)$/gm)?.[1];
  if (roadmapStatus !== "done: retained") {
    violations.push(`${PHASE_8_ROADMAP_PATH}: retained Phase 8 roadmap status drifted`);
  }
  const design = index.getText(PHASE_8_DESIGN_PATH);
  const designStatus = uniqueMatch(design, /^\*\*Status:\*\* (.+)$/gm)?.[1];
  if (designStatus !== "Implemented and retained; [verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)") {
    violations.push(`${PHASE_8_DESIGN_PATH}: retained Phase 8 design status drifted`);
  }
  return violations;
}

function evaluateTelegramPhase8BAuthorityIntegrity(index) {
  const phase8A = index.getText(PHASE_8A_PLAN_PATH);
  const phase8B = index.getText(PHASE_8B_PLAN_PATH);
  const violations = [];
  if (!sameJson(index.getJson(SYMBOL_MAP_PATH), generateSymbolAuthority(phase8B))) {
    violations.push(`${SYMBOL_MAP_PATH}: generated authority drifted`);
  }
  if (!sameJson(index.getJson(TEST_IDENTITIES_PATH), generateIdentityAuthority(phase8A, phase8B))) {
    violations.push(`${TEST_IDENTITIES_PATH}: generated authority drifted`);
  }

  const stagingSource = index.getText(STAGING_SHA_PATH).replaceAll("\r\n", "\n");
  const staging = index.getJson(STAGING_SHA_PATH);
  const paths = Array.isArray(staging?.files) ? staging.files.map((entry) => entry?.path) : [];
  const validRecords = Array.isArray(staging?.files)
    && staging.files.length === 19
    && staging.files.every((entry) => entry
      && Object.keys(entry).sort().join(",") === "path,sha256"
      && typeof entry.path === "string"
      && /^[a-f0-9]{64}$/.test(entry.sha256));
  if (staging?.schemaVersion !== 1
    || staging?.algorithm !== "sha256"
    || staging?.root !== "src-tauri/src/telegram_impl"
    || !validRecords
    || new Set(paths).size !== paths.length
    || paths.some((value, index) => index > 0 && paths[index - 1].localeCompare(value) >= 0)) {
    violations.push(`${STAGING_SHA_PATH}: frozen artifact schema drifted`);
  }
  const contentAddress = createHash("sha256").update(stagingSource).digest("hex");
  if (contentAddress !== FROZEN_STAGING_SHA256) {
    violations.push(`${STAGING_SHA_PATH}: frozen content address drifted`);
  }
  violations.push(...retainedPhase8StatusViolations(index));
  return violations;
}

function workspacePackage(metadata, name) {
  const members = new Set(metadata?.workspace_members ?? []);
  const matches = (metadata?.packages ?? []).filter((candidate) => candidate?.name === name && members.has(candidate.id));
  return matches.length === 1 ? matches[0] : undefined;
}

function resolvedNode(metadata, packageId) {
  const matches = (metadata?.resolve?.nodes ?? []).filter((candidate) => candidate?.id === packageId);
  return matches.length === 1 ? matches[0] : undefined;
}

function dependencyTuple(dependency) {
  return [
    dependency?.name,
    dependency?.kind ?? null,
    [...(dependency?.features ?? [])].sort(),
    dependency?.uses_default_features ?? true,
    dependency?.target ?? null,
    dependency?.rename ?? null,
  ];
}

function sortedJson(values) {
  return values.map((value) => JSON.stringify(value)).sort().join("\n");
}

function exactDependencyInventory(actual, expected) {
  return sortedJson(actual.map(dependencyTuple)) === sortedJson(expected);
}

function exactDependencyKinds(edge, expectedKinds) {
  const actual = (edge?.dep_kinds ?? []).map(({ kind, target }) => [kind ?? null, target ?? null]);
  return sortedJson(actual) === sortedJson(expectedKinds.map((kind) => [kind, null]));
}

function evaluateTelegramCrateManifestBoundary(index) {
  const metadata = index.getCargoMetadata();
  const violations = [];
  const app = workspacePackage(metadata, "extractum");
  const producer = workspacePackage(metadata, "extractum-telegram");
  if (!app) violations.push("Cargo metadata: missing extractum workspace package");
  if (!producer) return [...violations, "Cargo metadata: missing extractum-telegram workspace package"];
  const producerTargets = (producer.targets ?? []).filter((target) => target?.kind?.includes("lib"));
  if (producerTargets.length !== 1
    || producerTargets[0]?.name !== "extractum_telegram"
    || sortedJson(producerTargets[0]?.kind ?? []) !== sortedJson(["lib"])) {
    violations.push("extractum-telegram: library target inventory drifted");
  }
  if (!sameJson(producer.features ?? {}, { "app-test-support": [] })) {
    violations.push("extractum-telegram: feature inventory must contain only app-test-support");
  }
  if (!exactDependencyInventory(producer.dependencies ?? [], EXPECTED_PRODUCER_DEPENDENCIES)) {
    violations.push("extractum-telegram: dependency and dev-dependency inventory drifted");
  }
  if (app) {
    const edges = (app.dependencies ?? []).filter((dependency) => dependency?.name === "extractum-telegram");
    const normal = edges.filter((dependency) => dependency.kind === null);
    const development = edges.filter((dependency) => dependency.kind === "dev");
    if (normal.length !== 1
      || !exactDependencyInventory(normal, [EXPECTED_APP_TELEGRAM_DEPENDENCIES[0]])) {
      violations.push("extractum: production extractum-telegram edge must be feature-free");
    }
    if (development.length !== 1
      || !exactDependencyInventory(development, [EXPECTED_APP_TELEGRAM_DEPENDENCIES[1]])) {
      violations.push("extractum: dev extractum-telegram edge must enable only app-test-support");
    }
    if (!exactDependencyInventory(edges, EXPECTED_APP_TELEGRAM_DEPENDENCIES)) {
      violations.push("extractum: Telegram dependency kinds or targets drifted");
    }

    const appNode = resolvedNode(metadata, app.id);
    const resolvedEdges = (appNode?.deps ?? []).filter(({ pkg }) => pkg === producer.id);
    if (resolvedEdges.length !== 1 || !exactDependencyKinds(resolvedEdges[0], [null, "dev"])) {
      violations.push("extractum: resolver-v2 Telegram normal/dev edge semantics drifted");
    }
  }

  const producerNode = resolvedNode(metadata, producer.id);
  if (!producerNode || sortedJson(producerNode.features ?? []) !== sortedJson(["app-test-support"])) {
    violations.push("extractum-telegram: resolved app-test-support feature drifted");
  }
  const expectedResolvedDependencies = EXPECTED_PRODUCER_DEPENDENCIES
    .filter(([name, kind]) => !(name === "tokio" && kind === "dev"))
    .map(([name]) => String(name).replaceAll("-", "_"));
  const actualResolvedDependencies = (producerNode?.deps ?? []).map(({ name }) => name);
  if (sortedJson(actualResolvedDependencies) !== sortedJson(expectedResolvedDependencies)) {
    violations.push("extractum-telegram: resolved dependency inventory drifted");
  }
  const tokioResolved = (producerNode?.deps ?? []).filter(({ name }) => name === "tokio");
  if (tokioResolved.length !== 1 || !exactDependencyKinds(tokioResolved[0], [null, "dev"])) {
    violations.push("extractum-telegram: resolved Tokio normal/dev edge semantics drifted");
  }

  const workspaceMembers = new Set(metadata?.workspace_members ?? []);
  const featureMentions = [];
  for (const workspace of (metadata?.packages ?? []).filter(({ id }) => workspaceMembers.has(id))) {
    if (Object.prototype.hasOwnProperty.call(workspace.features ?? {}, "app-test-support")) {
      featureMentions.push(`${workspace.name}|feature`);
    }
    for (const dependency of workspace.dependencies ?? []) {
      if ((dependency.features ?? []).includes("app-test-support")) {
        featureMentions.push(`${workspace.name}|${dependency.kind ?? "normal"}`);
      }
    }
  }
  if (sortedJson(featureMentions) !== sortedJson([
    "extractum-telegram|feature",
    "extractum|dev",
  ])) {
    violations.push("Cargo metadata: app-test-support feature mentions drifted");
  }
  return violations;
}

function evaluateTelegramCrateDependencyOwnership(index) {
  const metadata = index.getCargoMetadata();
  const baseline = index.getJson(GRAMMERS_BASELINE_PATH);
  const violations = [];
  let generatedBaseline;
  try {
    generatedBaseline = generateFeatureBaseline(metadata);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    violations.push(`Cargo metadata: cannot generate Grammers feature baseline: ${detail}`);
    return violations;
  }
  if (!sameJson(baseline, generatedBaseline)) {
    violations.push(`${GRAMMERS_BASELINE_PATH}: generated baseline is not canonical or reproducible`);
  }
  const app = workspacePackage(metadata, "extractum");
  const producer = workspacePackage(metadata, "extractum-telegram");
  if (!app || !producer) return ["Cargo metadata: missing Telegram owner package"];
  const appGrammers = (app.dependencies ?? []).filter((dependency) => dependency?.name?.startsWith("grammers-"));
  if (appGrammers.length) violations.push(`extractum: direct Grammers dependencies are forbidden: ${appGrammers.map(({ name }) => name).sort().join(", ")}`);

  const expectedPackages = Array.isArray(baseline?.packages) ? baseline.packages : [];
  const producerGrammers = (producer.dependencies ?? []).filter((dependency) => dependency?.name?.startsWith("grammers-"));
  const actualNames = producerGrammers.map(({ name }) => name).sort();
  const expectedNames = expectedPackages.map(({ name }) => name).sort();
  if (actualNames.join("\n") !== expectedNames.join("\n")) {
    violations.push("extractum-telegram: direct Grammers dependency inventory drifted");
  }
  for (const expected of expectedPackages) {
    const packages = (metadata.packages ?? []).filter(({ name }) => name === expected.name);
    if (packages.length !== 1) {
      violations.push(`${expected.name}: expected one Cargo metadata package`);
      continue;
    }
    const selected = packages[0];
    const expectedSource = `git+https://codeberg.org/Lonami/grammers?rev=${baseline.revision}#${baseline.revision}`;
    if (selected.source !== expectedSource) violations.push(`${expected.name}: source revision drifted`);
    if (Object.keys(selected.features ?? {}).sort().join("\n") !== [...expected.universe].sort().join("\n")) {
      violations.push(`${expected.name}: feature universe drifted`);
    }
    const node = resolvedNode(metadata, selected.id);
    if (!node) {
      violations.push(`${expected.name}: missing resolved Cargo node`);
      continue;
    }
    const enabled = [...(node.features ?? [])].sort();
    if (enabled.join("\n") !== [...expected.required].sort().join("\n")
      || enabled.some((feature) => expected.forbidden.includes(feature))) {
      violations.push(`${expected.name}: enabled feature closure drifted`);
    }
  }
  return violations;
}

const evaluators = new Map([
  ["rule:analysis-evidence-highlight-token-styling", evaluateAnalysisEvidenceHighlightTokenStyling],
  ["rule:analysis-source-browser-canonical-composition", evaluateAnalysisSourceBrowserCanonicalComposition],
  ["rule:analysis-source-browser-explicit-subject-contract", evaluateAnalysisSourceBrowserExplicitSubjectContract],
  ["rule:analysis-source-group-activity-boundary", evaluateAnalysisSourceGroupActivityBoundary],
  ["rule:analysis-source-group-tab-leaf-boundary", evaluateAnalysisSourceGroupTabLeafBoundary],
  ["rule:analysis-source-reader-surface-composition", evaluateAnalysisSourceReaderSurfaceComposition],
  ["rule:extractum-llm-public-api-boundary", evaluateExtractumLlmPublicApiBoundary],
  ["rule:extractum-grid-wrapper-boundary", evaluateExtractumGridWrapperBoundary],
  ["rule:telegram-crate-dependency-ownership", evaluateTelegramCrateDependencyOwnership],
  ["rule:telegram-crate-manifest-boundary", evaluateTelegramCrateManifestBoundary],
  ["rule:telegram-phase-8b-authority-integrity", evaluateTelegramPhase8BAuthorityIntegrity],
]);

export const registeredRuleIds = Object.freeze([...evaluators.keys()].sort());

export function evaluateRule({ id, index }) {
  const evaluator = evaluators.get(id);
  if (!evaluator) throw new Error(`Unknown repository rule ID: ${id}`);
  try {
    return { id, violations: evaluator(index) };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { id, violations: [`INFRA_ERROR: ${detail}`] };
  }
}
