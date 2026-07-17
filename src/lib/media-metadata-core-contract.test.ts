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
