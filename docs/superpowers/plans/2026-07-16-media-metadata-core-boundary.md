# Media Metadata Core Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the pure media metadata model, codec, and label function into
`extractum-core` while preserving every current application import path,
stored-data behavior, and workspace test.

**Architecture:** Add a curated `extractum_core::media_metadata` module that
uses the existing core error/compression APIs and the shared `serde_json`
dependency. Keep `src-tauri/src/media.rs` as the grammers-specific adapter and
explicitly re-export the four core API items at application-crate visibility.

**Tech Stack:** Rust 2021, Cargo workspaces, serde/serde_json, zstd,
Tauri 2, Vitest source contracts, PowerShell 5.1 on Windows.

## Global Constraints

- Implement only
  `docs/superpowers/specs/2026-07-16-media-metadata-core-boundary-design.md`.
- This is an architectural preparation slice; do not apply a performance
  threshold or extract `notebooklm_export`.
- Do not move grammers types, Telegram extraction, payload types,
  content-kind constants, `DocumentSignals`, `derive_content_kind`, or
  `derive_document_media_kind` into core.
- Keep every current application consumer on `crate::media::...` through an
  explicit `pub(crate) use`; do not mass-rewrite imports.
- Core dependency roots after this slice are exactly `serde`, `serde_json`,
  `time`, and `zstd`.
- Keep `serde_json` at its currently resolved version. Add it to
  `[workspace.dependencies]`; both the root package and core inherit it.
- Preserve the exact seven metadata fields, derives, JSON representation,
  zstd compression behavior, label strings, and typed internal errors.
- Do not add migrations, value-registry entries, Tauri commands, UI changes,
  glob exports, or a new workspace member.
- Use canonical `src-tauri/target`; do not set `CARGO_TARGET_DIR`, use
  `--target-dir`, or run `cargo clean`.
- Use `npm.cmd`, not plain `npm`, for npm scripts on Windows.
- Full MSI bundling remains excluded because of the documented baseline WiX
  failure. Build with `npm.cmd run tauri -- build --no-bundle`.
- Inspect the dirty worktree before commits and stage only files owned by this
  plan.

---

### Task 1: Capture Inventory and Characterize Absent Metadata

**Files:**
- Modify: `src-tauri/src/media.rs` (test module)
- Read/write temporary evidence only below `$env:TEMP`

**Interfaces:**
- Consumes: current `decode_media_metadata(Option<&[u8]>)` behavior.
- Produces: a committed characterization test named
  `media::tests::absent_media_metadata_decodes_to_default` before any module
  movement.

- [ ] **Step 1: Require a clean approved starting state**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
$spec = 'docs/superpowers/specs/2026-07-16-media-metadata-core-boundary-design.md'
$plan = 'docs/superpowers/plans/2026-07-16-media-metadata-core-boundary.md'
$specTracked = @(git ls-files --error-unmatch $spec 2>$null).Count -eq 1
$planTracked = @(git ls-files --error-unmatch $plan 2>$null).Count -eq 1
"STATUS_COUNT=$($status.Count)"
"SPEC_TRACKED=$specTracked"
"PLAN_TRACKED=$planTracked"
"HEAD=$((git rev-parse HEAD).Trim())"
if ($status.Count -ne 0 -or -not $specTracked -or -not $planTracked) { exit 1 }
```

Expected: clean tree and both approved documents committed. Stop if this is
not true; do not mix unrelated changes into the slice.

- [ ] **Step 2: Capture the complete baseline Rust inventory**

Run:

```powershell
$head = (git rev-parse HEAD).Trim()
$scratch = Join-Path $env:TEMP "extractum-media-metadata-$head"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$scratch | Set-Content -LiteralPath (Join-Path $env:TEMP 'extractum-media-metadata-current.txt')
$log = Join-Path $scratch 'baseline-inventory.log'
& cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list 2>&1 |
  Tee-Object -FilePath $log
