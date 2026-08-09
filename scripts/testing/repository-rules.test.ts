import path from "node:path";
import * as svelte from "svelte/compiler";
import ts from "typescript";
import { describe, expect, it } from "vitest";

import sourceContractLedger from "../../testing/source-contract-ledger.json";
import { createRepositoryIndex } from "./repository-index.mjs";
import { evaluateRule, registeredRuleIds } from "./repository-rules.mjs";

const root = path.resolve("repository-rule-fixture");
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";
const SOURCE_BROWSER_SHELL_PATH = "src/lib/components/analysis/source-browser-shell.svelte";
const SOURCE_GROUP_SOURCES_PATH = "src/lib/components/analysis/source-group-sources-view.svelte";
const SOURCE_GROUP_ACTIVITY_PATH = "src/lib/components/analysis/source-group-activity-view.svelte";
const SNAPSHOT_GROUP_SOURCES_PATH = "src/lib/components/analysis/snapshot-group-sources-view.svelte";
const SNAPSHOT_ITEMS_PATH = "src/lib/components/analysis/snapshot-items-view.svelte";
const RUN_SNAPSHOT_METADATA_PATH = "src/lib/components/analysis/run-snapshot-metadata-view.svelte";
const TELEGRAM_TIMELINE_PATH = "src/lib/components/analysis/telegram-timeline-reader.svelte";
const YOUTUBE_TRANSCRIPT_PATH = "src/lib/components/analysis/youtube-transcript-reader.svelte";
const UNIVERSAL_ITEMS_PATH = "src/lib/components/analysis/universal-items-view.svelte";
const YOUTUBE_COMMENTS_PATH = "src/lib/components/analysis/youtube-comments-view.svelte";
const SYMBOL_MAP_PATH = "src/lib/telegram-8b-symbol-map.json";
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";
const PHASE_8B_PLAN_PATH = "docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md";
const PHASE_8_ROADMAP_PATH = "docs/superpowers/specs/2026-07-17-crate-roadmap.md";
const PHASE_8_DESIGN_PATH = "docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md";
const DATA_GRID_PATH = "src/lib/components/extractum-ui/DataGrid.svelte";
const TREE_DATA_GRID_PATH = "src/lib/components/extractum-ui/TreeDataGrid.svelte";
const GRID_SELECT_CELL_PATH = "src/lib/components/extractum-ui/GridSelectCell.svelte";
const LLM_LIB_PATH = "src-tauri/crates/extractum-llm/src/lib.rs";
const LLM_TYPES_PATH = "src-tauri/crates/extractum-llm/src/types.rs";
const LLM_PROVIDER_PATH = "src-tauri/crates/extractum-llm/src/provider.rs";
const LLM_RUNNER_PATH = "src-tauri/crates/extractum-llm/src/runner.rs";
const LLM_SCHEDULER_PATH = "src-tauri/crates/extractum-llm/src/scheduler.rs";

const llmPublicApiBoundarySources = {
  [LLM_LIB_PATH]: `
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
#[cfg(test)]
mod public_api_tests { const SAMPLE: &str = r#"{ pub mod ignored; }"#; }
`,
  [LLM_TYPES_PATH]: `
#[derive(Clone)]
pub struct LlmProviderAccess { provider: ProviderKind, api_key: SecretString, base_url: String }
impl LlmProviderAccess {
  pub fn new(provider: ProviderKind, api_key: SecretString, base_url: String) -> Self { todo!() }
  pub(super) fn provider(&self) -> ProviderKind { todo!() }
  pub(super) fn api_key(&self) -> &SecretString { todo!() }
  pub(super) fn base_url(&self) -> &str { todo!() }
}
trait CredentialTrait { fn credential_trait(&self); }
impl CredentialTrait for LlmProviderAccess { fn credential_trait(&self) {} }
#[derive(Clone)]
pub struct ResolvedLlmProfile { profile_id: String, default_model: String, provider_access: LlmProviderAccess }
impl ResolvedLlmProfile {
  pub fn new(profile_id: String, default_model: String, provider_access: LlmProviderAccess) -> Self { todo!() }
  pub fn profile_id(&self) -> &str { todo!() }
  pub fn provider(&self) -> ProviderKind { todo!() }
  pub fn default_model(&self) -> &str { todo!() }
  pub fn base_url(&self) -> &str { todo!() }
  pub(super) fn provider_access(&self) -> &LlmProviderAccess { todo!() }
}
`,
  [LLM_PROVIDER_PATH]: `
pub enum ProviderKind { Gemini, OpenAiCompatible }
impl ProviderKind {
  pub fn as_str(self) -> &'static str { todo!() }
  pub fn parse(value: &str) -> AppResult<Self> { todo!() }
  extern "C" fn private_ffi_probe() {}
}
`,
  "src-tauri/crates/extractum-llm/src/gemini.rs": "",
  "src-tauri/crates/extractum-llm/src/openai_compat.rs": "",
  [LLM_RUNNER_PATH]: "fn runner_positive_control() { format!(\"runner\"); }",
  [LLM_SCHEDULER_PATH]: `
pub struct LlmRequestControl;
impl LlmRequestControl { pub async fn run_cancellable(&self) { todo!() } }
pub struct LlmSchedulerState;
impl LlmSchedulerState {
  pub fn new() -> Self { todo!() }
  pub async fn cancel_request(&self) -> bool { todo!() }
  pub async fn cancel_run_requests(&self) -> usize { todo!() }
  pub async fn request_snapshots(&self) { todo!() }
  pub async fn active_owner_run_ids(&self) { todo!() }
  pub async fn run_request(&self) { todo!() }
}
`,
  "src-tauri/crates/extractum-llm/src/streaming.rs": "",
};

