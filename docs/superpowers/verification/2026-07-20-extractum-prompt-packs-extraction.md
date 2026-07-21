# Extractum Prompt Packs Crate Extraction Verification

## Scope and commit identities

Phase 6 implements the owner-approved
[Prompt Packs crate boundary](../specs/2026-07-20-prompt-packs-crate-boundary-design.md)
through the
[execution plan](../plans/2026-07-20-extractum-prompt-packs-extraction.md).
It extracts the Prompt Pack domain and YouTube Summary execution engine while
retaining Tauri integration, migrations, credentials, foreign source reads,
concrete Gemini Browser operations, spawning, and IPC emission in the
application.

The clean start commit was:

- bfdebc5c2a1244b7795de1a90224ae057b175668 — docs: make phase 6 checkpoint status explicit

The retained preparation checkpoints were:

- Checkpoint 1 GREEN: 0a6b152c3fb517d4c731d65169ae55676306e8d2 — test: characterize prompt-pack boundary and start phase 6
- Checkpoint 2 GREEN: 22df00ceeb2075b953b40d140b4f0ea34465125f — refactor: prepare prompt-pack public construction
- Checkpoint 3 GREEN: ed3ee4a0cf91f8fb7cb5fbafd7975a3eff2fa39c — refactor: prepare prompt-pack execution handoff
- Checkpoint 4 GREEN: 86581433c15f4ed8c8c0e37a1aa2a3e0385e6b40 — test: prepare prompt-pack crate fixtures

The intermediate preparation commits were:

- 08296228dcaffd3d194edcb72e8ba1534db3d8cd — refactor: isolate prompt-pack source reads
- 49fb503846dd7577cfc382917582c094d6c48d0f — refactor: isolate prompt-pack browser and events
- 8150f209b78dbb423975bb9eb60314b0ec0cc0b7 — refactor: isolate prompt-pack pool services and assets

Checkpoint 5 is the separately committed intentional RED boundary:

- 44ecda89436578d485e783c4b1af353639888126 — test: define prompt-pack crate boundary

The dedicated contract executed eight cases. Seven preparation-compatible
cases passed and the manifest/edge case failed on the intended assertion text:

    extractum-prompt-packs Cargo.toml is intentionally absent before the mechanical move

The RED output contained extractum-prompt-packs and no zero-test or unrelated
application failure. The mechanical extraction, and therefore the candidate
identity used for post-move evidence, is:

- 3bbec6135ee5820359e8a36c4402d1dda7115abb — refactor: extract prompt-pack domain crate

## Source inventory and final ownership

At the start identity, src-tauri/src/prompt_packs contained exactly 46 Rust
files: 21 root files and 25 youtube_summary files.

Root inventory (21):

    completion_transport.rs
    dto.rs
    gemini_browser_stage.rs
    json_repair.rs
    library.rs
    mod.rs
    models.rs
    projections.rs
    result_builder.rs
    result_commands.rs
    run_control.rs
    run_store.rs
    runtime.rs
    runtime_config.rs
    seed.rs
    stage_execution.rs
    stage_io.rs
    stage_output_normalization.rs
    stage_request_policy.rs
    store.rs
    validation.rs

youtube_summary inventory (25):

    entities.rs
    entities_tests.rs
    execution.rs
    execution_result.rs
    execution_tests.rs
    facade_tests.rs
    gem_analysis.rs
    mod.rs
    outputs.rs
    outputs_tests.rs
    preflight.rs
    preflight_tests.rs
    progress.rs
    result_validation.rs
    snapshots.rs
    snapshots_tests.rs
    sources.rs
    store.rs
    synthesis_execution.rs
    synthesis_input.rs
    synthesis_input_tests.rs
    tail_stages.rs
    test_support.rs
    transcript_execution.rs
    types.rs

The preparation checkpoints expanded that mixed inventory to 59 Rust files by
introducing explicit ports/adapters/services and splitting test fixtures before
the move. Final ownership is 11 application Rust files and 50 crate Rust files.