$exit = $LASTEXITCODE
$names = @(
  Get-Content -LiteralPath $log |
    ForEach-Object { $_.ToString() } |
    Where-Object { $_ -match ': test$' } |
    ForEach-Object { ($_ -replace ': test$', '').Trim() }
)
$unique = @($names | Sort-Object -Unique)
$unique | Set-Content -LiteralPath (Join-Path $scratch 'baseline-test-names.txt')
@{
  head = $head
  exit = $exit
  count = $names.Count
  unique_count = $unique.Count
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'baseline-inventory.json')
if ($exit -ne 0 -or $names.Count -eq 0 -or $unique.Count -ne $names.Count) { exit 1 }
```

Expected on the current approved commit: 1125 unique tests. Treat that number
as observed evidence rather than a permanent repository constant. The file
must contain all five current `media::tests` names.

- [ ] **Step 3: Add the absent-blob characterization test before moving code**

In the existing `#[cfg(test)] mod tests` in `src-tauri/src/media.rs`, add this
test next to the other metadata tests:

```rust
#[test]
fn absent_media_metadata_decodes_to_default() {
    let decoded = decode_media_metadata(None).expect("decode absent metadata");

    assert_eq!(decoded, ItemMediaMetadata::default());
}
```

Do not change production code. The existing shared `use super::{...}` already
imports both required items.

- [ ] **Step 4: Run the characterization test on the old implementation**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib media::tests::absent_media_metadata_decodes_to_default -- --exact
```

Expected: exactly one test runs and passes. A zero-test run fails this step.
This is a characterization GREEN against existing behavior, intentionally
recorded before movement; the source-boundary contract in Task 2 supplies RED.

- [ ] **Step 5: Run the complete current media test module**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib media::tests
```

Expected: six tests pass: the five baseline tests plus the new absent-blob
test.

- [ ] **Step 6: Commit the characterization checkpoint**

Run:

```powershell
git diff --check
git add -- src-tauri/src/media.rs
git diff --cached --check
git commit -m "test: characterize absent media metadata"
git status --short
```

Expected: the commit contains only the new test and the tree is clean.

---

### Task 2: Add the RED Contract and Extract the Pure Module

**Files:**
- Create: `src/lib/media-metadata-core-contract.test.ts`
- Create: `src-tauri/crates/extractum-core/src/media_metadata.rs`
- Modify: `src-tauri/crates/extractum-core/src/lib.rs`
- Modify: `src-tauri/crates/extractum-core/Cargo.toml`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/media.rs`

**Interfaces:**
- Produces:
  `extractum_core::media_metadata::{ItemMediaMetadata,
  encode_media_metadata, decode_media_metadata, media_label}`.
- Preserves:
  `crate::media::{ItemMediaMetadata, encode_media_metadata,
  decode_media_metadata, media_label}` through explicit app re-exports.

- [ ] **Step 1: Create the source-boundary contract**

Create `src/lib/media-metadata-core-contract.test.ts`:

```typescript
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const readOptionalSource = (relativePath: string) =>
  existsSync(path.join(repoRoot, relativePath)) ? readSource(relativePath) : "";

const rootCargo = readSource("src-tauri/Cargo.toml");
const coreCargo = readSource("src-tauri/crates/extractum-core/Cargo.toml");
const coreLib = readSource("src-tauri/crates/extractum-core/src/lib.rs");
const coreMedia = readOptionalSource(
  "src-tauri/crates/extractum-core/src/media_metadata.rs",
);
const appMedia = readSource("src-tauri/src/media.rs");