const extractumGridBoundarySources = {
  [DATA_GRID_PATH]: `
    <script lang="ts">
      import { Grid, Willow } from "@svar-ui/svelte-grid";
      import { Locale } from "@svar-ui/svelte-core";
    </script>
    <Locale><Willow fonts={false}><Grid /></Willow></Locale>
  `,
  [TREE_DATA_GRID_PATH]: `
    <script lang="ts">
      import { Grid, Willow } from "@svar-ui/svelte-grid";
      import { Locale } from "@svar-ui/svelte-core";
    </script>
    <Locale><Willow fonts={false}><Grid tree /></Willow></Locale>
    <style>.extractum-tree-data-grid :global(.wx-cell) { padding: 4px; }</style>
  `,
  [GRID_SELECT_CELL_PATH]: `<input data-action="ignore-click" />`,
  "src/lib/components/research-projects/FeatureGrid.svelte": `<section>Feature</section>`,
};

type RuleFixture = {
  positive: Record<string, string>;
  mutations: Record<string, Record<string, string>>;
};

const explicitSubjectSurface = `
  <script lang="ts">
    import SourceBrowserShell from "$lib/components/analysis/source-browser-shell.svelte";
    const source = {};
    const group = {};
    const snapshot = {};
  </script>
  <SourceBrowserShell subject={source} />
  <SourceBrowserShell subject={group} />
  <SourceBrowserShell subject={snapshot} />
`;

function evidenceStyle(tag: string, token: "primary" | "accent", paired = true) {
  const selected = paired ? `${tag}.selected, ` : "";
  return `<${tag}>Example</${tag}><style>${selected}${tag}[data-evidence-highlighted="true"] { background: var(--${token}); }</style>`;
}

const evidenceHighlightSources = {
  [TELEGRAM_TIMELINE_PATH]: evidenceStyle("li", "primary"),
  [YOUTUBE_TRANSCRIPT_PATH]: evidenceStyle("li", "primary"),
  [SNAPSHOT_ITEMS_PATH]: evidenceStyle("article", "accent"),
  [SNAPSHOT_GROUP_SOURCES_PATH]: evidenceStyle("li", "accent"),
  [UNIVERSAL_ITEMS_PATH]: evidenceStyle("article", "accent", false),
  [YOUTUBE_COMMENTS_PATH]: evidenceStyle("article", "accent", false),
};

const canonicalCompositionSources = {
  [ANALYSIS_SURFACE_PATH]: `
    <script lang="ts">import SourceBrowserShell from "$lib/components/analysis/source-browser-shell.svelte";</script>
    <SourceBrowserShell />
  `,
  [SOURCE_BROWSER_SHELL_PATH]: `
    <script lang="ts">
      import SourceGroupSourcesView from "$lib/components/analysis/source-group-sources-view.svelte";
      import SnapshotGroupSourcesView from "$lib/components/analysis/snapshot-group-sources-view.svelte";
      import SnapshotItemsView from "$lib/components/analysis/snapshot-items-view.svelte";
      import RunSnapshotMetadataView from "$lib/components/analysis/run-snapshot-metadata-view.svelte";
    </script>
    <SourceGroupSourcesView />
    <SnapshotGroupSourcesView />
    <SnapshotItemsView />
    <RunSnapshotMetadataView />
  `,
  [SOURCE_GROUP_SOURCES_PATH]: "<section>Group sources</section>",
  [SNAPSHOT_GROUP_SOURCES_PATH]: "<section>Snapshot sources</section>",
  [SNAPSHOT_ITEMS_PATH]: "<section>Snapshot items</section>",
  [RUN_SNAPSHOT_METADATA_PATH]: "<section>Snapshot metadata</section>",
};

const ruleFixtures: Record<string, RuleFixture> = {
  "rule:extractum-llm-public-api-boundary": {
    positive: llmPublicApiBoundarySources,
    mutations: {
      "publicizes an internal module": {
        ...llmPublicApiBoundarySources,
        [LLM_LIB_PATH]: llmPublicApiBoundarySources[LLM_LIB_PATH].replace("mod runner;", "pub mod runner;"),
      },
      "adds an uncurated root export": {
        ...llmPublicApiBoundarySources,
        [LLM_LIB_PATH]: `${llmPublicApiBoundarySources[LLM_LIB_PATH]}\npub use types::InternalCredential;`,
      },
      "adds a credential field": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace("api_key: SecretString", "pub api_key: SecretString"),
      },
      "removes Clone from LlmProviderAccess": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace("#[derive(Clone)]\npub struct LlmProviderAccess", "pub struct LlmProviderAccess"),
      },
      "removes Clone from ResolvedLlmProfile": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace("#[derive(Clone)]\npub struct ResolvedLlmProfile", "pub struct ResolvedLlmProfile"),
      },
      "adds a credential-returning accessor under a novel name": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace("pub(super) fn api_key", "pub fn credential_material"),
      },
      "leaks an internal helper": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace("pub(super) fn provider_access", "pub fn provider_access"),
      },
      "adds a credential accessor in a second inherent impl": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}\nimpl LlmProviderAccess { pub fn reveal_material(&self) -> &SecretString { todo!() } }`,
      },
      "adds a Cyrillic public credential method": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
impl LlmProviderAccess { pub fn показать_секрет(&self) -> &SecretString { todo!() } }`,
      },
      "adds a qualified credential impl in a different production module": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}
