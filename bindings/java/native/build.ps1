[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..\..')).Path
$javaExecutable = (Get-Command javac -ErrorAction Stop).Source
$javaHome = Split-Path -Parent (Split-Path -Parent $javaExecutable)
$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
$nativeOutput = Join-Path $PSScriptRoot '..\target\native'
$rustLibrary = Join-Path $repositoryRoot 'target\debug\kmipkit_ffi.lib'
$jniSource = Join-Path $PSScriptRoot 'kmipkit_jni.cpp'
$zeroizingTestSource = Join-Path $PSScriptRoot 'tests\zeroizing_bytes_test.cpp'
$cHeader = Join-Path $repositoryRoot 'bindings\c\include'

if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
    throw 'Visual Studio C++ build tools are required to build the Windows JNI bridge.'
}

$devCommand = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
    -find 'Common7\Tools\VsDevCmd.bat' | Select-Object -First 1
if ([string]::IsNullOrWhiteSpace($devCommand) -or -not (Test-Path -LiteralPath $devCommand -PathType Leaf)) {
    throw 'Visual Studio VsDevCmd.bat was not found for the installed C++ toolchain.'
}

New-Item -ItemType Directory -Path $nativeOutput -Force | Out-Null
cargo build --locked -p kmipkit-ffi
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE."
}
if (-not (Test-Path -LiteralPath $rustLibrary -PathType Leaf)) {
    throw 'Cargo did not produce target\debug\kmipkit_ffi.lib for the JNI link.'
}

$zeroizingTestObject = Join-Path $nativeOutput 'zeroizing_bytes_test.obj'
$zeroizingTestExecutable = Join-Path $nativeOutput 'zeroizing_bytes_test.exe'
$zeroizingTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$zeroizingTestCompiler += ' /Fo' + '"' + $zeroizingTestObject + '"'
$zeroizingTestCompiler += ' ' + '"' + $zeroizingTestSource + '"'
$zeroizingTestCompiler += ' /Fe:' + '"' + $zeroizingTestExecutable + '"'
$zeroizingTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$zeroizingTestCommand += $zeroizingTestCompiler
& $env:ComSpec /d /s /c $zeroizingTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI scratch zeroization test compile failed with exit code $LASTEXITCODE."
}
& $zeroizingTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI scratch zeroization test failed with exit code $LASTEXITCODE."
}

$jniInclude = Join-Path $javaHome 'include'
$jniPlatformInclude = Join-Path $jniInclude 'win32'
$objectPath = Join-Path $nativeOutput 'kmipkit_jni.obj'
$dllPath = Join-Path $nativeOutput 'kmipkit_jni.dll'
$importPath = Join-Path $nativeOutput 'kmipkit_jni.lib'
$compiler = 'cl.exe /nologo /std:c++17 /EHsc /MD /LD'
$compiler += ' /I' + '"' + $jniInclude + '"'
$compiler += ' /I' + '"' + $jniPlatformInclude + '"'
$compiler += ' /I' + '"' + $cHeader + '"'
$compiler += ' /I' + '"' + $PSScriptRoot + '"'
$compiler += ' /Fo' + '"' + $objectPath + '"'
$compiler += ' ' + '"' + $jniSource + '"'
$compiler += ' ' + '"' + $rustLibrary + '"'
$compiler += ' /link kernel32.lib ntdll.lib userenv.lib ws2_32.lib dbghelp.lib'
$compiler += ' /OUT:' + '"' + $dllPath + '"' + ' /IMPLIB:' + '"' + $importPath + '"'
$commandLine = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && ' + $compiler
& $env:ComSpec /d /s /c $commandLine
if ($LASTEXITCODE -ne 0) {
    throw "MSVC JNI bridge build failed with exit code $LASTEXITCODE."
}

Write-Output "Built $dllPath"
