[CmdletBinding()]
param([string] $Distribution)

$ErrorActionPreference = 'Stop'
$modulePath = Join-Path $PSScriptRoot 'KmipKit.WslTest.psm1'
Import-Module -Name $modulePath -Force -ErrorAction Stop

function Stop-WithError {
    param([Parameter(Mandatory)][string] $Message)
    [Console]::Error.WriteLine("KMIPKit WSL test preflight: $Message")
    exit 2
}

if ($PSVersionTable.PSVersion -lt [version]'7.3') {
    Stop-WithError 'PowerShell 7.3 or newer is required. Install or select pwsh 7.3+ and rerun this command.'
}

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$currentDirectory = [System.IO.Path]::GetFullPath((Get-Location).Path)
if (-not [string]::Equals($repositoryRoot.TrimEnd('\'), $currentDirectory.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) {
    Stop-WithError 'Run this command from the KMIPKit repository root.'
}

$wsl = Get-Command -Name 'wsl.exe' -ErrorAction SilentlyContinue
if (-not $wsl) {
    Stop-WithError 'wsl.exe was not found. Install/enable WSL2 and an Ubuntu distribution manually.'
}
$listing = & $wsl.Source --list --verbose 2>&1
if ($LASTEXITCODE -ne 0) {
    Stop-WithError 'WSL could not list installed distributions. Check `wsl --status` and start the Ubuntu distribution manually.'
}
$distributions = @(ConvertFrom-WslListVerbose -Output ([string]::Join([Environment]::NewLine, @($listing))))
try {
    $selected = Select-UbuntuDistribution -Distributions $distributions -Name $Distribution
    Assert-Wsl2Distribution -DistributionName $selected.Name -Version $selected.Version
}
catch {
    Stop-WithError $_.Exception.Message
}

$linuxPathOutput = & $wsl.Source --distribution $selected.Name --exec wslpath -u $repositoryRoot 2>&1
if ($LASTEXITCODE -ne 0) {
    Stop-WithError 'wslpath is unavailable or could not translate the checkout path. Verify the Ubuntu installation and its /mnt drive mounts.'
}
$linuxRepositoryRoot = ([string]::Join([Environment]::NewLine, @($linuxPathOutput))).Trim()
if (-not $linuxRepositoryRoot.StartsWith('/')) {
    Stop-WithError 'wslpath returned an invalid Linux checkout path.'
}

$linuxHomeOutput = & $wsl.Source --distribution $selected.Name --exec printenv HOME 2>&1
if ($LASTEXITCODE -ne 0) {
    Stop-WithError 'Could not determine the WSL user home directory. Verify the selected Ubuntu distribution is usable.'
}
$linuxHome = ([string]::Join([Environment]::NewLine, @($linuxHomeOutput))).Trim()
if (-not $linuxHome.StartsWith('/')) {
    Stop-WithError 'WSL returned an invalid user home directory.'
}
$linuxCargo = $linuxHome.TrimEnd('/') + '/.cargo/bin/cargo'

& $wsl.Source --distribution $selected.Name --cd $linuxRepositoryRoot --exec $linuxCargo +1.94.0 --version *> $null
if ($LASTEXITCODE -ne 0) {
    Stop-WithError "Rust 1.94.0 is not available in '$($selected.Name)'. Install it in that distribution, then rerun this command."
}

$arguments = New-WslCargoArgumentList -Distribution $selected.Name -LinuxRepositoryRoot $linuxRepositoryRoot -CargoPath $linuxCargo
& $wsl.Source @arguments
$cargoExitCode = Get-WslCargoExitCode -NativeExitCode $LASTEXITCODE
exit $cargoExitCode
