# Wasm Size Checker Script for Soroban Contracts (PowerShell Version)
# This script builds the contract and checks if it's within the 64KB Soroban limit

param(
    [switch]$Verbose
)

Write-Host "🔨 Building Soroban contract for wasm32-unknown-unknown target..." -ForegroundColor Green

# Build the contract
$buildResult = cargo build --release --target wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ ERROR: Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "📏 Checking Wasm file size..." -ForegroundColor Yellow

# Find the built .wasm file
$wasmFile = Get-ChildItem -Path "target\wasm32-unknown-unknown\release" -Filter "*.wasm" | Select-Object -First 1

if (-not $wasmFile) {
    Write-Host "❌ ERROR: No .wasm file found!" -ForegroundColor Red
    exit 1
}

# Get file size in bytes
$fileSize = $wasmFile.Length
$fileSizeKB = [math]::Round($fileSize / 1KB, 2)

Write-Host "📁 Wasm file: $($wasmFile.FullName)" -ForegroundColor Cyan
Write-Host "📊 File size: $fileSize bytes ($fileSizeKB KB)" -ForegroundColor Cyan

# Soroban limit is 64KB (65536 bytes)
$limitBytes = 65536
$limitKB = 64
$remainingBytes = $limitBytes - $fileSize
$remainingKB = [math]::Round($remainingBytes / 1KB, 2)

if ($fileSize -gt $limitBytes) {
    Write-Host "❌ ERROR: Wasm file size ($fileSizeKB KB) exceeds Soroban limit of $limitKB KB!" -ForegroundColor Red
    Write-Host "   Current size: $fileSize bytes" -ForegroundColor Red
    Write-Host "   Limit: $limitBytes bytes" -ForegroundColor Red
    Write-Host "   Over by: $($fileSize - $limitBytes) bytes" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ SUCCESS: Wasm file size ($fileSizeKB KB) is within Soroban limit of $limitKB KB" -ForegroundColor Green
    Write-Host "   Remaining capacity: $remainingBytes bytes ($remainingKB KB)" -ForegroundColor Green
}

# Generate size report
Write-Host ""
Write-Host "## Wasm Size Report" -ForegroundColor Magenta
Write-Host "| Metric | Value |" -ForegroundColor Magenta
Write-Host "|--------|-------|" -ForegroundColor Magenta
Write-Host "| File | ``$($wasmFile.Name)`` |" -ForegroundColor Magenta
Write-Host "| Size | $fileSize bytes ($fileSizeKB KB) |" -ForegroundColor Magenta
Write-Host "| Soroban Limit | $limitBytes bytes ($limitKB KB) |" -ForegroundColor Magenta
Write-Host "| Status | ✅ Within limit |" -ForegroundColor Magenta
Write-Host "| Remaining Capacity | $remainingBytes bytes ($remainingKB KB) |" -ForegroundColor Magenta
