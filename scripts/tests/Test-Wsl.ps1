$ErrorActionPreference = 'Stop'

$modulePath = Join-Path $PSScriptRoot '..\KmipKit.WslTest.psm1'
$moduleAvailable = Test-Path -LiteralPath $modulePath
if ($moduleAvailable) {
    try {
        Import-Module -Name $modulePath -Force -ErrorAction Stop
    }
    catch {
        $moduleAvailable = $false
        $moduleLoadError = $_.Exception.Message
    }
}

$failures = [System.Collections.Generic.List[string]]::new()

function Assert-True {
    param([bool] $Condition, [string] $Message)
    if (-not $Condition) { throw $Message }
}

function Assert-Equal {
    param($Expected, $Actual, [string] $Message)
    $expectedJson = ConvertTo-Json -InputObject @($Expected) -Compress -Depth 8
    $actualJson = ConvertTo-Json -InputObject @($Actual) -Compress -Depth 8
    if ($expectedJson -cne $actualJson) {
        throw "$Message Expected $expectedJson; got $actualJson."
    }
}

function Assert-Throws {
    param([scriptblock] $Action, [string] $MessagePattern, [string] $Message)
    try { & $Action }
    catch {
        if ($_.Exception.Message -match $MessagePattern) { return }
        throw "$Message Unexpected error: $($_.Exception.Message)"
    }
    throw "$Message Expected an exception."
}

function Invoke-Test {
    param([string] $Name, [scriptblock] $Body)
    try {
        if (-not $moduleAvailable) {
            $loadDetail = if ($moduleLoadError) { " $moduleLoadError" } else { '' }
            throw "Required module is missing or failed to load: $modulePath.$loadDetail"
        }
        & $Body
        Write-Output "PASS $Name"
    }
    catch {
        $failures.Add("FAIL $Name`: $($_.Exception.Message)")
    }
}

$wslListing = @'
  NAME                   STATE           VERSION
* Ubuntu-26.04           Running         2
  Ubuntu-Test            Stopped         2
  Debian                 Stopped         1
'@

Invoke-Test 'parses WSL verbose listing and trims the default marker' {
    $entries = ConvertFrom-WslListVerbose -Output $wslListing
    Assert-Equal @('Ubuntu-26.04', 'Ubuntu-Test', 'Debian') @($entries.Name) 'Distribution names were not parsed.'
    Assert-Equal @(2, 2, 1) @($entries.Version) 'WSL versions were not parsed.'
}

Invoke-Test 'selects the sole installed Ubuntu distribution by default' {
    $entries = ConvertFrom-WslListVerbose -Output $wslListing
    $singleUbuntu = @($entries | Where-Object { $_.Name -eq 'Ubuntu-Test' })
    $selected = Select-UbuntuDistribution -Distributions $singleUbuntu
    Assert-Equal 'Ubuntu-Test' $selected.Name 'Unique default Ubuntu selection failed.'
}

Invoke-Test 'selects an explicitly named Ubuntu distribution' {
    $entries = ConvertFrom-WslListVerbose -Output $wslListing
    $selected = Select-UbuntuDistribution -Distributions $entries -Name 'Ubuntu-Test'
    Assert-Equal 'Ubuntu-Test' $selected.Name 'Explicit Ubuntu selection failed.'
}

Invoke-Test 'reports ambiguous Ubuntu distributions without an explicit name' {
    $entries = ConvertFrom-WslListVerbose -Output $wslListing
    Assert-Throws { Select-UbuntuDistribution -Distributions $entries } 'ambiguous|multiple|specify' 'Ambiguous selection was not rejected.'
}

Invoke-Test 'rejects a selected non-Ubuntu distribution' {
    $entries = ConvertFrom-WslListVerbose -Output $wslListing
    Assert-Throws { Select-UbuntuDistribution -Distributions $entries -Name 'Debian' } 'Ubuntu|distribution' 'Non-Ubuntu selection was not rejected.'
}

Invoke-Test 'rejects WSL 1 for the selected Ubuntu distribution' {
    Assert-Throws { Assert-Wsl2Distribution -DistributionName 'Ubuntu-Test' -Version 1 } 'WSL 2|version 2' 'WSL 1 was not rejected.'
}

Invoke-Test 'builds direct argv with translated checkout as --cd, preserving Unicode and spaces' {
    $arguments = New-WslCargoArgumentList -Distribution 'Ubuntu-26.04' -LinuxRepositoryRoot '/mnt/c/Users/Ada Lovelace/Prueba ñ/KMIPKit'
    Assert-Equal @('--distribution', 'Ubuntu-26.04', '--cd', '/mnt/c/Users/Ada Lovelace/Prueba ñ/KMIPKit', '--exec', 'cargo', '+1.94.0', 'test', '--workspace', '--all-features') $arguments 'WSL argv changed or was split.'
}

Invoke-Test 'preserves the native Cargo process exit status' {
    Assert-Equal 0 (Get-WslCargoExitCode -NativeExitCode 0) 'Success status changed.'
    Assert-Equal 37 (Get-WslCargoExitCode -NativeExitCode 37) 'Failure status changed.'
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ -ErrorAction Continue }
    exit 1
}

Write-Output 'All PowerShell WSL tests passed.'
