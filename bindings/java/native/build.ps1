[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..\..')).Path
$javaExecutable = (Get-Command javac -ErrorAction Stop).Source
$javaHome = Split-Path -Parent (Split-Path -Parent $javaExecutable)
$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
$cargoTargetDirectory = Join-Path $repositoryRoot 'target'
$nativeOutput = Join-Path $PSScriptRoot '..\target\native'
$env:CARGO_TARGET_DIR = $cargoTargetDirectory
$rustLibrary = Join-Path $cargoTargetDirectory 'debug\kmipkit_ffi.lib'
$jniSource = Join-Path $PSScriptRoot 'kmipkit_jni.cpp'
$zeroizingTestSource = Join-Path $PSScriptRoot 'tests\zeroizing_bytes_test.cpp'
$clientBatchItemTransferTestSource = Join-Path $PSScriptRoot 'tests\client_batch_item_transfer_test.cpp'
$responseIdentifierTestSource = Join-Path $PSScriptRoot 'tests\response_identifier_validation_test.cpp'
$responseBridgeTestSource = Join-Path $PSScriptRoot 'tests\response_bridge_test.cpp'
$responseSecretArrayScopeTestSource = Join-Path $PSScriptRoot 'tests\response_secret_array_scope_test.cpp'
$cHeader = Join-Path $repositoryRoot 'bindings\c\include'
$internalHeader = Join-Path $repositoryRoot 'tests\fixtures\api\generated'

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
    throw "Cargo did not produce '$rustLibrary' for the JNI link."
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

$clientBatchItemTransferTestObject = Join-Path $nativeOutput 'client_batch_item_transfer_test.obj'
$clientBatchItemTransferTestExecutable = Join-Path $nativeOutput 'client_batch_item_transfer_test.exe'
$clientBatchItemTransferTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$clientBatchItemTransferTestCompiler += ' /Fo' + '"' + $clientBatchItemTransferTestObject + '"'
$clientBatchItemTransferTestCompiler += ' ' + '"' + $clientBatchItemTransferTestSource + '"'
$clientBatchItemTransferTestCompiler += ' /Fe:' + '"' + $clientBatchItemTransferTestExecutable + '"'
$clientBatchItemTransferTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$clientBatchItemTransferTestCommand += $clientBatchItemTransferTestCompiler
& $env:ComSpec /d /s /c $clientBatchItemTransferTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI client batch item transfer test compile failed with exit code $LASTEXITCODE."
}
& $clientBatchItemTransferTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI client batch item transfer test failed with exit code $LASTEXITCODE."
}

$nativeHandleTransferTestSource = Join-Path $PSScriptRoot 'tests\native_handle_transfer_test.cpp'
$nativeHandleTransferTestObject = Join-Path $nativeOutput 'native_handle_transfer_test.obj'
$nativeHandleTransferTestExecutable = Join-Path $nativeOutput 'native_handle_transfer_test.exe'
$nativeHandleTransferTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$nativeHandleTransferTestCompiler += ' /Fo' + '"' + $nativeHandleTransferTestObject + '"'
$nativeHandleTransferTestCompiler += ' ' + '"' + $nativeHandleTransferTestSource + '"'
$nativeHandleTransferTestCompiler += ' /Fe:' + '"' + $nativeHandleTransferTestExecutable + '"'
$nativeHandleTransferTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$nativeHandleTransferTestCommand += $nativeHandleTransferTestCompiler
& $env:ComSpec /d /s /c $nativeHandleTransferTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI native handle transfer test compile failed with exit code $LASTEXITCODE."
}
& $nativeHandleTransferTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI native handle transfer test failed with exit code $LASTEXITCODE."
}

