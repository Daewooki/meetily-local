# Run in PowerShell 7 from an x64 Visual Studio developer environment.
param([switch]$SkipTests)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
    throw 'Windows x64 with PowerShell 7 is required.'
}
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$env:NEXT_TELEMETRY_DISABLED = '1'
$env:CARGO_BUILD_JOBS = '4'
$env:CMAKE_BUILD_PARALLEL_LEVEL = '4'
$env:CMAKE_POLICY_VERSION_MINIMUM = '3.5'
$env:CMAKE_PROJECT_INCLUDE = (Join-Path $root '.github/force-portable-ggml.cmake').Replace('\', '/')
$env:RUSTFLAGS = '-C target-cpu=x86-64-v2'
$env:PYTHONUTF8 = '1'
if (-not $env:LIBCLANG_PATH -and (Test-Path 'C:/Program Files/LLVM/bin/libclang.dll')) {
    $env:LIBCLANG_PATH = 'C:/Program Files/LLVM/bin'
}

function Invoke-Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed (exit $LASTEXITCODE)." }
}

if (-not (Test-Path 'frontend/src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe')) {
    throw 'First run scripts/build-ffmpeg-local-windows.sh in MSYS2 MINGW64.'
}
Invoke-Checked cargo @('build', '--locked', '--release', '-p', 'llama-helper')
Copy-Item 'target/release/llama-helper.exe' 'frontend/src-tauri/binaries/llama-helper-x86_64-pc-windows-msvc.exe'

# Use the redistributable CRT from the selected MSVC installation, not DLLs found on PATH.
$crt = Join-Path $env:VCToolsRedistDir 'x64/Microsoft.VC143.CRT'
if (-not (Test-Path "$crt/vcruntime140.dll")) { throw 'Visual Studio 2022 x64 CRT redistributables are required.' }
$crtStage = 'frontend/src-tauri/binaries/msvc'
New-Item -ItemType Directory -Force $crtStage | Out-Null
Copy-Item "$crt/*.dll" $crtStage

Push-Location frontend
try {
    Invoke-Checked pnpm @('install', '--frozen-lockfile')
    # generate_context! embeds the static frontend even in release-mode Rust tests.
    Invoke-Checked pnpm @('run', 'build')
} finally { Pop-Location }
Invoke-Checked python @('scripts/collect-license-notices.py', '--target', 'x86_64-pc-windows-msvc')

if (-not $SkipTests) {
    Invoke-Checked cargo @('test', '--locked', '--release', '-p', 'llama-helper')
    foreach ($filter in @('atlassian::', 'summary::', 'local_build_rejects_external_ai_without_opening_a_socket', 'external_link_tests', 'analytics::', 'onnx_runtime_tests')) {
        Invoke-Checked cargo @('test', '--locked', '--release', '-p', 'meetily', '--lib', $filter)
    }
    Invoke-Checked node @('frontend/tests/lib/editor-security.test.cjs')
    Push-Location frontend
    try { Invoke-Checked npx @('--yes', 'bun@1.3.14', 'test', 'tests') } finally { Pop-Location }
}
# Do not mix installers from a restored cache or a previous version into this build.
if (Test-Path 'target/release/bundle/nsis') {
    Remove-Item 'target/release/bundle/nsis' -Recurse -Force
}
Push-Location frontend
try { Invoke-Checked pnpm @('exec', 'tauri', 'build', '--bundles', 'nsis') } finally { Pop-Location }
Invoke-Checked node @('.github/verify-portable-ggml.cjs', 'target/release')
Write-Host 'Windows installer: target/release/bundle/nsis/'
