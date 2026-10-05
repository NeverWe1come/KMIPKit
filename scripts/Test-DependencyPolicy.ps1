[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$rootManifest = Join-Path $repositoryRoot 'Cargo.toml'
$fuzzManifest = Join-Path $repositoryRoot 'fuzz/Cargo.toml'
$lockfiles = @(
    (Join-Path $repositoryRoot 'Cargo.lock'),
    (Join-Path $repositoryRoot 'fuzz/Cargo.lock')
)
$reviewedHostTriples = @(
    'x86_64-unknown-linux-gnu',
    'aarch64-unknown-linux-gnu',
    'x86_64-pc-windows-msvc',
    'aarch64-apple-darwin'
)
$expectedDenyVersion = '0.20.2'
$rustSecRemote = 'https://github.com/RustSec/advisory-db'
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("KMIPKit-DependencyPolicy-" + [guid]::NewGuid().ToString('N'))
$cargoHome = Join-Path $tempRoot 'cargo-home'
$metadataRoot = Join-Path $tempRoot 'metadata'
$originalCargoHome = $env:CARGO_HOME

function Invoke-CapturedCommand {
    param(
        [Parameter(Mandatory = $true)][string]$Executable,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][string]$Operation,
        [string]$StdoutPath
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Executable
    $startInfo.WorkingDirectory = $repositoryRoot
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $Arguments) {
        [void]$startInfo.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    try {
        if (-not $process.Start()) {
            throw "$Operation could not be started."
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        [void]$stderrTask.GetAwaiter().GetResult()
        if ($StdoutPath) {
            [System.IO.File]::WriteAllText($StdoutPath, $stdout, [System.Text.UTF8Encoding]::new($false))
        }
        if ($process.ExitCode -ne 0) {
            throw "$Operation failed with exit code $($process.ExitCode)."
        }
        return $stdout
    }
    finally {
        $process.Dispose()
    }
}

function Get-LockfileHashes {
    $hashes = @{}
    foreach ($lockfile in $lockfiles) {
        if (-not (Test-Path -LiteralPath $lockfile -PathType Leaf)) {
            throw "Required lockfile is missing: $([System.IO.Path]::GetFileName($lockfile))"
        }
        $hashes[$lockfile] = (Get-FileHash -LiteralPath $lockfile -Algorithm SHA256).Hash
    }
    return $hashes
}

function Assert-LockfilesUnchanged {
    param([hashtable]$Before)

    foreach ($lockfile in $lockfiles) {
        $afterHash = (Get-FileHash -LiteralPath $lockfile -Algorithm SHA256).Hash
        if ($afterHash -ne $Before[$lockfile]) {
            throw 'Cargo.lock SHA256 comparison failed; both lockfiles must remain unchanged.'
        }
    }
}

function Get-RustSecEvidence {
    param([Parameter(Mandatory = $true)][string]$Workspace)

    # cargo-deny 0.20.2 stores sources under advisory-dbs/advisory-db-<stable-url-hash>.
    $databaseRoot = Join-Path $cargoHome 'advisory-dbs'
    if (-not (Test-Path -LiteralPath $databaseRoot -PathType Container)) {
        throw "RustSec Advisory DB is missing after the successful $Workspace workspace check."
    }

    $verifiedDatabases = @()
    foreach ($candidate in Get-ChildItem -LiteralPath $databaseRoot -Directory -Recurse -Force) {
        if (-not (Test-Path -LiteralPath (Join-Path $candidate.FullName '.git'))) {
            continue
        }
        $candidateRemote = (Invoke-CapturedCommand -Executable $gitExecutable -Arguments @('-C', $candidate.FullName, 'remote', 'get-url', 'origin') -Operation 'RustSec remote verification').Trim()
        $normalizedRemote = $candidateRemote.TrimEnd('/')
        if ($normalizedRemote.EndsWith('.git', [System.StringComparison]::OrdinalIgnoreCase)) {
            $normalizedRemote = $normalizedRemote.Substring(0, $normalizedRemote.Length - 4)
        }
        if ([string]::Equals($normalizedRemote, $rustSecRemote, [System.StringComparison]::OrdinalIgnoreCase)) {
            $verifiedDatabases += $candidate.FullName
        }
    }
    if ($verifiedDatabases.Count -ne 1) {
        throw "RustSec Advisory DB remote verification failed; expected one verified checkout after the successful $Workspace workspace check."
    }
    $database = $verifiedDatabases[0]

    $commit = (Invoke-CapturedCommand -Executable $gitExecutable -Arguments @('-C', $database, 'rev-parse', '--verify', 'HEAD') -Operation 'RustSec commit verification').Trim()
    if ($commit -notmatch '^[0-9a-fA-F]{40}$') {
        throw "RustSec Advisory DB commit SHA is missing after the successful $Workspace workspace check."
    }
    $timestamp = (Invoke-CapturedCommand -Executable $gitExecutable -Arguments @('-C', $database, 'show', '-s', '--format=%cI', 'HEAD') -Operation 'RustSec commit timestamp verification').Trim()
    $parsedTimestamp = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse($timestamp, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind, [ref]$parsedTimestamp)) {
        throw "RustSec Advisory DB commit timestamp is missing or not ISO 8601 after the successful $Workspace workspace check."
    }
    return [pscustomobject]@{ Commit = $commit; Timestamp = $timestamp }
}

try {
    foreach ($requiredPath in @($rootManifest, $fuzzManifest) + $lockfiles) {
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Required dependency-policy input is missing: $([System.IO.Path]::GetFileName($requiredPath))"
        }
    }

    $lockHashesBefore = Get-LockfileHashes
    $cargoExecutable = (Get-Command cargo -ErrorAction Stop).Source
    $rustcExecutable = (Get-Command rustc -ErrorAction Stop).Source
    $pythonExecutable = (Get-Command python -ErrorAction Stop).Source
    $gitExecutable = (Get-Command git -ErrorAction Stop).Source

    New-Item -ItemType Directory -Path $cargoHome, $metadataRoot -Force | Out-Null
    $env:CARGO_HOME = $cargoHome

    # Capture rustc -vV and validate its actual host against the reviewed target inventory.
    $rustcOutputPath = Join-Path $tempRoot 'rustc-vv.txt'
    [void](Invoke-CapturedCommand -Executable $rustcExecutable -Arguments @('-vV') -Operation 'rustc -vV capture' -StdoutPath $rustcOutputPath)
    $hostValues = ConvertTo-Json -InputObject @($reviewedHostTriples) -Compress
    $hostValidation = "import sys; from pathlib import Path; from scripts.dependency_policy import parse_rustc_host, validate_host_triple; host = parse_rustc_host(Path(sys.argv[1]).read_text(encoding=`"utf-8`")); validate_host_triple(host, $hostValues); print(host)"
    $observedHost = (Invoke-CapturedCommand -Executable $pythonExecutable -Arguments @('-c', $hostValidation, $rustcOutputPath) -Operation 'reviewed Rust host validation').Trim()
    Write-Output "Reviewed Rust host: $observedHost"

    # Install the exact reviewed cargo-deny release into this run's isolated CARGO_HOME.
    [void](Invoke-CapturedCommand -Executable $cargoExecutable -Arguments @('install', '--locked', '--version', $expectedDenyVersion, 'cargo-deny') -Operation 'cargo-deny installation')
    $denyExecutableName = if ($IsWindows) { 'cargo-deny.exe' } else { 'cargo-deny' }
    $denyExecutable = Join-Path (Join-Path $cargoHome 'bin') $denyExecutableName
    if (-not (Test-Path -LiteralPath $denyExecutable -PathType Leaf)) {
        throw 'The exact cargo-deny installation did not produce its executable.'
    }
    $denyVersion = (Invoke-CapturedCommand -Executable $denyExecutable -Arguments @('--version') -Operation 'cargo-deny version verification').Trim()
    if ($denyVersion -notmatch ('^cargo-deny\s+' + [regex]::Escape($expectedDenyVersion) + '(?:\s|$)')) {
        throw 'Installed cargo-deny version did not match reviewed version 0.20.2.'
    }
    Write-Output "Verified cargo-deny $expectedDenyVersion"

    # Generate unfiltered all-feature metadata for both workspaces before path/exception validation.
    $rootMetadata = Join-Path $metadataRoot 'root.json'
    $fuzzMetadata = Join-Path $metadataRoot 'fuzz.json'
    [void](Invoke-CapturedCommand -Executable $cargoExecutable -Arguments @('metadata', '--manifest-path', $rootManifest, '--locked', '--format-version', '1', '--all-features') -Operation 'root cargo metadata' -StdoutPath $rootMetadata)
    [void](Invoke-CapturedCommand -Executable $cargoExecutable -Arguments @('metadata', '--manifest-path', $fuzzManifest, '--locked', '--format-version', '1', '--all-features') -Operation 'fuzz cargo metadata' -StdoutPath $fuzzMetadata)

    # T007's validator checks canonical workspace paths and the reviewed exception/config correspondence.
    [void](Invoke-CapturedCommand -Executable $pythonExecutable -Arguments @(
        (Join-Path $repositoryRoot 'scripts/dependency_policy.py'),
        '--checkout-root', $repositoryRoot,
        '--root-metadata', $rootMetadata,
        '--fuzz-metadata', $fuzzMetadata
    ) -Operation 'dependency policy metadata and exception validation')
    Write-Output 'Dependency metadata and exception register are valid.'

    foreach ($workspace in @(
        [pscustomobject]@{ Name = 'root'; Manifest = $rootManifest },
        [pscustomobject]@{ Name = 'fuzz'; Manifest = $fuzzManifest }
    )) {
        [void](Invoke-CapturedCommand -Executable $denyExecutable -Arguments @(
            '--manifest-path', $workspace.Manifest,
            '--config', (Join-Path $repositoryRoot '.cargo/deny.toml'),
            '--workspace', '--all-features', '--locked', 'check'
        ) -Operation "$($workspace.Name) cargo-deny workspace check")
        $rustSecEvidence = Get-RustSecEvidence -Workspace $workspace.Name
        Write-Output "RustSec $($workspace.Name): remote $rustSecRemote; commit $($rustSecEvidence.Commit); timestamp $($rustSecEvidence.Timestamp)"
    }

    Assert-LockfilesUnchanged -Before $lockHashesBefore
    Write-Output 'Dependency policy checks passed; Cargo.lock and fuzz/Cargo.lock SHA256 hashes are unchanged.'
}
catch {
    Write-Error $_.Exception.Message
    exit 1
}
finally {
    if ($null -eq $originalCargoHome) {
        Remove-Item Env:CARGO_HOME -ErrorAction SilentlyContinue
    }
    else {
        $env:CARGO_HOME = $originalCargoHome
    }
    if (Test-Path -LiteralPath $tempRoot -PathType Container) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
