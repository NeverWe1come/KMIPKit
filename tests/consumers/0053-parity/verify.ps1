[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$consumerRoot = $PSScriptRoot
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $consumerRoot '..\..\..')).Path

Push-Location $repositoryRoot
try {
    cargo +1.94 run --locked --manifest-path (Join-Path $consumerRoot 'rust\Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw "Rust consumer failed with exit code $LASTEXITCODE." }

    cargo build --locked -p kmipkit-ffi
    if ($LASTEXITCODE -ne 0) { throw "KMIPKit FFI build failed with exit code $LASTEXITCODE." }

    $vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
    $devCommand = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -find 'Common7\Tools\VsDevCmd.bat' | Select-Object -First 1
    if ([string]::IsNullOrWhiteSpace($devCommand)) { throw 'Visual Studio C tools were not found.' }
    $cExecutable = Join-Path $consumerRoot 'c\kmipkit_0053_c_consumer.exe'
    $cCompile = 'cl.exe /nologo /std:c11 /I"' + (Join-Path $repositoryRoot 'bindings\c\include') + '" '
    $cCompile += '"' + (Join-Path $consumerRoot 'c\main.c') + '" '
    $cCompile += '"' + (Join-Path $repositoryRoot 'target\debug\kmipkit_ffi.dll.lib') + '" '
    $cCompile += ' /Fe:"' + $cExecutable + '"'
    $cCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && ' + $cCompile
    & $env:ComSpec /d /s /c $cCommand
    if ($LASTEXITCODE -ne 0) { throw "C consumer build failed with exit code $LASTEXITCODE." }
    $env:PATH = (Join-Path $repositoryRoot 'target\debug') + ';' + $env:PATH
    & $cExecutable
    if ($LASTEXITCODE -ne 0) { throw "C consumer failed with exit code $LASTEXITCODE." }

    & (Join-Path $repositoryRoot 'bindings\java\native\build.ps1')
    if ($LASTEXITCODE -ne 0) { throw "Java native bridge build failed with exit code $LASTEXITCODE." }
    mvn -q -f (Join-Path $repositoryRoot 'bindings\java\pom.xml') `
        '-DskipTests' '-Dexec.skip=true' package
    if ($LASTEXITCODE -ne 0) { throw "Java package build failed with exit code $LASTEXITCODE." }
    mvn -q install:install-file `
        "-Dfile=$(Join-Path $repositoryRoot 'bindings\java\target\kmipkit-java-1.0.0.jar')" `
        "-DpomFile=$(Join-Path $repositoryRoot 'bindings\java\pom.xml')"
    if ($LASTEXITCODE -ne 0) { throw "Java package installation failed with exit code $LASTEXITCODE." }
    mvn -q -f (Join-Path $consumerRoot 'java\pom.xml') `
        "-Dkmipkit.nativeLibrary=$(Join-Path $repositoryRoot 'bindings\java\target\native\kmipkit_jni.dll')" compile exec:java
    if ($LASTEXITCODE -ne 0) { throw "Java consumer failed with exit code $LASTEXITCODE." }

    uv run --locked --python 3.12 --project (Join-Path $consumerRoot 'python') `
        python (Join-Path $consumerRoot 'python\main.py')
    if ($LASTEXITCODE -ne 0) { throw "Python consumer failed with exit code $LASTEXITCODE." }
}
finally {
    Pop-Location
}