describe("media metadata core boundary", () => {
  it("inherits serde_json in both workspace packages", () => {
    expect(rootCargo).toMatch(
      /\[workspace\.dependencies\][\s\S]*serde_json\s*=\s*"1"/,
    );
    expect(rootCargo).toMatch(
      /\[dependencies\][\s\S]*serde_json\s*=\s*\{\s*workspace\s*=\s*true\s*\}/,
    );
    expect(coreCargo).toMatch(
      /\[dependencies\][\s\S]*serde_json\.workspace\s*=\s*true/,
    );
  });

  it("exposes one curated pure media metadata module", () => {
    expect(coreLib).toContain("pub mod media_metadata;");
    expect(coreMedia).not.toBe("");
    expect(coreMedia).toMatch(/pub\s+struct\s+ItemMediaMetadata/);

    for (const field of [
      "summary",
      "file_name",
      "mime_type",
      "size_bytes",
      "width",
      "height",
      "duration_seconds",
    ]) {
      expect(coreMedia).toMatch(new RegExp(`pub\\s+${field}\\s*:`));
    }

    for (const functionName of [
      "encode_media_metadata",
      "decode_media_metadata",
      "media_label",
    ]) {
      expect(coreMedia).toMatch(new RegExp(`pub\\s+fn\\s+${functionName}\\b`));
    }
  });

  it("keeps application and heavyweight dependencies out of core media metadata", () => {
    for (const forbidden of [
      "grammers",
      "tauri",
      "sqlx",
      "notebooklm_export",
      "takeout_import",
      "crate::media",
      "crate::sources",
    ]) {
      expect(coreMedia).not.toContain(forbidden);
    }
    expect(coreMedia).not.toMatch(/(?:pub\s+use|use)\s+[^;]*\*/);
  });

  it("preserves one explicit application facade without duplicate definitions", () => {
    expect(appMedia).toMatch(
      /pub\(crate\)\s+use\s+extractum_core::media_metadata::\{[\s\S]*decode_media_metadata[\s\S]*encode_media_metadata[\s\S]*media_label[\s\S]*ItemMediaMetadata[\s\S]*\};/,
    );

    expect(appMedia).not.toMatch(/pub\(crate\)\s+struct\s+ItemMediaMetadata/);
    for (const functionName of [
      "encode_media_metadata",
      "decode_media_metadata",
      "media_label",
    ]) {
      expect(appMedia).not.toMatch(
        new RegExp(`pub\\(crate\\)\\s+fn\\s+${functionName}\\b`),
      );
    }
    expect(appMedia).not.toMatch(/extractum_core::media_metadata::\*/);
  });

  it("moves rather than copies all pure metadata tests", () => {
    for (const testName of [
      "media_label_covers_known_and_fallback_kinds",
      "media_metadata_roundtrip_through_zstd",
      "media_metadata_decode_failures_are_typed_internal_errors",
      "absent_media_metadata_decodes_to_default",
    ]) {
      expect(appMedia).not.toContain(`fn ${testName}()`);
      expect(coreMedia).toContain(`fn ${testName}()`);
    }
  });
});
```

- [ ] **Step 2: Run the contract to verify RED**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/media-metadata-core-contract.test.ts
```

Expected: five tests are collected and the file fails because
`media_metadata.rs`, workspace `serde_json`, the facade, and moved tests are
absent. An import-resolution error or a zero-test run is not the intended RED.

- [ ] **Step 3: Share the existing serde_json dependency**

In `src-tauri/Cargo.toml`, add this exact workspace dependency next to serde:

```toml
serde_json = "1"
```

Change the existing root dependency from:

```toml
serde_json = "1"
```

to:

```toml
serde_json = { workspace = true }
```

In `src-tauri/crates/extractum-core/Cargo.toml`, add:

```toml
serde_json.workspace = true
```

Do not change any version or unrelated dependency.

- [ ] **Step 4: Create the complete pure core module**

Create `src-tauri/crates/extractum-core/src/media_metadata.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::compression::{compress_json_bytes, decompress_bytes};
use crate::error::{AppError, AppResult};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ItemMediaMetadata {
    pub summary: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_seconds: Option<f64>,
}

pub fn encode_media_metadata(metadata: &ItemMediaMetadata) -> AppResult<Vec<u8>> {
    let json =
        serde_json::to_vec(metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

pub fn decode_media_metadata(bytes: Option<&[u8]>) -> AppResult<ItemMediaMetadata> {
    let Some(bytes) = bytes else {
        return Ok(ItemMediaMetadata::default());
    };
    let decoded = decompress_bytes(bytes).map_err(AppError::internal)?;
    serde_json::from_slice(&decoded).map_err(|error| AppError::internal(error.to_string()))
}

pub fn media_label(kind: &str) -> &'static str {
    match kind {
        "photo" => "Photo",
        "video" => "Video",
        "audio" => "Audio",
        "voice" => "Voice message",
        "image" => "Image",
        "animation" => "Animation",
        "sticker" => "Sticker",
        "contact" => "Contact card",
        "poll" => "Poll",
        "location" => "Location",
        "live_location" => "Live location",
        "venue" => "Venue",
        "webpage" => "Web page preview",
        "dice" => "Dice",
        _ => "Document",
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppErrorKind;

    use super::{
        decode_media_metadata, encode_media_metadata, media_label, ItemMediaMetadata,
    };

    #[test]
    fn media_label_covers_known_and_fallback_kinds() {
        assert_eq!(media_label("photo"), "Photo");
        assert_eq!(media_label("live_location"), "Live location");
        assert_eq!(media_label("unknown"), "Document");
    }

    #[test]
    fn media_metadata_roundtrip_through_zstd() {
        let original = ItemMediaMetadata {
            summary: Some("Video".to_string()),
            file_name: Some("clip.mp4".to_string()),
            mime_type: Some("video/mp4".to_string()),
            size_bytes: Some(42),
            width: Some(1920),
            height: Some(1080),
            duration_seconds: Some(12.5),
        };

        let encoded = encode_media_metadata(&original).expect("encode");
        let decoded = decode_media_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded, original);
    }

    #[test]
    fn media_metadata_decode_failures_are_typed_internal_errors() {
        let error = decode_media_metadata(Some(&[0x00])).expect_err("reject corrupt metadata");

        assert_eq!(error.kind, AppErrorKind::Internal);
    }

    #[test]
    fn absent_media_metadata_decodes_to_default() {
        let decoded = decode_media_metadata(None).expect("decode absent metadata");

        assert_eq!(decoded, ItemMediaMetadata::default());
    }
}
```

