# Media Metadata Core Boundary Design

**Status:** Ready for review

## Context

The first Rust workspace slice established `extractum-core` with the shared
`error`, `time`, and `compression` modules. The next domain candidate is the
pure part of `notebooklm_export`, but its production code currently imports
`ItemMediaMetadata`, `decode_media_metadata`, and `media_label` through the
application-owned `media.rs` module.

`media.rs` mixes two responsibilities:

- a pure, serializable media-metadata model and codec;
- Telegram/grammers-specific media inspection and payload construction.

Moving the complete file into core would make core depend on the heavyweight
grammers git dependencies. Moving only the metadata type would leave the
future NotebookLM crate dependent on application-owned codec behavior. This
slice therefore separates the complete pure metadata API while retaining all
Telegram extraction in the application.

This is an architectural preparation slice. It preserves behavior and import
paths and has no performance-retention threshold. Performance is measured when
the pure NotebookLM domain is extracted into its own crate.

## Goals

- Create a pure `extractum_core::media_metadata` module.
- Keep `extractum-core` independent of grammers, Tauri, sqlx, and application
  domains.
- Preserve all existing `crate::media::...` paths in the application.
- Preserve the serialized JSON/zstd representation and typed error behavior.
- Keep all current consumers and the complete test inventory working.
- Prepare `notebooklm_export` for a later crate extraction without performing
  that extraction now.

## Non-Goals

- Do not extract `notebooklm_export` in this slice.
- Do not move Telegram media extraction, payload types, content-kind logic, or
  document-signal classification into core.
- Do not migrate existing application consumers to direct
  `extractum_core::media_metadata` imports.
- Do not add a new `extractum-media` crate.
- Do not change the stored media-metadata schema, database migrations, media
  labels, Tauri commands, UI behavior, or value-registry values.
- Do not use this slice to judge the performance value of domain extraction.

## Selected Architecture

Add `src-tauri/crates/extractum-core/src/media_metadata.rs` and expose it from
the curated core root with:

```rust
pub mod media_metadata;
```

The new module owns exactly this production API:

```rust
pub struct ItemMediaMetadata {
    pub summary: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_seconds: Option<f64>,
}

pub fn encode_media_metadata(metadata: &ItemMediaMetadata) -> AppResult<Vec<u8>>;
pub fn decode_media_metadata(bytes: Option<&[u8]>) -> AppResult<ItemMediaMetadata>;
pub fn media_label(kind: &str) -> &'static str;
```

The structure keeps its existing derives: `Clone`, `Default`, `Serialize`,
`Deserialize`, `Debug`, and `PartialEq`. The seven fields become public because
application and future member-crate consumers construct the structure with
literals. The functions use the existing core-owned `compression` and `error`
modules.

The codec also uses `serde_json`, which is currently a direct application
dependency but not a core dependency. Add the existing `serde_json = "1"`
declaration to `[workspace.dependencies]`, make both the application and core
inherit it with `workspace = true`, and do not upgrade its resolved version.
The resulting core dependency roots are exactly `serde`, `serde_json`, `time`,
and `zstd`.

The application `media.rs` removes the old definitions and explicitly
re-exports the core API at crate visibility:

```rust
pub(crate) use extractum_core::media_metadata::{
    decode_media_metadata, encode_media_metadata, media_label, ItemMediaMetadata,
};
```

This facade preserves current consumers under `notebooklm_export`, `sources`,
and `takeout_import`. It is intentionally explicit; neither the core root nor
the application facade uses a glob export.

## Application-Owned Media Adapter

The application continues to own all behavior that depends on grammers or is
specific to ingest orchestration:

- `ExtractedMediaPayload` and `ExtractedItemPayload`;
- `DocumentSignals`;
- the three content-kind constants;
- `derive_content_kind` and `derive_document_media_kind`;
- document/contact helpers;
- `extract_media_payload` and `extract_item_payload`;
- Telegram media inspection and conversion into `ItemMediaMetadata`.

The adapter constructs the now-public core type through the same structure
literals. No consumer is rewritten merely to reveal the new crate boundary.

## Data and Error Semantics

Encoding remains:

1. serialize `ItemMediaMetadata` with `serde_json::to_vec`;
2. compress the JSON bytes through the existing core zstd helper.

Decoding remains:

1. return `ItemMediaMetadata::default()` when the stored blob is absent;
2. decompress through the existing core helper;
3. deserialize with `serde_json::from_slice`.

Serialization, decompression, and JSON errors remain typed
`AppErrorKind::Internal` errors. Field names, optionality, default values,
compression level, and label strings remain unchanged. Existing database blobs
therefore require no migration.

## Testing Strategy

Move the pure tests with their implementation into
`extractum-core::media_metadata`:

- JSON/zstd roundtrip;
- corrupt metadata maps to a typed internal error;
- known and fallback media labels.

Add one characterization test for the existing absent-blob behavior:
`decode_media_metadata(None)` returns `ItemMediaMetadata::default()`.

