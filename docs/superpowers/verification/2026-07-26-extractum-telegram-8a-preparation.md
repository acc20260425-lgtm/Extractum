# Extractum Telegram Phase 8A Preparation Verification

**Disposition:** Phase 8A preparation retained. Phase 8 incomplete; 8B not
authorized.

**Pre-final evidence HEAD:** `eaa2dc2b1055d35015ec8b3781cb24dd69e9d540`

**Final evidence SHA:** handoff-only (not self-recorded)

## Authority

- [Implementation plan](../plans/2026-07-26-extractum-telegram-8a-preparation.md)
- [Telegram crate boundary design](../specs/2026-07-26-telegram-crate-boundary-design.md)
- [Crate roadmap](../specs/2026-07-17-crate-roadmap.md)
- [Focused Rust loop design](../specs/2026-07-17-focused-rust-loop-design.md)

Execution began from clean, tracked, owner-approved authority. The approved
boundary commit `9f56a4584b0c2abb331c1b2ab7f198ccb89db042` is an ancestor
of the recorded pre-Task-0 start.

## Commit and Authority Ledger

| Stage | SHA | Subject or role |
| --- | --- | --- |
| Pre-Task-0 start | `b50ca97b356b0eb77afbf02974af92358cf4de9e` | `docs: clarify Phase 8A coverage evidence` |
| Authority synchronization | `e9ed82a0211efad6982599ec52ce0cd50f794a53` | `docs: correct Phase 8 Telegram dependent surface` |
| Checkpoint 1 | `5948e0cde27049359a74811878d5e13721532c96` | `refactor: freeze Phase 8A Telegram boundary` |
| Checkpoint 2 | `9a3778cb2af6e971cc9c55f896712977a88812f5` | `test: characterize Phase 8A Telegram behavior` |
| Checkpoint 3 | `cf8a386fd28c7eb7ca45ab7452dce9a7011d43ff` | `refactor: prepare Telegram DTO and media ownership` |
| Plan correction | `b69501c6e3906ac939e04678bfb6e7c3304f44db` | `docs: correct Telegram session key representation` |
| Checkpoint 4 | `2d7332a028ea146d98f1def3d6823f6f9d556ab2` | `refactor: isolate Telegram session codec` |
| Plan correction | `06b5d175bf4f207b85170e485be83b0826bc4f06` | `docs: correct Checkpoint 5 status-gate ordering` |
| Checkpoint 5 | `3f412bf2af7371bb816c263c43921b01dc491ee0` | `refactor: prepare opaque Telegram runtime` |
| Fix ledger row 1 | `69032633fc2346d56e0854475be045cfb3e8a35b` | `fix: correct Phase 8A Checkpoint 5 gate` |
| Repository-correction amendment | `dbe884634d257f8a28cef7e929de76d0e9ddfbaa` | `docs: authorize Phase 8A repository contract correction` |
| Parser-safety amendment | `ee25edc4123f6155d6dba94fdda16a84f21fcddd` | `docs: harden Phase 8A cfg match-arm correction` |
| Fix ledger row 2 | `eaa2dc2b1055d35015ec8b3781cb24dd69e9d540` | `test: reconcile Phase 8A repository contracts` |

The four plan corrections are durable authority amendments, not preparation
checkpoints. The first preserves clone capability through the private shared
zeroizing session-key container. The second fixes Checkpoint 5 status-gate
ordering. The last two authorize and then harden only the observed
cross-checkpoint repository-contract correction.

## Task 6 Fix Ledger

