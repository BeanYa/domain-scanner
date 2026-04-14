<#
.SYNOPSIS
    Domain Scanner - One-Click Deploy Script for Windows
.DESCRIPTION
    Automated build & packaging script for Tauri 2.0 desktop application.
    Supports GPU feature selection (DirectML/CUDA/CPU) and produces NSIS/MSI installers.
.EXAMPLE
    .\deploy.ps1                    # Interactive mode (prompts for GPU choice)
    .\deploy.ps1 -GpuMode directml # Non-interactive, use DirectML (AMD on Windows)
    .\deploy.ps1 -GpuMode cpu      # Pure CPU mode, no GPU dependencies
    .\deploy.ps1 -SkipDeps         # Skip npm install, build only
    .\deploy.ps1 -Target nsis      # Only build NSIS installer
#>

[CmdletBinding()]
param(
    [ValidateSet("directml", "cuda", "cpu")]
    [string]$GpuMode = "",

    [ValidateSet("all", "nsis", "msi")]
    [string]$Target = "all",

    [switch]$SkipDeps,

    [switch]$PortableOnly
)

$ErrorActionPreference = "Stop"

# ============================================================
# Color Helpers
# ============================================================
function Write-Info  { param([string]$msg); Write-Host "  [INFO] $msg" -ForegroundColor Cyan }
function Write-Ok    { param([string]$msg); Write-Host "  [OK]   $msg" -ForegroundColor Green }
function Write-Warn  { param([string]$msg); Write-Host "  [WARN] $msg" -ForegroundColor Yellow }
function Write-Err   { param([string]$msg); Write-Host "  [ERR]  $msg" -ForegroundColor Red }
function Write-Sep   { Write-Host ("-" * 60) -ForegroundColor DarkGray }

# ============================================================
# Read version from package.json
# ============================================================
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$packageJson = Get-Content "$ScriptDir\package.json" -Raw | ConvertFrom-Json
$AppVersion = $packageJson.version

# ============================================================
# Banner
# ============================================================
Write-Host ""
Write-Host @"
 ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ██╗ █████╗ ████████╗████████╗
██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗  ██║██╔══██╚══██╔══╝╚══██╔══╝
██║     ███████║██████╔╝██║   ██║██╔██╗ ██║███████║   ██║      ██║   
██║     ██╔══██║██╔══██╗██║   ██║██║╚██╗██║██╔══██║   ██║      ██║   
╚██████╗██║  ██║██████╔╝╚██████╔╝██║ ╚████║██║  ██║   ██║      ██║   
 ╚═════╝╚═╝  ╚═╝╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝      ╚═╝   
                                                                         
              One-Click Deploy for Windows  |  v$AppVersion
"@ -ForegroundColor Green
Write-Sep

# ============================================================
# Project Root Detection
# ============================================================
if (-not (Test-Path "$ScriptDir\package.json") -or -not (Test-Path "$ScriptDir\src-tauri\Cargo.toml")) {
    Write-Err "Must run from project root (where package.json and src-tauri/ exist)"
    exit 1
}
Set-Location $ScriptDir
Write-Ok "Project root: $ScriptDir"

# ============================================================
# 1. Environment Check
# ============================================================
Write-Host "`n>>> Step 1/6: Environment Check`n" -ForegroundColor Magenta

$errors = @()

# Node.js
$nodeVersion = ""
try { $nodeVersion = node --version 2>$null } catch {}
if ($nodeVersion -match "^v?(1[89]|[2-9]\d)") {
    Write-Ok "Node.js $nodeVersion"
} else {
    $errors += "Node.js >= 18 required (found: '$nodeVersion'). Install from https://nodejs.org/"
}

# Rust / cargo
$rustVersion = ""
try { $rustVersion = rustc --version 2>$null } catch {}
if ($rustVersion) {
    Write-Ok "Rust $rustVersion"
} else {
    $errors += "Rust not found. Install from https://rustup.rs/"
}