This is a mechanical move of the existing struct, codec, label function, and
three pure tests plus the Task 1 characterization test. Do not alter field
order, derives, labels, compression, or error mapping.

- [ ] **Step 5: Expose the curated module from core**

In `src-tauri/crates/extractum-core/src/lib.rs`, keep the explicit alphabetical
module list:

```rust
pub mod compression;
pub mod error;
pub mod media_metadata;
pub mod time;
```

- [ ] **Step 6: Replace app definitions with the compatibility facade**

At the top of `src-tauri/src/media.rs`:

1. remove `use serde::{Deserialize, Serialize};`;
2. remove the `crate::compression` and `crate::error` imports;
3. add this explicit facade after the grammers import:

```rust
pub(crate) use extractum_core::media_metadata::{
    decode_media_metadata, encode_media_metadata, media_label, ItemMediaMetadata,
};
```

Delete only these production definitions from the app module:

- `ItemMediaMetadata`;
- `encode_media_metadata`;
- `decode_media_metadata`;
- `media_label`.

Keep all payload types, constants, grammers logic, and calls to those four
names unchanged; they resolve through the facade.

- [ ] **Step 7: Move the four pure tests out of the app test module**

Delete these test functions from `src-tauri/src/media.rs` because their exact
implementations now live in core:

```text
media_label_covers_known_and_fallback_kinds
media_metadata_roundtrip_through_zstd
media_metadata_decode_failures_are_typed_internal_errors
absent_media_metadata_decodes_to_default
```

Replace the app test module's shared import with the exact remaining adapter
surface:

```rust
use super::{
    derive_content_kind, derive_document_media_kind, DocumentSignals,
    CONTENT_KIND_MEDIA_ONLY, CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA,
};
```

The two remaining tests and their bodies must remain unchanged.

- [ ] **Step 8: Refresh Cargo metadata and lock data**

Run:

```powershell
$metadata = cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$object = $metadata | ConvertFrom-Json
$core = $object.packages | Where-Object name -eq 'extractum-core'
"MEMBERS=$(@($object.workspace_members).Count)"
"TARGET=$($object.target_directory)"
"CORE_DEPS=$((@($core.dependencies.name | Sort-Object) -join ','))"
```

Expected: two workspace members, canonical `src-tauri/target`, and core roots
exactly `serde,serde_json,time,zstd`. `Cargo.lock` may add `serde_json` to the
local core dependency list but must not change an external version.

- [ ] **Step 9: Format and run focused Rust tests**

