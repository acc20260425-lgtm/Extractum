[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Setup', 'Deterministic', 'Advisories')]
    [string]$Mode,

    [switch]$AddToGitHubPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Test-Digest {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Expected
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $false
    }

    $stream = [System.IO.File]::OpenRead($Path)
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        $actual = [System.BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-', '').ToLowerInvariant()
        return $actual -ceq $Expected
    } finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Test-AuthenticatedBinary {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][pscustomobject]$Manifest
    )

    if (-not (Test-Digest -Path $Path -Expected $Manifest.binarySha256)) {
        return $false
    }

    $version = (& $Path --version | Out-String).Trim()
    return $LASTEXITCODE -eq 0 -and $version -ceq "cargo-deny $($Manifest.version)"
}

function Test-CargoDenyTestSeam {
    param([Parameter(Mandatory = $true)][string]$Name)

    return $env:EXTRACTUM_CARGO_DENY_TESTING -ceq '1' -and [Environment]::GetEnvironmentVariable($Name, 'Process') -ceq '1'
}

function Copy-OrDownloadArchive {
    param(
        [Parameter(Mandatory = $true)][pscustomobject]$Manifest,
        [Parameter(Mandatory = $true)][string]$Destination
    )

    $testArchive = [Environment]::GetEnvironmentVariable('EXTRACTUM_CARGO_DENY_TEST_ARCHIVE', 'Process')
    if ($env:EXTRACTUM_CARGO_DENY_TESTING -ceq '1' -and -not [string]::IsNullOrWhiteSpace($testArchive)) {
        Copy-Item -LiteralPath $testArchive -Destination $Destination -Force
        return
    }

    Invoke-WebRequest -Uri $Manifest.url -OutFile $Destination
}

