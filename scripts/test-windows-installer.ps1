# Run only on a disposable Windows CI runner. Never use a personal meeting profile.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'This installation smoke test requires an isolated GitHub runner.' }
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$installer = @(Get-ChildItem 'target/release/bundle/nsis/*-setup.exe')
if ($installer.Count -ne 1) { throw 'Expected one NSIS installer.' }
$directory = Join-Path $env:RUNNER_TEMP 'Meetily Local 설치 검증'
$setup = Start-Process -FilePath $installer[0].FullName -ArgumentList "/S /D=$directory" -PassThru
if (-not $setup.WaitForExit(120000)) { $setup.Kill(); throw 'Installer timed out.' }
if ($setup.ExitCode -ne 0) { throw "Installer failed: $($setup.ExitCode)" }
foreach ($file in @('meetily.exe', 'llama-helper.exe', 'ffmpeg.exe', 'onnxruntime.dll', 'onnxruntime_providers_shared.dll', 'vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll', 'licenses/DEPENDENCY-NOTICES.txt', 'licenses/onnxruntime-LICENSE.txt', 'templates/standard_meeting.json')) {
    if (-not (Test-Path (Join-Path $directory $file))) { throw "Missing installed resource: $file" }
}

# The helper must start without a model and complete its stdin/stdout protocol.
$start = [Diagnostics.ProcessStartInfo]::new((Join-Path $directory 'llama-helper.exe'))
$start.UseShellExecute = $false
$start.RedirectStandardInput = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$helper = [Diagnostics.Process]::Start($start)
$stderr = $helper.StandardError.ReadToEndAsync()
try {
    $helper.StandardInput.WriteLine('{"type":"ping"}')
    $helper.StandardInput.WriteLine('{"type":"shutdown"}')
    $helper.StandardInput.Close()
    $stdout = $helper.StandardOutput.ReadToEndAsync()
    if (-not $helper.WaitForExit(30000)) { throw 'Installed helper timed out.' }
    if ($helper.ExitCode -ne 0) { throw "Installed helper failed: $($stderr.Result)" }
    $responses = @($stdout.Result.Trim() -split '\r?\n' | ForEach-Object { $_ | ConvertFrom-Json })
    if ($responses.Count -ne 2 -or $responses[0].type -ne 'pong' -or $responses[1].type -ne 'goodbye') {
        throw 'Installed helper protocol did not complete.'
    }
} finally { if (-not $helper.HasExited) { $helper.Kill() }; $helper.Dispose() }

# Installed FFmpeg must handle local paths with spaces/non-ASCII and reject network protocols.
$env:MEETILY_SMOKE_DIR = $directory
python -c "import os,wave; from pathlib import Path; p=Path(os.environ['MEETILY_SMOKE_DIR'])/'녹음 검사.wav'; w=wave.open(str(p),'wb'); w.setparams((1,2,16000,0,'NONE','not compressed')); w.writeframes(b'\0\0'*16000); w.close()"
if ($LASTEXITCODE -ne 0) { throw 'Failed to prepare synthetic audio.' }
& "$directory/ffmpeg.exe" -hide_banner -loglevel error -y -i "$directory/녹음 검사.wav" -c:a aac "$directory/변환 검사.m4a"
if ($LASTEXITCODE -ne 0) { throw 'Installed FFmpeg conversion failed.' }
$protocols = & "$directory/ffmpeg.exe" -hide_banner -protocols 2>&1 | Out-String
if ($LASTEXITCODE -ne 0 -or $protocols -match '(?m)^\s*(https?|tcp|tls|udp)\s*$') { throw 'FFmpeg protocol policy failed.' }

$app = Start-Process (Join-Path $directory 'meetily.exe') -PassThru
try {
    Start-Sleep -Seconds 15
    $app.Refresh()
    if ($app.HasExited) { throw "Installed application exited early: $($app.ExitCode)" }
    if ($app.MainWindowHandle -eq 0) { throw 'Installed application did not create a window.' }
    Write-Host "Installed window: $($app.MainWindowTitle)"
} finally { if (-not $app.HasExited) { Stop-Process -Id $app.Id -Force } }
Write-Host 'Installed application, resources, helper IPC and local audio conversion passed.'
