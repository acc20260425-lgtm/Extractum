[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$InstallDirectory,

    [switch]$AddToGitHubPath,

    [switch]$PrependToProcessPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repositoryRoot '.github/tools/cargo-deny.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

if ($manifest.schemaVersion -ne 1) {
    throw "Unsupported cargo-deny manifest schema version: $($manifest.schemaVersion)"
}

$resolvedInstallDirectory = [System.IO.Path]::GetFullPath($InstallDirectory)
New-Item -ItemType Directory -Path $resolvedInstallDirectory -Force | Out-Null

$archivePath = Join-Path $resolvedInstallDirectory 'cargo-deny.tar.gz'
$extractDirectory = Join-Path $resolvedInstallDirectory 'extract'
New-Item -ItemType Directory -Path $extractDirectory -Force | Out-Null

Invoke-WebRequest -Uri $manifest.url -OutFile $archivePath
$actualSha256 = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualSha256 -ne $manifest.sha256.ToLowerInvariant()) {
    throw "cargo-deny checksum mismatch: expected $($manifest.sha256), got $actualSha256"
}

tar -xzf $archivePath -C $extractDirectory
if ($LASTEXITCODE -ne 0) {
    throw "Failed to extract cargo-deny archive (exit code $LASTEXITCODE)."
}

$extractedBinary = Get-ChildItem -LiteralPath $extractDirectory -Filter 'cargo-deny.exe' -File -Recurse | Select-Object -First 1
if ($null -eq $extractedBinary) {
    throw 'The verified cargo-deny archive did not contain cargo-deny.exe.'
}

$installedBinary = Join-Path $resolvedInstallDirectory 'cargo-deny.exe'
Copy-Item -LiteralPath $extractedBinary.FullName -Destination $installedBinary -Force

if ($AddToGitHubPath) {
    if ([string]::IsNullOrWhiteSpace($env:GITHUB_PATH)) {
        throw 'GITHUB_PATH is required when -AddToGitHubPath is specified.'
    }

    Add-Content -LiteralPath $env:GITHUB_PATH -Value $resolvedInstallDirectory
}

if ($PrependToProcessPath) {
    $env:PATH = "$resolvedInstallDirectory$([System.IO.Path]::PathSeparator)$env:PATH"
}

& $installedBinary --version
if ($LASTEXITCODE -ne 0) {
    throw "Pinned cargo-deny failed its version check (exit code $LASTEXITCODE)."
}