impl crate::LlmProviderAccess {
  pub fn cross_module_credential(&self) -> &SecretString { todo!() }
}`,
      },
      "adds an inherent impl through a credential type alias": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
type AccessAlias = LlmProviderAccess;
impl AccessAlias { pub fn alias_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds an inherent impl through a raw credential type alias": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
type r#type = LlmProviderAccess;
impl r#type { pub fn raw_alias_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds an inherent impl through a Unicode credential type alias": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
type Доступ = LlmProviderAccess;
impl Доступ { pub fn материал(&self) -> &SecretString { todo!() } }`,
      },
      "adds an inherent impl through a credential alias chain": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
type AccessAlias = LlmProviderAccess;
type ChainedAccessAlias = AccessAlias;
impl ChainedAccessAlias { pub fn chained_alias_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds a credential alias with a pre-equals where clause": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
type WhereAccessAlias where LlmProviderAccess: Sized = LlmProviderAccess;
impl WhereAccessAlias { pub fn where_alias_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds a cross-module qualified credential alias impl": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}
type RunnerAccessAlias = crate::LlmProviderAccess;
impl RunnerAccessAlias { pub fn runner_alias_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds an unregistered production Rust module file": {
        ...llmPublicApiBoundarySources,
        "src-tauri/crates/extractum-llm/src/generated_access.rs": "pub fn generated_access() {}",
      },
      "adds a crate build script generation path": {
        ...llmPublicApiBoundarySources,
        "src-tauri/crates/extractum-llm/build.rs": "fn main() {}",
      },
      "generates a public credential accessor from an associated item macro": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
macro_rules! expose { () => { pub fn generated_credential(&self) -> &SecretString { todo!() } } }
impl LlmProviderAccess { expose!(); }`,
      },
      "generates a whole credential impl from a module item macro": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
macro_rules! expose_impl {
  () => { impl LlmProviderAccess { pub fn generated_module_credential(&self) -> &SecretString { todo!() } } }
}
expose_impl!();`,
      },
      "generates an owned impl from a function-local macro": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
fn local_impl_scope() {
  macro_rules! expose_local_impl {
    () => { impl LlmProviderAccess { pub fn local_generated_credential(&self) -> &SecretString { todo!() } } }
  }
  expose_local_impl!();
}`,
      },
      "adds a self use-alias credential impl": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
use self::LlmProviderAccess as SelfAccessAlias;
impl SelfAccessAlias { pub fn self_import_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds a raw renamed credential import impl": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}
use crate::LlmProviderAccess as r#type;
impl r#type { pub fn raw_import_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds a Unicode renamed credential import impl": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}
use crate::LlmProviderAccess as Доступ;
impl Доступ { pub fn материал(&self) -> &SecretString { todo!() } }`,
      },
      "leaves a recognized fn without an identifier": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}\nimpl LlmProviderAccess { pub fn (&self) {} }`,
      },
      "leaves recognized type and use tokens without identifiers": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}\ntype = LlmProviderAccess;\nuse ;`,
      },
      "adds a grouped crate use-alias credential impl": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}
use crate::{LlmProviderAccess as GroupAccessAlias};
impl GroupAccessAlias { pub fn grouped_import_credential(&self) -> &SecretString { todo!() } }`,
      },
      "adds an owned impl inside a const block": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
const _: () = {
  impl LlmProviderAccess { pub fn const_block_credential(&self) -> &SecretString { todo!() } }
};`,
      },
      "invokes an unknown macro inside a function body": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
fn external_impl_scope() { foreign_impl_generator!(); }`,
      },
      "adds an unknown module item attribute": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace(
          "#[derive(Clone)]\npub struct LlmProviderAccess",
          "#[doc(hidden)]\n#[derive(Clone)]\npub struct LlmProviderAccess",
        ),
      },
      "adds a cfg_attr module item transformer": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace(
          "#[derive(Clone)]\npub struct LlmProviderAccess",
          "#[cfg_attr(any(), derive(Debug))]\n#[derive(Clone)]\npub struct LlmProviderAccess",
        ),
      },
      "includes generated module items": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}\ninclude!("generated_items.rs");`,
      },
      "adds an attribute to an otherwise exact credential method": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: llmPublicApiBoundarySources[LLM_TYPES_PATH].replace(
          "pub(super) fn api_key",
          "#[doc(hidden)] pub(super) fn api_key",
        ),
      },
      "adds a credential accessor in a multiline where-clause inherent impl": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
impl
  LlmProviderAccess