Keep the grammers/content-kind tests in application `media.rs`. Their imports
are mechanically narrowed after the three pure tests move: the current shared
`use super::{...}` list no longer needs the codec, label, or metadata items for
the two remaining adapter tests. This import cleanup is part of the move, not a
behavioral change.

Moving the tests changes their module prefix. Inventory comparison therefore
uses this fixed rename map rather than requiring the old full names to remain:

| Baseline app test | Post-workspace core test |
| --- | --- |
| `media::tests::media_label_covers_known_and_fallback_kinds` | `media_metadata::tests::media_label_covers_known_and_fallback_kinds` |
| `media::tests::media_metadata_roundtrip_through_zstd` | `media_metadata::tests::media_metadata_roundtrip_through_zstd` |
| `media::tests::media_metadata_decode_failures_are_typed_internal_errors` | `media_metadata::tests::media_metadata_decode_failures_are_typed_internal_errors` |

For each baseline name, the inventory gate accepts either the unchanged name
or exactly its declared replacement. No other missing baseline name is
allowed. Three tests move, one new absent-blob characterization test is added,
and the two adapter tests remain under `media::tests`, so the complete Rust
inventory must grow by exactly one.

Add a source-level contract that verifies:

- `media_metadata.rs` exists and the core root explicitly declares it;
- the four intended API items are public and all seven fields are public;
- core media metadata contains no grammers, Tauri, sqlx, or application-domain
  dependency;
- the root and core manifests inherit the existing `serde_json` version from
  `[workspace.dependencies]`;
- application `media.rs` explicitly re-exports the four items;
- application `media.rs` no longer defines the metadata struct, codec
  functions, or label function;
- the three moved pure-test names no longer occur in application `media.rs`;
- neither side introduces a glob export.

Verification covers:

- the source contract in RED and GREEN states;
- `extractum-core` checks and tests;
- focused tests for `notebooklm_export`, `sources`, and `takeout_import`;
- complete workspace formatting, checking, and tests;
- before/after Rust test inventory using the fixed three-entry rename map,
  with no undeclared missing test and an exact net increase of one;
- the repository-wide `npm.cmd run verify` gate;
- `npm.cmd run tauri -- build --no-bundle` and release startup smoke.

Full MSI bundling remains excluded for the already documented baseline WiX
failure. The no-bundle build produces the release executable needed for the
smoke check.

## Alternatives Considered

### Separate `extractum-media` Crate

Rejected for this slice. Four pure operations do not justify another workspace
member, manifest, and dependency edge. The existing core already owns their
error and compression dependencies.

### Move Only `ItemMediaMetadata`

Rejected because the future NotebookLM crate would still need the
application-owned codec and label behavior. That would preserve the boundary
problem instead of resolving it.

### Move All of `media.rs`

Rejected because core would inherit grammers-client and Telegram-specific
types. This would reverse the purpose of the foundational crate boundary and
increase its rebuild surface.

## Risks and Mitigations

- **Public-field expansion:** the seven fields must become public across the
  crate boundary. The curated module API and source contract keep this
  expansion explicit.
- **Stored-format drift:** moving code could accidentally alter derives,
  serialization, or compression. Mechanical movement plus roundtrip tests and
  unchanged database fixtures protect the wire format.
- **Lost tests after workspace movement:** the complete `--workspace
  --all-targets` inventory is compared through the fixed rename map, requires
  both adapter tests to remain, and requires the exact `+1` total.
- **Copied rather than moved tests:** the source contract forbids all three
  pure-test names in application `media.rs`, while core execution requires
  their renamed forms.
- **Facade divergence:** the source contract requires an explicit four-item
  re-export and forbids duplicate application definitions.
- **Scope expansion into Telegram extraction:** dependency checks reject
  grammers in core, and the application-owned list defines the stopping point.

## Acceptance Criteria

1. `extractum-core` exposes the curated `media_metadata` module and no new
   external dependency other than the already resolved shared `serde_json`
   codec dependency.
2. The pure metadata type, seven fields, codec functions, and label function
   live only in core.
3. Existing application `crate::media` imports continue to compile through an
   explicit compatibility facade.
4. Telegram/grammers extraction stays entirely application-owned.
5. Stored data and typed error behavior are unchanged and require no migration.
6. Core, focused consumer, workspace, repository, and no-bundle build checks
   pass.
7. The complete Rust test inventory grows by exactly one; the three declared
   renamed tests execute under `extractum-core`, the two adapter tests remain
   under `media::tests`, and no undeclared baseline test disappears.
8. No performance claim or stop/go decision is made until the later
   `notebooklm_export` crate extraction.

## Follow-Up

After this boundary is verified, map the current `notebooklm_export`
dependencies and design the smallest pure member crate containing its models,
chunking, filename handling, links, mapping, glossary, and rendering. That
later slice reuses the existing domain and shell baseline probes and applies
the predeclared performance stop/go rule.
