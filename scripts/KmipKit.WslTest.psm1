Set-StrictMode -Version Latest

function ConvertFrom-WslListVerbose {
    [CmdletBinding()]
    param([Parameter(Mandatory)][AllowEmptyString()][string] $Output)

    $distributions = [System.Collections.Generic.List[object]]::new()
    $normalizedOutput = $Output.Replace([string][char]0, '')
    foreach ($line in ($normalizedOutput -split "`r?`n")) {
        if ($line -match '^\s*\*?\s*(?<name>.+?)\s{2,}(?<state>Running|Stopped)\s+(?<version>[12])\s*$') {
            $distributions.Add([pscustomobject]@{
                Name = $Matches.name.Trim()
                State = $Matches.state
                Version = [int]$Matches.version
            })
        }
    }
    return $distributions.ToArray()
}

function Select-UbuntuDistribution {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]] $Distributions,
        [string] $Name
    )

    $ubuntu = @($Distributions | Where-Object { $_.Name -match '^Ubuntu(?:$|[-\s])' })
    if ($Name) {
        if ($Name -notmatch '^Ubuntu(?:$|[-\s])') {
            throw "Distribution '$Name' is not an Ubuntu distribution."
        }
        $match = @($ubuntu | Where-Object { $_.Name -ceq $Name })
        if ($match.Count -ne 1) {
            throw "Ubuntu distribution '$Name' is not installed; choose an exact name from the WSL list."
        }
        return $match[0]
    }

    if ($ubuntu.Count -eq 0) {
        throw 'No installed Ubuntu WSL distribution was found.'
    }
    if ($ubuntu.Count -gt 1) {
        $names = ($ubuntu.Name -join ', ')
        throw "Ubuntu selection is ambiguous ($names); specify -Distribution with an exact name."
    }
    return $ubuntu[0]
}

function Assert-Wsl2Distribution {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string] $DistributionName,
        [Parameter(Mandatory)][int] $Version
    )

    if ($Version -ne 2) {
        throw "Distribution '$DistributionName' must use WSL 2 (reported version $Version)."
    }
}

function New-WslCargoArgumentList {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string] $Distribution,
        [Parameter(Mandatory)][string] $LinuxRepositoryRoot,
        [Parameter(Mandatory)][string] $CargoPath
    )

    if (-not $LinuxRepositoryRoot.StartsWith('/')) {
        throw 'The translated repository path must be an absolute Linux path.'
    }
    if (-not $CargoPath.StartsWith('/')) {
        throw 'The Cargo executable path must be an absolute Linux path.'
    }
    return @(
        '--distribution', $Distribution,
        '--cd', $LinuxRepositoryRoot,
        '--exec', $CargoPath, '+1.94.0', 'test', '--workspace', '--all-features'
    )
}

function Get-WslCargoExitCode {
    [CmdletBinding()]
    param([Parameter(Mandatory)][int] $NativeExitCode)
    return $NativeExitCode
}

Export-ModuleMember -Function ConvertFrom-WslListVerbose, Select-UbuntuDistribution, Assert-Wsl2Distribution, New-WslCargoArgumentList, Get-WslCargoExitCode