<table>
  <thead>
    <tr>
      <th>sequence</th>
      <th>fix_sha</th>
      <th>owning_checkpoint</th>
      <th>failing_gate</th>
      <th>failure_summary</th>
      <th>exact_changed_paths</th>
      <th>owning_allowed_set</th>
      <th>focused_rerun</th>
      <th>checkpoint_rerun</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>1</td>
      <td><code>69032633fc2346d56e0854475be045cfb3e8a35b</code></td>
      <td>5</td>
      <td><code>npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts</code> =&gt; exit <code>1</code>; one file failed; <code>126/128</code> passed and two failed.</td>
      <td>The final Task 6 status exposed two Checkpoint-5-only assertions: one required the Checkpoint 5 roadmap/design status and one required lifecycle <code>8a-checkpoint-5</code> instead of admitting <code>8a-retained</code>.</td>
      <td><code>src/lib/telegram-crate-boundary-contract.test.ts</code></td>
      <td>
        <code>docs/superpowers/specs/2026-07-17-crate-roadmap.md</code><br>
        <code>docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md</code><br>
        <code>src-tauri/src/sources/store.rs</code><br>
        <code>src-tauri/src/sources/sync.rs</code><br>
        <code>src-tauri/src/takeout_import/mod.rs</code><br>
        <code>src-tauri/src/telegram.rs</code><br>
        <code>src-tauri/src/telegram/runtime.rs</code><br>
        <code>src/lib/crate-extraction-shell-cap-contract.test.ts</code><br>
        <code>src/lib/telegram-crate-boundary-contract.test.ts</code>
      </td>
      <td>
        <code>npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts</code> =&gt; exit <code>0</code>; one file; <code>128/128</code> passed.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::missing_account_authentication_is_false -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::sign_in_without_code_request_preserves_auth_error -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::failed_sign_in_retains_pending_attempt -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::tests::runtime_status_maps_to_existing_wire_strings -- --exact</code> =&gt; exit <code>0</code>; one passed, 718 filtered.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list</code> =&gt; exit <code>0</code>; 719 listed/719 unique and store prefix <code>24/24</code>.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib telegram::tests:: -- --nocapture</code> =&gt; exit <code>0</code>; <code>7/7</code> passed.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib sources::store::tests:: -- --nocapture</code> =&gt; exit <code>0</code>; <code>24/24</code> passed.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib sources::sync::tests:: -- --nocapture</code> =&gt; exit <code>0</code>; <code>6/6</code> passed.<br>
        <code>cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib takeout_import::tests:: -- --nocapture</code> =&gt; exit <code>0</code>; <code>17/17</code> passed.<br>
        <code>npm.cmd run check:rustfmt</code> =&gt; exit <code>0</code>.<br>
        <code>npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts</code> =&gt; exit <code>0</code>; one file; <code>8/8</code> passed.
      </td>
      <td>
        <code>cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets</code> =&gt; exit <code>0</code>.<br>
        <code>cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets</code> =&gt; exit <code>0</code>; library <code>719/719</code> plus binary <code>0/0</code>.
      </td>
    </tr>
    <tr>
      <td>2</td>
      <td><code>eaa2dc2b1055d35015ec8b3781cb24dd69e9d540</code></td>
      <td>3-5 (plan-amended repository-contract correction)</td>
      <td><code>npm.cmd run verify</code> =&gt; nonzero; the 38-record snapshots had two ingest plus one store hash drift, breadth expected/actual was <code>121/125</code>, and the Checkpoint 5 function-value-alias mutation timed out at <code>5000ms</code>.</td>
      <td>Authorized by <code>dbe884634d257f8a28cef7e929de76d0e9ddfbaa</code> and <code>ee25edc4123f6155d6dba94fdda16a84f21fcddd</code>: Checkpoint 3 added two reachable files and import-only ingest hash drift, Checkpoint 4 added one file, and Checkpoint 5 added one file, unrelated store-command hash drift, and a local-timeout gap. Pre-commit quality RED additionally closed the cfg-disabled match-arm continuation leak. No product or SQL behavior regressed.</td>
      <td>
        <code>src/lib/analysis-application-contract.test.ts</code><br>
        <code>src/lib/analysis-crate-boundary-contract.test.ts</code><br>
        <code>src/lib/telegram-crate-boundary-contract.test.ts</code>
      </td>
      <td>
        <code>src/lib/analysis-application-contract.test.ts</code><br>
        <code>src/lib/analysis-crate-boundary-contract.test.ts</code><br>
        <code>src/lib/telegram-crate-boundary-contract.test.ts</code>
      </td>
      <td>
        <code>npm.cmd run test -- src/lib/analysis-application-contract.test.ts -t 'keeps analysis SQL ownership and borrowed coordinator capabilities fail closed'</code> =&gt; exit <code>0</code>; one passed, 50 skipped (51).<br>
        <code>npm.cmd run test -- src/lib/analysis-crate-boundary-contract.test.ts -t 'keeps production SQL in the exact six-table owner'</code> =&gt; exit <code>0</code>; one passed, 15 skipped (16).<br>
        <code>npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts -t 'sync authorized lookup through functi'</code> =&gt; exit <code>0</code>; one passed, 127 skipped (128).
      </td>
      <td>
        <code>npm.cmd run test -- src/lib/analysis-application-contract.test.ts src/lib/analysis-crate-boundary-contract.test.ts</code> =&gt; exit <code>0</code>; two files; <code>67</code> passed.<br>
        <code>npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts</code> =&gt; exit <code>0</code>; <code>128</code> passed.<br>
        <code>npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts</code> =&gt; exit <code>0</code>; <code>8</code> passed.<br>
        <code>cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets</code> =&gt; exit <code>0</code>; library <code>719/719</code> plus binary <code>0/0</code>.<br>
        <code>npm.cmd run test</code> =&gt; exit <code>0</code>; <code>177</code> files and <code>1608</code> tests passed.
      </td>
    </tr>
  </tbody>