# npm
$npmVersion = ""
try { $npmVersion = npm --version 2>$null } catch {}
if ($npmVersion) {
    Write-Ok "npm $npmVersion"
} else {
    $errors += "npm not found. Install Node.js with npm included."
}

# Visual Studio Build Tools (needed for Rust compilation)
$msbuild = ""
try {
    $vsPaths = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise\VC\Tools\MSVC",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\*\VC\Tools\MSVC"
    )
    foreach ($p in $vsPaths) {
        $found = Resolve-Path -Path "$p\*\bin\Hostx64\x64\cl.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) { $msbuild = $found.Path; break }
    }
} catch {}
if ($msbuild) {
    Write-Ok "Visual Studio Build Tools detected"
} else {
    Write-Warn "Visual Studio Build Tools not confirmed. Rust compilation may fail without C++ build tools."
    Write-Info "Install with: winget install Microsoft.VisualStudio.2022.BuildTools --override '--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'"
}

# Check NSIS/WiX tools for Tauri bundler
$tauriDataDir = "${env:LOCALAPPDATA}\tauri"
if (Test-Path "$tauriDataDir\NSIS") {
    Write-Ok "NSIS tools: found"
} else {
    Write-Info "NSIS tools: not cached (will be auto-downloaded on first build)"
}
if (Test-Path "$tauriDataDir\WiX314") {
    Write-Ok "WiX tools: found"
} else {
    Write-Info "WiX tools: not cached (will be auto-downloaded on first build)"
}

if ($errors.Count -gt 0) {
    Write-Sep
    Write-Err "Environment check failed:"
    $errors | ForEach-Object { Write-Err "  - $_" }
    Write-Host "`nFix the issues above and re-run this script.`n" -ForegroundColor Yellow
    exit 1
}
Write-Ok "Environment check passed"

# ============================================================
# 2. GPU Mode Selection
# ============================================================
Write-Host "`n>>> Step 2/6: GPU Configuration`n" -ForegroundColor Magenta

if (-not $GpuMode) {
    Write-Host "Select GPU acceleration mode:`n" -ForegroundColor White
    Write-Host "  [1] DirectML  - AMD/Intel GPUs on Windows (recommended for AMD 5700XT)"
    Write-Host "  [2] CUDA      - NVIDIA GPUs only"
    Write-Host "  [3] CPU-only  - No GPU dependency, uses remote Embedding API"
    Write-Host ""
    $choice = Read-Host "Enter choice (1-3) [default: 1]"
    switch ($choice) {
        "2" { $GpuMode = "cuda"; break }
        "3" { $GpuMode = "cpu"; break }
        default { $GpuMode = "directml" }
    }
}

$cargoFeatures = ""
$gpuLabel = ""
switch ($GpuMode) {
    "directml" {
        $cargoFeatures = "--features gpu-directml"
        $gpuLabel = "DirectML"
        Write-Ok "GPU mode: DirectML (AMD/Intel Windows)"
    }
    "cuda" {
        $cargoFeatures = "--features gpu-cuda"
        $gpuLabel = "CUDA"
        Write-Ok "GPU mode: CUDA (NVIDIA)"
    }
    "cpu" {
        $cargoFeatures = ""
        $gpuLabel = "CPU"
        Write-Ok "GPU mode: CPU-only (no local GPU inference)"
    }
}

# ============================================================
# 3. Install Dependencies
# ============================================================
Write-Host "`n>>> Step 3/6: Install Dependencies`n" -ForegroundColor Magenta

if ($SkipDeps) {
    Write-Info "Skipping npm install (-SkipDeps flag)"
} else {
    Write-Info "Running npm install..."
    npm install 2>&1 | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
    if ($LASTEXITCODE -ne 0) { Write-Err "npm install failed"; exit 1 }
    Write-Ok "Frontend dependencies installed"
}