where
  LlmProviderAccess: Clone,
{
  pub fn reveal_where_material(&self) -> &SecretString { todo!() }
}`,
      },
      "adds an ambiguous crate glob import": {
        ...llmPublicApiBoundarySources,
        [LLM_RUNNER_PATH]: `${llmPublicApiBoundarySources[LLM_RUNNER_PATH]}\nuse crate::*;`,
      },
      "hides a novel accessor after a closing-brace char literal": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
impl LlmProviderAccess {
  const QUOTE: char = '\\'';
  const BYTE_MASK: u8 = b'}';
  const MASK: char = '}';
  pub fn reveal_after_char(&self) -> &SecretString { todo!() }
}`,
      },
      "hides a novel accessor after an arbitrary-hash raw string": {
        ...llmPublicApiBoundarySources,
        [LLM_TYPES_PATH]: `${llmPublicApiBoundarySources[LLM_TYPES_PATH]}
impl LlmProviderAccess {
  const MASK: &str = r#################"embedded quote \" and }"#################;
  pub fn reveal_after_raw(&self) -> &SecretString { todo!() }
}`,
      },
      "drifts ProviderKind public methods": {
        ...llmPublicApiBoundarySources,
        [LLM_PROVIDER_PATH]: `${llmPublicApiBoundarySources[LLM_PROVIDER_PATH]}\nimpl ProviderKind { pub fn label(self) -> &'static str { todo!() } }`,
      },
      "drifts ProviderKind with a public const method": {
        ...llmPublicApiBoundarySources,
        [LLM_PROVIDER_PATH]: `${llmPublicApiBoundarySources[LLM_PROVIDER_PATH]}\nimpl ProviderKind { pub const fn const_probe() {} }`,
      },
      "drifts ProviderKind with a public async method": {
        ...llmPublicApiBoundarySources,
        [LLM_PROVIDER_PATH]: `${llmPublicApiBoundarySources[LLM_PROVIDER_PATH]}\nimpl ProviderKind { pub async fn async_probe() {} }`,
      },
      "drifts ProviderKind with a public unsafe method": {
        ...llmPublicApiBoundarySources,
        [LLM_PROVIDER_PATH]: `${llmPublicApiBoundarySources[LLM_PROVIDER_PATH]}\nimpl ProviderKind { pub unsafe fn unsafe_probe() {} }`,
      },
      "drifts ProviderKind with a public extern method": {
        ...llmPublicApiBoundarySources,
        [LLM_PROVIDER_PATH]: `${llmPublicApiBoundarySources[LLM_PROVIDER_PATH]}\nimpl ProviderKind { pub extern "C" fn extern_probe() {} }`,
      },
      "drifts ProviderKind with ordered const unsafe extern qualifiers": {
        ...llmPublicApiBoundarySources,
        [LLM_PROVIDER_PATH]: `${llmPublicApiBoundarySources[LLM_PROVIDER_PATH]}\nimpl ProviderKind { pub const unsafe extern "C" fn qualified_probe() {} }`,
      },
      "drifts LlmRequestControl public methods": {
        ...llmPublicApiBoundarySources,
        [LLM_SCHEDULER_PATH]: `${llmPublicApiBoundarySources[LLM_SCHEDULER_PATH]}\nimpl LlmRequestControl { pub fn bypass(&self) {} }`,
      },
      "drifts LlmSchedulerState public methods": {
        ...llmPublicApiBoundarySources,
        [LLM_SCHEDULER_PATH]: `${llmPublicApiBoundarySources[LLM_SCHEDULER_PATH]}\nimpl LlmSchedulerState { pub fn queue_depth(&self) -> usize { 0 } }`,
      },
      "adds an indented public module after cfg(test)": {
        ...llmPublicApiBoundarySources,
        [LLM_LIB_PATH]: `${llmPublicApiBoundarySources[LLM_LIB_PATH]}\n   pub mod escaped_module;`,
      },
      "adds an indented public export after cfg(test)": {
        ...llmPublicApiBoundarySources,
        [LLM_LIB_PATH]: `${llmPublicApiBoundarySources[LLM_LIB_PATH]}\n   pub use types::EscapedCredential;`,
      },
    },
  },
  "rule:extractum-grid-wrapper-boundary": {
    positive: extractumGridBoundarySources,
    mutations: {
      "imports SVAR from a feature component": {
        ...extractumGridBoundarySources,
        "src/lib/components/research-projects/FeatureGrid.svelte": `
          <script lang="ts">import { Grid } from "@svar-ui/svelte-grid";</script>
          <Grid />
        `,
      },
      "drops the tree wrapper scoped SVAR style": {
        ...extractumGridBoundarySources,
        [TREE_DATA_GRID_PATH]: extractumGridBoundarySources[TREE_DATA_GRID_PATH].replace(
          '<style>.extractum-tree-data-grid :global(.wx-cell) { padding: 4px; }</style>',
          "",
        ),
      },
    },
  },
  "rule:analysis-source-reader-surface-composition": {
    positive: {
      [ANALYSIS_SURFACE_PATH]: `
        <script lang="ts">
          import SourceBrowserShell from "$lib/components/analysis/source-browser-shell.svelte";
        </script>
        <SourceBrowserShell />
      `,
    },
    mutations: {
      "restores a transitional source reader": {
        [ANALYSIS_SURFACE_PATH]: `
          <script lang="ts">
            import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
          </script>
          <TelegramTimelineReader />
        `,
      },
      "aliases an unrelated module as SourceBrowserShell": {
        [ANALYSIS_SURFACE_PATH]: `
          <script lang="ts">import SourceBrowserShell from "$lib/components/ui/Button.svelte";</script>
          <SourceBrowserShell />
        `,
      },
    },
  },
  "rule:analysis-source-browser-explicit-subject-contract": {
    positive: { [ANALYSIS_SURFACE_PATH]: explicitSubjectSurface },
    mutations: {
      "restores a legacy source prop on one shell": {
        [ANALYSIS_SURFACE_PATH]: explicitSubjectSurface.replace(
          "<SourceBrowserShell subject={group} />",
          "<SourceBrowserShell source={group} />",
        ),
      },
      "aliases an unrelated module as SourceBrowserShell": {
        [ANALYSIS_SURFACE_PATH]: explicitSubjectSurface.replace(
          "$lib/components/analysis/source-browser-shell.svelte",
          "$lib/components/ui/Button.svelte",
        ),
      },
    },
  },
  "rule:analysis-evidence-highlight-token-styling": {
    positive: evidenceHighlightSources,
    mutations: {
      "hardcodes one evidence highlight color": {
        ...evidenceHighlightSources,
        [YOUTUBE_COMMENTS_PATH]: evidenceHighlightSources[YOUTUBE_COMMENTS_PATH].replace("var(--accent)", "red"),
      },
    },
  },
  "rule:analysis-source-group-tab-leaf-boundary": {
    positive: {
      [SOURCE_GROUP_SOURCES_PATH]: `
        <script lang="ts">
          import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
          import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
        </script>
        <TelegramTimelineReader />
        <YoutubeTranscriptReader />
      `,
    },
    mutations: {
      "nests the route-owning source browser shell": {
        [SOURCE_GROUP_SOURCES_PATH]: `
          <script lang="ts">import SourceBrowserShell from "$lib/components/analysis/source-browser-shell.svelte";</script>
          <SourceBrowserShell />
        `,
      },
      "imports a route API": {
        [SOURCE_GROUP_SOURCES_PATH]: `
          <script lang="ts">
            import { invoke } from "@tauri-apps/api/core";
            import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
            import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
          </script>
          <TelegramTimelineReader />
          <YoutubeTranscriptReader />
        `,
      },
      "aliases the Telegram leaf to the YouTube module": {
        [SOURCE_GROUP_SOURCES_PATH]: `
          <script lang="ts">
            import TelegramTimelineReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
            import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
          </script>
          <TelegramTimelineReader />
          <YoutubeTranscriptReader />
        `,
      },
    },
  },
  "rule:analysis-source-browser-canonical-composition": {
    positive: canonicalCompositionSources,
    mutations: {
      "drops one canonical snapshot leaf": {
        ...canonicalCompositionSources,
        [SOURCE_BROWSER_SHELL_PATH]: canonicalCompositionSources[SOURCE_BROWSER_SHELL_PATH]
          .replace("    <SnapshotItemsView />\n", ""),
      },
      "moves route API ownership into a leaf": {
        ...canonicalCompositionSources,
        [SNAPSHOT_ITEMS_PATH]: `
          <script lang="ts">import { invoke } from "@tauri-apps/api/core";</script>
          <section>Snapshot items</section>
        `,
      },
      "aliases a canonical leaf to the wrong module": {
        ...canonicalCompositionSources,
        [SOURCE_BROWSER_SHELL_PATH]: canonicalCompositionSources[SOURCE_BROWSER_SHELL_PATH].replace(
          "$lib/components/analysis/source-group-sources-view.svelte",
          "$lib/components/analysis/source-group-metadata-view.svelte",
        ),
      },
    },
  },
  "rule:analysis-source-group-activity-boundary": {
    positive: {
      [SOURCE_BROWSER_SHELL_PATH]: `
        <script lang="ts">
          import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
          import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
        </script>
        {#if activeTab === "activity" && groupSubject}<SourceGroupActivityView />{/if}
        {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
      `,
      [SOURCE_GROUP_ACTIVITY_PATH]: `
        <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
        <EmptyState />
      `,
    },
    mutations: {
      "nests per-source activity inside group activity": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && groupSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">
            import EmptyState from "$lib/components/ui/EmptyState.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          <EmptyState />
          <SourceActivityView />
        `,
      },
      "aliases group activity to the per-source module": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && groupSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "swaps group and source activity branches": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && sourceSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && groupSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "negates the group subject": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && !groupSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "compares the group subject with null": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && groupSubject !== null}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "uses an inactive tab condition": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab !== "activity" && groupSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "uses a misleading OR branch": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" || groupSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "adds a misleading compound subject": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && groupSubject && sourceSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && sourceSubject && sourceData}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
        `,
      },
      "uses a false source subject branch": {
        [SOURCE_BROWSER_SHELL_PATH]: `
          <script lang="ts">
            import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
            import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
          </script>
          {#if activeTab === "activity" && groupSubject}<SourceGroupActivityView />{/if}
          {#if activeTab === "activity" && false && sourceSubject}<SourceActivityView />{/if}
        `,
        [SOURCE_GROUP_ACTIVITY_PATH]: `
          <script lang="ts">import EmptyState from "$lib/components/ui/EmptyState.svelte";</script>
          <EmptyState />
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
    listFiles: () => Object.keys(sources),
  });
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

const grammersBaseline = {
  schemaVersion: 1,
  revision: "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
  packages: [
    { name: "grammers-client", required: [], forbidden: ["default"], universe: ["default"] },
    { name: "grammers-mtsender", required: [], forbidden: ["proxy"], universe: ["proxy"] },
    { name: "grammers-session", required: ["serde"], forbidden: ["default"], universe: ["default", "serde"] },
    { name: "grammers-tl-types", required: ["default", "deserializable-functions"], forbidden: ["impl-serde"], universe: ["default", "deserializable-functions", "impl-serde"] },
  ],
};

function cargoMetadata() {
  const revision = grammersBaseline.revision;
  const grammers = grammersBaseline.packages.map((entry) => ({
    id: `${entry.name} 0.1.0 (git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision})`,
    name: entry.name,
    source: `git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision}`,
    features: Object.fromEntries(entry.universe.map((feature) => [feature, []])),
    targets: [{ kind: ["lib"], name: entry.name.replaceAll("-", "_") }],
    dependencies: [],
  }));
  const app = {
    id: "path+file:///repo/src-tauri#extractum@0.2.0",
    name: "extractum",
    source: null,
    manifest_path: "C:/repo/src-tauri/Cargo.toml",
    features: {},
    targets: [{ kind: ["lib"], name: "extractum_lib" }],
    dependencies: [
      { name: "extractum-telegram", kind: null, source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: [], uses_default_features: true },
      { name: "extractum-telegram", kind: "dev", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: ["app-test-support"], uses_default_features: true },
    ],
  };
  const normalProducerDependencies = [
    ["base64", [], true],
    ["chacha20poly1305", ["std"], true],
    ["extractum-core", [], true],
    ["grammers-client", [], false],
    ["grammers-mtsender", [], true],
    ["grammers-session", ["serde"], false],
    ["grammers-tl-types", ["deserializable-functions"], true],
    ["rand_core", ["getrandom"], true],
    ["secrecy", [], true],
    ["serde", ["derive"], true],
    ["serde_json", [], true],
    ["tokio", ["rt", "sync", "time"], true],
  ].map(([name, features, usesDefaultFeatures]) => ({
    name,
    kind: null,
    source: String(name).startsWith("grammers-")
      ? `git+https://codeberg.org/Lonami/grammers?rev=${revision}`
      : null,
    path: name === "extractum-core" ? "C:/repo/src-tauri/crates/extractum-core" : null,
    target: null,
    rename: null,
    features,
    uses_default_features: usesDefaultFeatures,
  }));
  const producer = {
    id: "path+file:///repo/src-tauri/crates/extractum-telegram#0.2.0",
    name: "extractum-telegram",
    source: null,
    manifest_path: "C:/repo/src-tauri/crates/extractum-telegram/Cargo.toml",
    features: { "app-test-support": [] },
    targets: [{ kind: ["lib"], name: "extractum_telegram" }],
    dependencies: [
      ...normalProducerDependencies,
      { name: "tokio", kind: "dev", source: null, path: null, target: null, rename: null, features: ["macros", "test-util"], uses_default_features: true },
    ],
  };
  const producerNodeDependencies = normalProducerDependencies.map(({ name }) => {
    const grammersPackage = grammers.find((candidate) => candidate.name === name);
    return {
      name: String(name).replaceAll("-", "_"),
      pkg: grammersPackage?.id ?? `registry+fixture#${name}@1.0.0`,
      dep_kinds: name === "tokio"
        ? [{ kind: null, target: null }, { kind: "dev", target: null }]
        : [{ kind: null, target: null }],
    };
  });
  return {
    packages: [app, producer, ...grammers],
    workspace_members: [app.id, producer.id],
    resolve: {
      nodes: [
        { id: app.id, features: [], deps: [{ name: "extractum_telegram", pkg: producer.id, dep_kinds: [{ kind: null, target: null }, { kind: "dev", target: null }] }] },
        { id: producer.id, features: ["app-test-support"], deps: producerNodeDependencies },
        ...grammers.map((entry) => ({ id: entry.id, features: [...grammersBaseline.packages.find(({ name }) => name === entry.name)!.required], deps: [] })),
      ],
    },
  };
}