</table>

Row 1 changed a non-empty strict subset of the Checkpoint 5 allowlist. Row 2
is the single plan-authorized exception: its exact three-path changed set
equals its declared cross-checkpoint owning set and cannot be generalized.

The row 2 correction also passed:

- `npm.cmd run check:rustfmt`: exit `0`;
- `npm.cmd run check`: exit `0`, zero errors and zero warnings;
- `cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets`:
  exit `0`;
- all nine Checkpoint 5 exact Rust identities: each exit `0`, one passed and
  718 filtered;
- the exact store identity assertion: `24/24`;
- the broad Telegram/store/sync/Takeout suites: `7/24/6/17`.

## Changed-Path and Range Manifest

Task 0, authority synchronization
`e9ed82a0211efad6982599ec52ce0cd50f794a53`, changed exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
```

Checkpoint 1 `5948e0cde27049359a74811878d5e13721532c96` changed exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
src-tauri/src/sources/types.rs
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/telegram-contract-paths.ts
src/lib/telegram-crate-boundary-contract.test.ts
```

Checkpoint 2 `9a3778cb2af6e971cc9c55f896712977a88812f5` changed exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
src-tauri/src/takeout_import/state.rs
src-tauri/src/telegram.rs
src-tauri/src/telegram_session_store.rs
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/telegram-crate-boundary-contract.test.ts
```

Checkpoint 3 `cf8a386fd28c7eb7ca45ab7452dce9a7011d43ff` changed exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
src-tauri/src/ingest_provenance.rs
src-tauri/src/media.rs
src-tauri/src/sources/items.rs
src-tauri/src/sources/mod.rs
src-tauri/src/sources/sync.rs
src-tauri/src/sources/types.rs
src-tauri/src/takeout_import/migrated_history.rs
src-tauri/src/takeout_import/mod.rs
src-tauri/src/takeout_import/raw_parse.rs
src-tauri/src/telegram.rs
src-tauri/src/telegram/dto.rs
src-tauri/src/telegram/media.rs
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/telegram-crate-boundary-contract.test.ts
```

Plan correction `b69501c6e3906ac939e04678bfb6e7c3304f44db` changed only:

```text
docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
```

Checkpoint 4 `2d7332a028ea146d98f1def3d6823f6f9d556ab2` changed exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
src-tauri/src/telegram.rs
src-tauri/src/telegram/session.rs
src-tauri/src/telegram_session_store.rs
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/telegram-crate-boundary-contract.test.ts
```

Plan correction `06b5d175bf4f207b85170e485be83b0826bc4f06` changed only:

```text
docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
```

Checkpoint 5 `3f412bf2af7371bb816c263c43921b01dc491ee0` changed exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
src-tauri/src/sources/store.rs
src-tauri/src/sources/sync.rs
src-tauri/src/takeout_import/mod.rs
src-tauri/src/telegram.rs
src-tauri/src/telegram/runtime.rs
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/telegram-crate-boundary-contract.test.ts
```