Application-owned files (11):

    src-tauri/src/prompt_packs/browser_adapter.rs
    src-tauri/src/prompt_packs/event_adapter.rs
    src-tauri/src/prompt_packs/library_command.rs
    src-tauri/src/prompt_packs/mod.rs
    src-tauri/src/prompt_packs/result_commands.rs
    src-tauri/src/prompt_packs/runtime_commands.rs
    src-tauri/src/prompt_packs/seed_command.rs
    src-tauri/src/prompt_packs/source_adapter.rs
    src-tauri/src/prompt_packs/youtube_summary/mod.rs
    src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs
    src-tauri/src/prompt_packs/youtube_summary/test_support.rs

Crate-owned root files (26):

    assets.rs
    browser_port.rs
    completion_transport.rs
    dto.rs
    events.rs
    gemini_browser_stage.rs
    json_repair.rs
    lib.rs
    library.rs
    models.rs
    projections.rs
    result_builder.rs
    result_service.rs
    run_control.rs
    run_store.rs
    runtime.rs
    runtime_config.rs
    seed.rs
    source_port.rs
    stage_execution.rs
    stage_io.rs
    stage_output_normalization.rs
    stage_request_policy.rs
    store.rs
    test_schema.rs
    validation.rs

Crate-owned youtube_summary files (24):

    entities.rs
    entities_tests.rs
    execution.rs
    execution_result.rs
    execution_tests.rs
    facade_tests.rs
    gem_analysis.rs
    mod.rs
    outputs.rs
    outputs_tests.rs
    preflight.rs
    preflight_tests.rs
    progress.rs
    result_validation.rs
    snapshots.rs
    snapshots_tests.rs
    store.rs
    synthesis_execution.rs
    synthesis_input.rs
    synthesis_input_tests.rs
    tail_stages.rs
    test_support.rs
    transcript_execution.rs
    types.rs

The application facade is private and explicitly re-exports its command
surface. The crate root declares private modules and explicitly re-exports only
the approved API. There is no public module or glob export.

## Frozen baseline and new characterization tests

Appendix A parsed to exactly 225 unique logical module/leaf identities. Final
executable ownership is:

- 223 baseline identities in extractum-prompt-packs;
- 2 baseline identities in extractum;
- 225 total, each declared once.

The two application identities are:

    prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer
    prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled

New tests are counted separately and do not replace Appendix A identities:

- extractum-prompt-packs: 248 executable tests = 223 baseline + 25 new characterizations;
- extractum Prompt Pack modules: 12 executable tests = 2 baseline + 10 new app-adapter characterizations;
- total new Prompt Pack characterizations: 35.

The boundary scanner found no disabled false-cfg copy, commented test,
legacy_disabled_* substitute, duplicate baseline leaf at an unapproved path,
or renamed baseline replacement.

## Manifest and Cargo.lock proof

The workspace now contains the exact member
crates/extractum-prompt-packs. The application has exactly one path edge:

    extractum-prompt-packs = { path = "crates/extractum-prompt-packs" }

The crate production dependency roots are exactly:

    extractum-core
    extractum-gemini-browser
    extractum-llm
    jsonschema
    serde
    serde_json
    sha2
    sqlx
    tokio
    tokio-util

Development roots are exactly tempfile, time, and tokio. sha2 and sqlx are
canonical workspace dependencies; jsonschema moved from the app to the new
crate. The lockfile contains one source-less/checksum-less
extractum-prompt-packs package with exactly these twelve roots:

    extractum-core
    extractum-gemini-browser
    extractum-llm
    jsonschema
    serde
    serde_json
    sha2
    sqlx
    tempfile
    time
    tokio
    tokio-util

The application lock package gained exactly the new crate edge and lost
jsonschema. extractum-core, extractum-gemini-browser, and extractum-llm have no
reverse edge. Registry versions remained:

    jsonschema 0.46.5
    sha2 0.10.9
    sqlx 0.8.6
    tempfile 3.27.0
    time 0.3.47
    tokio 1.52.1
    tokio-util 0.7.18

The exact manifest source contract and
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1
both passed.

## SQL and migration ownership