function cargoIndex(metadata = cargoMetadata()) {
  return createRepositoryIndex({
    root,
    readFile(absolutePath: string) {
      const relativePath = path.relative(root, absolutePath).replaceAll("\\", "/");
      if (relativePath === GRAMMERS_BASELINE_PATH) return JSON.stringify(grammersBaseline);
      throw new Error(`missing fixture: ${relativePath}`);
    },
    ts,
    svelte,
    loadCargoMetadata: () => metadata,
  });
}

function realAuthorityIndex() {
  return createRepositoryIndex({ root: process.cwd() });
}

const telegramStructuredFixtures = {
  "rule:telegram-phase-8b-authority-integrity": {
    positive: () => realAuthorityIndex(),
    mutations: {
      "changes the generated symbol authority": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getJson(inputPath: string) {
            const value = index.getJson(inputPath);
            return inputPath === SYMBOL_MAP_PATH ? { ...value, schemaVersion: 2 } : value;
          },
        };
      },
      "changes the generated test identity authority": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getJson(inputPath: string) {
            const value = index.getJson(inputPath);
            return inputPath === "src/lib/telegram-8b-test-identities.json"
              ? { ...value, schemaVersion: 2 }
              : value;
          },
        };
      },
      "changes the frozen staging content address": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getText(inputPath: string) {
            const value = index.getText(inputPath);
            return inputPath === "src/lib/telegram-8b-staging-sha256.json" ? `${value} ` : value;
          },
        };
      },
      "changes the retained Checkpoint 8 status pair": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getText(inputPath: string) {
            const value = index.getText(inputPath);
            return inputPath === PHASE_8B_PLAN_PATH
              ? value.replace(
                  "`8B preparation Checkpoint 8 retained`; design",
                  "`8B preparation Checkpoint 7 retained`; design",
                )
              : value;
          },
        };
      },
      "changes the retained roadmap status": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getText(inputPath: string) {
            const value = index.getText(inputPath);
            return inputPath === PHASE_8_ROADMAP_PATH
              ? value.replace("### Phase 8 — `extractum-telegram` (done: retained)", "### Phase 8 — `extractum-telegram` (not retained)")
              : value;
          },
        };
      },
      "changes the retained design status": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getText(inputPath: string) {
            const value = index.getText(inputPath);
            return inputPath === PHASE_8_DESIGN_PATH
              ? value.replace("**Status:** Implemented and retained;", "**Status:** Approved; 8B preparation retained; 8C pending;")
              : value;
          },
        };
      },
    },
  },
  "rule:telegram-crate-manifest-boundary": {
    positive: () => cargoIndex(),
    mutations: {
      "removes the producer library target": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.targets = [];
        return cargoIndex(metadata);
      },
      "enables app test support on the production edge": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies[0].features = ["app-test-support"];
        return cargoIndex(metadata);
      },
      "removes the dev-only feature edge": () => {
        const metadata = clone(cargoMetadata());
        const app = metadata.packages.find(({ name }: any) => name === "extractum")!;
        app.dependencies = app.dependencies.filter(({ name, kind }: any) =>
          name !== "extractum-telegram" || kind !== "dev");
        return cargoIndex(metadata);
      },
      "adds an undeclared producer feature": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.features.debug = [];
        return cargoIndex(metadata);
      },
      "adds an undeclared producer dependency": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.dependencies.push({
          name: "anyhow", kind: null, source: null, path: null, target: null, rename: null, features: [],
        });
        return cargoIndex(metadata);
      },
      "widens the producer Tokio dev features": () => {
        const metadata = clone(cargoMetadata());
        const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
        producer.dependencies.find(({ name, kind }: any) => name === "tokio" && kind === "dev")!.features.push("rt-multi-thread");
        return cargoIndex(metadata);
      },
      "adds a build edge to the producer": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies.push({
          name: "extractum-telegram", kind: "build", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: [],
        });
        return cargoIndex(metadata);
      },
      "drops the dev dependency from the resolved app edge": () => {
        const metadata = clone(cargoMetadata());
        const app = metadata.packages.find(({ name }: any) => name === "extractum")!;
        const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
        metadata.resolve.nodes.find(({ id }: any) => id === app.id)!
          .deps.find(({ pkg }: any) => pkg === producer.id)!.dep_kinds = [{ kind: null, target: null }];
        return cargoIndex(metadata);
      },
      "adds a second workspace feature mention": () => {
        const metadata = clone(cargoMetadata());
        const observer = {
          id: "path+file:///repo/src-tauri/crates/observer#0.1.0",
          name: "observer",
          source: null,
          manifest_path: "C:/repo/src-tauri/crates/observer/Cargo.toml",
          features: {},
          targets: [{ kind: ["lib"], name: "observer" }],
          dependencies: [{ name: "extractum-telegram", kind: "dev", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: ["app-test-support"] }],
        };
        metadata.packages.push(observer);
        metadata.workspace_members.push(observer.id);
        metadata.resolve.nodes.push({ id: observer.id, features: [], deps: [] });
        return cargoIndex(metadata);
      },
    },
  },
  "rule:telegram-crate-dependency-ownership": {
    positive: () => cargoIndex(),
    mutations: {
      "adds a direct app Grammers dependency": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies.push({ name: "grammers-client", kind: null });
        return cargoIndex(metadata);
      },
      "drifts the Grammers source revision": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "grammers-client")!.source = "git+https://codeberg.org/Lonami/grammers?rev=wrong#wrong";
        return cargoIndex(metadata);
      },
      "enables a forbidden Grammers feature": () => {
        const metadata = clone(cargoMetadata());
        const selected = metadata.packages.find(({ name }: any) => name === "grammers-tl-types")!;
        metadata.resolve.nodes.find(({ id }: any) => id === selected.id)!.features.push("impl-serde");
        return cargoIndex(metadata);
      },
      "reorders the generated baseline packages": () => {
        const index = cargoIndex();
        return {
          ...index,
          getJson(inputPath: string) {
            const value = index.getJson(inputPath);
            return inputPath === GRAMMERS_BASELINE_PATH
              ? { ...value, packages: [...value.packages].reverse() }
              : value;
          },
        };
      },
    },
  },
} as const;

