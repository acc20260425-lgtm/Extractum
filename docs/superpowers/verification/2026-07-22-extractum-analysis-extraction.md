# Extractum Analysis Crate Extraction Verification

## Result and commit identities

**Result: implemented and retained.**

Phase 7 implements the owner-approved
[analysis crate boundary](../specs/2026-07-22-analysis-crate-boundary-design.md)
through the
[execution plan](../plans/2026-07-22-extractum-analysis-extraction.md).
The portable analysis domain and the runtime SQL for its six owned tables now
compile in `extractum-analysis`. Tauri commands and events, spawning, pool and
profile acquisition, migrations, dev fixtures, foreign source/project reads,
and cross-domain transaction coordination remain in the application.

The clean execution start and retained checkpoint chain are:

| Stage | Commit | Subject |
| --- | --- | --- |
| Start | `baf118eb66b6e52b8dc32fd0ec9c6838051288d1` | `docs: harden analysis plan test gates` |
| Checkpoint 1 | `026a3b7660c4fc54a8a47ddb284120e54b0a49b9` | `test: freeze analysis extraction boundary` |
| Checkpoint 2 | `026a55518725d2f2f9dd8275f81f23b33bf8e6d9` | `refactor: prepare analysis value boundary` |
| Checkpoint 3 | `ae9fbef99918ac68d1673da848b5a95c78cce416` | `refactor: introduce analysis corpus boundary` |
| Checkpoint 4 | `2b4559c929ec11bd95bd1896e472f8baf3d8b65b` | `refactor: introduce analysis runtime boundary` |
| Checkpoint 5 | `1a69e56881037589c5da40548f39505d7d46bf38` | `refactor: isolate analysis storage boundary` |
| Intentional RED contract | `5e9bf940549f3d38ac30ae58f61d09b0983910a4` | `test: define analysis crate boundary` |
| Mechanical extraction | `aaf9c1655f5ab180bcec3277609b0b7d88415aef` | `refactor: extract analysis domain crate` |

No `fix: complete analysis crate wiring` commit was required.

Raw history command:

```powershell
git log -12 --pretty=format:'%H%x09%s'
git merge-base --is-ancestor baf118eb66b6e52b8dc32fd0ec9c6838051288d1 aaf9c1655f5ab180bcec3277609b0b7d88415aef
git rev-list --count baf118eb66b6e52b8dc32fd0ec9c6838051288d1..aaf9c1655f5ab180bcec3277609b0b7d88415aef
```

The ancestry check exited 0 and the range contains exactly seven retained
implementation commits.

The extraction commit contains 84 changed paths, all within the exact
145-path Task 7 allowlist. Its cached diff passed `git diff --cached --check`,
and three independent final reviews reported
`0 Critical / 0 Important / 0 Minor`.

## Frozen 54-file disposition

`A` means application-owned, `C` means crate-owned, and `S` means the baseline
mixed file was split during a retained green checkpoint. Every baseline path
has one final disposition:

| # | Baseline path under `src-tauri/src/analysis` | LOC | Final owner/path |
| ---: | --- | ---: | --- |
| 1 | `chat.rs` | 695 | S: app `chat.rs`; crate `chat.rs` |
| 2 | `corpus.rs` | 216 | S: app `corpus.rs`; crate `corpus.rs` |
| 3 | `corpus/live.rs` | 318 | A: `corpus/live.rs` |
| 4 | `corpus/snapshot.rs` | 316 | C: `corpus/snapshot.rs` |
| 5 | `corpus/source_resolution.rs` | 314 | S: app and crate `corpus/source_resolution.rs` |
| 6 | `corpus/tests/harness.rs` | 548 | S: app and crate `corpus/tests/harness.rs` |
| 7 | `corpus/tests/live.rs` | 683 | S: app and crate `corpus/tests/live.rs` |
| 8 | `corpus/tests/mod.rs` | 5 | S: app and crate `corpus/tests/mod.rs` |
| 9 | `corpus/tests/preflight.rs` | 257 | S: app and crate `corpus/tests/preflight.rs` |
| 10 | `corpus/tests/snapshot.rs` | 484 | C: `corpus/tests/snapshot.rs` |
| 11 | `corpus/tests/source_resolution.rs` | 234 | S: app and crate `corpus/tests/source_resolution.rs` |
| 12 | `events.rs` | 12 | A: `events.rs` |
| 13 | `fixtures.rs` | 374 | A: `fixtures.rs` |
| 14 | `fixtures/seed.rs` | 611 | A: `fixtures/seed.rs` |
| 15 | `fixtures/seed/runs.rs` | 407 | A: `fixtures/seed/runs.rs` |
| 16 | `fixtures/tests/active_runs.rs` | 75 | A: `fixtures/tests/active_runs.rs` |
| 17 | `fixtures/tests/clear.rs` | 321 | A: `fixtures/tests/clear.rs` |
| 18 | `fixtures/tests/harness.rs` | 54 | A: `fixtures/tests/harness.rs` |
| 19 | `fixtures/tests/mod.rs` | 6 | A: `fixtures/tests/mod.rs` |
| 20 | `fixtures/tests/seed.rs` | 366 | A: `fixtures/tests/seed.rs` |
| 21 | `fixtures/tests/snapshot.rs` | 226 | A: `fixtures/tests/snapshot.rs` |
| 22 | `fixtures/tests/summary.rs` | 25 | A: `fixtures/tests/summary.rs` |
| 23 | `groups.rs` | 315 | S: app `groups.rs`; crate `groups.rs` |
| 24 | `mod.rs` | 552 | S: app `mod.rs`; crate `domain.rs` and `tests.rs` |
| 25 | `models.rs` | 300 | C: `models.rs` |
| 26 | `report.rs` | 552 | S: app `report.rs`; crate `report.rs` |
| 27 | `report/capture.rs` | 37 | C: `report/capture.rs` |
| 28 | `report/lifecycle.rs` | 135 | S: app and crate `report/lifecycle.rs` |
| 29 | `report/phases.rs` | 389 | C: `report/phases.rs` |
| 30 | `report/requests.rs` | 266 | C: `report/requests.rs` |
| 31 | `report/tests/architecture.rs` | 10 | C: `report/tests/architecture.rs` |
| 32 | `report/tests/capture.rs` | 102 | A: `report/tests/capture.rs` |
| 33 | `report/tests/harness.rs` | 135 | C: `report/tests/harness.rs` |
| 34 | `report/tests/lifecycle.rs` | 99 | C: `report/tests/lifecycle.rs` |
| 35 | `report/tests/mod.rs` | 8 | S: app and crate `report/tests/mod.rs` |
| 36 | `report/tests/phases.rs` | 69 | C: `report/tests/phases.rs` |
| 37 | `report/tests/preflight.rs` | 48 | C: `report/tests/preflight.rs` |
| 38 | `report/tests/requests.rs` | 81 | C: `report/tests/requests.rs` |
| 39 | `report/tests/scope.rs` | 89 | C: `report/tests/scope.rs` |
| 40 | `report_commands.rs` | 56 | A: `report_commands.rs` |
| 41 | `state.rs` | 94 | C: `state.rs` |
| 42 | `store.rs` | 24 | S: app `store.rs`; crate `store.rs` |
| 43 | `store/read_model.rs` | 456 | S: app and crate `store/read_model.rs` |
| 44 | `store/runs.rs` | 208 | C: `store/runs.rs` |
| 45 | `store/setup.rs` | 144 | S: app and crate `store/setup.rs` |
| 46 | `store/snapshot.rs` | 297 | C: `store/snapshot.rs` |
| 47 | `store/tests/harness.rs` | 2 | C: `store/tests/harness.rs` |
| 48 | `store/tests/mod.rs` | 5 | S: app and crate `store/tests/mod.rs` |
| 49 | `store/tests/read_model.rs` | 708 | S: app and crate `store/tests/read_model.rs` |
| 50 | `store/tests/runs.rs` | 532 | C: `store/tests/runs.rs` |
| 51 | `store/tests/setup.rs` | 62 | S: app and crate `store/tests/setup.rs` |
| 52 | `store/tests/snapshot.rs` | 208 | C: `store/tests/snapshot.rs` |
| 53 | `templates.rs` | 200 | S: app `templates.rs`; crate `templates.rs` |
| 54 | `trace.rs` | 457 | C: `trace.rs` |