# Verify Tauri CLI is installed locally
if (-not (Test-Path "node_modules\.bin\tauri.cmd")) {
    Write-Info "Installing Tauri CLI..."
    npm install 2>&1 | Out-Null
}
Write-Ok "Tauri CLI ready"

# ============================================================
# 4. Pre-Build Validation
# ============================================================
Write-Host "`n>>> Step 4/6: Pre-Build Check`n" -ForegroundColor Magenta

# Validate icons exist
$iconMissing = @()
@("icons\32x32.png", "icons\128x128.png", "icons\128x128@2x.png", "icons\icon.ico") | ForEach-Object {
    if (-not (Test-Path "src-tauri\$_")) { $iconMissing += "src-tauri\$_" }
}
if ($iconMissing.Count -gt 0) {
    Write-Warn "Missing bundle icons:"
    $iconMissing | ForEach-Object { Write-Warn "  - $_" }
    Write-Info "Generating placeholder icons..."
    New-Item -ItemType Directory -Path "src-tauri\icons" -Force | Out-Null
    if (Get-Command npx -ErrorAction SilentlyContinue) {
        Push-Location src-tauri
        try { npx tauri icon --help 2>$null | Out-Null } catch {}
        Pop-Location
    }
}

# Validate NSIS hooks exist
if (-not (Test-Path "src-tauri\nsis-hooks.nsh")) {
    Write-Warn "NSIS hooks file not found: src-tauri/nsis-hooks.nsh"
}

# Quick compile check to catch errors early
Write-Info "Running quick compile check (this may take a moment)..."
Push-Location src-tauri
$checkArgs = @("check")
if ($cargoFeatures) { $checkArgs += $cargoFeatures.Split(" ") }
& cargo @checkArgs 2>&1 | Select-Object -Last 3 | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
$checkCode = $LASTEXITCODE
Pop-Location

if ($checkCode -ne 0) {
    Write-Err "Compile check failed! Fix errors above before building."
    exit 1
}
Write-Ok "Compile check passed"

# ============================================================
# 5. Build Application
# ============================================================
Write-Host "`n>>> Step 5/6: Building Application`n" -ForegroundColor Magenta

$buildArgs = @("tauri", "build")
if ($cargoFeatures) { $buildArgs += $cargoFeatures.Split(" ") }

# Add bundle target
switch ($Target) {
    "nsis" { $buildArgs += @("--bundles", "nsis") }
    "msi"  { $buildArgs += @("--bundles", "msi") }
    # "all" is default, no extra args needed
}

Write-Info "Build command: npm run $(($buildArgs -join ' '))"
Write-Info "This will take several minutes...`n"

$sw = [System.Diagnostics.Stopwatch]::StartNew()
npm run @buildArgs 2>&1 | ForEach-Object {
    $line = "$_".Trim()
    if ($line) {
        if ($line -match "error|Error|FAILED") { Write-Host "  $line" -ForegroundColor Red }
        elseif ($line -match "warning|Warning") { Write-Host "  $line" -ForegroundColor Yellow }
        elseif ($line -match "Compiling|Building|Packaging|Finished|Optimizing") { Write-Host "  $line" -ForegroundColor Green }
        else { Write-Host "  $line" -ForegroundColor Gray }
    }
}
$buildCode = $LASTEXITCODE
$sw.Stop()

if ($buildCode -ne 0) {
    Write-Sep
    Write-Err "Build failed after $($sw.Elapsed.ToString('mm\:ss'))!"
    Write-Info "Check the output above for error details."
    exit 1
}
Write-Ok "Build completed in $($sw.Elapsed.ToString('mm\:ss'))"

# ============================================================
# 6. Collect Artifacts + SHA256 Checksums
# ============================================================
Write-Host "`n>>> Step 6/6: Collect Artifacts`n" -ForegroundColor Magenta

$bundleDir = "src-tauri\target\release\bundle"
$releaseDir = "$ScriptDir\releases"
New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null

$artifacts = @()