Fix ledger row 1 `69032633fc2346d56e0854475be045cfb3e8a35b`
changed only:

```text
src/lib/telegram-crate-boundary-contract.test.ts
```

Repository-correction amendment
`dbe884634d257f8a28cef7e929de76d0e9ddfbaa` changed only:

```text
docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
```

Parser-safety amendment
`ee25edc4123f6155d6dba94fdda16a84f21fcddd` changed only:

```text
docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
```

Fix ledger row 2 `eaa2dc2b1055d35015ec8b3781cb24dd69e9d540`
changed exactly:

```text
src/lib/analysis-application-contract.test.ts
src/lib/analysis-crate-boundary-contract.test.ts
src/lib/telegram-crate-boundary-contract.test.ts
```

The complete committed range
`b50ca97b356b0eb77afbf02974af92358cf4de9e..eaa2dc2b1055d35015ec8b3781cb24dd69e9d540`
contains exactly these 25 unique paths:

```text
docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
src/lib/analysis-application-contract.test.ts
src/lib/analysis-crate-boundary-contract.test.ts
src/lib/crate-extraction-shell-cap-contract.test.ts
src/lib/telegram-contract-paths.ts
src/lib/telegram-crate-boundary-contract.test.ts
src-tauri/src/ingest_provenance.rs
src-tauri/src/media.rs
src-tauri/src/sources/items.rs
src-tauri/src/sources/mod.rs
src-tauri/src/sources/store.rs
src-tauri/src/sources/sync.rs
src-tauri/src/sources/types.rs
src-tauri/src/takeout_import/migrated_history.rs
src-tauri/src/takeout_import/mod.rs
src-tauri/src/takeout_import/raw_parse.rs
src-tauri/src/takeout_import/state.rs
src-tauri/src/telegram.rs
src-tauri/src/telegram/dto.rs
src-tauri/src/telegram/media.rs
src-tauri/src/telegram/runtime.rs
src-tauri/src/telegram/session.rs
src-tauri/src/telegram_session_store.rs
```

Task 6 changes exactly:

```text
docs/superpowers/specs/2026-07-17-crate-roadmap.md
docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md
docs/superpowers/verification/2026-07-26-extractum-telegram-8a-preparation.md
src/lib/crate-extraction-shell-cap-contract.test.ts
```

The eventual retained range therefore contains exactly 26 unique paths: the
preceding 25 plus the new verification document. It contains no Cargo
manifest, lockfile, migration, schema/value registry, frontend runtime, or
frontend UI path. The TypeScript paths are test-only boundary contracts and
their helper.

## Rollback Manifest

The canonical reverse order for the observed history is:

1. final evidence SHA, recorded in the handoff only;
2. repository-correction row 2
   `eaa2dc2b1055d35015ec8b3781cb24dd69e9d540`;
3. parser-safety amendment
   `ee25edc4123f6155d6dba94fdda16a84f21fcddd`;
4. repository-correction amendment
   `dbe884634d257f8a28cef7e929de76d0e9ddfbaa`;
5. Checkpoint 5 fix row 1
   `69032633fc2346d56e0854475be045cfb3e8a35b`;
6. Checkpoint 5 `3f412bf2af7371bb816c263c43921b01dc491ee0`;
7. Checkpoint 4 `2d7332a028ea146d98f1def3d6823f6f9d556ab2`;
8. Checkpoint 3 `cf8a386fd28c7eb7ca45ab7452dce9a7011d43ff`;
9. Checkpoint 2 `9a3778cb2af6e971cc9c55f896712977a88812f5`;
10. Checkpoint 1 `5948e0cde27049359a74811878d5e13721532c96`.

For ordinary rollback below the repository correction, row 2 is reverted
first, but both repository-correction amendments remain durable authority;
their positions above are retained/no-op steps unless an owner-approved
authority amendment explicitly supersedes one. The earlier plan corrections
`b69501c6e3906ac939e04678bfb6e7c3304f44db` and
`06b5d175bf4f207b85170e485be83b0826bc4f06`, and Task 0 authority
synchronization `e9ed82a0211efad6982599ec52ce0cd50f794a53`, likewise remain
durable authority through ordinary checkpoint rollback.