The baseline reconciles to 14 whole app files, 20 split files, 20 whole crate
files, and 13,187 physical lines. Baseline `mod.rs` contributes crate
`domain.rs` and `tests.rs` exactly once.

Four preparation-added paths sit outside that baseline map:

- app `tests_application.rs`;
- crate `report/tests/corpus_port.rs`;
- crate `report/tests/runtime.rs`;
- crate `test_schema.rs`.

Generated crate root `lib.rs` is counted separately.

The retained Checkpoint 5 topology used `#[path]` and plain `mod` declarations,
not the plan's stale prose describing 19 temporary `include!` seams. Exactly
three staging files were intentionally unreachable before the move:
`corpus/tests/harness_portable.rs`, `corpus/tests/mod_portable.rs`, and
`store/tests/mod_portable.rs`. They are ordinary reachable crate files after
the extraction. No staging filename, compatibility include, or production
copy remains.

## Final physical topology

The physical-line command counts blank lines:

```powershell
$finalAppFiles = Get-ChildItem -LiteralPath src-tauri/src/analysis -Recurse -File -Filter '*.rs'
$finalAppLines = ($finalAppFiles | ForEach-Object { @(Get-Content -LiteralPath $_.FullName).Count } | Measure-Object -Sum).Sum
$finalCrateFiles = Get-ChildItem -LiteralPath src-tauri/crates/extractum-analysis/src -Recurse -File -Filter '*.rs'
$finalCrateLines = ($finalCrateFiles | ForEach-Object { @(Get-Content -LiteralPath $_.FullName).Count } | Measure-Object -Sum).Sum
```

Outcome:

```text
app_files=35 app_lines=7230 crate_files=45 crate_lines=11336
```

Application files (35):

```text
chat.rs
corpus.rs
corpus/live.rs
corpus/source_resolution.rs
corpus/tests/harness.rs
corpus/tests/live.rs
corpus/tests/mod.rs
corpus/tests/preflight.rs
corpus/tests/source_resolution.rs
events.rs
fixtures.rs
fixtures/seed.rs
fixtures/seed/runs.rs
fixtures/tests/active_runs.rs
fixtures/tests/clear.rs
fixtures/tests/harness.rs
fixtures/tests/mod.rs
fixtures/tests/seed.rs
fixtures/tests/snapshot.rs
fixtures/tests/summary.rs
groups.rs
mod.rs
report.rs
report/lifecycle.rs
report/tests/capture.rs
report/tests/mod.rs
report_commands.rs
store.rs
store/read_model.rs
store/setup.rs
store/tests/mod.rs
store/tests/read_model.rs
store/tests/setup.rs
templates.rs
tests_application.rs
```

Crate files (45):

```text
chat.rs
corpus.rs
corpus/snapshot.rs
corpus/source_resolution.rs
corpus/tests/harness.rs
corpus/tests/live.rs
corpus/tests/mod.rs
corpus/tests/preflight.rs
corpus/tests/snapshot.rs
corpus/tests/source_resolution.rs
domain.rs
groups.rs
lib.rs
models.rs
report.rs
report/capture.rs
report/lifecycle.rs
report/phases.rs
report/requests.rs
report/tests/architecture.rs
report/tests/corpus_port.rs
report/tests/harness.rs
report/tests/lifecycle.rs
report/tests/mod.rs
report/tests/phases.rs
report/tests/preflight.rs
report/tests/requests.rs
report/tests/runtime.rs
report/tests/scope.rs
state.rs
store.rs
store/read_model.rs
store/runs.rs
store/setup.rs
store/snapshot.rs
store/tests/harness.rs
store/tests/mod.rs
store/tests/read_model.rs
store/tests/runs.rs
store/tests/setup.rs
store/tests/snapshot.rs
templates.rs
test_schema.rs
tests.rs
trace.rs
```

## Frozen identities and added tests

The boundary contract parses Appendix A fail-closed and proves exactly 143
unique, reachable, enabled identities:

- `extractum-analysis`: 95 frozen identities;
- `extractum`: 48 frozen `analysis::` identities.