# Collect NSIS installer
$nsisExe = Get-ChildItem -Path "$bundleDir\nsis\*-setup.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nsisExe) {
    # Rename with GPU variant suffix
    $newName = $nsisExe.Name -replace '-setup\.exe$', "_${gpuLabel}-setup.exe"
    $destNsis = "$releaseDir\$newName"
    Copy-Item $nsisExe.FullName $destNsis -Force
    $size = [math]::Round($nsisExe.Length / 1MB, 1)
    $artifacts += @{ Name = $newName; Size = $size; Path = $destNsis }
    Write-Ok "NSIS installer: $newName ($size MB)"
} elseif ($Target -eq "nsis" -or $Target -eq "all") {
    Write-Warn "NSIS installer not found in $bundleDir\nsis\"
}

# Collect MSI package
$msiFile = Get-ChildItem -Path "$bundleDir\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($msiFile) {
    $newName = $msiFile.Name -replace '_x64\.msi$', "_x64_${gpuLabel}.msi"
    $destMsi = "$releaseDir\$newName"
    Copy-Item $msiFile.FullName $destMsi -Force
    $size = [math]::Round($msiFile.Length / 1MB, 1)
    $artifacts += @{ Name = $newName; Size = $size; Path = $destMsi }
    Write-Ok "MSI package: $newName ($size MB)"
} elseif ($Target -eq "msi" -or $Target -eq "all") {
    Write-Warn "MSI package not found in $bundleDir\msi\"
}

# Portable exe (optional)
$exeFile = Get-ChildItem -Path "src-tauri\target\release\domain_scanner_app.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($exeFile -and $PortableOnly) {
    $portableDir = "$releaseDir\portable"
    New-Item -ItemType Directory -Path $portableDir -Force | Out-Null
    Copy-Item $exeFile.FullName "$portableDir\Domain Scanner.exe" -Force
    Write-Ok "Portable exe: Domain Scanner.exe ($([math]::Round($exeFile.Length / 1MB)) MB)"
}

# Generate SHA256 checksums
if ($artifacts.Count -gt 0) {
    Write-Info "Generating SHA256 checksums..."
    $checksumFile = "$releaseDir\SHA256SUMS.txt"
    $checksums = @()
    foreach ($a in $artifacts) {
        $hash = (Get-FileHash -Path $a.Path -Algorithm SHA256).Hash.ToLower()
        $checksums += "$hash  $($a.Name)"
        Write-Host "  $hash  $($a.Name)" -ForegroundColor Gray
    }
    Set-Content -Path $checksumFile -Value ($checksums -join "`n") -Encoding UTF8
    Write-Ok "Checksums saved to SHA256SUMS.txt"
}

# Validate artifacts
if ($artifacts.Count -eq 0) {
    Write-Err "No build artifacts found! Check build output for errors."
    exit 1
}

Write-Sep
Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "  DEPLOY COMPLETE!" -ForegroundColor Green -BackgroundColor Black
Write-Host "=========================================`n" -ForegroundColor Green

Write-Host "Artifacts collected in:" -ForegroundColor White
Write-Host "  $releaseDir`n" -ForegroundColor Cyan

foreach ($a in $artifacts) {
    Write-Host "  * $($a.Name)  ($($a.Size) MB)" -ForegroundColor Yellow
}

Write-Host "`nTo install, double-click the .exe (NSIS) or .msi file.`n" -ForegroundColor White

if ($GpuMode -eq "cpu") {
    Write-Warn "Built in CPU mode. GPU acceleration disabled."
    Write-Info "Re-run with '-GpuMode directml' for AMD GPU support."
}

# ============================================================
# Summary
# ============================================================
Write-Sep
Write-Host "Build Summary:`n" -ForegroundColor White
Write-Host "  Product : Domain Scanner v$AppVersion"
Write-Host "  GPU     : $gpuLabel"
Write-Host "  Target  : $Target"
Write-Host "  Time    : $($sw.Elapsed.ToString('mm\:ss'))"
Write-Host "  Output  : $releaseDir`n"
