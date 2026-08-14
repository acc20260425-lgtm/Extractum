# Rust Infrastructure Wave 0 Verification

**Baseline date:** 2026-08-12

**Final local verification:** 2026-08-14

**Platform:** Windows `x86_64-pc-windows-msvc`

**Local deterministic status:** PASS

**Advisory status:** FAIL on three recorded RustSec advisories

**Release status:** BLOCKED by the advisory result; no bundle acceptance is claimed

## Scope and commit history

Wave 0 began after `77c739a3` (`docs: avoid duplicate Rust fast CI runs`). The
reviewed Task 1–7 implementation range is `77c739a3..83452554`; this record and
the canonical documentation changes are the following documentation commit.

| Task | Commits | Evidence summary |
| --- | --- | --- |
| Fresh-checkout correction | `afae0b76` | Corrected bootstrap ordering after proving that a checkout without `.svelte-kit/tsconfig.json` needs `svelte-kit sync`. The authoritative preflight `verify` passed in 990.6s. |
| 1: toolchain/MSRV | `ea28d86e` | Locked metadata and focused `extractum-core` feature-off and `extractum` all-target checks passed. All seven packages reported Edition 2021 and MSRV 1.95; `Cargo.lock` was unchanged. |
| 2: verifier contract | `3287e851` | RED proved two Cargo steps remained; GREEN was 2/2 focused verifier tests plus rustfmt. `verify` now owns one locked workspace/all-target test and bootstrap owns Svelte sync. |
| 3: Clippy baseline | scope corrections `97a66c42`, `e3dc434b`, `fae6526a`; implementation `432dd4a6`; canonical-helper fix `512db24c` | Three discovery stages were classified and resolved in the authorized 55-file final source scope. Package checks/tests, two feature-off checks, fail-fast workspace Clippy, and authoritative full verification passed. |
| 4: supply chain | acceptance correction `ceb9676c`; implementation `8f0a0e71`; bootstrap hardening `ebcb6af1` | Pinned/checksummed cargo-deny bootstrap passed transactional failure harnesses. Deterministic `bans licenses sources` and the full fast gate passed; three advisories remained explicit release blockers without exceptions. |
| 5: duplicate baseline | `27db4d57`, `859d4f4e`; scope clarification `575e7271` | Active Windows graph measured 32 duplicate names / 80 duplicate-version instances. Parser/rule tests, 828/828 full unit tests, and Svelte check passed; same-aggregate package replacement is a non-blocking review signal. |
| 6: executable policy | `0877ebbe` | Focused policy/convention tests passed 21/21, full unit passed 828/828, and Svelte check passed. `Cargo.lock` was unchanged. |
| 7: CI executors | `aa664ff3`, `83452554` | Workflow contracts passed 7/7, full unit passed 835/835, and Svelte check passed. Fast/full permissions are read-only; advisory follow-up is serialized; release depends on advisory success. |

The final metadata check on 2026-08-14 reported all seven packages at Edition
2021 and `rust-version = 1.95`. `git diff 77c739a3..83452554 --
src-tauri/Cargo.lock` is empty. Edition was not migrated and dependency versions
were not intentionally upgraded in Wave 0.

## Toolchain and pinned tool

```text
rustc 1.95.0 (59807616e 2026-04-14)
commit-hash: 59807616e1fa2540724bfbac14d7976d7e4a3860
host: x86_64-pc-windows-msvc
LLVM version: 22.1.2
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
cargo-deny 0.20.2
```

The root `rust-toolchain.toml` is the canonical pin. All package manifests
inherit workspace MSRV 1.95. The cargo-deny Windows release asset is pinned in
`.github/tools/cargo-deny.json` with SHA-256
`975a22143262fd27476d19ee00c7af67978426e40e1dee94eed6bbade1cf87dc`.
The installed verified `cargo-deny.exe` used for final local evidence had
SHA-256 `f7292fab58c706638c999e64c4ba82e5128ae628130ba55e3266a768ee431fbf`.

## Clippy discovery and resolution evidence

The normal gate deliberately omits `--all-features`. Producer feature-off
compilation remains separate from feature-on consumer/all-target evidence.

### Stage 1: initial workspace frontier

Command:

```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked --keep-going --message-format=short -- -D warnings
```

Result: exit 1 with exactly eight unique source findings:

1. `extractum-gemini-browser/src/execution.rs:42` — `large_enum_variant`, public `CancelRunOutcome`.
2. `extractum-gemini-browser/src/execution.rs:53` — `large_enum_variant`, private `ExecutionSelection`.
3. `extractum-gemini-browser/src/types.rs:256` — `large_enum_variant`, public sidecar response wire type.
4. `extractum-llm/src/lib.rs:49` — `needless_maybe_sized`, test-only deserialize API probe.
5. `extractum-llm/src/scheduler.rs:322` — `new_without_default`, public `LlmSchedulerState`.
6. `extractum-telegram/src/live/peer.rs:9` — `large_enum_variant`, private `ListedPeer`.
7. `extractum-telegram/src/runtime.rs:209` — `new_without_default`, public `TelegramRuntime`.
8. `extractum-telegram/src/runtime.rs:484` — `items_after_test_module`, test-only item ordering.

The enum payloads were boxed at their ownership boundaries, defaults delegate
to the existing constructors, the ineffective `?Sized` bounds were removed,
and test-only impls moved before the test module. The sidecar round-trip test
proved the JSON wire shape remained unchanged.

### Stage 2: analysis and prompt-pack frontier

The same command was run again after Stage 1. It returned 15 target diagnostics
at 14 unique source locations; `snapshots.rs:335` was emitted for both library
and library-test targets.

| # | Package and location | Lint | Owning item | Resolution |
| --- | --- | --- | --- | --- |
| 1 | `extractum-analysis/src/report.rs:355` | `too_many_arguments` | `prepare_analysis_report_execution` | Owning-function allow with public API ownership comment. |
| 2 | `extractum-analysis/src/state.rs:22` | `new_without_default` | `AnalysisState::new` | `Default` delegates to `new`. |
| 3 | `extractum-analysis/src/store/read_model.rs:53` | `too_many_arguments` | `resolve_run_scope_label_parts` | Owning-function allow with helper ownership comment. |
| 4 | `extractum-analysis/src/report/tests/corpus_port.rs:136` | `vec_init_then_push` (then `useless_vec`) | event setup | Direct array initialization. |
| 5 | `extractum-analysis/src/tests.rs:64` | `explicit_auto_deref` | test detail preparation | Direct mutable reference. |
| 6 | `extractum-prompt-packs/src/dto.rs:10` | `derivable_impls` | `PromptPackRuntimeProvider` | Derived `Default`; existing `Api` is `#[default]`; serialization test retained. |
| 7 | `extractum-prompt-packs/src/youtube_summary/gem_analysis.rs:644` | `needless_return` | transcript-stage match arm | Removed explicit return. |
| 8 | `extractum-prompt-packs/src/youtube_summary/result_validation.rs:622` | `needless_lifetimes` | `synthesis_item_arrays` | Elided lifetime. |
| 9 | `extractum-prompt-packs/src/youtube_summary/snapshots.rs:292` | `needless_borrow` | `insert_material` call | Passed the existing reference directly. |
| 10 | `extractum-prompt-packs/src/youtube_summary/snapshots.rs:335` | `too_many_arguments` | `insert_material`, lib | Owning-function allow with ownership comment. |
| 11 | `extractum-prompt-packs/src/youtube_summary/synthesis_input.rs:87` | `redundant_closure` | merged-graph fallback | Passed function directly. |
| 12 | `extractum-prompt-packs/src/result_builder.rs:1114` | `too_many_arguments` | test helper | Owning-function allow with test ownership comment. |
| 13 | `extractum-prompt-packs/src/runtime.rs:866` | `needless_maybe_sized` | test-only API probe | Removed ineffective bound. |
| 14 | `extractum-prompt-packs/src/lib.rs:152` | `assertions_on_constants` | feature-off surface probe | Removed only tautological assertion; retained probe. |
| 15 | `extractum-prompt-packs/src/youtube_summary/snapshots.rs:335` | `too_many_arguments` | `insert_material`, lib-test | Same source fix as #10. |

### Stage 3: root application frontier

Command:

```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml -p extractum --all-targets --locked --keep-going --message-format=short -- -D warnings
```

Result: 62 library-test diagnostics at 60 unique file-and-line locations in 35
files. The library target emitted 50 occurrences; the library-test target
repeated those 50 and added 12 test-only findings, giving 112 occurrences
across both compilations. `src/lib.rs:402` is one macro-definition location for
three generated diagnostics: `start_project_analysis` (11 arguments),
`list_analysis_runs` (11), and `start_analysis_report` (12). Short output may
coalesce the two 11/7 expansions, while the target diagnostic total remains 62.