## Executable Identity Reconciliation

Fresh evidence from clean correction HEAD
`eaa2dc2b1055d35015ec8b3781cb24dd69e9d540` established:

- `npm.cmd run test -- src/lib/telegram-crate-boundary-contract.test.ts`:
  exit `0`, one file and `128/128` tests passed;
- `cargo test --color never --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list`:
  exit `0`, 719 listed and 719 unique identities;
- the same Cargo list matched the exact `sources::store::tests::` identity set
  `24/24`;
- the direct initialized-client identity occurred exactly once.

The committed map contract proved:

| Metric | Exact result |
| --- | ---: |
| Immutable baseline primaries | 140 |
| App / future-crate primary assignments | 99 / 41 |
| Post-8A baseline-derived identities | 141 |
| Post-8A tracked identities | 158 |
| Eventual baseline-derived identities | 143 |
| Eventual tracked identities | 160 |
| Plan-added rows / additional verification identities | 18 / 17 |
| Declared / current / deferred companions | 3 / 1 / 2 |

The exact 18-row breakdown is Checkpoint 2: three additional identities;
Checkpoint 3: one additional identity plus the current item-kind companion;
Checkpoint 4: four additional identities; Checkpoint 5: nine additional
identities. All plan additions are present once, with no duplicate,
unclassified, or deleted baseline identity. The two raw-parse companions
remain correctly deferred to 8B.

## Ownership-Perimeter Accounting

The original direct perimeter remains exactly 119 identities: 43
helper-dependent, three credential-SQL, and 73 residual identities. The
helper-dependent split is identity 2, sync 3, items 19, topics 5, Takeout root
12, and Takeout forum topics 2.

The three credential-SQL identities are
`legacy_api_hash_migrates_to_secret_store_and_blanks_column`,
`legacy_api_hash_remains_when_secret_write_fails`, and
`missing_secure_api_hash_for_blank_legacy_account_is_auth_error`.

The transitive closure remains 21 identities: `sources::types` 8,
`ingest_provenance` 7, and `takeout_import::migrated_history` 6. It has 20 app
primaries, one future-crate identity-validation primary, and the item-kind
companion.

The immutable ownership/move surface remains 19 paths and 140 baseline tests.
`src-tauri/src/sources/store.rs` is the sole dependent-only raw-client consumer
outside that map. Its exact 24 identities are disjoint from the baseline map
and are broad regressions rather than direct command or lookup evidence.

A fresh non-normative direct-Grammers scan at the correction HEAD found 14
Rust paths, 11,614 physical lines, and 122 test attributes. These physical
counts do not replace executable identity accounting.

## Core-Error Normalization

Exactly six identity-validation occurrences use the shared core contract: one
`extractum_core::error::AppResult`, three
`extractum_core::error::AppError::validation`, and two
`extractum_core::error::AppErrorKind::Validation`.

The live facade sentinel inventory remains 44 references: 37 `crate::error`,
five `crate::compression`, and two `crate::time`. The boundary contract proved
the six normalized occurrences and all 44 semantic sentinel sites.

## Observable Compatibility and Ownership

The boundary evidence preserves:

- all twelve account/Telegram command declarations, their single handler
  registration, and IPC keys;
- Telegram and Takeout event names, payloads, statuses, exact errors, and
  mutation/emission/cancellation/finalization ordering;
- secret identifiers, session filename and temporary-path behavior, envelope
  version/algorithm/field order, key/nonce sizes, base64, AAD, and all
  encrypted/legacy compensation paths;
- source-sync and Takeout limits, ordering, fallback identity, incremental
  persistence, cancellation, warning/provenance, and finalization.

`telegram/dto.rs`, `telegram/media.rs`, `telegram/session.rs`, and
`telegram/runtime.rs` uniquely own their prepared DTO, media, session, and
runtime subjects. The app retains commands/events, SQL, generic secure
storage, paths and filesystem writes, source persistence, Takeout
orchestration, diagnostics, and status mapping.