The production crate SQL scan found only the approved 32-table allowlist:

    prompt_packs
    prompt_pack_versions
    prompt_pack_stage_templates
    prompt_pack_schema_assets
    prompt_pack_runs
    prompt_pack_run_scopes
    prompt_pack_run_source_snapshots
    prompt_pack_run_source_origins
    prompt_pack_run_material_snapshots
    prompt_pack_stage_runs
    prompt_pack_stage_artifacts
    prompt_pack_results
    prompt_pack_result_source_refs
    prompt_pack_result_claims
    prompt_pack_result_evidence
    prompt_pack_result_ref_edges
    prompt_pack_result_unknowns
    prompt_pack_result_verification_tasks
    prompt_pack_result_warnings
    prompt_pack_result_limitations
    prompt_pack_result_quality_flags
    prompt_pack_result_audit_refs
    prompt_pack_youtube_videos
    prompt_pack_youtube_segments
    prompt_pack_youtube_key_points
    prompt_pack_youtube_quotes
    prompt_pack_youtube_action_items
    prompt_pack_youtube_open_questions
    prompt_pack_youtube_synthesis_items
    prompt_pack_result_validation_findings
    prompt_pack_audit_events
    prompt_pack_result_quarantine_artifacts

The five source-owned tables sources, youtube_video_sources,
youtube_playlist_items, youtube_transcript_segments, and items were absent
from production crate SQL and remain behind source_adapter.rs. projects was
also absent, preserving the explicit no-project-query assertion. Tauri,
AppHandle, State, Emitter, Manager, get_pool, application modules, and concrete
Browser integration were absent from production crate source.

The standing parser proved exact ordered parity between the first twelve
non-Apalis build_migrations entries and the private fixture:

    0001_current_schema_baseline.sql
    0002_migrated_history_opt_in_schema.sql
    0003_analysis_telegram_history_scope.sql
    0004_source_delete_cascade_indexes.sql
    0005_projects_mvp.sql
    0006_prompt_pack_mvp.sql
    0007_prompt_pack_run_idempotency.sql
    0008_prompt_pack_run_labels.sql
    0009_prompt_pack_intermediate_entities_artifacts.sql
    0010_prompt_pack_runtime_provider.sql
    0011_prompt_pack_stage_browser_provenance.sql
    0012_projects_redesign.sql

All twelve paths are unique, exist, resolve to the canonical migration files,
and occur in the same order. The fixture is cfg(test)-private, uses one
transaction and sqlx::raw_sql, and contains no Apalis entry or second migration
copy.

## Bundled asset identity

All eight includes are centralized in crate-private assets.rs. Raw file
SHA-384 values are:

| Relative asset | SHA-384 |
| --- | --- |
| pack.json | 21d0e7803f25474bb761cbe5c9fe6e45ef363cf5d9c7f030f7c84ee02ef9b7d8dd3664dfed782a3e8c607b7a0f37cf06 |
| runtime/synthesis.json | 36b1c4653bc4befdcd168b482929f3b34980c58d9179cb0e0e3db9ac4d3760f9e66dc834ad6a799df6df62618b28d367 |
| runtime/transcript_analysis.json | a9ba63c8ff582429866042aad354693cf9a583f5fc05f319189f44266d9eec6871b0ceb40758719a4b0d95dc8f25ee8f |
| schemas/canonical-result.json | 067ac18d452b6ec6ca2000899d3e7d8df87ace30e4676c7f88080a59cc4731887032943c7ea961ac39b69ab17e9697fd |
| schemas/stage-io-youtube-summary-synthesis-output.json | ff518213fba16805dfbde2c6c55f8d3ca204ca7f772fb2348cfc375e83070289bfd29623ea4af1b78044504e92a22dac |
| schemas/stage-io-youtube-summary-transcript-analysis-input.json | bb75aad9fd645912f723ad470a715f7b43c3af964ee4ea74cd84bebb635a1d3bc5bb0ac5460c9608e15eabee07b74419 |
| schemas/stage-io-youtube-summary-transcript-analysis-output.json | 9d3d32cf7b7bfd00fdc5ae6d74dac8ad06f488b05e31e52866553aeaa1cd836c1d6599d5dd21c2228abf51e4bcc5f693 |
| stages/transcript_analysis.json | 1b4f18dc3b1baf4b01389a6187d54b96ed689dc044aefd6338a2a176779f433359b0bdc77364fec1ef2ccb58a9088793 |