function expectSemanticViolations(violations: string[], label: string) {
  expect(violations, label).not.toEqual([]);
  expect(violations.some((violation) => violation.startsWith("INFRA_ERROR:")), label).toBe(false);
}

const analysisRuleIds = [
  "rule:analysis-source-reader-surface-composition",
  "rule:analysis-source-browser-explicit-subject-contract",
  "rule:analysis-evidence-highlight-token-styling",
  "rule:analysis-source-group-tab-leaf-boundary",
  "rule:analysis-source-browser-canonical-composition",
  "rule:analysis-source-group-activity-boundary",
] as const;

describe("analysis source-reader structured rules", () => {
  for (const id of analysisRuleIds) {
    it(`${id} accepts its positive fixture and rejects every mutation`, () => {
      const fixture = ruleFixtures[id];
      expect(evaluateRule({ id, index: indexFor(fixture.positive) }), `${id} positive`).toEqual({
        id,
        violations: [],
      });
      for (const [name, mutation] of Object.entries(fixture.mutations)) {
        expectSemanticViolations(evaluateRule({ id, index: indexFor(mutation) }).violations, `${id}: ${name}`);
      }
    });
  }

  it("accepts the current repository snapshot for all six analysis source-reader rules", () => {
    const index = realAuthorityIndex();
    for (const id of analysisRuleIds) {
      expect(evaluateRule({ id, index }), id).toEqual({ id, violations: [] });
    }
  });
});

