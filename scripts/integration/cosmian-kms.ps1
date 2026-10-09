[CmdletBinding()]
param(
    [ValidateSet('up', 'down', 'status', 'logs', 'test')]
    [string]$Action = 'up'
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$composeFile = Join-Path $repositoryRoot 'tests/integration/cosmian/compose.yaml'
$certificateDirectory = Join-Path $repositoryRoot '.local/cosmian-kms/certs'

New-Item -ItemType Directory -Path $certificateDirectory -Force | Out-Null
$env:KMIPKIT_COSMIAN_CERT_DIR = $certificateDirectory

switch ($Action) {
    'up' {
        docker compose --file $composeFile up --detach
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        docker compose --file $composeFile ps
        exit $LASTEXITCODE
    }
    'down' {
        docker compose --file $composeFile down --volumes --remove-orphans
        exit $LASTEXITCODE
    }
    'status' {
        docker compose --file $composeFile ps
        exit $LASTEXITCODE
    }
    'logs' {
        docker compose --file $composeFile logs --follow cosmian-kms
        exit $LASTEXITCODE
    }
    'test' {
        Push-Location $repositoryRoot
        try {
            cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1
            exit $LASTEXITCODE
        }
        finally {
            Pop-Location
        }
    }
}