The persisted value is unchanged:

    bundled_source_path = "src-tauri/prompt-packs/youtube_summary/1.0.0"

No runtime filesystem read, packaging copy, build script, or second asset copy
was introduced.

The eight asset paths have explicit Git EOL attributes matching these hashes:
five CRLF assets, two LF assets, and the historically mixed-line-ending
canonical-result schema stored byte-for-byte with `-text` and checked with
`whitespace=cr-at-eol`. This makes the raw hash contract stable in the main
checkout, linked worktrees, and fresh clones without false trailing-whitespace
reports from Git.

## Focused, package, workspace, and repository gates

The extraction boundary group passed 14 files and 101 Vitest tests. It proved
the manifest/lock surface, curated APIs, 223/2 ownership, absence of copied
legacy tests, SQL/integration ownership, port handoffs, eight assets, and
twelve-entry migration parity.

Ten exact crate tests and both exact app tests each executed one test and
passed. The exact crate cases covered IPC serialization, fresh source reads,
active Browser cancellation, terminal state cleanup, scheduler metadata and
typed cancellation, Browser provenance, transactional rollback, backend-owned
ID rejection, validation rollback, and queued-run completion. The exact app
cases were the two retained Appendix A identities above.

Completion command outcomes:

    cargo check --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets: PASS
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --all-targets: 248 passed
    cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets: PASS
    cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets: 785 passed
    npm.cmd run check:rustfmt: PASS
    cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets: PASS
    cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets: PASS

The first sandboxed npm.cmd run verify had one unrelated Windows process-tree
diagnostic failure: expected timeout, received termination_unconfirmed after
taskkill reported ERROR: Access denied. The unchanged focused diagnostic then
passed 13/13 outside the sandbox. The full verify rerun outside the sandbox
passed: 173 Vitest files and 1,398 tests, Svelte diagnostics 0 errors and
0 warnings, rustfmt success, and successful Cargo workspace check/tests. No
correctness failure was excluded.

## Advisory focused timing

The baseline and candidate probe had the same original SHA-256:

    e2ec313a7ef91f0d21411cdd5bdba7ea1fe83480d619fc9943ebc30346ceeed5

Scratch evidence:

    BASELINE_RAW_MS=[11286, 9669, 9006]
    BASELINE_MEDIAN_MS=9669
    CANDIDATE_RAW_MS=[2456, 2409, 2460]
    CANDIDATE_DISPOSITION=incomplete / no performance conclusion

The candidate warm-up and three samples passed. Removing the marker with the
patch helper changed one CRLF boundary to a bare LF, so the immediate
restoration hash proof failed. Per protocol, the candidate series was marked
exactly incomplete / no performance conclusion and was not retried. A
mechanical byte repair restored the exact SHA-256 above, the source diff was
empty, and the worktree was clean. The candidate raw values are retained as
observations but have no admitted median, delta, percentage, or causal
performance conclusion. Timing did not decide retention.

## Ordinary workspace timing signal

The single timed mandatory workspace check completed in 11,669 ms:

    workspace_check,1,11669

This result is below 15,000 ms. Phase 5 was 10,410 ms, also below 15,000 ms,
so the adjacent completed-slice sequence remains below threshold and no
performance investigation is triggered. Mechanically, because Phase 5 was
below threshold, even a Phase 6 result at or above 15,000 ms would only be the
first possible member of a future adjacent above-threshold pair.

## Release and startup evidence

The required non-installer command passed:

    npm.cmd run tauri -- build --no-bundle

The controlled confirmation exited 0, reported:

    Finished release profile [optimized] target(s) in 4m 36s

It produced:

    G:\Develop\Extractum\.worktrees\prompt-packs-extraction\src-tauri\target\release\extractum.exe

The executable was 39,542,784 bytes. MSI/WiX packaging was not run.

The bounded smoke started that exact executable hidden. PID 13480 remained
alive after five seconds, was force-stopped by exact PID, disappeared within
the bounded cleanup wait, and the final Extractum process count was zero. No
provider request or account-data mutation was performed.