Lint totals for the 62-diagnostic inventory were: 22 `explicit_auto_deref`, 17
`too_many_arguments`, three `type_complexity`, three
`unnecessary_literal_unwrap`, two each of `bool_assert_comparison`,
`redundant_pattern_matching`, and `unwrap_or_default`, and one each of
`duplicate_mod`, `nonminimal_bool`, `manual_repeat_n`, `needless_question_mark`,
`needless_borrow`, `unnecessary_unwrap`, `implied_bounds_in_impls`,
`match_result_ok`, `field_reassign_with_default`, `get_first`, and
`await_holding_lock`.

| Location | Lint | Target relationship | Final disposition |
| --- | --- | --- | --- |
| `src/prompt_packs/source_adapter.rs:210` | `duplicate_mod` | lib + lib-test | Canonical test support is loaded once; a narrow test-only delegating seam replaces copied helpers. |
| `src/apalis_jobs.rs:729` | `nonminimal_bool` | lib + lib-test | Combined predicate mechanically. |
| `src/ingest_provenance.rs:372` | `explicit_auto_deref` | lib + lib-test | Direct mutable reference. |
| `src/job_helpers.rs:68` | `unwrap_or_default` | lib + lib-test | `or_default()`. |
| `src/projects/read_model.rs:144` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/projects/mod.rs:522` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/topic_memberships.rs:289` | `manual_repeat_n` | lib + lib-test | `repeat_n`. |
| `src/topic_memberships.rs:520` | `too_many_arguments` | lib + lib-test | Owning-function allow with state-shape ownership comment. |
| `src/prompt_packs/runtime_commands.rs:141` | `too_many_arguments` | lib + lib-test | Owning-command allow preserving IPC shape. |
| `src/prompt_packs/runtime_commands.rs:175` | `too_many_arguments` | lib + lib-test | Owning-command allow preserving IPC shape. |
| `src/accounts.rs:133` | `too_many_arguments` | lib + lib-test | Owning-command allow preserving wiring. |
| `src/takeout_import/recovery.rs:185` | `needless_question_mark` | lib + lib-test | Returned mapped result directly. |
| `src/sources/items/query.rs:54` | `too_many_arguments` | lib + lib-test | Owning-function allow preserving query shape. |
| `src/sources/items/query.rs:161` | `too_many_arguments` | lib + lib-test | Owning-function allow preserving query shape. |
| `src/sources/items.rs:250` | `needless_borrow` | lib + lib-test | Passed identity directly. |
| `src/sources/items.rs:562` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/youtube/captions.rs:331` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/youtube/process_runtime.rs:343` | `too_many_arguments` | lib + lib-test | Owning-function allow preserving lifecycle inputs. |
| `src/youtube/process_runtime.rs:385` | `unwrap_or_default` | lib + lib-test | `unwrap_or_default()`. |
| `src/youtube/process_runtime.rs:401` | `too_many_arguments` | lib + lib-test | Owning-function allow preserving lifecycle inputs. |
| `src/youtube/process_runtime.rs:447` | `redundant_pattern_matching` | lib + lib-test | Audited and changed to `.is_err()`; no error temporary owns lifecycle state. |
| `src/youtube/process_runtime.rs:456` | `redundant_pattern_matching` | lib + lib-test | Same audited `.is_err()` result as line 447. |
| `src/youtube/source_metadata.rs:785` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/youtube/source_metadata.rs:787` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/youtube/source_metadata.rs:812` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/notebooklm_export/query.rs:111` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/llm/profiles.rs:64` | `type_complexity` | lib + lib-test | Private type alias. |
| `src/llm/mod.rs:280` | `unnecessary_unwrap` | lib + lib-test | Matched key/base-URL pair via private selection helper. |
| `src/gemini_browser/jobs.rs:77` | `implied_bounds_in_impls` | lib + lib-test | Removed implied bound. |
| `src/gemini_browser/sidecar.rs:208` | `match_result_ok` | lib + lib-test | Matched `Ok(handle)` directly. |
| `src/analysis/fixtures/seed/runs.rs:20` | `too_many_arguments` | lib + lib-test | Owning fixture-function allow. |
| `src/analysis/fixtures/seed/runs.rs:65` | `too_many_arguments` | lib + lib-test | Owning fixture-function allow. |
| `src/analysis/fixtures/seed.rs:284` | `too_many_arguments` | lib + lib-test | Owning fixture-function allow. |
| `src/analysis/fixtures.rs:184` | `field_reassign_with_default` | lib + lib-test | Struct literal with `..Default::default()`. |
| `src/analysis/store/read_model.rs:131` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:132` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:133` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:144` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:145` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:156` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:157` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:168` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/read_model.rs:169` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/setup.rs:83` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/setup.rs:86` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/setup.rs:97` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/store/setup.rs:99` | `explicit_auto_deref` | lib + lib-test | Direct transaction reference. |
| `src/analysis/mod.rs:141` | `too_many_arguments` | lib + lib-test | Owning public-command allow. |
| `src/lib.rs:402` | `too_many_arguments` ×3 | lib + lib-test | Allows attach only to the three named generated IPC functions; macro/module remain unallowed. |
| `src/external_process.rs:287` | `type_complexity` | lib-test only | Test-only alias. |
| `src/diagnostics/database.rs:345` | `bool_assert_comparison` | lib-test only | Boolean assertion. |
| `src/diagnostics/database.rs:362` | `bool_assert_comparison` | lib-test only | Boolean assertion. |
| `src/library_sources/mod.rs:529` | `too_many_arguments` | lib-test only | Owning fixture-function allow. |
| `src/migrations.rs:1035` | `get_first` | lib-test only | `.first()`. |
| `src/telegram.rs:1026` | `unnecessary_literal_unwrap` | lib-test only | Used literal success value. |
| `src/telegram.rs:1031` | `unnecessary_literal_unwrap` | lib-test only | Used `true` directly. |
| `src/telegram.rs:1032` | `unnecessary_literal_unwrap` | lib-test only | Used `true` directly. |
| `src/takeout_import/mod.rs:2054` | `await_holding_lock` | lib-test only | Guard constrained to inner scope; copied assertion data before await. |
| `src/sources/store.rs:1169` | `type_complexity` | lib-test only | Test-only tuple alias. |
| `src/analysis/tests_application.rs:61` | `too_many_arguments` | lib-test only | Owning fixture-function allow. |