New tests are counted separately and do not substitute for the baseline:

- `extractum-analysis`: 112 total = 95 frozen + 17 added;
- app analysis modules: 57 total = 48 frozen + 9 added.

The 17 added crate identities are:

```text
chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message
chat::tests::chat_execution_persists_turns_before_completed_event
report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads
report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure
report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure
report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails
report::tests::runtime::report_execution_publishes_typed_events_in_existing_order
report::tests::runtime::terminal_cleanup_always_removes_active_report_state
report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes
report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities
report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label
store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes
test_schema::tests::canonical_fixture_applies_analysis_consumed_schema
test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys
trace::tests::legacy_trace_bytes_decode_after_core_compression_handoff
trace::tests::decode_trace_data_returns_typed_internal_for_invalid_json
trace::tests::trace_ref_json_is_byte_compatible_for_telegram_and_youtube
```

The nine added app identities are:

```text
analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion
analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence
analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels
analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit
analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot
analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects
analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id
analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads
analysis::tests_application::report_profile_resolution_failure_prevents_run_creation
```

Fresh list commands and outcomes:

```powershell
cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum-analysis --lib -- --list
# 112 tests, 0 benchmarks

cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib 'analysis::' -- --list
# 57 tests, 0 benchmarks
```

## Command and IPC inventory

All 21 release analysis commands remain app-owned:

```text
list_analysis_sources
list_analysis_runs
list_active_analysis_runs
get_analysis_run
list_analysis_run_messages
get_analysis_run_trace
delete_analysis_run
resolve_analysis_trace_refs
list_analysis_prompt_templates
create_analysis_prompt_template
update_analysis_prompt_template
delete_analysis_prompt_template
list_analysis_source_groups
create_analysis_source_group
update_analysis_source_group
delete_analysis_source_group
list_analysis_chat_messages
clear_analysis_chat_messages
ask_analysis_run_question
start_analysis_report
cancel_analysis_run
```

The three project commands are `start_project_analysis`,
`list_project_runs`, and `get_project_data_range`. The three dev commands are
`seed_analysis_redesign_fixtures`, `clear_analysis_redesign_fixtures`, and
`clear_analysis_redesign_fixture_active_runs`.

The contracts prove the exact 21 + 3 + 3 inventory, Tauri registration,
camelCase parameters, return shapes, request IDs, event channels and order,
and `AppError` JSON. No persisted or wire vocabulary changed, so
`docs/value-registry.md` was not modified.

## Curated crate API and visibility

`src-tauri/crates/extractum-analysis/src/lib.rs` contains only private `mod`
declarations and explicit `pub use` statements. It has no `pub mod`, glob
re-export, public internal SQL row, secret getter, or public test support.

The exact public root type allowlist is:

```text
AnalysisSourceOption
AnalysisPromptTemplate
AnalysisSourceGroupMember
AnalysisSourceGroup
AnalysisTraceRef
AnalysisTraceData
AnalysisSnapshotState
AnalysisRunSummary
AnalysisRunDetail
AnalysisRunMessageCursor
AnalysisRunMessage
AnalysisRunMessagesPage
AnalysisRunEvent
AnalysisChunkSummaryEvent
AnalysisChatEvent
AnalysisChatTurn
AnalysisChatMessage
AnalysisScopeKind
AnalysisSourceKind
ResolvedAnalysisScope
YoutubeCorpusMode
AnalysisCorpusRequest
AnalysisCorpusMessage
AnalysisRunPreflightLimits
AnalysisRunPreflight
AnalysisPortFuture
AnalysisCorpusReader
AnalysisEventSink
StartAnalysisReportRequest
AnalysisRunListFilters
AskAnalysisRunQuestionRequest
AnalysisReportPreparationTicket
AnalysisReportScopeTicket
AnalysisReportExecutionTicket
AnalysisChatExecutionTicket
AnalysisChatCompletionTicket
AnalysisExecutionError
AnalysisState
AnalysisReportCancellationWait
AnalysisForeignLabelMatch
AnalysisSourceLabel
AnalysisProjectLabel
AnalysisForeignLabels
AnalysisForeignLabelRef
AnalysisRunSummaryEnrichment
AnalysisRunDetailEnrichment
AnalysisChatRunEnrichment
AnalysisChatRun
AnalysisSourceGroupInput
AnalysisSourceGroupRecord
ProjectAnalysisRunAggregate
AnalysisRunDiagnosticCount
```