Run:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-core --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-core --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib media::tests
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib notebooklm_export
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib sources
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib takeout_import
```

Expected: core has 22 tests (the previous 18 plus four media metadata tests),
app `media::tests` has exactly two tests, and every consumer filter executes a
nonzero passing set. Inspect each command's output; exit 0 with zero matched
tests is a failure of this step.

- [ ] **Step 10: Run the contract to verify GREEN**

Run:

```powershell
node scripts/run-vitest.mjs run src/lib/media-metadata-core-contract.test.ts
```

Expected: one file, five tests, all pass.

---

### Task 3: Prove Inventory, Correctness, and Application Startup

**Files:**
- Read: temporary baseline evidence
- Review: all Task 2 implementation files

**Interfaces:**
- Verifies the rename map and exact net test increase.
- Produces the implementation checkpoint used by the verification record.

- [ ] **Step 1: Compare the complete post-workspace inventory**

Run:

```powershell
$locator = Join-Path $env:TEMP 'extractum-media-metadata-current.txt'
if (-not (Test-Path -LiteralPath $locator)) { throw 'Scratch locator missing' }
$scratch = (Get-Content -LiteralPath $locator -Raw).Trim()
$baseline = @(Get-Content -LiteralPath (Join-Path $scratch 'baseline-test-names.txt'))
$postLog = Join-Path $scratch 'post-inventory.log'
& cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- --list 2>&1 |
  Tee-Object -FilePath $postLog
$exit = $LASTEXITCODE
$post = @(
  Get-Content -LiteralPath $postLog |
    ForEach-Object { $_.ToString() } |
    Where-Object { $_ -match ': test$' } |
    ForEach-Object { ($_ -replace ': test$', '').Trim() }
)
$renameMap = @{
  'media::tests::media_label_covers_known_and_fallback_kinds' = 'media_metadata::tests::media_label_covers_known_and_fallback_kinds'
  'media::tests::media_metadata_roundtrip_through_zstd' = 'media_metadata::tests::media_metadata_roundtrip_through_zstd'
  'media::tests::media_metadata_decode_failures_are_typed_internal_errors' = 'media_metadata::tests::media_metadata_decode_failures_are_typed_internal_errors'
}
$missing = @(
  foreach ($name in $baseline) {
    $expected = if ($renameMap.ContainsKey($name)) { $renameMap[$name] } else { $name }
    if ($expected -notin $post) { $name }
  }
)
$required = @(
  'media::tests::derive_content_kind_tracks_text_and_media_presence'
  'media::tests::derive_document_media_kind_prefers_specific_signals'
  'media_metadata::tests::absent_media_metadata_decodes_to_default'
)
$missingRequired = @($required | Where-Object { $_ -notin $post })
$uniquePost = @($post | Sort-Object -Unique)
@{
  exit = $exit
  baseline_count = $baseline.Count
  post_count = $post.Count
  missing_count = $missing.Count
  missing_required_count = $missingRequired.Count
  unique_post_count = $uniquePost.Count
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $scratch 'post-inventory.json')
if ($exit -ne 0 -or
    $post.Count -ne ($baseline.Count + 1) -or
    $uniquePost.Count -ne $post.Count -or
    $missing.Count -ne 0 -or
    $missingRequired.Count -ne 0) { exit 1 }
```

Expected from the current baseline: 1125 becomes exactly 1126. The three
declared old names are replaced by their mapped core names, both adapter tests
remain, the new absent-blob test exists only in core, and no other name is
missing.

- [ ] **Step 2: Run complete Rust and repository gates**

Run:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Expected: all commands pass. Complete Cargo output includes both packages and
1126 Rust tests. Existing unrelated release-only warnings are not new failures;
the files owned by this slice must add no warning.

- [ ] **Step 3: Verify dependency and source boundaries mechanically**

Run:

```powershell
$metadata = cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps |
  ConvertFrom-Json
$core = $metadata.packages | Where-Object name -eq 'extractum-core'
$coreDeps = @($core.dependencies.name | Sort-Object)
$expected = @('serde', 'serde_json', 'time', 'zstd')
$memberProfiles = @(
  Get-ChildItem src-tauri/crates -Recurse -Filter Cargo.toml |
    Select-String -Pattern '^\[profile\.'
)
$forbiddenCore = @(
  Get-ChildItem src-tauri/crates/extractum-core -Recurse -File |
    Select-String -Pattern 'grammers|tauri|sqlx'
)
"CORE_DEPS=$($coreDeps -join ',')"
"MEMBER_PROFILES=$($memberProfiles.Count)"
"FORBIDDEN_CORE=$($forbiddenCore.Count)"
if (($coreDeps -join ',') -ne ($expected -join ',') -or
    $memberProfiles.Count -ne 0 -or
    $forbiddenCore.Count -ne 0) { exit 1 }