Disposition totals were 43 behavior-neutral mechanical diagnostics at 43
locations, 17 `too_many_arguments` diagnostics at 15 locations, and two
drop-order-audited lifecycle diagnostics. Only function- or generated-function
allows were used; no module, crate, macro, or global allow was added. The
process audit confirmed `terminate_and_reap` error temporaries own neither the
child, cookie, nor operation, so `.is_err()` preserves ordering. Exact
cancellation and injected-timeout tests confirmed reaping, detach ownership,
and cookie lifetime.

### Exact Task 3 staged source set

The corrected final scope was exactly these 55 Rust files:

```text
src-tauri/crates/extractum-analysis/src/report.rs
src-tauri/crates/extractum-analysis/src/report/tests/corpus_port.rs
src-tauri/crates/extractum-analysis/src/state.rs
src-tauri/crates/extractum-analysis/src/store/read_model.rs
src-tauri/crates/extractum-analysis/src/tests.rs
src-tauri/crates/extractum-gemini-browser/src/execution.rs
src-tauri/crates/extractum-gemini-browser/src/types.rs
src-tauri/crates/extractum-llm/src/lib.rs
src-tauri/crates/extractum-llm/src/scheduler.rs
src-tauri/crates/extractum-prompt-packs/src/dto.rs
src-tauri/crates/extractum-prompt-packs/src/lib.rs
src-tauri/crates/extractum-prompt-packs/src/result_builder.rs
src-tauri/crates/extractum-prompt-packs/src/runtime.rs
src-tauri/crates/extractum-prompt-packs/src/youtube_summary/gem_analysis.rs
src-tauri/crates/extractum-prompt-packs/src/youtube_summary/result_validation.rs
src-tauri/crates/extractum-prompt-packs/src/youtube_summary/snapshots.rs
src-tauri/crates/extractum-prompt-packs/src/youtube_summary/synthesis_input.rs
src-tauri/crates/extractum-telegram/src/live/peer.rs
src-tauri/crates/extractum-telegram/src/runtime.rs
src-tauri/src/accounts.rs
src-tauri/src/analysis/fixtures.rs
src-tauri/src/analysis/fixtures/seed.rs
src-tauri/src/analysis/fixtures/seed/runs.rs
src-tauri/src/analysis/mod.rs
src-tauri/src/analysis/store/read_model.rs
src-tauri/src/analysis/store/setup.rs
src-tauri/src/analysis/tests_application.rs
src-tauri/src/apalis_jobs.rs
src-tauri/src/diagnostics/database.rs
src-tauri/src/external_process.rs
src-tauri/src/gemini_browser/jobs.rs
src-tauri/src/gemini_browser/sidecar.rs
src-tauri/src/ingest_provenance.rs
src-tauri/src/job_helpers.rs
src-tauri/src/lib.rs
src-tauri/src/library_sources/mod.rs
src-tauri/src/llm/mod.rs
src-tauri/src/llm/profiles.rs
src-tauri/src/migrations.rs
src-tauri/src/notebooklm_export/query.rs
src-tauri/src/projects/mod.rs
src-tauri/src/projects/read_model.rs
src-tauri/src/prompt_packs/runtime_commands.rs
src-tauri/src/prompt_packs/source_adapter.rs
src-tauri/src/prompt_packs/youtube_summary/mod.rs
src-tauri/src/sources/items.rs
src-tauri/src/sources/items/query.rs
src-tauri/src/sources/store.rs
src-tauri/src/takeout_import/mod.rs
src-tauri/src/takeout_import/recovery.rs
src-tauri/src/telegram.rs
src-tauri/src/topic_memberships.rs
src-tauri/src/youtube/captions.rs
src-tauri/src/youtube/process_runtime.rs
src-tauri/src/youtube/source_metadata.rs
```

