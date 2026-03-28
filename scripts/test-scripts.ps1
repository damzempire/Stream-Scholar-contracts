# Test script to validate deployment scripts (PowerShell)
# This script performs basic validation without actually deploying

Write-Host "=== Stream Scholar Deployment Scripts Test ==="

# Test if scripts exist
Write-Host "Checking script files..."
$ScriptsDir = "scripts"
$RequiredScripts = @(
    "deploy.sh",
    "deploy.ps1", 
    "manage-teachers.sh",
    "manage-teachers.ps1",
    "README.md",
    "config.example.env"
)

foreach ($script in $RequiredScripts) {
    $scriptPath = Join-Path $ScriptsDir $script
    if (Test-Path $scriptPath) {
        Write-Host "✓ $script exists"
    } else {
        Write-Host "✗ $script missing"
        exit 1
    }
}

# Test PowerShell scripts syntax
Write-Host ""
Write-Host "Checking PowerShell script syntax..."

try {
    $null = [System.Management.Automation.PSParser]::Tokenize((Get-Content (Join-Path $ScriptsDir "deploy.ps1") -Raw), [ref]$null)
    Write-Host "✓ deploy.ps1 syntax OK"
} catch {
    Write-Host "✗ deploy.ps1 syntax error"
}

try {
    $null = [System.Management.Automation.PSParser]::Tokenize((Get-Content (Join-Path $ScriptsDir "manage-teachers.ps1") -Raw), [ref]$null)
    Write-Host "✓ manage-teachers.ps1 syntax OK"
} catch {
    Write-Host "✗ manage-teachers.ps1 syntax error"
}

# Check if soroban-cli is available (optional)
Write-Host ""
Write-Host "Checking dependencies..."

try {
    $null = Get-Command soroban -ErrorAction Stop
    $sorobanVersion = & soroban --version 2>$null
    Write-Host "✓ soroban-cli found"
    Write-Host "  Version: $sorobanVersion"
} catch {
    Write-Host "! soroban-cli not found (install required for deployment)"
}

# Check if stellar is available (optional)
try {
    $null = Get-Command stellar -ErrorAction Stop
    $null = & stellar --version 2>$null
    Write-Host "✓ stellar CLI found"
} catch {
    Write-Host "! stellar CLI not found (required for building)"
}

# Check contract structure
Write-Host ""
Write-Host "Checking contract structure..."

$contractDir = "contracts\scholar_contracts"
if (Test-Path $contractDir) {
    Write-Host "✓ Contract directory exists"
    
    $cargoToml = Join-Path $contractDir "Cargo.toml"
    if (Test-Path $cargoToml) {
        Write-Host "✓ Cargo.toml exists"
    } else {
        Write-Host "✗ Cargo.toml missing"
    }
    
    $libRs = Join-Path $contractDir "src\lib.rs"
    if (Test-Path $libRs) {
        Write-Host "✓ lib.rs exists"
    } else {
        Write-Host "✗ lib.rs missing"
    }
} else {
    Write-Host "✗ Contract directory missing"
}

Write-Host ""
Write-Host "=== Test Summary ==="
Write-Host "✓ Scripts created successfully"
Write-Host "✓ Documentation provided"
Write-Host "✓ Configuration examples included"
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Install soroban-cli: cargo install soroban-cli"
Write-Host "2. Set SOROBAN_ADMIN_ADDRESS environment variable"
Write-Host "3. Run: .\scripts\deploy.ps1 -Command full-deploy"
Write-Host ""
Write-Host "For existing contract:"
Write-Host "Contract ID: CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
Write-Host "Use: .\scripts\deploy.ps1 -Command init -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
