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
$resolvedGitHubPath = $null
$githubPathExisted = $false
$githubPathOriginalLength = 0L
$processPathOriginal = $env:PATH

if ($AddToGitHubPath) {
    if ([string]::IsNullOrWhiteSpace($env:GITHUB_PATH)) {
        throw 'GITHUB_PATH is required when -AddToGitHubPath is specified.'
    }

    $resolvedGitHubPath = [System.IO.Path]::GetFullPath($env:GITHUB_PATH)
    $githubPathParent = [System.IO.Path]::GetDirectoryName($resolvedGitHubPath)
    if ([string]::IsNullOrWhiteSpace($githubPathParent) -or -not (Test-Path -LiteralPath $githubPathParent -PathType Container)) {
        throw "GITHUB_PATH parent directory does not exist: $githubPathParent"
    }

    $githubPathExisted = Test-Path -LiteralPath $resolvedGitHubPath -PathType Leaf
    if ($githubPathExisted) {
        $githubPathOriginalLength = (Get-Item -LiteralPath $resolvedGitHubPath).Length
    }

    $githubPathStream = $null
    try {
        $githubPathStream = [System.IO.File]::Open(
            $resolvedGitHubPath,
            [System.IO.FileMode]::OpenOrCreate,
            [System.IO.FileAccess]::Write,
            [System.IO.FileShare]::ReadWrite
        )
    } finally {
        if ($null -ne $githubPathStream) {
            $githubPathStream.Dispose()
        }
    }
}

New-Item -ItemType Directory -Path $resolvedInstallDirectory -Force | Out-Null

$installParent = [System.IO.Path]::GetDirectoryName($resolvedInstallDirectory.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar))
if ([string]::IsNullOrWhiteSpace($installParent)) {
    $installParent = [System.IO.Path]::GetTempPath()
}

$transactionId = [guid]::NewGuid().ToString('N')
$stagingDirectory = Join-Path $installParent ".cargo-deny-staging-$transactionId"
$archivePath = Join-Path $stagingDirectory 'cargo-deny.tar.gz'
$extractDirectory = Join-Path $stagingDirectory 'extract'
$installedBinary = Join-Path $resolvedInstallDirectory 'cargo-deny.exe'
$backupBinary = Join-Path $resolvedInstallDirectory "cargo-deny.exe.backup-$transactionId"
$installStarted = $false
$backupCreated = $false
$preserveBackup = $false

try {
    New-Item -ItemType Directory -Path $extractDirectory -Force | Out-Null

    $maximumDownloadAttempts = 3
    for ($attempt = 1; $attempt -le $maximumDownloadAttempts; $attempt++) {
        Remove-Item -LiteralPath $archivePath -Force -ErrorAction SilentlyContinue
        try {
            Invoke-WebRequest -Uri $manifest.url -OutFile $archivePath
            break
        } catch {
            if ($attempt -eq $maximumDownloadAttempts) {
                throw "Failed to download cargo-deny after $maximumDownloadAttempts attempts: $($_.Exception.Message)"
            }

            Start-Sleep -Seconds $attempt
        }
    }

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

    $stagedVersion = (& $extractedBinary.FullName --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "Staged cargo-deny failed its version check (exit code $LASTEXITCODE)."
    }
    $expectedVersion = "cargo-deny $($manifest.version)"
    if ($stagedVersion -ne $expectedVersion) {
        throw "Staged cargo-deny version mismatch: expected '$expectedVersion', got '$stagedVersion'."
    }
    Write-Output $stagedVersion

    if (Test-Path -LiteralPath $installedBinary -PathType Leaf) {
        Copy-Item -LiteralPath $installedBinary -Destination $backupBinary
        $backupCreated = $true
    }

    $installStarted = $true
    Copy-Item -LiteralPath $extractedBinary.FullName -Destination $installedBinary -Force

    if ($AddToGitHubPath) {
        Add-Content -LiteralPath $resolvedGitHubPath -Value $resolvedInstallDirectory
    }

    if ($PrependToProcessPath) {
        $env:PATH = "$resolvedInstallDirectory$([System.IO.Path]::PathSeparator)$env:PATH"
    }
} catch {
    $installError = $_
    $rollbackErrors = [System.Collections.Generic.List[string]]::new()

    if ($PrependToProcessPath) {
        $env:PATH = $processPathOriginal
    }

    if ($AddToGitHubPath) {
        try {
            if ($githubPathExisted) {
                $githubPathStream = [System.IO.File]::Open(
                    $resolvedGitHubPath,
                    [System.IO.FileMode]::Open,
                    [System.IO.FileAccess]::Write,
                    [System.IO.FileShare]::ReadWrite
                )
                try {
                    $githubPathStream.SetLength($githubPathOriginalLength)
                } finally {
                    $githubPathStream.Dispose()
                }
            } else {
                Remove-Item -LiteralPath $resolvedGitHubPath -Force -ErrorAction SilentlyContinue
            }
        } catch {
            $rollbackErrors.Add("GITHUB_PATH rollback failed: $($_.Exception.Message)")
        }
    }

    if ($installStarted) {
        Remove-Item -LiteralPath $installedBinary -Force -ErrorAction SilentlyContinue
        if ($backupCreated) {
            try {
                Move-Item -LiteralPath $backupBinary -Destination $installedBinary -Force
                $backupCreated = $false
            } catch {
                $preserveBackup = $true
                $rollbackErrors.Add("binary rollback failed; the previous binary is preserved at '$backupBinary': $($_.Exception.Message)")
            }
        }
    }

    if ($rollbackErrors.Count -ne 0) {
        throw "cargo-deny installation failed: $($installError.Exception.Message). $($rollbackErrors -join ' ')"
    }

    throw $installError
} finally {
    Remove-Item -LiteralPath $stagingDirectory -Recurse -Force -ErrorAction SilentlyContinue
    if (-not $preserveBackup) {
        Remove-Item -LiteralPath $backupBinary -Force -ErrorAction SilentlyContinue
    }
}