The following fully qualified exact focused-test commands selected one
non-empty test and passed (1 passed, 0 failed). The six
`prompt_packs::source_adapter` commands prove the canonical-fixture seam:

| Command | Result |
| --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib types::tests::sidecar_run_result_response_keeps_wire_shape --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::cancel_gemini_browser_job_cancels_queued_run_and_waiter --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib dto::tests::prompt_pack_runtime_provider_default_remains_api_and_serializes_as_api --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib public_api_tests::cancellation_smoke_services_remain_test_only --locked -- --exact` | PASS: fresh run 2026-08-14 14:49:15.597 UTC–14:50:46.480 UTC selected 1 test: 1 passed, 0 failed, 252 filtered (test duration 90.12s). |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib llm::tests::configured_provider_access_requires_key_and_base_url_together --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib takeout_import::tests::cancelled_job_emits_persisted_terminal_record --locked -- --exact` | PASS: 1 passed, 0 failed. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |
| `cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback --locked -- --exact` | PASS: 1 passed, 0 failed; 688 filtered. |

The final workspace Clippy command without diagnostic flags passed.

## License and duplicate policy

The deterministic license allowlist is:

```text
Apache-2.0
Apache-2.0 WITH LLVM-exception
BSD-3-Clause
CDLA-Permissive-2.0
ISC
MIT
MIT-0
MPL-2.0
Unicode-3.0
Zlib
```

All seven private workspace packages are ignored by `[licenses.private]`
because they are explicitly unpublished and do not declare license expressions.
No license clarification and no license/advisory/duplicate-growth exception was
required or committed. `Apache-2.0 WITH LLVM-exception` is currently unused and
therefore produces the configured non-failing warning.

The active Windows graph is unchanged from the generated baseline:

- 448 distinct package-version identities (evidence only);
- 400 distinct package names (evidence only);
- 32 duplicated package names (policy field);
- 80 package-version instances across duplicated names (policy field);
- duplicate cardinality matches the committed mapping exactly.

Cardinality is `2` for `bitflags`, `block-buffer`, `cipher`, `cpufeatures`,
`crypto-common`, `digest`, `foldhash`, `indexmap`, `inout`, `phf_codegen`,
`phf_macros`, `png`, `sha1`, `sha2`, `siphasher`, `syn`, `thiserror`,
`thiserror-impl`, `webpki-roots`, `windows-link`, and `winnow`; `3` for `phf`,
`phf_generator`, `phf_shared`, `rand`, `rand_chacha`, `windows_x86_64_msvc`,
and `windows-targets`; `4` for `getrandom`, `hashbrown`, and `rand_core`; and `5`
for `windows-sys`. Compared with the committed Task 5 baseline, aggregate and
per-name cardinality changes are all zero.

## Final local command evidence

All timestamps are UTC on 2026-08-14.

