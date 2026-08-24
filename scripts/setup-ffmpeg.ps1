# setup-ffmpeg.ps1 — Download & verifikasi FFmpeg/FFprobe build BtbN (lgpl) untuk Windows.
# Replikasi logika CI: .github/workflows/build-verify.yml (job build-windows)
#
# Usage:  powershell -ExecutionPolicy Bypass -File scripts/setup-ffmpeg.ps1
#         (atau:  npm run setup:ffmpeg)

param()

$ErrorActionPreference = "Stop"

$FFMPEG_RELEASE_TAG = "autobuild-2026-08-19-19-21"
$BASE_URL = "https://github.com/BtbN/FFmpeg-Builds/releases/download/$FFMPEG_RELEASE_TAG"
$COMMIT_HASH = "N-126217-ge1e325235e"
$ASSET_NAME = "ffmpeg-$COMMIT_HASH-win64-lgpl.zip"
$FFMPEG_BIN = "ffmpeg-x86_64-pc-windows-msvc.exe"
$FFPROBE_BIN = "ffprobe-x86_64-pc-windows-msvc.exe"

$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$PROJECT_ROOT = Split-Path -Parent $SCRIPT_DIR
$BINARIES_DIR = Join-Path $PROJECT_ROOT "src-tauri\binaries"
$WORKDIR = Join-Path $PROJECT_ROOT ".ffmpeg-setup-tmp"

Write-Host ">> Tag      : $FFMPEG_RELEASE_TAG"
Write-Host ">> Asset    : $ASSET_NAME"
Write-Host ""

if (Test-Path $WORKDIR) { Remove-Item -Recurse -Force $WORKDIR }
New-Item -ItemType Directory -Force -Path $WORKDIR, $BINARIES_DIR | Out-Null
Set-Location $WORKDIR

Write-Host ">> [1/4] Downloading FFmpeg archive..."
$archivePath = Join-Path $WORKDIR $ASSET_NAME
Invoke-WebRequest -Uri "$BASE_URL/$ASSET_NAME" -OutFile $archivePath

Write-Host ">> [2/4] Verifying sha256 against release checksums..."
$checksumPath = Join-Path $WORKDIR "checksums.sha256"
Invoke-WebRequest -Uri "$BASE_URL/checksums.sha256" -OutFile $checksumPath

$expectedLine = Select-String -Path $checksumPath -Pattern ([regex]::Escape($ASSET_NAME))
if (-not $expectedLine) {
    Write-Host "WARNING: Asset tidak ditemukan di checksums.sha256 rilis ini."
    Write-Host "Jangan lanjutkan tanpa verifikasi manual. sha256 hasil download:"
    $hash = (Get-FileHash $archivePath -Algorithm SHA256).Hash
    Write-Host "   $hash"
    Remove-Item -Recurse -Force $WORKDIR
    exit 1
}

$expectedHash = ($expectedLine.Line -split '\s+')[0]
$actualHash = (Get-FileHash $archivePath -Algorithm SHA256).Hash
if ($actualHash -ne $expectedHash) {
    Write-Error "Checksum FFmpeg Windows tidak cocup. Expected: $expectedHash, Actual: $actualHash"
    exit 1
}
Write-Host ">> Checksum OK."

Write-Host ">> [3/4] Extracting..."
$extractPath = Join-Path $WORKDIR "ffmpeg-extracted"
Expand-Archive -Path $archivePath -DestinationPath $extractPath
$root = Get-ChildItem $extractPath | Select-Object -First 1

Write-Host ">> [4/4] Installing binaries..."
Copy-Item "$($root.FullName)\bin\ffmpeg.exe" (Join-Path $BINARIES_DIR $FFMPEG_BIN)
Copy-Item "$($root.FullName)\bin\ffprobe.exe" (Join-Path $BINARIES_DIR $FFPROBE_BIN)

Set-Location $PROJECT_ROOT
Remove-Item -Recurse -Force $WORKDIR

Write-Host ""
Write-Host ">> Selesai! Binaries tersedia di:"
Write-Host "   $BINARIES_DIR\$FFMPEG_BIN"
Write-Host "   $BINARIES_DIR\$FFPROBE_BIN"
Write-Host ">> Jalankan: cargo tauri dev"