The exact public root function allowlist is:

```text
prepare_analysis_report
prepare_analysis_report_execution
preflight_analysis_corpus
capture_analysis_corpus
execute_analysis_report
finalize_analysis_report_execution
prepare_analysis_chat
execute_analysis_chat
complete_analysis_chat
publish_analysis_chat_execution_error
publish_analysis_chat_persistence_error
prepare_analysis_run_summaries
prepare_active_analysis_run_summaries
prepare_analysis_run_detail
prepare_legacy_analysis_chat_run
load_analysis_source_groups_for_enrichment
load_analysis_source_group_for_enrichment
load_project_analysis_run_aggregates
delete_project_analysis_runs
list_analysis_prompt_templates
create_analysis_prompt_template
update_analysis_prompt_template
delete_analysis_prompt_template
create_analysis_source_group
update_analysis_source_group
delete_analysis_source_group
get_analysis_source_group_record
list_analysis_run_messages
get_analysis_run_trace
delete_analysis_run
resolve_analysis_trace_refs
list_analysis_chat_messages
clear_analysis_chat_messages
load_analysis_chat_run
analysis_run_ids_depending_on_sources
load_analysis_run_diagnostics
mark_interrupted_analysis_runs
request_analysis_run_cancel
resolve_analysis_telegram_history_scope
```

Opaque fields and exact constructors/accessors are enforced by
`keeps a curated crate API and exhaustive visibility allowlist`.

## SQL ownership and transaction families

Production crate SQL may name exactly six tables:

```text
analysis_runs
analysis_run_messages
analysis_chat_messages
analysis_prompt_templates
analysis_source_groups
analysis_source_group_members
```

The foreign denylist covers `analysis_documents`, `sources`, `items`,
`telegram_messages`, every YouTube table, `projects`, `project_sources`,
prompt-pack/provider/settings/account tables, and every table outside the
six-item allowlist. The exact current foreign migration-table inventory is:

```text
accounts
analysis_documents
app_settings
archive_read_items
archive_read_model_state
ingest_batch_warnings
ingest_batches
ingest_item_observations
item_topic_memberships
items
jobs
project_sources
projects
prompt_pack_audit_events
prompt_pack_result_audit_refs
prompt_pack_result_claims
prompt_pack_result_evidence
prompt_pack_result_limitations
prompt_pack_result_quality_flags
prompt_pack_result_quarantine_artifacts
prompt_pack_result_ref_edges
prompt_pack_result_source_refs
prompt_pack_result_unknowns
prompt_pack_result_validation_findings
prompt_pack_result_verification_tasks
prompt_pack_result_warnings
prompt_pack_results
prompt_pack_run_material_snapshots
prompt_pack_run_scopes
prompt_pack_run_source_origins
prompt_pack_run_source_snapshots
prompt_pack_runs
prompt_pack_schema_assets
prompt_pack_stage_artifacts
prompt_pack_stage_runs
prompt_pack_stage_templates
prompt_pack_versions
prompt_pack_youtube_action_items
prompt_pack_youtube_key_points
prompt_pack_youtube_open_questions
prompt_pack_youtube_quotes
prompt_pack_youtube_segments
prompt_pack_youtube_synthesis_items
prompt_pack_youtube_videos
prompt_packs
source_identity_repair_notes
sources
telegram_forum_topics
telegram_messages
telegram_migrated_history_capabilities
telegram_sources
telegram_takeout_batches
telegram_topic_resolution_state
workers
youtube_playlist_items
youtube_playlist_sources
youtube_transcript_segments
youtube_video_sources
```

Migrations and `analysis_documents` remain app-owned.

The only public borrowed-connection participants and their nine coordinator
call chains are:

| Family | Participant; first argument is `&mut SqliteConnection` | App coordinator(s) owning transaction lifecycle |
| --- | --- | --- |
| 1 | `prepare_analysis_run_summaries` | `list_analysis_runs_in_pool` |
| 1 | `prepare_active_analysis_run_summaries` | `list_active_analysis_runs_in_pool` |
| 1 | `prepare_analysis_run_detail` | `get_analysis_run_in_pool` |
| 1 | `prepare_legacy_analysis_chat_run` | legacy branch of `resolve_legacy_analysis_chat_run_in_pool` |
| 2 | `load_analysis_source_groups_for_enrichment` | `list_analysis_source_groups_in_pool` |
| 2 | `load_analysis_source_group_for_enrichment` | `get_analysis_source_group_response_in_pool`; `load_export_source_group_in_pool` |
| 3 | `load_project_analysis_run_aggregates` | `projects::read_model::list_research_projects_in_pool` |
| 4 | `delete_project_analysis_runs` | `projects::delete_project_in_pool` |

Their exact parameter/result shapes are:

```rust
pub async fn prepare_analysis_run_summaries(
    conn: &mut SqliteConnection,
    filters: AnalysisRunListFilters,
    matches: Vec<AnalysisForeignLabelMatch>,
) -> AppResult<AnalysisRunSummaryEnrichment>;

pub async fn prepare_active_analysis_run_summaries(
    conn: &mut SqliteConnection,
    run_ids: &HashSet<i64>,
) -> AppResult<AnalysisRunSummaryEnrichment>;

pub async fn prepare_analysis_run_detail(
    conn: &mut SqliteConnection,
    run_id: i64,
) -> AppResult<AnalysisRunDetailEnrichment>;

pub async fn prepare_legacy_analysis_chat_run(
    conn: &mut SqliteConnection,
    run_id: i64,
) -> AppResult<AnalysisChatRunEnrichment>;

pub async fn load_analysis_source_groups_for_enrichment(
    conn: &mut SqliteConnection,
) -> AppResult<Vec<AnalysisSourceGroupRecord>>;

pub async fn load_analysis_source_group_for_enrichment(
    conn: &mut SqliteConnection,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroupRecord>>;

pub async fn load_project_analysis_run_aggregates(
    conn: &mut SqliteConnection,
    project_ids: &[i64],
) -> AppResult<Vec<ProjectAnalysisRunAggregate>>;

pub async fn delete_project_analysis_runs(
    conn: &mut SqliteConnection,
    project_id: i64,
) -> AppResult<()>;
```

All eight participants borrow the connection first, take no pool, and contain
no acquire/begin/commit/rollback. Each app coordinator owns one explicit
transaction and threads the same `&mut *transaction` through every SQL step.

The nine coordinator call chains are pinned as:

```text
list_analysis_runs_in_pool:
  begin -> match foreign labels -> prepare summaries -> load labels -> finish -> commit
list_active_analysis_runs_in_pool:
  begin -> prepare active summaries -> load labels -> finish -> commit
get_analysis_run_in_pool:
  begin -> prepare detail -> load labels -> finish -> commit
resolve_legacy_analysis_chat_run_in_pool:
  pool read -> only blank/null legacy branch -> begin -> re-read projection
  -> load labels -> finish -> commit
list_analysis_source_groups_in_pool:
  begin -> load group records -> enrich records -> commit
get_analysis_source_group_response_in_pool:
  begin -> load one group record -> enrich -> commit
load_export_source_group_in_pool:
  begin -> load one group record -> load foreign source rows -> commit
projects::read_model::list_research_projects_in_pool:
  begin -> load projects -> load analysis aggregates -> compose -> commit
projects::delete_project_in_pool:
  begin -> delete analysis runs -> delete project_sources -> delete project -> commit
```

Family 1 remains closed to list, active, get, and the legacy blank/null chat
label fallback. `get_analysis_run_trace`, `delete_analysis_run`,
`list_analysis_chat_messages`, `clear_analysis_chat_messages`, lifecycle-only
reads, and trace/ref reads remain ordinary pool calls.

The fail-closed parsed ownership scan covered production and test SQL,
QueryBuilder fragments, aliases, quoted/schema-qualified identifiers,
transaction control, cfg reachability, and the three private foreign-row
negative sentinels. It passed with no production exception.