| Command | Time | Result |
| --- | --- | --- |
| `npm.cmd run test:unit -- scripts/verify.test.ts scripts/testing/repository-rules.test.ts scripts/testing/test-conventions.test.ts` | 03:05:24.215–03:05:31.952 | PASS, 3 files / 23 tests. |
| `npm.cmd run check` | 03:05:38.037–03:05:56.824 | PASS, 0 errors / 0 warnings. |
| `npm.cmd run bootstrap:testing` | 03:06:04.732–03:06:25.859 | PASS; Svelte sync ran first and the 45,200,478-byte sidecar binary was built and checked. |
| `npm.cmd run check:rust:fast` | 03:06:32.735–03:07:46.038 | PASS; rustfmt, locked workspace/all-target Clippy, both feature-off checks, and cargo-deny `bans ok, licenses ok, sources ok`. |
| `npm.cmd run verify` (sandbox attempt) | 03:07:52.332–03:11:24.442 | Non-authoritative sandbox failure at OS integration: unit 835/835, component 281/281, architecture 2/2 passed; sandboxed `taskkill` produced `termination_unconfirmed` and 5/43 OS tests failed. The exact reported test PIDs had exited before the fallback. |
| `npm.cmd run verify` (documented outside-sandbox fallback) | 03:11:47.511–03:22:33.147 | PASS in 645.6s: unit 113 files/835 tests, component 62/281, architecture 2/2, OS integration 4 files/43 tests, adapter Playwright 79/79, app Playwright 3/3, Svelte 0/0, rustfmt, locked workspace/all-target Cargo tests, and final diff check. |
| `npm.cmd run check:rust:advisories` (sandbox attempt) | 03:22:41.974–03:22:45.433 | Infrastructure-only failure: sandbox could not take the read-only user advisory-database lock. |
| `npm.cmd run check:rust:advisories` (authoritative outside sandbox) | 03:22:58.160–03:23:14.580 | Expected exit 1: exactly the three blockers below; warning-only yanked `spin 0.9.8`. |
| `node scripts/rust-duplicate-baseline.mjs` plus committed-baseline comparison | 03:23:32.721–03:23:34.490 | PASS: 32 names, 80 instances, exact cardinality match. |

The current advisory blockers are exactly:

- `RUSTSEC-2026-0190` — `anyhow 1.0.102` (fixed in `>=1.0.103`);
- `RUSTSEC-2026-0221` — `event-listener 5.4.1` (fixed in `>=5.4.2`);
- `RUSTSEC-2026-0097` — `rand 0.7.3` (fixed in supported branches described by the advisory).

There are no advisory exceptions. The ordinary deterministic local acceptance
contract is green, but release acceptance is red until these findings are
resolved or reviewed, time-bounded exceptions are committed.

## CI and release evidence boundary

Committed workflow definitions (local source links; these are not remote
executor entry pages):

- [Rust Fast workflow](../../../.github/workflows/rust-fast.yml)
- [Rust Full workflow](../../../.github/workflows/rust-full.yml)
- [scheduled advisory and Rust Release workflow](../../../.github/workflows/rust-release.yml)

At the local evidence cutoff, the Wave 0 branch had not been pushed and no PR
or manual release run existed, so there are no truthful GitHub Actions run URLs
to record. Local commands do not establish CI completion. In particular, no
green `Rust Fast`/`Rust Full` remote run is claimed.

The release job was not dispatched because the authoritative advisory command
is red and the workflow requires advisory success before `windows-bundle`.
Consequently no release executable, MSI, or NSIS artifact was produced and no
artifact hash or five-second packaged-application/sidecar smoke result exists.
The configured output paths are:

```text
src-tauri/target/x86_64-pc-windows-msvc/release/extractum.exe
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/*.msi
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*-setup.exe
```

This is an explicit incomplete release boundary, not a green release claim.
Any later wave touching Tauri, `windows-sys`, Keyring, or SQLx must resolve the
advisory gate as required, dispatch the bundle job, and add the run URL,
artifact paths/hashes, and both smoke results to that wave's evidence.

## Final conclusion

Wave 0 establishes the reproducible local infrastructure baseline: pinned
Rust/MSRV, a green Clippy frontier, separate feature-off production checks, a
single locked workspace test in the full verifier, pinned deterministic
supply-chain policy, executable duplicate/dependency rules, and distinct
fast/full/advisory/release workflows. The moving advisory lane and therefore
release acceptance remain red on the three explicitly recorded RustSec IDs.