describe("extractum-llm public API structured rule", () => {
  it("accepts the current repository snapshot and rejects every closed-surface mutation", () => {
    const id = "rule:extractum-llm-public-api-boundary";
    expect(evaluateRule({ id, index: realAuthorityIndex() })).toEqual({ id, violations: [] });
    const uncaught: string[] = [];
    const infrastructureFailures: string[] = [];
    for (const [name, mutation] of Object.entries(ruleFixtures[id].mutations)) {
      const violations = evaluateRule({ id, index: indexFor(mutation) }).violations;
      if (violations.length === 0) uncaught.push(name);
      if (violations.some((violation) => violation.startsWith("INFRA_ERROR:"))) infrastructureFailures.push(name);
    }
    expect({ uncaught, infrastructureFailures }).toEqual({ uncaught: [], infrastructureFailures: [] });
  });
});

describe("repository rule registry", () => {
  const allowedRuleIds = new Set(
    sourceContractLedger.rows
      .flatMap((row) => "subgroups" in row
        ? row.subgroups.flatMap((subgroup) => subgroup.replacementIds ?? [])
        : row.replacementIds ?? [])
      .filter((id): id is string => id.startsWith("rule:")),
  );

  it("derives and registers the complete truthful ledger rule allowlist", () => {
    expect(allowedRuleIds.size).toBe(11);
    expect(registeredRuleIds).toEqual([
      "rule:analysis-evidence-highlight-token-styling",
      "rule:analysis-source-browser-canonical-composition",
      "rule:analysis-source-browser-explicit-subject-contract",
      "rule:analysis-source-group-activity-boundary",
      "rule:analysis-source-group-tab-leaf-boundary",
      "rule:analysis-source-reader-surface-composition",
      "rule:extractum-grid-wrapper-boundary",
      "rule:extractum-llm-public-api-boundary",
      "rule:telegram-crate-dependency-ownership",
      "rule:telegram-crate-manifest-boundary",
      "rule:telegram-phase-8b-authority-integrity",
    ]);
    for (const id of registeredRuleIds) expect(allowedRuleIds.has(id), id).toBe(true);
  });

  it("gives every registered evaluator its own positive fixture and violating mutation", () => {
    expect([...Object.keys(ruleFixtures), ...Object.keys(telegramStructuredFixtures)].sort()).toEqual(registeredRuleIds);

    for (const id of registeredRuleIds) {
      const fixture = ruleFixtures[id];
      const structured = telegramStructuredFixtures[id as keyof typeof telegramStructuredFixtures];
      const positiveIndex = structured ? structured.positive() : indexFor(fixture.positive);
      expect(evaluateRule({ id, index: positiveIndex }), `${id} positive`).toEqual({
        id,
        violations: [],
      });
      const mutations = structured?.mutations ?? fixture.mutations;
      expect(Object.keys(mutations), `${id} mutations`).not.toEqual([]);
      for (const [name, mutation] of Object.entries(mutations)) {
        const mutationIndex = structured ? mutation() : indexFor(mutation);
        expectSemanticViolations(evaluateRule({ id, index: mutationIndex }).violations, `${id}: ${name}`);
      }
    }
  });

  it("rule:telegram-crate-dependency-ownership rejects a reordered generated baseline", () => {
    const fixture = telegramStructuredFixtures["rule:telegram-crate-dependency-ownership"];
    expect(evaluateRule({
      id: "rule:telegram-crate-dependency-ownership",
      index: fixture.positive(),
    })).toEqual({
      id: "rule:telegram-crate-dependency-ownership",
      violations: [],
    });

    expectSemanticViolations(
      evaluateRule({
        id: "rule:telegram-crate-dependency-ownership",
        index: fixture.mutations["reorders the generated baseline packages"](),
      }).violations,
      "rule:telegram-crate-dependency-ownership: reordered generated baseline",
    );
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