An earlier smoke also survived five seconds on PID 5020 and stopped that PID,
but an immediate process-name enumeration briefly observed the terminating
process. A read-only follow-up showed PID 5020 absent and zero Extractum
processes. The successful repeated smoke used bounded polling for complete PID
removal and supplied the final evidence above.

## Moved-not-copied review

git show --summary --find-renames=50% recognized 47 moved source identities in
the mechanical commit; 44 were 90–100% renames. The remaining new crate roots
were the manifest/root modules and prepared asset/test-support splits whose
app-side source fragments were removed in the same commit.

The exact final source maps contain only the 11 approved app files and 50
approved crate files. Source-contract scans found no disabled or copied legacy
implementation, no duplicate test identity, no old domain module left in the
app, no Tauri/app integration in production crate code, and no unapproved
foreign SQL. Manual review therefore classifies the extraction as
moved/split-not-copied; no behavior-bearing duplicate was retained.

## Post-merge checkout remediation

The first full verification from the main worktree discovered that Vitest also
scanned the ignored linked worktree below `.worktrees/`, doubling source tests
and importing Playwright suites into Vitest. The same run exposed that three
asset hash expectations had captured CRLF-converted feature-worktree bytes
instead of the owner-approved hashes in the execution plan.

The Vitest wrapper now excludes `.worktrees/**`. The asset contract uses the
eight approved hashes and requires the explicit per-file EOL attributes above.
The canonical-result schema's exact mixed bytes are stored in Git rather than
being line-ending normalized. Focused RED/GREEN and the repeated full merged
verification cover both remediations.

## Post-audit remediation

Commit `fb39c6af` (`fix: close prompt-pack extraction audit gaps`) closes the
four actionable findings from the retained-tree audit:

- a terminal persistence failure now still removes the active run and emits
  the terminal event before returning the persistence error;
- the spawned execution task reuses the pool acquired by the start command
  instead of acquiring a second pool;
- crate-owned cancellation fixtures are available only to crate tests or the
  non-default `dev-fixtures` feature, while the app commands require both
  Tauri `dev` mode and `prompt-pack-dev-fixtures`; only the MCP development
  overlay enables that feature, and the base/release configuration excludes
  it;
- the frozen source-reader baseline now asserts the exact typed transcript
  segment order and joined bytes rather than comparing two paths backed by the
  same query.

The new terminal-cleanup and pool-reuse tests first failed against the audited
tree and then passed with the fixes. The frozen-baseline mutation proof changed
the adapter ordering from ascending to descending: the exact test failed with
the reversed segment vector, then passed again after the mutation was removed.
The prompt-pack boundary and Tauri security contracts passed 15/15 together;
the final focused security run passed 6/6.

Both application compile modes passed: ordinary features and
`--features prompt-pack-dev-fixtures`. Package checkpoints passed 249/249 for
`extractum-prompt-packs` and 786/786 for `extractum`. On the exact code commit,
`npm.cmd run verify` passed 173/173 Vitest files and 1399/1399 tests,
`svelte-check` reported 0 errors and 0 warnings, and rustfmt plus the complete
workspace check/test gates passed.

The ordinary production command `npm.cmd run tauri -- build --no-bundle`
completed in release mode without the development feature. Its executable was
`src-tauri/target/release/extractum.exe`, 39,557,120 bytes, SHA-256
`1e4a813fc4a02d49546e893c5af57fbb7234b9d358301c78965bc4b4c876596b`.
An exact-PID smoke started PID 18000 hidden, observed it alive after five
seconds, stopped that PID, confirmed its bounded removal, and finished with
zero Extractum processes. No provider request or live account-data mutation
was performed.

This section supersedes the earlier release/startup evidence for the current
retained tree; the earlier section remains the historical evidence for the
original extraction.

## Result and next phase

**Result: implemented and retained.**

All correctness, ownership, package, workspace, repository, release, startup,
cleanup, and documentation gates passed. Advisory timing produced no candidate
performance conclusion and did not affect retention. Phase 7
extractum-analysis remains future work and requires a fresh owner-approved JIT
boundary design and explicit implementation instruction.