$responseIdentifierTestObject = Join-Path $nativeOutput 'response_identifier_validation_test.obj'
$responseIdentifierTestExecutable = Join-Path $nativeOutput 'response_identifier_validation_test.exe'
$responseIdentifierTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$responseIdentifierTestCompiler += ' /I' + '"' + $internalHeader + '"'
$responseIdentifierTestCompiler += ' /Fo' + '"' + $responseIdentifierTestObject + '"'
$responseIdentifierTestCompiler += ' ' + '"' + $responseIdentifierTestSource + '"'
$responseIdentifierTestCompiler += ' /Fe:' + '"' + $responseIdentifierTestExecutable + '"'
$responseIdentifierTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$responseIdentifierTestCommand += $responseIdentifierTestCompiler
& $env:ComSpec /d /s /c $responseIdentifierTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI response identifier validation test compile failed with exit code $LASTEXITCODE."
}
& $responseIdentifierTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI response identifier validation test failed with exit code $LASTEXITCODE."
}

$responseBridgeTestObject = Join-Path $nativeOutput 'response_bridge_test.obj'
$responseBridgeTestExecutable = Join-Path $nativeOutput 'response_bridge_test.exe'
$responseBridgeTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$responseBridgeTestCompiler += ' /I' + '"' + $internalHeader + '"'
$responseBridgeTestCompiler += ' /Fo' + '"' + $responseBridgeTestObject + '"'
$responseBridgeTestCompiler += ' ' + '"' + $responseBridgeTestSource + '"'
$responseBridgeTestCompiler += ' /Fe:' + '"' + $responseBridgeTestExecutable + '"'
$responseBridgeTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$responseBridgeTestCommand += $responseBridgeTestCompiler
& $env:ComSpec /d /s /c $responseBridgeTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI response bridge test compile failed with exit code $LASTEXITCODE."
}
& $responseBridgeTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI response bridge test failed with exit code $LASTEXITCODE."
}

$responseSecretArrayScopeTestObject = Join-Path $nativeOutput 'response_secret_array_scope_test.obj'
$responseSecretArrayScopeTestExecutable = Join-Path $nativeOutput 'response_secret_array_scope_test.exe'
$responseSecretArrayScopeTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$responseSecretArrayScopeTestCompiler += ' /Fo' + '"' + $responseSecretArrayScopeTestObject + '"'
$responseSecretArrayScopeTestCompiler += ' ' + '"' + $responseSecretArrayScopeTestSource + '"'
$responseSecretArrayScopeTestCompiler += ' /Fe:' + '"' + $responseSecretArrayScopeTestExecutable + '"'
$responseSecretArrayScopeTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$responseSecretArrayScopeTestCommand += $responseSecretArrayScopeTestCompiler
& $env:ComSpec /d /s /c $responseSecretArrayScopeTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI response secret-array scope test compile failed with exit code $LASTEXITCODE."
}
& $responseSecretArrayScopeTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI response secret-array scope test failed with exit code $LASTEXITCODE."
}

$responseArrayConversionTestSource = Join-Path $PSScriptRoot 'tests\response_array_conversion_test.cpp'
$responseArrayConversionTestObject = Join-Path $nativeOutput 'response_array_conversion_test.obj'
$responseArrayConversionTestExecutable = Join-Path $nativeOutput 'response_array_conversion_test.exe'
$responseArrayConversionTestCompiler = 'cl.exe /nologo /std:c++17 /EHsc /MD'
$responseArrayConversionTestCompiler += ' /Fo' + '"' + $responseArrayConversionTestObject + '"'
$responseArrayConversionTestCompiler += ' ' + '"' + $responseArrayConversionTestSource + '"'
$responseArrayConversionTestCompiler += ' /Fe:' + '"' + $responseArrayConversionTestExecutable + '"'
$responseArrayConversionTestCommand = 'call "' + $devCommand + '" -no_logo -arch=x64 -host_arch=x64 && '
$responseArrayConversionTestCommand += $responseArrayConversionTestCompiler
& $env:ComSpec /d /s /c $responseArrayConversionTestCommand
if ($LASTEXITCODE -ne 0) {
    throw "JNI response array conversion test compile failed with exit code $LASTEXITCODE."
}
& $responseArrayConversionTestExecutable
if ($LASTEXITCODE -ne 0) {
    throw "JNI response array conversion test failed with exit code $LASTEXITCODE."
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
$compiler += ' /I' + '"' + $internalHeader + '"'
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
