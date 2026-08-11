# Grammers crates.io 0.10.0 Provenance Verification

**Date:** 2026-08-11

## Purpose

Verify that all eight crates.io artifacts in Extractum's resolved Grammers
graph originate from the current Codeberg pin
`5c6d44ff30e02d6c9295bcf1fcb51403ad77c981`. This evidence supports retaining
the live Telegram and Takeout smoke record from the preceding upgrade.

## Inputs

Cargo populated these cache locations before comparison:

- registry artifacts under
  `~/.cargo/registry/src/index.crates.io-*/grammers-*`;
- Codeberg checkout:
  `~/.cargo/git/checkouts/grammers-*/5c6d44f`.

Toolchain used:

```text
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
rustc 1.95.0 (59807616e 2026-04-14)
```

Recorded with:

```powershell
cargo --version
rustc --version
```

The eight registry artifacts were fetched with:

```powershell
cargo info grammers-client@0.10.0
cargo info grammers-crypto@0.10.0
cargo info grammers-mtproto@0.10.0
cargo info grammers-mtsender@0.10.0
cargo info grammers-session@0.10.0
cargo info grammers-tl-gen@0.10.0
cargo info grammers-tl-parser@1.2.2
cargo info grammers-tl-types@0.10.0
```

## Recorded Git Identity

The checkout identity was read with:

```powershell
git -c safe.directory=<checkout> -C <checkout> rev-parse HEAD
```

Observed output:

```text
5c6d44ff30e02d6c9295bcf1fcb51403ad77c981
```

Each artifact identity was read from its
`<crate-name>-<resolved-version>/.cargo_vcs_info.json`:

```powershell
Get-Content <registry-crate>/.cargo_vcs_info.json
```

Observed for all eight crates:

```json
{
  "git": {
    "sha1": "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981"
  },
  "path_in_vcs": "<crate-name>"
}
```

## File Hash Comparison

This PowerShell procedure discovers the populated cache roots, excludes
normalized or generated Cargo packaging files, and compares every remaining
shared file by relative path and SHA-256 hash:

Published `Cargo.toml` files differ by design: Cargo normalizes manifests and
rewrites path dependencies into registry version requirements during packaging.
The procedure therefore excludes normalized `Cargo.toml` and the retained
`Cargo.toml.orig` rather than treating this expected transformation as source
drift.

```powershell
$provenancePin = "5c6d44ff30e02d6c9295bcf1fcb51403ad77c981"
$provenanceCheckout = Get-ChildItem `
  "$env:USERPROFILE/.cargo/git/checkouts/grammers-*/5c6d44f" -Directory |
  Where-Object {
    git -c "safe.directory=$($_.FullName.Replace('\', '/'))" `
      -C $_.FullName rev-parse HEAD 2>$null | Where-Object { $_ -eq $provenancePin }
  } | Select-Object -First 1 -ExpandProperty FullName
$provenanceRegistry = Get-ChildItem `
  "$env:USERPROFILE/.cargo/registry/src" -Directory |
  Where-Object {
    Test-Path (Join-Path $_.FullName "grammers-client-0.10.0")
  } | Select-Object -First 1 -ExpandProperty FullName
$provenanceVersions = [ordered]@{
  "grammers-client" = "0.10.0"
  "grammers-crypto" = "0.10.0"
  "grammers-mtproto" = "0.10.0"
  "grammers-mtsender" = "0.10.0"
  "grammers-session" = "0.10.0"
  "grammers-tl-gen" = "0.10.0"
  "grammers-tl-parser" = "1.2.2"
  "grammers-tl-types" = "0.10.0"
}

foreach ($entry in $provenanceVersions.GetEnumerator()) {
  $provenanceCrate = $entry.Key
  $provenanceVersion = $entry.Value
  $gitRoot = Join-Path $provenanceCheckout $provenanceCrate
  $registryRoot = Join-Path `
    $provenanceRegistry "$provenanceCrate-$provenanceVersion"
  $vcs = Get-Content (Join-Path $registryRoot ".cargo_vcs_info.json") -Raw |
    ConvertFrom-Json
  $gitMap = @{}
  Get-ChildItem $gitRoot -Recurse -File |
    Where-Object { $_.Name -notin @("Cargo.toml", "Cargo.lock") } |
    ForEach-Object {
      $relative = $_.FullName.Substring($gitRoot.Length + 1).Replace("\", "/")
      $gitMap[$relative] = (Get-FileHash $_.FullName -Algorithm SHA256).Hash
    }
  $registryMap = @{}
  Get-ChildItem $registryRoot -Recurse -File |
    Where-Object {
      $_.Name -notin @(
        "Cargo.toml",
        "Cargo.lock",
        ".cargo_vcs_info.json",
        "Cargo.toml.orig",
        ".cargo-ok"
      )
    } | ForEach-Object {
      $relative = $_.FullName.Substring($registryRoot.Length + 1).Replace("\", "/")
      $registryMap[$relative] = (Get-FileHash $_.FullName -Algorithm SHA256).Hash
    }
  $common = @($gitMap.Keys | Where-Object { $registryMap.ContainsKey($_) })
  $hashDrift = @($common | Where-Object { $gitMap[$_] -ne $registryMap[$_] })
  $gitOnly = @($gitMap.Keys | Where-Object { -not $registryMap.ContainsKey($_) })
  $registryOnly = @(
    $registryMap.Keys | Where-Object { -not $gitMap.ContainsKey($_) }
  )
  Write-Output (
    "$provenanceCrate@$provenanceVersion vcs=$($vcs.git.sha1) " +
    "common=$($common.Count) " +
    "hash_drift=$($hashDrift.Count) checkout_only=$($gitOnly.Count) " +
    "registry_only=$($registryOnly.Count)"
  )
  if ($gitOnly.Count) { Write-Output ($gitOnly | Sort-Object) }
  if ($registryOnly.Count) { Write-Output ($registryOnly | Sort-Object) }
}
```

Observed output after the packaging exclusions:

```text
grammers-client@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=58 hash_drift=0 checkout_only=0 registry_only=0
grammers-crypto@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=15 hash_drift=0 checkout_only=0 registry_only=0
grammers-mtproto@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=15 hash_drift=0 checkout_only=0 registry_only=0
grammers-mtsender@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=13 hash_drift=0 checkout_only=0 registry_only=0
grammers-session@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=17 hash_drift=0 checkout_only=0 registry_only=0
grammers-tl-gen@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=9 hash_drift=0 checkout_only=0 registry_only=0
grammers-tl-parser@1.2.2 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=14 hash_drift=0 checkout_only=0 registry_only=0
grammers-tl-types@0.10.0 vcs=5c6d44ff30e02d6c9295bcf1fcb51403ad77c981 common=8 hash_drift=0 checkout_only=2 registry_only=0
DEPS.md
tests/deps.rs
```

These two checkout-only files are dependency-maintenance test support and do
not contribute to the published library runtime.

## Result

All eight crates.io artifacts record the existing pinned revision, and every
shared published file is byte-identical to that checkout. No runtime source
drift was found.