The public-API and secrecy scans found no extra public item, raw Grammers/TL,
SQLx, Tauri, keyring, raw constructor/conversion, public secret field,
plaintext getter, secret-bearing `Debug`, wrapper `ExposeSecret`
implementation, or plaintext logging/serialization surface. Public fallible
Telegram seams return `extractum_core::error::AppResult` directly.

The analysis contract correction retained exactly 38 production SQL-consumer
records and the exact six-table SQL owner. It only refreshed two ingest and
one store whole-source fingerprints, updated reachable app breadth from 121
to 125, and hardened test-only cfg-disabled match-arm parsing. It changed no
product source, SQL text, SQL owner, migration, command, or wire behavior.

## Runtime Lookup and Consumer Evidence

The direct initialized-client identity proved internal lock ownership, owned
opaque-handle return, exact missing-account behavior, last-insert-wins
semantics, and detached replacement-runner behavior.

The lifecycle contract proves exactly:

- two store `get_client`/raw-client sites in `list_telegram_sources` and
  `add_telegram_source`;
- one source-sync authorized/raw-client site in `sync_telegram_source`;
- three Takeout authorized/raw-client/raw-session workflows;
- no caller-held outer accounts lock, old `AuthorizedTelegramRuntime`, or
  `get_authorized_runtime`;
- the legacy `#[allow(dead_code)]` on
  `AuthorizedTelegramRuntime.session` is absent together with the old runtime
  type;
- neither new raw adapter has a dead-code escape.

The store/source-sync/Takeout suites of 24/6/17 are broad regressions, not
direct facade, command, lookup, or deadlock evidence. The owner-compatible
Checkpoint 5 rerun covered Telegram/store/sync/Takeout at 7/24/6/17 and the
package at 719/719. Fresh Task 6 evidence separately re-listed 719 unique
identities, proved the exact 24-store set, and found the direct lookup identity
once.

## Cargo and Package-Graph Identity

The clean correction HEAD retained the exact Task 1 hashes:

```text
src-tauri/Cargo.toml
81A773E6FFB5E4BC1AF7C25D2B3F723424E060A515289B2CB28B7C45360A31FF

src-tauri/Cargo.lock
720E38EA632D7B932B2A23D1481528845EC9304376035B1C851C546EA402E43C
```

`cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1`
exited `0`, parsed 735 packages and six workspace members (`extractum`,
`extractum-analysis`, `extractum-core`, `extractum-gemini-browser`,
`extractum-llm`, and `extractum-prompt-packs`), and found zero
`extractum-telegram` package, member, or path edge. Neither
`src-tauri/crates/extractum-telegram` nor `src-tauri/src/telegram_impl` exists.

All four direct Grammers dependencies remain app-owned:
`grammers-client`, `grammers-mtsender`, `grammers-session`, and
`grammers-tl-types`. Each resolves from
`git+https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa`.
There is no dependency, feature, revision, manifest, lock, package, workspace,
or path-edge change.

## Release and Startup Disposition

Release --no-bundle build: not applicable to 8A by approved design.
Startup smoke: not applicable to 8A by approved design.
Live credentialed Telegram request: intentionally not a gate.

Phase 8 incomplete; 8B not authorized.

## Completion Gates

The exact Task 6 completion block passed in order:

- `npm.cmd run check:rustfmt`: exit `0`;
- `cargo metadata --manifest-path src-tauri/Cargo.toml --locked
  --format-version 1`: exit `0`, 735 packages and six workspace members;
- `cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets`:
  exit `0`; the single admitted advisory measurement was `1,143 ms`;
- `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets`:
  exit `0`; seven test binaries reported `719`, `0`, `112`, `22`, `77`, `37`,
  and `249` passed respectively, for `1,216` passed and zero failed;
- `npm.cmd run verify`: exit `0`; its internal Vitest run passed 177 files and
  1,608 tests, Svelte check reported zero errors and zero warnings, rustfmt and
  workspace check passed, and workspace tests repeated the same successful
  seven-binary result.

The `1,143 ms` workspace-check value is diagnostic Phase 8A evidence only. It
does not participate in the completed-phase adjacent `>= 15,000 ms` rule, and
the correctness recheck inside `npm.cmd run verify` is not a second timing
sample.