function Add-GitHubPathOnce {
    param([Parameter(Mandatory = $true)][string]$Path)

    if ([string]::IsNullOrWhiteSpace($env:GITHUB_PATH)) {
        throw 'GITHUB_PATH is required when -AddToGitHubPath is specified.'
    }

    $normalizedPath = [System.IO.Path]::GetFullPath($Path)
    $githubPath = [System.IO.Path]::GetFullPath($env:GITHUB_PATH)
    $parent = [System.IO.Path]::GetDirectoryName($githubPath)
    if ([string]::IsNullOrWhiteSpace($parent) -or -not (Test-Path -LiteralPath $parent -PathType Container)) {
        throw "GITHUB_PATH parent directory does not exist: $parent"
    }

    $existing = @()
    if (Test-Path -LiteralPath $githubPath -PathType Leaf) {
        $existing = @(Get-Content -LiteralPath $githubPath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    }
    foreach ($line in $existing) {
        if ([System.IO.Path]::GetFullPath($line.Trim()) -ceq $normalizedPath) {
            return
        }
    }

    Add-Content -LiteralPath $githubPath -Value $normalizedPath
}

function Get-AuthenticatedCargoDeny {
    param(
        [Parameter(Mandatory = $true)][pscustomobject]$Manifest,
        [Parameter(Mandatory = $true)][string]$StableBinary
    )

    $stableDirectory = Split-Path -Parent $StableBinary
    $cacheDirectory = Split-Path -Parent $stableDirectory
    $forceReinstall = Test-CargoDenyTestSeam -Name 'EXTRACTUM_CARGO_DENY_TEST_FORCE_REINSTALL'
    if (-not $forceReinstall -and (Test-AuthenticatedBinary -Path $StableBinary -Manifest $Manifest)) {
        return [System.IO.Path]::GetFullPath($StableBinary)
    }

    New-Item -ItemType Directory -Path $cacheDirectory -Force | Out-Null
    $transactionId = [guid]::NewGuid().ToString('N')
    $stageDirectory = Join-Path $cacheDirectory ".stage-$transactionId"
    $archivePath = Join-Path $stageDirectory 'cargo-deny.tar.gz'
    $extractDirectory = Join-Path $stageDirectory 'extract'
    $candidateDirectory = Join-Path $stageDirectory 'candidate'
    $candidateBinary = Join-Path $candidateDirectory 'cargo-deny.exe'
    $backupDirectory = Join-Path $cacheDirectory ".backup-$transactionId"
    $backupCreated = $false
    $preserveBackup = $false
    $published = $false

    try {
        New-Item -ItemType Directory -Path $extractDirectory, $candidateDirectory -Force | Out-Null
        Copy-OrDownloadArchive -Manifest $Manifest -Destination $archivePath
        if (-not (Test-Digest -Path $archivePath -Expected $Manifest.sha256)) {
            throw 'cargo-deny archive checksum mismatch.'
        }

        & tar.exe -xzf $archivePath -C $extractDirectory
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to extract cargo-deny archive (exit code $LASTEXITCODE)."
        }

        $extractedBinary = Get-ChildItem -LiteralPath $extractDirectory -Filter 'cargo-deny.exe' -File -Recurse | Select-Object -First 1
        if ($null -eq $extractedBinary) {
            throw 'The verified cargo-deny archive did not contain cargo-deny.exe.'
        }
        Copy-Item -LiteralPath $extractedBinary.FullName -Destination $candidateBinary -Force
        if (-not (Test-AuthenticatedBinary -Path $candidateBinary -Manifest $Manifest)) {
            throw 'Staged cargo-deny binary authentication failed.'
        }

        if (Test-CargoDenyTestSeam -Name 'EXTRACTUM_CARGO_DENY_TEST_FAIL_PRE_PUBLISH') {
            throw 'Test pre-publish failure.'
        }

        if (Test-Path -LiteralPath $stableDirectory) {
            Move-Item -LiteralPath $stableDirectory -Destination $backupDirectory
            $backupCreated = $true
        }
        Move-Item -LiteralPath $candidateDirectory -Destination $stableDirectory
        $published = $true

        if (Test-CargoDenyTestSeam -Name 'EXTRACTUM_CARGO_DENY_TEST_FAIL_POST_PUBLISH') {
            throw 'Test post-publish failure.'
        }

        if ($backupCreated) {
            Remove-Item -LiteralPath $backupDirectory -Recurse -Force -ErrorAction SilentlyContinue
        }
        return [System.IO.Path]::GetFullPath($StableBinary)
    } catch {
        $installError = $_
        if ($published -and (Test-Path -LiteralPath $stableDirectory)) {
            Remove-Item -LiteralPath $stableDirectory -Recurse -Force -ErrorAction SilentlyContinue
        }
        if ($backupCreated -and (Test-Path -LiteralPath $backupDirectory)) {
            try {
                Move-Item -LiteralPath $backupDirectory -Destination $stableDirectory -Force
                $backupCreated = $false
            } catch {
                $preserveBackup = $true
                throw "cargo-deny installation failed: $($installError.Exception.Message). Previous stable cache is preserved at '$backupDirectory'."
            }
        }
        throw $installError
    } finally {
        Remove-Item -LiteralPath $stageDirectory -Recurse -Force -ErrorAction SilentlyContinue
        if ($backupCreated -and -not $preserveBackup) {
            Remove-Item -LiteralPath $backupDirectory -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repositoryRoot '.github/tools/cargo-deny.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

if ($manifest.schemaVersion -ne 1) {
    throw "Unsupported cargo-deny manifest schema version: $($manifest.schemaVersion)"
}
if ($manifest.version -notmatch '^\d+\.\d+\.\d+$') {
    throw "Invalid cargo-deny manifest version: $($manifest.version)"
}
if ($manifest.target -cne 'x86_64-pc-windows-msvc') {
    throw "Unsupported cargo-deny target: $($manifest.target)"
}
if ($manifest.sha256 -notmatch '^[0-9a-f]{64}$' -or $manifest.binarySha256 -notmatch '^[0-9a-f]{64}$') {
    throw 'cargo-deny manifest digests must be lowercase SHA-256 values.'
}

if ($AddToGitHubPath -and $Mode -cne 'Setup') {
    throw '-AddToGitHubPath is only valid in Setup mode.'
}

$stableDirectory = Join-Path $repositoryRoot "src-tauri/target/.extractum-tools/cargo-deny/$($manifest.version)-$($manifest.target)"
$stableBinary = Join-Path $stableDirectory 'cargo-deny.exe'
$authenticatedBinary = Get-AuthenticatedCargoDeny -Manifest $manifest -StableBinary $stableBinary

if ($Mode -ceq 'Setup') {
    if ($AddToGitHubPath) {
        Add-GitHubPathOnce -Path $stableDirectory
    }
    exit 0
}

if (-not (Test-AuthenticatedBinary -Path $authenticatedBinary -Manifest $manifest)) {
    throw 'Stable cargo-deny binary authentication failed before execution.'
}

$DenyToml = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot 'deny.toml'))
$CargoToml = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot 'src-tauri/Cargo.toml'))
$Common = @('--config', $DenyToml, '--manifest-path', $CargoToml, '--locked', 'check')
$DeterministicChecks = @('bans', 'licenses', 'sources')
$AdvisoryChecks = @('advisories')

if ($Mode -ceq 'Deterministic') {
    & $authenticatedBinary @Common @DeterministicChecks
    exit $LASTEXITCODE
}

& $authenticatedBinary @Common @AdvisoryChecks
exit $LASTEXITCODE