## Runtime and compatibility evidence

The retained behavior contracts prove:

- report preflight read A and capture read B are independent; A supplies the
  started-event summary and B supplies captured/executed content;
- report profile resolution fails before run creation, while chat profile
  failure remains asynchronous after request-ID acceptance;
- report and chat use one detached spawn each and one managed scheduler state;
- the app event adapter is synchronous, performs exactly one emit, and has no
  database, retry, blocking, or task-spawn side effect;
- report events preserve the complete typed sequence and fields;
- chat turns persist before the completed event;
- cancellation uses opaque lifecycle capabilities and terminal cleanup always
  removes active state, including persistence-failure paths;
- trace bytes and JSON remain compatible after compression moved to
  `extractum-core`;
- request IDs, profile snapshots, labels, filters, ordering, serialized values,
  and error JSON remain unchanged;
- startup interruption cleanup, account deletion, diagnostics, NotebookLM,
  project reads/deletion, and all named cross-domain consumers remain wired.

All 27 Task 7 exact Rust witnesses ran separately and non-empty before the
mechanical commit. The post-commit boundary/application/migration/safety suite
then passed 87/87, and the broader 11-file contract suite passed 134/134.

## Manifest, lockfile, movement, and fixture proof

The workspace has one member `crates/extractum-analysis` and the app has one
path dependency:

```toml
extractum-analysis = { path = "crates/extractum-analysis" }
```

The crate's seven direct dependency roots are:

```text
extractum-core
extractum-llm
serde
serde_json
sqlx
tokio
tokio-util
```

The only dev-dependency declaration is the same workspace `tokio` dependency
with the additional `time` feature; it adds no separate lockfile root.

The app no longer has a direct `zstd` dependency or direct `zstd::` use.
Lower crates have no reverse `extractum-analysis` edge. `Cargo.lock` has one
source-less/checksum-less `extractum-analysis` package with those exact roots,
and `cargo metadata --locked` succeeds.

All 44 mechanical sources were moved, not copied: 23 whole moves and 21
prepared split moves. The final crate topology has no stale portable filename.
The private `test_schema` is the sole analysis migration-fixture owner, embeds
the canonical ordered 12-migration non-Apalis prefix through the final
`../../../migrations/` root, and stays in parity with the app registry and the
independent prompt-pack migration contract.

## Completion gates

Fresh post-commit commands and outcomes:

```powershell
npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts src/lib/analysis-application-contract.test.ts src/lib/analysis-migration-fixture-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts
# 4 files / 87 tests passed

cargo check --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets
# exit 0

cargo test --manifest-path src-tauri/Cargo.toml -p extractum-analysis --all-targets
# 112 passed

cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
# exit 0

cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
# 701 passed

npm.cmd run check:rustfmt
# exit 0

cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
# exit 0

cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
# exit 0

npm.cmd run verify
# 176 Vitest files / 1476 tests passed
# svelte-check: 0 errors / 0 warnings
# rustfmt, workspace check, and workspace tests: exit 0

cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
# exit 0
```

Only pre-existing `dead_code`/unused warnings remain; no warning was widened
into this slice.

## Single advisory timing observation

The only Phase 7 timing observation was the ordinary mandatory workspace
check. `%TEMP%\extractum-phase7-analysis-workspace-check.txt` contains:

```text
phase=7
command=cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
milliseconds=5162
```

Phase 6 recorded 11,669 ms. Both are below 15,000 ms, so the
two-adjacent-slices investigation rule does not trigger. No focused probe,
warm-up, sample series, alternate target, retry for timing, A/B harness, or
retention threshold was introduced. The observation was advisory and did not
decide retention.

## Release build and exact-PID startup

Required build command:

```powershell
npm.cmd run tauri -- build --no-bundle
```

Tauri completed the release profile and reported:

```text
Finished `release` profile [optimized] target(s) in 5m 33s
Built application at: G:\Develop\Extractum\src-tauri\target\release\extractum.exe
```

No installer was built. Before startup there was no existing `extractum`
process. The bounded smoke launched exactly the built executable, observed it
for five seconds, inspected the exact PID path, and cleaned up only the owned
PID:

```powershell
$releaseExe = (Resolve-Path -LiteralPath 'src-tauri/target/release/extractum.exe').Path
$existing = @(Get-Process -Name extractum -ErrorAction SilentlyContinue)
if ($existing.Count -ne 0) {
    throw '[infrastructure] pre-existing extractum process prevents exact ownership proof'
}

$ownedProcess = $null
$startupFailure = $null
$startupEvidence = $null
$cleanupEvidence = $null
try {
    try {
        $ownedProcess = Start-Process -FilePath $releaseExe -PassThru -WindowStyle Hidden
    } catch {
        throw "[infrastructure] release launch failed: $($_.Exception.Message)"
    }
    if ($null -eq $ownedProcess) {
        throw '[infrastructure] launch returned no process'
    }

    Start-Sleep -Seconds 5
    try {
        $ownedProcess.Refresh()
    } catch {
        throw "[infrastructure] exact PID refresh failed: $($_.Exception.Message)"
    }
    if ($ownedProcess.HasExited) {
        throw "[completion] release process exited during the 5-second startup window with code $($ownedProcess.ExitCode)"
    }

    try {
        $ownedPath = (Get-Process -Id $ownedProcess.Id -ErrorAction Stop).Path
    } catch {
        throw "[infrastructure] exact PID path inspection failed: $($_.Exception.Message)"
    }
    if ([System.IO.Path]::GetFullPath($ownedPath) -ne [System.IO.Path]::GetFullPath($releaseExe)) {
        throw '[infrastructure] exact PID path does not match the built executable'
    }
    $startupEvidence =
        "startup_pid=$($ownedProcess.Id) path=$ownedPath observed_seconds=5"
} catch {
    $startupFailure = $_
} finally {
    if ($null -ne $ownedProcess) {
        try {
            $ownedProcess.Refresh()
            if (-not $ownedProcess.HasExited) {
                Stop-Process -Id $ownedProcess.Id -Force -ErrorAction Stop
            }
            $deadline = [DateTime]::UtcNow.AddSeconds(10)
            while (
                (Get-Process -Id $ownedProcess.Id -ErrorAction SilentlyContinue) -and
                [DateTime]::UtcNow -lt $deadline
            ) {
                Start-Sleep -Milliseconds 100
            }
            if (Get-Process -Id $ownedProcess.Id -ErrorAction SilentlyContinue) {
                throw 'owned PID did not exit within cleanup bound'
            }
            $cleanupEvidence =
                "cleanup_pid=$($ownedProcess.Id) exited=true bound_seconds=10"
        } catch {
            if ($null -eq $startupFailure) {
                $startupFailure = [System.Management.Automation.RuntimeException]::new(
                    "[infrastructure] exact PID cleanup failed: $($_.Exception.Message)"
                )
            }
        }
    }

    $residue = @(
        Get-Process -Name extractum -ErrorAction SilentlyContinue |
            Where-Object {
                try {
                    [System.IO.Path]::GetFullPath($_.Path) -eq
                        [System.IO.Path]::GetFullPath($releaseExe)
                } catch {
                    $false
                }
            }
    )
    if ($residue.Count -ne 0 -and $null -eq $startupFailure) {
        $startupFailure = [System.Management.Automation.RuntimeException]::new(
            '[infrastructure] matching release process residue remains'
        )
    }
}
if ($null -ne $startupFailure) { throw $startupFailure }
$startupEvidence
$cleanupEvidence
'matching_release_residue=0'
```

Outcome:

```text
startup_pid=16316 path=G:\Develop\Extractum\src-tauri\target\release\extractum.exe observed_seconds=5
cleanup_pid=16316 exited=true bound_seconds=10
matching_release_residue=0
```

The owner did not request optional credential-free live MCP plus self-managed
analysis smoke, so no provider request, account mutation, or extra
process-control harness was used.

## Retention disposition

Phase 7 is retained because the exact owner split, frozen identities, curated
API, SQL boundary, transaction families, behavior contracts, package gates,
workspace/repository verification, release build, and bounded startup proof
all passed. Timing is recorded only as advisory context.

Phase 8 remains unauthorized. It requires a fresh design, implementation plan,
and explicit owner instruction.
