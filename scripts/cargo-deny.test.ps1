[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Script:RepositoryRoot = Split-Path -Parent $PSScriptRoot
$Script:FakeExecutable = $null

function Assert-True {
    param(
        [Parameter(Mandatory = $true)][bool]$Condition,
        [Parameter(Mandatory = $true)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Assert-Equal {
    param(
        [Parameter(Mandatory = $true)]$Expected,
        [Parameter(Mandatory = $true)]$Actual,
        [Parameter(Mandatory = $true)][string]$Message
    )

    if ($Expected -cne $Actual) {
        throw "$Message Expected '$Expected', got '$Actual'."
    }
}

function Invoke-Runner {
    param(
        [Parameter(Mandatory = $true)][string]$Runner,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][hashtable]$Environment
    )

    $originalEnvironment = @{}
    foreach ($key in $Environment.Keys) {
        $originalEnvironment[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
    }

    try {
        foreach ($key in $Environment.Keys) {
            [Environment]::SetEnvironmentVariable($key, [string]$Environment[$key], 'Process')
        }

        $argumentList = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $Runner) + $Arguments
        $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = 'powershell.exe'
        $startInfo.UseShellExecute = $false
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        $startInfo.Arguments = (($argumentList | ForEach-Object {
            '"' + $_.Replace('"', '\"') + '"'
        }) -join ' ')

        $process = [System.Diagnostics.Process]::new()
        $process.StartInfo = $startInfo
        [void]$process.Start()
        $stdout = $process.StandardOutput.ReadToEnd()
        $stderr = $process.StandardError.ReadToEnd()
        $process.WaitForExit()

        return [pscustomobject]@{
            ExitCode = $process.ExitCode
            Stdout = $stdout
            Stderr = $stderr
        }
    } finally {
        foreach ($key in $originalEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($key, $originalEnvironment[$key], 'Process')
        }
    }
}

function Get-FakeExecutable {
    if ($null -ne $Script:FakeExecutable) {
        return $Script:FakeExecutable
    }

    $fakeDirectory = Join-Path ([System.IO.Path]::GetTempPath()) 'extractum-cargo-deny-test-fake'
    New-Item -ItemType Directory -Path $fakeDirectory -Force | Out-Null
    $fakePath = Join-Path $fakeDirectory 'cargo-deny.exe'
    $source = @'
using System;
using System.IO;
public static class Program {
    public static int Main(string[] args) {
        var log = Environment.GetEnvironmentVariable("EXTRACTUM_DENY_LOG");
        if (!String.IsNullOrWhiteSpace(log)) File.AppendAllText(log, String.Join("\u001f", args) + Environment.NewLine);
        if (args.Length == 1 && args[0] == "--version") {
            Console.WriteLine(Environment.GetEnvironmentVariable("EXTRACTUM_DENY_VERSION") ?? "cargo-deny 0.20.2");
            return 0;
        }
        var configured = Environment.GetEnvironmentVariable("EXTRACTUM_DENY_EXIT");
        int exitCode;
        return Int32.TryParse(configured, out exitCode) ? exitCode : 0;
    }
}
'@
    Add-Type -TypeDefinition $source -OutputAssembly $fakePath -OutputType ConsoleApplication
    $Script:FakeExecutable = $fakePath
    return $fakePath
}

function New-Fixture {
    param([string]$Version = '0.20.2')

    $root = Join-Path ([System.IO.Path]::GetTempPath()) ("extractum-cargo-deny-" + [guid]::NewGuid().ToString('N'))
    $scripts = Join-Path $root 'scripts'
    $tools = Join-Path $root '.github/tools'
    $target = Join-Path $root 'src-tauri/target'
    New-Item -ItemType Directory -Path $scripts, $tools, $target -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $root 'deny.toml') -Value '# fixture' -NoNewline
    Set-Content -LiteralPath (Join-Path $root 'src-tauri/Cargo.toml') -Value "[workspace]`nmembers = []" -NoNewline

    $fake = Get-FakeExecutable
    $archiveSource = Join-Path $root 'archive-source'
    New-Item -ItemType Directory -Path $archiveSource -Force | Out-Null
    Copy-Item -LiteralPath $fake -Destination (Join-Path $archiveSource 'cargo-deny.exe')
    $archive = Join-Path $root 'cargo-deny.tar.gz'
    & tar.exe -czf $archive -C $archiveSource cargo-deny.exe
    if ($LASTEXITCODE -ne 0) { throw 'Could not create fake cargo-deny archive.' }

    $manifest = [ordered]@{
        schemaVersion = 1
        version = $Version
        target = 'x86_64-pc-windows-msvc'
        url = 'https://example.invalid/cargo-deny.tar.gz'
        sha256 = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        binarySha256 = (Get-FileHash -LiteralPath $fake -Algorithm SHA256).Hash.ToLowerInvariant()
    } | ConvertTo-Json
    Set-Content -LiteralPath (Join-Path $tools 'cargo-deny.json') -Value $manifest -NoNewline
    $runner = Join-Path $scripts 'cargo-deny.ps1'
    $sourceRunner = Join-Path $Script:RepositoryRoot 'scripts/cargo-deny.ps1'
    if (Test-Path -LiteralPath $sourceRunner -PathType Leaf) {
        Copy-Item -LiteralPath $sourceRunner -Destination $runner
    }

    return @{
        Root = $root
        Runner = $runner
        Archive = $archive
        Log = (Join-Path $root 'argv.log')
        GitHubPath = (Join-Path $root 'github-path.txt')
        Stable = (Join-Path $target (".extractum-tools/cargo-deny/$Version-x86_64-pc-windows-msvc/cargo-deny.exe"))
    }
}

function Get-TestEnvironment {
    param([hashtable]$Fixture)

    return @{
        EXTRACTUM_CARGO_DENY_TESTING = '1'
        EXTRACTUM_CARGO_DENY_TEST_ARCHIVE = $Fixture.Archive
        EXTRACTUM_DENY_LOG = $Fixture.Log
        EXTRACTUM_DENY_EXIT = '0'
        EXTRACTUM_DENY_VERSION = 'cargo-deny 0.20.2'
        GITHUB_PATH = $Fixture.GitHubPath
    }
}

function Invoke-Setup {
    param([hashtable]$Fixture, [hashtable]$Environment)
    return Invoke-Runner -Runner $Fixture.Runner -Arguments @('-Mode', 'Setup', '-AddToGitHubPath') -Environment $Environment
}

function Get-LogLines {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return @() }
    return @(Get-Content -LiteralPath $Path)
}

function Remove-Fixture {
    param([hashtable]$Fixture)
    Remove-Item -LiteralPath $Fixture.Root -Recurse -Force -ErrorAction SilentlyContinue
}

function Test-ColdAndWarmInstall {
    $fixture = New-Fixture
    try {
        $environment = Get-TestEnvironment $fixture
        $cold = Invoke-Setup $fixture $environment
        Assert-Equal 0 $cold.ExitCode 'Cold Setup must succeed.'
        Assert-True (Test-Path -LiteralPath $fixture.Stable -PathType Leaf) 'Cold Setup must publish the stable executable.'
        $stableDigest = (Get-FileHash -LiteralPath $fixture.Stable -Algorithm SHA256).Hash
        Remove-Item -LiteralPath $fixture.Archive -Force
        $warm = Invoke-Setup $fixture $environment
        Assert-Equal 0 $warm.ExitCode 'Warm Setup must not need the archive.'
        Assert-Equal $stableDigest (Get-FileHash -LiteralPath $fixture.Stable -Algorithm SHA256).Hash 'Warm Setup must reuse the stable executable.'
        $pathLines = @(Get-Content -LiteralPath $fixture.GitHubPath)
        Assert-Equal 1 $pathLines.Count 'Setup must publish the stable directory to GITHUB_PATH once.'
        Assert-Equal (Split-Path -Parent $fixture.Stable) $pathLines[0] 'GITHUB_PATH must receive the stable directory.'
    } finally { Remove-Fixture $fixture }
}

function Test-RejectsArchiveDigest {
    $fixture = New-Fixture
    try {
        $manifestPath = Join-Path $fixture.Root '.github/tools/cargo-deny.json'
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $manifest.sha256 = ('0' * 64)
        $manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -NoNewline
        $result = Invoke-Setup $fixture (Get-TestEnvironment $fixture)
        Assert-True ($result.ExitCode -ne 0) 'An archive digest mismatch must fail.'
        Assert-True (-not (Test-Path -LiteralPath $fixture.Stable)) 'A rejected archive must not publish a stable binary.'
        Assert-Equal 0 (@(Get-LogLines $fixture.Log)).Count 'A rejected archive must never execute the fake binary.'
    } finally { Remove-Fixture $fixture }
}

function Test-RejectsBinaryDigest {
    $fixture = New-Fixture
    try {
        $manifestPath = Join-Path $fixture.Root '.github/tools/cargo-deny.json'
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $manifest.binarySha256 = ('0' * 64)
        $manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -NoNewline
        $result = Invoke-Setup $fixture (Get-TestEnvironment $fixture)
        Assert-True ($result.ExitCode -ne 0) 'A binary digest mismatch must fail.'
        Assert-True (-not (Test-Path -LiteralPath $fixture.Stable)) 'A rejected binary must not publish a stable binary.'
        Assert-Equal 0 (@(Get-LogLines $fixture.Log)).Count 'A rejected binary must never execute the fake binary.'
    } finally { Remove-Fixture $fixture }
}

function Test-RejectsVersionMismatch {
    $fixture = New-Fixture
    try {
        $environment = Get-TestEnvironment $fixture
        $environment.EXTRACTUM_DENY_VERSION = 'cargo-deny 0.20.1'
        $result = Invoke-Setup $fixture $environment
        Assert-True ($result.ExitCode -ne 0) 'A version mismatch must fail.'
        Assert-True (-not (Test-Path -LiteralPath $fixture.Stable)) 'A version mismatch must not publish a stable binary.'
        $lines = @(Get-LogLines $fixture.Log)
        Assert-Equal 1 $lines.Count 'A version mismatch must probe exactly once.'
        Assert-Equal '--version' $lines[0] 'The version mismatch probe must use --version.'
    } finally { Remove-Fixture $fixture }
}

function Test-FirstInstallRollback {
    $fixture = New-Fixture
    try {
        $environment = Get-TestEnvironment $fixture
        $environment.EXTRACTUM_CARGO_DENY_TEST_FAIL_PRE_PUBLISH = '1'
        $originalPath = [Environment]::GetEnvironmentVariable('PATH', 'Process')
        $result = Invoke-Setup $fixture $environment
        Assert-True ($result.ExitCode -ne 0) 'The pre-publish fault seam must fail Setup.'
        Assert-True (-not (Test-Path -LiteralPath (Split-Path -Parent $fixture.Stable))) 'The pre-publish fault seam must leave no stable directory.'
        Assert-True (-not (Get-ChildItem -LiteralPath (Join-Path $fixture.Root 'src-tauri/target/.extractum-tools/cargo-deny') -Force -ErrorAction SilentlyContinue | Where-Object Name -match 'backup|stage')) 'The pre-publish fault seam must clean stage and backup directories.'
        Assert-Equal $originalPath ([Environment]::GetEnvironmentVariable('PATH', 'Process')) 'The harness must restore process PATH.'
        Assert-True (-not (Test-Path -LiteralPath $fixture.GitHubPath)) 'The pre-publish fault seam must not append GITHUB_PATH.'
    } finally { Remove-Fixture $fixture }
}

function Test-RollsBackPublishedCandidate {
    $fixture = New-Fixture
    try {
        $environment = Get-TestEnvironment $fixture
        Assert-Equal 0 (Invoke-Setup $fixture $environment).ExitCode 'Initial Setup must succeed before rollback test.'
        $previousDigest = (Get-FileHash -LiteralPath $fixture.Stable -Algorithm SHA256).Hash
        $environment.EXTRACTUM_CARGO_DENY_TEST_FORCE_REINSTALL = '1'
        $environment.EXTRACTUM_CARGO_DENY_TEST_FAIL_POST_PUBLISH = '1'
        $result = Invoke-Setup $fixture $environment
        Assert-True ($result.ExitCode -ne 0) 'The post-publish fault seam must fail Setup.'
        Assert-Equal $previousDigest (Get-FileHash -LiteralPath $fixture.Stable -Algorithm SHA256).Hash 'Rollback must restore the prior stable binary byte-for-byte.'
    } finally { Remove-Fixture $fixture }
}

function Test-RollsBackWhenGitHubPathPublicationFails {
    $fixture = New-Fixture
    try {
        $environment = Get-TestEnvironment $fixture
        $failingSink = Join-Path $fixture.Root 'github-path-directory'
        New-Item -ItemType Directory -Path $failingSink | Out-Null
        $environment.GITHUB_PATH = $failingSink
        $cold = Invoke-Setup $fixture $environment
        Assert-True ($cold.ExitCode -ne 0) 'A real GITHUB_PATH sink failure must fail cold Setup.'
        Assert-True (-not (Test-Path -LiteralPath $fixture.Stable)) 'A failed cold PATH publication must remove the first accepted cache.'
        Assert-True (Test-Path -LiteralPath $failingSink -PathType Container) 'A failed PATH publication must leave the failing sink unchanged.'

        $environment.GITHUB_PATH = $fixture.GitHubPath
        Assert-Equal 0 (Invoke-Setup $fixture $environment).ExitCode 'Setup must establish the prior authenticated cache.'
        $previousDigest = (Get-FileHash -LiteralPath $fixture.Stable -Algorithm SHA256).Hash
        $previousSink = Get-Content -LiteralPath $fixture.GitHubPath -Raw
        $environment.EXTRACTUM_CARGO_DENY_TEST_FORCE_REINSTALL = '1'
        $environment.GITHUB_PATH = $failingSink
        $failedReplacement = Invoke-Setup $fixture $environment
        Assert-True ($failedReplacement.ExitCode -ne 0) 'A real PATH publication failure must fail replacement Setup.'
        Assert-Equal $previousDigest (Get-FileHash -LiteralPath $fixture.Stable -Algorithm SHA256).Hash 'A failed replacement PATH publication must restore the prior cache.'
        Assert-Equal $previousSink (Get-Content -LiteralPath $fixture.GitHubPath -Raw) 'A failed replacement PATH publication must leave the prior sink unchanged.'
    } finally { Remove-Fixture $fixture }
}

function Test-ExcludesAmbientExecutable {
    $fixture = New-Fixture
    try {
        $ambient = Join-Path $fixture.Root 'ambient'
        New-Item -ItemType Directory -Path $ambient -Force | Out-Null
        Copy-Item -LiteralPath (Get-FakeExecutable) -Destination (Join-Path $ambient 'cargo-deny.exe')
        $manifestPath = Join-Path $fixture.Root '.github/tools/cargo-deny.json'
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $manifest.sha256 = ('0' * 64)
        $manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -NoNewline
        $environment = Get-TestEnvironment $fixture
        $originalPath = [Environment]::GetEnvironmentVariable('PATH', 'Process')
        $environment.PATH = "$ambient$([IO.Path]::PathSeparator)$originalPath"
        $result = Invoke-Setup $fixture $environment
        Assert-True ($result.ExitCode -ne 0) 'Invalid archive authentication must fail even when PATH contains cargo-deny.'
        Assert-Equal 0 (@(Get-LogLines $fixture.Log)).Count 'The runner must never execute cargo-deny from PATH.'
        Assert-Equal $originalPath ([Environment]::GetEnvironmentVariable('PATH', 'Process')) 'The harness must restore the parent PATH after a custom child PATH.'
    } finally { Remove-Fixture $fixture }
}

function Test-DispatchAndExitPropagation {
    $fixture = New-Fixture
    try {
        $environment = Get-TestEnvironment $fixture
        Assert-Equal 0 (Invoke-Setup $fixture $environment).ExitCode 'Setup must succeed before dispatch tests.'
        $environment.EXTRACTUM_DENY_EXIT = '23'
        $deterministic = Invoke-Runner -Runner $fixture.Runner -Arguments @('-Mode', 'Deterministic') -Environment $environment
        Assert-Equal 23 $deterministic.ExitCode 'Deterministic mode must propagate cargo-deny exit code.'
        $expectedDeterministic = "--config$([char]0x1f)$($fixture.Root)\deny.toml$([char]0x1f)--manifest-path$([char]0x1f)$($fixture.Root)\src-tauri\Cargo.toml$([char]0x1f)--locked$([char]0x1f)check$([char]0x1f)bans$([char]0x1f)licenses$([char]0x1f)sources"
        Assert-Equal $expectedDeterministic (Get-LogLines $fixture.Log | Select-Object -Last 1) 'Deterministic mode must use absolute root-level locked arguments.'
        $environment.EXTRACTUM_DENY_EXIT = '0'
        $advisory = Invoke-Runner -Runner $fixture.Runner -Arguments @('-Mode', 'Advisories') -Environment $environment
        Assert-Equal 0 $advisory.ExitCode 'Advisory mode must propagate successful cargo-deny exit code.'
        $expectedAdvisory = "--config$([char]0x1f)$($fixture.Root)\deny.toml$([char]0x1f)--manifest-path$([char]0x1f)$($fixture.Root)\src-tauri\Cargo.toml$([char]0x1f)--locked$([char]0x1f)check$([char]0x1f)advisories"
        Assert-Equal $expectedAdvisory (Get-LogLines $fixture.Log | Select-Object -Last 1) 'Advisory mode must use absolute root-level locked arguments.'
        $invalid = Invoke-Runner -Runner $fixture.Runner -Arguments @('-Mode', 'Deterministic', '-AddToGitHubPath') -Environment $environment
        Assert-True ($invalid.ExitCode -ne 0) 'Only Setup may use -AddToGitHubPath.'
    } finally { Remove-Fixture $fixture }
}

$tests = @(
    ${function:Test-ColdAndWarmInstall},
    ${function:Test-RejectsArchiveDigest},
    ${function:Test-RejectsBinaryDigest},
    ${function:Test-RejectsVersionMismatch},
    ${function:Test-FirstInstallRollback},
    ${function:Test-RollsBackPublishedCandidate},
    ${function:Test-RollsBackWhenGitHubPathPublicationFails},
    ${function:Test-ExcludesAmbientExecutable},
    ${function:Test-DispatchAndExitPropagation}
)

$passed = 0
foreach ($test in $tests) {
    & $test
    $passed++
}

Write-Output "PASS: $passed cargo-deny runner behavioral cases"