```

Expected: exact dependency roots, no member profiles, no heavyweight core
dependency. Review `Cargo.lock` and fail on any external version change.

- [ ] **Step 4: Build the release executable without WiX**

Run:

```powershell
npm.cmd run tauri -- build --no-bundle
```

Expected: exit 0 and
`src-tauri/target/release/extractum.exe` is produced. Do not interpret build
duration as a performance result for this architectural slice.

- [ ] **Step 5: Run the release startup smoke**

Launch `src-tauri/target/release/extractum.exe` visibly. Confirm that the main
window renders, perform one ordinary navigation action, close it normally, and
confirm no `extractum` process remains.

Expected: startup, navigation, and normal shutdown succeed. If desktop
automation cannot perform navigation, record that limitation and require a
human observation rather than claiming the unperformed action passed.

- [ ] **Step 6: Review the implementation diff and exact scope**

Run:

```powershell
git diff --check
git status --short --untracked-files=all
git diff --stat
git diff -- src-tauri/Cargo.lock
```

The implementation diff must be limited to:

```text
src/lib/media-metadata-core-contract.test.ts
src-tauri/Cargo.lock
src-tauri/Cargo.toml
src-tauri/crates/extractum-core/Cargo.toml
src-tauri/crates/extractum-core/src/lib.rs
src-tauri/crates/extractum-core/src/media_metadata.rs
src-tauri/src/media.rs
```

Confirm the four pure production items and four tests moved mechanically, the
seven fields alone widened from `pub(crate)` to `pub`, the two adapter tests are
unchanged, no consumer imports changed, and no probe or build artifact is
tracked.

- [ ] **Step 7: Commit the verified implementation**

Run:

```powershell
git add -- src/lib/media-metadata-core-contract.test.ts `
  src-tauri/Cargo.lock `
  src-tauri/Cargo.toml `
  src-tauri/crates/extractum-core/Cargo.toml `
  src-tauri/crates/extractum-core/src/lib.rs `
  src-tauri/crates/extractum-core/src/media_metadata.rs `
  src-tauri/src/media.rs
git diff --cached --check
git commit -m "refactor: move media metadata into core"
git status --short
```

Expected: commit succeeds, only the listed files are included, and the tree is
clean. Record the commit hash. Do not push.

---

### Task 4: Record Verification Evidence

**Files:**
- Create:
  `docs/superpowers/verification/2026-07-16-media-metadata-core-boundary.md`

**Interfaces:**
- Consumes: baseline/post inventory, implementation commit, command results,
  dependency evidence, and release smoke result.
- Produces: the permanent verification record for the architectural slice.

- [ ] **Step 1: Write the verification record**

Create the document with these sections and actual observed values:

```markdown
# Media Metadata Core Boundary Verification

## Scope and Commits
## Dependency Boundary
## Test Inventory and Rename Map
## Focused and Complete Verification
## Release Build and Startup Smoke
## Limitations
## Result and Follow-Up
```

It must record:

- starting, characterization, and implementation commit hashes;
- exact baseline and post test counts;
- all three declared test renames, the two retained adapter tests, and the new
  absent-blob test;
- `extractum-core`'s exact four dependency roots;
- source-contract RED and GREEN outcomes;
- focused consumer, full workspace, repository, and no-bundle build results;
- release startup/navigation/shutdown evidence, including any manual-only
  limitation;
- that no performance threshold applies to this slice;
- that the next proposed slice is the fresh dependency map and design for the
  smallest pure `notebooklm_export` crate.

- [ ] **Step 2: Run final hygiene with evidence as the only change**

Run:

```powershell
npm.cmd run check:rustfmt
npm.cmd run verify
git diff --check
$status = @(git status --short --untracked-files=all)
$status
if ($status.Count -ne 1 -or
    $status[0] -notmatch '^\?\? docs/superpowers/verification/2026-07-16-media-metadata-core-boundary\.md$') {
  exit 1
}
```

Expected: all gates pass and the only uncommitted path is the verification
document. No temporary inventory, logs, build artifacts, or source edits are
inside the repository.

- [ ] **Step 3: Review and commit the verification document**

Run:

```powershell
git add -- docs/superpowers/verification/2026-07-16-media-metadata-core-boundary.md
git diff --cached --check
git diff --cached --stat
git commit -m "docs: record media metadata core verification"
git status --short
```

Expected: one documentation file is committed and the worktree is clean. Do
not push unless the user asks.
